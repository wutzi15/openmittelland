//! `SMP22KHZ/SMPL_xxx.DAT`, `SMP44KHZ/SMPL_xxx.DAT`: sound effects.
//! 20-byte header of five `u32` (see `formats.md`; only partly understood), then unsigned 8-bit mono PCM
//! at 22050 or 44100 Hz depending on the directory.

use crate::{FormatError, err, u32le};

pub struct Sample<'a> {
    /// Raw header words. The loader adds the PCM base address to words 0..=3 (offsets into the PCM);
    /// word 4 is only used when a sound option flag is set. [UNVERIFIED] start/loop/end semantics.
    pub header: [u32; 5],
    pub pcm: &'a [u8],
}

impl<'a> Sample<'a> {
    pub fn parse(d: &'a [u8]) -> Result<Self, FormatError> {
        if d.len() < 20 {
            return err("SMPL header truncated", d.len());
        }
        let mut header = [0; 5];
        for (k, h) in header.iter_mut().enumerate() {
            *h = u32le(d, 4 * k)?;
        }
        Ok(Self { header, pcm: &d[20..] })
    }

    /// Whole PCM payload as a RIFF WAVE file (8-bit unsigned mono).
    pub fn to_wav(&self, rate: u32) -> Vec<u8> {
        let n = self.pcm.len() as u32;
        let mut w = Vec::with_capacity(44 + self.pcm.len());
        w.extend_from_slice(b"RIFF");
        w.extend_from_slice(&(36 + n).to_le_bytes());
        w.extend_from_slice(b"WAVEfmt ");
        for v in [16u32, 0x0001_0001, rate, rate, 0x0008_0001] {
            w.extend_from_slice(&v.to_le_bytes()); // fmt size, PCM+mono, rate, byte rate, align 1 + 8 bits
        }
        w.extend_from_slice(b"data");
        w.extend_from_slice(&n.to_le_bytes());
        w.extend_from_slice(self.pcm);
        w
    }
}

/// File name for sound number `n` (two hex digits) and variant digit `v`: `(0x2D, 0)` -> `SMPL_2D0.DAT`.
pub fn file_name(n: u8, v: u8) -> String {
    format!("SMPL_{n:02X}{v:X}.DAT")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_wav() {
        let mut d = vec![0u8; 20];
        d[12] = 2;
        d.extend([0x80, 0x7F, 0x81]);
        let s = Sample::parse(&d).unwrap();
        assert_eq!(s.header, [0, 0, 0, 2, 0]);
        let w = s.to_wav(22050);
        assert_eq!(&w[..4], b"RIFF");
        assert_eq!(w.len(), 44 + 3);
        assert_eq!(u32::from_le_bytes(w[24..28].try_into().unwrap()), 22050);
        assert_eq!(file_name(0x7B, 1), "SMPL_7B1.DAT");
        assert!(Sample::parse(&[0; 19]).is_err());
    }

    /// 22 kHz and 44 kHz versions of a sample last equally long (within 1%).
    #[test]
    fn real_rates_consistent() {
        let (Some(a), Some(b)) = (crate::data_file("SMP22KHZ/SMPL_310.DAT"), crate::data_file("SMP44KHZ/SMPL_310.DAT")) else {
            return;
        };
        let (a, b) = (Sample::parse(&a).unwrap(), Sample::parse(&b).unwrap());
        let (ta, tb) = (a.pcm.len() as f64 / 22050.0, b.pcm.len() as f64 / 44100.0);
        assert!((ta - tb).abs() / ta < 0.01);
    }
}
