//! `SZENARIO.DAT`: 5 battlefield layouts, each a 46-row x 30-column byte grid (6900 bytes).
//! Cell bits as used by the battle set-up code: low 5 bits = cell code (compared against a
//! deployment code derived from unit type and side), high 3 bits = facing 0..7 along which a
//! unit line is laid out. Meaning of individual codes: see `formats.md` ([UNVERIFIED] beyond that).

use crate::{FormatError, err};

pub const SCENARIOS: usize = 5;
pub const ROWS: usize = 46;
pub const COLS: usize = 30;

pub struct Szenario {
    pub cells: Vec<u8>,
}

impl Szenario {
    pub fn parse(d: &[u8]) -> Result<Self, FormatError> {
        if d.len() != SCENARIOS * ROWS * COLS {
            return err("SZENARIO.DAT must be 6900 bytes", d.len());
        }
        Ok(Self { cells: d.to_vec() })
    }

    pub fn cell(&self, scenario: usize, row: usize, col: usize) -> u8 {
        self.cells[scenario * ROWS * COLS + row * COLS + col]
    }
}

pub fn code(cell: u8) -> u8 {
    cell & 0x1F
}
pub fn facing(cell: u8) -> u8 {
    cell >> 5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexing() {
        let mut d = vec![0u8; 6900];
        d[2 * ROWS * COLS + 3 * COLS + 4] = 0xC7;
        let s = Szenario::parse(&d).unwrap();
        let c = s.cell(2, 3, 4);
        assert_eq!((code(c), facing(c)), (7, 6));
        assert!(Szenario::parse(&d[1..]).is_err());
    }

    /// Every layout is framed by code 2 (two border rows/columns each side).
    #[test]
    fn real_border() {
        let Some(d) = crate::data_file("SZENARIO.DAT") else { return };
        let s = Szenario::parse(&d).unwrap();
        for k in 0..SCENARIOS {
            for c in 0..COLS {
                assert_eq!(s.cell(k, 0, c), 2);
                assert_eq!(s.cell(k, ROWS - 1, c), 2);
            }
            for r in 0..ROWS {
                assert_eq!(s.cell(k, r, 0), 2);
                assert_eq!(s.cell(k, r, COLS - 1), 2);
            }
        }
    }
}
