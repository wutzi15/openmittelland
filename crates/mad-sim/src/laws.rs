//! Laws, privileges, offences, evidence (docs/re/rules-04b-crime-trials-privileges.md §1, §2, §4).

use crate::heap::ANY;
use crate::state::*;

/// Global office index (0x43FF0): idx for cities, +10 counties, +23 principality; 0 without office.
pub fn global_office(p: &Person) -> usize {
    if !p.has_office() {
        return 0;
    }
    let i = p.idx() as usize;
    match p.loc() {
        0..=9 => i,
        10..=12 => i + 10,
        _ => i + 23,
    }
}

/// `privilege(p, p, code)` (§4.1): 0 no, 1 yes, 2 yes and adjustable. Rule code always calls it
/// with viewer = p, so "self" entries count.
pub fn privilege(st: &State, p: usize, code: u8) -> u8 {
    let pr = &st.persons[p];
    let g = global_office(pr);
    let off = pr.has_office();
    let loc = pr.loc();
    let ch = pr.church();
    let yes = |b: bool| b as u8;
    let set = |b: bool| if b { 2 } else { 0 };
    match code {
        1 | 13 => 1,
        4 => yes(pr.spouse() != NONE),
        6 => yes(st.heap.all(31, p as u8, ANY).into_iter().any(|i| st.heap.recs[i][3] < 45)),
        8 | 9 | 12 | 27 => yes(off),
        10 => yes(off && g > 2 && !ch),
        11 => yes(off && ch && g >= 9),
        14 | 15 => set(off && g == 5),
        16 | 17 => set(off && g == 3),
        18..=20 => yes(off && g == 4),
        21 => yes(off && g == 6),
        22 => set(off && g == 6),
        23 | 24 => yes(off && g == 9),
        25 => yes(off && loc > 9),
        26 => yes(off && loc > 9 && !ch),
        28 | 29 => yes(off && g > 12 && g != 18 && g != 19),
        30 => yes(off && g == 15),
        31 => set(off && g == 15 && !st.globals.forbidden(0, 0)),
        32 => set(off && g == 15),
        33..=35 => yes(off && g == 14),
        36..=38 => yes(off && g == 13),
        39 | 40 => yes(off && g == 16),
        41 | 42 => yes(off && g == 17),
        43 => yes(off && g == 19),
        44 => yes(off && g == 18),
        45 => yes(off && g == 21),
        46..=50 => yes(off && g == 20),
        51 => set(off && g == 22),
        52..=55 => yes(off && g == 22),
        56..=58 => yes(off && g == 28),
        59 | 60 | 62 | 63 => yes(off && g == 27),
        61 => set(off && g == 27),
        64 => set(off && g == 26),
        65..=67 => yes(off && g == 26),
        68 | 69 => yes(off && g == 29),
        70 => yes(off && g == 30),
        71 => set(off && g == 30),
        72..=76 => yes(off && g == 31),
        77..=80 => yes(off && g == 32),
        81 => yes(off && g == 32 && st.globals.b(ga::INQUISITION) == 0),
        82..=86 => yes(off && g == 33),
        _ => 0,
    }
}

/// The "self" entries (rules-04b §4.1): listed only when the viewer is the holder.
pub const SELF_ONLY: [u8; 40] = [1, 4, 6, 10, 11, 13, 19, 20, 21, 24, 30, 34, 35, 36, 37, 38, 39, 41, 42, 43, 44, 45, 46, 47, 49, 50, 52, 57, 60, 63, 65, 66, 67, 68, 70, 72, 73, 75, 76, 83];

/// `privilege(p, viewer, code)`: as `privilege`, without the self entries when someone else looks (rules-38 §1.4).
pub fn privilege_for(st: &State, p: usize, viewer: usize, code: u8) -> u8 {
    if p != viewer && SELF_ONLY.contains(&code) { 0 } else { privilege(st, p, code) }
}

pub fn has(st: &State, p: usize, code: u8) -> bool {
    privilege(st, p, code) != 0
}

/// `offences(p)` (0x447D0, §2.2). Rolls `rand(100)` while bribery is legal.
pub fn offences(st: &mut State, p: usize) -> u32 {
    let pr = st.persons[p].clone();
    let law = |st: &State, b, i| st.globals.forbidden(b, i);
    let mut w = if pr.human() { st.globals.d(ga::TURN_FLAGS) } else { 0 };
    let pb = p as u8;
    if law(st, 0, 0) && st.heap.has(56, pb) {
        w |= 1 << 0;
    }
    if pr.has_office() && pr.loc() < 10 && pr.idx() == 6 && st.cities[pr.loc() as usize].tax() as u32 > st.globals.law(0, 1) {
        w |= 1 << 1;
    }
    if !law(st, 0, 2) {
        w &= !(1 << 2);
        if st.r(100) < 20 {
            let d = st.persons[p].deeds() | 0x80;
            st.persons[p].set_deeds(d);
        }
    }
    if !law(st, 0, 3) {
        w &= !(1 << 3);
    }
    if law(st, 0, 4) && st.props_of(p, KIND_CASTLE).count() >= 2 {
        w |= 1 << 4;
    }
    if (pr.pay_workers() as u32) < st.globals.law(0, 5) {
        w |= 1 << 5;
    }
    if (pr.pay_mercs() as u32) < st.globals.law(0, 6) && st.props_of(p, KIND_CASTLE).count() > 0 {
        w |= 1 << 6;
    }
    if pr.cash().max(0) as u32 > st.globals.law(0, 7) {
        w |= 1 << 7;
    }
    let branches: Vec<usize> = st.props_of(p, KIND_BRANCH).collect();
    for &b in &branches {
        let pp = &st.props[b];
        if law(st, 0, 9) && pp.has_residence() && pp.b3() & 0xA0 != 0 {
            w |= 1 << 9;
        }
        if law(st, 2, 4) && pp.has_residence() && pp.b3() & 0x1F == 9 {
            w |= 1 << 24;
        }
    }
    if branches.len() as u32 > st.globals.law(0, 8) {
        w |= 1 << 8;
    }
    let mut deeds = st.persons[p].deeds();
    if law(st, 1, 1) && st.heap.all(6, pb, ANY).into_iter().any(|i| st.heap.recs[i][9] != 0) {
        w |= 1 << 11;
        deeds |= 0x08;
    }
    if law(st, 1, 2) && st.heap.has(7, pb) {
        w |= 1 << 12;
        deeds |= 0x04;
    }
    let traded = |st: &State, g: usize| (0..CITIES).any(|c| st.globals.sales(g, c) != 0);
    for (bit, law_i, g) in [(13, 3, 14), (14, 4, 7), (15, 5, 1)] {
        if law(st, 1, law_i) && pr.human() && traded(st, g) {
            w |= 1 << bit;
        }
    }
    if law(st, 1, 6) && st.heap.has(25, pb) {
        w |= 1 << 16;
        deeds |= 0x20;
    }
    if !law(st, 1, 7) {
        w &= !(1 << 17);
    }
    if law(st, 1, 8) && st.props_of(p, KIND_CAMP).count() > 0 {
        w |= 1 << 18;
        deeds |= 0x02;
    }
    for (bit, b, i) in [(19, 1, 9), (20, 2, 0), (21, 2, 1)] {
        if !law(st, b, i) {
            w &= !(1 << bit);
        }
    }
    if law(st, 2, 2) && st.heap.has(29, pb) {
        w |= 1 << 22;
        deeds |= 0x01;
    }
    if law(st, 2, 3) && st.heap.all(9, pb, ANY).into_iter().any(|i| st.heap.recs[i][9] % 3 == 2) {
        w |= 1 << 23;
    }
    if !law(st, 2, 5) {
        w &= !(1 << 25);
    }
    if law(st, 2, 6) && st.heap.has(28, pb) {
        w |= 1 << 26;
    }
    for (bit, law_i, g) in [(27, 7, 11), (28, 8, 12)] {
        if law(st, 2, law_i) && pr.human() && traded(st, g) {
            w |= 1 << bit;
        }
    }
    if pr.has_office() && (10..=12).contains(&pr.loc()) && pr.idx() == 12 {
        let tithe = st.globals.b(ga::TITHE + pr.loc() as usize - 10) as u32;
        if tithe > st.globals.law(2, 9) {
            w |= 1 << 29;
        }
    }
    st.persons[p].set_deeds(deeds);
    w
}

/// `add_evidence(p, w)` (0x44E00, §2.3).
pub fn add_evidence(st: &mut State, p: usize, w: u32) {
    if w == 0 {
        return;
    }
    for i in st.heap.all(6, ANY, ANY) {
        let r = &mut st.heap.recs[i];
        if r[9] != 0 && r[4] as usize == p {
            let e = crate::heap::r32(r, 0x0B) | w;
            crate::heap::w32(r, 0x0B, e);
        }
    }
    if let Some(i) = st.heap.find(0, p as u8, ANY) {
        let r = &mut st.heap.recs[i];
        if r[4] == 16 || r[4] == 26 {
            let e = crate::heap::r32(r, 5) | w;
            crate::heap::w32(r, 5, e);
        }
    }
    if w.count_ones() > 3 {
        let d = st.persons[p].deeds() | 0x40;
        st.persons[p].set_deeds(d);
    }
}

/// Start values of the laws and law-like globals (§1; data-object initial values).
pub fn init_laws(g: &mut Globals) {
    let fin = [0u8, 15, 1, 0, 0, 100, 100];
    for (i, v) in fin.iter().enumerate() {
        g.set_b(0x85A21 + i, *v);
    }
    g.set_d(0x85A28, 500_000);
    g.set_b(0x85A2D, 1);
    let crim = [1u8, 1, 1, 1, 0, 0, 1, 0, 1, 0];
    let church = [1u8, 0, 1, 1, 1, 0, 1, 1, 1, 15];
    for i in 0..10 {
        g.set_b(0x85A2E + i, crim[i]);
        g.set_b(0x85A38 + i, church[i]);
    }
    for k in 0..3 {
        g.set_b(ga::TITHE + k, 10);
        g.set_b(ga::MAX_INTEREST + k, 35);
        g.set_b(ga::TOLL_SHARE + k, 25);
    }
    g.set_b(ga::STATE_RELIGION, 1);
    g.set_b(ga::INQUISITION, 0);
    g.set_b(ga::HARSHNESS, 50);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rules-04b §1: law table start values and the u32 cash cap at 0x85A28.
    #[test]
    fn rules04b_s1_law_start_values() {
        let mut g = Globals([0; 0x4E2]);
        init_laws(&mut g);
        assert_eq!(g.law(0, 1), 15);
        assert_eq!(g.law(0, 7), 500_000);
        assert_eq!(g.law(0, 9), 1);
        assert_eq!(g.law(2, 9), 15);
        assert!(g.forbidden(1, 0) && !g.forbidden(1, 4));
    }

    /// rules-04b §4.1: privilege examples.
    #[test]
    fn rules04b_s4_1_privileges() {
        let mut st = State::blank(1);
        let mut p = Person::empty();
        p.set_gender(0);
        p.set_loc(11);
        p.set_idx(6); // Kanzler, global 16
        st.persons[0] = p;
        assert_eq!(global_office(&st.persons[0]), 16);
        assert_eq!(privilege(&st, 0, 39), 1);
        assert_eq!(privilege(&st, 0, 25), 1);
        assert_eq!(privilege(&st, 0, 28), 1);
        assert_eq!(privilege(&st, 0, 44), 0);
        st.persons[0].set_loc(13);
        st.persons[0].set_idx(5); // Finanzminister 28
        assert_eq!(privilege(&st, 0, 57), 1);
        st.persons[0].set_loc(3);
        st.persons[0].set_idx(6);
        assert_eq!(privilege(&st, 0, 22), 2);
    }

    /// rules-04b §2.3: evidence only flows into active contracts on p; popcount > 3 sets deed bit 0x40.
    #[test]
    fn rules04b_s2_3_add_evidence() {
        let mut st = State::blank(1);
        st.persons[5] = Person::empty();
        st.persons[5].set_gender(0);
        let a = st.heap.alloc(6, 1, 0, 20).unwrap();
        st.heap.recs[a][4] = 5;
        st.heap.recs[a][9] = 1;
        let b = st.heap.alloc(6, 2, 0, 20).unwrap();
        st.heap.recs[b][4] = 5;
        add_evidence(&mut st, 5, 0b1111);
        assert_eq!(crate::heap::r32(&st.heap.recs[a], 0x0B), 0b1111);
        assert_eq!(crate::heap::r32(&st.heap.recs[b], 0x0B), 0);
        assert_eq!(st.persons[5].deeds() & 0x40, 0x40);
    }
}
