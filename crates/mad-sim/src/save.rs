//! `GAMEn.DAT` load/store (rules-00 §2, via `mad_assets::save`) and our own snapshot format.
//!
//! Snapshot = the 79703-byte original image, then a trailer the original does not save:
//! `"MADS" u32 version, u32 rng state, u16 options, 15 weather bytes, u8 last fate event,
//! 3 x (u8 toll rate, u32 acc)`. (Loan offers and church prices live in the globals block.)

use crate::heap::Heap;
use crate::state::*;
use mad_assets::save::{BLOCKS, SIZE, Save};

const MAGIC: &[u8; 4] = b"MADS";
const VERSION: u32 = 3;

fn offset(i: usize) -> usize {
    BLOCKS[..i].iter().map(|b| b.1).sum()
}

impl State {
    /// Load an original save. The RNG keeps `seed` (the original does not save it, rules-01 §1).
    /// The castle table (rules-00 §10) is not in GAMEn.DAT. Deviation (rules-55 §14): the original keeps the running
    /// program's values (start values only after a fresh program start); we always load the start values
    /// (toll 5 %, accumulator 0). Our snapshot saves the table, also a deviation.
    pub fn from_game_dat(d: &[u8], seed: u32) -> Result<Self, String> {
        let s = Save::parse(d).map_err(|e| format!("{e:?}"))?;
        let mut st = State::blank(seed);
        for (i, n) in st.names.iter_mut().enumerate() {
            n.copy_from_slice(&s.block(0)[i * 24..(i + 1) * 24]);
        }
        for (i, p) in st.persons.iter_mut().enumerate() {
            p.0.copy_from_slice(&s.block(1)[i * 0xF8..(i + 1) * 0xF8]);
        }
        for (i, p) in st.props.iter_mut().enumerate() {
            p.0.copy_from_slice(&s.block(2)[i * 0x60..(i + 1) * 0x60]);
        }
        let used = rd32(s.block(8), 0) as usize;
        st.heap = Heap::parse(s.block(3), used)?;
        st.year = rd16(s.block(4), 0);
        st.current = s.block(5)[0];
        st.phase = s.block(6)[0];
        for (i, c) in st.cities.iter_mut().enumerate() {
            c.0.copy_from_slice(&s.block(9)[i * 0x8C..(i + 1) * 0x8C]);
        }
        st.goods.copy_from_slice(s.block(10));
        st.globals.0.copy_from_slice(s.block(11));
        Ok(st)
    }

    pub fn to_game_dat(&self) -> Vec<u8> {
        let mut d = vec![0u8; SIZE];
        let mut put = |i: usize, bytes: &[u8]| {
            let o = offset(i);
            d[o..o + bytes.len()].copy_from_slice(bytes);
        };
        put(0, &self.names.concat());
        put(1, &self.persons.iter().flat_map(|p| p.0).collect::<Vec<_>>());
        put(2, &self.props.iter().flat_map(|p| p.0).collect::<Vec<_>>());
        put(3, &self.heap.image(None));
        put(4, &self.year.to_le_bytes());
        put(5, &[self.current]);
        put(6, &[self.phase]);
        let used = self.heap.used_bytes() as u32;
        // free = 10000 - used (rules-39 §10)
        put(7, &(10000 - used).to_le_bytes());
        put(8, &used.to_le_bytes());
        put(9, &self.cities.iter().flat_map(|c| c.0).collect::<Vec<_>>());
        put(10, &self.goods);
        put(11, &self.globals.0);
        d
    }

    pub fn to_snapshot(&self) -> Vec<u8> {
        let mut d = self.to_game_dat();
        d.extend_from_slice(MAGIC);
        d.extend_from_slice(&VERSION.to_le_bytes());
        d.extend_from_slice(&self.rng.s.to_le_bytes());
        d.extend_from_slice(&self.options.to_le_bytes());
        d.extend_from_slice(&self.weather);
        d.push(self.last_fate);
        for c in &self.castles {
            d.push(c.toll_rate);
            d.extend_from_slice(&c.acc.to_le_bytes());
        }
        d
    }

    pub fn from_snapshot(d: &[u8]) -> Result<Self, String> {
        let (game, t) = d.split_at_checked(SIZE).ok_or("snapshot too short")?;
        if t.len() != 4 + 4 + 4 + 2 + 15 + 1 + 15 || &t[..4] != MAGIC || rd32(t, 4) != VERSION {
            return Err("bad snapshot trailer".into());
        }
        let mut st = State::from_game_dat(game, rd32(t, 8))?;
        // the battle fields are game data; the caller re-attaches them
        st.options = rd16(t, 12);
        st.weather.copy_from_slice(&t[14..29]);
        st.last_fate = t[29];
        for (k, c) in st.castles.iter_mut().enumerate() {
            c.toll_rate = t[30 + 5 * k];
            c.acc = rd32(t, 31 + 5 * k);
        }
        Ok(st)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_round_trip() {
        let mut st = State::new_world(1234);
        st.r(10);
        st.castles[1].acc = 77;
        st.options = 0x1234;
        let snap = st.to_snapshot();
        assert_eq!(State::from_snapshot(&snap).unwrap(), st);
    }

    /// `$MAD_SAVE`: a real GAMEn.DAT loads and writes back byte-identically (heap tail past the
    /// used bytes excluded, it is stale memory in the original).
    #[test]
    fn real_save_round_trip() {
        let Some(d) = std::env::var("MAD_SAVE").ok().and_then(|p| std::fs::read(p).ok()) else { return };
        let st = State::from_game_dat(&d, 1).unwrap();
        let out = st.to_game_dat();
        let h = offset(3);
        let used = st.heap.used_bytes();
        assert_eq!(out[..h + used], d[..h + used]);
        assert_eq!(out[offset(4)..], d[offset(4)..]);
        // and the loaded world keeps running
        let mut st = st;
        st.run(&mut crate::Auto);
        for _ in 0..12 {
            st.apply(crate::Command::EndTurn, &mut crate::Auto);
        }
    }
}
