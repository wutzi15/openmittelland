//! Mittelland AD shell. Usage:
//!   mad-app [--data DIR] [--config CONFIG.DAT] [--saves DIR] [--save-dir DIR] [--music DIR] [--start intro|menu|missions|players] [--mute] [--mode 2|3] [--sample NNv]
//!   mad-app ... --screenshot out.png|out.ppm [--frames N] [--mouse X,Y] [--click X,Y@TICK]... [--right X,Y@TICK]... [--type TEXT@TICK]...
//! `--data` (or `MAD_DATA`) is the original `MITTELL` directory. With `--screenshot`, runs N fixed
//! ticks headless (no window, no sound), then writes the 640x480 framebuffer.
//! Without either, the game files are searched in the usual places or asked for (`setup`).

// the downloaded Windows build starts without a console; errors go to a message box (`setup::fatal`)
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod assets;
mod audio;
mod gfx;
mod input;
mod screens;
mod setup;
mod text;

use assets::Assets;
use audio::Audio;
use gfx::Frame;
use input::{Event, Input, Key};
use screens::{Ctx, Next, Screen};
use std::{num::NonZeroU32, path::PathBuf, rc::Rc, time::{Duration, Instant}};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key as WKey, NamedKey},
    window::{Window, WindowId},
};

/// Fixed logic tick. Films advance by their own per-frame delay, accumulated over ticks.
const TICK_MS: u32 = 10;

struct Args {
    data: PathBuf,
    config: Option<PathBuf>,
    saves: Option<PathBuf>,
    save_dir: Option<PathBuf>,
    music: Option<PathBuf>,
    start: String,
    mute: bool,
    /// `--mode 2|3`: 8-bit or hicolor art (default: CONFIG.DAT byte 0, hicolor when absent)
    mode: Option<u8>,
    sample: Option<String>,
    screenshot: Option<String>,
    frames: u32,
    mouse: (i32, i32),
    clicks: Vec<(u32, Event)>,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        data: std::env::var_os("MAD_DATA").map(PathBuf::from).unwrap_or_default(),
        config: None,
        saves: None,
        save_dir: None,
        music: std::env::var_os("MAD_MUSIC").map(PathBuf::from),
        start: "intro".into(),
        mute: false,
        mode: None,
        sample: None,
        screenshot: None,
        frames: 1,
        mouse: (320, 240),
        clicks: vec![],
    };
    let xy = |s: &str| -> Result<(i32, i32), String> {
        let (x, y) = s.split_once(',').ok_or("expected X,Y")?;
        Ok((x.parse().map_err(|_| "bad X")?, y.parse().map_err(|_| "bad Y")?))
    };
    let mut it = std::env::args().skip(1);
    while let Some(k) = it.next() {
        let mut v = || it.next().ok_or(format!("{k} needs a value"));
        match k.as_str() {
            "--data" => a.data = v()?.into(),
            "--config" => a.config = Some(v()?.into()),
            "--saves" => a.saves = Some(v()?.into()),
            "--save-dir" => a.save_dir = Some(v()?.into()),
            "--music" => a.music = Some(v()?.into()),
            "--start" => a.start = v()?,
            "--mute" => a.mute = true,
            "--mode" => a.mode = Some(v()?.parse().map_err(|_| "bad --mode")?),
            "--sample" => a.sample = Some(v()?),
            "--screenshot" => a.screenshot = Some(v()?),
            "--frames" => a.frames = v()?.parse().map_err(|_| "bad --frames")?,
            "--mouse" => a.mouse = xy(&v()?)?,
            "--click" | "--right" => {
                let s = v()?;
                let (p, t) = s.split_once('@').ok_or("expected X,Y@TICK")?;
                let (x, y) = xy(p)?;
                let e = if k == "--click" { Event::Left(x, y) } else { Event::Right(x, y) };
                a.clicks.push((t.parse().map_err(|_| "bad TICK")?, e));
            }
            "--type" => {
                let s = v()?;
                let (text, t) = s.rsplit_once('@').ok_or("expected TEXT@TICK")?;
                let t: u32 = t.parse().map_err(|_| "bad TICK")?;
                a.clicks.extend(text.chars().filter_map(|c| if c == '\n' { Some((t, Event::Key(Key::Enter))) } else { input::cp437(c).map(|b| (t, Event::Key(Key::Char(b)))) }));
            }
            _ => return Err(format!("unknown argument {k}")),
        }
    }
    if a.data.as_os_str().is_empty() {
        if a.screenshot.is_some() {
            return Err("set --data or MAD_DATA to the original MITTELL directory".into());
        }
        a.data = setup::find().or_else(setup::ask).ok_or("Ohne die Spieldaten kann das Spiel nicht starten. / The game cannot start without its data files.")?;
    }
    // The shipped install keeps the OGG soundtrack three levels above `game/disk/MITTELL`.
    a.music = a.music.or_else(|| Some(a.data.join("../../../soundtrack")).filter(|p| p.is_dir()));
    Ok(a)
}

fn first_screen(start: &str, ctx: &mut Ctx) -> Box<dyn Screen> {
    match start {
        "menu" => screens::menu::main_menu(ctx),
        "missions" => screens::menu::missions(ctx.assets),
        "players" => Box::new(screens::setup::PlayerSetup::new(3, 2)),
        _ => screens::menu::intro(ctx.assets),
    }
}

/// The current screen plus framebuffer, audio and pending input.
struct Game {
    assets: Assets,
    audio: Audio,
    music: Option<PathBuf>,
    screen: Box<dyn Screen>,
    frame: Frame,
    input: Input,
}

impl Game {
    /// Returns false when the game wants to quit.
    fn tick(&mut self) -> bool {
        self.input.tick_repeat(TICK_MS);
        let mut ctx = Ctx { assets: &self.assets, audio: &mut self.audio, music_dir: self.music.clone() };
        let next = self.screen.update(&mut ctx, &self.input, TICK_MS);
        self.input.events.clear();
        match next {
            Next::Stay => true,
            Next::Go(s) => {
                self.screen = s;
                true
            }
            Next::Quit => false,
        }
    }

    fn render(&mut self) {
        let mouse = self.input.mouse;
        if let Some(c) = self.screen.draw(&mut self.frame, &self.assets, mouse) {
            self.frame.blit(&self.assets.icon(c.icon())[0], mouse.0, mouse.1);
        }
    }
}

type Surface = softbuffer::Surface<Rc<Window>, Rc<Window>>;

struct App {
    game: Game,
    window: Option<(Rc<Window>, Surface)>,
    last: Instant,
    acc: Duration,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("Mittelland AD").with_inner_size(winit::dpi::LogicalSize::new(1280, 960));
        let w = Rc::new(el.create_window(attrs).expect("create window"));
        w.set_cursor_visible(false); // the game draws its own pointer
        let ctx = softbuffer::Context::new(w.clone()).expect("softbuffer context");
        let surface = Surface::new(&ctx, w.clone()).expect("softbuffer surface");
        self.window = Some((w, surface));
        self.last = Instant::now();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some((w, surface)) = &mut self.window else { return };
        let input = &mut self.game.input;
        let size = w.inner_size();
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(p) = gfx::to_frame(gfx::fit(size.width as usize, size.height as usize), position.x, position.y) {
                    input.mouse = p;
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                let (x, y) = input.mouse;
                match button {
                    MouseButton::Left => {
                        input.events.push(Event::Left(x, y));
                        input.held = Some(((x, y), 0, 170, 170));
                    }
                    MouseButton::Right => input.events.push(Event::Right(x, y)),
                    _ => {}
                }
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => input.held = None,
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                let key = match &event.logical_key {
                    WKey::Named(NamedKey::Escape) => Some(Key::Escape),
                    WKey::Named(NamedKey::Enter) => Some(Key::Enter),
                    WKey::Named(NamedKey::Backspace) => Some(Key::Backspace),
                    _ => event.text.as_ref().and_then(|t| t.chars().next()).and_then(input::cp437).map(Key::Char),
                };
                input.events.extend(key.map(Event::Key));
            }
            WindowEvent::RedrawRequested => {
                let (Some(dw), Some(dh)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return };
                surface.resize(dw, dh).expect("resize surface");
                self.game.render();
                let mut buf = surface.buffer_mut().expect("surface buffer");
                self.game.frame.present(&mut buf, size.width as usize, size.height as usize);
                buf.present().expect("present");
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        let tick = Duration::from_millis(TICK_MS as u64);
        let now = Instant::now();
        // Cap catch-up after a stall (window drag, breakpoint) at 100 ms.
        self.acc = (self.acc + (now - self.last)).min(tick * 10);
        self.last = now;
        while self.acc >= tick {
            self.acc -= tick;
            if !self.game.tick() {
                return el.exit();
            }
        }
        if let Some((w, _)) = &self.window {
            w.request_redraw();
        }
        el.set_control_flow(ControlFlow::WaitUntil(now + tick - self.acc));
    }
}

fn main() {
    let args = parse_args().unwrap_or_else(|e| setup::fatal(&e, 2));
    let mut assets = Assets::load(args.data.clone(), args.config.clone(), args.saves.clone()).unwrap_or_else(|e| setup::fatal(&format!("loading {}: {e}", args.data.display()), 1));
    if let Some(d) = &args.save_dir {
        assets.user_saves = d.clone();
    }
    if let Some(m) = args.mode
        && let Err(e) = assets.set_hicolor(m == 3)
    {
        setup::fatal(&format!("--mode {m}: {e}"), 1)
    }
    let mut audio = Audio::new(!args.mute && args.screenshot.is_none());
    // the stored master levels apply from the start (driver init 0x12A08, rules-44 §1.2)
    if let Some(c) = assets.config.borrow().as_ref() {
        audio.set_volumes(c.raw[0x195], c.raw[0x194]);
    }
    if let Some(s) = &args.sample {
        let rel = format!("SMP22KHZ/SMPL_{s}.DAT");
        match assets.read(&rel).map_err(|e| e.to_string()).and_then(|d| mad_assets::smpl::Sample::parse(&d).map(|s| s.to_wav(22050)).map_err(|e| e.to_string())) {
            Ok(wav) => audio.play_wav(wav),
            Err(e) => eprintln!("{rel}: {e}"),
        }
    }
    let screen = first_screen(&args.start, &mut Ctx { assets: &assets, audio: &mut audio, music_dir: args.music.clone() });
    let mut game = Game { frame: Frame::new(assets.palette), assets, audio, music: args.music, screen, input: Input::default() };
    game.input.mouse = args.mouse;

    if let Some(path) = &args.screenshot {
        for t in 0..args.frames {
            game.input.events.extend(args.clicks.iter().filter(|c| c.0 == t).map(|c| c.1));
            if !game.tick() {
                break;
            }
        }
        game.render();
        gfx::write_screenshot(path, &game.frame).unwrap_or_else(|e| eprintln!("{path}: {e}"));
        return;
    }

    let el = EventLoop::new().expect("event loop");
    let mut app = App { game, window: None, last: Instant::now(), acc: Duration::ZERO };
    el.run_app(&mut app).expect("event loop");
}
