//! Colours. 8-bit graphics use one 256-entry VGA palette stored in `MITTELL.EXE`
//! (the graphics init uploads it with INT 10h AX=1012h); 16-bit graphics are RGB555.

use crate::{FormatError, err, le::LeExe};

/// Linear address of the 768-byte, 6-bit-per-channel palette in the shipped `MITTELL.EXE` (796945 bytes).
pub const EXE_PALETTE_ADDR: u32 = 0x8383D;

pub type Palette = [[u8; 3]; 256];

fn six_to_eight(c: u8) -> u8 {
    (c << 2) | (c >> 4)
}

/// Reads the game palette from the executable, expanded to 8 bits per channel.
pub fn from_exe(exe: &[u8]) -> Result<Palette, FormatError> {
    let le = LeExe::parse(exe)?;
    let Some(raw) = le.read(EXE_PALETTE_ADDR, 768) else { return err("palette address not mapped", 0) };
    // Sanity: VGA DAC values, black first and white last in this build.
    if raw.iter().any(|&c| c > 63) || raw[..3] != [0, 0, 0] || raw[765..] != [63, 63, 63] {
        return err("no VGA palette at expected address (different EXE build?)", 0);
    }
    let mut p = [[0; 3]; 256];
    for (i, c) in p.iter_mut().enumerate() {
        *c = [0, 1, 2].map(|k| six_to_eight(raw[3 * i + k]));
    }
    Ok(p)
}

/// RGB555 (`0RRRRRGGGGGBBBBB`) to 8-bit RGB.
pub fn rgb555(v: u16) -> [u8; 3] {
    [(v >> 10) & 31, (v >> 5) & 31, v & 31].map(|c| ((c << 3) | (c >> 2)) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        assert_eq!(rgb555(0x7FFF), [255, 255, 255]);
        assert_eq!(rgb555(0x7C00), [255, 0, 0]);
        assert_eq!(rgb555(0x001F), [0, 0, 255]);
        assert_eq!(six_to_eight(63), 255);
        assert_eq!(six_to_eight(0), 0);
    }

    /// Mean channel error between the 8-bit and hicolor renditions of the same icons stays small
    /// (they are separately quantised, so it is not zero; measured ~14/255 on several full screens).
    #[test]
    fn real_palette_matches_hicolor_art() {
        let (Some(exe), Some(a), Some(b)) =
            (crate::data_file("MITTELL.EXE"), crate::data_file("ICONS2/ICON32.DAT"), crate::data_file("ICONS3/ICON32.DAT"))
        else {
            return;
        };
        let pal = from_exe(&exe).unwrap();
        let i8 = &crate::sprite::decode_icon_file(&a, 1).unwrap()[0];
        let i16 = &crate::sprite::decode_icon_file(&b, 2).unwrap()[0];
        let (mut sum, mut n) = (0u64, 0u64);
        for k in 0..i8.pixels.len().min(i16.pixels.len()) {
            if i8.opaque[k] && i16.opaque[k] {
                let (p, q) = (pal[i8.pixels[k] as usize], rgb555(i16.pixels[k]));
                sum += (0..3).map(|c| (p[c] as i32 - q[c] as i32).unsigned_abs() as u64).sum::<u64>();
                n += 3;
            }
        }
        assert!(n > 0 && sum / n < 32, "mean error {}", sum / n.max(1));
    }
}
