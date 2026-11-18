//! Sieges, plunder, tribute and relief (docs/re/rules-07-castles-bandits-combat.md §6).

use crate::heap::{self, ANY};
use crate::laws::has;
use crate::state::*;
use crate::{Ask, Choices, Event};

impl State {
    /// §6.3 type-25 handler, after the countdown.
    pub fn siege_tick(&mut self, ri: usize, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        let r = self.heap.recs[ri].clone();
        let (b, c, a) = (r[1] as usize, r[4] as usize, heap::r16(&r, 5) as u32);
        for x in self.computers().filter(|&x| self.persons[x].loc() as usize == c).collect::<Vec<_>>() {
            let v = 1 + self.ri(5);
            self.favour_decrease(b, x, v);
        }
        if r[3] >= 3 {
            ev.push(Event::SiegeArrives { city: c as u8, besieger: b as u8, army: a as u16 });
            return;
        }
        let human = self.persons[b].human();
        let p = self.cities[c].population();
        let n = 100 - 10 * has(self, b, 38) as u32 - 20 * has(self, b, 67) as u32;
        let k = if human { 50 } else { 55 };
        let (mut won_choice, mut paid) = (3u8, (0u32, 0u8));
        if self.r(n) < a.wrapping_mul(k) / p.max(1) {
            self.heap.delete(ri);
            if !human {
                self.plunder(b, c);
            } else {
                if let Some(i) = self.heap.find(0, b as u8, ANY)
                    && self.heap.recs[i][4] == 22
                {
                    let v = heap::r32(&self.heap.recs[i], 5) + 1;
                    heap::w32(&mut self.heap.recs[i], 5, v);
                }
                let choice = ch.ask(&Ask::SiegeWon { city: c as u8 }).min(2);
                if choice != 1 {
                    self.plunder(b, c);
                }
                if choice != 0 {
                    paid = self.tribute(b, c);
                }
                won_choice = choice as u8;
            }
            ev.push(Event::SiegeWon { besieger: b as u8, city: c as u8, choice: won_choice, tribute: paid });
        } else if a < p / 5 && self.r(100) < 50 {
            self.heap.delete(ri);
            ev.push(Event::SiegeCollapsed { city: c as u8, besieger: b as u8 });
        } else if !human {
            ev.push(Event::SiegeGoesOn { city: c as u8, besieger: b as u8 });
            self.heap.recs[ri][3] = 2;
        } else if ch.ask(&Ask::ContinueSiege { city: c as u8, cost: (a * 5) as i32 }) == 0 {
            self.pay(b, (a * 5) as i32);
            self.heap.recs[ri][3] = 2;
        } else {
            self.heap.delete(ri);
        }
    }

    /// §6.4 plunder (0x1F100).
    pub fn plunder(&mut self, b: usize, c: usize) {
        let w = (self.cities[c].wealth() as i32 - 20 - self.ri(30)).max(0);
        self.cities[c].set_wealth(w as u8);
        let p = (self.cities[c].population() as u64 * (70 + self.r(20)) as u64 / 100).max(1000);
        self.cities[c].set_population(p as u32);
        for x in self.computers().filter(|&x| self.persons[x].loc() as usize == c).collect::<Vec<_>>() {
            let v = 20 + self.ri(40);
            self.favour_decrease(b, x, v);
        }
        for i in 0..PROPS {
            let pr = &mut self.props[i];
            if pr.kind() == KIND_BRANCH && pr.loc() as usize == c && pr.owner() as usize != b {
                for g in 0..GOODS {
                    pr.set_stock(g, pr.stock(g) >> 1);
                }
                pr.set_wood(pr.wood() >> 1);
                for t in 0..TRANSPORTERS {
                    pr.set_cond(t, pr.cond(t) >> 1);
                }
            }
        }
    }

    /// §6.5 tribute: type 27, ransom 2x at once.
    /// Draw order rand(1000) (amount), then rand(4) (years) as traced in rules-21 §2.2.
    fn tribute(&mut self, b: usize, c: usize) -> (u32, u8) {
        let amount = self.cities[c].population() + 100 + self.r(1000);
        let n = 3 + self.r(4) as u8;
        if let Some(i) = self.heap.alloc(27, b as u8, NONE, n) {
            self.heap.recs[i][4] = c as u8;
            heap::w32(&mut self.heap.recs[i], 5, amount);
        }
        self.add_cash(b, (2 * amount) as i32);
        (amount, n)
    }

    /// §6.5 type-27 handler.
    pub fn tribute_tick(&mut self, ri: usize, ev: &mut Vec<Event>) {
        let r = &self.heap.recs[ri];
        let (o, amount, c, city) = (r[1] as usize, heap::r32(r, 5) as i32, r[3], r[4]);
        self.add_cash(o, amount);
        ev.push(Event::Tribute { city, amount });
        if c == 0 {
            self.heap.delete(ri);
        }
    }

    /// §6.6 relief resolution (type 26).
    pub fn relief_tick(&mut self, ri: usize, ev: &mut Vec<Event>) {
        let r = self.heap.recs[ri].clone();
        self.heap.delete(ri);
        let (o, c, ar) = (r[1] as usize, r[4], heap::r16(&r, 5) as u32);
        let Some(si) = self.heap.all(25, ANY, ANY).into_iter().find(|&i| self.heap.recs[i][4] == c) else {
            ev.push(Event::ReliefNoSiege { city: c, owner: o as u8 });
            return;
        };
        let s = self.heap.recs[si][1] as usize;
        let as_ = heap::r16(&self.heap.recs[si], 5) as u32;
        let p = self.cities[c as usize].population();
        let bonus = |st: &State, x: usize| 10 * has(st, x, 38) as u32 + 20 * has(st, x, 67) as u32;
        let (bs, br) = (bonus(self, s), bonus(self, o));
        let ok = if self.persons[o].human() {
            as_ + bs < ar + p / 10 + br
        } else {
            let x = self.r(100);
            let y = self.r(100);
            as_ + y + bs < (p >> 3) + ar + x + br
        };
        let mut song = 0;
        if ok {
            self.heap.delete(si);
            if self.persons[o].human() {
                self.add_cash(o, p as i32);
            }
            // sound(0x1E) is the random fanfare group {7, 0x0E}, drawn from the game RNG (rules-14 §4.1)
            song = [7, 0x0E][self.r(2) as usize];
        }
        ev.push(Event::Relief { city: c, ok, besieger: s as u8, owner: o as u8, pop: p, song });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rules-07 §6.2/§6.3: counter 3 only talks; an army of the whole population wins next time.
    #[test]
    fn rules07_s6_siege_timeline() {
        let mut st = State::new_world(1);
        let b = 140;
        let i = st.heap.alloc(25, b as u8, NONE, 3).unwrap();
        st.heap.recs[i][4] = 2;
        heap::w16(&mut st.heap.recs[i], 5, 50000);
        let mut ev = vec![];
        st.siege_tick(i, &mut crate::Auto, &mut ev);
        assert!(st.heap.live(i));
        st.heap.recs[i][3] = 2;
        let pop = st.cities[2].population();
        st.siege_tick(i, &mut crate::Auto, &mut ev);
        assert!(!st.heap.live(i), "army*55/pop >= 100 always wins");
        assert!(st.cities[2].population() < pop);
    }

    /// rules-07 §6.6 human relief needs A_s + bonus < A_r + P/10 + bonus; no RNG for the outcome, one `rand(2)` for the
    /// fanfare group of `sound(0x1E)` on success (rules-14 §4.1).
    #[test]
    fn rules07_s6_6_relief() {
        let mut st = State::new_game(1, 3, &[crate::world::HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        let s = st.heap.alloc(25, 140, NONE, 2).unwrap();
        st.heap.recs[s][4] = 2;
        heap::w16(&mut st.heap.recs[s], 5, 600);
        let r = st.heap.alloc(26, h as u8, NONE, 3).unwrap();
        st.heap.recs[r][4] = 2;
        heap::w16(&mut st.heap.recs[r], 5, 400); // 600 < 400 + 2500/10 = 650
        let mut rng = st.rng;
        let mut ev = vec![];
        st.relief_tick(r, &mut ev);
        assert!(matches!(ev[..], [Event::Relief { city: 2, ok: true, song: 7 | 14, .. }]));
        assert!(!st.heap.live(s));
        rng.rand();
        assert_eq!(st.rng, rng);
    }
}
