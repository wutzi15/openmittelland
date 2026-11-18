//! Autodesk FLI/FLC animation decoder (public, documented format).

use std::fmt;

#[derive(Debug)]
pub enum FlcError {
    Truncated,
    BadMagic(u16),
    UnsupportedDepth(u16),
}

impl fmt::Display for FlcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => write!(f, "truncated FLC data"),
            Self::BadMagic(m) => write!(f, "bad FLC magic {m:#06x}"),
            Self::UnsupportedDepth(d) => write!(f, "unsupported FLC depth {d}"),
        }
    }
}
impl std::error::Error for FlcError {}

pub struct Flc<'a> {
    data: &'a [u8],
    pub width: usize,
    pub height: usize,
    pub frames: usize,
    /// Milliseconds per frame.
    pub speed_ms: u32,
    first: usize,
    /// Next frame offset, and index of the frame it holds.
    pos: usize,
    next: usize,
    pub palette: [[u8; 3]; 256],
    pub pixels: Vec<u8>,
}

fn u16le(d: &[u8], o: usize) -> Result<u16, FlcError> {
    d.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]])).ok_or(FlcError::Truncated)
}
fn u32le(d: &[u8], o: usize) -> Result<u32, FlcError> {
    d.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).ok_or(FlcError::Truncated)
}

impl<'a> Flc<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, FlcError> {
        let magic = u16le(data, 4)?;
        if magic != 0xAF12 && magic != 0xAF11 {
            return Err(FlcError::BadMagic(magic));
        }
        let depth = u16le(data, 12)?;
        if depth != 8 {
            return Err(FlcError::UnsupportedDepth(depth));
        }
        let (w, h) = (u16le(data, 8)? as usize, u16le(data, 10)? as usize);
        let raw_speed = u32le(data, 16)?;
        // FLI (0xAF11) speed is in 1/70 s jiffies; FLC is in ms.
        let speed_ms = if magic == 0xAF11 { raw_speed * 1000 / 70 } else { raw_speed };
        let first = if magic == 0xAF12 { u32le(data, 80)? as usize } else { 128 };
        Ok(Self {
            data,
            width: w,
            height: h,
            frames: u16le(data, 6)? as usize,
            speed_ms,
            first,
            pos: first,
            next: first,
            palette: [[0; 3]; 256],
            pixels: vec![0; w * h],
        })
    }

    pub fn rewind(&mut self) {
        self.pos = self.first;
        self.next = self.first;
    }

    /// Decode the next frame into `pixels`/`palette`. Returns false at end of animation.
    pub fn step(&mut self) -> Result<bool, FlcError> {
        if self.next + 16 > self.data.len() {
            return Ok(false);
        }
        self.pos = self.next;
        let d = self.data;
        let fsize = u32le(d, self.pos)? as usize;
        let ftype = u16le(d, self.pos + 4)?;
        let nchunks = u16le(d, self.pos + 6)? as usize;
        self.next = self.pos + fsize;
        if ftype != 0xF1FA {
            return Ok(self.step_skip()); // PREFIX/other frame types carry no pixels
        }
        let mut c = self.pos + 16;
        for _ in 0..nchunks {
            let csize = u32le(d, c)? as usize;
            let ctype = u16le(d, c + 4)?;
            let body = d.get(c + 6..c + csize.max(6)).ok_or(FlcError::Truncated)?;
            match ctype {
                4 => self.color(body, 0)?,
                11 => self.color(body, 2)?,
                7 => self.delta_flc(body)?,
                12 => self.delta_fli(body)?,
                13 => self.pixels.fill(0),
                15 => self.byte_run(body)?,
                16 => {
                    let n = self.pixels.len().min(body.len());
                    self.pixels[..n].copy_from_slice(&body[..n]);
                }
                _ => {} // 18 = postage stamp, others ignored
            }
            c += csize;
        }
        Ok(true)
    }

    fn step_skip(&mut self) -> bool {
        true
    }

    fn color(&mut self, b: &[u8], shift: u32) -> Result<(), FlcError> {
        let packets = u16le(b, 0)? as usize;
        let (mut p, mut idx) = (2, 0usize);
        for _ in 0..packets {
            idx += *b.get(p).ok_or(FlcError::Truncated)? as usize;
            let mut n = *b.get(p + 1).ok_or(FlcError::Truncated)? as usize;
            if n == 0 {
                n = 256;
            }
            p += 2;
            for i in 0..n {
                let rgb = b.get(p..p + 3).ok_or(FlcError::Truncated)?;
                // COLOR_64 values are 6-bit (VGA DAC); scale like a VGA palette (<<2 | >>4).
                self.palette[(idx + i) & 255] = if shift == 2 {
                    [rgb[0] << 2 | rgb[0] >> 4, rgb[1] << 2 | rgb[1] >> 4, rgb[2] << 2 | rgb[2] >> 4]
                } else {
                    [rgb[0], rgb[1], rgb[2]]
                };
                p += 3;
            }
            idx += n;
        }
        Ok(())
    }

    fn byte_run(&mut self, b: &[u8]) -> Result<(), FlcError> {
        let (w, h) = (self.width, self.height);
        let mut p = 0;
        for y in 0..h {
            p += 1; // packet count byte is obsolete
            let mut x = 0;
            while x < w {
                let c = *b.get(p).ok_or(FlcError::Truncated)? as i8;
                p += 1;
                if c >= 0 {
                    let v = *b.get(p).ok_or(FlcError::Truncated)?;
                    p += 1;
                    let n = (c as usize).min(w - x);
                    self.pixels[y * w + x..y * w + x + n].fill(v);
                    x += c as usize;
                } else {
                    let n = (-(c as i16)) as usize;
                    let src = b.get(p..p + n).ok_or(FlcError::Truncated)?;
                    let m = n.min(w - x);
                    self.pixels[y * w + x..y * w + x + m].copy_from_slice(&src[..m]);
                    p += n;
                    x += n;
                }
            }
        }
        Ok(())
    }

    fn delta_fli(&mut self, b: &[u8]) -> Result<(), FlcError> {
        let w = self.width;
        let y0 = u16le(b, 0)? as usize;
        let lines = u16le(b, 2)? as usize;
        let mut p = 4;
        for y in y0..y0 + lines {
            let packets = *b.get(p).ok_or(FlcError::Truncated)?;
            p += 1;
            let mut x = 0;
            for _ in 0..packets {
                x += *b.get(p).ok_or(FlcError::Truncated)? as usize;
                let c = *b.get(p + 1).ok_or(FlcError::Truncated)? as i8;
                p += 2;
                if c >= 0 {
                    let src = b.get(p..p + c as usize).ok_or(FlcError::Truncated)?;
                    self.pixels[y * w + x..y * w + x + src.len()].copy_from_slice(src);
                    p += c as usize;
                    x += c as usize;
                } else {
                    let n = (-(c as i16)) as usize;
                    let v = *b.get(p).ok_or(FlcError::Truncated)?;
                    p += 1;
                    self.pixels[y * w + x..y * w + x + n].fill(v);
                    x += n;
                }
            }
        }
        Ok(())
    }

    fn delta_flc(&mut self, b: &[u8]) -> Result<(), FlcError> {
        let w = self.width;
        let lines = u16le(b, 0)? as usize;
        let mut p = 2;
        let mut y = 0usize;
        let mut done = 0;
        while done < lines {
            let op = u16le(b, p)?;
            p += 2;
            match op >> 14 {
                0 => {
                    // packet count for this line
                    let mut x = 0;
                    for _ in 0..op {
                        x += *b.get(p).ok_or(FlcError::Truncated)? as usize;
                        let c = *b.get(p + 1).ok_or(FlcError::Truncated)? as i8;
                        p += 2;
                        if c >= 0 {
                            let n = c as usize * 2;
                            let src = b.get(p..p + n).ok_or(FlcError::Truncated)?;
                            self.pixels[y * w + x..y * w + x + n].copy_from_slice(src);
                            p += n;
                            x += n;
                        } else {
                            let n = (-(c as i16)) as usize * 2;
                            let v = b.get(p..p + 2).ok_or(FlcError::Truncated)?;
                            p += 2;
                            for i in 0..n {
                                self.pixels[y * w + x + i] = v[i & 1];
                            }
                            x += n;
                        }
                    }
                    y += 1;
                    done += 1;
                }
                3 => y += (0u16.wrapping_sub(op)) as usize, // skip lines
                2 => {
                    // last-pixel-of-line byte
                    if y < self.height {
                        self.pixels[y * w + w - 1] = (op & 0xFF) as u8;
                    }
                }
                _ => return Err(FlcError::Truncated),
            }
        }
        Ok(())
    }

    /// RGBA (0xAARRGGBB-packed u32) view of the current frame.
    pub fn to_rgb32(&self, out: &mut [u32]) {
        for (o, &i) in out.iter_mut().zip(&self.pixels) {
            let [r, g, b] = self.palette[i as usize];
            *o = u32::from_be_bytes([0, r, g, b]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-built 4x2 FLC with one BYTE_RUN frame and a COLOR_256 chunk.
    fn tiny() -> Vec<u8> {
        let mut pal = vec![];
        pal.extend_from_slice(&1u16.to_le_bytes()); // 1 packet
        pal.extend_from_slice(&[1, 2, 10, 20, 30, 40, 50, 60]); // skip 1, 2 colors
        let run = vec![1, 4, 7, 1, 4, 9]; // two lines, each one RLE packet
        let chunk = |t: u16, b: &[u8]| {
            let mut v = ((b.len() + 6) as u32).to_le_bytes().to_vec();
            v.extend_from_slice(&t.to_le_bytes());
            v.extend_from_slice(b);
            v
        };
        let mut chunks = chunk(4, &pal);
        chunks.extend(chunk(15, &run));
        let mut frame = ((16 + chunks.len()) as u32).to_le_bytes().to_vec();
        frame.extend_from_slice(&0xF1FAu16.to_le_bytes());
        frame.extend_from_slice(&2u16.to_le_bytes());
        frame.extend_from_slice(&[0; 8]);
        frame.extend(chunks);
        let mut h = vec![0u8; 128];
        h[4..6].copy_from_slice(&0xAF12u16.to_le_bytes());
        h[6..8].copy_from_slice(&1u16.to_le_bytes());
        h[8..10].copy_from_slice(&4u16.to_le_bytes());
        h[10..12].copy_from_slice(&2u16.to_le_bytes());
        h[12..14].copy_from_slice(&8u16.to_le_bytes());
        h[16..20].copy_from_slice(&40u32.to_le_bytes());
        h[80..84].copy_from_slice(&128u32.to_le_bytes());
        h.extend(frame);
        h
    }

    #[test]
    fn decodes_byte_run_and_palette() {
        let data = tiny();
        let mut f = Flc::new(&data).unwrap();
        assert!(f.step().unwrap());
        assert_eq!(f.pixels, [7, 7, 7, 7, 9, 9, 9, 9]);
        assert_eq!(f.palette[1], [10, 20, 30]);
        assert_eq!(f.palette[2], [40, 50, 60]);
        assert!(!f.step().unwrap());
    }

    #[test]
    fn rejects_bad_magic() {
        assert!(matches!(Flc::new(&[0; 128]), Err(FlcError::BadMagic(0))));
    }
}
