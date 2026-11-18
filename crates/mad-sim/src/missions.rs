//! Missions (docs/re/rules-10-missions-scoring.md §3, §5).

use crate::heap::{self, ANY};
use crate::economy::{SCHIFF, TAUCHBOOT, WEIN};
use crate::state::*;
use crate::{Ask, Choices, Event};

/// Law-book strictness (rules-04b §1.1, 0x48DE0), integer / 2.
pub fn strictness(g: &Globals, book: usize) -> u32 {
    let f = |i| g.law(book, i);
    let s = match book {
        0 => {
            f(0) + (f(1) > 9) as u32 + f(2) + f(3) + f(4) + (f(5) > 99) as u32 + (f(6) > 99) as u32
                + (f(7) < 10000) as u32 + (f(8) < 4) as u32 + f(9)
        }
        1 => (0..10).map(f).sum(),
        _ => (0..9).map(f).sum::<u32>() + (f(9) > 9) as u32,
    };
    s / 2
}

fn office(p: &Person, lo: u8, hi: u8, idx: u8) -> bool {
    p.has_office() && (lo..=hi).contains(&p.loc()) && p.idx() == idx
}

/// Mission check at the end of an own turn (0x17930). Deletes the record on success.
pub fn check(st: &mut State, h: usize, ch: &mut dyn Choices) -> Vec<Event> {
    let Some(ri) = st.heap.find(0, h as u8, ANY) else { return vec![] };
    st.heap.recs[ri][3] = 100;
    let m = st.heap.recs[ri][4];
    let cnt = heap::r32(&st.heap.recs[ri], 5);
    let p = st.persons[h].clone();
    let nw = st.net_worth(h);
    let ships = || -> u32 { st.props_of(h, KIND_BRANCH).map(|b| st.props[b].trans(SCHIFF) as u32 + st.props[b].trans(TAUCHBOOT) as u32).sum() };
    let same_rel = |pred: &dyn Fn(u8) -> bool| (0..PERSONS).filter(|&i| st.persons[i].used() && pred(st.persons[i].loc()) && st.persons[i].religion() == p.religion()).count();
    // (condition, streak limit or None for state/counter)
    let (ok, streak): (bool, Option<u32>) = match m {
        0 => (nw > 49_999, None),
        1 => (ships() >= 12, Some(4)),
        2 => (p.certificate() == 2, None),
        3 => (p.title() > 3, None),
        4 => (office(&p, 0, 9, 6), Some(4)),
        5 => (office(&p, 0, 9, 9), Some(6)),
        6 | 7 => (cnt > 2, None),
        8 => (cnt > 9_999, None),
        // the high byte of the residence word, +0x04 (rules-40 §25)
        9 => (st.props_of(h, KIND_BRANCH).any(|b| st.props[b].has_residence() && st.props[b].b3() == 0xE6 && st.props[b].b4() >= 84), Some(2)),
        10 => (nw > 200_000, None),
        11 => {
            let wine = crate::laws::has(st, h, 44);
            ((0..GOODS).filter(|&g| p.skill(g) >= 140 || (g == WEIN && wine)).count() >= 3, None)
        }
        12 => (cnt > 499, None),
        13 => (p.title() > 6, None),
        14 => (office(&p, 10, 12, 6), Some(4)),
        15 => (office(&p, 10, 12, 12), Some(4)),
        16 => (cnt.count_ones() > 15, None),
        17 => ((10..=12).any(|c| same_rel(&|l| l == c) > 10), None),
        18 => (cnt > 159_999, None),
        19 => (cnt > 39_999, None),
        20 => (nw > 750_000, None),
        21 => ((0..3).all(|b| strictness(&st.globals, b) >= 2), Some(3)),
        22 => (cnt > 6, None),
        23 => (st.props_of(h, KIND_CASTLE).count() >= 3, Some(2)),
        24 => (office(&p, 13, 255, 6), Some(6)),
        25 => (office(&p, 13, 255, 10), Some(6)),
        26 => (cnt.count_ones() > 23, None),
        27 => (same_rel(&|l| (10..=13).contains(&l)) > 39, None),
        28 => (cnt >= 600_000, None),
        29 => (cnt > 149_999, None),
        _ => (false, None),
    };
    let done = match streak {
        None => ok,
        Some(limit) => {
            let v = if ok { cnt + 1 } else { 0 };
            heap::w32(&mut st.heap.recs[ri], 5, v);
            v > limit
        }
    };
    if !done {
        return vec![];
    }
    st.heap.delete(ri);
    let mut ev = vec![Event::MissionDone { person: h as u8, mission: m }];
    // rules-14 §4: music(0x1D) on the history screen when it has entries; the outro speaks 13 instead
    // of drawing music, as with a sound card in the large-memory build (rules-56 §4)
    let song = if st.heap.all(57, h as u8, heap::ANY).is_empty() { 0 } else { st.song(0x1D) };
    if ch.ask(&Ask::ContinueAfterMission { person: h as u8, song }) == 1 {
        ev.push(Event::GameOver);
    }
    ev
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::HumanSetup;

    /// rules-10 §3: mission 4 (Bürgermeister) needs a streak of 5 turns.
    #[test]
    fn rules10_s3_streak() {
        let mut st = State::new_game(1, 0, &[HumanSetup { mission: Some(4), ..HumanSetup::test(2) }]).unwrap();
        let h = st.current as usize;
        st.persons[h].set_loc(2);
        st.persons[h].set_idx(6);
        for _ in 0..4 {
            assert!(check(&mut st, h, &mut crate::Auto).is_empty());
        }
        assert_eq!(check(&mut st, h, &mut crate::Auto), vec![Event::MissionDone { person: h as u8, mission: 4 }]);
    }

    /// rules-04b §1.1 strictness of the start laws.
    #[test]
    fn rules04b_s1_1_strictness() {
        let mut g = Globals([0; 0x4E2]);
        crate::laws::init_laws(&mut g);
        g.set_b(ga::MAX_BRANCHES, 10);
        // finance: cap 15>9, bribery 1, min wages 100 (2), parks 1 => 5 / 2 = 2
        assert_eq!(strictness(&g, 0), 2);
        // criminal: 1+1+1+1+0+0+1+0+1+0 = 6 / 2
        assert_eq!(strictness(&g, 1), 3);
        // church: 1+0+1+1+1+0+1+1+1 = 7, max tithe 15 > 9 => 8 / 2
        assert_eq!(strictness(&g, 2), 4);
    }
}
