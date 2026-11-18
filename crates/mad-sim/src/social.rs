//! Feasts, duels, church (docs/re/rules-08-auction-feast-duel.md §2-§4).

use crate::economy::*;
use crate::heap::{self, ANY};
use crate::laws::global_office;
use crate::state::*;
use crate::{Ask, Choices, DuelScene, Event};

/// Duel effect samples (rules-25 §2): swings by `rand(7)` (0x53 is never loaded, so silent), hits by `rand(6)`, `rand(2)`.
const SWING: [u8; 7] = [0x83, 0x5F, 0x80, 0x81, 0x82, 0x83, 0x53];
const HIT: [[u8; 2]; 6] = [[0x39, 0x3A], [0x31, 0x3B], [0x32, 0x3C], [0x3D, 0x3E], [0x3E, 0x3F], [0x40, 0x41]];

impl State {
    /// rules-02 §7.11 office salary of p for this year.
    pub fn office_salary(&self, p: usize) -> i32 {
        let pr = &self.persons[p];
        if !pr.has_office() {
            return 0;
        }
        let o = global_office(pr);
        let l = if pr.loc() < 10 { 2 * self.cities[pr.loc() as usize].income_level() as i64 + 300 } else { 400 };
        (crate::career::SALARY[o] as i64 * self.year as i64 * l / 300000) as i32
    }

    /// 0x173D0 feast cost estimate.
    fn cost500(&mut self) -> i32 {
        ((70 + self.r(60)) * 500 * (self.year as u32 - 1570) / 3000 + 1) as i32
    }

    /// §2.1 plan (or cancel) a feast. `size` 0..3, `musicians` 0..4, `years` 0..10 ahead.
    pub fn plan_feast(&mut self, p: usize, branch: usize, size: u8, musicians: u8, years: u8) -> Option<Event> {
        if let Some(i) = self.heap.find(28, p as u8, ANY) {
            self.heap.delete(i);
            return None;
        }
        if !crate::laws::has(self, p, 8) || self.props[branch].owner() as usize != p || self.props[branch].kind() != KIND_BRANCH {
            return Some(Event::Refused("no feast"));
        }
        self.cost500();
        if let Some(i) = self.heap.alloc(28, p as u8, branch as u8, years.min(10) + 1) {
            self.heap.recs[i][4] = musicians.min(4);
            self.heap.recs[i][5] = size.min(3);
        }
        None
    }

    /// §2.2 hold the feast (type-28 record with counter 0).
    pub fn hold_feast(&mut self, ri: usize, ev: &mut Vec<Event>) {
        let r = self.heap.recs[ri].clone();
        let (p, b, mus, size) = (r[1] as usize, r[2] as usize, r[4] as i32, r[5] as u32);
        let c = (mus + 1) * 1000 + self.cost500();
        self.pay(p, c);
        let n = (self.office_salary(p) + self.standing(p, 1)) / 130;
        let mut g = 0u32;
        for _ in 0..n.max(0) {
            g += self.r(size + 2) + 2;
        }
        let take = |st: &mut State, need: u32, goods: [usize; 3]| -> bool {
            let mut need = need;
            for good in goods {
                let s = st.props[b].stock(good) as u32;
                let x = s.min(need);
                st.props[b].set_stock(good, (s - x) as u16);
                need -= x;
            }
            need > 0
        };
        let food_short = take(self, 3 * g, [5, 1, 0]) as i32;
        let drink_short = take(self, 2 * g, [WEIN, 2, 2]) as i32;
        let q = (3 - food_short - 2 * drink_short + mus / 2).clamp(0, 5) as u32;
        self.history(p, 2, q as u8, 0);
        if q == 5
            && let Some(i) = self.heap.find(0, p as u8, ANY)
                && self.heap.recs[i][4] == 12 {
                    let v = heap::r32(&self.heap.recs[i], 5) + g;
                    heap::w32(&mut self.heap.recs[i], 5, v);
                }
        for _ in 0..g / 4 {
            let x = self.r(192) as usize;
            let v = self.ri(q as i32 + 1);
            self.favour_increase(p, x, v);
        }
        self.heap.delete(ri);
        ev.push(Event::Feast { host: p as u8, guests: g, quality: q as u8, loc: self.props[b].loc(), cost: c, food_short: food_short != 0, drink_short: drink_short != 0 });
    }

    /// §3 insult (privilege 26). A human insulted person may refuse.
    pub fn insult(&mut self, p: usize, target: usize, ch: &mut dyn Choices) -> Option<Event> {
        if !crate::laws::has(self, p, 26) {
            return Some(Event::Refused("cannot insult"));
        }
        // rules-24 §1: an old insult of the same target may be taken back; any other pending duel stops a new one
        if let Some(i) = self.heap.find(22, p as u8, ANY).filter(|&i| self.heap.recs[i][4] as usize == target) {
            if ch.ask(&Ask::Apologise { target: target as u8 }) == 0 {
                self.heap.delete(i);
            }
            return None;
        }
        if self.heap.has(22, p as u8) {
            return Some(Event::DuelForbidden);
        }
        // rules-24 §1: rand(12) taunt, rand(2) voice, then rand(6) place (human target) or rand(8) adverb, rand(6) place
        let (taunt, voice) = (self.r(12) as u8, self.r(2) as u8);
        let mut told = None;
        if self.persons[target].human() {
            let place = self.r(6) as u8;
            if ch.ask(&Ask::AcceptDuel { insulter: p as u8, insulted: target as u8, rolls: [taunt, voice, 0, place] }) != 0 {
                return None; // rules-24 §1: a refusal has no further message
            }
        } else {
            let adverb = self.r(8) as u8;
            let place = self.r(6) as u8;
            told = Some(Event::Insulted { insulter: p as u8, target: target as u8, rolls: [taunt, voice, adverb, place] });
        }
        if let Some(i) = self.heap.alloc(22, p as u8, NONE, 1) {
            self.heap.recs[i][4] = target as u8;
        }
        told
    }

    /// §3 duel (0x2C750).
    pub fn duel(&mut self, ri: usize, ev: &mut Vec<Event>) {
        let (a, b) = (self.heap.recs[ri][1] as usize, self.heap.recs[ri][4] as usize);
        self.heap.delete(ri);
        if !self.persons[b].used() {
            return;
        }
        // rules-22 §9: the scene's rolls come first, in the original's order (all drawn whatever the UI shows)
        let place = self.r(11) as u8;
        let ans0 = self.r(8) as u8;
        let ans1 = ((ans0 as u32 + 1 + self.r(7)) % 8) as u8;
        let fog = self.r(8) as u8;
        for _ in 1..480 {
            self.r(9);
            self.r(25);
        }
        let n = 4 + self.r(5);
        let mut taunt = self.r(63);
        let (mut taunts, mut fx) = (vec![], vec![]);
        for _ in 0..n {
            for side in 0..2u32 {
                let y = self.r(240) + if side == 0 { 120 } else { 100 };
                let x = self.r(240) + if side == 0 { 20 } else { 150 };
                taunt = (taunt + 1 + self.r(62)) % 63;
                let swings = 4 + self.r(5);
                let (mut sounds, mut at) = (vec![], 0u16);
                for _ in 0..swings {
                    at += 100 + self.r(300) as u16;
                    sounds.push((at, SWING[self.r(7) as usize]));
                }
                let hits = 1 + self.r(2);
                for _ in 0..hits {
                    let c = self.r(2) as usize;
                    at += 100;
                    sounds.push((at, HIT[self.r(6) as usize][c]));
                }
                fx.push(sounds.into_iter().filter(|&(_, x)| x != 0x53).collect());
                taunts.push((side as u8, taunt as u8, x as u16, y as u16));
            }
        }
        let after = self.r(5) as u8;
        let score = |st: &mut State, x: usize, y: usize| {
            let l = if st.persons[x].human() { st.persons[x].life() as i32 } else { st.persons[y].life() as i32 + st.ri(10) };
            global_office(&st.persons[x]) as i32 + l + st.persons[x].fencing() as i32 + st.ri(20)
        };
        let sa = score(self, a, b);
        let sb = score(self, b, a);
        for _ in 1..480 {
            self.r(9);
            self.r(25);
        }
        let loser = if sa >= sb { b } else { a };
        let mut office_lost = false;
        // SPEC-GAP closed: office_index 0 is "no office" and also the first Ratsherr seat (rules-22 §7)
        if global_office(&self.persons[loser]) != 0 && self.r(100) <= 32 {
            self.persons[loser].set_loc(0);
            self.persons[loser].set_idx(15);
            office_lost = true;
        }
        if self.persons[loser].human() {
            let l = self.persons[loser].life().wrapping_sub(1 + self.r(5) as u8);
            self.persons[loser].set_life(l);
        }
        for (x, y) in [(a, b), (b, a)] {
            if self.persons[x].favour(y) <= 100 {
                self.persons[x].set_favour(y, 50);
            }
        }
        ev.push(Event::Duel { winner: if loser == a { b } else { a } as u8, loser: loser as u8, office_lost, scene: DuelScene { insulter: a as u8, place, answers: [ans0, ans1], fog, after, taunts, fx } });
    }

    /// §4 Konvertieren.
    pub fn convert(&mut self, p: usize) {
        let c = self.net_worth(p).max(0) / 30;
        self.pay(p, c);
        let r = if self.persons[p].religion() == 0 { 1 } else { 0 };
        self.persons[p].set_religion(r);
        self.turn_flag(0x100000);
    }

    /// §4 Austreten.
    pub fn leave_church(&mut self, p: usize) {
        let c = self.net_worth(p).max(0) / 10;
        self.pay(p, c);
        self.persons[p].set_religion(2);
        if self.persons[p].church() {
            let pr = &mut self.persons[p];
            pr.set_loc(0);
            pr.set_idx(15);
            pr.set_flags10(pr.flags10() & 0xF0);
        }
        for i in self.heap.all(5, p as u8, ANY) {
            let r = &self.heap.recs[i];
            if r[5] > 7 || (r[5] == 7 && r[4] < 10) {
                self.heap.delete(i);
            }
        }
        if let Some(i) = self.heap.find(30, p as u8, ANY) {
            let partner = self.heap.recs[i][9] as usize;
            self.favour_decrease(p, partner, 20);
            self.heap.delete(i);
        }
        self.turn_flag(0x100000);
    }

    pub fn turn_flag(&mut self, bit: u32) {
        let f = self.globals.d(ga::TURN_FLAGS) | bit;
        self.globals.set_d(ga::TURN_FLAGS, f);
    }

    /// §4 Ablass kaufen (0x59720).
    /// Price of the indulgence: 7 plus 1 % of net worth per kind of evidence others hold on `p`.
    pub fn indulgence_price(&self, p: usize) -> i32 {
        let e = self.heap.all(6, ANY, ANY).into_iter().filter(|&i| self.heap.recs[i][4] as usize == p).fold(0u32, |a, i| a | heap::r32(&self.heap.recs[i], 0x0B)).count_ones() as i64;
        (self.net_worth(p).max(0) as i64 * e / 100 + 7) as i32
    }

    pub fn indulgence(&mut self, p: usize) -> i32 {
        let spies: Vec<usize> = self.heap.all(6, ANY, ANY).into_iter().filter(|&i| self.heap.recs[i][4] as usize == p).collect();
        let price = self.indulgence_price(p);
        self.pay(p, price);
        if let Some(i) = self.heap.find(0, p as u8, ANY)
            && self.heap.recs[i][4] == 8 {
                let v = heap::r32(&self.heap.recs[i], 5).wrapping_add(price as u32);
                heap::w32(&mut self.heap.recs[i], 5, v);
            }
        if self.persons[p].has_office() {
            let a = self.globals.d(ga::INDULGENCES).wrapping_add(price as u32);
            self.globals.set_d(ga::INDULGENCES, a);
        }
        for i in spies {
            if self.heap.recs[i][9] == 0 {
                self.heap.delete(i);
            } else {
                heap::w32(&mut self.heap.recs[i], 0x0B, 0);
            }
        }
        self.turn_flag(0x200000);
        price
    }

    /// Kirchgang prices [candles, alms]: candles `0x85D41`, alms `0x85D3D` (rules-40 §12).
    pub fn church_prices(&self) -> [i32; 2] {
        [self.globals.d(0x85D41) as i32, self.globals.d(0x85D3D) as i32]
    }
    pub fn set_church_prices(&mut self, p: [i32; 2]) {
        self.globals.set_d(0x85D41, p[0] as u32);
        self.globals.set_d(0x85D3D, p[1] as u32);
    }

    /// §4 Kerzen stiften / Spenden, with the turn-start prices.
    pub fn candles(&mut self, p: usize) {
        self.pay(p, self.church_prices()[0]);
        for x in 0..PERSONS {
            if self.persons[x].used() && self.persons[x].church() {
                // favour_increase (rules-39 §10): halved across religions, cap 100, blackmail values untouched
                let v = self.ri(3);
                self.favour_increase(p, x, v);
            }
        }
    }

    pub fn alms(&mut self, p: usize) {
        self.pay(p, self.church_prices()[1]);
        for x in self.computers().collect::<Vec<_>>() {
            let add = self.ri(6);
            if !self.persons[x].church() {
                self.favour_increase(p, x, add);
            }
        }
    }

    pub fn pray(&mut self) {
        let v = self.globals.b(ga::MIRACLE_CHANCE).wrapping_add(1);
        self.globals.set_b(ga::MIRACLE_CHANCE, v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::HumanSetup;

    /// rules-08 §3: the insulter wins ties; favours reset to 50.
    #[test]
    fn rules08_s3_duel() {
        let mut st = State::new_game(1, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        let i = st.heap.alloc(22, h as u8, NONE, 1).unwrap();
        st.heap.recs[i][4] = 5;
        let mut ev = vec![];
        st.duel(i, &mut ev);
        assert!(matches!(ev[0], Event::Duel { .. }));
        assert_eq!(st.persons[h].favour(5), 50);
        assert_eq!(st.persons[5].favour(h), 50);
    }

    /// rules-24 §1: a human who refuses the duel ends it without a message and without a record.
    #[test]
    fn rules24_s1_refused_duel_is_silent() {
        struct No;
        impl Choices for No {
            fn ask(&mut self, _: &Ask) -> u32 {
                1
            }
        }
        let mut st = State::new_game(1, 3, &[HumanSetup::test(1), HumanSetup::test(3)]).unwrap();
        let (h, v) = (st.current as usize, st.humans().find(|&p| p != st.current as usize).unwrap());
        st.persons[h].set_loc(10);
        st.persons[h].set_idx(0);
        assert_eq!(st.insult(h, v, &mut No), None);
        assert!(!st.heap.has(22, h as u8));
    }

    /// rules-08 §2.2: an empty branch gives food and drink shortage: q = 3 - 1 - 2 + B/2.
    #[test]
    fn rules08_s2_2_feast_quality() {
        let mut st = State::new_game(1, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        st.persons[h].set_loc(1);
        st.persons[h].set_idx(6);
        let b = st.branch_at(h, 1).unwrap();
        st.plan_feast(h, b, 1, 4, 0);
        let i = st.heap.find(28, h as u8, ANY).unwrap();
        st.heap.recs[i][3] = 0;
        let mut ev = vec![];
        st.hold_feast(i, &mut ev);
        let Event::Feast { quality, guests, .. } = ev[0] else { panic!() };
        assert_eq!(quality, if guests == 0 { 5 } else { 2 });
    }

    /// rules-08 §4: leaving the church costs NW/10 and drops a church office.
    #[test]
    fn rules08_s4_leave_church() {
        let mut st = State::new_game(1, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        st.persons[h].set_loc(1);
        st.persons[h].set_idx(9);
        st.persons[h].set_flags10(0x11);
        st.leave_church(h);
        assert_eq!(st.persons[h].religion(), 2);
        assert!(!st.persons[h].has_office());
    }
}
