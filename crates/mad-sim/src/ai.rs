//! Computer persons (docs/re/rules-09-ai.md §2-§4): year-end triggers, `npc_act`, revenge.

use crate::economy::*;
use crate::heap::{self, ANY};
use crate::laws::{add_evidence, has};
use crate::state::*;
use crate::{Event, WArg};

impl State {
    /// `person_pick_random(loc, computer)` (rules-24 §2): one `rand(count)` over computer persons whose place byte is `loc` (any place for None).
    pub fn pick_computer(&mut self, loc: Option<u8>) -> Option<usize> {
        let pool: Vec<usize> = self.computers().filter(|&x| loc.is_none_or(|l| self.persons[x].loc() == l)).collect();
        if pool.is_empty() {
            return None;
        }
        let p = pool[self.r(pool.len() as u32) as usize];
        self.last_person = p as u8;
        Some(p)
    }

    /// §2 steps 2..5.
    pub fn npc_turns(&mut self, ev: &mut Vec<Event>) {
        for h in self.humans().collect::<Vec<_>>() {
            if self.persons[h].has_office() && self.r(100) < 40 {
                let l = self.persons[h].loc();
                if let Some(p) = self.pick_computer(Some(l)) {
                    self.npc_act(p, ev);
                }
            }
        }
        for c in 10..13u8 {
            if self.r(100) < 50
                && let Some(p) = self.pick_computer(Some(c)) {
                    self.npc_act(p, ev);
                }
        }
        if self.r(100) < 90
            && let Some(p) = self.pick_computer(Some(13)) {
                self.npc_act(p, ev);
            }
        for i in self.heap.all(6, ANY, ANY) {
            let r = &self.heap.recs[i];
            let (o, t) = (r[1] as usize, r[4] as usize);
            if r[9] != 0 && o < PERSONS && self.persons[o].human() && self.persons[t].used() && !self.persons[t].human() && self.r(100) < 50 {
                self.npc_act(t, ev);
            }
        }
    }

    /// Rotate the holders of indices 3, 4, 5 at `loc` (rules-04b §4.3, 40/69).
    pub fn rotate_advisers(&mut self, loc: usize) {
        let hs: Vec<(usize, u8)> = (3..6).filter_map(|i| self.holder(loc, i).map(|p| (p, i as u8))).collect();
        for (p, i) in hs {
            self.persons[p].set_idx((i + 1 - 3) % 3 + 3);
        }
    }

    /// §3 `npc_act(p)` (0x59F80).
    pub fn npc_act(&mut self, p: usize, ev: &mut Vec<Event>) {
        if self.persons[p].human() {
            return;
        }
        let mut w = 0u32;
        // Step A, law change
        if self.r(100) < 40 {
            // rules-32 §3.1: one line per law book, printed before k is drawn
            let k = if has(self, p, 56) && self.r(100) < 80 {
                line(ev, 0xB0A87, vec![WArg::P(p as u8)]);
                Some(self.r(10) as usize)
            } else if has(self, p, 59) && self.r(100) < 90 {
                line(ev, 0xB0B39, vec![WArg::P(p as u8)]);
                Some(10 + self.r(10) as usize)
            } else if has(self, p, 82) {
                line(ev, 0xAF0E7, vec![WArg::P(p as u8)]);
                Some(20 + self.r(10) as usize)
            } else {
                None
            };
            if let Some(k) = k {
                match k {
                    1 => {
                        let v = self.r(33) as u8;
                        self.globals.set_b(0x85A22, v);
                    }
                    5 | 6 => {
                        let v = 50 + self.r(100) as u8;
                        self.globals.set_b(0x85A21 + k, v);
                    }
                    7 => {
                        let v = 200_000 + 1000 * self.r(700);
                        self.globals.set_d(0x85A28, v);
                    }
                    8 => {}
                    29 => {
                        let v = self.r(25) as u8;
                        self.globals.set_b(0x85A41, v);
                    }
                    _ => {
                        let a = if k <= 7 { 0x85A21 + k } else { 0x85A24 + k };
                        let v = self.globals.b(a) ^ 1;
                        self.globals.set_b(a, v);
                    }
                }
                if self.globals.forbidden(0, 3) {
                    w |= 0x08;
                }
                add_evidence(self, p, w);
                return;
            }
        }
        // Step B
        for _ in 0..10 {
            let c = self.r(24);
            let done = match c {
                0 => self.r(100) >= 40 && has(self, p, 5),
                1 => {
                    if self.r(100) >= 30 && has(self, p, 8) {
                        let b = self.r(5) as i32;
                        let city = self.r(10) as u8;
                        if self.human_at(city) {
                            line(ev, 0x9DD6A, vec![WArg::P(p as u8), WArg::Id(2022 + b as u32), WArg::Loc(city)]);
                        }
                        let cost = 10000 - b * self.ri(2000);
                        self.pay(p, cost);
                        for x in 0..PERSONS {
                            if self.persons[x].used() && self.persons[x].loc() == city {
                                self.favour_increase(p, x, 4 - b);
                            }
                        }
                        if self.globals.forbidden(2, 6) {
                            w |= 0x4000000;
                        }
                        true
                    } else {
                        false
                    }
                }
                2 => {
                    if self.r(100) >= 20 && has(self, p, 9) {
                        let b = self.r(5);
                        let city = self.r(10) as usize;
                        if self.rights_holder(city).is_some() {
                            line(ev, 0xA4127, vec![WArg::P(p as u8), WArg::Loc(city as u8), WArg::Id(2520 + b)]);
                        }
                        let cost = 10000 - self.ri(2000) * b as i32;
                        self.pay(p, cost);
                        let wl = (self.cities[city].wealth() as u32 + 10 - 2 * b).min(99) as u8;
                        self.cities[city].set_wealth(wl);
                        true
                    } else {
                        false
                    }
                }
                3 => self.npc_case3(p, ev),
                4 => {
                    let loc = self.persons[p].loc() as usize;
                    if has(self, p, 32) && (10..13).contains(&loc) {
                        let a = ga::TOLL_SHARE + loc - 10;
                        let v = ((self.globals.b(a) as u32 + 28 + self.r(10)) % 33) as u8;
                        self.globals.set_b(a, v);
                        line(ev, 0xAEC8E, vec![WArg::P(p as u8), WArg::Loc(loc as u8 + 4), WArg::Num(v as i32)]);
                        true
                    } else if !has(self, p, 54) && has(self, p, 84) {
                        line(ev, 0xA9794, vec![WArg::P(p as u8), WArg::Loc(loc as u8), WArg::Id(2865 + self.persons[p].religion() as u32 % 2)]);
                        if let Some(i) = self.heap.alloc(24, p as u8, NONE, 5) {
                            self.heap.recs[i][4] = loc as u8;
                            self.heap.recs[i][5] = self.persons[p].religion() % 2;
                        }
                        true
                    } else if has(self, p, 61) {
                        line(ev, 0xA73E2, vec![WArg::P(p as u8)]);
                        let v = self.r(100) as u8;
                        self.globals.set_b(ga::HARSHNESS, v);
                        true
                    } else {
                        false
                    }
                }
                5 => {
                    if has(self, p, 55) || has(self, p, 86) {
                        line(ev, 0xA1FFE, vec![WArg::P(p as u8)]);
                        self.divine(ev);
                        true
                    } else if has(self, p, 64) {
                        let v = self.globals.b(ga::INQUISITION) ^ 1;
                        self.globals.set_b(ga::INQUISITION, v);
                        line(ev, 0xAB8D0, vec![WArg::P(p as u8), WArg::Id(2981 + v as u32)]);
                        true
                    } else if has(self, p, 69) {
                        line(ev, 0xAEFAF, vec![WArg::P(p as u8)]);
                        // rules-32 §3.2: the rotation runs at loc(p)
                        self.rotate_advisers(self.persons[p].loc() as usize);
                        true
                    } else if has(self, p, 71) {
                        let v = ((self.globals.b(ga::STATE_RELIGION) as u32 + 1 + self.r(2)) % 3) as u8;
                        self.globals.set_b(ga::STATE_RELIGION, v);
                        line(ev, 0xA98DF, vec![WArg::P(p as u8), WArg::Id(2871 + v as u32)]);
                        if self.globals.forbidden(2, 0) {
                            w |= 0x100000;
                        }
                        true
                    } else {
                        false
                    }
                }
                6 => {
                    if has(self, p, 85) {
                        // rules-32 §3.2: the saint is a random human, from the persons with controller 1 (rules-67 §2)
                        let humans: Vec<usize> = (0..PERSONS).filter(|&x| self.persons[x].used() && self.persons[x].flags10() >> 4 == 1).collect();
                        if humans.is_empty() {
                            continue;
                        }
                        let r = humans[self.r(humans.len() as u32) as usize];
                        line(ev, 0xAD376, vec![WArg::P(p as u8), WArg::P(r as u8), WArg::P(r as u8)]);
                        // the computer clergy, church nibble 1 (rules-67 §2)
                        let used: Vec<usize> = self.computers().collect();
                        for x in used {
                            if self.persons[x].flags10() & 0x0F == 1 {
                                let v = self.ri(10);
                                self.favour_increase(r, x, v);
                            }
                        }
                        if self.globals.forbidden(0, 3) {
                            w |= 0x08;
                        }
                        true
                    } else {
                        false
                    }
                }
                7..=11 => {
                    let (g, bit, book, law) = [(14, 0x2000, 1, 3), (7, 0x4000, 1, 4), (1, 0x8000, 1, 5), (11, 0x8000000, 2, 7), (12, 0x10000000, 2, 8)][(c - 7) as usize];
                    let escorted = self.npc_shipment(p, g) == Some(3);
                    if self.globals.forbidden(book, law) {
                        w |= bit;
                    }
                    // an escorted shipment while Eskortierung is outlawed (law book 1, row 7; rules-66 §2)
                    if escorted && self.globals.forbidden(1, 7) {
                        w |= 0x20000;
                    }
                    true
                }
                // not an action: the try is used up and the loop goes on (rules-66 §2)
                12 => {
                    if self.globals.forbidden(0, 2) {
                        w |= 0x04;
                    }
                    false
                }
                13 | 14 => {
                    let g = [0, 2, 3, 4, 5, 6, 8, 9, 10, 13][self.r(10) as usize];
                    if self.npc_shipment(p, g) == Some(3) && self.globals.forbidden(1, 7) {
                        w |= 0x20000;
                    }
                    true
                }
                15 => {
                    if self.r(100) >= 30 && has(self, p, 27) && self.persons[p].fencing() < 80 {
                        let f = self.persons[p].fencing() as u32 + 5 + self.r(5) + self.r(10);
                        self.persons[p].set_fencing(f as u8);
                        if self.human_at(self.persons[p].loc()) {
                            line(ev, 0xAEB23, vec![WArg::P(p as u8)]);
                        }
                        true
                    } else {
                        false
                    }
                }
                16 => {
                    if self.r(100) >= 40 && has(self, p, 25) {
                        let i = 2 * self.r(2) + 1;
                        let bit = if i == 1 { 2 } else { 8 };
                        let v = self.persons[p].insurance() ^ bit;
                        self.persons[p].set_insurance(v);
                        if v & bit != 0 && self.human_at(self.persons[p].loc()) {
                            line(ev, 0xA5BF2, vec![WArg::P(p as u8), WArg::Id(2634 + i)]);
                        }
                    }
                    // the insurance toggle is not an action: the try loop goes on (rules-67 §2)
                    false
                }
                _ => {
                    // 17..23 [L mapping]: offence bits only, not counted as the action
                    let (bit, forb) = match c {
                        17 => (0x01, self.globals.forbidden(0, 0)),
                        18 => (0x04, self.globals.forbidden(0, 2)),
                        19 => (0x08, self.globals.forbidden(0, 3)),
                        20 => (0x200, self.globals.forbidden(0, 9)),
                        21 => (0x20, true),
                        22 => (0x800000, self.globals.forbidden(2, 3)),
                        _ => (0x1000000, self.globals.forbidden(2, 4)),
                    };
                    if forb {
                        w |= bit;
                    }
                    false
                }
            };
            if done {
                break;
            }
        }
        add_evidence(self, p, w);
    }

    fn npc_case3(&mut self, p: usize, ev: &mut Vec<Event>) -> bool {
        let loc = self.persons[p].loc() as usize;
        let pa = WArg::P(p as u8);
        if has(self, p, 16) && loc < CITIES {
            line(ev, 0xA7326, vec![pa, WArg::Loc(loc as u8)]);
            let v = ((self.cities[loc].building_level() as u32 + 80 + 40 * self.r(2)) % 100) as u8;
            self.cities[loc].set_building_level(v);
        } else if has(self, p, 31) && (10..13).contains(&loc) {
            let a = ga::MAX_INTEREST + loc - 10;
            let v = ((self.globals.b(a) as u32 + 55 + self.r(10)) % 60) as u8;
            self.globals.set_b(a, v);
            line(ev, 0xAEBBB, vec![pa, WArg::Loc(loc as u8), WArg::Num(v as i32)]);
        } else if has(self, p, 40) {
            line(ev, 0xAED7A, vec![pa, WArg::Loc(loc as u8)]);
            self.rotate_advisers(loc);
        } else if has(self, p, 53) && (10..13).contains(&loc) {
            line(ev, 0xACF76, vec![pa, WArg::Loc(loc as u8)]);
            match self.heap.find(23, p as u8, ANY) {
                Some(i) => self.heap.recs[i][3] = 5,
                None => {
                    if let Some(i) = self.heap.alloc(23, p as u8, NONE, 5) {
                        self.heap.recs[i][4] = loc as u8;
                    }
                }
            }
        } else if has(self, p, 58) {
            // rules-32 §3.2: the project is a random human's, from the persons with controller 1 (rules-67 §2); none, no action
            let humans: Vec<usize> = (0..PERSONS).filter(|&x| self.persons[x].used() && self.persons[x].flags10() >> 4 == 1).collect();
            if humans.is_empty() {
                return false;
            }
            let x = humans[self.r(humans.len() as u32) as usize];
            let s = self.standing(x, 20);
            self.add_cash(x, s);
            line(ev, 0xAD06F, vec![pa, WArg::P(x as u8), WArg::Num(s)]);
        } else {
            return false;
        }
        true
    }

    /// §3 trade shipment of computer person p.
    /// Returns the record's kind (3 = escorted), or None when no branch exists.
    fn npc_shipment(&mut self, p: usize, g: usize) -> Option<u8> {
        let avg = self.globals.d(ga::AVG_SHIPMENT) as i32;
        let mut qty = avg * (40 + self.ri(120)) / ((50 * g as i32) / 4 + 50);
        // the minimum compares the low 16 bits (rules-67 §2)
        if qty & 0xFFFF < 100 {
            qty = 100;
        }
        // `property_pick_random(branch, any, any)`: one draw over every branch of every owner (rules-66 §2)
        let branches: Vec<usize> = (0..PROPS).filter(|&i| self.props[i].kind() == KIND_BRANCH).collect();
        if branches.is_empty() {
            return None;
        }
        let src = branches[self.r(branches.len() as u32) as usize];
        self.last_pick = self.props[src].loc();
        let sc = self.props[src].loc() as usize;
        // the draw picks the kind (2 plain, 3 escorted); the countdown is always 2 (1619 parity, rec `c1 [03 …]`)
        let kind = 2 + self.r(2) as u8;
        let dest = ((sc as u32 + 1 + self.r(9)) % 10) as usize;
        let (mut t, mut n) = (KARREN, (qty + 99) / 100);
        if navigable(sc) && navigable(dest) {
            t = SCHIFF;
            qty *= 3;
            n = (qty + 999) / 1000;
        }
        let qty = qty.min(u16::MAX as i32);
        if let Some(i) = self.heap.alloc(9, p as u8, src as u8, 2) {
            let r = &mut self.heap.recs[i];
            r[4] = kind;
            heap::w16(r, 5, qty as u16);
            r[7] = g as u8;
            r[8] = n as u8; // truncated to a byte (rules-67 §2)
            r[9] = t as u8;
            r[10] = dest as u8;
            r[11] = 90;
        }
        if dest != 0 {
            let k = COUNTY[dest] as usize - 10;
            let share = self.globals.b(ga::TOLL_SHARE + k) as i32;
            let toll = self.castles[k].toll_rate as i32 * qty * value(g) / 100;
            self.castles[k].acc = self.castles[k].acc.wrapping_add(((100 - share) * toll / 100) as u32);
        }
        Some(kind)
    }

    /// rules-04b §3.1 computer trial creation (also revenge cases 3, 4).
    pub fn npc_sue(&mut self, a: usize, v: usize) -> bool {
        let lvl = |p: &Person| match (p.has_office(), p.loc()) {
            (false, _) | (true, 0..=9) => 0,
            (true, 10..=12) => 1,
            _ => 2,
        };
        if lvl(&self.persons[v]) > lvl(&self.persons[a]) {
            return false;
        }
        // a running trial against v ends the try as done, without a new record (rules-45 §1)
        if self.heap.all(18, a as u8, ANY).into_iter().any(|i| self.heap.recs[i][4] as usize == v) {
            return true;
        }
        let ci = match self.spy_contract(a, v) {
            Some(i) => {
                self.heap.recs[i][9] = 1;
                i
            }
            None => {
                let Some(i) = self.heap.alloc(6, a as u8, NONE, 10) else { return false };
                self.heap.recs[i][4] = v as u8;
                self.heap.recs[i][9] = 1;
                i
            }
        };
        // no evidence and the 10 % roll fails: not done (rules-45 §1); the trial counter is 1..4
        if heap::r32(&self.heap.recs[ci], 0x0B) == 0 && self.r(100) >= 10 {
            return false;
        }
        let c = self.r(4) as u8 + 1;
        if let Some(i) = self.heap.alloc(18, a as u8, NONE, c) {
            self.heap.recs[i][4] = v as u8;
            self.heap.recs[i][5] = has(self, a, 81) as u8;
        }
        true
    }

    /// §4 revenge handler for a type-50 record after the countdown.
    pub fn revenge_tick(&mut self, ri: usize, ch: &mut dyn crate::Choices, ev: &mut Vec<Event>) {
        let r = self.heap.recs[ri].clone();
        if r[3] == 0 {
            self.heap.delete(ri);
            return;
        }
        let (a, v) = (r[1] as usize, r[4] as usize);
        if !self.persons[v].used() {
            return;
        }
        let mut w = 0u32;
        for _ in 0..10 {
            let c = self.r(10);
            let done = match c {
                0 => {
                    if let Some(i) = self.heap.alloc(7, a as u8, NONE, 5) {
                        self.heap.recs[i][4] = v as u8;
                    }
                    true
                }
                1 => {
                    self.revenge_military(a, v, ev);
                    false // the only case that never ends the year's revenge [C]
                }
                2 => {
                    let x = self.ri(30);
                    self.favour_decrease(v, a, x);
                    true
                }
                3 | 4 => self.npc_sue(a, v),
                5 => match self.harass(a, v, &mut w, ev) {
                    Some(done) => done,
                    None => self.duel_challenge(a, v, ch), // rules-09 §4b row 8
                },
                6 => self.duel_challenge(a, v, ch),
                7 => {
                    if !has(self, a, 28) {
                        false
                    } else {
                        match self.pick_property(KIND_BRANCH, v) {
                            Some(x) if !self.city_blocked(self.props[x].loc() as usize) => {
                                let c = self.props[x].loc();
                                let army = (self.cities[c as usize].population() / 4) as u16;
                                if let Some(i) = self.heap.alloc(25, a as u8, NONE, 4) {
                                    self.heap.recs[i][4] = c;
                                    heap::w16(&mut self.heap.recs[i], 5, army);
                                }
                                self.pay(a, 20000);
                                true
                            }
                            _ => false,
                        }
                    }
                }
                8 => {
                    let s = self.heap.find(25, v as u8, ANY);
                    match s.filter(|_| has(self, a, 29)) {
                        Some(s) => {
                            let c = self.heap.recs[s][4];
                            if self.heap.all(26, a as u8, ANY).into_iter().any(|i| self.heap.recs[i][4] == c) {
                                false
                            } else {
                                let army = (self.cities[c as usize].population() / 6) as u16;
                                if let Some(i) = self.heap.alloc(26, a as u8, NONE, 4) {
                                    self.heap.recs[i][4] = c;
                                    heap::w16(&mut self.heap.recs[i], 5, army);
                                }
                                true
                            }
                        }
                        None => false,
                    }
                }
                _ => {
                    // the denunciation (rules-45 §1): one computer person at the actor's place, the actor included,
                    // one rand(n) also for n = 1; a or v drawn ends the try undone
                    let loc = self.persons[a].loc();
                    let pool: Vec<usize> = self.computers().filter(|&x| self.persons[x].loc() == loc).collect();
                    let rr = (!pool.is_empty()).then(|| pool[self.r(pool.len() as u32) as usize]);
                    match rr.filter(|&rr| rr != a && rr != v) {
                        Some(rr) => {
                            line(ev, 0xBFCFC, vec![WArg::P(a as u8), WArg::P(v as u8), WArg::P(rr as u8)]);
                            let x = self.ri(30);
                            self.favour_decrease(v, rr, x);
                            true
                        }
                        None => false,
                    }
                }
            };
            if done {
                break;
            }
        }
        add_evidence(self, a, w);
    }

    /// §4 case 1, military (never ends the year's revenge).
    fn revenge_military(&mut self, a: usize, v: usize, ev: &mut Vec<Event>) {
        let order = |st: &mut State, t: u8, home: usize, b4: u8, lo: u32, span: u32| {
            if let Some(i) = st.heap.alloc(t, a as u8, home as u8, 2) {
                st.heap.recs[i][4] = b4;
                for k in 0..4 {
                    let n = st.props[home].troops(k) as u32 * (lo + st.r(span)) / 100;
                    st.heap.recs[i][5 + k] = n as u8;
                }
            }
        };
        if let Some(k) = self.pick_property(KIND_CAMP, a) {
            if let Some(cs) = self.pick_property(KIND_CASTLE, v) {
                let l = self.props[cs].loc() - 14;
                order(self, 51, k, l, 50, 30);
            } else {
                let c = self.r(3) as u8;
                order(self, 54, k, c, 20, 40);
            }
        } else if let Some(k) = self.pick_property(KIND_CASTLE, a) {
            if let Some(cp) = self.pick_property(KIND_CAMP, v) {
                let l = self.props[cp].loc() - 14;
                order(self, 51, k, l, 50, 40);
            } else {
                let ci = self.props[k].loc() as usize - 14;
                let rate = self.castles[ci].toll_rate as u32;
                if rate < 14 {
                    let nr = (rate + 1 + self.r(14 - rate)) as u8;
                    self.castles[ci].toll_rate = nr;
                    line(ev, 0xBFCC0, vec![WArg::P(a as u8), WArg::Loc(self.props[k].loc()), WArg::Num(nr as i32)]);
                }
            }
        } else if self.persons[a].loc() >= 10 {
            let l = 14 + self.r(7) as u8;
            // every slot with place byte l, any kind and owner, one rand(count) (rules-40 §10)
            let at: Vec<usize> = (0..PROPS).filter(|&p| self.props[p].loc() == l).collect();
            if !at.is_empty() {
                let x = at[self.r(at.len() as u32) as usize];
                if self.heap.find(17, a as u8, x as u8).is_none() {
                    self.heap.alloc(17, a as u8, x as u8, 4);
                }
            }
        }
    }

    /// §4 revenge c = 6: the duel challenge.
    fn duel_challenge(&mut self, a: usize, v: usize, ch: &mut dyn crate::Choices) -> bool {
        let ok = has(self, a, 26)
            && (self.persons[v].fencing() as u32) < self.persons[a].fencing() as u32 + 20
            && !self.heap.all(22, a as u8, ANY).into_iter().any(|i| self.heap.recs[i][4] as usize == v);
        if ok {
            // rules-39 §2: taunt, voice, reason, occasion
            let rolls = [12, 2, 8, 6].map(|m| self.r(m) as u8);
            if ch.ask(&crate::Ask::AcceptDuel { insulter: a as u8, insulted: v as u8, rolls }) == 0 {
                if let Some(i) = self.heap.alloc(22, a as u8, NONE, 1) {
                    self.heap.recs[i][4] = v as u8;
                }
            } else {
                self.favour_decrease(v, a, 10);
                if self.persons[v].has_office() {
                    let l = self.persons[v].loc();
                    // every person at b's place (rules-39 §2: persons(loc, any, any, any))
                    for x in 0..PERSONS {
                        if self.persons[x].used() && self.persons[x].loc() == l {
                            let d = 1 + self.ri(6);
                            self.favour_decrease(v, x, d);
                        }
                    }
                }
            }
        }
        ok
    }

    /// rules-09 §4b harassment (revenge c = 5): the first privilege held picks the row; a failed row
    /// ends the try (`Some(false)`); no privilege falls through to the duel challenge (`None`).
    fn harass(&mut self, a: usize, v: usize, w: &mut u32, ev: &mut Vec<Event>) -> Option<bool> {
        let loc = self.persons[a].loc() as usize;
        let branch = if loc < CITIES { self.branch_at(v, loc) } else { None };
        let pa = WArg::P(a as u8);
        let church_bit = |st: &State, w: &mut u32| if st.globals.b(0x85A24) != 0 { *w |= 0x08 };
        if has(self, a, 14) {
            if branch.is_none() || self.cities[loc].wage_level() >= 100 {
                return Some(false);
            }
            self.cities[loc].set_wage_level(100);
            line(ev, 2723, vec![pa, WArg::Loc(loc as u8)]);
        } else if has(self, a, 15) {
            // unreachable in the original (14 and 15 share their condition), kept for completeness
            if self.persons[v].loc() as usize != loc || loc >= CITIES || self.cities[loc].income_level() == 0 {
                return Some(false);
            }
            self.cities[loc].set_income_level(0);
            if self.human_at(loc as u8) {
                line(ev, 3107, vec![pa, WArg::Loc(loc as u8)]);
            }
        } else if has(self, a, 17) {
            if branch.is_none() || self.cities[loc].storage_level() >= 100 {
                return Some(false);
            }
            self.cities[loc].set_storage_level(100);
            line(ev, 3110, vec![pa, WArg::Loc(loc as u8)]);
        } else if has(self, a, 22) {
            let t = if loc < CITIES { self.cities[loc].tax() as u32 } else { 33 };
            if branch.is_none() || t >= 33 {
                return Some(false);
            }
            let nt = t + self.r(33 - t);
            self.cities[loc].set_tax(nt as u8);
            line(ev, 3113, vec![pa, WArg::Loc(loc as u8), WArg::Num(nt as i32)]);
            if nt > self.globals.law(0, 1) {
                *w |= 0x02;
            }
        } else if let Some(keep) = [(48, 80), (74, 70), (79, 60)].into_iter().find(|&(c, _)| has(self, a, c)).map(|(_, k)| k) {
            line(ev, 2974, vec![pa, WArg::Num(100 - keep), WArg::P(v as u8)]);
            for i in self.heap.all(8, v as u8, ANY) {
                let wk = self.heap.recs[i][5] as u32 * keep as u32 / 100;
                self.heap.recs[i][5] = wk as u8;
            }
            church_bit(self, w);
        } else if has(self, a, 51) && (10..13).contains(&loc) {
            let addr = ga::TITHE + loc - 10;
            let t = (self.globals.b(addr) as u32 + self.r(10)).min(25) as u8;
            self.globals.set_b(addr, t);
            line(ev, 3128, vec![pa, WArg::Loc(loc as u8), WArg::Num(t as i32)]);
            church_bit(self, w);
        } else if has(self, a, 80) {
            // unreachable in the original (79 and 80 share their condition)
            let f = self.computers().filter(|&x| self.persons[x].favour(v) != 101 && self.persons[x].favour(v) > 0).max_by_key(|&x| (self.persons[x].favour(v), std::cmp::Reverse(x)));
            let d = 30 + self.ri(20);
            if let Some(f) = f {
                self.favour_decrease(v, f, d);
                line(ev, 2803, vec![pa, WArg::P(v as u8), WArg::P(f as u8)]);
            }
            church_bit(self, w);
        } else {
            return None;
        }
        Some(true)
    }
}

/// A year-end window line (rules-32).
fn line(ev: &mut Vec<Event>, text: u32, args: Vec<WArg>) {
    ev.push(Event::WorldText { text, args });
}

impl State {
    /// Some human has `loc` as his office/home location (0x433E0 with controller 1).
    pub fn human_at(&self, loc: u8) -> bool {
        self.humans().any(|h| self.persons[h].loc() == loc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rules-09 §4: retaliate only from computer actors against human victims, once.
    #[test]
    fn rules09_s4_retaliate_once() {
        let mut st = State::new_game(1, 3, &[crate::world::HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        st.retaliate(5, h, 3);
        st.retaliate(5, h, 4);
        st.retaliate(h, 5, 3);
        assert_eq!(st.heap.all(50, ANY, ANY).len(), 1);
    }

    /// rules-04b §4.3 rotation 3→4, 4→5, 5→3.
    #[test]
    fn rules04b_s4_3_rotate() {
        let mut st = State::new_world(1);
        let (a, b, c) = (st.holder(11, 3).unwrap(), st.holder(11, 4).unwrap(), st.holder(11, 5).unwrap());
        st.rotate_advisers(11);
        assert_eq!((st.holder(11, 4), st.holder(11, 5), st.holder(11, 3)), (Some(a), Some(b), Some(c)));
    }

    /// npc_act runs for every computer office holder without panicking and stays deterministic.
    #[test]
    fn rules09_s3_npc_act_smoke() {
        let mut a = State::new_world(4);
        let mut ev = vec![];
        for p in 100..150 {
            a.npc_act(p, &mut ev);
        }
        let mut b = State::new_world(4);
        for p in 100..150 {
            b.npc_act(p, &mut ev);
        }
        assert!(a == b);
    }
}
