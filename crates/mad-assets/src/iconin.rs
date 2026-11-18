//! `ICONS2/ICONIN.DAT`, `ICONS3/ICONIN.DAT`: per-icon index, 512 entries, 4608 bytes.
//! Layout: `u32 size[512]`, `u8 flags[512]`, `u16 width[512]`, `u16 height[512]`.

use crate::{FormatError, err, u16le, u32le};

pub const ENTRIES: usize = 512;
/// File lives in the `B` directory (`ICONS2B`/`ICONS3B`), which a CD install reads from the CD.
pub const FLAG_B_DIR: u8 = 0x80;
/// Set on every raw (`w,h` header) file. [UNVERIFIED] whether the game reads it.
pub const FLAG_RAW: u8 = 0x01;
/// Set on most bank (offset table) files, see `formats.md` for exceptions. [UNVERIFIED] meaning.
pub const FLAG_2: u8 = 0x02;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Entry {
    pub size: u32,
    pub flags: u8,
    pub width: u16,
    pub height: u16,
}

pub struct IconIndex {
    pub entries: [Entry; ENTRIES],
}

impl IconIndex {
    pub fn parse(d: &[u8]) -> Result<Self, FormatError> {
        if d.len() != 9 * ENTRIES {
            return err("ICONIN.DAT must be 4608 bytes", d.len());
        }
        let mut entries = [Entry::default(); ENTRIES];
        for (i, e) in entries.iter_mut().enumerate() {
            *e = Entry {
                size: u32le(d, 4 * i)?,
                flags: d[0x800 + i],
                width: u16le(d, 0xA00 + 2 * i)?,
                height: u16le(d, 0xE00 + 2 * i)?,
            };
        }
        Ok(Self { entries })
    }
}

const DIGITS: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Icon id -> file name: high digit base 36 (`id / 16`), low digit hex. `0x1B9` -> `ICONR9.DAT`.
pub fn file_name(id: u16) -> String {
    let (hi, lo) = (DIGITS[(id as usize / 16) % 36] as char, DIGITS[id as usize % 16] as char);
    format!("ICON{hi}{lo}.DAT")
}

/// Inverse of [`file_name`]; `None` for `ICONCS`, `ICONIN` and anything else not of that shape.
pub fn id_from_name(name: &str) -> Option<u16> {
    let b = name.to_ascii_uppercase().into_bytes();
    if b.len() != 10 || &b[..4] != b"ICON" || &b[6..] != b".DAT" {
        return None;
    }
    let hi = DIGITS.iter().position(|&c| c == b[4])?;
    let lo = DIGITS[..16].iter().position(|&c| c == b[5])?;
    let id = hi * 16 + lo;
    (id < ENTRIES).then_some(id as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        assert_eq!(file_name(0x22), "ICON22.DAT");
        assert_eq!(file_name(27 * 16 + 9), "ICONR9.DAT");
        assert_eq!(id_from_name("ICONR9.DAT"), Some(441));
        assert_eq!(id_from_name("ICONCS.DAT"), None);
        for id in 0..ENTRIES as u16 {
            assert_eq!(id_from_name(&file_name(id)), Some(id));
        }
    }

    #[test]
    fn parse_fixture() {
        let mut d = vec![0u8; 4608];
        d[4..8].copy_from_slice(&1604u32.to_le_bytes());
        d[0x801] = FLAG_RAW;
        d[0xA02] = 40;
        d[0xE02] = 40;
        let ix = IconIndex::parse(&d).unwrap();
        assert_eq!(ix.entries[1], Entry { size: 1604, flags: 1, width: 40, height: 40 });
        assert!(IconIndex::parse(&d[1..]).is_err());
    }

    /// The 8-bit index matches every file's size and directory.
    #[test]
    fn real_index_matches_files() {
        let Some(d) = crate::data_file("ICONS2/ICONIN.DAT") else { return };
        let root = std::env::var("MAD_DATA").unwrap();
        let ix = IconIndex::parse(&d).unwrap();
        let mut n = 0;
        for (id, e) in ix.entries.iter().enumerate() {
            let dir = if e.flags & FLAG_B_DIR != 0 { "ICONS2B" } else { "ICONS2" };
            if let Ok(m) = std::fs::metadata(format!("{root}/{dir}/{}", file_name(id as u16))) {
                assert_eq!(m.len(), e.size as u64, "{id}");
                n += 1;
            } else {
                assert_eq!(e.size, 0, "{id} missing");
            }
        }
        assert_eq!(n, 442);
    }
}
