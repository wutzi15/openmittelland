//! FLC cutscenes. Films run in VGA 320×200 mode (`formats.md` §8.1, §8 answer 6): a 320-wide film is
//! shown 1:1 in that mode, which the 640×480 framebuffer reproduces by mapping screen (x, y) to mode
//! (x/2, y·200/480). Escape, Enter or a click skips.

use super::{Ctx, Next, Screen};
use crate::{assets::Assets, gfx::{Frame, H, W}, input::{Cursor, Event, Input, Key}};
use mad_assets::flc::Flc;

type Then = Box<dyn FnOnce(&mut Ctx) -> Next>;

/// Intro sound sync (§8.1): before segment `i` the speech slot `v % 100` from `INTRO_SAMPLES[i]`
/// starts, then the film plays until cumulative frame `INTRO_UNTIL[i]` (0 = to the end).
/// The play mode in the hundreds digit (3 / 1 / 7) is [UNVERIFIED] and ignored.
const INTRO_SAMPLES: [u32; 24] = [4, 3, 3, 3, 5, 12, 111, 9, 6, 7, 0, 8, 8, 8, 8, 0, 0, 102, 110, 210, 1, 3, 211, 202];
const INTRO_UNTIL: [usize; 24] = [72, 88, 109, 135, 173, 1020, 1069, 1092, 1118, 1143, 1150, 1159, 1165, 1171, 1182, 1241, 1242, 1295, 1389, 1408, 1510, 1580, 1581, 0];
/// The intro's `flc_play` delay: 10 hundredths of a second per frame, whatever the FLC header says.
const INTRO_FRAME_MS: u32 = 100;

struct Sync {
    /// Speech WAVs `0000000.WAV`..`0000012.WAV`, preloaded like the original.
    wavs: Vec<Vec<u8>>,
    seg: usize,
    started: bool,
}

pub struct Film {
    flc: Flc<'static>,
    frame_ms: u32,
    acc_ms: u32,
    /// Frames shown so far; FLC files end with a ring frame that loops back, which is not shown.
    shown: usize,
    sync: Option<Sync>,
    then: Option<Then>,
    /// sample played when the film first updates (the avalanche rumble)
    pub cue: Option<u8>,
    /// song (rules-14 §4.1) the film's scene asks for
    pub song: Option<u8>,
    /// speech file started with the film (the outro's slot 13)
    pub voice: Option<String>,
    /// origin in the 320×200 mode; centred otherwise
    at: Option<(usize, usize)>,
    /// ms of the started speech still to run: after a normal end the last frame waits for it (the outro's
    /// sound-idle loop, rules-56 §10); a skip does not wait
    pub hold_ms: u32,
    ended: bool,
}

impl Film {
    /// `None` if the file is missing or not an FLC (the caller then skips the film).
    pub fn new(assets: &Assets, name: &str, then: impl FnOnce(&mut Ctx) -> Next + 'static) -> Option<Self> {
        let data = assets.read(&format!("FLCFILME/{name}")).map_err(|e| eprintln!("{name}: {e}")).ok()?;
        // ponytail: leaks each played film's bytes (a few MB); own the buffer in Flc if films replay often.
        let mut flc = Flc::new(Vec::leak(data)).map_err(|e| eprintln!("{name}: {e}")).ok()?;
        flc.step().ok()?;
        let frame_ms = flc.speed_ms.max(1);
        // rules-56 §10: the player maps the caller's (x, y) to (x/2, table[y]); every film but the informant box is
        // centred that way, INFORMNB (224×116) is called at (96, 100) → (48, 41)
        let at = (name == "INFORMNB.FLC").then_some((48, 41));
        Some(Self { flc, frame_ms, acc_ms: 0, shown: 1, sync: None, then: Some(Box::new(then)), cue: None, song: None, voice: None, at, hold_ms: 0, ended: false })
    }

    /// The intro film with its fixed 10 fps timing and speech cues (§8.1).
    pub fn intro(assets: &Assets, name: &str, then: impl FnOnce(&mut Ctx) -> Next + 'static) -> Option<Self> {
        let mut f = Self::new(assets, name, then)?;
        f.frame_ms = INTRO_FRAME_MS;
        // 44 kHz set when the configured rate (u32 at config 0x187) is at least 44000.
        let rate = assets.config.borrow().as_ref().map_or(0, |c| u32::from_le_bytes(c.raw[0x187..0x18B].try_into().unwrap()));
        let dir = if rate >= 44000 { "SPC44KHZ" } else { "SPC22KHZ" };
        let wavs = (0..13).map(|k| assets.read(&format!("{dir}/{k:07}.WAV")).map_err(|e| eprintln!("{dir}/{k:07}.WAV: {e}")).unwrap_or_default()).collect();
        f.sync = Some(Sync { wavs, seg: 0, started: false });
        Some(f)
    }

    /// The caller's frame delay in 1/100 s, which the player uses instead of the header's (rules-56 §10).
    pub fn delay_cs(mut self, cs: u32) -> Self {
        self.frame_ms = cs * 10;
        self
    }

    /// True once the film has ended (or was skipped); a `Film` embedded in a popup is dropped then.
    pub fn done(&self) -> bool {
        self.then.is_none()
    }

    fn finish(&mut self, ctx: &mut Ctx) -> Next {
        self.then.take().map_or(Next::Quit, |f| f(ctx))
    }

    /// Starts a segment's sample when the shown-frame count enters that segment.
    fn cue(&mut self, ctx: &mut Ctx) {
        let Some(s) = &mut self.sync else { return };
        let seg = intro_segment(self.shown);
        if s.started && seg == s.seg {
            return;
        }
        (s.seg, s.started) = (seg, true);
        let wav = &s.wavs[(INTRO_SAMPLES[seg] % 100) as usize];
        if !wav.is_empty() {
            ctx.audio.play_wav(wav.clone());
        }
    }
}

/// Segment that frame count `shown` belongs to: the first whose end frame is not yet reached.
fn intro_segment(shown: usize) -> usize {
    INTRO_UNTIL.iter().position(|&end| end == 0 || shown < end).unwrap_or(23)
}

impl Screen for Film {
    fn update(&mut self, ctx: &mut Ctx, input: &Input, dt_ms: u32) -> Next {
        let skip = input.events.iter().any(|e| matches!(e, Event::Left(..) | Event::Right(..) | Event::Key(Key::Escape | Key::Enter)));
        if skip {
            return self.finish(ctx);
        }
        self.hold_ms = self.hold_ms.saturating_sub(dt_ms);
        if self.ended {
            return if self.hold_ms == 0 { self.finish(ctx) } else { Next::Stay };
        }
        self.cue(ctx);
        self.acc_ms += dt_ms;
        while self.acc_ms >= self.frame_ms {
            self.acc_ms -= self.frame_ms;
            self.shown += 1;
            if self.shown > self.flc.frames || !matches!(self.flc.step(), Ok(true)) {
                if self.hold_ms > 0 {
                    self.ended = true;
                    return Next::Stay;
                }
                return self.finish(ctx);
            }
            self.cue(ctx);
        }
        Next::Stay
    }

    fn draw(&self, frame: &mut Frame, _: &Assets, _: (i32, i32)) -> Option<Cursor> {
        let (fw, fh) = (self.flc.width, self.flc.height);
        // Centred in the 320×200 mode: INTRO (320×180) lands at (0,10), the original's (0,24) in
        // logical 640×480 coordinates.
        let (x0, y0) = self.at.unwrap_or((320usize.saturating_sub(fw) / 2, 200usize.saturating_sub(fh) / 2));
        frame.palette = self.flc.palette;
        frame.hicolor = false; // films are 8-bit VGA
        for y in 0..H {
            let my = y * 200 / H;
            for x in 0..W {
                let mx = x / 2;
                let inside = (x0..x0 + fw).contains(&mx) && (y0..y0 + fh).contains(&my);
                frame.pixels[y * W + x] = if inside { self.flc.pixels[(my - y0) * fw + mx - x0] as u16 } else { 0 };
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn intro_segments() {
        use super::intro_segment as seg;
        assert_eq!((seg(1), seg(71), seg(72), seg(88)), (0, 0, 1, 2));
        // 1241 ends segment 15 and 1242 segment 16: frame 1242 is already segment 17
        assert_eq!((seg(1240), seg(1241), seg(1242)), (15, 16, 17));
        assert_eq!((seg(1581), seg(1590)), (23, 23));
    }
}
