//! Credits on quitting (0x4D880 / 0x4DA50, docs/re/rules-38 §6): music 0x18, one of three pictures, fifteen lines at
//! y 155, each opened by the middle-band reveal (rules-16b §8 mode 1); the last line plays sample 0xCC and stays
//! 2.5 s. Esc ends it. Then the program ends, as the original's `exit(1)`.

use super::{Ctx, Next, Screen, reveal::Reveal};
use crate::{assets::Assets, gfx::Frame, input::{Cursor, Event, Input, Key}, text};

const PICS: [u16; 3] = [0xA3, 0x102, 0x104];
const LINES: usize = 15;
const LAST_MS: u32 = 2500;

pub struct Credits {
    pic: u16,
    lines: Vec<Vec<u8>>,
    line: usize,
    reveal: Option<Reveal>,
    started: bool,
    ms: u32,
}

impl Credits {
    /// `pic` = rand(3), `idea` = rand(2), drawn from the game RNG by the caller.
    pub fn new(assets: &Assets, pic: u32, idea: u32) -> Self {
        let mut lines = vec![assets.text_id(2242 + idea.min(1))];
        lines.extend((2244..=2257).map(|n| assets.text_id(n)));
        Self { pic: PICS[pic as usize % 3], lines, line: 0, reveal: Some(Reveal::new(155, 155..325, false)), started: false, ms: 0 }
    }

    fn page(&self, assets: &Assets, line: usize) -> Frame {
        let mut f = Frame::new(assets.palette);
        f.blit(&assets.icon(self.pic)[0], 0, 0);
        if let Some(src) = self.lines.get(line) {
            text::draw(&mut f, &assets.font, &text::layout(&assets.font, src, &[], &[], 640, &[]), 0, 155 - 12);
        }
        f
    }
}

impl Screen for Credits {
    fn update(&mut self, ctx: &mut Ctx, input: &Input, dt_ms: u32) -> Next {
        if !self.started {
            self.started = true;
            ctx.audio.play_song(ctx.music_dir.as_deref(), 0x18);
        }
        if input.events.iter().any(|e| matches!(e, Event::Key(Key::Escape))) {
            return Next::Quit;
        }
        if self.line + 1 >= LINES.min(self.lines.len()) {
            // the last line: no reveal, the closing chord, 2.5 s
            if self.ms == 0 && let Some(w) = ctx.assets.sample_wav(0xCC) {
                ctx.audio.play_wav(w);
            }
            self.reveal = None;
            self.ms += dt_ms;
            return if self.ms >= LAST_MS { Next::Quit } else { Next::Stay };
        }
        // one reveal frame on every tick, a DOS clock tick on every odd frame (rules-16b §8, 10 ms here)
        if let Some(r) = &mut self.reveal {
            r.step();
            r.step();
            if r.finished() {
                self.line += 1;
                self.reveal = Some(Reveal::new(155 + self.line as u32, 155..325, false));
            }
        }
        Next::Stay
    }

    fn draw(&self, frame: &mut Frame, assets: &Assets, _: (i32, i32)) -> Option<Cursor> {
        frame.begin(assets);
        frame.blit(&assets.icon(self.pic)[0], 0, 0);
        let full = self.page(assets, self.line);
        match &self.reveal {
            Some(r) => r.draw(frame, &full),
            None => frame.pixels.copy_from_slice(&full.pixels),
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fifteen lines; the screen ends by itself.
    #[test]
    fn real_credits_run() {
        let Ok(root) = std::env::var("MAD_DATA") else { return };
        let assets = Assets::load(root.into(), None, None).unwrap();
        let mut c = Credits::new(&assets, 1, 0);
        assert_eq!(c.lines.len(), LINES);
        let mut audio = crate::audio::Audio::new(false);
        let mut ctx = Ctx { assets: &assets, audio: &mut audio, music_dir: None };
        for _ in 0..60 {
            c.update(&mut ctx, &Input::default(), 10);
        }
        if let Ok(dir) = std::env::var("MAD_SHOT_DIR") {
            let mut f = Frame::new(assets.palette);
            c.draw(&mut f, &assets, (0, 0));
            crate::gfx::write_screenshot(&format!("{dir}/credits.png"), &f).unwrap();
        }
        let mut n = 0;
        while !matches!(c.update(&mut ctx, &Input::default(), 10), Next::Quit) {
            n += 1;
            assert!(n < 100_000);
        }
        assert_eq!(c.line, LINES - 1);
    }
}
