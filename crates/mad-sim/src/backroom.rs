//! Back room (docs/re/rules-04-backroom-laws.md §1-§3, corrected by rules-04b §4.2).

use crate::{Event, WArg};

/// rules-04 §5 message box with person args.
fn mbox(text: u32, ps: &[usize], rect: [i32; 4]) -> Event {
    Event::MsgBox { text, args: ps.iter().map(|&p| WArg::P(p as u8)).collect(), rect }
}
use crate::heap::{self, ANY};
use crate::laws::has;
use crate::state::*;

impl State {
    /// rules-04 §2 / rules-04b §4.2: office-tree price for every person q (2 RNG calls each, index
    /// order). Multiplier default 10 (rules-04b corrects rules-04's 7); 47 → 7, 73 → 6, 78 → 5.
    pub fn office_tree_prices(&mut self, player: usize) -> Vec<u32> {
        let mut mult = 10;
        for (code, m) in [(47, 7), (73, 6), (78, 5)] {
            if has(self, player, code) {
                mult = m;
            }
        }
        (0..PERSONS)
            .map(|q| {
                if !self.persons[q].used() {
                    return 0;
                }
                let a = self.standing(player, 10) as u32;
                let b = self.standing(q, 10) as u32;
                (a + 100 + b) * mult / 10
            })
            .collect()
    }

    /// §3.1 bribe. §5.1: no cash → 0xBC5CC; amount 0 (or a right click) closes silently.
    pub fn bribe(&mut self, player: usize, target: usize, x: i32) -> Option<Event> {
        if self.persons[player].cash() <= 0 {
            return Some(mbox(0xBC5CC, &[target], [160, 160, 320, 160]));
        }
        if x <= 0 {
            return None;
        }
        self.pay(player, x);
        self.add_cash(target, x);
        let mut gain = (x as i64 * 400 / self.net_worth(target).max(100) as i64).min(100) as i32;
        if self.persons[player].gender() != self.persons[target].gender() {
            gain = gain * 11 / 10;
        }
        self.favour_increase(player, target, gain);
        let f = self.globals.d(ga::TURN_FLAGS) | 0x04;
        self.globals.set_d(ga::TURN_FLAGS, f);
        if let Some(i) = self.heap.find(0, player as u8, ANY)
            && self.heap.recs[i][4] == 6 {
                let r = &mut self.heap.recs[i];
                let v = heap::r32(r, 9).wrapping_add(x as u32);
                heap::w32(r, 9, v);
            }
        None
    }

    pub fn spy_contract(&self, owner: usize, target: usize) -> Option<usize> {
        self.heap.all(6, owner as u8, ANY).into_iter().find(|&i| self.heap.recs[i][4] as usize == target)
    }

    /// §3.2 hire / pause / resume spies. `price` from `office_tree_prices`.
    pub fn spies(&mut self, player: usize, target: usize, price: u32) {
        match self.spy_contract(player, target) {
            None => {
                if let Some(i) = self.heap.alloc(6, player as u8, NONE, 20) {
                    let r = &mut self.heap.recs[i];
                    r[4] = target as u8;
                    heap::w32(r, 5, price / 3);
                    r[9] = 1;
                }
            }
            Some(i) => {
                let r = &mut self.heap.recs[i];
                if r[9] != 0 {
                    r[9] = 0;
                } else {
                    r[9] = 1;
                    r[3] = 20;
                }
            }
        }
    }

    /// Cancel a paused contract without evidence.
    pub fn cancel_spies(&mut self, player: usize, target: usize) {
        if let Some(i) = self.spy_contract(player, target) {
            let r = &self.heap.recs[i];
            if r[9] == 0 && heap::r32(r, 0x0B) == 0 {
                self.heap.delete(i);
            }
        }
    }

    /// §3.2 per-contract turn-end handler, after the countdown decrement.
    pub fn spy_tick(&mut self, ri: usize, ev: &mut Vec<Event>) {
        let r = &mut self.heap.recs[ri];
        if r[3] == 0 {
            self.heap.delete(ri);
            return;
        }
        if r[3] < 16 {
            r[9] = 0;
        } else if r[10] == 0 && heap::r32(r, 0x0B).count_ones() > 2 {
            r[10] = 1;
            let (owner, target) = (r[1], r[4]);
            ev.push(Event::SpyNotified { owner, target, film: std::mem::take(&mut self.informant_gate) });
        }
    }

    /// §3.3 saboteurs.
    pub fn saboteur(&mut self, player: usize, target: usize, price: u32) {
        if let Some(i) = self.heap.alloc(7, player as u8, NONE, 5) {
            let r = &mut self.heap.recs[i];
            r[4] = target as u8;
            heap::w32(r, 5, price);
        }
    }

    /// The player's running saboteur contract against `target`, if any.
    pub fn saboteur_on(&self, player: usize, target: usize) -> Option<usize> {
        self.heap.all(7, player as u8, ANY).into_iter().find(|&i| self.heap.recs[i][4] == target as u8)
    }

    /// Recall the saboteurs (the dialog's "Saboteure zurückpfeifen").
    pub fn cancel_saboteur(&mut self, player: usize, target: usize) {
        if let Some(i) = self.saboteur_on(player, target) {
            self.heap.delete(i);
        }
    }

    /// §3.4 blackmail. Outcome = bit 1 of the low cash byte of the target (quirk [C]); texts §5.2.
    pub fn blackmail(&mut self, player: usize, target: usize) -> Event {
        if self.persons[target].favour(player) > 100 {
            return mbox(0xBC678, &[target], [160, 180, 320, 120]);
        }
        let c = self.spy_contract(player, target);
        let Some(c) = c.filter(|&c| heap::r32(&self.heap.recs[c], 0x0B).count_ones() >= 3) else {
            return mbox(0xBC698, &[target], [160, 160, 320, 160]);
        };
        if self.persons[target].0[0x18] & 2 == 0 {
            return mbox(0xBC760, &[target], [160, 140, 320, 200]);
        }
        self.persons[target].set_favour(player, 101);
        self.heap.delete(c);
        if let Some(i) = self.heap.find(0, player as u8, ANY)
            && self.heap.recs[i][4] == 6 {
                let r = &mut self.heap.recs[i];
                let v = heap::r32(r, 5) + 1;
                heap::w32(r, 5, v);
            }
        mbox(0xBC6CC, &[target; 3], [160, 140, 320, 200])
    }

    /// §3.5 denounce v to listener l, texts and order §5.3. `None`: l = v, the pick is repeated.
    pub fn denounce(&mut self, player: usize, v: usize, l: usize) -> Option<Event> {
        if l == v {
            return None;
        }
        let a = ga::DENOUNCE + l;
        Some(match self.globals.b(a) {
            0 => {
                let c = self.spy_contract(player, v).filter(|&i| heap::r32(&self.heap.recs[i], 0x0B) != 0);
                // rand(100) is always drawn; no evidence takes the failure branch
                if (self.r(100) as u8) < self.persons[l].favour(player) && c.is_some() {
                    self.persons[v].set_denounced(1);
                    let x = self.ri(20);
                    self.favour_decrease(v, l, x);
                    let y = self.ri(10);
                    self.favour_decrease(player, l, y);
                    self.globals.set_b(a, 1);
                    mbox(0xBCF60, &[l], [160, 180, 320, 120])
                } else {
                    let x = self.ri(30);
                    self.favour_decrease(player, l, x);
                    let y = 10 + self.ri(10);
                    self.favour_decrease(player, v, y);
                    if self.r(100) < 30 {
                        self.retaliate(v, player, 4);
                    }
                    mbox(0xBCF98, &[l, v], [160, 160, 320, 160])
                }
            }
            1 => {
                self.globals.set_b(a, 2);
                mbox(0xBCE3C, &[l, player], [160, 160, 320, 160])
            }
            2 => {
                let x = 1 + self.ri(7);
                self.favour_decrease(player, l, x);
                self.globals.set_b(a, 3);
                mbox(0xBCEA8, &[l, player], [160, 160, 320, 160])
            }
            _ => {
                let x = 10 + self.ri(20);
                self.favour_decrease(player, l, x);
                // deviation (rules-56 §5): the original pushes L only, its %1N0 reads an unset slot; we pass P as the siblings do
                mbox(0xBCEFC, &[l, player], [160, 140, 320, 200])
            }
        })
    }

    /// §3.6 set a and b against each other.
    pub fn sow_discord(&mut self, player: usize, a: usize, b: usize) {
        let x = 20 + self.ri(20);
        self.favour_decrease(a, b, x);
        let y = 20 + self.ri(20);
        self.favour_decrease(b, a, y);
        if self.r(100) < 50 {
            self.retaliate(a, b, 4);
            self.retaliate(b, a, 4);
        }
        if self.r(100) < 20 {
            self.retaliate(a, player, 3);
        }
        if self.r(100) < 20 {
            self.retaliate(b, player, 3);
        }
    }

    /// rules-09 §4 `retaliate(actor, victim, years)`: type-50 record if actor computer, victim
    /// human and none exists yet.
    pub fn retaliate(&mut self, actor: usize, victim: usize, years: u8) {
        if self.persons[actor].human() || !self.persons[victim].human() {
            return;
        }
        if self.heap.all(50, actor as u8, ANY).into_iter().any(|i| self.heap.recs[i][4] as usize == victim) {
            return;
        }
        if let Some(i) = self.heap.alloc(50, actor as u8, NONE, years) {
            self.heap.recs[i][4] = victim as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::HumanSetup;

    /// rules-04 §3.1: gain = X*400/max(100, NW) capped 100, +10 % across genders.
    #[test]
    fn rules04_s3_1_bribe() {
        let mut st = State::new_game(2, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        let t = (0..PERSONS).find(|&i| !st.persons[i].human() && st.persons[i].religion() == st.persons[h].religion() && st.persons[i].gender() == 0).unwrap();
        st.persons[t].set_favour(h, 40);
        let nw = st.net_worth(t) + 500; // the target already holds X when the gain is computed
        st.bribe(h, t, 500);
        let gain = (500i64 * 400 / nw.max(100) as i64).min(100) as u8;
        assert_eq!(st.persons[t].favour(h), 40 + gain);
        assert_eq!(st.globals.d(ga::TURN_FLAGS) & 4, 4);
    }

    /// rules-04 §3.2: contract goes inactive once the counter is below 16. The doc's "(after 4
    /// turn ends)" does not match 20 - 4 = 16; the explicit condition is used (5th turn end).
    #[test]
    fn rules04_s3_2_spy_lifecycle() {
        let mut st = State::blank(1);
        st.spies(0, 5, 300);
        let i = st.spy_contract(0, 5).unwrap();
        assert_eq!(heap::r32(&st.heap.recs[i], 5), 100);
        let mut ev = vec![];
        for _ in 0..4 {
            st.heap.recs[i][3] -= 1;
            st.spy_tick(i, &mut ev);
        }
        assert_eq!(st.heap.recs[i][9], 1);
        st.heap.recs[i][3] -= 1;
        st.spy_tick(i, &mut ev);
        assert_eq!(st.heap.recs[i][9], 0);
    }

    fn boxed(e: Option<Event>) -> (u32, [i32; 4]) {
        match e {
            Some(Event::MsgBox { text, rect, .. }) => (text, rect),
            e => panic!("{e:?}"),
        }
    }

    /// rules-04 §5.1: no cash → 0xBC5CC; amount 0 is silent; neither pays, changes favour, draws or sets the flag.
    #[test]
    fn rules04_s5_1_bribe_refusals() {
        let mut st = State::new_game(2, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        let t = (0..PERSONS).find(|&i| st.persons[i].used() && !st.persons[i].human()).unwrap();
        let (rng, fav, f0) = (st.rng, st.persons[t].favour(h), st.globals.d(ga::TURN_FLAGS));
        assert_eq!(st.bribe(h, t, 0), None);
        st.persons[h].set_cash_raw(0);
        assert_eq!(boxed(st.bribe(h, t, 500)), (0xBC5CC, [160, 160, 320, 160]));
        assert_eq!((st.rng, st.persons[t].favour(h), st.globals.d(ga::TURN_FLAGS), st.persons[h].cash()), (rng, fav, f0, 0));
    }

    /// rules-04 §5.2: the order fav > 100, evidence < 3, cash bit 1; no RNG anywhere.
    #[test]
    fn rules04_s5_2_blackmail_texts() {
        let mut st = State::new_game(2, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        let t = (0..PERSONS).find(|&i| st.persons[i].used() && !st.persons[i].human()).unwrap();
        let rng = st.rng;
        st.persons[t].set_favour(h, 50);
        assert_eq!(boxed(Some(st.blackmail(h, t))).0, 0xBC698, "no contract counts as 0 bits");
        st.spies(h, t, 0);
        let c = st.spy_contract(h, t).unwrap();
        heap::w32(&mut st.heap.recs[c], 0x0B, 0b11);
        assert_eq!(boxed(Some(st.blackmail(h, t))), (0xBC698, [160, 160, 320, 160]));
        heap::w32(&mut st.heap.recs[c], 0x0B, 0b111);
        st.persons[t].set_cash_raw(4);
        assert_eq!(boxed(Some(st.blackmail(h, t))), (0xBC760, [160, 140, 320, 200]));
        assert!(st.spy_contract(h, t).is_some());
        st.persons[t].set_cash_raw(6);
        assert_eq!(boxed(Some(st.blackmail(h, t))), (0xBC6CC, [160, 140, 320, 200]));
        assert!(st.spy_contract(h, t).is_none());
        assert_eq!(boxed(Some(st.blackmail(h, t))), (0xBC678, [160, 180, 320, 120]));
        assert_eq!(st.rng, rng);
    }

    /// rules-04 §5.3: per-listener states with their draws; no evidence is the failure branch (4 draws).
    #[test]
    fn rules04_s5_3_denounce_states() {
        let mut st = State::new_game(2, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        let mut cs = (0..PERSONS).filter(|&i| st.persons[i].used() && !st.persons[i].human());
        let (v, l) = (cs.next().unwrap(), cs.next().unwrap());
        assert_eq!(st.denounce(h, v, v), None);
        st.persons[l].set_favour(h, 100);
        let mut r = st.rng;
        assert_eq!(boxed(st.denounce(h, v, l)), (0xBCF98, [160, 160, 320, 160]));
        for n in [100, 30, 10, 100] {
            r.n(n);
        }
        assert_eq!((st.rng, st.globals.b(ga::DENOUNCE + l)), (r, 0));
        st.spies(h, v, 0);
        let c = st.spy_contract(h, v).unwrap();
        heap::w32(&mut st.heap.recs[c], 0x0B, 1);
        st.persons[l].set_favour(h, 100);
        assert_eq!(boxed(st.denounce(h, v, l)), (0xBCF60, [160, 180, 320, 120]));
        assert_eq!((st.globals.b(ga::DENOUNCE + l), st.persons[v].denounced()), (1, 1));
        let (r, f) = (st.rng, st.persons[l].favour(h));
        assert_eq!(boxed(st.denounce(h, v, l)), (0xBCE3C, [160, 160, 320, 160]));
        assert_eq!((st.rng, st.persons[l].favour(h), st.globals.b(ga::DENOUNCE + l)), (r, f, 2));
        let mut r2 = r;
        r2.n(7);
        assert_eq!(boxed(st.denounce(h, v, l)), (0xBCEA8, [160, 160, 320, 160]));
        assert_eq!((st.rng, st.globals.b(ga::DENOUNCE + l)), (r2, 3));
        r2.n(20);
        assert_eq!(boxed(st.denounce(h, v, l)), (0xBCEFC, [160, 140, 320, 200]));
        assert_eq!((st.rng, st.globals.b(ga::DENOUNCE + l)), (r2, 3));
    }
}
