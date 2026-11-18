//! `ICONS2/ICONCS.DAT`, `ICONS3/ICONCS.DAT`: the proportional Fraktur font.
//! Layout: `u8 advance[256]`, `u32 glyph_offset[256]` (relative to 0x500), then RLE glyphs
//! (same encoding as [`crate::sprite`]). Codes 0x00-0x7F are a gold rendition, 0x80-0xFF the
//! same glyphs dark; see `formats.md` for the code-to-character map.

use crate::{FormatError, err, sprite::{Bpp, Image, decode_rle}, u32le};

const GLYPHS_AT: usize = 0x500;

pub struct Font {
    /// Horizontal advance in pixels. Some glyph bitmaps are 1 px wider than their advance (8-bit set).
    pub advance: [u8; 256],
    pub glyphs: Vec<Image>,
}

impl Font {
    pub fn parse(d: &[u8], bpp: Bpp) -> Result<Self, FormatError> {
        if d.len() < GLYPHS_AT {
            return err("font header truncated", d.len());
        }
        let mut advance = [0; 256];
        advance.copy_from_slice(&d[..256]);
        let mut glyphs = Vec::with_capacity(256);
        for i in 0..256 {
            let (img, end) = decode_rle(d, GLYPHS_AT + u32le(d, 256 + 4 * i)? as usize, bpp)?;
            let next = if i < 255 { GLYPHS_AT + u32le(d, 256 + 4 * (i + 1))? as usize } else { d.len() };
            if end != next {
                return err("glyph length mismatch", end);
            }
            glyphs.push(img);
        }
        Ok(Self { advance, glyphs })
    }
}

/// CP437 text byte (as stored in `MITTELL.EXE`) to glyph code, gold variant; OR 0x80 for the dark one.
/// Umlaut mapping read off the rendered glyph sheet; other non-ASCII bytes map to `None`.
pub fn glyph_for_cp437(b: u8) -> Option<u8> {
    Some(match b {
        // `^` and `_` draw the left/right ornaments used around titles ("^ Leer _").
        0x20..=0x5A | 0x5E | 0x5F | 0x61..=0x7A => b,
        // status circles (broken, plain, crossed) of the order lines
        0x0D..=0x0F => b,
        // small digits (`%B`, `$18c` = small ² of "m²")
        0x10..=0x19 => b,
        // no-break space: blank like the space, but word wrap does not break there
        0xFF => 0xA0,
        0x8E => 0x5B, // Ä
        0x99 => 0x5C, // Ö
        0x9A => 0x5D, // Ü
        0x82 => 0x1E, // é (glyph sheet row 1; only "Exposé")
        0x84 => 0x7B, // ä
        0x94 => 0x7C, // ö
        0x81 => 0x7D, // ü
        0xE1 => 0x7F, // ß
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fixture() {
        let mut d = vec![0u8; GLYPHS_AT];
        d[b'A' as usize] = 3;
        // glyph 0..=255 all empty (single 0xFF) except 'A' = 2 px
        let mut body = vec![];
        for i in 0..256usize {
            d[256 + 4 * i..260 + 4 * i].copy_from_slice(&(body.len() as u32).to_le_bytes());
            if i == b'A' as usize { body.extend([0, 2, 9, 9, 0xFF]) } else { body.push(0xFF) }
        }
        d.extend(body);
        let f = Font::parse(&d, 1).unwrap();
        assert_eq!(f.advance[b'A' as usize], 3);
        assert_eq!(f.glyphs[b'A' as usize].pixels, [9, 9]);
        assert_eq!(glyph_for_cp437(0x81), Some(0x7D));
    }

    #[test]
    fn real_fonts() {
        for (p, bpp) in [("ICONS2/ICONCS.DAT", 1), ("ICONS3/ICONCS.DAT", 2)] {
            let Some(d) = crate::data_file(p) else { return };
            let f = Font::parse(&d, bpp).unwrap();
            assert_eq!(f.glyphs.len(), 256);
            assert!(f.glyphs.iter().all(|g| g.height == 24 || g.height == 28));
            assert!(f.glyphs[b'A' as usize].width > 0);
        }
    }
}
