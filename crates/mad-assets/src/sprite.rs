//! `ICONS2*/ICONxx.DAT` (8-bit indexed) and `ICONS3*/ICONxx.DAT` (15-bit RGB555) bitmaps.
//!
//! Three containers, all sharing one pixel encoding per directory:
//! - raw: `u16 width, u16 height`, then `width*height` pixels;
//! - RLE: rows of `(skip, count, count pixels)` segments, `0xFE` ends a row, `0xFF` ends the image;
//! - bank: `u32` offset table (first offset = 4 * entry count), each entry an RLE image.

use crate::{FormatError, err, u16le, u32le};

/// Bytes per pixel: 1 for `ICONS2*` (palette index), 2 for `ICONS3*` (RGB555, little endian).
pub type Bpp = usize;

/// Decoded image. `pixels[i]` is a palette index (bpp 1) or an RGB555 value (bpp 2);
/// it is only meaningful where `opaque[i]` is set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u16>,
    pub opaque: Vec<bool>,
}

fn px(d: &[u8], o: usize, bpp: Bpp) -> u16 {
    if bpp == 2 { u16::from_le_bytes([d[o], d[o + 1]]) } else { d[o] as u16 }
}

/// Raw format; succeeds only if the header size matches the data length exactly.
pub fn decode_raw(d: &[u8], bpp: Bpp) -> Result<Image, FormatError> {
    let (w, h) = (u16le(d, 0)? as usize, u16le(d, 2)? as usize);
    if 4 + w * h * bpp != d.len() {
        return err("raw size mismatch", 0);
    }
    let pixels = (0..w * h).map(|i| px(d, 4 + i * bpp, bpp)).collect();
    Ok(Image { width: w, height: h, pixels, opaque: vec![true; w * h] })
}

/// Walks one RLE image at `off`, calling `f(x, y, value)` per pixel. Returns (width, height, end offset).
/// Width is the furthest pixel or skip extent seen; the original's nominal size lives in `ICONIN.DAT`.
fn walk(d: &[u8], off: usize, bpp: Bpp, mut f: impl FnMut(usize, usize, u16)) -> Result<(usize, usize, usize), FormatError> {
    let (mut i, mut x, mut y, mut w) = (off, 0, 0, 0);
    loop {
        match d.get(i) {
            None => return err("RLE image not terminated", i),
            Some(0xFF) => return Ok((w.max(x), y + 1, i + 1)),
            Some(0xFE) => {
                w = w.max(x);
                (x, y, i) = (0, y + 1, i + 1);
            }
            Some(&skip) => {
                let Some(&n) = d.get(i + 1) else { return err("truncated RLE segment", i) };
                let n = n as usize;
                if i + 2 + n * bpp > d.len() {
                    return err("truncated RLE pixels", i);
                }
                x += skip as usize;
                for k in 0..n {
                    f(x + k, y, px(d, i + 2 + k * bpp, bpp));
                }
                x += n;
                i += 2 + n * bpp;
            }
        }
    }
}

/// One RLE image at `off`. Returns the image and the offset just past its `0xFF` terminator.
pub fn decode_rle(d: &[u8], off: usize, bpp: Bpp) -> Result<(Image, usize), FormatError> {
    let (w, h, end) = walk(d, off, bpp, |_, _, _| {})?;
    let mut img = Image { width: w, height: h, pixels: vec![0; w * h], opaque: vec![false; w * h] };
    walk(d, off, bpp, |x, y, v| {
        img.pixels[y * w + x] = v;
        img.opaque[y * w + x] = true;
    })?;
    Ok((img, end))
}

/// Bank: `u32` offsets, entry count = first offset / 4. Each entry must end exactly where the next begins.
pub fn decode_bank(d: &[u8], bpp: Bpp) -> Result<Vec<Image>, FormatError> {
    let first = u32le(d, 0)? as usize;
    if first == 0 || !first.is_multiple_of(4) || first > d.len() {
        return err("bad bank header", 0);
    }
    let offs: Vec<usize> = (0..first / 4).map(|k| u32le(d, 4 * k).map(|o| o as usize)).collect::<Result<_, _>>()?;
    let mut out = Vec::with_capacity(offs.len());
    for (k, &o) in offs.iter().enumerate() {
        let (img, end) = decode_rle(d, o, bpp)?;
        if end != offs.get(k + 1).copied().unwrap_or(d.len()) {
            return err("bank entry length mismatch", o);
        }
        out.push(img);
    }
    Ok(out)
}

/// Any `ICONxx.DAT` except `ICONCS` (font, see [`crate::font`]) and `ICONIN` (index).
/// Tries raw, then single RLE spanning the whole file, then bank.
pub fn decode_icon_file(d: &[u8], bpp: Bpp) -> Result<Vec<Image>, FormatError> {
    if let Ok(img) = decode_raw(d, bpp) {
        return Ok(vec![img]);
    }
    if let Ok((img, end)) = decode_rle(d, 0, bpp)
        && end == d.len()
    {
        return Ok(vec![img]);
    }
    decode_bank(d, bpp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_8bit() {
        let img = decode_raw(&[2, 0, 1, 0, 7, 9], 1).unwrap();
        assert_eq!((img.width, img.height, img.pixels.clone()), (2, 1, vec![7, 9]));
        assert!(decode_raw(&[2, 0, 1, 0, 7], 1).is_err());
    }

    #[test]
    fn rle_skip_rows_and_16bit() {
        // row0: skip 1, 2 px; row1: empty; row2: skip 0, 1 px, skip 1, 1 px
        let d = [1, 2, 5, 6, 0xFE, 0xFE, 0, 1, 7, 1, 1, 8, 0xFF];
        let (img, end) = decode_rle(&d, 0, 1).unwrap();
        assert_eq!(end, d.len());
        assert_eq!((img.width, img.height), (3, 3));
        assert_eq!(img.pixels, [0, 5, 6, 0, 0, 0, 7, 0, 8]);
        assert_eq!(img.opaque, [false, true, true, false, false, false, true, false, true]);
        let d16 = [0, 1, 0x34, 0x12, 0xFF];
        assert_eq!(decode_rle(&d16, 0, 2).unwrap().0.pixels, [0x1234]);
        assert!(decode_rle(&[0, 3, 1], 0, 1).is_err());
    }

    #[test]
    fn bank_and_autodetect() {
        let mut d = vec![8, 0, 0, 0, 13, 0, 0, 0];
        d.extend([0, 2, 1, 2, 0xFF]); // entry 0 at 8
        d.push(0xFF); // entry 1 at 13: empty 0x1 image
        let v = decode_icon_file(&d, 1).unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].pixels, [1, 2]);
        assert_eq!((v[1].width, v[1].height), (0, 1));
        d[4] = 12; // entry 0 no longer ends where entry 1 starts
        assert!(decode_bank(&d, 1).is_err());
    }

    /// Every icon file in the install decodes; every single image matches `ICONIN.DAT` dimensions.
    #[test]
    fn real_files() {
        let Ok(root) = std::env::var("MAD_DATA") else { return };
        for (dir, bpp) in [("ICONS2", 1), ("ICONS2B", 1), ("ICONS3", 2), ("ICONS3B", 2)] {
            let index = crate::iconin::IconIndex::parse(&std::fs::read(format!("{root}/{}/ICONIN.DAT", &dir[..6])).unwrap()).unwrap();
            for e in std::fs::read_dir(format!("{root}/{dir}")).unwrap() {
                let p = e.unwrap().path();
                let name = p.file_name().unwrap().to_str().unwrap().to_string();
                if !name.ends_with(".DAT") || name == "ICONIN.DAT" || name == "ICONCS.DAT" {
                    continue;
                }
                let d = std::fs::read(&p).unwrap();
                let id = crate::iconin::id_from_name(&name).unwrap();
                let entry = index.entries[id as usize];
                if d.len() as u32 != entry.size {
                    // Known install defects: 8-bit copies or a truncated file in the hicolor set.
                    eprintln!("skip {dir}/{name}: size {} != index {}", d.len(), entry.size);
                    continue;
                }
                let imgs = decode_icon_file(&d, bpp).unwrap_or_else(|e| panic!("{dir}/{name}: {e}"));
                // ICONC6 has a zero size in the index; 1-byte files are empty placeholders.
                if imgs.len() == 1 && entry.width > 0 && d.len() > 1 {
                    let i = &imgs[0];
                    assert!(i.width <= entry.width as usize && i.height == entry.height as usize, "{dir}/{name}");
                }
            }
        }
    }
}
