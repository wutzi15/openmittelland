//! Text layout with the game's `$`/`%` markup in the `ICONCS` font, per `docs/re/formats.md` §8.3.
//! Coordinates are relative to the window's top-left corner.

use crate::gfx::Frame;
use mad_assets::font::{Font, glyph_for_cp437};

pub const LINE_H: i32 = 24;
/// Default margins and top inset of a window (`$H`/`$P`).
pub const INSET: i32 = 12;
const DARK: u8 = 0x80;
pub const SPACE: u8 = b' ';

/// Applies the game's font-load fix-ups (§8.3): space advance 5 px, and seven gold codes take the
/// advance of their dark twins.
pub fn fix_font(font: &mut Font) {
    for c in [0x22, 0x23, 0x2A, 0x2F, 0x3D, 0x40, 0x60] {
        font.advance[c] = font.advance[c | 0x80];
    }
    // Measured against the original (DOSBox-X, docs/FIDELITY.md): the dark half advances like the
    // gold half (the file lists one pixel more for a few dark letters such as u and l), and every
    // glyph but the space advances one pixel more than the file says.
    for c in 0..0x80 {
        font.advance[c | 0x80] = font.advance[c];
    }
    // The space is 5 px in the game's font fix-up (§8.3), 6 on screen with the same extra pixel.
    font.advance[SPACE as usize] = 5;
    for a in font.advance.iter_mut() {
        *a += 1;
    }
    font.advance[SPACE as usize | 0x80] = font.advance[SPACE as usize];
    font.advance[PAD as usize] = 1;
    font.advance[PAD as usize | 0x80] = 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Item {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

#[derive(Debug, Default)]
pub struct Laid {
    /// (x, y, glyph code).
    pub glyphs: Vec<(i32, i32, u8)>,
    /// Bounding box of each `%m`/`%a` choice, in order.
    pub items: Vec<Item>,
    /// y just below the last line.
    pub height: i32,
    /// Indices into `glyphs` of the highlighted (hovered) choices: drawn as a flat colour.
    pub hot: Vec<usize>,
}

#[derive(Clone, Copy, PartialEq)]
enum Align {
    L,
    C,
    R,
    /// Justified (Blocksatz), the window default: wrapped lines stretch their spaces.
    B,
}

#[derive(Clone)]
struct Seg {
    align: Align,
    /// Margins in force when the segment started; alignment or margin changes start a new segment
    /// on the same line (the banner puts a right-aligned and a centred part on one row, §9.1).
    lm: i32,
    rm: i32,
    /// Set inside `$( ... $)`: if it would overlap another segment of the row it drops to the next
    /// row (the banner's name line, measured against the original).
    keep: bool,
    glyphs: Vec<u8>,
    /// Per glyph: inside `$h` (drawn highlighted like a lit item); shorter than `glyphs` means false.
    hl: Vec<bool>,
    /// Glyph spacing relative to the default 1 px (`$0s` gives -1, formats §8.6).
    sp: i32,
    /// (first glyph, end glyph, continues an item from the line above) of each choice, in item order.
    items: Vec<(usize, usize, bool)>,
}

struct Layout<'a> {
    font: &'a Font,
    width: i32,
    out: Laid,
    line: Vec<Seg>,
    align: Align,
    y: i32,
    lm: i32,
    rm: i32,
    lit: Vec<usize>,
    keep: bool,
    sp: i32,
    /// `$h` is on (rules-42 §3 highlight: solid red on dark text; live A/B: the year-end denunciation line)
    hl: bool,
}

impl Layout<'_> {
    fn adv(&self, g: &[u8], sp: i32) -> i32 {
        g.iter().map(|&c| self.font.advance[c as usize] as i32 + if c == PAD { 0 } else { sp }).sum()
    }

    fn seg(&mut self) -> &mut Seg {
        let (align, lm, rm, sp) = (self.align, self.lm, self.rm, self.sp);
        match self.line.last_mut() {
            Some(s) if s.glyphs.is_empty() => (s.align, s.lm, s.rm, s.sp) = (align, lm, rm, sp),
            Some(s) if (s.align, s.lm, s.rm, s.sp) == (align, lm, rm, sp) => {}
            _ => self.line.push(Seg { align, lm, rm, keep: self.keep, glyphs: vec![], hl: vec![], sp, items: vec![] }),
        }
        self.line.last_mut().unwrap()
    }

    fn push(&mut self, bytes: &[u8], gold: bool) {
        let g: Vec<u8> = bytes.iter().filter_map(|&b| glyph_for_cp437(b)).map(|g| if gold || g == SPACE { g } else { g | DARK }).collect();
        let hl = self.hl;
        let s = self.seg();
        let n = s.glyphs.len();
        s.hl.resize(n, false);
        s.hl.extend(std::iter::repeat_n(hl, g.len()));
        s.glyphs.extend(g);
    }

    fn set_align(&mut self, a: Align) {
        self.align = a;
    }

    /// Places one segment at the current y; `stretch` spreads the free width over its spaces.
    fn place(&mut self, s: &Seg, stretch: bool) {
        let box_w = self.width - s.lm - s.rm;
        let w = self.adv(&s.glyphs, s.sp);
        let spaces = s.glyphs.iter().filter(|&&g| g == SPACE).count() as i32;
        let (mut x, extra) = match s.align {
            Align::C => (s.lm + (box_w - w) / 2, 0),
            Align::R => (s.lm + box_w - w, 0),
            Align::B if stretch && spaces > 0 => (s.lm, box_w - w),
            _ => (s.lm, 0),
        };
        let (mut seen, mut xs) = (0, Vec::with_capacity(s.glyphs.len() + 1));
        for &g in &s.glyphs {
            xs.push(x);
            self.out.glyphs.push((x, self.y, g));
            x += self.font.advance[g as usize] as i32 + if g == PAD { 0 } else { s.sp };
            if g == SPACE && extra > 0 {
                seen += 1;
                // rules-47 §4: free / count each, plus 1 for the first free mod count spaces
                x += extra / spaces + (seen <= extra % spaces) as i32;
            }
        }
        xs.push(x);
        let base = self.out.glyphs.len() - s.glyphs.len();
        self.out.hot.extend(s.hl.iter().enumerate().filter(|&(_, &h)| h).map(|(k, _)| base + k));
        for &(a, b, cont) in &s.items {
            if cont && let Some(last) = self.out.items.last_mut() {
                // a choice wrapped onto this line: its box grows downwards and sideways
                let (x0, x1) = (last.x.min(xs[a]), (last.x + last.w).max(xs[b]));
                *last = Item { x: x0, y: last.y, w: x1 - x0, h: self.y + LINE_H - last.y };
                if self.lit.contains(&(self.out.items.len() - 1)) {
                    self.out.hot.extend(base + a..base + b);
                }
                continue;
            }
            if self.lit.contains(&self.out.items.len()) {
                self.out.hot.extend(base + a..base + b);
            }
            self.out.items.push(Item { x: xs[a], y: self.y, w: xs[b] - xs[a], h: LINE_H });
        }
    }

    /// Emits the pending line at y, then moves down by `dy` (0 for `$E`).
    fn flush(&mut self, dy: i32) {
        // ponytail: greedy word wrap only for a lone segment (choices are carried over); mixed
        // alignments on one row are not wrapped.
        // the last glyph's spacing does not count against the box (rules-47 §8)
        if let [s] = self.line.as_slice()
            && self.adv(&s.glyphs, s.sp) - (1 + s.sp) > self.width - s.lm - s.rm
        {
            let piece = |s: &Seg, lo: usize, hi: usize| Seg {
                glyphs: s.glyphs[lo..hi].to_vec(),
                hl: s.hl.get(lo..hi.min(s.hl.len())).unwrap_or(&[]).to_vec(),
                items: s.items.iter().filter(|&&(a, b, _)| b > lo && a < hi).map(|&(a, b, c)| (a.max(lo) - lo, b.min(hi) - lo, c || a < lo)).collect(),
                ..s.clone()
            };
            let box_w = self.width - s.lm - s.rm;
            let s = self.line.pop().unwrap();
            let (mut start, mut space, mut w) = (0, None, 0);
            for (i, &g) in s.glyphs.iter().enumerate() {
                // rules-47 §8: before a character is appended the line wraps if box < width so far + its advance
                // without the letter spacing (a space counts 5); the cut is at the last space before it
                let add = self.font.advance[g as usize] as i32 + s.sp;
                if w + add - (1 + s.sp) > box_w
                    && let Some((at, hyphen)) = space.take()
                {
                    // a space is dropped at the cut, a hyphen stays on the upper line (rules-47 §8.3)
                    let end = if hyphen { at + 1 } else { at };
                    self.place(&piece(&s, start, end), true);
                    self.y += LINE_H;
                    start = at + 1;
                    w = self.adv(&s.glyphs[start..i], s.sp);
                }
                w += add;
                if g == SPACE {
                    space = Some((i, false));
                } else if g & !DARK == b'-' {
                    space = Some((i, true));
                }
            }
            self.line.push(piece(&s, start, s.glyphs.len()));
        }
        let mut taken: Vec<(i32, i32)> = vec![];
        let mut dropped = vec![];
        for s in std::mem::take(&mut self.line) {
            let (box_w, w) = (self.width - s.lm - s.rm, self.adv(&s.glyphs, s.sp));
            let x0 = match s.align {
                Align::C => s.lm + (box_w - w) / 2,
                Align::R => s.lm + box_w - w,
                _ => s.lm,
            };
            if s.keep && taken.iter().any(|&(a, b)| x0 < b && a < x0 + w) {
                dropped.push(s);
                continue;
            }
            taken.push((x0, x0 + w));
            self.place(&s, false);
        }
        for s in dropped {
            self.y += LINE_H;
            self.place(&s, false);
        }
        self.y += dy;
        self.out.height = self.out.height.max(self.y);
    }
}

/// Lays out `src` for a window `width` wide. `%m`/`%a` consume `items` in order, `%d`/`%s`/`%c`
/// consume `args`.
///
/// Colour ([UNVERIFIED] reading of the §8.3 flags): `$t` selects the dark font half and `$g` the gold
/// one; the default is dark (parchment windows). This fits every use seen: the banner, the Erträge
/// pages and the player form start with `$t` (dark on parchment/book), "Nächster Spieler" uses `$g`
/// (on a dark picture). `$h` text is drawn highlighted like a lit item. Titles (`^ … _`) and the items in `lit`
/// (hovered/selected) are always gold.
pub fn layout(font: &Font, src: &[u8], items: &[&[u8]], args: &[&[u8]], width: i32, lit: &[usize]) -> Laid {
    let mut l = Layout { font, width, out: Laid::default(), line: vec![], align: Align::B, y: INSET, lm: INSET, rm: INSET, lit: lit.to_vec(), keep: false, sp: 0, hl: false };
    let (mut dark, mut next_item, mut next_arg) = (true, 0, 0);
    let mut i = 0;
    while i < src.len() {
        let c = src[i];
        i += 1;
        let gold = !dark;
        match c {
            b'$' => {
                let digits = src[i..].iter().take_while(|b| b.is_ascii_digit()).count();
                let n: i32 = std::str::from_utf8(&src[i..i + digits]).ok().and_then(|s| s.parse().ok()).unwrap_or(0);
                i += digits;
                let Some(&code) = src.get(i) else { break };
                i += 1;
                match code {
                    b'$' => l.push(b"$", gold),
                    // `$e` stands for the accented e of "Exposé" (seen in 0xBAE64; glyph 0x1E per the original catalogue page)
                    b'e' => l.push(b"\x82", gold),
                    b'N' => l.flush(LINE_H),
                    // §8.3 reads `$Z` as a half-height break. Taken literally, text on the line above
                    // overlaps the next line, so a line with text moves down a full line and only an
                    // empty line moves half ([UNVERIFIED]).
                    b'Z' => l.flush(if l.line.is_empty() { LINE_H / 2 } else { LINE_H + LINE_H / 2 }),
                    b'Y' => l.flush(LINE_H + if l.line.is_empty() { LINE_H / 2 } else { LINE_H }),
                    b'X' => l.flush(2 * LINE_H),
                    b'E' => l.flush(0),
                    b'L' => l.set_align(Align::L),
                    b'R' => l.set_align(Align::R),
                    b'C' => l.set_align(Align::C),
                    b'B' => l.set_align(Align::B),
                    b'H' | b'P' => {
                        if !l.line.is_empty() {
                            l.flush(0);
                        }
                        (l.lm, l.rm, l.y) = (INSET, INSET, INSET + n * LINE_H);
                    }
                    // ours, not in the EXE: margins in pixels (item boxes of fixed rectangles)
                    b'[' => l.lm = n,
                    // ours: n px of horizontal space (the unused part of a reserved item width)
                    b'}' => {
                        for _ in 0..n {
                            l.seg().glyphs.push(PAD);
                        }
                    }
                    b'{' => {
                        if !l.line.is_empty() {
                            l.flush(0);
                        }
                        l.y = n;
                    }
                    b']' => l.rm = n,
                    // margin codes print a pending line first and move down (rules-59 §8: `$2>` at 0x32801); after `$E`
                    // the line is empty and nothing moves. `$|` = `$2>$2<`, so a row ending in `$|$N` is 48 px high.
                    b'>' | b'<' | b'|' => {
                        if !l.line.is_empty() {
                            l.flush(LINE_H);
                        }
                        match code {
                            b'>' => l.lm = n * 6,
                            b'<' => l.rm = n * 6,
                            _ => (l.lm, l.rm) = (12, 12),
                        }
                    }
                    b'=' => l.rm = width - l.lm - n * 6,
                    b'/' => {
                        let m = src.get(i).filter(|b| b.is_ascii_digit()).map_or(2, |b| (b - b'0') as i32);
                        i += src.get(i).is_some_and(u8::is_ascii_digit) as usize;
                        let col = (width - 2 * INSET) / m.max(1);
                        (l.lm, l.rm) = (INSET + n * col, width - INSET - (n + 1) * col);
                    }
                    // `$<n>c`: glyph code n itself (`$18c` is the small ², formats §8.3)
                    b'c' => l.push(&[n as u8], gold),
                    // `$ns`: flush the line, then glyph spacing n (formats §8.6; ours bakes in 1)
                    b's' => {
                        if !l.line.is_empty() {
                            l.flush(0);
                        }
                        l.sp = n - 1;
                    }
                    b'h' => l.hl = !l.hl,
                    b'(' => l.keep = true,
                    b')' => l.keep = false,
                    b't' => dark = true,
                    b'g' => dark = false,
                    _ => {} // flags d n ( ), waits W V, spacing s, `$a`: no visible effect here
                }
            }
            b'%' => {
                let Some(&code) = src.get(i) else { break };
                i += 1;
                match code {
                    b'm' | b'a' => {
                        let a = l.seg().glyphs.len();
                        let it = items.get(next_item).copied().unwrap_or(b"");
                        // `PAD_MARK w`: the item reserves w px (formats §8.6); the shortfall is padded after its text
                        let (it, reserve) = match it {
                            [PAD_MARK, w, rest @ ..] => (rest, *w as i32),
                            _ => (it, 0),
                        };
                        l.push(it, gold);
                        let drawn = l.adv(&l.line.last().map_or(vec![], |s| s.glyphs[a..].to_vec()), l.sp);
                        for _ in drawn..reserve {
                            l.seg().glyphs.push(PAD);
                        }
                        let b = l.seg().glyphs.len();
                        l.seg().items.push((a, b, false));
                        next_item += 1;
                    }
                    b'd' | b's' | b'c' => {
                        l.push(args.get(next_arg).copied().unwrap_or(b""), gold);
                        next_arg += 1;
                    }
                    b'%' => l.push(b"%", gold),
                    // `%1N1` (person reference) or `%4E`: need the sim, skipped
                    b'0'..=b'9' => i += if src.get(i) == Some(&b'N') { 2 } else { 1 },
                    _ => {}
                }
            }
            // title ornaments take the current font: dark on parchment, gold in a gold caption (live A/B, the raid
            // value caption "Information", 2026-11-17)
            b'^' | b'_' => l.push(&[c], gold),
            _ => l.push(&[c], gold),
        }
    }
    if !l.line.is_empty() {
        l.flush(LINE_H);
    }
    l.out
}

/// Colour of highlighted dark text (solid colour 1, rules-42 §3), measured in the headless harness capture of the
/// branch screen's hovered verb (the earlier (182, 56, 50) came from a Display-P3 window capture).
pub const HOT: [u8; 3] = [198, 41, 41];

/// The name-entry caret (rules-42 §3): glyph 0x24 (a solid ◄) after the text, solid colour 1 = palette 0x25.
pub fn draw_caret(frame: &mut Frame, font: &Font, x: i32, y: i32) {
    frame.blit_rgb_solid(&font.glyphs[0x24], x, y, HOT);
}

/// First byte of an item text that reserves a width: `[PAD_MARK, w, text...]`.
pub const PAD_MARK: u8 = 0x01;
/// Blank glyph of advance 1 used to fill a reserved item width (set in `fix_font`).
const PAD: u8 = 0x0B;

/// One glyph; gold ones (below 0x80) carry an outline: the dark twin (`| 0x80`) drawn with its own
/// colours at the eight neighbours in raster order, then the glyph (pixel-exact against the
/// original's Löhne headings, headless DOSBox-X capture).
pub fn blit_glyph(frame: &mut Frame, font: &Font, g: u8, x: i32, y: i32) {
    if g < 0x80 && g != SPACE {
        for (dx, dy) in OUTLINE {
            frame.blit(&font.glyphs[g as usize | 0x80], x + dx, y + dy);
        }
    }
    frame.blit(&font.glyphs[g as usize], x, y);
}

const OUTLINE: [(i32, i32); 8] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

pub fn draw(frame: &mut Frame, font: &Font, laid: &Laid, x: i32, y: i32) {
    if std::env::var_os("MAD_DUMP_GLYPHS").is_some() {
        for &(gx, gy, g) in &laid.glyphs {
            eprintln!("G {} {} {}", x + gx, y + gy, g);
        }
        eprintln!("G-END");
    }
    for (i, &(gx, gy, g)) in laid.glyphs.iter().enumerate() {
        if laid.hot.contains(&i) && g < 0x80 && g != SPACE {
            // gold text: outline as usual, the glyph in solid colour 9 = palette 0x01 (rules-42 §3)
            for (dx, dy) in OUTLINE {
                frame.blit(&font.glyphs[g as usize | 0x80], x + gx + dx, y + gy + dy);
            }
            frame.blit_rgb_solid(&font.glyphs[g as usize], x + gx, y + gy, frame.palette[0x01]);
        } else if laid.hot.contains(&i) {
            frame.blit_rgb_solid(&font.glyphs[g as usize], x + gx, y + gy, HOT);
        } else {
            blit_glyph(frame, font, g, x + gx, y + gy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mad_assets::sprite::Image;

    fn font() -> Font {
        let mut advance = [10; 256];
        advance[SPACE as usize] = 5;
        advance[PAD as usize] = 1; // as `fix_font` sets it
        Font { advance, glyphs: vec![Image { width: 0, height: 0, pixels: vec![], opaque: vec![] }; 256] }
    }

    /// rules-47 §8.3: a hyphen is a break point and stays on the upper line.
    #[test]
    fn wrap_after_hyphen() {
        // box = 100 - 24 = 76: "aaa-" is 40 wide, "bbbb" 40 more
        let l = layout(&font(), b"$Laaa-bbbb", &[], &[], 100, &[]);
        let ys: Vec<i32> = l.glyphs.iter().map(|g| g.1).collect();
        assert!(ys[..4].iter().all(|&y| y == ys[0]));
        assert!(ys[4..].iter().all(|&y| y > ys[0]), "{ys:?}");
        assert_eq!(l.glyphs[3].2 & !DARK, b'-');
    }

    #[test]
    fn lines_alignment_and_items() {
        // box is 100 - 2*12 = 76 wide, starting at x = 12, y = 12
        let l = layout(&font(), b"$C^ T _$Z$LA %m$NB$E$R%m", &[b"xy", b"z"], &[], 100, &[1]);
        // "^ T _" = 3 glyphs + 2 spaces = 40 px, centred, dark like the original's titles
        assert_eq!(l.glyphs[0], (12 + 18, 12, b'^' | DARK));
        // `$Z` after text: a line and a half (measured against the original, docs/FIDELITY.md)
        assert_eq!(l.glyphs[5], (12, 48, b'A' | DARK));
        assert_eq!(l.items[0], Item { x: 27, y: 48, w: 20, h: LINE_H });
        // `$E` keeps the row: "B" left, item 1 right-aligned and lit (items add no padding, rules-47)
        assert_eq!(l.items[1], Item { x: 78, y: 72, w: 10, h: LINE_H });
        assert_eq!(*l.glyphs.last().unwrap(), (78, 72, b'z' | DARK));
        assert_eq!(l.hot, vec![l.glyphs.len() - 1], "the lit item is drawn as a flat colour");
        assert_eq!(l.height, 96);
        // `$Z` on an empty line: half a line only
        assert_eq!(layout(&font(), b"$ZA", &[], &[], 100, &[]).glyphs[0].1, 24);
    }

    /// §9.1 banner: the right-aligned part on line 0; `$|` prints it and the centred part goes to line 1
    /// (rules-59 §8: margin codes flush a pending line); `$t`/`$g` pick dark/gold.
    #[test]
    fn banner_segments_and_colour() {
        let l = layout(&font(), b"$t$|$H$3<$Rab$|$C$gcd", &[], &[], 640, &[]);
        assert_eq!(l.glyphs[0], (640 - 18 - 20, 12, b'a' | DARK));
        assert_eq!(l.glyphs[2], (310, 36, b'c'));
    }

    /// rules-59 §8: a row ending in `$|$N` is two lines high; after `$E` a margin code does not move.
    #[test]
    fn bar_then_newline_is_two_lines() {
        let l = layout(&font(), b"$La$|$Nb$E$10>c$|$N", &[], &[], 640, &[]);
        let y = |ch: u8| l.glyphs.iter().find(|g| g.2 & !DARK == ch).unwrap().1;
        assert_eq!((y(b'a'), y(b'b'), y(b'c')), (12, 60, 60));
    }

    #[test]
    fn home_margins_and_digits() {
        let l = layout(&font(), b"x$4H$10>$56<$LA", &[], &[], 640, &[]);
        assert_eq!(l.glyphs[1], (60, 12 + 4 * 24, b'A' | DARK));
        // right column: margins 336 / 60
        let l = layout(&font(), b"$56>$10<$RA", &[], &[], 640, &[]);
        assert_eq!(l.glyphs[0], (640 - 60 - 10, 12, b'A' | DARK));
    }

    #[test]
    fn wraps_and_justifies() {
        // box 60 wide: "aaa" / "bbbb c" (stretched, c moves 5 px right) / "dd" (last line, not stretched)
        let l = layout(&font(), b"aaa bbbb c dd$N", &[], &[], 84, &[]);
        let at = |c: u8| l.glyphs.iter().find(|g| g.2 & !DARK == c).map(|g| (g.0, g.1));
        assert_eq!(at(b'a'), Some((12, 12)));
        assert_eq!(at(b'c'), Some((12 + 40 + 5 + 5, 36)));
        assert_eq!(at(b'd'), Some((12, 60)));
        // centred wrap
        let l = layout(&font(), b"$Caaa bbb ccc", &[], &[], 84, &[]);
        let a: Vec<(i32, i32)> = l.glyphs.iter().filter(|g| g.2 & !DARK == b'a').map(|g| (g.0, g.1)).collect();
        assert_eq!(a[0], (12 + (60 - 30) / 2, 12));
    }

    #[test]
    fn justified_stretches_wrapped_lines() {
        // box 60: "aa bb cc" (70) is too wide, so "aa bb" (45) gets 15 extra px on its one space
        let l = layout(&font(), b"aa bb cc", &[], &[], 84, &[]);
        let b0 = l.glyphs.iter().find(|g| g.2 & !DARK == b'b').unwrap();
        assert_eq!(b0.0, 12 + 20 + 5 + 15);
    }

    /// rules-47 §4: the remainder goes to the first spaces (box 61: 11 free px over 2 spaces = 6 + 5).
    #[test]
    fn justify_remainder_to_first_spaces() {
        let l = layout(&font(), b"a b cc dddddd", &[], &[], 85, &[]);
        let at = |c: u8| l.glyphs.iter().find(|g| g.2 & !DARK == c).unwrap().0;
        assert_eq!((at(b'b'), at(b'c')), (12 + 15 + 6, 12 + 15 + 6 + 15 + 5));
    }

    #[test]
    fn umlauts_args_and_skipped_codes() {
        let l = layout(&font(), b"\x84%d$4>%1N1%s%%$t$(x", &[], &[b"7", b"\x81"], 100, &[]);
        let g: Vec<u8> = l.glyphs.iter().map(|g| g.2 & !DARK).collect();
        assert_eq!(g, [0x7B, b'7', 0x7D, b'%', b'x']);
    }
}
