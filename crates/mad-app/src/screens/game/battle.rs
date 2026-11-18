//! Battle playback: the recorded fight (`mad_sim::fight::Replay`) drawn on the battlefield
//! (`docs/re/formats.md` §10.9, §11.4). Grid cell 24 x 12 px, unit sprites from the `ICONBF1` bank
//! (id 0xF1), team colour through the blitter ramp `5 * colour` (§8.4). A human side sets its
//! tactics in the menu of §11.5 (click a unit = next allowed tactic, right click = fight on, ESC =
//! computer takes over); the menu opens before every round, a deviation from the original's
//! hotkey. Positions of the view window on the 720 x 552 field are [UNVERIFIED]; we show the middle
//! of it.

use crate::{assets::Assets, gfx::Frame, input::Cursor, screens::Window, text};
use mad_sim::{
    Ask,
    battle::{Group, Snap, SnapMan, SnapObj},
    fight::Replay,
};

/// Background icons by battle code (`0x857FE`); code 5 reads past the table in the original.
/// Landscape per battle field (`0x857FE[field]`, formats §11): field = [0, 1, 1, 2, 3, 4][code] (rules-07 §5.1).
pub(super) const BACKGROUND: [u16; 5] = [0x26, 0x2A, 0x29, 0x28, 0x27];
const FIELD_OF_CODE: [usize; 6] = [0, 1, 1, 2, 3, 4];
const UNITS: u16 = 0xF1;
const CELL: (i32, i32) = (24, 12);
/// Field pixel shown at the screen's top-left corner (§11.4: mouse x = (mx + 40) / 24).
const VIEW: (i32, i32) = (40, 40);
const SPRITE_DY: i32 = 28;
/// Trace frames per game tick (one tick is 10 ms; the original frame pace is [UNVERIFIED]).
const TICKS_PER_FRAME: u32 = 4;

// EXE tables (linear addresses, formats §11.4): `i16[22][11]` base, count, x offset; dir4; ramps.
const BASE: u32 = 0x85200;
const XOFF: u32 = 0x853E4;
const COUNT: u32 = 0x855C8;
const DIR4: u32 = 0x857EE;
const RAMPS: u32 = 0x83B79;
/// First sprite of the effect types 12..=19 (single sequences, §11.4).
const OBJ_BASE: [usize; 8] = [1444, 1450, 1458, 1466, 1474, 1482, 1483, 1484];

/// EXE strings (formats §11.5): opening message, "Seid Ihr zufrieden?" with `%m %m`, Ja, Nein.
const INTRO: u32 = 0xB4219;
const SATISFIED: u32 = 0xBB744;
const JA: u32 = 0xC0268;
const NEIN: u32 = 0xC0260;

/// The after-battle report window `win_open(80, 80, 480, 320, 0x8810)` (rules-59 §0).
pub const REPORT_RECT: [i32; 4] = [80, 80, 480, 320];

/// An inline sprite of a report page (`%I`, rules-59 §8): drawn top-left at (pen x, line y − raise).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inline {
    /// a frame of the unit bank in a team colour
    Bank(u16, u8),
    /// the front walking view of a unit type, `BASE[type][3] + 2` (rules-59 §5), in a team colour
    Unit(u8, u8),
    /// a plain icon, as stored
    Icon(u16),
}

/// A report page: text with one width-reserving `%m` item per sprite, and the sprites with their raise.
pub type ReportPage = (Vec<u8>, Vec<(Inline, i32)>);

/// A click page over the field: (box, text, framed window); unframed is a gold caption.
pub type Page = ([i32; 4], Vec<u8>, bool);

/// Message boxes over the field.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Overlay {
    Intro,
    Satisfied,
}

/// What a click on the overlay did.
pub(super) enum Overlaid {
    None,
    /// "Ja": the menu is final
    Submit,
}

/// Open tactic menu of one participant (`Ask::Tactics`); `tactics` is what the player has set.
struct Menu {
    part: u8,
    name: String,
    groups: Vec<Group>,
    tactics: Vec<u8>,
}

pub(super) struct BattleView {
    frames: Vec<Snap>,
    code: u8,
    colours: [u8; 4],
    /// still waiting for sim answers (the fight is not finished)
    pub live: bool,
    menu: Option<Menu>,
    overlay: Option<Overlay>,
    /// config 0x17D bit 0x10: ask "Seid Ihr zufrieden?" before the menu closes
    pub confirm: bool,
    tactic_names: Vec<Vec<u8>>,
    pos: usize,
    ticks: u32,
    base: Vec<i16>,
    xoff: Vec<i16>,
    count: Vec<i16>,
    dir4: [u8; 8],
    ramps: [[u8; 16]; 4],
    /// sample effects of the frames played so far, drained by the game screen (docs/re/rules-14 §2.3)
    sounds: Vec<u8>,
    rng: u32,
    /// the battle song (group 0x1B drawn by the sim)
    pub song: u8,
    /// (answer index of a tactic question, frame count once its frames are in): where a hotkey can rewind to
    marks: Vec<(usize, usize)>,
    /// intro captions and pre-fight window pages (framed, rules-52) still to click away; the field waits under them
    /// (rules-50 §1.2)
    captions: std::collections::VecDeque<Page>,
    /// after-battle report pages, shown over the last frame once the fight has played (rules-59 §0)
    report: std::collections::VecDeque<ReportPage>,
}

fn table(assets: &Assets, addr: u32, n: usize) -> Vec<i16> {
    assets.exe_bytes(addr, n * 2).chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
}

/// Names of troop types 0..11 (rules-07 §1: castle then camp).
const TYPE_NAMES: [&str; 12] = ["Söldner", "Musketiere", "Kanoniere", "Offiziere", "General", "Räuber", "Flintenschützen", "Kanoniere", "Bombenleger", "Räuberhauptmann", "Fuhrleute", "Matrosen"];

impl BattleView {
    fn base(assets: &Assets, code: u8, colours: [u8; 4], frames: Vec<Snap>, live: bool, confirm: bool) -> Self {
        let ramp = |c: u8| -> [u8; 16] { assets.exe_bytes(RAMPS + 16 * 5 * c as u32, 16).try_into().unwrap() };
        // a u16[8] table (rules-51 §3: 0,1,1,1,2,3,3,3); read as bytes every odd facing drew the back view
        let words = assets.exe_bytes(DIR4, 16);
        let dir4: [u8; 8] = std::array::from_fn(|i| words[2 * i]);
        Self {
            frames,
            code,
            colours,
            live,
            menu: None,
            overlay: None,
            confirm,
            tactic_names: assets.name_table((0xB69DC, 11)).iter().map(|n| super::t(n.trim_end_matches(" (%d)"))).collect(),
            pos: 0,
            ticks: 0,
            base: table(assets, BASE, 22 * 11),
            xoff: table(assets, XOFF, 22 * 11),
            count: table(assets, COUNT, 22 * 11),
            dir4,
            ramps: colours.map(ramp),
            sounds: vec![],
            rng: 0x2545_F491,
            marks: vec![],
            song: 0x10,
            captions: Default::default(),
            report: Default::default(),
        }
    }

    /// The finished fight, played from trace frame `from` on.
    pub fn from_replay(assets: &Assets, r: Replay, from: usize) -> Self {
        let mut v = Self::base(assets, r.code, r.colours, r.frames, false, false);
        v.song = r.song;
        v.pos = from;
        v
    }

    /// Intro captions (box, text) to click through before the fight plays on.
    pub fn set_captions(&mut self, c: Vec<([i32; 4], Vec<u8>)>) {
        self.captions = c.into_iter().map(|(r, s)| (r, s, false)).collect();
    }

    /// More pages to click through: (box, text, framed window).
    pub fn add_captions(&mut self, c: Vec<Page>) {
        self.captions.extend(c);
    }

    /// The report pages, after the fight's last frame.
    pub fn add_report(&mut self, pages: Vec<ReportPage>) {
        self.report.extend(pages);
    }

    /// A report page is on screen (the fight has played to its end).
    pub fn report_open(&self) -> bool {
        self.done() && !self.report.is_empty()
    }

    /// The report pages still to read (tests).
    #[cfg(test)]
    pub fn report_pages(&self) -> Vec<Vec<u8>> {
        self.report.iter().map(|p| p.0.clone()).collect()
    }

    /// A click turns the report page.
    pub fn next_report(&mut self) {
        self.report.pop_front();
    }

    /// Nothing left to show: the fight played and the report was read.
    pub fn finished(&self) -> bool {
        !self.live && self.done() && self.report.is_empty() && self.captions.is_empty()
    }

    pub fn caption_open(&self) -> bool {
        !self.captions.is_empty()
    }

    /// A click on a caption closes it; true when it was the last one.
    pub fn next_caption(&mut self) -> bool {
        self.captions.pop_front();
        self.captions.is_empty()
    }

    /// The intro of a castle/camp attack or the first tactic question of a fight (answer index `at`): the view
    /// starts live.
    pub fn from_ask(assets: &Assets, ask: &Ask, at: usize, confirm: bool) -> Self {
        let (Ask::Tactics { code, colours, song, .. } | Ask::BattleIntro { code, colours, song, .. } | Ask::PreFight { code, colours, song, .. }) = ask else { unreachable!() };
        let mut v = Self::base(assets, *code, *colours, vec![], true, confirm);
        v.song = *song;
        v.extend(ask, at);
        v
    }

    /// The frames of tactic question `at`, answered "no change" by the game (rules-44 §2); the first one brings the
    /// opening message.
    pub fn extend(&mut self, ask: &Ask, at: usize) {
        match ask {
            Ask::BattleIntro { frames, .. } => self.frames.extend(frames.iter().cloned()),
            // the pre-fight window shows over the field with the attackers deployed (rules-52 §0)
            Ask::PreFight { frames, .. } if !frames.is_empty() => {
                self.pos = self.frames.len();
                self.frames.extend(frames.iter().cloned());
            }
            Ask::Tactics { frames, .. } => {
                if self.marks.is_empty() {
                    // over the fully deployed field (live capture 2026-11-16)
                    self.overlay = Some(Overlay::Intro);
                    self.pos = self.frames.len();
                }
                self.frames.extend(frames.iter().cloned());
                self.marks.push((at, self.frames.len()));
            }
            _ => {}
        }
    }

    /// The side's hotkey opened the menu at this question; it shows once the frames so far are played.
    pub fn open_menu(&mut self, ask: &Ask, name: String) {
        let Ask::Tactics { part, groups, .. } = ask else { return };
        self.menu = Some(Menu { part: *part, name, groups: groups.clone(), tactics: groups.iter().map(|g| g.tactic).collect() });
    }

    /// The hotkey was pressed: the question that ends the activation on screen. Frames after it are dropped (the
    /// fight is computed again from there); `None` when it is the last question so far.
    pub fn rewind(&mut self) -> Option<usize> {
        let k = self.marks.iter().position(|&(_, len)| len > self.pos).unwrap_or(self.marks.len().saturating_sub(1));
        if k + 1 >= self.marks.len() {
            return None;
        }
        self.frames.truncate(self.marks[k].1);
        self.marks.truncate(k + 1);
        Some(self.marks[k].0)
    }

    /// The fight is over: all of it, continuing where the live view stands.
    pub fn finish(&mut self, r: Replay) {
        self.frames = r.frames;
        self.menu = None;
        self.overlay = None;
        self.live = false;
    }

    /// Centre of a sprite of group `k` (tests).
    #[cfg(test)]
    pub fn group_pos(&self, k: usize) -> (i32, i32) {
        let typ = self.menu.as_ref().unwrap().groups[k].typ;
        let m = self.current().men.iter().filter(|m| !m.fallen && m.typ == typ && m.side == self.menu.as_ref().unwrap().part & 1).max_by_key(|m| m.y).unwrap();
        let [x, y, w, h] = self.sprite_rect(m);
        (x + w / 2, y + h / 2)
    }

    pub fn overlay_open(&self) -> bool {
        self.overlay.is_some() || self.caption_open()
    }

    fn overlay_window(&self, assets: &Assets, lit: &[usize]) -> Option<Window> {
        Some(match self.overlay? {
            // gold caption text on the field, no parchment (live capture 2026-11-16); box (0, 180, 640, 120) per rules-52 §0.4
            Overlay::Intro => super::caption(assets, [0, 180, 640, 120], &assets.text(INTRO)),
            Overlay::Satisfied => {
                let (ja, nein) = (assets.text(JA), assets.text(NEIN));
                Window::new(assets, [160, 180, 320, 120], &assets.text(SATISFIED), &[&ja, &nein], &[], lit)
            }
        })
    }

    /// Right click with the menu open: ask for confirmation if the config says so.
    pub fn finish_menu(&mut self) -> Overlaid {
        if self.confirm {
            self.overlay = Some(Overlay::Satisfied);
            Overlaid::None
        } else {
            Overlaid::Submit
        }
    }

    pub fn click_overlay(&mut self, assets: &Assets, pos: (i32, i32)) -> Overlaid {
        match self.overlay {
            Some(Overlay::Intro) => self.overlay = None,
            Some(Overlay::Satisfied) => match self.overlay_window(assets, &[]).and_then(|w| w.item_at(pos)) {
                Some(0) => {
                    self.overlay = None;
                    return Overlaid::Submit;
                }
                Some(_) => self.overlay = None,
                None => {}
            },
            None => {}
        }
        Overlaid::None
    }

    /// Right click on a message box: closes it ("Nein" for the question).
    pub fn dismiss_overlay(&mut self) {
        self.overlay = None;
    }

    pub fn menu_open(&self) -> bool {
        self.menu.is_some()
    }

    pub fn done(&self) -> bool {
        self.pos + 1 >= self.frames.len()
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
        if self.ticks.is_multiple_of(TICKS_PER_FRAME) && !self.done() && !self.overlay_open() {
            self.pos += 1;
            self.cue();
        }
    }

    /// The effects the original plays for the animation steps of the frame just reached (rules-14 §2.3).
    /// Only the sample variants that exist and are loaded are played (`0x5F` is the only melee one);
    /// the random picks are presentation-only and come from a private generator.
    fn cue(&mut self) {
        let (Some(prev), Some(now)) = (self.pos.checked_sub(1).and_then(|k| self.frames.get(k)), self.frames.get(self.pos)) else { return };
        let mut out = vec![];
        let roll = |n: u32, rng: &mut u32| {
            *rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (*rng >> 16) % n
        };
        let mut rng = self.rng;
        for (m, p) in now.men.iter().zip(&prev.men) {
            if (m.state, m.frame) == (p.state, p.frame) {
                continue;
            }
            match (m.state, m.frame) {
                (3, 2 | 5) if roll(100, &mut rng) < 60 => out.push(0x59 + roll(6, &mut rng) as u8),
                (1, 3) => match m.typ {
                    1 | 6 => out.push(0x46 + roll(3, &mut rng) as u8),
                    10 | 11 => out.push(0x31 + roll(2, &mut rng) as u8),
                    // 0x5F + rand(5): only 0x5F is loaded in the original, the other four rolls are silent
                    _ => out.extend((roll(5, &mut rng) == 0).then_some(0x5F)),
                },
                (2, 3) => {
                    if matches!(m.typ, 1 | 6 | 10 | 11 | 2 | 7) {
                        out.push(0x31 + roll(2, &mut rng) as u8);
                    } else {
                        out.extend((roll(5, &mut rng) == 0).then_some(0x5F));
                    }
                }
                (5, 5) | (6, 6) => out.push(0x58),
                _ => {}
            }
        }
        for fx in &now.fx {
            match fx {
                mad_sim::battle::Fx::Hit => out.push(0x39 + roll(9, &mut rng) as u8),
                mad_sim::battle::Fx::Explosion => {
                    out.push(0x33 + roll(6, &mut rng) as u8);
                    if roll(100, &mut rng) < 90 {
                        out.push(0x33 + roll(6, &mut rng) as u8);
                    }
                }
                mad_sim::battle::Fx::Ambience => out.push(0x4D),
            }
        }
        // a new projectile in flight: the cannon's report
        if now.objs.len() > prev.objs.len() {
            out.push(0x33 + roll(6, &mut rng) as u8);
        }
        self.rng = rng;
        out.truncate(6);
        self.sounds.extend(out);
    }

    /// Effects queued since the last call.
    pub fn take_sounds(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.sounds)
    }

    /// A fight played through frame by frame queues some effects (needs MAD_DATA for the battle).
    #[cfg(test)]
    pub fn all_sounds(&mut self) -> Vec<u8> {
        let mut out = vec![];
        while !self.done() {
            for _ in 0..TICKS_PER_FRAME {
                self.tick();
            }
            out.extend(self.take_sounds());
        }
        out
    }

    /// Open captions and the frame on screen (tests).
    #[cfg(test)]
    pub fn captions(&self) -> (Vec<Vec<u8>>, usize) {
        (self.captions.iter().map(|c| c.1.clone()).collect(), self.pos)
    }

    #[cfg(test)]
    pub fn frames(&self) -> usize {
        self.frames.len()
    }

    #[cfg(test)]
    pub fn seek(&mut self, k: usize) {
        self.pos = k.min(self.frames.len().saturating_sub(1));
    }

    pub fn skip(&mut self) {
        self.pos = self.frames.len().saturating_sub(1);
    }

    /// A unit frame in its participant's colour `0xDB7FC[+0x10]` (rules-51 §1.4).
    fn sprite(&self, assets: &Assets, frame: &mut Frame, i: usize, (x, y): (i32, i32), part: u8) {
        self.sprite_c(assets, frame, i, (x, y), self.colours[part as usize & 3]);
    }

    /// A unit-bank frame blitted with colour flag 0x80 + 5c: every byte p maps through ramp entry 80c + p (rules-51 §1).
    fn sprite_c(&self, assets: &Assets, frame: &mut Frame, i: usize, (x, y): (i32, i32), c: u8) {
        let bank = assets.icon8(UNITS);
        let Some(img) = bank.get(i) else { return };
        let img = assets.remap(img, 5 * c);
        frame.blit(&img, x - VIEW.0, y - VIEW.1);
    }

    fn draw_man(&self, assets: &Assets, frame: &mut Frame, m: &SnapMan) {
        let t = m.typ as usize;
        let k = t * 11 + m.state as usize;
        let (Some(&base), Some(&n)) = (self.base.get(k), self.count.get(k)) else { return };
        if base < 0 || n <= 0 {
            return;
        }
        let f = (m.frame as i32).min(n as i32 - 1);
        let i = base as usize + self.dir4[m.facing as usize & 7] as usize + 4 * f as usize;
        let (cx, cy) = (m.x as i32 * CELL.0, m.y as i32 * CELL.1);
        let (tx, ty) = (m.tx as i32 * CELL.0, m.ty as i32 * CELL.1);
        let (px, py) = (cx + (tx - cx) * m.progress as i32 / 255, cy + (ty - cy) * m.progress as i32 / 255);
        self.sprite(assets, frame, i, (px - self.xoff[k] as i32, py - SPRITE_DY), m.part);
    }

    /// A prisoner (type 12 made from a captured man) is drawn as a banner in the captor's colour, at his cell
    /// shifted right by row mod 4 (rules-43 §1).
    fn draw_prisoner(&self, assets: &Assets, frame: &mut Frame, m: &SnapMan) {
        let i = OBJ_BASE[0] + (m.frame as usize).min(5);
        let (px, py) = (m.x as i32 * CELL.0 + (m.y as i32).rem_euclid(4) - 8, m.y as i32 * CELL.1);
        self.sprite(assets, frame, i, (px, py - SPRITE_DY), m.part);
    }

    /// Goods markers (type 20) and wagons (21) are plain icons, as stored (rules-51 §1.1), at x = col·24 + 1 / − 9
    /// (§2.1). They stand on their cell's bottom edge, row·12 + 12 (live A/B of a raid, 2026-11-17: exact for the
    /// 24 px crates and the 69 px wagons; rules-51's y = row·12 − 28 is the men's). `ICONR8`/`ICONR7` are their
    /// wrecked pictures, used for a fallen one [UNVERIFIED].
    fn draw_cargo(&self, assets: &Assets, frame: &mut Frame, m: &SnapMan) {
        let (id, dx) = match (m.typ, m.fallen) {
            (20, false) => (0x2C, 1),
            (20, true) => (0x1B8, 1),
            (_, false) => (0x2B, -9),
            (_, true) => (0x1B7, -9),
        };
        let img = &assets.icon(id)[0];
        let (x, y) = (m.x as i32 * CELL.0 + dx, (m.y as i32 + 1) * CELL.1 - img.height as i32);
        frame.blit(img, x - VIEW.0, y - VIEW.1);
    }

    fn draw_obj(&self, assets: &Assets, frame: &mut Frame, o: &SnapObj) {
        let Some(&base) = OBJ_BASE.get((o.typ as usize).wrapping_sub(12)) else { return };
        let len = OBJ_BASE.get(o.typ as usize - 11).map_or(1, |n| n - base).max(1);
        // objects: BASE + D + anim without the ×4 of men; D is the raw facing for a cannon (type 16), DIR4[facing]
        // otherwise (rules-51 §3)
        let d = if o.typ == 16 { o.facing as usize & 7 } else { self.dir4[o.facing as usize & 7] as usize };
        let i = base + if o.typ <= 16 { (d + o.frame as usize) % len } else { 0 };
        let (cx, cy) = (o.x as i32 * CELL.0, o.y as i32 * CELL.1);
        let (tx, ty) = (o.tx as i32 * CELL.0, o.ty as i32 * CELL.1);
        let p = o.progress as i32;
        let (mut px, mut py) = (cx + (tx - cx) * p / 255, cy + (ty - cy) * p / 255);
        if o.typ == 17 {
            // projectile parabola (§11.4): y offset = -(d - t) t 160 / d^2
            py -= (255 - p) * p * 160 / (255 * 255);
        }
        // the type's x offset from the same XOFF table as the men (action 0; a cannon's is 8, rules-51 §3)
        px -= self.xoff.get(o.typ as usize * 11).map_or(4, |&v| v as i32);
        // projectiles and explosions pass colour 0 (flag 0x80), cannons their owner's colour (rules-51 §1)
        let c = if o.typ == 16 { self.colours[o.part as usize & 3] } else { 0 };
        self.sprite_c(assets, frame, i, (px, py - SPRITE_DY), c);
    }

    pub fn draw(&self, frame: &mut Frame, assets: &Assets, mouse: (i32, i32)) -> Option<Cursor> {
        frame.begin(assets);
        // indexed by field, not by code (live A/B: a castle attack, code 4, is field 3, the castle wall)
        frame.blit(&assets.icon(BACKGROUND[FIELD_OF_CODE[self.code as usize % 6]])[0], 0, 0);
        let Snap { men, objs, .. } = &self.frames[self.pos.min(self.frames.len() - 1)];
        // painter's order: rows top to bottom, corpses under the living
        let mut order: Vec<(bool, i32, i32, Result<&SnapMan, &SnapObj>)> = vec![];
        order.extend(men.iter().filter(|m| m.typ <= 12 || matches!(m.typ, 20 | 21)).map(|m| (!m.fallen, m.y.max(m.ty) as i32, m.x as i32, Ok(m))));
        order.extend(objs.iter().map(|o| (true, o.y.max(o.ty) as i32, o.x as i32, Err(o))));
        order.sort_by_key(|e| (e.0, e.1, e.2));
        for (_, _, _, e) in order {
            match e {
                Ok(m) if m.typ == 12 => self.draw_prisoner(assets, frame, m),
                Ok(m) if m.typ >= 20 => self.draw_cargo(assets, frame, m),
                Ok(m) => self.draw_man(assets, frame, m),
                Err(o) => self.draw_obj(assets, frame, o),
            }
        }
        if let Some((r, src, framed)) = self.captions.front() {
            // wait_click with the wheel cursor (rules-50 §1.2); a page of the pre-fight window in its parchment frame
            if *framed {
                Window::new(assets, *r, src, &[], &[], &[]).draw(frame, assets);
            } else {
                super::caption(assets, *r, src).draw_text(frame, assets);
            }
            return Some(Cursor::Wheel((self.ticks * 10 / 55 % 16) as u8));
        }
        if let Some((src, sprites)) = self.report.front().filter(|_| self.done()) {
            // each sprite's `%m` item reserves its width; a zero width lets the next one draw over it
            let items: Vec<Vec<u8>> = sprites.iter().map(|&(s, _)| vec![crate::text::PAD_MARK, inline_width(s)]).collect();
            let refs: Vec<&[u8]> = items.iter().map(Vec::as_slice).collect();
            let w = Window::new(assets, REPORT_RECT, src, &refs, &[], &[]);
            w.draw(frame, assets);
            for (k, &(s, raise)) in sprites.iter().enumerate().filter(|&(k, _)| k < w.laid.items.len()) {
                let it = w.item_rect(k);
                let (x, y) = (it.x, it.y - raise);
                match s {
                    Inline::Bank(f, c) => self.sprite_c(assets, frame, f as usize, (x + VIEW.0, y + VIEW.1), c),
                    Inline::Unit(t, c) => {
                        if let Some(&b) = self.base.get(t as usize * 11 + 3) {
                            self.sprite_c(assets, frame, b as usize + 2, (x + VIEW.0, y + VIEW.1), c);
                        }
                    }
                    Inline::Icon(id) => frame.blit(&assets.icon(id)[0], x, y),
                }
            }
            return Some(Cursor::Wheel((self.ticks * 10 / 55 % 16) as u8));
        }
        if self.overlay == Some(Overlay::Intro) {
            self.overlay_window(assets, &[]).unwrap().draw_text(frame, assets);
            return Some(Cursor::Arrow);
        }
        if self.done() && self.menu.is_some() {
            self.draw_menu(frame, assets, mouse);
            if let Some(w) = self.overlay_window(assets, &[]) {
                let hover: Vec<usize> = w.item_at(mouse).into_iter().collect();
                self.overlay_window(assets, &hover).unwrap().draw(frame, assets);
            }
        }
        Some(Cursor::Arrow)
    }

    fn current(&self) -> &Snap {
        &self.frames[self.pos.min(self.frames.len() - 1)]
    }

    /// Screen rectangle of a man's sprite.
    fn sprite_rect(&self, m: &SnapMan) -> [i32; 4] {
        let k = m.typ as usize * 11 + m.state as usize;
        [m.x as i32 * CELL.0 - self.xoff[k] as i32 - VIEW.0, m.y as i32 * CELL.1 - SPRITE_DY - VIEW.1, 25, 40]
    }

    /// The group (index into the menu's groups) of the own man under the pointer; the front-most one.
    fn group_at(&self, (mx, my): (i32, i32)) -> Option<usize> {
        let menu = self.menu.as_ref()?;
        let typs: Vec<u8> = menu.groups.iter().map(|g| g.typ).collect();
        self.current()
            .men
            .iter()
            .filter(|m| !m.fallen && m.side == menu.part & 1 && typs.contains(&m.typ))
            .filter(|m| {
                let [x, y, w, h] = self.sprite_rect(m);
                mx >= x && mx < x + w && my >= y && my < y + h
            })
            .max_by_key(|m| m.y)
            .and_then(|m| typs.iter().position(|&t| t == m.typ))
    }

    /// Status bar and group frames of the tactic menu (§11.5).
    fn draw_menu(&self, frame: &mut Frame, assets: &Assets, mouse: (i32, i32)) {
        let Some(menu) = &self.menu else { return };
        for x in (0..640).step_by(40) {
            frame.blit(&assets.icon(0x05)[0], x, 0);
        }
        let hover = self.group_at(mouse);
        let (left, right) = match hover {
            Some(k) => {
                let g = &menu.groups[k];
                let name = TYPE_NAMES.get(g.typ as usize).copied().unwrap_or("?");
                // the group's cells in corner frames (ICONF4..F7)
                for m in self.current().men.iter().filter(|m| !m.fallen && m.side == menu.part & 1 && m.typ == g.typ) {
                    let (x, y) = (m.x as i32 * CELL.0 - VIEW.0, m.y as i32 * CELL.1 - VIEW.1);
                    for (id, dx, dy) in [(0xF4, 0, 0), (0xF5, CELL.0 - 8, 0), (0xF6, 0, CELL.1 - 5), (0xF7, CELL.0 - 8, CELL.1 - 5)] {
                        frame.blit(&assets.icon(id)[0], x + dx, y + dy);
                    }
                }
                let mut r = b"$C".to_vec();
                r.extend_from_slice(&menu.tactics.get(k).and_then(|&t| self.tactic_names.get(t as usize)).cloned().unwrap_or_default());
                ([b"$C$g".as_slice(), &super::t(&format!("{} {name} ({}%)", g.men, g.strength))].concat(), r)
            }
            None => ([b"$C$g".as_slice(), &super::t(&menu.name)].concat(), [b"$C".as_slice(), &super::t("Wählt eine Einheit!")].concat()),
        };
        for (src, x) in [(left, 0), (right, 320)] {
            text::draw(frame, &assets.font, &text::layout(&assets.font, &src, &[], &[], 320, &[]), x, -4);
        }
        if let Some(flag) = assets.icon8(UNITS).get(1444) {
            let r = &self.ramps[menu.part as usize & 3];
            let mut f = flag.clone();
            f.pixels.iter_mut().for_each(|p| if *p < 16 { *p = r[*p as usize] as u16 });
            frame.blit_idx(&f, 304, 0);
        }
    }

    /// Left click: the clicked group takes its next allowed tactic (§11.5).
    pub fn click(&mut self, pos: (i32, i32)) {
        let Some(k) = self.group_at(pos) else { return };
        let Some(menu) = &mut self.menu else { return };
        let g = menu.groups[k];
        let cur = menu.tactics[k] as usize;
        if let Some(t) = (1..=11).map(|d| (cur + d) % 11).find(|&t| g.allowed & (1 << t) != 0) {
            menu.tactics[k] = t as u8;
        }
    }

    /// The sim answer for the menu: nibble k = tactic + 1 of group k.
    pub fn answer(&self) -> u32 {
        self.menu.as_ref().map_or(0, |m| m.tactics.iter().enumerate().take(8).fold(0, |a, (k, &t)| a | (t as u32 + 1) << (4 * k)))
    }

    pub fn close_menu(&mut self) {
        self.menu = None;
    }
}

/// The pen advance of a report sprite (rules-59 §3, §5): the banner's companion frame 691 is zero wide.
pub fn inline_width(s: Inline) -> u8 {
    match s {
        Inline::Bank(691, _) => 0,
        Inline::Icon(_) => 46,
        _ => 40,
    }
}

#[cfg(test)]
mod tests {
    /// rules-52 §0.4/§5: "Wohlan …" in the caption box (0, 180, 640, 120) with the caption text 2 px into the box puts
    /// the glyph tops at y 181 and 217, as in the live capture (2026-11-16).
    #[test]
    fn real_wohlan_caption_rows() {
        let Some(assets) = super::super::tests::test_assets() else { return };
        let mut frame = crate::gfx::Frame::new(assets.palette);
        frame.begin(&assets);
        super::super::caption(&assets, [0, 180, 640, 120], &assets.text(super::INTRO)).draw_text(&mut frame, &assets);
        let rows: Vec<usize> = (0..480).filter(|&y| frame.pixels[y * 640..(y + 1) * 640].iter().any(|&p| p != 0)).collect();
        let tops: Vec<usize> = rows.iter().copied().filter(|&y| !rows.contains(&y.wrapping_sub(1))).collect();
        assert_eq!(tops, [181, 217]);
    }
}
