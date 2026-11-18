//! Year-end castle/camp management (rules-07 §3) and sabotage strikes (rules-04b §4b).

use crate::heap::{self, ANY};
use crate::state::*;
use crate::{Ask, Choices, Event, WArg};

/// Wages per troop slot (rules-07 §3a): castle mercenaries, camp robbers.
const WAGE: [[i64; 4]; 2] = [[100, 125, 125, 150], [75, 100, 100, 125]];

impl State {
    /// rules-08 §1.5: nobody interested, then half the time a computer person becomes so.
    pub fn castle_offers(&mut self, pi: usize) {
        if self.heap.has_key(17, pi as u8) || self.r(100) >= 50 {
            return;
        }
        // rules-24 §3: used computer persons whose place byte (+0x0F) is 13, slot order
        let cs: Vec<usize> = self.computers().filter(|&x| self.persons[x].loc() == 13).collect();
        if !cs.is_empty() {
            let x = cs[self.r(cs.len() as u32) as usize];
            self.heap.alloc(17, x as u8, pi as u8, 4);
        }
    }

    /// rules-23 §4.2: private sale to an interested person at the property's value; a human buyer must agree.
    pub fn sell_castle(&mut self, o: usize, pi: usize, x: usize, ch: &mut dyn Choices) {
        if x >= PERSONS || self.heap.find(17, x as u8, pi as u8).is_none() {
            return;
        }
        if self.persons[x].human() && ch.ask(&Ask::ConfirmCastleSale { buyer: x as u8 }) != 0 {
            return;
        }
        let v = self.property_value(pi) as i32;
        self.pay(x, v);
        self.add_cash(o, v);
        self.props[pi].set_owner(x as u8);
        if self.persons[x].human() {
            self.history(x, 9, self.props[pi].loc(), 0);
        }
        for i in self.heap.all(17, ANY, pi as u8) {
            self.heap.delete(i);
        }
    }

    /// Order type base of a castle (0x10) or camp (0x20); `None` for other properties.
    fn troop_base_type(&self, p: usize) -> Option<u8> {
        match self.props[p].kind() {
            KIND_CASTLE => Some(0x10),
            KIND_CAMP => Some(0x20),
            _ => None,
        }
    }

    /// Slot `k` of castle/camp `p` is a troop order (types base+1..base+3).
    pub(crate) fn is_troop_order(&self, p: usize, k: usize) -> bool {
        let o = self.props[p].order(k)[0];
        self.troop_base_type(p).is_some_and(|b| (b + 1..=b + 3).contains(&o))
    }

    /// `order_feasible` of the castle/camp order book (rules-07 §3a).
    pub fn troop_feasible(&self, p: usize, k: usize) -> bool {
        if !self.is_troop_order(p, k) {
            return true;
        }
        let o = self.props[p].order(k);
        let base = self.troop_base_type(p).unwrap_or(0x10);
        let owner = self.props[p].owner();
        if o[0] == base + 1 {
            let loc = o[2].saturating_add(14);
            if (0..PROPS).any(|q| self.props[q].loc() == loc && matches!(self.props[q].kind(), KIND_CASTLE | KIND_CAMP) && self.props[q].owner() == owner) {
                return false;
            }
        }
        (0..4).all(|j| o[3 + j] <= self.props[p].troops(j)) && o[3..7].iter().any(|&n| n > 0)
    }

    /// Confirming a troop order takes its troops out of the garrison; unconfirming returns them.
    pub fn troop_reserve(&mut self, p: usize, k: usize, on: bool) {
        if !self.is_troop_order(p, k) {
            return;
        }
        let o = self.props[p].order(k);
        for j in 0..4 {
            let t = self.props[p].troops(j);
            self.props[p].set_troops(j, if on { t.saturating_sub(o[3 + j]) } else { t.saturating_add(o[3 + j]) });
        }
    }

    /// Owner's turn end, first step: yearly pay and training of the troops of every castle and camp
    /// (confirmed orders' troops counted in). Returns the cost (rules-07 §3a).
    /// Pay of `h`'s castle troops (Söldner) and camp troops (Räuber), the two bill rows.
    pub fn troop_pay(&mut self, h: usize) -> [i32; 2] {
        let mut cost = [0i64; 2];
        for p in 0..PROPS {
            let Some(base) = self.troop_base_type(p) else { continue };
            if self.props[p].owner() as usize != h {
                continue;
            }
            let camp = base == 0x20;
            let pay = if camp { self.persons[h].pay_robbers() } else { self.persons[h].pay_mercs() } as i64;
            let mut n = [0i64; 4];
            for (j, v) in n.iter_mut().enumerate() {
                *v = self.props[p].troops(j) as i64;
            }
            for k in 0..4 {
                if self.is_troop_order(p, k) && self.props[p].order(k)[7] == crate::economy::ST_CONFIRMED {
                    for (j, v) in n.iter_mut().enumerate() {
                        *v += self.props[p].order(k)[3 + j] as i64;
                    }
                }
            }
            for j in 0..4 {
                cost[camp as usize] += WAGE[camp as usize][j] * n[j] * (pay + 100) / 200;
                let tr = self.props[p].training(j) as i64;
                if pay < 100 {
                    let d = self.r(((100 - pay) / 10 + 1) as u32) as i64;
                    self.props[p].set_training(j, (tr - d).max(0) as u8);
                } else if pay > 100 {
                    let d = self.r(((pay - 25) / 10 + 1) as u32) as i64;
                    self.props[p].set_training(j, (tr + d).min(100) as u8);
                }
            }
        }
        cost.map(|c| c as i32)
    }

    /// Owner's turn end: confirmed troop orders become troop records (rules-07 §3a).
    pub fn troop_execute(&mut self, h: usize) {
        for p in 0..PROPS {
            if self.troop_base_type(p).is_none() || self.props[p].owner() as usize != h {
                continue;
            }
            let base = self.troop_base_type(p).unwrap_or(0x10);
            for k in 0..4 {
                let mut o = self.props[p].order(k);
                if !self.is_troop_order(p, k) || o[7] != crate::economy::ST_CONFIRMED {
                    continue;
                }
                let (t, next) = match o[0] - base {
                    1 => (51, crate::economy::ST_POSSIBLE),
                    2 => (52, crate::economy::ST_POSSIBLE),
                    _ if base == 0x10 => (53, crate::economy::ST_DONE),
                    _ => (54, crate::economy::ST_DONE),
                };
                if let Some(i) = self.heap.alloc(t, h as u8, p as u8, 1) {
                    self.heap.recs[i][4] = o[2];
                    self.heap.recs[i][5..9].copy_from_slice(&o[3..7]);
                    o[7] = next;
                    self.props[p].set_order(k, o);
                }
            }
        }
    }

    /// rules-04b §4b sabotage execution.
    pub fn sabotage(&mut self, ev: &mut Vec<Event>) {
        let mut film = self.r(100) >= 90; // once per year end with 10 % (rules-12 §3.6)
        for i in self.heap.all(7, ANY, ANY) {
            let (s, t) = (self.heap.recs[i][1] as usize, self.heap.recs[i][4] as usize);
            if self.r(100) >= 25 || s >= PERSONS || !self.persons[t].used() {
                continue;
            }
            if !(self.persons[s].human() || self.persons[t].human()) {
                continue;
            }
            // 0x42B90 with kind and location 0xFF: list[rand(n)], no roll without property
            let owned: Vec<usize> = (0..PROPS).filter(|&p| self.props[p].kind() != KIND_FREE && self.props[p].owner() as usize == t).collect();
            let x = if owned.is_empty() { None } else { Some(owned[self.r(owned.len() as u32) as usize]) };
            let Some(x) = x else {
                // bug [C]: the original dereferences a null record here
                self.heap.delete(i);
                continue;
            };
            let before = self.property_value(x);
            let mut d = 0i64;
            match self.props[x].kind() {
                KIND_BRANCH => {
                    if self.props[x].has_residence() {
                        let v = self.props[x].b4() as u32 * (40 + self.r(40)) / 100;
                        self.props[x].set_b4(v as u8);
                    }
                    let w = self.props[x].wood() as u32 * (60 + self.r(30)) / 100;
                    self.props[x].set_wood(w as u16);
                    let st = self.props[x].stone() as u32 * (60 + self.r(30)) / 100;
                    self.props[x].set_stone(st as u16);
                    for k in 0..6 {
                        let n = self.props[x].trans(k) as u32 * (60 + self.r(30)) / 100;
                        self.props[x].set_trans(k, n as u8);
                    }
                    d = (before - self.property_value(x)).max(2);
                }
                KIND_CASTLE | KIND_CAMP => {
                    let a = self.props[x].b4() as u32 * (70 + self.r(26)) / 100;
                    self.props[x].set_b4(a as u8);
                    let b = self.props[x].b3() as u32 * (70 + self.r(26)) / 100;
                    self.props[x].set_b3(b as u8);
                    let k = self.r(4) as usize;
                    let n = self.props[x].troops(k) as u32 * (70 + self.r(26)) / 100;
                    self.props[x].set_troops(k, n as u8);
                    d = before - self.property_value(x);
                }
                _ => {
                    let x = (self.standing(s, 10) + self.standing(t, 10) + 100) * 2;
                    self.pay(t, x);
                }
            }
            // the message (with its rival check) comes before the refund and the retaliation (rules-04b §4b)
            let kind = match self.props[x].kind() { KIND_BRANCH => 0, KIND_CAMP => 1, KIND_CASTLE => 2, _ => 3 };
            let rival = kind == 0 && !self.persons[s].human() && self.persons[t].human() && self.rival_of(t) == Some(s);
            if self.persons[t].insurance() & 8 != 0 {
                self.add_cash(t, d as i32);
            }
            if self.r(100) < 70 {
                let y = 2 + self.r(4) as u8;
                self.retaliate(t, s, y);
            }
            ev.push(Event::Sabotage { saboteur: s as u8, target: t as u8, kind, loc: self.props[x].loc(), damage: d as i32, film: std::mem::take(&mut film), rival });
            self.heap.delete(i);
        }
    }

    fn troop_order(&mut self, t: u8, owner: usize, prop: usize, b4: u8, lo: u32, span: u32) -> Option<usize> {
        let i = self.heap.alloc(t, owner as u8, prop as u8, 2)?; // rules-07 §3b: counter 2
        self.heap.recs[i][4] = b4;
        for k in 0..4 {
            let n = self.props[prop].troops(k) as u32 * (lo + self.r(span)) / 100;
            self.heap.recs[i][5 + k] = n as u8;
        }
        Some(i)
    }

    /// `settlement_pre` (0x61410, rules-07 §4) for one owner: order validation, attacks, raids.
    pub fn settlement_pre(&mut self, owner: usize, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        let ob = owner as u8;
        for t in [51u8, 53, 54] {
            for i in self.heap.all(t, ob, ANY) {
                if !self.heap.live(i) {
                    continue;
                }
                let home = self.heap.recs[i][2] as usize;
                if self.props[home].owner() != ob || !matches!(self.props[home].kind(), KIND_CASTLE | KIND_CAMP) {
                    self.heap.delete(i);
                    continue;
                }
                match t {
                    51 => {
                        let loc = self.heap.recs[i][4] + 14;
                        let Some(tp) = (0..PROPS).find(|&q| self.props[q].loc() == loc && matches!(self.props[q].kind(), KIND_CASTLE | KIND_CAMP)) else { continue };
                        let to = self.props[tp].owner() as usize;
                        if self.persons[owner].human() || (to < PERSONS && self.persons[to].human()) {
                            let code = if self.heap.recs[i][4] < 3 { 4 } else { 5 };
                            self.battle(code, i, crate::fight::Target::Property(tp), ch, ev);
                        }
                    }
                    54 => self.raid(i, owner, ch, ev),
                    _ => {}
                }
            }
        }
    }

    /// rules-07 §4 type 52: troops move slot by slot to the garrison at location +4 + 14.
    pub fn troop_transfer(&mut self, i: usize) {
        let loc = self.heap.recs[i][4] + 14;
        if let Some(p) = (0..PROPS).find(|&p| self.props[p].loc() == loc && matches!(self.props[p].kind(), KIND_CASTLE | KIND_CAMP)) {
            for k in 0..4 {
                let v = self.props[p].troops(k).saturating_add(self.heap.recs[i][5 + k]);
                self.props[p].set_troops(k, v);
            }
        }
        self.heap.delete(i);
    }

    /// rules-07 §4 type-54 raid target selection.
    fn raid(&mut self, i: usize, owner: usize, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        let county = self.heap.recs[i][4];
        let ah = self.persons[owner].human();
        let cands: Vec<usize> = self
            .heap
            .all(9, ANY, ANY)
            .into_iter()
            .filter(|&s| {
                let r = &self.heap.recs[s];
                let (o, d) = (r[1] as usize, r[10] as usize);
                o != owner
                    && d < CITIES
                    && crate::economy::COUNTY[d] == county + 10
                    && r[9] <= 1
                    && (ah || (o < PERSONS && self.persons[o].human()))
            })
            .take(50)
            .collect();
        if cands.is_empty() {
            return;
        }
        let w = |st: &State, s: usize| heap::r16(&st.heap.recs[s], 5) as u32 * crate::economy::value(st.heap.recs[s][7] as usize) as u32;
        let total: u32 = cands.iter().map(|&s| w(self, s)).sum();
        let r = (self.r(1000) as u64 * total as u64 / 1000) as u32;
        let mut acc = 0;
        let mut pick = *cands.last().unwrap();
        for &s in &cands {
            acc += w(self, s);
            if acc > r {
                pick = s;
                break;
            }
        }
        let victim = self.heap.recs[pick][1] as usize;
        let vh = victim < PERSONS && self.persons[victim].human();
        let h = (ah && vh) as u32;
        if self.r(100) >= 30 + 40 * h {
            return;
        }
        if self.heap.recs[pick][4] & 1 != 0 {
            let e = (!vh) as u32;
            if self.r(100) >= 30 + 40 * e {
                return;
            }
        }
        let code = if self.heap.recs[pick][9] == 1 {
            3
        } else {
            let mut c = if county == 1 || (county == 0 && self.r(100) < 25) { 0 } else if self.r(100) < 50 { 1 } else { 2 };
            if c == 0 && self.r(100) < 20 {
                c = 1;
            }
            c
        };
        let troops: u32 = (5..9).map(|k| self.heap.recs[i][k] as u32).sum();
        if troops >= heap::r16(&self.heap.recs[pick], 5) as u32 / 100 {
            self.battle(code, i, crate::fight::Target::Shipment(pick), ch, ev);
        }
    }

    /// rules-07 §3 computer-owned castles and camps.
    pub fn computer_castles(&mut self, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        let ships = self.heap.all(9, ANY, ANY);
        if !ships.is_empty() {
            let avg = ships.iter().map(|&i| heap::r16(&self.heap.recs[i], 5) as u64).sum::<u64>() / ships.len() as u64;
            let f1 = (avg + self.globals.d(ga::AVG_SHIPMENT) as u64) / 2;
            self.globals.set_d(ga::AVG_SHIPMENT, f1 as u32);
        }
        // with no shipment in transit F1 is kept (rules-40 §41)
        for pi in 0..PROPS {
            let kind = self.props[pi].kind();
            let o = self.props[pi].owner() as usize;
            if !(kind == KIND_CASTLE || kind == KIND_CAMP) || o >= PERSONS || !self.persons[o].used() || self.persons[o].human() {
                continue;
            }
            let before = self.property_value(pi);
            if kind == KIND_CASTLE {
                let k = self.props[pi].loc() as usize - 14;
                let f1 = self.globals.d(ga::AVG_SHIPMENT) as u64;
                let inf = self.globals.d(ga::INFLATION) as u64;
                let add = (self.castles[k].toll_rate as u64 * 7 * f1 * inf / 400000) as u32;
                let acc = self.castles[k].acc.wrapping_add(self.r(100) + 1).wrapping_add(add);
                if let Some(v) = self.holder(10 + k, 7).filter(|&v| self.persons[v].human()) {
                    let amount = (acc as i64 * 30 / 100) as i32;
                    self.add_cash(v, amount);
                    // rules-12 §3.3: told in the year-end report when it is more than nothing
                    if amount > 0 {
                        ev.push(Event::VogtToll { vogt: v as u8, amount, castle: k as u8 });
                    }
                }
                self.castles[k].acc = 0;
            }
            // 3 purchase interest
            if !self.heap.has_key(16, pi as u8) {
                let ints: Vec<usize> = self.heap.all(17, ANY, pi as u8).into_iter().take(6).collect();
                let cand = ints
                    .iter()
                    .map(|&i| self.heap.recs[i][1] as usize)
                    .filter(|&x| x < PERSONS && self.props_of(x, kind).next().is_none())
                    .fold(None::<usize>, |best, x| match best {
                        Some(b) if self.persons[o].favour(b) > self.persons[o].favour(x) => Some(b),
                        _ => Some(x),
                    });
                if let Some(x) = cand {
                    let n = ints.len() as i64;
                    let price = (self.property_value(pi) * (100 + 10 * n) / 100) as i32;
                    if (self.r(100) as i64) < n * 80 / 6 {
                        let buy = !self.persons[x].human() || ch.ask(&Ask::BuyCastle { prop: pi as u8, price }) == 0;
                        if buy {
                            if self.persons[x].human() {
                                self.pay(x, price); // computer buyers do not pay [C, quirk]
                            }
                            self.add_cash(o, price);
                            self.props[pi].set_owner(x as u8);
                            for i in self.heap.all(17, ANY, pi as u8) {
                                self.heap.delete(i);
                            }
                            ev.push(Event::CastleSold { prop: pi as u8, buyer: x as u8, price });
                            continue;
                        }
                    }
                }
            }
            // 4 not sold
            let place = WArg::Loc(self.props[pi].loc());
            if self.props[pi].b3() < 50 && self.r(100) < 30 {
                let v = self.props[pi].b3() as u32 + 20 + self.r(10);
                self.props[pi].set_b3(v as u8);
                if kind == KIND_CASTLE {
                    ev.push(Event::WorldText { text: 0xC1E00, args: vec![WArg::P(o as u8), place.clone()] });
                }
            }
            if self.props[pi].training(0) < 70 && self.r(100) < 5 {
                ev.push(Event::WorldText { text: 0xC1E20, args: vec![WArg::P(o as u8), place.clone()] });
                for k in 0..4 {
                    let v = (self.props[pi].training(k) as u32 + 2 + self.r(10)).min(100);
                    self.props[pi].set_training(k, v as u8);
                }
            }
            if self.props[pi].b4() < 80 && self.r(100) < 50 {
                let v = self.props[pi].b4() + 1;
                self.props[pi].set_b4(v);
            }
            if self.r(100) < 40 {
                // rules-07 §3b step 6: training 40 + rand(40), then rand(4) men
                let tr = 40 + self.r(40);
                let n = self.r(4);
                self.add_troops(pi, n, tr as u8);
            }
            if kind == KIND_CASTLE && self.r(100) < 60 {
                let c = self.r(3) as u8;
                self.troop_order(53, o, pi, c, 20, 40);
            } else if kind == KIND_CAMP && self.year > 1607 && self.r(100) < 50 {
                let c = self.r(3) as u8;
                self.troop_order(54, o, pi, c, 40, 40);
            }
            // rules-07 §3b step 8: the attack branch is dead code in the original, only its two property
            // draws remain (camps, then castles)
            for kind in [KIND_CAMP, KIND_CASTLE] {
                let n = (0..PROPS).filter(|&q| self.props[q].kind() == kind).count() as u32;
                if n > 0 {
                    self.r(n);
                }
            }
            let after = self.property_value(pi);
            if after > before {
                self.pay(o, (after - before) as i32);
            }
            if kind == KIND_CASTLE && let Some(w) = self.heap.find(3, NOBODY, ANY) && !self.heap.has_key(16, pi as u8) && self.r(100) < 5 {
                self.heap.alloc(16, o as u8, pi as u8, 1);
                let year = self.year as i32 + self.heap.recs[w][3] as i32;
                ev.push(Event::WorldText { text: 0xC1E60, args: vec![place, WArg::Num(year)] });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rules-23 §4.2: a computer buyer pays the value, owns the castle and the interest records vanish.
    #[test]
    fn rules23_s4_2_private_sale() {
        let mut st = State::new_game(1, 3, &[crate::world::HumanSetup::test(1)]).unwrap();
        let o = st.current as usize;
        let pi = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CASTLE).unwrap();
        st.props[pi].set_owner(o as u8);
        let x = st.computers().next().unwrap();
        st.heap.alloc(17, x as u8, pi as u8, 4);
        let before = st.persons[o].cash();
        let v = st.property_value(pi) as i32;
        st.sell_castle(o, pi, x, &mut crate::Auto);
        assert_eq!(st.props[pi].owner() as usize, x);
        assert_eq!(st.persons[o].cash(), before + v);
        assert!(!st.heap.has_key(17, pi as u8));
    }

    /// rules-07 §3 and rules-04b §4b run on a fresh world without panicking; computer castles
    /// accumulate patrol/raid orders.
    #[test]
    fn rules07_s3_computer_castles_smoke() {
        let mut st = State::new_world(5);
        st.year = 1610;
        let mut ev = vec![];
        for _ in 0..10 {
            st.computer_castles(&mut crate::Auto, &mut ev);
        }
        assert!(!st.heap.all(53, ANY, ANY).is_empty() || !st.heap.all(54, ANY, ANY).is_empty());
        st.saboteur(0, 1, 100);
        for _ in 0..20 {
            st.sabotage(&mut ev);
        }
    }

    /// rules-07 §3a: order book of a camp: feasibility, reserving, execution into records, pay.
    #[test]
    fn rules07_s3a_order_book() {
        use crate::economy::{ST_CONFIRMED, ST_DONE, ST_IMPOSSIBLE, ST_POSSIBLE};
        let mut st = State::new_world(5);
        let camp = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CAMP).unwrap();
        let h = 0usize;
        st.props[camp].set_owner(h as u8);
        for k in 0..4 {
            st.props[camp].set_troops(k, 10);
            st.props[camp].set_training(k, 50);
        }
        // patrol (0x23) with 3 + 0 + 0 + 4 men
        let mut o = [0x23, 0, 1, 3, 0, 0, 4, ST_POSSIBLE];
        st.props[camp].set_order(0, o);
        assert!(st.order_feasible(camp, 0));
        // too many men, or none at all, are impossible
        o[3] = 11;
        st.props[camp].set_order(1, o);
        assert!(!st.order_feasible(camp, 1));
        o[3..7].copy_from_slice(&[0; 4]);
        st.props[camp].set_order(1, o);
        assert!(!st.order_feasible(camp, 1));
        // confirming takes the men out of the garrison, unconfirming returns them
        st.reserve(camp, 0, true);
        assert_eq!((st.props[camp].troops(0), st.props[camp].troops(3)), (7, 6));
        let mut c = st.props[camp].order(0);
        c[7] = ST_CONFIRMED;
        st.props[camp].set_order(0, c);
        // pay 100 %: wage * men * 200 / 200, the reserved men counted in; training untouched
        st.persons[h].set_pay_robbers(100);
        assert_eq!(st.troop_pay(h), [0, (75 + 100 + 100 + 125) * 10], "a camp pays the Räuber row");
        assert_eq!(st.props[camp].training(0), 50);
        // the patrol (type 54 for a camp) becomes a record and stays executed
        st.troop_execute(h);
        let r = st.heap.all(54, h as u8, ANY);
        assert_eq!(r.len(), 1);
        assert_eq!(&st.heap.recs[r[0]][4..9], &[1, 3, 0, 0, 4]);
        assert_eq!(st.props[camp].order(0)[7], ST_DONE);
        // the next turn start returns the men, then re-reserves the repeating order
        let _ = ST_IMPOSSIBLE;
        st.props[camp].set_b4(150); // room for all 40 men, else the garrison trim cuts them
        st.year = 1601; // the return is part of the report block, skipped in 1600 (rules-37 §1)
        st.turn_start(h);
        assert!(st.heap.all(54, h as u8, ANY).is_empty());
        assert_eq!((st.props[camp].troops(0), st.props[camp].troops(3)), (7, 6));
        assert_eq!(st.props[camp].order(0)[7], ST_CONFIRMED);
    }
}
