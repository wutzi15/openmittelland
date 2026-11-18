//! Minimal reader for the LE (DOS/4GW) executable `MITTELL.EXE`: maps linear addresses to file bytes.
//! Fixups are not applied, so only read data that holds no pointers (palettes, strings).

use crate::{FormatError, err, u32le};

pub struct Object {
    pub base: u32,
    pub vsize: u32,
    /// File offset and length of each page, in order.
    pages: Vec<(usize, usize)>,
}

impl Object {
    /// Bytes backed by file data; the rest of `vsize` is zero-initialised (BSS).
    pub fn file_len(&self) -> usize {
        self.pages.iter().map(|p| p.1).sum()
    }
}

pub struct LeExe<'a> {
    data: &'a [u8],
    pub page_size: u32,
    pub objects: Vec<Object>,
}

impl<'a> LeExe<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self, FormatError> {
        if data.get(..2) != Some(b"MZ") {
            return err("no MZ header", 0);
        }
        let le = u32le(data, 0x3C)? as usize;
        if data.get(le..le + 2) != Some(b"LE") {
            return err("no LE header", le);
        }
        let h = |o: usize| u32le(data, le + o);
        let (npages, psize, last) = (h(0x14)?, h(0x28)?, h(0x2C)?);
        let (objtab, nobj, pagemap, datapages) = (h(0x40)? as usize, h(0x44)?, h(0x48)? as usize, h(0x80)? as usize);
        let mut objects = vec![];
        for i in 0..nobj as usize {
            let o = le + objtab + 24 * i;
            let (vsize, base, first, count) = (u32le(data, o)?, u32le(data, o + 4)?, u32le(data, o + 12)?, u32le(data, o + 16)?);
            let mut pages = vec![];
            for p in first..first + count {
                let e = le + pagemap + 4 * (p as usize - 1);
                let b = data.get(e..e + 3).ok_or(FormatError { what: "truncated page map", offset: e })?;
                let num = (b[0] as usize) << 16 | (b[1] as usize) << 8 | b[2] as usize;
                let len = if p == npages { last } else { psize } as usize;
                pages.push((datapages + (num - 1) * psize as usize, len));
            }
            objects.push(Object { base, vsize, pages });
        }
        Ok(Self { data, page_size: psize, objects })
    }

    /// Bytes at a linear address as loaded (relocation-free), or `None` if not backed by file data.
    pub fn read(&self, addr: u32, len: usize) -> Option<&'a [u8]> {
        let o = self.objects.iter().find(|o| addr >= o.base && addr < o.base + o.vsize)?;
        let rel = (addr - o.base) as usize;
        let (page, inpage) = (rel / self.page_size as usize, rel % self.page_size as usize);
        let &(off, plen) = o.pages.get(page)?;
        // Objects are stored contiguously in this EXE; allow reads to run into the next page.
        let contiguous = o.pages.windows(2).all(|w| w[0].0 + w[0].1 == w[1].0);
        if !contiguous && inpage + len > plen {
            return None;
        }
        self.data.get(off + inpage..off + inpage + len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MZ stub + LE header with one object of one 16-byte page at base 0x10000.
    #[test]
    fn maps_linear_address() {
        let mut d = vec![0u8; 0x200];
        d[..2].copy_from_slice(b"MZ");
        d[0x3C] = 0x40;
        d[0x40..0x42].copy_from_slice(b"LE");
        let put = |d: &mut Vec<u8>, o: usize, v: u32| d[o..o + 4].copy_from_slice(&v.to_le_bytes());
        put(&mut d, 0x40 + 0x14, 1); // pages
        put(&mut d, 0x40 + 0x28, 16); // page size
        put(&mut d, 0x40 + 0x2C, 16); // last page bytes
        put(&mut d, 0x40 + 0x40, 0x60); // object table (rel)
        put(&mut d, 0x40 + 0x44, 1);
        put(&mut d, 0x40 + 0x48, 0x90); // page map (rel)
        put(&mut d, 0x40 + 0x80, 0x100); // data pages (abs)
        put(&mut d, 0xA0, 16); // vsize
        put(&mut d, 0xA4, 0x10000); // base
        put(&mut d, 0xAC, 1); // first page
        put(&mut d, 0xB0, 1); // page count
        d[0xD0..0xD4].copy_from_slice(&[0, 0, 1, 0]); // page 1
        d[0x105] = 0x42;
        let le = LeExe::parse(&d).unwrap();
        assert_eq!(le.read(0x10005, 1), Some(&[0x42u8][..]));
        assert_eq!(le.read(0x20000, 1), None);
    }

    #[test]
    fn real_exe_objects() {
        let Some(d) = crate::data_file("MITTELL.EXE") else { return };
        let le = LeExe::parse(&d).unwrap();
        assert_eq!(le.objects.iter().map(|o| o.base).collect::<Vec<_>>(), [0x10000, 0x80000]);
        assert_eq!(le.read(0x80450, 21), Some(&b"smpxxkhz\\smpl_xxx.dat"[..]));
    }
}
