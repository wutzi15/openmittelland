//! Asset decoders for the original game's data files.
//! Nothing here embeds original data; everything is parsed from the user's own install.
//! Format notes: `docs/re/formats.md`.

use std::fmt;

pub mod config;
pub mod flc;
pub mod font;
pub mod iconin;
pub mod le;
pub mod palette;
pub mod save;
pub mod smpl;
pub mod sprite;
pub mod szenario;

/// Malformed or truncated input: what was expected, and the byte offset where it failed.
#[derive(Debug, PartialEq, Eq)]
pub struct FormatError {
    pub what: &'static str,
    pub offset: usize,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at offset {:#x}", self.what, self.offset)
    }
}
impl std::error::Error for FormatError {}

pub(crate) fn err<T>(what: &'static str, offset: usize) -> Result<T, FormatError> {
    Err(FormatError { what, offset })
}
pub(crate) fn u16le(d: &[u8], o: usize) -> Result<u16, FormatError> {
    d.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]])).ok_or(FormatError { what: "truncated", offset: o })
}
pub(crate) fn u32le(d: &[u8], o: usize) -> Result<u32, FormatError> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or(FormatError { what: "truncated", offset: o })
}

/// Test helper: a file under `$MAD_DATA` (the original MITTELL directory), if set and present.
#[cfg(test)]
pub(crate) fn data_file(rel: &str) -> Option<Vec<u8>> {
    let p = std::path::Path::new(&std::env::var("MAD_DATA").ok()?).join(rel);
    std::fs::read(p).ok()
}
