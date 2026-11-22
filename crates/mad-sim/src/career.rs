//! Career (docs/re/rules-03-career.md): favour, titles, offices, eligibility, dismissal, elections.

use crate::heap::ANY;
use crate::laws::global_office;
use crate::state::*;
use crate::{Ask, Choices, Event};

// ---- tables ----

#[derive(Clone, Copy)]
enum At {
    Same,
    CountyOf,
    Abs(u8),
}
use At::*;

/// Appointer table (`0x87318`, §3): per global office, (place, index within level).
const APPOINTERS: [&[(At, u8)]; 34] = [
    &[(Same, 3), (Same, 4), (Same, 5)],
    &[(Same, 3), (Same, 4), (Same, 5)],
    &[(Same, 3), (Same, 4), (Same, 5)],
    &[(Same, 6)],
    &[(Same, 6)],
    &[(Same, 6)],
    &[(Same, 0), (Same, 1), (Same, 2)],
    &[(Same, 9)],
    &[(Same, 9)],
    &[(CountyOf, 12)],
    &[(Same, 3), (Same, 4), (Same, 5)],
    &[(Same, 3), (Same, 4), (Same, 5)],
    &[(Same, 3), (Same, 4), (Same, 5)],
    &[(Same, 6)],
    &[(Same, 6)],
    &[(Same, 6)],
    &[(Same, 7)],
    &[],
    &[(Same, 11)],
    &[(Same, 11)],
    &[(Same, 12)],
    &[(Same, 8), (Same, 9), (Same, 12)],
    &[(Abs(13), 10)],
    &[(Same, 3), (Same, 4), (Same, 5)],
    &[(Same, 3), (Same, 4), (Same, 5)],
    &[(Same, 3), (Same, 4), (Same, 5)],
    &[(Same, 6)],
    &[(Same, 6)],
    &[(Same, 6)],
    &[(Same, 7)],
    &[],
    &[(Same, 10)],
    &[(Same, 10)],
    &[(Abs(10), 12), (Abs(11), 12), (Abs(12), 12)],
];

/// Title cap by global office (`0x86FB8`).
pub const TITLE_CAP: [u8; 34] =
    [1, 1, 1, 2, 2, 2, 3, 1, 1, 3, 4, 4, 4, 5, 5, 5, 6, 7, 4, 4, 5, 5, 6, 8, 8, 8, 9, 9, 9, 10, 11, 8, 8, 10];
/// Career level unlocks (`0x868E9` secular, `0x8690B` church).
pub const UNLOCK_S: [u8; 34] =
    [5, 5, 5, 6, 6, 6, 12, 5, 5, 6, 15, 15, 15, 16, 16, 16, 25, 25, 15, 15, 16, 16, 25, 28, 28, 28, 29, 29, 29, 30, 30, 28, 28, 30];
pub const UNLOCK_C: [u8; 34] =
    [8, 8, 8, 9, 9, 9, 19, 9, 9, 19, 21, 21, 21, 22, 22, 22, 32, 32, 21, 21, 22, 22, 32, 32, 32, 32, 33, 33, 33, 33, 33, 33, 33, 33];
/// Office salary (`0x87264`, rules-02 §7.11).
pub const SALARY: [i32; 34] = [
    0, 0, 0, 250, 250, 250, 500, 0, 0, 0, 400, 400, 400, 750, 750, 750, 1000, 2000, 0, 0, 0, 0, 0, 1000, 1000,
    1000, 2500, 2500, 2500, 5000, 10000, 0, 0, 0,
];
/// Initial office wealth targets (`0x86BD0`, §4.4).
pub const OFFICE_WEALTH: [u32; 34] = [
    5000, 5000, 5000, 20000, 20000, 20000, 40000, 8000, 8000, 30000, 50000, 50000, 50000, 100000, 100000, 100000,
    120000, 120000, 50000, 50000, 100000, 100000, 100000, 200000, 200000, 200000, 300000, 300000, 300000, 500000,
    500000, 200000, 200000, 500000,
];
/// Net-worth thresholds for titles (`0x866E8`, §2.1).
pub const TITLE_NW: [i32; 10] = [0, 6000, 10000, 22000, 52000, 102000, 180000, 280000, 420000, 600000];

pub fn is_church_office(o: usize) -> bool {
    matches!(o, 7..=9 | 18..=22 | 31..=33)
}
/// Offices per level location: (count of within-level indices).
pub fn level_size(loc: usize) -> usize {
    match loc {
        0..=9 => 10,
        10..=12 => 13,
        _ => 11,
    }
}
pub fn global_of(loc: usize, idx: usize) -> usize {
    match loc {
        0..=9 => idx,
        10..=12 => idx + 10,
        _ => idx + 23,
    }
}
/// Seat groups (§4.6): indices 0..2 at every level (Ratsherren / Geheimräte / Hofräte), the two Priester
/// (city 7..8). Applicants of the group compete for each of its vacant seats (rules-40 §1).
pub fn seat_group(loc: usize, idx: usize) -> std::ops::Range<usize> {
    match (loc, idx) {
        (0..=9, 7 | 8) => 7..9,
        (_, 0..=2) => 0..3,
        _ => idx..idx + 1,
    }
}

/// Result of [`State::forecast`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Forecast {
    /// (voter, appointer he stands for or NONE), as in the election.
    pub electors: Vec<(usize, u8)>,
    pub candidates: Vec<usize>,
    pub privileged: bool,
    /// Per elector, per candidate: base score and whether the office branch (rival's rand(20)) applies.
    pub scores: Vec<Vec<(i32, bool)>>,
    /// Per elector: the candidate index he votes for; None for a human elector or without candidates.
    pub picks: Vec<Option<usize>>,
    pub votes: Vec<u32>,
    /// Candidate indices with the most votes (one: the winner; several: the lot decides).
    pub leaders: Vec<usize>,
}

// ---- favour primitives (§1) ----

impl State {
    /// `favour_decrease(a, b, x)`: favour(b→a) -= x (doubled across religions), floor 0.
    pub fn favour_decrease(&mut self, a: usize, b: usize, x: i32) {
        let x = if self.persons[a].religion() != self.persons[b].religion() { x * 2 } else { x };
        let f = (self.persons[b].favour(a) as i32 - x).max(0);
        self.persons[b].set_favour(a, f.min(255) as u8);
    }
    /// `favour_increase(a, b, x)`: favour(b→a) += x (halved across religions), cap 100; values
    /// above 100 (blackmail marker) are left alone.
    pub fn favour_increase(&mut self, a: usize, b: usize, x: i32) {
        let f = self.persons[b].favour(a) as i32;
        if f > 100 {
            return;
        }
        let x = if self.persons[a].religion() != self.persons[b].religion() { x / 2 } else { x };
        self.persons[b].set_favour(a, (f + x).clamp(0, 100) as u8);
    }
}

// ---- titles (§2) ----

impl State {
    /// §2.1 highest residence level over the player's branches.
    pub fn best_residence(&self, p: usize) -> i32 {
        self.props_of(p, KIND_BRANCH)
            .filter(|&b| self.props[b].has_residence())
            .map(|b| (self.props[b].residence() & 0x1F) as i32)
            .max()
            .unwrap_or(0)
    }

    /// §2.1 title_award for a human at settlement start.
    pub fn title_award(&mut self, p: usize) -> Option<Event> {
        if self.year == 1600 {
            return None;
        }
        let title = self.persons[p].title() as i32;
        if self.ri(100) < title * 6 || title >= 9 {
            return None;
        }
        let nw = self.net_worth(p).max(0);
        let k = TITLE_NW.iter().filter(|&&t| t <= nw).count() as i32;
        let cap = self.best_residence(p).min(k - 1);
        let new = if cap > title && self.ri(100) < 80 {
            title + 1
        } else if title == 0 {
            1
        } else {
            return None;
        };
        self.persons[p].set_title(new as u8);
        // rules-27 §1 draw order: the branch (titles 1, 2), the fanfare rand(2), the branch again (title 3)
        let branch = |s: &mut State| {
            let bs: Vec<usize> = s.props_of(p, KIND_BRANCH).collect();
            if bs.is_empty() {
                return s.persons[p].loc();
            }
            let i = s.r(bs.len() as u32) as usize;
            s.props[bs[i]].loc()
        };
        let mut city = if matches!(new, 1 | 2) { branch(self) } else { 0 };
        // the charter's authority (rules-41 §1): first person at the place with church flag 0 holding the office,
        // else any such person there, else one rand(count) over every used person
        let place = match new {
            1 => Some((city as usize, 6)),
            2 => Some(((city as usize + usize::from(city == 0) - 1) / 3 + 10, 7)),
            3 | 4 => Some((13, 7)),
            _ => None,
        };
        let awarder = place.map_or(NONE, |(l, o)| {
            let at = |st: &State, o: Option<u8>| {
                (0..PERSONS).find(|&x| {
                    let q = &st.persons[x];
                    q.used() && q.loc() as usize == l && !q.church() && o.is_none_or(|o| q.idx() == o)
                })
            };
            match at(self, Some(o)).or_else(|| at(self, None)) {
                Some(x) => x as u8,
                None => {
                    let all: Vec<usize> = (0..PERSONS).filter(|&x| self.persons[x].used()).collect();
                    all[self.r(all.len() as u32) as usize] as u8
                }
            }
        });
        let song = [7, 0x0E][self.r(2) as usize];
        if new == 3 {
            city = branch(self);
        }
        Some(Event::Title { person: p as u8, title: new as u8, city, song, awarder })
    }
}

// ---- offices ----

impl State {
    pub fn in_prison(&self, p: usize) -> bool {
        self.heap.has(20, p as u8) || self.heap.has(21, p as u8)
    }

    fn appointer_loc(loc: usize, at: At) -> usize {
        match at {
            Same => loc,
            CountyOf => crate::economy::COUNTY[loc] as usize,
            Abs(l) => l as usize,
        }
    }

    /// The appointer seats of (loc, idx) as (place, index within level); empty for Vogt and Regent (the lot).
    pub fn appointer_seats(loc: usize, idx: usize) -> Vec<(usize, usize)> {
        APPOINTERS[global_of(loc, idx)].iter().map(|&(at, i)| (Self::appointer_loc(loc, at), i as usize)).collect()
    }

    /// Holders of the appointer seats of (loc, idx); missing holders skipped.
    pub fn appointers(&self, loc: usize, idx: usize) -> Vec<usize> {
        Self::appointer_seats(loc, idx).into_iter().filter_map(|(l, i)| self.holder(l, i)).collect()
    }

    /// `office_vacate(p)` (0x45980).
    pub fn office_vacate(&mut self, p: usize) {
        let pr = &mut self.persons[p];
        pr.set_idx(15);
        pr.set_loc(0);
        pr.set_flags10(pr.flags10() & 0xF0);
        for j in 0..PERSONS {
            if pr.favour(j) > 100 {
                pr.set_favour(j, 30);
            }
        }
        if !self.persons[p].human() {
            for g in 0..GOODS {
                let s = 83 + self.r(34) as u8;
                self.persons[p].set_skill(g, s);
            }
        }
    }

    /// `office_eligibility(p, place, idx)` (0x47C20, §3.2): 0, 1 or 2.
    pub fn eligibility(&self, p: usize, place: usize, idx: usize) -> u8 {
        let pr = &self.persons[p];
        let o = global_of(place, idx);
        let church = is_church_office(o);
        let t = pr.title();
        if t == 0 || (t < 3 && place >= 10) || (t < 6 && place == 13) || (pr.religion() == 2 && church) || idx == 15 {
            return 0;
        }
        let level = if church { pr.level_church() } else { pr.level_secular() };
        if (level as usize) < o {
            return 0;
        }
        let cur = global_office(pr);
        if !pr.human() && pr.has_office() {
            let l = pr.loc() as usize;
            let ok = match l {
                0..=9 => place == l || (matches!(cur, 6 | 9) && place == crate::economy::COUNTY[l] as usize),
                // the county itself, not its cities; Kanzler, Vogt, Bischof also the principality (rules-55 §5)
                10..=12 => place == l || (matches!(cur, 16 | 17 | 22) && place == 13),
                _ => true,
            };
            if !ok {
                return 0;
            }
        }
        if pr.human() {
            let need = match place {
                0..=9 => self.branch_at(p, place).is_some(),
                10..=12 => (0..CITIES).any(|c| crate::economy::COUNTY[c] as usize == place && self.branch_at(p, c).is_some()),
                _ => true,
            };
            if !need {
                return 0;
            }
        }
        if self.holder(place, idx).is_some() {
            return 0;
        }
        if !pr.has_office() || TITLE_CAP[cur] < TITLE_CAP[o] { 2 } else { 1 }
    }

    /// Lowest free seat of a group, if any.
    pub fn free_seat(&self, loc: usize, idx: usize) -> Option<usize> {
        seat_group(loc, idx).find(|&i| self.holder(loc, i).is_none())
    }

    /// §4.6 appointment of `p` to (loc, idx).
    pub fn appoint(&mut self, p: usize, loc: usize, idx: usize) {
        let o = global_of(loc, idx);
        let human = self.persons[p].human();
        let pr = &mut self.persons[p];
        pr.set_loc(loc as u8);
        pr.set_idx(idx as u8);
        pr.set_flags10((pr.flags10() & 0xF0) | is_church_office(o) as u8);
        pr.set_level_secular(pr.level_secular().max(UNLOCK_S[o]));
        pr.set_level_church(pr.level_church().max(UNLOCK_C[o]));
        if human {
            let c = pr.certificate();
            if c == 0 || (c == 1 && loc >= 10 && idx >= 2) {
                pr.set_certificate_due(1);
            }
        } else {
            // §2.2: +1 title if < 3 (any), < 6 (county/principality), < 9 (principality)
            let t = pr.title();
            if t < 3 || (t < 6 && loc >= 10) || (t < 9 && loc == 13) {
                pr.set_title(t + 1);
            }
        }
        for i in self.heap.all(5, p as u8, ANY) {
            self.heap.delete(i);
        }
    }

    /// `person_init_wealth(p, pct, max)`: cash moves pct % of (target - NW), at most `max`.
    /// `person_init_wealth` 0x42EA0 (rules-66 §4): above the target the gap is paid (`pay`, floor -9 999 999),
    /// below it is added (`cash_add`, top +9 999 999), at most `max` either way.
    pub fn init_wealth(&mut self, p: usize, pct: i32, max: i32) {
        let target = self.globals.office_wealth(global_office(&self.persons[p])) as i32;
        let nw = self.net_worth(p);
        let d = (pct as i64 * (nw as i64 - target as i64) / 100) as i32;
        if d > 0 {
            self.pay(p, d.min(max));
        } else {
            self.add_cash(p, (-d).min(max));
        }
    }

    /// §4.1 human_rival_of(h).
    pub fn rival_of(&mut self, h: usize) -> Option<usize> {
        let pr = &self.persons[h];
        let l = if pr.has_office() {
            pr.loc() as usize
        } else if let Some(b) = self.pick_property(KIND_BRANCH, h) {
            // a random branch, one rand(n) also for n = 1 (rules-40 §3)
            self.props[b].loc() as usize
        } else {
            self.r(10) as usize
        };
        let pick = |st: &State, filt: &dyn Fn(usize) -> bool| {
            st.computers().filter(|&x| filt(x) && st.persons[x].favour(h) < 102).min_by_key(|&x| st.persons[x].favour(h))
        };
        // lowest favour at l, else elsewhere; the running minimum starts at 102 (rules-40 §3)
        pick(self, &|x| self.persons[x].loc() as usize == l).or_else(|| pick(self, &|_| true))
    }

    /// The human whose rival `p` is in the year-start snapshot (0xFF none): the `b` of the dismissal and election
    /// `%F` forms (rules-44 §4: the first matching human in slot order).
    fn snap_human(snap: &[(usize, usize)], p: usize) -> u8 {
        snap.iter().find(|&&(_, r)| r == p).map_or(NONE, |&(h, _)| h as u8)
    }

    /// §4.2-§4.4 as one loop over the person slots (rules-66 §1): per used person the favour floor of his row, the
    /// dismissal check, the net-worth cache, then a computer's title cap, skill gain and wealth drift or a human's
    /// office-wealth update; the denunciation flag is cleared for every slot.
    fn office_year(&mut self, snap: &[(usize, usize)], ev: &mut Vec<Event>) {
        for p in 0..PERSONS {
            if self.persons[p].used() {
                for b in 0..PERSONS {
                    if self.persons[p].favour(b) < 10 {
                        let v = self.persons[p].favour(b) + self.r(2) as u8;
                        self.persons[p].set_favour(b, v);
                    }
                }
                let raw = self.net_worth(p);
                let nw = raw.max(0);
                let o = if self.persons[p].has_office() { global_office(&self.persons[p]) } else { 0 };
                if self.persons[p].has_office() {
                    self.dismissal_check(p, nw, snap, ev);
                }
                self.persons[p].set_nw_cache(nw);
                if self.persons[p].human() {
                    let w = self.globals.office_wealth(o) as i64;
                    self.globals.set_office_wealth(o, (w + (raw as i64 - w) / 10).max(0) as u32);
                } else {
                    let pr = &self.persons[p];
                    // the office after the check (rules-67 §1): a dismissed person has place 0 and cap 1 here
                    let cap = TITLE_CAP[if pr.has_office() { global_office(pr) } else { 0 }];
                    if pr.loc() < 13 && pr.title() < cap {
                        let t = pr.title() + 1;
                        self.persons[p].set_title(t);
                    }
                    // skill gain: x = 2 + rand(5), good rand(15), cap 150 (rules-40 §4, rules-66 §1)
                    let x = 2 + self.r(5);
                    let g = self.r(15) as usize;
                    self.skill_up(p, g, x);
                    self.init_wealth(p, 20, 7000);
                }
            }
            self.persons[p].set_denounced(0);
        }
    }

    /// §4.3 dismissal for losing wealth, for one office holder.
    fn dismissal_check(&mut self, p: usize, nw: i32, snap: &[(usize, usize)], ev: &mut Vec<Event>) {
        {
            let old = self.persons[p].nw_cache();
            if nw < old || self.persons[p].denounced() != 0 {
                // NW·100 wraps in i32 as in the original (rules-55 §6). Deliberate deviation: old = 0 (reached
                // only after an Anschwärzen) is an unguarded divide fault in the original; r = 0 here
                let r = nw.wrapping_mul(100).checked_div(old).unwrap_or(0);
                let (loc, idx) = (self.persons[p].loc() as usize, self.persons[p].idx() as usize);
                let apps = self.appointers(loc, idx);
                let mut low = 0;
                let mut by = vec![];
                for &a in &apps {
                    let f = self.persons[a].favour(p) as i32;
                    self.favour_decrease(p, a, f - f * r / 100 + 1);
                    if !self.persons[a].human() && self.persons[a].favour(p) < 30 {
                        low += 1;
                        by.push(a as u8);
                    }
                }
                if low > apps.len() / 2 {
                    let rival = Self::snap_human(snap, p);
                    self.office_vacate(p);
                    if !self.persons[p].human() {
                        crate::family::purge(self, p, 3);
                    }
                    ev.push(Event::Dismissed { person: p as u8, by, loc: loc as u8, idx: idx as u8, rival });
                }
            }
        }
    }

    /// §4.5 / rules-05 §9 computer-person turnover.
    fn turnover(&mut self, ev: &mut Vec<Event>) {
        for l in 0..14usize {
            let pct = 15 + 10 * (l < 13) as u32 + 10 * (l < 10) as u32;
            if self.r(100) >= pct {
                continue;
            }
            let pool: Vec<usize> = self.computers().filter(|&x| self.persons[x].loc() as usize == l).collect();
            if pool.is_empty() {
                continue;
            }
            // person_pick_random: slot order, one rand(count) (rules-24 §2)
            let x = pool[self.r(pool.len() as u32) as usize];
            if !self.replaceable(x) {
                continue;
            }
            crate::family::purge(self, x, 1);
            // rules-03 §4.5: the office becomes vacant; others' favour toward the slot is kept.
            self.vacate_silently(x);
            let female = self.r(4) == 0;
            let name = if female { self.r(70) } else { self.r(117) };
            // 0x42CF0 writes only its own fields; the rest of the predecessor's record stays, the net-worth cache
            // among it, so the newcomer's first dismissal check compares with the old wealth (rules-66 §3)
            let old = self.persons[x].0;
            self.person_init(x, female as u8, name as u8, 0);
            for k in [0x01, 0x05, 0x06, 0x07, 0x0E, 0x14, 0x15, 0x16, 0x17, 0x1C, 0x1D, 0x1E, 0x1F, 0xF4] {
                self.persons[x].0[k] = old[k];
            }
            let t = 1 + self.r(2) as u8;
            self.persons[x].set_title(t);
            self.init_wealth(x, 100, 10_000_000);
            ev.push(Event::PersonReplaced { person: x as u8 });
        }
    }

    fn vacate_silently(&mut self, x: usize) {
        let pr = &mut self.persons[x];
        pr.set_idx(15);
        pr.set_loc(0);
        pr.set_flags10(pr.flags10() & 0xF0);
    }

    pub fn replaceable(&self, x: usize) -> bool {
        let xb = x as u8;
        self.props_of(x, KIND_CASTLE).next().is_none()
            && self.props_of(x, KIND_CAMP).next().is_none()
            && !(0..PROPS).any(|i| self.props[i].kind() == KIND_BRANCH && self.props[i].director() == xb)
            && !self.humans().any(|h| self.persons[h].spouse() == xb)
            && !self.heap.all(30, ANY, ANY).into_iter().any(|i| self.heap.recs[i][9] == xb)
            && !self.in_type44(x)
    }

    fn in_type44(&self, x: usize) -> bool {
        self.heap.all(44, ANY, ANY).into_iter().any(|i| self.heap.recs[i].get(4) == Some(&(x as u8)))
    }

    /// Application records (type 5: +4 place, +5 index within level; rules-00 §8.2). A grouped seat's record
    /// stores the seat listed or filled (rules-40 §5); elections accept any index of the group.
    fn apps(&self, owner: u8, loc: usize, group: &std::ops::Range<usize>) -> Vec<usize> {
        self.heap
            .all(5, owner, ANY)
            .into_iter()
            .filter(|&i| self.heap.recs[i][4] as usize == loc && group.contains(&(self.heap.recs[i][5] as usize)))
            .collect()
    }

    pub fn applicants(&self, loc: usize, group: &std::ops::Range<usize>) -> Vec<usize> {
        self.apps(ANY, loc, group)
            .into_iter()
            .map(|i| self.heap.recs[i][1] as usize)
            .filter(|&p| !self.in_prison(p))
            .collect()
    }

    fn new_application(&mut self, p: usize, loc: usize, idx: usize) {
        if let Some(i) = self.heap.alloc(5, p as u8, NONE, 2) {
            self.heap.recs[i][4] = loc as u8;
            self.heap.recs[i][5] = idx as u8;
        }
    }

    pub fn apply_for(&mut self, p: usize, loc: usize, idx: usize, on: bool) {
        let existing = self.apps(p as u8, loc, &(idx..idx + 1));
        if on && existing.is_empty() {
            self.new_application(p, loc, idx);
        }
        if !on {
            for i in existing {
                self.heap.delete(i);
            }
        }
    }

    /// §4.7 offices a human may apply for: eligibility 2, place 13 down to 0, index high to low,
    /// grouped seats once; at most 6.
    pub fn applicable_offices(&self, p: usize) -> Vec<(usize, usize)> {
        let mut v = vec![];
        for loc in (0..14).rev() {
            let mut skip = 0..0;
            for idx in (0..level_size(loc)).rev() {
                // the first eligible seat of a group, scanning down, is listed; the rest of the group is skipped (rules-40 §5)
                if skip.contains(&idx) {
                    continue;
                }
                if self.eligibility(p, loc, idx) == 2 && v.len() < 6 {
                    v.push((loc, idx));
                    skip = seat_group(loc, idx);
                }
            }
        }
        v
    }

    /// §4.6 filling vacancies.
    fn fill_vacancies(&mut self, snap: &[(usize, usize)], ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        if self.year < 1601 {
            return;
        }
        // every vacant seat gets its own election, highest index first (rules-40 §1)
        for loc in (0..14usize).rev() {
            let mut skip = 0..0;
            for idx in (0..level_size(loc)).rev() {
                if skip.contains(&idx) || self.holder(loc, idx).is_some() {
                    continue;
                }
                let group = seat_group(loc, idx);
                if !self.fill_one(loc, idx, &group, snap, ch, ev) {
                    // a human is eligible: the rest of the group waits for the next year
                    skip = group;
                }
            }
        }
        for i in self.heap.all(5, ANY, ANY) {
            if self.heap.recs[i][3] < 2 {
                self.heap.delete(i);
            }
        }
    }

    /// One election for (loc, seat); false when it stopped because a human may apply.
    fn fill_one(&mut self, loc: usize, seat: usize, group: &std::ops::Range<usize>, snap: &[(usize, usize)], ch: &mut dyn Choices, ev: &mut Vec<Event>) -> bool {
        let o = global_of(loc, seat);
        let electors = self.electors(loc, seat);
        let privileged = self.privileged(loc, group);
        // E | A (rules-44 §3): a human elector, or a human among all applicants (the shut-out ones too)
        let shown = electors.iter().any(|&(e, _)| self.persons[e].human()) || self.applicants(loc, group).iter().any(|&c| self.persons[c].human());
        let mut cands: Vec<usize> = match privileged {
            Some(c) => vec![c],
            None => self.applicants(loc, group),
        };
        cands.dedup();
        if cands.is_empty() {
            // rules-65 §1: the humans' rivals, then three draws from the eligible computers, one de-duplication over
            // all (same person or name index; the last entry fills the hole), every survivor registered (counter 2).
            // Only then does an eligible human hold the seat open for a year; the registrations stay and are his
            // opponents next year.
            let pool: Vec<usize> =
                self.computers().filter(|&x| self.eligibility(x, loc, seat) == 2 && !self.in_type44(x)).collect();
            let human_eligible = self.humans().any(|h| self.eligibility(h, loc, seat) == 2);
            for &(_, r) in snap {
                if self.eligibility(r, loc, seat) == 2 && !self.in_type44(r) {
                    cands.push(r);
                }
            }
            if !pool.is_empty() {
                for _ in 0..3 {
                    cands.push(pool[self.r(pool.len() as u32) as usize]);
                }
            }
            let mut i = 0;
            while i < cands.len() {
                let mut j = i + 1;
                while j < cands.len() {
                    if cands[j] == cands[i] || self.persons[cands[j]].name() == self.persons[cands[i]].name() {
                        cands.swap_remove(j);
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
            for &c in &cands {
                self.new_application(c, loc, seat);
            }
            if human_eligible {
                return false;
            }
        }
        if cands.is_empty() {
            return true;
        }
        let mut vote_log: Vec<(u8, u8)> = vec![];
        let rivals = |cs: &[usize]| -> Vec<u8> { cs.iter().map(|&c| Self::snap_human(snap, c)).collect() };
        let (winner, unique) = if cands.len() == 1 && privileged.is_none() {
            // no election (rules-44 §3 step 1): shown when the candidate is human or a human's rival
            let w = cands[0];
            if self.persons[w].human() || Self::snap_human(snap, w) != NONE {
                let religion = self.persons[w].religion();
                ev.push(Event::Election { loc: loc as u8, seat: seat as u8, candidates: vec![w as u8], votes: vec![], winner: w as u8, kind: 2, rivals: rivals(&[w]), privileged: false, religion });
            }
            (w, true)
        } else {
            // the displayed election's place picture draws its religion first (rules-44 §3)
            let religion = if shown { self.r(2) as u8 } else { 0 };
            let mut votes = vec![0u32; cands.len()];
            for &(e, for_) in &electors {
                // a blackmailer replacing two electors votes twice, all draws twice (rules-40 §6)
                let choice = if self.persons[e].human() {
                    let q = Ask::Vote {
                        elector: e as u8,
                        candidates: cands.iter().map(|&c| c as u8).collect(),
                        for_,
                        loc: loc as u8,
                        seat: seat as u8,
                        religion,
                        rivals: rivals(&cands),
                        privileged: privileged.is_some(),
                        votes: vote_log.clone(),
                    };
                    (ch.ask(&q) as usize)
                        .min(cands.len() - 1)
                } else {
                    let mut best = (i32::MIN, 0);
                    for (k, &c) in cands.iter().enumerate() {
                        let (mut s, office) = self.vote_score_base(e, c, loc, o);
                        // a human's year-start rival, not a human candidate (rules-65 §4)
                        if office && snap.iter().any(|&(_, r)| r == c) {
                            s += self.ri(20);
                        }
                        if s > best.0 {
                            best = (s, k);
                        }
                    }
                    best.1
                };
                votes[choice] += 1;
                vote_log.push((e as u8, cands[choice] as u8));
                for (k, &c) in cands.clone().iter().enumerate() {
                    if k == choice {
                        let x = self.ri(25);
                        self.favour_increase(e, c, x);
                    } else {
                        let x = self.ri(10);
                        self.favour_decrease(e, c, x);
                    }
                }
            }
            let max = *votes.iter().max().unwrap();
            let tied: Vec<usize> = (0..cands.len()).filter(|&k| votes[k] == max).collect();
            // the lot is drawn also for a single leader (rules-44 §3)
            let w = cands[tied[self.r(tied.len() as u32) as usize]];
            if shown {
                ev.push(Event::Election { loc: loc as u8, seat: seat as u8, candidates: cands.iter().map(|&c| c as u8).collect(), votes: vote_log, winner: w as u8, kind: (tied.len() > 1) as u8, rivals: rivals(&cands), privileged: privileged.is_some(), religion });
            } else if Self::snap_human(snap, w) != NONE {
                let religion = self.persons[w].religion();
                ev.push(Event::Election { loc: loc as u8, seat: seat as u8, candidates: vec![w as u8], votes: vec![], winner: w as u8, kind: 3, rivals: rivals(&[w]), privileged: false, religion });
            }
            (w, tied.len() == 1)
        };
        // history code 5 (rules-41 §2): a human wins a unique majority for offices 16, 22 or >= 29; a single
        // applicant counts as a unique majority [UNVERIFIED: rules-41 names only the tie and the computer rule as exceptions]
        if unique && self.persons[winner].human() && (o == 16 || o == 22 || o >= 29) {
            self.history(winner, 5, o as u8, loc as u8);
        }
        self.appoint(winner, loc, seat);
        ev.push(Event::Appointed { person: winner as u8, loc: loc as u8, idx: seat as u8 });
        true
    }

    /// The deterministic part of a computer elector's score for candidate `c` (§4.6, rules-65 §4): favour + title,
    /// and for a holder of an office of the vacant one's kind the colleague and seniority terms. The flag says that
    /// office branch applied, where the real election adds rand(20) for a human's rival. Shared by the election and
    /// the forecast so the two cannot drift.
    pub fn vote_score_base(&self, e: usize, c: usize, loc: usize, o: usize) -> (i32, bool) {
        let pc = &self.persons[c];
        let mut s = self.persons[e].favour(c) as i32 + pc.title() as i32;
        let office = pc.has_office() && is_church_office(global_office(pc)) == is_church_office(o);
        if office {
            if pc.loc() as usize == loc {
                let kind = pc.church();
                let col: Vec<usize> = self
                    .computers()
                    // place and kind only (rules-65 §4): at place 0 the officeless persons count for a secular seat
                    .filter(|&q| self.persons[q].loc() as usize == loc && self.persons[q].church() == kind)
                    .collect();
                let sum: i32 = col.iter().map(|&q| self.persons[q].favour(c) as i32).sum();
                if sum > 0 {
                    s += sum / (col.len() as i32 * 8);
                }
            }
            s += (33 - (o as i32 - global_office(pc) as i32).max(0)) >> 1;
        }
        (s, office)
    }

    /// Electors of (loc, seat) with the appointer each stands for when he blackmails him (NONE: his own vote).
    pub fn electors(&self, loc: usize, seat: usize) -> Vec<(usize, u8)> {
        let mut electors: Vec<(usize, u8)> = vec![];
        for a in self.appointers(loc, seat) {
            let bm: Vec<usize> = (0..PERSONS).filter(|&x| self.persons[a].favour(x) > 100 && self.persons[x].used()).collect();
            if bm.is_empty() { electors.push((a, NONE)) } else { electors.extend(bm.into_iter().map(|x| (x, a as u8))) }
        }
        electors
    }

    /// The privileged holder of (loc, group) (Erzdiakon, Diakon, Sakristan) when he applied: the only candidate.
    pub fn privileged(&self, loc: usize, group: &std::ops::Range<usize>) -> Option<usize> {
        match global_of(loc, group.start) {
            33 => self.holder(13, 8),
            22 => self.holder(loc, 10),
            21 => self.holder(loc, 9),
            _ => None,
        }
        .filter(|&c| self.applicants(loc, group).contains(&c))
    }

    /// Ours (office info): the likely outcome of the next election for a vacant (loc, seat) among the current
    /// applicants, without RNG. Per elector the first candidate with the highest base score (None for a human
    /// elector), then the candidates with most votes.
    pub fn forecast(&self, loc: usize, seat: usize) -> Forecast {
        let o = global_of(loc, seat);
        let group = seat_group(loc, seat);
        let privileged = self.privileged(loc, &group);
        let mut candidates = match privileged {
            Some(c) => vec![c],
            None => self.applicants(loc, &group),
        };
        candidates.dedup();
        let electors = self.electors(loc, seat);
        let scores: Vec<Vec<(i32, bool)>> =
            electors.iter().map(|&(e, _)| candidates.iter().map(|&c| self.vote_score_base(e, c, loc, o)).collect()).collect();
        let picks: Vec<Option<usize>> = electors
            .iter()
            .zip(&scores)
            .map(|(&(e, _), sc)| {
                if self.persons[e].human() || sc.is_empty() {
                    return None;
                }
                let mut best = (i32::MIN, 0);
                for (k, &(s, _)) in sc.iter().enumerate() {
                    if s > best.0 {
                        best = (s, k);
                    }
                }
                Some(best.1)
            })
            .collect();
        let mut votes = vec![0u32; candidates.len()];
        for k in picks.iter().flatten() {
            votes[*k] += 1;
        }
        let max = votes.iter().copied().max().unwrap_or(0);
        let leaders = (0..candidates.len()).filter(|&k| votes[k] == max).collect();
        Forecast { electors, candidates, privileged: privileged.is_some(), scores, picks, votes, leaders }
    }

    /// Year-end office cycle (§4.1 - §4.6), in order.
    pub fn year_end_offices(&mut self, ch: &mut dyn Choices) -> Vec<Event> {
        let mut ev = vec![];
        // the rival snapshot the dismissal and election texts read (rules-44 §4), with each human's friction
        let mut snap = vec![];
        for h in self.humans().collect::<Vec<_>>() {
            if let Some(r) = self.rival_of(h) {
                snap.push((h, r));
                let x = self.ri(3);
                self.favour_decrease(h, r, x);
            }
        }
        self.office_year(&snap, &mut ev);
        self.turnover(&mut ev);
        self.fill_vacancies(&snap, ch, &mut ev);
        ev
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Auto;

    fn person(st: &mut State, i: usize, loc: u8, idx: u8) {
        st.person_init(i, 0, 0, 0);
        st.persons[i].set_loc(loc);
        st.persons[i].set_idx(idx);
    }

    /// rules-03 §1: religion halves increases and doubles decreases; >100 is untouched.
    #[test]
    fn rules03_s1_favour_primitives() {
        let mut st = State::blank(1);
        person(&mut st, 0, 0, 15);
        person(&mut st, 1, 0, 15);
        st.persons[1].set_favour(0, 50);
        st.favour_increase(0, 1, 10);
        assert_eq!(st.persons[1].favour(0), 60);
        st.persons[1].set_religion(1);
        st.favour_increase(0, 1, 11);
        assert_eq!(st.persons[1].favour(0), 65);
        st.favour_decrease(0, 1, 40);
        assert_eq!(st.persons[1].favour(0), 0);
        st.persons[1].set_favour(0, 101);
        st.favour_increase(0, 1, 10);
        assert_eq!(st.persons[1].favour(0), 101);
    }

    /// rules-03 §3 table vs Appendix I (docs/spec/08): Bürgermeister by the 3 Ratsherren, Domherr by
    /// the county Bischof, Erzbischof by the 3 Bischöfe, Vogt and Regent by lot.
    #[test]
    fn rules03_s3_appointers_match_appendix_i() {
        let mut st = State::blank(1);
        for (i, (l, x)) in [(5, 0), (5, 1), (5, 2), (11, 12), (10, 12), (12, 12)].into_iter().enumerate() {
            person(&mut st, i, l, x);
        }
        assert_eq!(st.appointers(5, 6), vec![0, 1, 2]);
        assert_eq!(st.appointers(5, 9), vec![3]);
        assert_eq!(st.appointers(13, 10), vec![4, 3, 5]);
        assert!(st.appointers(11, 7).is_empty());
        assert!(st.appointers(13, 7).is_empty());
    }

    /// rules-03 §2.1: title n needs NW >= THR[n] and residence >= n; Bürger comes in 1601 anyway.
    #[test]
    fn rules03_s2_1_title_award() {
        let mut st = State::new_world(3);
        let h = st.setup_human(&crate::world::HumanSetup::test(2)).unwrap();
        st.year = 1601;
        st.title_award(h);
        assert_eq!(st.persons[h].title(), 1);
        // without a residence the cap stays at 0: no further titles however rich
        st.add_cash(h, 1_000_000);
        for _ in 0..20 {
            st.title_award(h);
        }
        assert_eq!(st.persons[h].title(), 1);
    }

    /// rules-55 §5: a computer county holder may move only to his county place itself; Kanzler, Vogt and
    /// Bischof (6, 7, 12) also to place 13. His county's cities are refused.
    #[test]
    fn rules55_s5_county_holder_moves() {
        let mut st = State::new_world(3);
        let x = st.computers().next().unwrap();
        let city = (0..CITIES).find(|&c| crate::economy::COUNTY[c] == 10).unwrap();
        for p in 0..PERSONS {
            if p != x && st.persons[p].used() && (st.persons[p].loc() as usize == city || st.persons[p].loc() == 13) {
                st.vacate_silently(p);
            }
        }
        st.persons[x].set_title(9);
        st.persons[x].set_level_secular(40);
        st.persons[x].set_level_church(40);
        st.persons[x].set_loc(10);
        st.persons[x].set_idx(6);
        assert_eq!(st.eligibility(x, city, 0), 0, "a city of his own county");
        assert_ne!(st.eligibility(x, 13, 0), 0, "Kanzler may take the principality");
        st.persons[x].set_idx(0);
        assert_eq!(st.eligibility(x, 13, 0), 0, "other county offices stay in the county");
    }

    /// rules-03 §3.2 eligibility basics.
    #[test]
    fn rules03_s3_2_eligibility() {
        let mut st = State::new_world(3);
        let h = st.setup_human(&crate::world::HumanSetup::test(2)).unwrap();
        st.persons[h].set_title(1);
        for p in 0..PERSONS {
            if st.persons[p].used() && st.persons[p].loc() == 2 && st.persons[p].idx() == 0 {
                st.vacate_silently(p);
            }
        }
        assert_eq!(st.eligibility(h, 2, 0), 2, "Ratsherr in own branch city");
        assert_eq!(st.eligibility(h, 3, 0), 0, "no branch there");
        assert_eq!(st.eligibility(h, 2, 6), 0, "career level 2 < Bürgermeister 6");
        st.persons[h].set_title(0);
        assert_eq!(st.eligibility(h, 2, 0), 0, "title 0");
    }

    /// rules-03 §4.6: vacancies are filled and the cycle is deterministic for a seed.
    #[test]
    fn rules03_s4_6_vacancies_filled() {
        let mut a = State::new_world(9);
        a.year = 1601;
        let v = a.holder(4, 6).unwrap();
        a.vacate_silently(v);
        let mut b = a.clone();
        a.year_end_offices(&mut Auto);
        b.year_end_offices(&mut Auto);
        assert!(a == b);
        assert!(a.holder(4, 6).is_some(), "Bürgermeister refilled");
    }

    /// Ours: the RNG-free forecast names the winner the real election picks when no human and no rival is involved
    /// (the only random term then is the lot among tied leaders).
    #[test]
    fn forecast_matches_election() {
        let mut checked = 0;
        for seed in 1..40u32 {
            let mut st = State::new_world(seed);
            st.year = 1601;
            let (loc, seat) = (4, 6);
            let v = st.holder(loc, seat).unwrap();
            st.vacate_silently(v);
            let cands: Vec<usize> = st.computers().filter(|&x| st.eligibility(x, loc, seat) == 2).take(3).collect();
            for &c in &cands {
                st.apply_for(c, loc, seat, true);
            }
            let f = st.forecast(loc, seat);
            assert_eq!(f.candidates, cands);
            assert_eq!(f.electors.len(), 3);
            if f.leaders.len() != 1 {
                continue;
            }
            let mut ev = vec![];
            st.fill_vacancies(&[], &mut Auto, &mut ev);
            assert_eq!(st.holder(loc, seat), Some(f.candidates[f.leaders[0]]), "seed {seed}: {f:?}");
            checked += 1;
        }
        assert!(checked >= 10, "{checked}");
    }

    /// rules-65 §1/§2: a seat a human could take stays open for a year, but the drawn computers are registered at
    /// once and stand against him when he applies (playtest report: every application won without a vote).
    #[test]
    fn rules65_registered_computers_contest_the_human() {
        let mut st = State::new_world(3);
        let h = st.setup_human(&crate::world::HumanSetup::test(2)).unwrap();
        st.persons[h].set_title(1);
        st.year = 1601;
        for p in 0..PERSONS {
            if st.persons[p].used() && st.persons[p].loc() == 2 && st.persons[p].idx() <= 2 && st.persons[p].has_office() {
                st.vacate_silently(p);
            }
        }
        assert_eq!(st.eligibility(h, 2, 0), 2);
        st.year_end_offices(&mut Auto);
        assert!(st.holder(2, 0).is_none(), "held open for the human");
        let registered: Vec<usize> = st.applicants(2, &(0..3)).into_iter().filter(|&c| !st.persons[c].human()).collect();
        assert!(!registered.is_empty(), "computers registered at once");
        st.apply_for(h, 2, 0, true);
        let ev = st.year_end_offices(&mut Auto);
        let el = ev.iter().find_map(|e| match e {
            Event::Election { loc: 2, candidates, kind, .. } if candidates.contains(&(h as u8)) => Some((candidates.len(), *kind)),
            _ => None,
        });
        assert!(matches!(el, Some((n, k)) if n >= 2 && k != 2), "a real vote, not 'mangels weiterer Bewerber': {el:?}");
    }

    /// rules-40 §1: two empty Ratsherr seats run two elections in the same year end.
    #[test]
    fn rules40_s1_each_seat_filled() {
        let mut st = State::new_world(9);
        st.year = 1601;
        for i in [0, 1] {
            let v = st.holder(4, i).unwrap();
            st.vacate_silently(v);
        }
        st.year_end_offices(&mut Auto);
        assert!(st.holder(4, 0).is_some() && st.holder(4, 1).is_some());
    }

    /// rules-44 §3: the year end lists, for 1600 too, every office or seat group a human could be promoted to, each
    /// group once under its base index, in the documented order.
    #[test]
    fn rules44_next_elections() {
        let mut st = State::new_game(9, 3, &[crate::world::HumanSetup::test(4)]).unwrap();
        let h = st.humans().next().unwrap();
        st.persons[h].set_title(1);
        let c = st.props_of(h, KIND_BRANCH).next().map(|b| st.props[b].loc() as usize).unwrap();
        for i in [1, 2, 5] {
            if let Some(v) = st.holder(c, i) {
                st.vacate_silently(v);
            }
        }
        let want: Vec<(u8, u8)> = (0..14usize)
            .rev()
            .flat_map(|l| (0..=(9 + 3 * (l >= 10) as usize - 2 * (l == 13) as usize)).rev().map(move |i| (l, i)))
            .filter(|&(l, i)| st.eligibility(h, l, i) == 2)
            .map(|(l, i)| (l as u8, if i <= 2 { 0 } else if l < 10 && i == 8 { 7 } else { i as u8 }))
            .fold(vec![], |mut v: Vec<(u8, u8)>, x| {
                if v.last() != Some(&x) {
                    v.push(x);
                }
                v
            });
        assert!(!want.is_empty(), "a new player can be promoted somewhere");
        let ev = st.year_end(&mut Auto);
        assert!(ev.iter().any(|e| matches!(e, Event::NextElections(l) if *l == want)));
    }
}
