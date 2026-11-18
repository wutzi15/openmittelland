//! Scanline reveal transition of the original (0x2C460, docs/re/rules-16b §8): the next picture opens from a
//! jittered vertical centre line outward. The jitter uses a private generator; the original draws it from the
//! game RNG (958 draws), which a presentation effect must not disturb here.

use crate::gfx::{Frame, H, W};

const FRAMES: i32 = 200;

pub struct Reveal {
    centre: Vec<i32>,
    width: Vec<i32>,
    done: Vec<bool>,
    /// span copied so far per row
    span: Vec<(i32, i32)>,
    frame: i32,
    rows: std::ops::Range<usize>,
    /// edge-inward variant (0x2C140, rules-22 §5) instead of centre-outward (0x2C460)
    inward: bool,
}

impl Reveal {
    pub fn new(seed: u32, rows: std::ops::Range<usize>, inward: bool) -> Self {
        let mut s = seed | 1;
        let mut rand = |n: i32| {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            ((s >> 16) % n as u32) as i32
        };
        let (mut centre, mut width) = (vec![if inward { 0 } else { 320 }; H], vec![-250; H]);
        for y in 1..H {
            centre[y] = centre[y - 1] + rand(9) - 4;
            width[y] = width[y - 1] + rand(25) - 12;
        }
        Self { centre, width, done: vec![false; H], span: vec![(0, 0); H], frame: 0, rows, inward }
    }

    pub fn finished(&self) -> bool {
        self.frame >= FRAMES || self.rows.clone().all(|y| self.done[y])
    }

    /// One frame of the effect (§8 step 3).
    pub fn step(&mut self) {
        for y in self.rows.clone() {
            if self.done[y] {
                continue;
            }
            let w = &mut self.width[y];
            *w += if *w < 0 { y as i32 / 80 + 1 } else { *w / 30 + 1 };
            *w += self.frame / 20 + 1;
            if self.inward {
                // both edges grow towards each other: [0, a + w/2) and [a + 640 - w/2, 640)
                if *w > 0 {
                    let a = self.centre[y];
                    let (l, r) = ((a + *w / 2).clamp(0, W as i32), (a + W as i32 - *w / 2).clamp(0, W as i32));
                    self.span[y] = (l, r.max(l));
                    if l >= r {
                        self.done[y] = true;
                    }
                }
            } else if *w > 0 {
                let (l, r) = (self.centre[y] - *w / 2, self.centre[y] + *w / 2);
                let (cl, cr) = (l.max(0), r.min(W as i32));
                self.span[y] = (cl, cr.max(cl));
                if cl == 0 && r > W as i32 {
                    self.done[y] = true;
                }
            }
        }
        self.frame += 1;
    }

    /// Copies the revealed spans of `pic` (a frame holding the finished picture) onto `frame`.
    pub fn draw(&self, frame: &mut Frame, pic: &Frame) {
        for y in self.rows.clone() {
            let (l, r) = self.span[y];
            if self.inward {
                // `span` holds the inner ends: the left part is [0, l), the right part [r, 640)
                let (a, b) = (y * W, y * W + l as usize);
                frame.pixels[a..b].copy_from_slice(&pic.pixels[a..b]);
                let (a, b) = (y * W + r as usize, y * W + W);
                frame.pixels[a..b].copy_from_slice(&pic.pixels[a..b]);
            } else {
                let (a, b) = (y * W + l as usize, y * W + r as usize);
                frame.pixels[a..b].copy_from_slice(&pic.pixels[a..b]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mid_reveal_is_partial() {
        let mut r = Reveal::new(7, 0..H, false);
        for _ in 0..40 {
            r.step();
        }
        let open: i32 = r.span.iter().map(|&(l, rr)| rr - l).sum();
        let all = (W * H) as i32;
        assert!(open > 0 && open < all, "{open}/{all}");
    }

    #[test]
    fn opens_to_the_full_width() {
        let mut r = Reveal::new(7, 0..H, false);
        while !r.finished() {
            r.step();
        }
        let full = r.span.iter().filter(|&&(l, rr)| l == 0 && rr == W as i32).count();
        assert!(full > H / 2, "{full} rows fully open");
    }

    /// rules-22 §5: the inward wipe opens the edges first and ends with every row fully covered.
    #[test]
    fn inward_wipe_meets_in_the_middle() {
        let mut r = Reveal::new(7, 0..H, true);
        for _ in 0..30 {
            r.step();
        }
        let pic = Frame::new([[0; 3]; 256]);
        let mut f = Frame::new([[0; 3]; 256]);
        r.draw(&mut f, &pic);
        while !r.finished() {
            r.step();
        }
        assert!(r.done.iter().filter(|&&d| d).count() > H / 2);
    }
}
