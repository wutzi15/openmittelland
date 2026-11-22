//! Start-up sequence, start menu and the first new-game questions, per `docs/re/formats.md` §7.1.
//! Texts are the EXE's own strings (`docs/re/strings.md`, region `0xBD418-0xBD5B0`).

use super::{Ctx, Next, Screen, Window, backdrop, film::Film, game::Game, placeholder::Placeholder, setup::PlayerSetup};
use crate::{assets::Assets, gfx::Frame, input::{Cursor, Event, Input, Key}};

mod addr {
    pub const TITLE: u32 = 0xBD418;
    pub const START: u32 = 0xBD450;
    pub const NEW_GAME: u32 = 0xBD434;
    pub const LOAD: u32 = 0xBD46C;
    pub const ITEM_LINE: u32 = 0xBD484;
    pub const CONTINUE: u32 = 0xBD48C;
    pub const NEW_TITLE: u32 = 0xBD4A8;
    pub const NO_MISSIONS: u32 = 0xBD4D0;
    pub const HARD: u32 = 0xBD4E0;
    pub const MEDIUM: u32 = 0xBD4E8;
    pub const EASY: u32 = 0xBD4F0;
    pub const MISSIONS: u32 = 0xBD4F8;
    pub const PLAYER_COUNT: u32 = 0xBD560;
}

const MENU_BOX: [i32; 4] = [140, 140, 360, 200];
const MISSIONS_BOX: [i32; 4] = [140, 100, 360, 280];
/// MIDI song played on the start screen; mode 1 keeps it running across the new-game dialogs.
const MENU_SONG: u8 = 4;

type Choose = Box<dyn FnMut(Option<usize>, &mut Ctx) -> Next>;

/// Backdrop with a banner title and one framed window of text with clickable `%m` choices.
/// `choose(None)` on right click / Escape.
pub struct Dialog {
    banner: Vec<u8>,
    rect: [i32; 4],
    text: Vec<u8>,
    items: Vec<Vec<u8>>,
    choose: Choose,
}

impl Dialog {
    pub fn new(banner: Vec<u8>, rect: [i32; 4], text: Vec<u8>, items: Vec<Vec<u8>>, choose: impl FnMut(Option<usize>, &mut Ctx) -> Next + 'static) -> Self {
        Self { banner, rect, text, items, choose: Box::new(choose) }
    }

    fn window(&self, assets: &Assets, lit: &[usize]) -> Window {
        let items: Vec<&[u8]> = self.items.iter().map(Vec::as_slice).collect();
        Window::new(assets, self.rect, &self.text, &items, &[], lit)
    }
}

impl Screen for Dialog {
    fn update(&mut self, ctx: &mut Ctx, input: &Input, _: u32) -> Next {
        for e in &input.events {
            let pick = match *e {
                Event::Left(x, y) => match self.window(ctx.assets, &[]).item_at((x, y)) {
                    Some(k) => Some(k),
                    None => continue,
                },
                Event::Right(..) | Event::Key(Key::Escape) => None,
                _ => continue,
            };
            return (self.choose)(pick, ctx);
        }
        Next::Stay
    }

    fn draw(&self, frame: &mut Frame, assets: &Assets, mouse: (i32, i32)) -> Option<Cursor> {
        backdrop(frame, assets, &self.banner, &[]);
        let hover: Vec<usize> = self.window(assets, &[]).item_at(mouse).into_iter().collect();
        self.window(assets, &hover).draw(frame, assets);
        Some(Cursor::Arrow)
    }
}

/// `INTRO.FLC` (config byte `0x273` = 0, the default) or `INTROB.FLC`, then the start menu.
pub fn intro(assets: &Assets) -> Box<dyn Screen> {
    let name = match assets.config.borrow().as_ref().map(|c| c.raw[0x273]) {
        Some(b) if b != 0 => "INTROB.FLC",
        _ => "INTRO.FLC",
    };
    match Film::intro(assets, name, |ctx| Next::Go(main_menu(ctx))) {
        Some(f) => Box::new(f),
        None => main_menu_screen(assets),
    }
}

/// Which start-menu items exist (§7.1): load if any slot has a save, continue if config `0xF4` < 6.
/// This app's own saves count too: load for any of its slots 0-4, continue for its autosave (5).
fn menu_items(assets: &Assets) -> [bool; 3] {
    let own_autosave = assets.user_saves.join("GAME5.DAT").is_file();
    let cont = own_autosave || assets.config.borrow().as_ref().is_some_and(|c| c.raw[0xF4] < 6);
    [true, (0..5).any(|k| slot_label(assets, k).is_some()), cont]
}

/// Slot for "Das letzte Spiel fortsetzen": this app's autosave, else config `0xF4`.
fn last_slot(assets: &Assets) -> u8 {
    if assets.user_saves.join("GAME5.DAT").is_file() { 5 } else { assets.config.borrow().as_ref().map_or(5, |c| c.raw[0xF4]) }
}

fn main_menu_screen(assets: &Assets) -> Box<dyn Screen> {
    let shown = menu_items(assets);
    if shown == [true, false, false] {
        return missions(assets); // nothing to load: the box is skipped (§7.1 step 4)
    }
    let t = |a| assets.text(a);
    let names = [t(addr::NEW_GAME), t(addr::LOAD), t(addr::CONTINUE)];
    // Unavailable items print `%M$E` in the original; drawn here as nothing.
    let mut text = t(addr::START);
    (1..3).filter(|&k| shown[k]).for_each(|_| text.extend(t(addr::ITEM_LINE)));
    let ids: Vec<usize> = (0..3).filter(|&k| shown[k]).collect();
    let items = ids.iter().map(|&k| names[k].clone()).collect();
    Box::new(Dialog::new(t(addr::TITLE), MENU_BOX, text, items, move |k, ctx| match k.map(|k| ids[k]) {
        Some(0) => Next::Go(missions(ctx.assets)),
        Some(1) => Next::Go(load_dialog(ctx.assets)),
        Some(_) => Next::Go(load_game(ctx.assets, last_slot(ctx.assets), None)),
        None => outro(ctx.assets),
    }))
}

pub fn main_menu(ctx: &mut Ctx) -> Box<dyn Screen> {
    ctx.audio.play_song(ctx.music_dir.as_deref(), MENU_SONG);
    main_menu_screen(ctx.assets)
}

/// Right click in the start menu plays the outro, then quits. Where the original plays the outro is
/// not in §7; this is [UNVERIFIED].
fn outro(assets: &Assets) -> Next {
    Film::new(assets, "OUTRO.FLC", |_| Next::Quit).map_or(Next::Quit, |f| Next::Go(Box::new(f.delay_cs(10))))
}

/// Stored at `0x85CBE` as the menu index: 0 Leicht, 1 Mittel, 2 Schwer, 3 none (rules-10 §1).
pub fn missions(assets: &Assets) -> Box<dyn Screen> {
    let t = |a| assets.text(a);
    let items = vec![t(addr::EASY), t(addr::MEDIUM), t(addr::HARD), t(addr::NO_MISSIONS)];
    Box::new(Dialog::new(t(addr::NEW_TITLE), MISSIONS_BOX, t(addr::MISSIONS), items, |k, ctx| match k {
        Some(k) => Next::Go(player_count(ctx.assets, k as u8)),
        None => Next::Go(main_menu_screen(ctx.assets)),
    }))
}

pub fn player_count(assets: &Assets, difficulty: u8) -> Box<dyn Screen> {
    let items = (1..=6).map(|n: u8| vec![b'0' + n]).collect();
    Box::new(Dialog::new(assets.text(addr::NEW_TITLE), MENU_BOX, assets.text(addr::PLAYER_COUNT), items, move |k, ctx| match k {
        // ours: with the Extras option the game number is asked first
        Some(n) if ctx.assets.prefs.borrow().seed_choice => Next::Go(Box::new(SeedPick { difficulty, count: n + 1, typed: vec![] })),
        Some(n) => Next::Go(Box::new(PlayerSetup::new(difficulty, n + 1, super::setup::clock_seed()))),
        None => Next::Go(missions(ctx.assets)),
    }))
}

/// Largest clock seed, seconds 59 · 256 + hundredths 99 (rules-70 §1).
pub const SEED_MAX: u32 = 59 * 256 + 99;

/// Ours, "Tagesspiel": one game number per calendar day, the days since 1970 spread over the 6000 clock values.
pub fn daily_seed() -> u32 {
    let days = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() / 86400) as u32;
    let v = days % 6000;
    v / 100 * 256 + v % 100
}

fn cp(s: &str) -> Vec<u8> {
    s.chars().filter_map(crate::input::cp437).collect()
}

/// Ours (Extras "Spielnummer wählen"): the game number before the player setup. Random is the original's clock
/// seed, the daily game is `daily_seed`, or a typed number 0..=SEED_MAX replays a game. Empty = random.
struct SeedPick {
    difficulty: u8,
    count: usize,
    typed: Vec<u8>,
}

impl SeedPick {
    fn window(&self, assets: &Assets, lit: &[usize]) -> Window {
        let p = assets.prefs.borrow();
        let text = cp(&format!("$Z$C^ {} _$Z$C%m$N%m$N%m$N", p.tr("Spielnummer", "Game number")));
        let typed = String::from_utf8_lossy(&self.typed);
        let items = [cp(p.tr("Zufall", "Random")), cp(p.tr("Tagesspiel", "Daily game")), cp(&format!("{}: {typed}", p.tr("Nummer", "Number")))];
        Window::new(assets, MENU_BOX, &text, &items.iter().map(Vec::as_slice).collect::<Vec<_>>(), &[], lit)
    }

    fn typed_seed(&self) -> u32 {
        String::from_utf8_lossy(&self.typed).parse().map_or_else(|_| super::setup::clock_seed(), |n: u32| n.min(SEED_MAX))
    }

    fn start(&self, seed: u32) -> Next {
        Next::Go(Box::new(PlayerSetup::new(self.difficulty, self.count, seed)))
    }
}

impl Screen for SeedPick {
    fn update(&mut self, ctx: &mut Ctx, input: &Input, _: u32) -> Next {
        for e in &input.events {
            match *e {
                Event::Left(x, y) => match self.window(ctx.assets, &[]).item_at((x, y)) {
                    Some(0) => return self.start(super::setup::clock_seed()),
                    Some(1) => return self.start(daily_seed()),
                    Some(_) => return self.start(self.typed_seed()),
                    None => {}
                },
                Event::Key(Key::Enter) => return self.start(self.typed_seed()),
                Event::Key(Key::Char(c)) if c.is_ascii_digit() && self.typed.len() < 5 => self.typed.push(c),
                Event::Key(Key::Backspace) => _ = self.typed.pop(),
                Event::Right(..) | Event::Key(Key::Escape) => return Next::Go(player_count(ctx.assets, self.difficulty)),
                _ => {}
            }
        }
        Next::Stay
    }

    fn draw(&self, frame: &mut Frame, assets: &Assets, mouse: (i32, i32)) -> Option<Cursor> {
        backdrop(frame, assets, &assets.text(addr::NEW_TITLE), &[]);
        let hover: Vec<usize> = self.window(assets, &[]).item_at(mouse).into_iter().collect();
        let w = self.window(assets, &hover);
        w.draw(frame, assets);
        // text cursor after the typed number, as in the name entry (rules-42 §3)
        let it = w.item_rect(2);
        crate::text::draw_caret(frame, &assets.font, it.x + it.w, it.y);
        Some(Cursor::Arrow)
    }
}

/// "Anno <year>, <first human>" of one of this app's saves; `None` if it does not parse.
fn save_label(path: &std::path::Path) -> Option<Vec<u8>> {
    let st = std::fs::read(path).ok().and_then(|d| mad_sim::State::from_game_dat(&d, 1).ok())?;
    let who = st.humans().next().map(|p| st.name_str(st.persons[p].name() as usize)).unwrap_or_default();
    Some(cp(&format!("Anno {}, {who}", st.year)))
}

/// Label for save slot k: this app's save ("Anno <year>, <first human>"), else the original's
/// CONFIG name if its `GAMEn.DAT` exists; `None` when the slot is empty.
pub fn slot_label(assets: &Assets, k: u8) -> Option<Vec<u8>> {
    let own = assets.user_saves.join(format!("GAME{k}.DAT"));
    // the name typed when saving (ours: a GAMEn.NAM next to the save; the original keeps it in CONFIG.DAT)
    if own.is_file()
        && let Ok(name) = std::fs::read(assets.user_saves.join(format!("GAME{k}.NAM")))
        && !name.is_empty()
    {
        return Some(name);
    }
    if let Some(l) = save_label(&own) {
        return Some(l);
    }
    let name: Vec<u8> = assets.config.borrow().as_ref()?.raw[k as usize * 0x30 + 1..].iter().take_while(|&&b| b != 0).copied().collect();
    (k < 5 && name != b"^ Leer _" && assets.save_path(k).is_file()).then_some(name)
}

/// Load dialog (`formats.md` §9.4): window (20,140) 600×200, five slots `%d.%m`; empty slots do
/// nothing when clicked.
const LOAD_BOX: [i32; 4] = [20, 140, 600, 200];
/// Ours: the history list, taller for its rows.
const HISTORY_BOX: [i32; 4] = [20, 100, 600, 280];
const HISTORY_ROWS: usize = 8;

fn load_dialog(assets: &Assets) -> Box<dyn Screen> {
    let labels: Vec<Option<Vec<u8>>> = (0..5).map(|k| slot_label(assets, k)).collect();
    let mut text = b"$Z$C^ Spiel Laden _$Z$C".to_vec();
    (1..=5).for_each(|k| text.extend(format!("{k}.%m$N").bytes()));
    let mut usable: Vec<bool> = labels.iter().map(Option::is_some).collect();
    let mut items: Vec<Vec<u8>> = labels.into_iter().map(|l| l.unwrap_or(b"^ Leer _".to_vec())).collect();
    // ours: the rolling history, only with its option on and something in it
    let history = assets.prefs.borrow().autosave_years > 0 && !crate::assets::history_years(&assets.history_dir()).is_empty();
    if history {
        text.extend(b"%m$N");
        items.push(cp(assets.prefs.borrow().tr("Verlauf...", "History...")));
        usable.push(true);
    }
    Box::new(Dialog::new(assets.text(addr::TITLE), LOAD_BOX, text, items, move |k, ctx| match k {
        Some(5) => Next::Go(history_dialog(ctx.assets)),
        Some(k) if usable[k] => Next::Go(load_game(ctx.assets, k as u8, None)),
        Some(_) => Next::Stay,
        None => Next::Go(main_menu_screen(ctx.assets)),
    }))
}

/// Ours: the newest history saves (`HISTORY/<year>.DAT`), newest first.
fn history_dialog(assets: &Assets) -> Box<dyn Screen> {
    let dir = assets.history_dir();
    let years: Vec<u16> = crate::assets::history_years(&dir).into_iter().take(HISTORY_ROWS).collect();
    let mut text = cp(&format!("$Z$C^ {} _$Z$C", assets.prefs.borrow().tr("Verlauf", "History")));
    let items = years.iter().map(|y| {
        text.extend(b"%m$N");
        save_label(&dir.join(format!("{y}.DAT"))).unwrap_or_else(|| cp(&format!("Anno {y}")))
    });
    let items = items.collect();
    Box::new(Dialog::new(assets.text(addr::TITLE), HISTORY_BOX, text, items, move |k, ctx| match k {
        Some(k) => Next::Go(load_path(ctx.assets, &dir.join(format!("{}.DAT", years[k])), None)),
        None => Next::Go(load_dialog(ctx.assets)),
    }))
}

/// Loads a `GAMEn.DAT` (this app's, else the original's) into the sim and opens the Kontor.
pub fn load_game(assets: &Assets, slot: u8, rng: Option<mad_sim::rng::Rng>) -> Box<dyn Screen> {
    load_path(assets, &assets.save_path(slot), rng)
}

/// Loads a save in the GAMEn.DAT format. The save holds no RNG state: a load inside a running game keeps `rng`,
/// one from the menu starts at the session's clock seed.
fn load_path(assets: &Assets, path: &std::path::Path, rng: Option<mad_sim::rng::Rng>) -> Box<dyn Screen> {
    let st = std::fs::read(path).map_err(|e| e.to_string()).and_then(|d| mad_sim::State::from_game_dat(&d, 1));
    match st {
        Ok(mut st) => {
            st.rng = rng.unwrap_or_else(|| mad_sim::rng::Rng::new(super::setup::clock_seed()));
            Box::new(Game::start(assets, st))
        }
        Err(e) => Box::new(Placeholder::new(&format!("{}: {e}", path.display()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds() {
        let d = daily_seed();
        assert!(d <= SEED_MAX && d % 256 < 100);
        let pick = SeedPick { difficulty: 3, count: 1, typed: b"99999".to_vec() };
        assert_eq!(pick.typed_seed(), SEED_MAX);
        assert_eq!(SeedPick { typed: b"42".to_vec(), ..pick }.typed_seed(), 42);
    }

    /// Ours: the history list shows the newest saves first; screenshots of it and the game-number screen with MAD_SHOT_DIR.
    #[test]
    fn real_history_dialog() {
        let Some(assets) = crate::screens::game::tests::test_assets() else { return };
        let st = mad_sim::State::new_game(4, 3, &[mad_sim::world::HumanSetup::test(4)]).unwrap();
        for y in 1601..1612 {
            let mut st = st.clone();
            st.year = y;
            assets.write_history(y, &st.to_game_dat(), 20).unwrap();
        }
        let dlg = history_dialog(&assets);
        let shot = |s: &dyn Screen, name: &str| {
            if let Ok(dir) = std::env::var("MAD_SHOT_DIR") {
                let mut frame = crate::gfx::Frame::new(assets.palette);
                s.draw(&mut frame, &assets, (300, 300));
                crate::gfx::write_screenshot(&format!("{dir}/{name}.png"), &frame).unwrap();
            }
        };
        shot(dlg.as_ref(), "history");
        shot(&SeedPick { difficulty: 3, count: 1, typed: b"12".to_vec() }, "seed");
        let years = crate::assets::history_years(&assets.history_dir());
        assert_eq!(years[0], 1611);
        assert!(save_label(&assets.history_dir().join("1611.DAT")).unwrap().starts_with(b"Anno 1611"));
        std::fs::remove_dir_all(&assets.user_saves).unwrap();
    }
}
