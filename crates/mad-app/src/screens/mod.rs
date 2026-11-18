//! Screens as a state machine: each screen updates on fixed ticks and draws into the framebuffer.

pub mod credits;
pub mod film;
pub mod game;
pub mod menu;
pub mod placeholder;
pub mod reveal;
pub mod setup;

use crate::{
    assets::Assets,
    audio::Audio,
    gfx::Frame,
    input::{Cursor, Input},
    text::{self, Item, Laid},
};
use std::path::PathBuf;

pub struct Ctx<'a> {
    pub assets: &'a Assets,
    pub audio: &'a mut Audio,
    pub music_dir: Option<PathBuf>,
}

pub enum Next {
    Stay,
    Go(Box<dyn Screen>),
    Quit,
}

pub trait Screen {
    /// One fixed tick of `dt_ms`, with the input events since the last tick.
    fn update(&mut self, ctx: &mut Ctx, input: &Input, dt_ms: u32) -> Next;
    /// Draws the screen; returns the pointer shape, `None` to hide it.
    fn draw(&self, frame: &mut Frame, assets: &Assets, mouse: (i32, i32)) -> Option<Cursor>;
}

/// Framed window tiles, `docs/re/formats.md` §7.2: 40x40 tiles `05-11` picked by window and tile
/// position. Transform 1 = as stored, 2 = flipped vertically, 3 = flipped horizontally, 4 = both
/// ([UNVERIFIED] per RE). Partial tiles at the right/bottom are clipped to the window.
const TILE: i32 = 40;

pub fn draw_window(frame: &mut Frame, assets: &Assets, [wx, wy, w, h]: [i32; 4]) {
    let (last_x, last_y) = ((w - 1) / TILE * TILE, (h - 1) / TILE * TILE);
    let m = |v: i32| (v / TILE % 4) as u16;
    let clip = [wx.max(0), wy.max(0), (wx + w).min(640), (wy + h).min(480)];
    for y in (0..h).step_by(TILE as usize) {
        for x in (0..w).step_by(TILE as usize) {
            let (top, bottom, left, right) = (y == 0, y == last_y, x == 0, x == last_x);
            let (id, fx, fy) = match () {
                _ if top && left => (0x06 + m(wx), false, false),
                _ if top && right => (0x06 + m(wy), true, false),
                _ if bottom && left => (0x06 + (m(wx) + 2) % 4, false, true),
                _ if bottom && right => (0x06 + (m(wy) + 2) % 4, true, true),
                _ if top => (0x0E + m(x), false, false),
                _ if bottom => (0x0E + m(x), false, true),
                _ if left => (0x0A + m(y), false, false),
                _ if right => (0x0A + m(y), true, false),
                _ => (0x05, false, false),
            };
            // palette index 0 is transparent in the frame tiles (the original shows the screen
            // behind the torn corners; measured against DOSBox-X, docs/DOSBOX.md)
            let mut tile = assets.icon(id)[0].clone();
            for (o, &p) in tile.opaque.iter_mut().zip(&tile.pixels) {
                *o &= p != 0;
            }
            frame.blit_clip(&tile, wx + x, wy + y, fx, fy, clip);
        }
    }
}

/// Scroll variant of the framed window (style flag 0x2000, docs/re/formats.md §7.2): only left edge, interior and
/// right edge tiles, with a roll above and below (`0x30670`: caps `0x12 + k`, middles `0x1A + k`, `k = (y % 16) / 2`).
/// `s` = scroll offset: the tiles move up with the text (`cy + s`) and the rolls turn (rules-42 §1).
pub fn draw_scroll(frame: &mut Frame, assets: &Assets, [wx, wy, w, h]: [i32; 4], s: i32) {
    let last_x = (w - 1) / TILE * TILE;
    let clip = [wx.max(0), wy.max(0), (wx + w).min(640), (wy + h).min(480)];
    let first = s.div_euclid(TILE) * TILE;
    for cy in (first..s + h).step_by(TILE as usize) {
        let y = cy - s;
        for x in (0..w).step_by(TILE as usize) {
            let id = if x == 0 || x == last_x { 0x0A + (cy / TILE).rem_euclid(4) as u16 } else { 0x05 };
            let mut tile = assets.icon(id)[0].clone();
            for (o, &p) in tile.opaque.iter_mut().zip(&tile.pixels) {
                *o &= p != 0;
            }
            frame.blit_clip(&tile, wx + x, wy + y, x == last_x && x != 0, false, clip);
        }
    }
    for by in [wy - 16, wy + h] {
        // the roll tile turns with the scroll offset alone (rules-42 §1, formats §8: k = (s mod 16) / 2)
        let k = (s.rem_euclid(16) / 2) as u16;
        // colour 0 is see-through here as in the edge tiles (live 1600 chronicle A/B: the cap corners)
        let clear = |mut t: mad_assets::sprite::Image| {
            for (o, &p) in t.opaque.iter_mut().zip(&t.pixels) {
                *o &= p != 0;
            }
            t
        };
        let cap = &clear(assets.icon(0x12 + k)[0].clone());
        let mid = &clear(assets.icon(0x1A + k)[0].clone());
        frame.blit(cap, wx, by);
        let mut x = wx + cap.width as i32;
        while x < wx + w - cap.width as i32 {
            frame.blit(mid, x, by);
            x += mid.width as i32;
        }
        frame.blit_clip(cap, wx + w - cap.width as i32, by, true, false, [0, 0, 640, 480]);
    }
}

/// A framed window at a fixed rectangle with laid-out markup text (12 px inset, top-aligned, §8.3).
pub struct Window {
    pub laid: Laid,
    rect: [i32; 4],
    /// Extra vertical offset of the text inside the frame.
    dy: i32,
    scroll: bool,
}

impl Window {
    pub fn new(assets: &Assets, rect: [i32; 4], src: &[u8], items: &[&[u8]], args: &[&[u8]], lit: &[usize]) -> Self {
        Self { laid: text::layout(&assets.font, src, items, args, rect[2], lit), rect, dy: 0, scroll: false }
    }

    /// Choice under the mouse, in screen coordinates.
    pub fn item_at(&self, (mx, my): (i32, i32)) -> Option<usize> {
        let (ox, oy) = self.origin();
        self.laid.items.iter().position(|it| it.contains(mx - ox, my - oy))
    }

    /// Screen rectangle of choice `k`.
    pub fn item_rect(&self, k: usize) -> Item {
        let (ox, oy) = self.origin();
        let it = self.laid.items[k];
        Item { x: it.x + ox, y: it.y + oy, ..it }
    }

    /// Moves the text down inside the frame (the branch panel's text sits 5 px lower than the inset).
    pub fn with_text_dy(mut self, dy: i32) -> Self {
        self.dy = dy;
        self
    }

    /// Draws the frame as a scroll (turn-end and year-end reports, style 0xA810).
    pub fn with_scroll(mut self) -> Self {
        self.scroll = true;
        self
    }

    fn origin(&self) -> (i32, i32) {
        (self.rect[0], self.rect[1] + self.dy)
    }

    pub fn draw(&self, frame: &mut Frame, assets: &Assets) {
        self.draw_over(frame, assets, |_| {});
    }

    /// Text only, for windows drawn straight onto a full-screen picture.
    pub fn draw_text(&self, frame: &mut Frame, assets: &Assets) {
        let (ox, oy) = self.origin();
        text::draw(frame, &assets.font, &self.laid, ox, oy);
    }

    /// Frame, then `back` (watermarks), then the text.
    pub fn draw_over(&self, frame: &mut Frame, assets: &Assets, back: impl FnOnce(&mut Frame)) {
        if self.scroll {
            draw_scroll(frame, assets, self.rect, -self.dy);
        } else {
            draw_window(frame, assets, self.rect);
        }
        back(frame);
        let (ox, oy) = self.origin();
        if !self.scroll {
            return text::draw(frame, &assets.font, &self.laid, ox, oy);
        }
        // a scroll's text is clipped to its rectangle (scrolled lines pass under the rolls, rules-42 §1)
        let before = frame.pixels.clone();
        text::draw(frame, &assets.font, &self.laid, ox, oy);
        let [x, y, w, h] = self.rect;
        for row in (0..480).filter(|&r| r < y || r >= y + h) {
            let (a, b) = ((row * 640 + x.max(0)) as usize, (row * 640 + (x + w).min(640)) as usize);
            frame.pixels[a..b].copy_from_slice(&before[a..b]);
        }
    }
}

/// The start/new-game screen backdrop (§7.1): open book `ICOND4` at y = 40 (bottom rows clipped,
/// [UNVERIFIED]), parchment banner `ICON22` at (0,1), then `text` printed into the full-screen window.
/// A full-screen picture drawn lower than y 0 wraps in the EXE's off-screen frame: its bottom rows
/// appear at the top, behind the banner (formats §10.10).
pub fn blit_wrapped(frame: &mut Frame, img: &mad_assets::sprite::Image, x: i32, y: i32) {
    frame.blit(img, x, y);
    if y > 0 && img.height == 480 {
        frame.blit(img, x, y - 480);
    }
}

pub fn backdrop(frame: &mut Frame, assets: &Assets, text: &[u8], args: &[&[u8]]) {
    frame.begin(assets);
    blit_wrapped(frame, &assets.icon(0xD4)[0], 0, 40);
    frame.blit(&assets.icon(0x22)[0], 0, 1);
    text::draw(frame, &assets.font, &text::layout(&assets.font, text, &[], args, 640, &[]), 0, 0);
}
