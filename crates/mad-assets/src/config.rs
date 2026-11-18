//! `CONFIG/CONFIG.DAT` (shipped default) and `CONFI2/CONFIG.DAT` (live): 633 bytes, read whole into
//! memory. Only the fields below are tied to code; the rest is documented as observed in `formats.md`.

use crate::{FormatError, err};

pub const SIZE: usize = 0x279;
/// Older 0x278-byte files lack the byte at 0x199; the game shifts 0x199.. up by one and sets it to 1.
pub const OLD_SIZE: usize = 0x278;

pub struct Config {
    pub raw: Vec<u8>,
}

impl Config {
    pub fn parse(d: &[u8]) -> Result<Self, FormatError> {
        match d.len() {
            SIZE => Ok(Self { raw: d.to_vec() }),
            OLD_SIZE => {
                let mut raw = d[..0x199].to_vec();
                raw.push(1);
                raw.extend_from_slice(&d[0x199..]);
                Ok(Self { raw })
            }
            n => err("CONFIG.DAT must be 632 or 633 bytes", n),
        }
    }

    /// Low bits of byte 0: video mode, passed to the mode setter (`0x12DFE`) and used as the icon
    /// directory digit (`ICONS<mode>`). 1 = VGA 320x200 (intro), 2 = VESA 101h 640x480x8 (`ICONS2`),
    /// 3 = VESA 110h 640x480x15 (`ICONS3`); 4/5 = VESA 111h/112h exist in the setter.
    /// In modes 2 and 3 the `ICONS?B` files come from [`Self::cd_path`] unless byte 0x26C / 0x26D
    /// ("B icons copied to disk") is set.
    pub fn video_mode(&self) -> u8 {
        self.raw[0] & 7
    }

    /// CD path at 0x19A, length byte at 0x262.
    pub fn cd_path(&self) -> String {
        let n = (self.raw[0x262] as usize).min(SIZE - 0x19A);
        self.raw[0x19A..0x19A + n].iter().map(|&b| b as char).collect()
    }

    /// Five 48-byte records at 0x00 whose bytes 1.. hold a NUL-terminated name (`"^ Leer _"` = empty).
    /// [UNVERIFIED] that these are the save-slot labels.
    pub fn slot_name(&self, k: usize) -> String {
        let s = &self.raw[k * 0x30 + 1..(k + 1) * 0x30];
        s.iter().take_while(|&&b| b != 0).map(|&b| b as char).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_and_old_format() {
        let mut d = vec![0u8; OLD_SIZE];
        d[0] = 3;
        d[1..4].copy_from_slice(b"Abc");
        d[0x199..0x199 + 3].copy_from_slice(b"E:\\"); // old file: path one byte earlier
        d[0x261] = 3;
        let c = Config::parse(&d).unwrap();
        assert_eq!((c.video_mode(), c.slot_name(0), c.cd_path()), (3, "Abc".into(), "E:\\".into()));
        assert_eq!(c.raw[0x199], 1);
        assert!(Config::parse(&[0; 10]).is_err());
    }

    #[test]
    fn real_default_config() {
        let Some(d) = crate::data_file("CONFIG/CONFIG.DAT") else { return };
        let c = Config::parse(&d).unwrap();
        assert_eq!(c.cd_path(), "E:\\MITTELL");
        assert!((0..5).all(|k| c.slot_name(k) == "^ Leer _"));
    }
}
