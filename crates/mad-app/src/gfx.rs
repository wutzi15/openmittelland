//! 640x480 framebuffer (palette indices, or RGB555 in hicolor mode), sprite blitting, and window scaling math.

use mad_assets::{palette::Palette, sprite::Image};

pub const W: usize = 640;
pub const H: usize = 480;

pub struct Frame {
    /// Palette indices, or `0RRRRRGGGGGBBBBB` when `hicolor` (video mode 3, `ICONS3`).
    pub pixels: Vec<u16>,
    pub palette: Palette,
    pub hicolor: bool,
}

/// RGB555 of an 8-bit palette entry.
pub fn rgb555([r, g, b]: [u8; 3]) -> u16 {
    ((r as u16 >> 3) << 10) | ((g as u16 >> 3) << 5) | (b as u16 >> 3)
}

/// 0x00RRGGBB of an RGB555 value (low bits replicated).
pub fn rgb555_to_u32(v: u16) -> u32 {
    let e = |c: u16| ((c << 3) | (c >> 2)) as u32;
    e(v >> 10 & 31) << 16 | e(v >> 5 & 31) << 8 | e(v & 31)
}

impl Frame {
    pub fn new(palette: Palette) -> Self {
        Self { pixels: vec![0; W * H], palette, hicolor: false }
    }

    /// Starts a screen drawn in the mode of `assets`: clears, takes its palette and colour depth.
    pub fn begin(&mut self, assets: &crate::assets::Assets) {
        self.palette = assets.palette;
        self.hicolor = assets.hicolor;
        self.pixels.fill(0);
    }

    /// Solid rectangle `[x, y, w, h]` in the palette entry nearest `rgb` (or exact in hicolor).
    pub fn fill_rgb(&mut self, [x, y, w, h]: [i32; 4], rgb: [u8; 3]) {
        let v = if self.hicolor {
            rgb555(rgb)
        } else {
            let d = |c: &[u8; 3]| (0..3).map(|i| (c[i] as i32 - rgb[i] as i32).pow(2)).sum::<i32>();
            (0..256).min_by_key(|&i| d(&self.palette[i])).unwrap_or(0) as u16
        };
        for py in y.max(0)..(y + h).min(H as i32) {
            for px in x.max(0)..(x + w).min(W as i32) {
                self.pixels[py as usize * W + px as usize] = v;
            }
        }
    }


    /// Draws an 8-bit (palette index) image in either mode: through the palette in hicolor.
    pub fn blit_idx(&mut self, img: &Image, x: i32, y: i32) {
        if !self.hicolor {
            return self.blit(img, x, y);
        }
        let conv = Image { pixels: img.pixels.iter().map(|&p| rgb555(self.palette[p as usize & 255])).collect(), ..img.clone() };
        self.blit(&conv, x, y);
    }


    /// Draws an 8-bit image with its top-left at (x, y), clipped, transparent where not opaque.
    pub fn blit(&mut self, img: &Image, x: i32, y: i32) {
        self.blit_clip(img, x, y, false, false, [0, 0, W as i32, H as i32]);
    }

    /// Blit mirrored horizontally (`fx`) and/or vertically (`fy`), clipped to `[x0, y0, x1, y1)`
    /// (which must lie on screen).
    pub fn blit_clip(&mut self, img: &Image, x: i32, y: i32, fx: bool, fy: bool, [x0, y0, x1, y1]: [i32; 4]) {
        for sy in 0..img.height {
            let dy = y + sy as i32;
            if !(y0..y1).contains(&dy) {
                continue;
            }
            let ry = if fy { img.height - 1 - sy } else { sy };
            for sx in 0..img.width {
                let dx = x + sx as i32;
                let rx = if fx { img.width - 1 - sx } else { sx };
                let i = ry * img.width + rx;
                if (x0..x1).contains(&dx) && img.opaque[i] {
                    self.pixels[dy as usize * W + dx as usize] = img.pixels[i];
                }
            }
        }
    }

    /// Blitter colour mode 1..0x7F (`formats.md` §8.4): every opaque pixel in palette index `c`.
    #[allow(dead_code)] // drawing aids for fidelity work
    pub fn blit_solid(&mut self, img: &Image, x: i32, y: i32, c: u8) {
        let v = if self.hicolor { rgb555(self.palette[c as usize]) } else { c as u16 };
        let solid = Image { pixels: vec![v; img.pixels.len()], ..img.clone() };
        self.blit(&solid, x, y);
    }

    /// Every opaque pixel in the colour `rgb` (the palette entry nearest to it in 8-bit mode).
    pub fn blit_rgb_solid(&mut self, img: &Image, x: i32, y: i32, rgb: [u8; 3]) {
        let v = if self.hicolor {
            rgb555(rgb)
        } else {
            let d = |c: &[u8; 3]| (0..3).map(|i| (c[i] as i32 - rgb[i] as i32).pow(2)).sum::<i32>();
            (0..256).min_by_key(|&i| d(&self.palette[i])).unwrap_or(0) as u16
        };
        let solid = Image { pixels: vec![v; img.pixels.len()], ..img.clone() };
        self.blit(&solid, x, y);
    }

    /// 1-px rectangle outline `[x, y, w, h]`, clipped.
    #[allow(dead_code)]
    pub fn outline(&mut self, [x, y, w, h]: [i32; 4], c: u8) {
        let c = if self.hicolor { rgb555(self.palette[c as usize]) } else { c as u16 };
        let mut put = |px: i32, py: i32| {
            if (0..W as i32).contains(&px) && (0..H as i32).contains(&py) {
                self.pixels[py as usize * W + px as usize] = c;
            }
        };
        for i in 0..w {
            put(x + i, y);
            put(x + i, y + h - 1);
        }
        for j in 0..h {
            put(x, y + j);
            put(x + w - 1, y + j);
        }
    }

    /// Converts to 0x00RRGGBB, scaled into a `dw`x`dh` buffer at `r` (nearest neighbour); the rest is black.
    pub fn present(&self, out: &mut [u32], dw: usize, dh: usize) {
        if dw == 0 || dh == 0 {
            return;
        }
        let lut: Vec<u32> = if self.hicolor {
            (0..32768u16).map(rgb555_to_u32).collect()
        } else {
            self.palette.iter().map(|&[r, g, b]| u32::from_be_bytes([0, r, g, b])).collect()
        };
        let r = fit(dw, dh);
        out.fill(0);
        for y in 0..r.h {
            let sy = y * H / r.h;
            let row = &mut out[(r.y + y) * dw + r.x..][..r.w];
            for (x, o) in row.iter_mut().enumerate() {
                *o = lut[self.pixels[sy * W + x * W / r.w] as usize];
            }
        }
    }

    pub fn to_rgb(&self) -> Vec<u8> {
        if self.hicolor {
            return self.pixels.iter().flat_map(|&v| rgb555_to_u32(v).to_be_bytes()[1..].to_vec()).collect();
        }
        self.pixels.iter().flat_map(|&i| self.palette[i as usize]).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

/// Where the 640x480 picture goes in a `dw`x`dh` window: largest integer scale that fits, centred;
/// below 1x, the largest 4:3 rectangle that fits.
pub fn fit(dw: usize, dh: usize) -> Rect {
    let s = (dw / W).min(dh / H);
    let (w, h) = if s >= 1 { (W * s, H * s) } else if dw * H < dh * W { (dw, dw * H / W) } else { (dh * W / H, dh) };
    let (w, h) = (w.max(1).min(dw.max(1)), h.max(1).min(dh.max(1)));
    Rect { x: (dw - w.min(dw)) / 2, y: (dh - h.min(dh)) / 2, w, h }
}

/// Window pixel to framebuffer pixel; `None` outside the picture.
pub fn to_frame(r: Rect, px: f64, py: f64) -> Option<(i32, i32)> {
    let (x, y) = (px - r.x as f64, py - r.y as f64);
    if x < 0.0 || y < 0.0 || x >= r.w as f64 || y >= r.h as f64 {
        return None;
    }
    Some(((x * W as f64 / r.w as f64) as i32, (y * H as f64 / r.h as f64) as i32))
}

pub fn write_screenshot(path: &str, frame: &Frame) -> std::io::Result<()> {
    let rgb = frame.to_rgb();
    if path.to_ascii_lowercase().ends_with(".png") {
        let mut enc = png::Encoder::new(std::fs::File::create(path)?, W as u32, H as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.write_header().and_then(|mut w| w.write_image_data(&rgb)).map_err(std::io::Error::other)
    } else {
        let mut out = format!("P6\n{W} {H}\n255\n").into_bytes();
        out.extend(rgb);
        std::fs::write(path, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_integer_and_fractional() {
        assert_eq!(fit(640, 480), Rect { x: 0, y: 0, w: 640, h: 480 });
        assert_eq!(fit(1920, 1080), Rect { x: 320, y: 60, w: 1280, h: 960 });
        assert_eq!(fit(1300, 1000), Rect { x: 10, y: 20, w: 1280, h: 960 });
        assert_eq!(fit(320, 480), Rect { x: 0, y: 120, w: 320, h: 240 });
        assert_eq!(fit(0, 0).w, 1);
    }

    #[test]
    fn window_to_frame() {
        let r = fit(1920, 1080);
        assert_eq!(to_frame(r, 320.0, 60.0), Some((0, 0)));
        assert_eq!(to_frame(r, 1599.9, 1019.9), Some((639, 479)));
        assert_eq!(to_frame(r, 319.0, 500.0), None);
        assert_eq!(to_frame(r, 1600.0, 500.0), None);
    }

    #[test]
    fn blit_clips_and_flips() {
        let mut f = Frame::new([[0; 3]; 256]);
        let img = Image { width: 2, height: 1, pixels: vec![1, 2], opaque: vec![true, false] };
        f.blit(&img, -1, 0);
        f.blit(&img, 638, 479);
        f.blit_clip(&img, 10, 10, true, false, [0, 0, 640, 480]);
        f.blit_clip(&img, 20, 10, true, false, [0, 0, 21, 480]);
        assert_eq!(&f.pixels[..2], &[0, 0]);
        assert_eq!(f.pixels[479 * W + 638], 1);
        assert_eq!((f.pixels[10 * W + 10], f.pixels[10 * W + 11]), (0, 1));
        assert_eq!(f.pixels[10 * W + 21], 0);
    }

    #[test]
    fn hicolor_conversion_and_indexed_blit() {
        assert_eq!(rgb555([255, 0, 8]), 0b11111_00000_00001);
        assert_eq!(rgb555_to_u32(0b11111_00000_00001), 0x00FF0008);
        let mut pal = [[0u8; 3]; 256];
        pal[7] = [248, 8, 0];
        let mut f = Frame::new(pal);
        f.hicolor = true;
        f.blit_idx(&Image { width: 1, height: 1, pixels: vec![7], opaque: vec![true] }, 3, 0);
        assert_eq!(f.pixels[3], rgb555(pal[7]));
        let mut out = vec![0u32; 640 * 480];
        f.present(&mut out, 640, 480);
        assert_eq!(out[3], 0x00FF0800);
    }
}
