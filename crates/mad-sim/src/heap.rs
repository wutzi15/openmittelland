//! Record heap (rules-00 §8): variable-length records, 10000 bytes. Byte 0 type (bit 7 = deleted),
//! byte 1 owner, byte 2 key, byte 3 counter. Kept as a list of records in heap order; appending and
//! index-based passes reproduce "records allocated during a pass are visited in the same pass".

use crate::state::{rd16, rd32, wr16, wr32};

pub const HEAP_BYTES: usize = 10000;
pub const DELETED: u8 = 0x80;
pub const ANY: u8 = 0xFF;

/// Record size by type (table `0x85980`, rules-00 §8.1).
pub const SIZE: [u8; 58] = [
    13, 5, 8, 4, 75, 6, 15, 9, 7, 12, 6, 6, 7, 6, 4, 6, 4, 4, 6, 8, 4, 4, 5, 5, 6, 7, 7, 9, 6, 4, 11, 6, 6, 9,
    13, 4, 11, 10, 11, 9, 11, 4, 6, 6, 11, 9, 12, 16, 9, 5, 5, 9, 9, 9, 9, 5, 10, 9,
];

pub type Rec = Vec<u8>;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Heap {
    pub recs: Vec<Rec>,
}

pub fn ty(r: &[u8]) -> u8 {
    r[0]
}
pub fn r16(r: &[u8], o: usize) -> u16 {
    rd16(r, o)
}
pub fn r32(r: &[u8], o: usize) -> u32 {
    rd32(r, o)
}
pub fn w16(r: &mut [u8], o: usize, v: u16) {
    wr16(r, o, v)
}
pub fn w32(r: &mut [u8], o: usize, v: u32) {
    wr32(r, o, v)
}

impl Heap {
    pub fn used_bytes(&self) -> usize {
        self.recs.iter().map(|r| r.len()).sum()
    }

    /// Append a record (0x424A0). Returns its index, or None when the heap is full.
    /// The original ends the program on a full heap (rules-39 §10); we refuse.
    pub fn alloc(&mut self, t: u8, owner: u8, key: u8, counter: u8) -> Option<usize> {
        let n = SIZE[t as usize] as usize;
        if self.used_bytes() + n > HEAP_BYTES {
            return None;
        }
        let mut r = vec![0u8; n];
        r[..4].copy_from_slice(&[t, owner, key, counter]);
        self.recs.push(r);
        Some(self.recs.len() - 1)
    }

    pub fn delete(&mut self, i: usize) {
        self.recs[i][0] |= DELETED;
    }

    pub fn live(&self, i: usize) -> bool {
        self.recs[i][0] & DELETED == 0
    }

    /// Compaction (0x42580).
    pub fn compact(&mut self) {
        self.recs.retain(|r| r[0] & DELETED == 0);
    }

    /// First live record matching (type, owner, key), 0xFF = wildcard for owner/key (0x426B0).
    pub fn find(&self, t: u8, owner: u8, key: u8) -> Option<usize> {
        self.find_from(0, t, owner, key)
    }

    pub fn find_from(&self, from: usize, t: u8, owner: u8, key: u8) -> Option<usize> {
        (from..self.recs.len()).find(|&i| {
            let r = &self.recs[i];
            r[0] == t && (owner == ANY || r[1] == owner) && (key == ANY || r[2] == key)
        })
    }

    pub fn all(&self, t: u8, owner: u8, key: u8) -> Vec<usize> {
        let mut v = vec![];
        let mut i = 0;
        while let Some(j) = self.find_from(i, t, owner, key) {
            v.push(j);
            i = j + 1;
        }
        v
    }

    pub fn has_key(&self, t: u8, key: u8) -> bool {
        self.find(t, ANY, key).is_some()
    }

    pub fn has(&self, t: u8, owner: u8) -> bool {
        self.find(t, owner, ANY).is_some()
    }

    /// Split a saved heap (`used` bytes valid).
    pub fn parse(raw: &[u8], used: usize) -> Result<Self, String> {
        let mut recs = vec![];
        let mut o = 0;
        let used = used.min(raw.len());
        while o < used {
            let t = (raw[o] & 0x7F) as usize;
            let n = *SIZE.get(t).ok_or(format!("record type {t} out of range at {o}"))? as usize;
            if o + n > used {
                return Err(format!("record type {t} at {o} overruns used size {used}"));
            }
            recs.push(raw[o..o + n].to_vec());
            o += n;
        }
        Ok(Self { recs })
    }

    /// 10000-byte image; bytes past the used part are `tail` (kept from a loaded save) or zero.
    pub fn image(&self, tail: Option<&[u8]>) -> Vec<u8> {
        let mut v: Vec<u8> = self.recs.concat();
        let n = v.len();
        v.resize(HEAP_BYTES, 0);
        if let Some(t) = tail {
            v[n..].copy_from_slice(&t[n..HEAP_BYTES]);
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rules-00 §8: append, delete by bit 7, compaction, wildcard search.
    #[test]
    fn rules00_s8_heap_ops() {
        let mut h = Heap::default();
        let a = h.alloc(8, 3, 1, 1).unwrap();
        let b = h.alloc(9, 3, 2, 1).unwrap();
        h.alloc(8, 4, 1, 1).unwrap();
        assert_eq!(h.recs[a].len(), 7);
        assert_eq!(h.recs[b].len(), 12);
        assert_eq!(h.all(8, ANY, ANY).len(), 2);
        assert_eq!(h.find(8, 4, ANY), Some(2));
        h.delete(a);
        assert_eq!(h.all(8, ANY, ANY), vec![2]);
        h.compact();
        assert_eq!(h.recs.len(), 2);
        let img = h.image(None);
        assert_eq!(Heap::parse(&img, h.used_bytes()).unwrap(), h);
    }

    #[test]
    fn heap_refuses_overflow() {
        let mut h = Heap::default();
        while h.alloc(4, 0, 0, 0).is_some() {}
        assert!(h.used_bytes() <= HEAP_BYTES);
        assert_eq!(h.used_bytes(), 75 * (HEAP_BYTES / 75));
    }
}
