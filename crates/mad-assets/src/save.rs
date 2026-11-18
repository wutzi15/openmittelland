//! `CONFI2/GAMEn.DAT` saved games (`n` = slot digit). 79703 bytes: twelve memory blocks written back to
//! back with no header, in the order below. Block contents are game-rule state, described (as far as
//! known) in `formats.md`; this module only splits the file.

use crate::{FormatError, err};

/// (name, length, linear address of the block in `MITTELL.EXE`'s data object).
/// Names from re-rules' analysis (`docs/re/rules-00-map.md`).
pub const BLOCKS: [(&str, usize, u32); 12] = [
    ("names", 0x300, 0xE377C),
    ("persons", 0xBA00, 0xE3A7C),
    ("properties", 0x4800, 0xDC86C),
    ("record_heap", 0x2710, 0xE106C),
    ("year", 2, 0xEF4B8),
    ("current_person", 1, 0xEF4C6),
    ("phase", 1, 0xEF4C5),
    ("heap_free", 4, 0x85A18),
    ("heap_used", 4, 0x85A1C),
    ("cities", 0x578, 0x85F04),
    ("goods", 0xE1, 0x86532),
    ("globals", 0x4E2, 0x85A20),
];
pub const SIZE: usize = 79703;
/// `persons` holds 192 records of 248 bytes; the loader post-processes each of the 192 after reading.
pub const PERSONS: usize = 192;
pub const PERSON_SIZE: usize = 0xF8;
/// `names` holds 32 NUL-padded 24-byte strings. [UNVERIFIED] what they name.
pub const NAMES: usize = 32;

pub struct Save {
    pub raw: Vec<u8>,
}

impl Save {
    pub fn parse(d: &[u8]) -> Result<Self, FormatError> {
        if d.len() != SIZE {
            return err("GAMEn.DAT must be 79703 bytes", d.len());
        }
        Ok(Self { raw: d.to_vec() })
    }

    pub fn block(&self, i: usize) -> &[u8] {
        let start: usize = BLOCKS[..i].iter().map(|b| b.1).sum();
        &self.raw[start..start + BLOCKS[i].1]
    }

    pub fn person(&self, i: usize) -> &[u8] {
        &self.block(1)[i * PERSON_SIZE..(i + 1) * PERSON_SIZE]
    }

    pub fn name(&self, i: usize) -> String {
        self.block(0)[i * 24..(i + 1) * 24].iter().take_while(|&&b| b != 0).map(|&b| b as char).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_sums_to_file_size() {
        assert_eq!(BLOCKS.iter().map(|b| b.1).sum::<usize>(), SIZE);
        assert_eq!(PERSONS * PERSON_SIZE, BLOCKS[1].1);
        assert_eq!(NAMES * 24, BLOCKS[0].1);
        let mut d = vec![0u8; SIZE];
        d[24..27].copy_from_slice(b"Bob");
        d[0x300 + PERSON_SIZE] = 9;
        let s = Save::parse(&d).unwrap();
        assert_eq!(s.name(1), "Bob");
        assert_eq!(s.person(1)[0], 9);
        assert_eq!(s.block(11).len(), 0x4E2);
        assert!(Save::parse(&d[1..]).is_err());
    }

    /// `$MAD_SAVE` points at a `GAMEn.DAT` (saves are not in the shipped MITTELL directory).
    #[test]
    fn real_save() {
        let Some(d) = std::env::var("MAD_SAVE").ok().and_then(|p| std::fs::read(p).ok()) else { return };
        let s = Save::parse(&d).unwrap();
        assert!(!s.name(0).is_empty());
    }
}
