//! Debt seizure and jury court (docs/re/rules-02-economy.md §10).

use crate::heap::{self, ANY};
use crate::state::*;
use crate::{Ask, Choices, Event};

impl State {
    /// The county of the jury court: table 0x868D4 over p's office location (cities by county, counties
    /// themselves, everything else 10; rules-15 §6.1).
    pub fn jury_county(loc: u8) -> u8 {
        match loc {
            0..=9 => crate::economy::COUNTY[loc as usize],
            10..=12 => loc,
            _ => 10,
        }
    }

    fn seize(&mut self, h: usize, debt: &mut i32, x: i32) {
        self.add_cash(h, x);
        *debt -= x;
    }

    /// §10.1 - §10.3. Returns true when the jury court ran (the death check is then skipped).
    pub fn debt_check(&mut self, h: usize, ch: &mut dyn Choices, ev: &mut Vec<Event>) -> bool {
        let nw = self.net_worth(h);
        let cash = self.persons[h].cash();
        if !(cash < 0 && cash < -nw / 20) {
            return false;
        }
        let mut debt = -nw / 20 - cash;
        let (debt0, nw0) = (debt.max(0), nw.max(0));
        let branches = |st: &State| st.props_of(h, KIND_BRANCH).collect::<Vec<_>>();
        let b_count = branches(self).len();
        let mut n = [0u32; 4];
        // stage 1: goods at half price
        for b in branches(self) {
            let c = self.props[b].loc() as usize;
            for g in 0..GOODS {
                if debt <= 0 {
                    break;
                }
                let price = self.cities[c].price(g) as i32;
                let q = (debt * 2 / price + 1).min(self.props[b].stock(g) as i32);
                let s = self.props[b].stock(g) - q as u16;
                self.props[b].set_stock(g, s);
                n[1] += q as u32;
                self.seize(h, &mut debt, price * q / 2);
            }
        }
        // stage 2: transporters at half the purchase price, condition ignored
        for b in branches(self) {
            let c = self.props[b].loc() as usize;
            let mut t = TRANSPORTERS as i32 - 1;
            while debt > 0 && t >= 0 {
                let tu = t as usize;
                if self.props[b].trans(tu) == 0 {
                    t -= 1;
                    continue;
                }
                let k = self.props[b].trans(tu) - 1;
                self.props[b].set_trans(tu, k);
                n[2] += 1;
                let x = self.cities[c].tprice(tu) as i32 / 2;
                self.seize(h, &mut debt, x);
            }
        }
        // stage 3: storage, at most 40 % per area
        for b in branches(self) {
            let c = self.props[b].loc() as usize;
            for stone in [false, true] {
                if debt <= 0 {
                    break;
                }
                let level = if stone { self.cities[c].level_b() } else { self.cities[c].level_a() } as i32;
                let area = if stone { self.props[b].stone() } else { self.props[b].wood() } as i32;
                let q = (debt / level.max(1) + 1).min(area * 2 / 5);
                if stone { self.props[b].set_stone((area - q) as u16) } else { self.props[b].set_wood((area - q) as u16) }
                n[3] += q as u32;
                self.seize(h, &mut debt, level * q / 2);
            }
        }
        // header and the stage 1..3 summaries come before the per-branch lines (rules-23 §3)
        ev.push(Event::Seized { person: h as u8, goods: n[1], transporters: n[2], storage: n[3] });
        // stage 4: residence. Quirk [C]: F = first branch, fixed for the stage.
        let bs = branches(self);
        if let Some(&f) = bs.first() {
            for b in bs.clone() {
                if debt <= 0 {
                    break;
                }
                if let Some(i) = self.heap.find(13, ANY, f as u8) {
                    let word = heap::r16(&self.heap.recs[i], 4);
                    let old = self.props[b].residence();
                    self.props[b].set_residence(word);
                    let v = self.residence_value_of(b) / 2;
                    self.props[b].set_residence(old);
                    self.heap.delete(i);
                    self.props[b].set_residence(0xFFFF);
                    self.seize(h, &mut debt, v as i32);
                    ev.push(Event::SeizedResidence { city: self.props[b].loc(), word, site: true });
                } else if self.props[b].has_residence() {
                    if let Some(i) = self.heap.find(14, ANY, f as u8) {
                        self.heap.delete(i);
                    }
                    let v = self.residence_value_of(b) / 2;
                    let word = self.props[b].residence();
                    self.props[b].set_residence(0xFFFF);
                    self.seize(h, &mut debt, v as i32);
                    ev.push(Event::SeizedResidence { city: self.props[b].loc(), word, site: false });
                }
            }
        }
        // stage 5: whole branch or jury
        let bs = branches(self);
        let f = bs.first().copied();
        let mut b_left = b_count;
        for (k, &b) in bs.iter().enumerate() {
            if self.props[b].kind() != KIND_BRANCH || self.props[b].owner() as usize != h {
                continue;
            }
            if debt <= 0 && self.r(50) == 0 {
                continue;
            }
            let (x, m) = bs[k..]
                .iter()
                .filter(|&&y| self.props[y].kind() == KIND_BRANCH && self.props[y].owner() as usize == h)
                .map(|&y| (y, self.property_value(y)))
                .min_by_key(|&(_, v)| v)
                .unwrap();
            let m = m.max(0) as i32;
            let jury = if b_left <= 1 && debt <= m / 2 {
                true
            } else {
                let r = self.r(100);
                r >= 50 || debt <= 0
            };
            if jury {
                self.jury(h, debt0, nw0, b_left, ch, ev);
                return true;
            }
            self.seize(h, &mut debt, m);
            let y = (0..PROPS).find(|&y| y != x && self.props[y].kind() == KIND_BRANCH && self.props[y].owner() as usize == h);
            // with no other branch left the shipments keep their source (rules-40 §28)
            if let (Some(f), Some(y)) = (f, y) {
                for i in self.heap.all(9, h as u8, f as u8) {
                    self.heap.recs[i][2] = y as u8;
                }
            }
            let city = self.props[x].loc();
            let redirect = y.filter(|_| f.is_some()).map(|y| self.props[y].loc());
            self.free_property(x);
            b_left -= 1;
            ev.push(Event::SeizedBranch { prop: x as u8, city, redirect });
        }
        false
    }

    /// §10.3 jury court.
    fn jury(&mut self, h: usize, debt0: i32, nw0: i32, b_left: usize, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        // the turn-end lines so far (a gift's reaction among them) come before the court (rules-63 §2 order)
        ch.show(ev);
        let x = ch.ask(&Ask::JuryBribe { person: h as u8, county: Self::jury_county(self.persons[h].loc()) }) as i32;
        self.pay(h, x);
        let pg = ((debt0 as i64 - x as i64) * 100 / nw0.max(1) as i64 + 50).max(0) as u32;
        self.r(4);
        let mut guilty = 0;
        for _ in 0..12 {
            self.r(12);
            if self.r(100) < pg {
                guilty += 1;
            }
            self.r(3);
        }
        let tower = guilty >= 7;
        if tower {
            self.heap.alloc(20, h as u8, NONE, 1);
            let d = self.persons[h].deeds() | 0x10;
            self.persons[h].set_deeds(d);
        }
        // the tower screen's ambient-sound rand() calls are deliberately not drawn (rules-02 §10.3); the acquittal's
        // sound(0x1E) is a group draw
        let song = if tower { 0x0B } else { self.song(0x1E) };
        ev.push(Event::Jury { person: h as u8, guilty: guilty as u8, tower, last_branch: b_left < 2, song });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::HumanSetup;

    /// rules-02 §10.1: no seizure while cash >= -NW/20.
    #[test]
    fn rules02_s10_1_trigger() {
        let mut st = State::new_game(1, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        let mut ev = vec![];
        assert!(!st.debt_check(h, &mut crate::Auto, &mut ev));
        assert!(ev.is_empty());
    }

    /// rules-02 §10.2: a large debt sells the stock first, then ends at the jury or a seized branch.
    #[test]
    fn rules02_s10_2_stages() {
        let mut st = State::new_game(1, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        let b = st.branch_at(h, 1).unwrap();
        st.props[b].set_stock(4, 100);
        st.persons[h].set_cash_raw(-1_000_000);
        let mut ev = vec![];
        let court = st.debt_check(h, &mut crate::Auto, &mut ev);
        assert_eq!(st.props[b].stock(4), 0);
        assert!(court || st.branches(h) == 0);
        // the debt here is far above NW, so pg > 100: always the tower
        assert!(ev.iter().any(|e| matches!(e, Event::Jury { tower: true, .. })) || !court);
    }
}
