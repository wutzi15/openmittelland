//! Events (docs/re/rules-06-events.md §2-§4): city disasters, divine events, world fate, the
//! 100 fate events. Sub-plots (§5) and contests (§6) are not implemented.

use crate::economy::*;
use crate::heap::{self, ANY};
use crate::state::*;
use crate::Event;

/// Disaster table `0x84B34` (§2): pop % max, wealth factor, goods kept %, wood %, stone %.
const DISASTER: [(u32, u32, u32, u32, u32); 4] = [(25, 4, 70, 60, 80), (20, 3, 75, 75, 85), (40, 2, 85, 90, 90), (20, 3, 90, 75, 80)];
/// Perishable goods (`0x84BDD`, fate event 31).
const PERISHABLE: [usize; 9] = [0, 1, 3, 4, 5, 8, 10, 11, 13];
/// Residence base values `0x86698` (§4 helpers).
const RB: [i64; 10] = [500, 1200, 2500, 4000, 6000, 8000, 11000, 15000, 19000, 25000];

impl State {
    pub fn skill_up(&mut self, p: usize, g: usize, x: u32) {
        let v = (self.persons[p].skill(g) as u32 + x).min(150);
        self.persons[p].set_skill(g, v as u8);
    }
    pub fn skill_down(&mut self, p: usize, g: usize, x: u32) {
        let v = (self.persons[p].skill(g) as i32 - x as i32).max(50);
        self.persons[p].set_skill(g, v as u8);
    }
    pub fn wealth_up(&mut self, c: usize, x: u32) {
        let v = (self.cities[c].wealth() as u32 + x).min(99);
        self.cities[c].set_wealth(v as u8);
    }
    pub fn wealth_down(&mut self, c: usize, x: u32) {
        let v = (self.cities[c].wealth() as i32 - x as i32).max(0);
        self.cities[c].set_wealth(v as u8);
    }
    /// `pick_property(kind, owner)` (0x42B90): one RNG call, none (and no call) without property.
    pub fn pick_property(&mut self, kind: u8, owner: usize) -> Option<usize> {
        let v: Vec<usize> = self.props_of(owner, kind).collect();
        if v.is_empty() {
            return None;
        }
        let p = v[self.r(v.len() as u32) as usize];
        self.last_pick = self.props[p].loc();
        Some(p)
    }
    fn year_gate(&mut self) -> bool {
        self.r(6) + 1605 <= self.year as u32
    }
    /// `Fy(k)` (§0).
    pub fn fy(&mut self, k: u32) -> i32 {
        (((70 + self.r(60)) * (self.year as u32 - 1570) * k) / 3000 + 1) as i32
    }

    /// Residence value V (0x44230).
    pub fn residence_value_of(&self, branch: usize) -> i64 {
        let p = &self.props[branch];
        self.residence_value_word(p.loc() as usize, p.residence())
    }

    /// V of residence word `r` in city `city` (0 for none).
    pub fn residence_value_word(&self, city: usize, r: u16) -> i64 {
        if r == 0xFFFF || city >= CITIES {
            return 0;
        }
        let c = &self.cities[city];
        // a level index >= 10 is outside RB (unreachable in play, rules-40 §13); clamped.
        let rb = RB[((r & 0x1F) as usize).min(9)];
        let v0 = (((r >> 8) as i64) + 150) * ((c.building_level() as i64 + 550) * rb / 600) / 250;
        let mut v = v0;
        if r & 0x40 != 0 {
            v += v0 / 10;
        }
        if r & 0x20 != 0 {
            v += v / 5;
        }
        if r & 0x80 != 0 {
            v += v / 3;
        }
        (c.wealth() as i64 + 150) * v / 200
    }

    // ---- §2 disasters ----

    pub fn disaster(&mut self, ev: &mut Vec<Event>) {
        let c = self.r(10) as usize;
        let mask = self.cities[c].fire();
        if self.cities[c].population() <= 1499 || mask & 0x80 != 0 || mask & 0x0F == 0 {
            return;
        }
        let d = loop {
            let d = self.r(4);
            if mask & (1 << d) != 0 {
                break d as usize;
            }
        };
        self.cities[c].set_fire(mask | 0x80);
        if let Some(i) = self.heap.alloc(2, NOBODY, c as u8, 0x83) {
            let y = self.year;
            let r = &mut self.heap.recs[i];
            r[4] = d as u8;
            r[5] = (d != 2) as u8;
            heap::w16(r, 6, y);
        }
        let variant = self.r(2) as u8;
        let (tp, tw, keep, wood, stone) = DISASTER[d];
        let pop = self.cities[c].population();
        let mut l = (self.r(tp) as u64 * pop as u64 / 100) as u32 + 10;
        if l + 1010 > pop {
            l = pop - 1000;
        }
        self.cities[c].set_population(pop - l);
        let adj = self.r(tw);
        let x = (adj * (3 + self.r(7))) as u8;
        self.wealth_down(c, x as u32);
        ev.push(Event::Disaster { city: c as u8, kind: d as u8, variant, lost: l, adj: adj as u8 });
        for p in 0..PERSONS {
            if !self.persons[p].used() {
                continue;
            }
            let mut loss = 0i64;
            let mut goods = false;
            let mut hit = false;
            for b in self.props_of(p, KIND_BRANCH).filter(|&b| self.props[b].loc() as usize == c).collect::<Vec<_>>() {
                hit = true;
                for g in 0..GOODS {
                    let old = self.props[b].stock(g) as u32;
                    goods |= old > 0;
                    let new = old * keep / 100;
                    self.props[b].set_stock(g, new as u16);
                    loss += (old - new) as i64 * self.cities[c].price(g) as i64;
                }
                let ow = self.props[b].wood() as u32;
                let nw = ow * (wood + self.r(10)) / 100;
                let os = self.props[b].stone() as u32;
                let ns = os * (stone + self.r(10)) / 100;
                self.props[b].set_wood(nw as u16);
                self.props[b].set_stone(ns as u16);
                loss += (ow - nw) as i64 * self.cities[c].level_a() as i64 + (os - ns) as i64 * self.cities[c].level_b() as i64;
                if self.persons[p].human() && (d == 1 || d == 2) {
                    self.history(p, if d == 2 { 6 } else { 7 }, c as u8, 0);
                }
            }
            if hit && self.persons[p].human() {
                ev.push(Event::DisasterBranches { person: p as u8, goods });
            }
            if loss > 0 && self.persons[p].insurance() & 1 != 0 {
                if self.persons[p].human() {
                    ev.push(Event::DisasterInsured { person: p as u8, loss: loss as i32 });
                }
                self.add_cash(p, loss as i32);
            }
        }
    }

    /// §2 aftermath, type-2 records at year end (world record loop).
    pub fn disaster_aftermath(&mut self) {
        for i in self.heap.all(2, NOBODY, ANY) {
            let c = self.heap.recs[i][2] as usize;
            let d = self.heap.recs[i][4];
            let r = &mut self.heap.recs[i];
            r[3] = r[3].wrapping_sub(1);
            if r[3] & 0x80 != 0 {
                r[3] &= 0x7F;
                continue;
            }
            if r[3] == 0 {
                let f = self.cities[c].fire() & 0x7F;
                self.cities[c].set_fire(f);
                self.heap.delete(i);
                continue;
            }
            if self.r(10) < 2 {
                if d == 2 {
                    // 64-bit product: since `city_growth` wraps at 32 bits, populations stay below about
                    // 4.3 million, where `pop * 95` fits 32 bits either way
                    let p = (self.cities[c].population() as u64 * 95 / 100) as u32;
                    self.cities[c].set_population(p);
                }
                if self.cities[c].population() < 1000 {
                    self.cities[c].set_population(1000);
                } else {
                    self.heap.recs[i][3] += 1;
                }
            }
        }
    }

    // ---- §3 ----

    /// §3.1 divine event (0x59940).
    pub fn divine(&mut self, ev: &mut Vec<Event>) {
        let f = self.globals.d(ga::TURN_FLAGS) | 0x08;
        self.globals.set_d(ga::TURN_FLAGS, f);
        let mut k = self.r(11);
        let mut mad = None;
        let (mut city, mut illness) = (0xFFu8, 0u8);
        if k == 3 {
            let c = self.r(10) as u8;
            city = c;
            mad = self.pick_computer(Some(c));
            if mad.is_none() {
                k = 2;
            }
        }
        let located = |st: &State, c: usize| (0..PERSONS).filter(|&p| st.persons[p].used() && st.persons[p].loc() as usize == c).collect::<Vec<_>>();
        match k {
            0 => {
                let c = self.r(10) as usize;
                city = c as u8;
                let x = 5 + self.r(10);
                self.wealth_up(c, x);
            }
            1 | 9 => {
                let c = self.r(10) as usize;
                city = c as u8;
                illness = self.r(12) as u8;
                for p in located(self, c) {
                    let l = self.persons[p].life() as i32;
                    let l = if k == 1 { if l + 5 >= 95 { 100 } else { l + 5 } } else if l - 5 < 5 { 0 } else { l - 5 };
                    self.persons[p].set_life(l as u8);
                }
            }
            2 => {
                let c = self.r(10) as usize;
                city = c as u8;
                let p = self.cities[c].population();
                self.cities[c].set_population(if p < 1100 { 1000 } else { (p as u64 * 10 / 11) as u32 });
            }
            3 => self.office_vacate(mad.unwrap()),
            4 | 5 => {
                let c = self.r(10) as u8;
                city = c;
                for i in self.heap.all(8, ANY, ANY) {
                    let r = &self.heap.recs[i];
                    if (r[4] == 0 || r[4] == 7) && self.props[r[2] as usize].loc() == c {
                        let w = if k == 4 { 0 } else { (r[5] as u32 * 2).min(99) as u8 };
                        self.heap.recs[i][5] = w;
                    }
                }
            }
            6 => {
                let c = self.r(10) as usize;
                city = c as u8;
                illness = self.r(12) as u8;
                for p in located(self, c) {
                    let g = self.r(15) as usize;
                    self.skill_up(p, g, 5);
                }
            }
            8 => {
                let a = self.r(10) as usize;
                let b = ((a as u32 + 1 + self.r(9)) % 10) as usize;
                let n = 50 + self.r(50);
                // six cosmetic draws in the original's order (rules-15 §4.1): hospitals, museums, churches of B, then of A
                let (h2, m2, c2) = (2 + self.r(3), 2 + self.r(4), 2 + self.r(5));
                let (h1, m1, c1) = (4 + self.r(3), 5 + self.r(3), 6 + self.r(3));
                ev.push(Event::Tornado { a: a as u8, b: b as u8, n, churches: [c1, c2], museums: [m1, m2], hospitals: [h1, h2] });
                let x = 5 + self.r(10);
                self.wealth_down(a, x);
                let y = 5 + self.r(10);
                self.wealth_up(b, y);
                let pa = self.cities[a].population();
                self.cities[a].set_population(pa.saturating_sub(n).max(1000));
                let pb = self.cities[b].population();
                self.cities[b].set_population(pb + n / 2 + 1);
            }
            10 => {
                for c in 0..CITIES {
                    let p = self.cities[c].price(12) / 2 + 1;
                    self.cities[c].set_price(12, p);
                }
            }
            _ => {}
        }
        ev.push(Event::Divine { kind: k as u8, city, with: mad.map_or(0xFF, |m| m as u8), illness });
    }

    /// §3.2 world fate (0x26640).
    pub fn world_fate(&mut self, ev: &mut Vec<Event>) {
        let k = self.r(5);
        let mut aux = [0u8; 2];
        match k {
            0 | 1 => {
                let g = self.r(15) as usize;
                aux[0] = g as u8;
                for c in 0..CITIES {
                    let p = self.cities[c].price(g) as u32;
                    let p = if k == 0 { (p * 3 / 2).min(99) } else { (p / 2).max(1) };
                    self.cities[c].set_price(g, p as u8);
                }
            }
            2 | 3 => {
                let c = self.r(10) as usize;
                aux[0] = c as u8;
                let x = 5 + self.r(5);
                if k == 2 { self.wealth_up(c, x) } else { self.wealth_down(c, x) }
            }
            _ => {
                aux = [self.r(10) as u8, self.r(12) as u8];
                if self.r(2) != 0 {
                    // shipments to or from c turn back: destination := byte +2, sells become deliveries.
                    // +2 is a property index (quirk, rules-40 §14: outcome-equivalent); used literally when it is a valid
                    // city, else the source branch's city (the original then reads past the city table).
                    let c = aux[0] as usize;
                    for i in self.heap.all(9, crate::heap::ANY, crate::heap::ANY) {
                        let (src, dest) = (self.heap.recs[i][2] as usize, self.heap.recs[i][10] as usize);
                        let from = (src < PROPS).then(|| self.props[src].loc() as usize);
                        if dest == c || from == Some(c) {
                            let r = &mut self.heap.recs[i];
                            r[10] = if src < CITIES { src as u8 } else { from.unwrap_or(c) as u8 };
                            if r[4] == 2 || r[4] == 3 {
                                r[4] += 2;
                            }
                        }
                    }
                    aux[1] |= 0x80;
                }
            }
        }
        ev.push(Event::WorldFate { kind: k as u8, aux });
    }

    // ---- §4 fate events ----

    fn director_branch(&mut self, h: usize) -> Option<(usize, usize)> {
        // the drawn branch, if it has a director; no re-draw (rules-40 §15)
        let b = self.pick_property(KIND_BRANCH, h)?;
        let d = self.props[b].director();
        (d != NONE).then_some((b, d as usize))
    }

    fn first_branch(&self, h: usize, f: impl Fn(&Property) -> bool) -> Option<usize> {
        self.props_of(h, KIND_BRANCH).find(|&b| f(&self.props[b]))
    }

    fn cut_goods(&mut self, b: usize, lo: u32, span: u32) {
        for g in 0..GOODS {
            let x = lo + self.r(span);
            let s = self.props[b].stock(g) as u32;
            self.props[b].set_stock(g, (s - s * x / 100) as u16);
        }
    }

    fn located_computers(&self, loc: usize, clergy: Option<bool>) -> Vec<usize> {
        self.computers().filter(|&p| self.persons[p].loc() as usize == loc && clergy.is_none_or(|c| self.persons[p].church() == c)).collect()
    }

    fn first_shipment(&self, h: usize, f: impl Fn(&[u8]) -> bool) -> Option<usize> {
        self.heap.all(9, h as u8, ANY).into_iter().find(|&i| f(&self.heap.recs[i]))
    }

    fn total(p: &Property) -> u32 {
        (0..4).map(|k| p.troops(k) as u32).sum()
    }

    fn recruit(&mut self, p: usize) {
        let mut z = 1 + self.r(6);
        let t = Self::total(&self.props[p]);
        let cap = self.props[p].b4() as u32;
        if t + z > cap {
            z = cap.saturating_sub(t);
        }
        if z > 0 {
            let mut y = self.r(4) as usize;
            for _ in 0..z {
                let pr = &mut self.props[p];
                let c = pr.troops(y) as u32;
                pr.set_training(y, ((c * pr.training(y) as u32 + 30) / (c + 1)) as u8);
                pr.set_troops(y, (c + 1) as u8);
                y = (y + 1) % 4;
            }
        }
    }

    fn transfer(&mut self, from: usize, to: usize) {
        let mut z = (1 + self.r(6)).min(Self::total(&self.props[from]));
        let (tq, cap) = (Self::total(&self.props[to]), self.props[to].b4() as u32);
        if tq + z > cap {
            z = cap.saturating_sub(tq);
        }
        if z > 0 {
            let mut y = self.r(4) as usize;
            for _ in 0..z {
                while self.props[from].troops(y) == 0 {
                    y = (y + 1) % 4;
                }
                let a = self.props[from].troops(y) - 1;
                self.props[from].set_troops(y, a);
                let b = self.props[to].troops(y) + 1;
                self.props[to].set_troops(y, b);
            }
        }
    }

    fn defect(&mut self, h: usize, kind: u8, out: bool) -> bool {
        let Some(p) = self.pick_property(kind, h) else { return false };
        let humans: Vec<usize> = self.humans().collect();
        for _ in 0..20 {
            let x = humans[self.r(humans.len() as u32) as usize];
            if x == h {
                continue;
            }
            if let Some(q) = self.pick_property(kind, x) {
                self.last_person = x as u8;
                self.last_pick = self.props[if out { p } else { q }].loc();
                if out { self.transfer(p, q) } else { self.transfer(q, p) }
                return true;
            }
        }
        false
    }

    fn troop_event(&mut self, h: usize, kind: u8, e: u32) -> bool {
        let Some(p) = self.pick_property(kind, h) else { return false };
        match e {
            0 => {
                for k in 0..4 {
                    let v = (self.props[p].troops(k) as i32 - self.ri(3)).max(0);
                    self.props[p].set_troops(k, v as u8);
                }
            }
            1 => {
                for k in 0..4 {
                    let v = (self.props[p].training(k) as i32 - self.ri(5)).max(0);
                    self.props[p].set_training(k, v as u8);
                }
            }
            2 => {
                self.last_aux = [self.r(18) as u8, self.r(10) as u8];
                for k in 0..4 {
                    let v = (self.props[p].training(k) as u32 + self.r(5)).min(100);
                    self.props[p].set_training(k, v as u8);
                }
            }
            3 => {
                let x = Self::total(&self.props[p]) as i32 * (10 + self.ri(10));
                self.pay(h, x);
            }
            _ => self.cut_goods(p, 10, 30),
        }
        true
    }

    /// One fate event for human h (0x226F0). Returns false when nothing happened.
    pub fn fate(&mut self, h: usize, ev: &mut Vec<Event>) {
        let mut k = self.r(100) as u8;
        if k == self.last_fate {
            k = 100;
        }
        self.last_fate = k;
        let hb = h as u8;
        let loc = self.persons[h].loc() as usize;
        let cash0 = self.persons[h].cash();
        self.last_pick = 0xFF;
        self.last_aux = [0xFF; 2];
        self.last_person = 0xFF;
        self.last_amount = -1;
        // residence word of the branch the text names, before the effect changes it
        let res0 = match k {
            29 | 53 => self.first_branch(h, |p| p.has_residence()),
            78 => self.first_branch(h, |p| p.has_residence() && p.b3() & 0x60 != 0),
            _ => None,
        }
        .map(|b| (self.props[b].residence(), self.props[b].loc(), self.props[b].b3() & 0x40 == 0));
        // the shipment a transport kind names (same selection as its arm), before the effect
        let ship0 = match k {
            56 | 61 | 67 | 68 => self.first_shipment(h, |_| true),
            62 => self.first_shipment(h, |r| r[9] as usize == BALLON),
            63 | 64 => self.first_shipment(h, |r| r[9] % 3 != 1 && r[8] >= 2),
            66 => self.first_shipment(h, |r| r[4] == 3 || r[4] == 5),
            _ => None,
        }
        .map(|i| (i, self.heap.recs[i][7], self.heap.recs[i][8], self.heap.recs[i][9], self.heap.recs[i][10]));
        let happened = match k {
            0 | 2 | 7 | 10 | 22 | 26 => {
                let gate = self.year_gate();
                let ok = gate
                    && match k {
                        0 => self.pick_property(KIND_BRANCH, h).is_some(),
                        7 => self.persons[h].religion() != 2,
                        _ => true,
                    };
                if ok {
                    let kk = [(0, 25), (2, 30), (7, 30), (10, 20), (22, 15), (26, 15)].iter().find(|x| x.0 == k).unwrap().1;
                    let s = self.standing(h, kk);
                    self.pay(h, s);
                }
                ok
            }
            1 | 3 | 8 => {
                let ok = match k {
                    1 => self.pick_property(KIND_BRANCH, h).is_some(),
                    8 => self.persons[h].religion() != 2,
                    _ => true,
                };
                if ok {
                    let s = self.standing(h, if k == 1 { 25 } else { 30 });
                    self.add_cash(h, s);
                }
                ok
            }
            4 | 24 | 51 => match self.pick_computer(Some(loc as u8)) {
                Some(x) => {
                    let v = match k {
                        4 => 2 + self.ri(5),
                        24 => {
                            self.r(10);
                            1 + self.ri(10)
                        }
                        _ => 3 + self.ri(10),
                    };
                    self.favour_increase(h, x, v);
                    true
                }
                None => false,
            },
            5 => match self.pick_property(KIND_BRANCH, h) {
                Some(b) => {
                    let c = self.props[b].loc() as usize;
                    let v = self.r(2);
                    self.last_aux[0] = v as u8;
                    for p in self.located_computers(c, Some(false)) {
                        let x = 1 + self.ri(6);
                        if v == 0 { self.favour_increase(h, p, x) } else { self.favour_decrease(h, p, x) }
                    }
                    true
                }
                None => false,
            },
            6 | 39 => match self.pick_property(KIND_BRANCH, h) {
                Some(b) => {
                    if k == 6 { self.cut_goods(b, 5, 35) } else { self.cut_goods(b, 10, 35) }
                    true
                }
                None => false,
            },
            9 | 38 => {
                let x = self.fy(200);
                self.add_cash(h, x);
                true
            }
            11 | 12 => {
                let ok = self.year_gate();
                if ok {
                    let s = self.standing(h, if k == 11 { 25 } else { 30 });
                    self.last_aux[0] = self.r(30) as u8;
                    self.pay(h, s);
                }
                ok
            }
            13 => {
                let s = self.standing(h, 20);
                self.last_aux[0] = self.r(30) as u8;
                self.add_cash(h, s);
                true
            }
            15..=18 => match self.director_branch(h) {
                Some((_, d)) => {
                    if k >= 17 {
                        self.last_aux[0] = self.r(49) as u8;
                    }
                    for g in 0..GOODS {
                        if k % 2 == 1 {
                            let x = 1 + self.r(5);
                            self.skill_down(d, g, x);
                        } else {
                            let x = 3 + self.r(6);
                            self.skill_up(d, g, x);
                        }
                    }
                    true
                }
                None => false,
            },
            19 => {
                let ok = self.year > 1620;
                if ok {
                    self.last_aux = [self.r(12) as u8, 0xFF];
                    let l = (self.persons[h].life() as i32 - 1 - self.ri(4)).max(0);
                    self.persons[h].set_life(l as u8);
                }
                ok
            }
            20 => {
                let ok = self.year_gate();
                if ok {
                    let s = self.standing(h, 35);
                    self.pay(h, s);
                    let l = (self.persons[h].life() as u32 + 1 + self.r(3)).min(100);
                    self.persons[h].set_life(l as u8);
                }
                ok
            }
            21 => {
                let ok = self.year_gate();
                if ok {
                    let s = self.standing(h, 30);
                    let city = 1 + self.r(9) as usize;
                    let county = (city - 1) / 3 + 10;
                    self.last_pick = city as u8;
                    self.pay(h, s);
                    for p in self.located_computers(county, Some(true)) {
                        let x = 1 + self.ri(8);
                        self.favour_increase(h, p, x);
                    }
                }
                ok
            }
            25 => {
                let ok = self.r(100) < 33;
                if ok {
                    self.last_aux = [self.r(16) as u8 + 2, self.r(2) as u8];
                }
                ok
            }
            27 => {
                let ok = self.year_gate();
                if ok {
                    let x = self.fy(150);
                    self.pay(h, x);
                }
                ok
            }
            28 => {
                let x = self.fy(150);
                self.add_cash(h, x);
                true
            }
            29 => match self.pick_property(KIND_BRANCH, h) {
                Some(b) if self.props[b].has_residence() => {
                    let l = (self.persons[h].life() as u32 + 1 + self.r(4)).min(100);
                    self.persons[h].set_life(l as u8);
                    true
                }
                _ => false,
            },
            30 => {
                let ok = self.year > 1610;
                match self.pick_property(KIND_BRANCH, h).filter(|_| ok) {
                    Some(b) => {
                        let x = self.r(40);
                        let p = &mut self.props[b];
                        p.set_wood((p.wood() as u32 * (100 - x) / 100) as u16);
                        p.set_stone((p.stone() as u32 * (100 - x / 2) / 100) as u16);
                        true
                    }
                    None => false,
                }
            }
            31 => match self.pick_property(KIND_BRANCH, h) {
                Some(b) => {
                    for g in PERISHABLE {
                        let x = self.r(30);
                        let s = self.props[b].stock(g) as u32;
                        self.props[b].set_stock(g, (s - s * x / 100) as u16);
                    }
                    true
                }
                None => false,
            },
            32 => {
                let county = 10 + self.r(3) as usize;
                let ok = (0..PERSONS).any(|p| p != h && self.persons[p].used() && self.persons[p].loc() as usize == county);
                if ok {
                    self.last_pick = county as u8;
                    self.last_person = (0..PERSONS).find(|&p| p != h && self.persons[p].used() && self.persons[p].loc() as usize == county).map_or(0xFF, |p| p as u8);
                    for p in self.located_computers(county, None) {
                        let x = self.ri(10);
                        self.favour_increase(h, p, x);
                    }
                }
                ok
            }
            33 => {
                let ok = self.year_gate() && self.pick_property(KIND_BRANCH, h).is_some();
                if ok {
                    let s = self.standing(h, 20);
                    self.pay(h, s);
                }
                ok
            }
            34 => match self.pick_property(KIND_BRANCH, h) {
                Some(b) => {
                    self.add_transporters(b, KARREN, 1, 100);
                    true
                }
                None => false,
            },
            35 => match self.first_branch(h, |p| p.director() != NONE) {
                Some(_) => {
                    let s = self.standing(h, 20);
                    self.pay(h, s);
                    let v = self.r(2);
                    self.last_aux = [self.r(30) as u8, v as u8];
                    if v == 0 {
                        for b in self.props_of(h, KIND_BRANCH).collect::<Vec<_>>() {
                            let d = self.props[b].director();
                            if d != NONE {
                                for g in 0..GOODS {
                                    let x = 5 + self.r(5);
                                    self.skill_up(d as usize, g, x);
                                }
                            }
                        }
                    }
                    true
                }
                None => false,
            },
            36 | 77 => {
                let x = self.fy(250);
                if k == 77 {
                    self.last_aux = [self.r(10) as u8 + 2, 0];
                }
                self.add_cash(h, x);
                true
            }
            37 => {
                let ok = self.year_gate();
                if ok {
                    let x = self.fy(350);
                    self.last_pick = self.r(10) as u8;
                    self.pay(h, x);
                    for p in self.computers().filter(|&p| self.persons[p].has_office() && self.persons[p].loc() < 10 && self.persons[p].idx() == 6).collect::<Vec<_>>() {
                        let v = 1 + self.ri(10);
                        self.favour_increase(h, p, v);
                    }
                }
                ok
            }
            40 => match self.director_branch(h) {
                Some((b, _)) => {
                    self.props[b].set_director(NONE);
                    true
                }
                None => false,
            },
            42 => match self.pick_property(KIND_BRANCH, h) {
                Some(_) => {
                    let x = self.fy(200);
                    self.add_cash(h, x);
                    true
                }
                None => false,
            },
            43 => {
                let ok = self.year_gate();
                match self.pick_property(KIND_BRANCH, h).filter(|_| ok) {
                    Some(b) => {
                        let x = self.fy(200);
                        self.pay(h, x);
                        let c = self.props[b].loc() as usize;
                        let (a, bb) = (self.cities[c].level_a().max(1) as i32, self.cities[c].level_b().max(1) as i32);
                        let p = &mut self.props[b];
                        p.set_wood((p.wood() as i32 + x * 5 / (a * 4)).min(4999) as u16);
                        p.set_stone((p.stone() as i32 + x * 5 / (bb * 4)).min(4999) as u16);
                        true
                    }
                    None => false,
                }
            }
            44 => match self.pick_property(KIND_BRANCH, h) {
                Some(b) if self.props[b].trans(KARREN) >= 1 => {
                    let n = self.props[b].trans(KARREN) - 1;
                    self.props[b].set_trans(KARREN, n);
                    true
                }
                _ => false,
            },
            45 | 46 => {
                let g = self.r(15) as usize;
                self.last_aux = [self.r(49) as u8, g as u8];
                let x = 1 + self.r(10);
                if k == 45 { self.skill_up(h, g, x) } else { self.skill_down(h, g, x) }
                true
            }
            47 => {
                let ok = self.year_gate();
                let others: Vec<usize> = (0..PERSONS).filter(|&p| p != h && self.persons[p].used()).collect();
                if ok && !others.is_empty() {
                    let x = others[self.r(others.len() as u32) as usize];
                    self.last_person = x as u8;
                    let m = self.fy(200);
                    self.pay(h, m);
                    self.add_cash(x, m);
                }
                ok
            }
            48 => {
                let c = self.r(10) as usize;
                self.last_pick = c as u8;
                let x = 3 + self.r(5);
                self.wealth_up(c, x);
                for p in self.located_computers(c, Some(false)) {
                    let v = 1 + self.ri(8);
                    self.favour_increase(h, p, v);
                }
                true
            }
            49 | 50 => match self.first_branch(h, |p| p.has_residence()) {
                Some(b) => {
                    if k == 50 {
                        // helpers' payment (shown, never paid) and the colour of the salon
                        self.last_amount = 1 + self.r(1000) as i32;
                        self.last_aux = [self.r(3) as u8, 0];
                        self.last_pick = self.props[b].loc();
                    }
                    let v = (self.props[b].b4() as i32 - 1 - self.ri(30)).max(0);
                    self.props[b].set_b4(v as u8);
                    true
                }
                None => false,
            },
            52 => {
                let runs = self.heap.all(8, hb, ANY);
                let ok = self.year >= 1608 && self.persons[h].pay_workers() <= 99 && !runs.is_empty();
                if ok {
                    let j = self.r(runs.len() as u32) as usize;
                    let b = self.heap.recs[runs[j]][2];
                    for i in runs {
                        if self.heap.recs[i][2] == b {
                            let w = self.heap.recs[i][5] as u32 * (70 + self.r(20)) / 100;
                            self.heap.recs[i][5] = w as u8;
                        }
                    }
                }
                ok
            }
            53 => {
                let ok = self.year_gate();
                match self.first_branch(h, |p| p.has_residence()).filter(|_| ok) {
                    Some(b) => {
                        let v = self.residence_value_of(b);
                        let x = v * (3 + self.r(6)) as i64 / 100;
                        self.pay(h, x as i32);
                        true
                    }
                    None => false,
                }
            }
            54 => {
                let ok = self.heap.has(13, hb);
                if ok {
                    let i = self.heap.find(13, hb, ANY).unwrap();
                    let (b, word) = (self.heap.recs[i][2] as usize, heap::r16(&self.heap.recs[i], 4));
                    let (city, v) = (self.props[b].loc(), self.residence_value_word(self.props[b].loc() as usize, word));
                    self.last_pick = city;
                    self.last_aux = word.to_le_bytes();
                    self.last_amount = (v * (3 + self.r(6)) as i64 / 100) as i32; // bug [C]: never paid
                }
                ok
            }
            55 => match self.heap.all(8, hb, ANY).into_iter().find(|&i| self.heap.recs[i][4] == 0) {
                Some(i) => {
                    self.heap.recs[i][5] /= 2;
                    true
                }
                None => false,
            },
            56 | 61 | 67 | 68 => match self.first_shipment(h, |_| true) {
                Some(i) => {
                    match k {
                        56 => {
                            let q = heap::r16(&self.heap.recs[i], 5) / 2;
                            heap::w16(&mut self.heap.recs[i], 5, q);
                        }
                        61 => {
                            let c = self.heap.recs[i][11] as u32 * (30 + self.r(40)) / 100;
                            self.heap.recs[i][11] = c as u8;
                        }
                        67 => {
                            let q = heap::r16(&self.heap.recs[i], 5) as u32 * (70 + self.r(20)) / 100;
                            heap::w16(&mut self.heap.recs[i], 5, q as u16);
                        }
                        _ => self.blow_off_course(i),
                    }
                    true
                }
                None => false,
            },
            57 => match self.pick_computer(None) {
                Some(x) => {
                    let v = 1 + self.ri(10);
                    self.favour_increase(h, x, v);
                    true
                }
                None => false,
            },
            59 | 60 => {
                let p = self.props_of(h, if k == 59 { KIND_CASTLE } else { KIND_CAMP }).next();
                if let Some(p) = p {
                    self.last_pick = self.props[p].loc();
                }
                p.is_some()
            }
            62 => match self.first_shipment(h, |r| r[9] as usize == BALLON) {
                Some(i) => {
                    self.blow_off_course(i);
                    let r = &mut self.heap.recs[i];
                    if r[4] == 4 || r[4] == 5 {
                        r[4] -= 2;
                    }
                    true
                }
                None => false,
            },
            63 | 64 => match self.first_shipment(h, |r| r[9] % 3 != 1 && r[8] >= 2) {
                Some(i) => {
                    self.heap.recs[i][8] -= 1;
                    if k == 64 {
                        let others: Vec<usize> = (0..PROPS).filter(|&b| self.props[b].kind() == KIND_BRANCH && self.props[b].owner() != hb).collect();
                        if !others.is_empty() {
                            let b = others[self.r(others.len() as u32) as usize];
                            self.last_person = self.props[b].owner();
                            let (t, c) = (self.heap.recs[i][9] as usize, self.heap.recs[i][11]);
                            self.add_transporters(b, t, 1, c);
                        }
                    }
                    true
                }
                None => false,
            },
            65 => match self.first_branch(h, |_| true) {
                Some(b) => {
                    let n = 1 + self.r(2) as u8;
                    let t = 2 * self.r(2) as usize;
                    self.last_aux = [n, t as u8];
                    self.last_pick = self.props[b].loc();
                    self.add_transporters(b, t, n, 100);
                    true
                }
                None => false,
            },
            66 => match self.first_shipment(h, |r| r[4] == 3 || r[4] == 5) {
                Some(i) => {
                    self.heap.recs[i][4] -= 1;
                    true
                }
                None => false,
            },
            69 => {
                let runs = self.heap.all(8, hb, ANY);
                // year >= 1610 (rules-40 §16, correcting rules-06 §4); the branch of the first run in heap order
                let ok = self.year >= 1610 && !runs.is_empty();
                if ok {
                    let b = self.heap.recs[runs[0]][2] as usize;
                    self.last_pick = self.props[b].loc();
                    for s in 0..4 {
                        self.props[b].set_order(s, [0; 8]);
                    }
                    let n = 5 * self.persons[h].certificate() as u32 + 5;
                    for i in runs {
                        if self.heap.recs[i][2] as usize != b {
                            continue;
                        }
                        let (g, w, m) = (self.heap.recs[i][4] as usize, self.heap.recs[i][5] as i32, self.heap.recs[i][6] as i32);
                        let z = ((g as u32 + 1 + self.r(n - 1)) % n) as usize;
                        let r = &mut self.heap.recs[i];
                        r[4] = z as u8;
                        r[5] = (w * wage_base(g) / wage_base(z)).clamp(1, 60) as u8;
                        r[6] = (m * MEANS_COST[g] / MEANS_COST[z]).clamp(1, 60) as u8;
                    }
                }
                ok
            }
            70 => match self.pick_property(KIND_BRANCH, h) {
                Some(b) => {
                    let p = &mut self.props[b];
                    p.set_wood((p.wood() as u32 * 106 / 100).min(4999) as u16);
                    p.set_stone((p.stone() as u32 * 106 / 100).min(4999) as u16);
                    true
                }
                None => false,
            },
            71 => match self.pick_property(KIND_BRANCH, h) {
                Some(b) if self.props[b].wood() > 0 => {
                    let p = &mut self.props[b];
                    let s = p.stone() as u32 + p.wood() as u32;
                    if s < 5000 {
                        p.set_stone(s as u16);
                        p.set_wood(0);
                    } else {
                        p.set_wood((s - 4999) as u16);
                        p.set_stone(4999);
                    }
                    true
                }
                _ => false,
            },
            72 | 73 => {
                let ok = k == 73 || self.year_gate();
                let w: i32 = self.heap.all(8, hb, ANY).into_iter().map(|i| self.heap.recs[i][5] as i32).sum();
                let ok = ok && w > 0;
                if ok {
                    let per = 2 + self.ri(5);
                    let x = per * w;
                    self.last_aux = [per as u8, 0];
                    if k == 72 { self.pay(h, x) } else { self.add_cash(h, x) }
                }
                ok
            }
            74 => match self.director_branch(h) {
                Some(_) => {
                    let x = self.fy(1000);
                    self.pay(h, x);
                    true
                }
                None => false,
            },
            75 => {
                let ok = self.year_gate();
                let others: Vec<usize> = self.humans().filter(|&x| x != h).collect();
                if ok && !others.is_empty() {
                    let x = others[self.r(others.len() as u32) as usize];
                    self.last_person = x as u8;
                    let s = self.standing(h, 40);
                    self.pay(h, s);
                    self.add_cash(x, s);
                }
                ok && !others.is_empty()
            }
            76 => {
                let ok = self.year > 1604;
                match self.pick_property(KIND_BRANCH, h).filter(|_| ok) {
                    Some(b) => {
                        let c = 10 + self.r(80) as u8;
                        self.add_transporters(b, DAMPFKARREN, 1, c);
                        true
                    }
                    None => false,
                }
            }
            78 => match self.first_branch(h, |p| p.has_residence() && p.b3() & 0x60 != 0) {
                Some(b) => {
                    let r = self.props[b].b3();
                    self.props[b].set_b3(if r & 0x40 == 0 { r & !0x20 } else { r & !0x40 });
                    true
                }
                None => false,
            },
            79 | 88 => match self.pick_property(if k == 79 { KIND_CAMP } else { KIND_CASTLE }, h) {
                Some(p) => {
                    self.recruit(p);
                    true
                }
                None => false,
            },
            80..=84 => self.troop_event(h, KIND_CAMP, (k - 80) as u32),
            89..=93 => self.troop_event(h, KIND_CASTLE, (k - 89) as u32),
            85 | 86 => self.defect(h, KIND_CAMP, k == 85),
            94 | 95 => self.defect(h, KIND_CASTLE, k == 94),
            87 => match self.pick_property(KIND_CAMP, h) {
                Some(_) => {
                    self.last_aux = [self.r(18) as u8, self.r(10) as u8];
                    let x = self.fy(1500) / 2;
                    self.add_cash(h, x);
                    true
                }
                None => false,
            },
            96..=98 => match self.pick_property(KIND_CASTLE, h) {
                Some(p) => {
                    match k {
                        96 | 97 => {
                            self.r(18);
                            self.r(10);
                            let x = self.r(10);
                            if k == 96 {
                                let v = (self.props[p].b3() as u32 + x).min(100);
                                self.props[p].set_b3(v as u8);
                            } else {
                                let v = (self.props[p].b4() as u32 + x).min(150);
                                self.props[p].set_b4(v as u8);
                            }
                        }
                        _ => {
                            self.last_aux = [self.r(9) as u8, 0];
                            let l = (self.persons[h].life() as u32 + 1 + self.r(5)).min(100);
                            self.persons[h].set_life(l as u8);
                        }
                    }
                    true
                }
                None => false,
            },
            99 => {
                let p = self.pick_property(KIND_CASTLE, h);
                let p = match p {
                    Some(p) => Some(p),
                    None => self.pick_property(KIND_CAMP, h),
                };
                match p {
                    Some(p) => {
                        // bug [C]: unsigned floor test, wraps
                        let v = self.props[p].b3().wrapping_sub(self.r(10) as u8);
                        self.props[p].set_b3(v);
                        true
                    }
                    None => false,
                }
            }
            23 | 58 => true,
            _ => false,
        };
        if happened {
            let amount = (self.persons[h].cash() - cash0).abs();
            let (mut aux, mut city, mut with) = (self.last_aux, self.last_pick, self.last_person);
            if let Some((word, loc, fire)) = res0 {
                // residence kinds: aux = the residence word (little endian), with = 1 for the fire variant of kind 78
                aux = word.to_le_bytes();
                city = loc;
                with = fire as u8;
            }
            let mut amount = if self.last_amount >= 0 { self.last_amount } else { amount };
            if let Some((i, good, count, typ, dest)) = ship0 {
                // transport kinds: with = good, aux = (count, type), city = old destination, amount = new destination
                (with, aux, city) = (good, [count, typ], dest);
                amount = if k == 64 { self.last_person as i32 } else { self.heap.recs[i][10] as i32 };
            }
            ev.push(Event::Fate { person: hb, kind: k, amount, city, aux, with });
        }
    }

    fn blow_off_course(&mut self, i: usize) {
        let d = ((self.heap.recs[i][10] as u32 + 1 + self.r(9)) % 10) as u8;
        self.heap.recs[i][10] = d;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Soak regression: the plague aftermath's `pop * 95 / 100` on a late-game population (over 45 million)
    /// must not overflow (found in year 1993, seed 35 of `soak`). Counter 100: the record outlives the loop.
    #[test]
    fn rules06_s2_aftermath_large_population() {
        let mut st = State::new_world(1);
        st.cities[3].set_population(160_000_000);
        let i = st.heap.alloc(2, NOBODY, 3, 100).unwrap();
        st.heap.recs[i][4] = 2;
        for _ in 0..40 {
            st.disaster_aftermath();
        }
        assert!(st.cities[3].population() < 160_000_000);
    }

    /// rules-06 §2: a disaster removes population (floor 1000), sets bit 7 and a type-2 record.
    #[test]
    fn rules06_s2_disaster() {
        let mut st = State::new_world(1);
        let mut ev = vec![];
        for _ in 0..50 {
            st.disaster(&mut ev);
        }
        let hit: Vec<usize> = (0..CITIES).filter(|&c| st.cities[c].fire() & 0x80 != 0).collect();
        assert!(!hit.is_empty());
        for c in hit {
            assert!(st.cities[c].population() >= 1000);
            assert!(st.heap.find(2, NOBODY, c as u8).is_some());
        }
        // aftermath: at least two year ends, then bit 7 clears
        for _ in 0..30 {
            st.disaster_aftermath();
        }
        assert!((0..CITIES).all(|c| st.cities[c].fire() & 0x80 == 0));
    }

    /// rules-06 §4 residence value formula (0x44230): Kate, condition 0, levels 50, wealth 50.
    #[test]
    fn rules06_s4_residence_value() {
        let mut st = State::new_world(1);
        let b = st.property_create(KIND_BRANCH, 0, 0).unwrap();
        st.props[b].set_residence(0x0000);
        st.cities[0].set_building_level(50);
        st.cities[0].set_wealth(50);
        // V0 = 150 * (600*500/600)/250 = 300; V = 200*300/200
        assert_eq!(st.residence_value_of(b), 300);
        st.props[b].set_residence(0x0020);
        assert_eq!(st.residence_value_of(b), 360);
    }

    /// The fate table runs for all 101 k without panicking.
    #[test]
    fn rules06_s4_fate_smoke() {
        let mut st = State::new_game(3, 3, &[crate::world::HumanSetup::test(1), crate::world::HumanSetup::test(2)]).unwrap();
        st.year = 1650;
        let h = st.current as usize;
        let mut ev = vec![];
        for _ in 0..2000 {
            st.fate(h, &mut ev);
        }
        assert!(ev.len() > 100);
    }
}
