//! Auctions (docs/re/rules-08-auction-feast-duel.md §1).

use crate::economy::*;
use crate::heap::ANY;
use crate::state::*;
use crate::{Ask, Choices, Event};

/// Thresholds 0x84BEC and increments 0x84C38, 19 entries each; 15..17 repeat 12..14 (rules-39 §10).
const STEP_AT: [i32; 19] = [100, 200, 500, 1000, 2000, 5000, 10000, 20000, 50000, 100000, 200000, 500000, 1_000_000, 2_000_000, 5_000_000, 1_000_000, 2_000_000, 5_000_000, 10_000_000];
/// Auctioneer frame table (rules-25 §1.1) per bank 0xD5..0xD8: hammer ends T0..T2, gesture end T3, return start T4.
const AUCTIONEER: [[u8; 5]; 4] = [[27, 39, 63, 14, 56], [31, 43, 67, 19, 60], [35, 47, 71, 22, 64], [35, 47, 71, 24, 64]];
const STEP: [i32; 19] = [5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000, 25000, 50000, 100000, 250000, 50000, 100000, 250000, 1_000_000];

/// `i` = the number of *leading* thresholds ≤ price, capped at 18 (rules-39 §10): from 5 000 000 on the scan runs
/// through the repeated entries to i = 18.
pub fn increment(price: i32) -> i32 {
    STEP[STEP_AT.iter().take_while(|&&t| t <= price).count().min(18)]
}

/// Lot kinds for bidder eligibility: city 0..9 (residence), 10 none, 11 port branch.
#[derive(Clone, Copy)]
enum Need {
    Any,
    Port,
    BranchIn(usize),
}

impl State {
    /// Yearly announcement step (§1.1).
    pub fn auction_announce(&mut self, ev: &mut Vec<Event>) {
        let pause = self.globals.b(ga::AUCTION_PAUSE);
        if pause != 0 {
            self.globals.set_b(ga::AUCTION_PAUSE, pause - 1);
            return;
        }
        if self.r(100) < 20 && self.heap.find(3, NOBODY, ANY).is_none() {
            let c = 2 + self.r(3) as u8;
            self.heap.alloc(3, NOBODY, NONE, c);
            ev.push(Event::AuctionAnnounced { years: c });
        }
    }

    pub fn auction_pending(&self) -> bool {
        self.heap.find(3, NOBODY, ANY).is_some()
    }

    /// rules-24 §2: one lot holds at most 99 transporters per owner, branch and type.
    pub fn lot_full(&self, owner: usize, b: usize, t: usize, n: u8) -> bool {
        let held: u32 = self.heap.all(12, owner as u8, b as u8).into_iter().filter(|&i| self.heap.recs[i][4] as usize == t).map(|i| self.heap.recs[i][5] as u32).sum();
        held + n as u32 >= 100
    }

    /// Put n transporters of type t into the auction (type 12).
    pub fn lot_transporters(&mut self, owner: usize, b: usize, t: usize, n: u8) {
        let n = n.min(self.props[b].trans(t));
        if n == 0 || !self.auction_pending() || self.lot_full(owner, b, t, n) {
            return;
        }
        // rules-24 §2: one lot per owner, branch and type; more of the same joins it at the count-weighted condition
        let old = self.heap.all(12, owner as u8, b as u8).into_iter().find(|&i| self.heap.recs[i][4] as usize == t);
        let lot = old.or_else(|| {
            let i = self.heap.alloc(12, owner as u8, b as u8, 0)?;
            self.heap.recs[i][4] = t as u8;
            Some(i)
        });
        if let Some(i) = lot {
            let r = &mut self.heap.recs[i];
            let (have, cond) = (r[5] as u32, self.props[b].cond(t) as u32);
            r[6] = if have == 0 { cond } else { (have * r[6] as u32 + n as u32 * cond) / (have + n as u32) } as u8;
            r[5] = (have + n as u32) as u8;
            let left = self.props[b].trans(t) - n;
            self.props[b].set_trans(t, left);
        }
    }

    /// Put a residence (type 14) or a castle (type 16) into the auction.
    pub fn lot_property(&mut self, owner: usize, prop: usize) {
        let t = if self.props[prop].kind() == KIND_BRANCH { 14 } else { 16 };
        if self.auction_pending() && !self.heap.has_key(t, prop as u8) && (t == 16 || self.props[prop].has_residence()) {
            // rules-35 §4: a residence lot's counter is the auction's
            let counter = if t == 14 { self.heap.find(3, NOBODY, ANY).map_or(0, |w| self.heap.recs[w][3]) } else { 0 };
            self.heap.alloc(t, owner as u8, prop as u8, counter);
        }
    }

    /// A line of the auction hall that waits for a click (rules-15 §1): `step` 0 opening, 1 lot intro, 2 sold, 3 unsold, 4 end.
    #[allow(clippy::too_many_arguments)]
    fn hall_line(&self, ch: &mut dyn Choices, step: u8, pic: u16, kind: u8, seller: u8, winner: u8, v: [i32; 4]) {
        if self.humans().next().is_some() {
            ch.ask(&Ask::AuctionLine { step, pic, kind, seller, winner, a: v[0], b: v[1], c: v[2], d: v[3] });
        }
    }

    fn may_bid(&self, p: usize, need: Need, price: i32) -> bool {
        let ok = match need {
            Need::Any => true,
            Need::Port => self.props_of(p, KIND_BRANCH).any(|b| navigable(self.props[b].loc() as usize)),
            Need::BranchIn(c) => self.branch_at(p, c).is_some_and(|b| !self.heap.has_key(13, b as u8) && !self.heap.has_key(14, b as u8)),
        };
        ok && self.persons[p].cash() >= price && !self.in_prison(p)
    }

    /// `auction_main` (0x26A50). Returns (winner, price).
    fn bid(&mut self, base: i64, need: Need, lot: u8, pic: u16, ch: &mut dyn Choices) -> Option<(usize, i32)> {
        let mut s = (base / 3) as i32;
        s -= s % increment(s);
        if s == 0 {
            s = 5;
        }
        let humans: Vec<usize> = self.humans().take(6).collect();
        let mut pressed = vec![false; humans.len()];
        let mut comp = vec![];
        for _ in 0..6 {
            let loc = if s < 1000 {
                self.r(10) as u8
            } else if s < 40000 {
                10 + self.r(3) as u8
            } else {
                10 + self.r(4) as u8
            };
            let p = self.pick_computer(Some(loc));
            let l = self.r(300) as i64 * s as i64 / 100;
            comp.push(p.map(|p| (p, l)));
        }
        let (mut price, mut high, mut hammer, mut tick) = (s, None::<usize>, 0, 0);
        // the auctioneer's strip (rules-25 §1): `f` next frame, `bank` loaded bank
        let (mut strip, mut fresh, mut blow, mut f, mut bank) = (vec![], 0usize, false, 1u8, 0u16);
        loop {
            let mut bidder = None;
            if price < 4 * s {
                let k = ch.ask(&Ask::AuctionTick { lot, price, high: high.map(|h| h as u8), pic, strip: strip.clone(), fresh: fresh as u8, blow, hammer: if blow { hammer as u8 } else { 0 } }) as usize;
                (fresh, blow) = (strip.len(), false);
                if k >= 1 && k <= humans.len() {
                    let h = humans[k - 1];
                    // the refusal reason, checks in this order [UNVERIFIED]
                    let text = if pressed[k - 1] {
                        Some(0xBAC1C)
                    } else if self.in_prison(h) {
                        Some(0xBAD48)
                    } else if !self.may_bid(h, need, i32::MIN) {
                        Some(if matches!(need, Need::Port) { 0xBACD8 } else { 0xBABB0 })
                    } else if self.persons[h].cash() < price {
                        Some(0xBAC78)
                    } else {
                        None
                    };
                    pressed[k - 1] = true;
                    match text {
                        Some(text) => _ = ch.ask(&Ask::AuctionRefused { person: h as u8, text, city: if let Need::BranchIn(c) = need { c as u8 } else { 0 } }),
                        None if high != Some(h) => bidder = Some(h),
                        None => {}
                    }
                }
                if bidder.is_none() {
                    let j = self.r(6) as usize;
                    if let Some((p, l)) = comp[j]
                        && high != Some(p) && price as i64 <= l {
                            bidder = Some(p);
                        }
                }
                // Esc (rules-39 §9): the tick's rand(6) and rand(100) are drawn, the computer bid is dropped and the
                // lot ends unsold, high bidder or not
                if k == crate::AUCTION_ESC as usize {
                    self.r(100);
                    return None;
                }
            }
            match bidder {
                Some(b) => {
                    price += increment(price);
                    high = Some(b);
                    hammer = 0;
                    let nb = 0xD5 + (b % 4) as u16;
                    if f > 1 && nb != bank {
                        let t = AUCTIONEER[(bank - 0xD5) as usize];
                        strip.push((bank, t[4], t[2]));
                        f = 1;
                    }
                    if f == 1 && self.r(100) < 20 {
                        bank = nb;
                        let t3 = AUCTIONEER[(nb - 0xD5) as usize][3];
                        strip.push((bank, 1, t3));
                        f = t3;
                    }
                }
                None if tick >= 2 => {
                    high?;
                    if self.r(100) < 70 {
                        let nb = 0xD5 + (high.unwrap_or(0) % 4) as u16;
                        let to = AUCTIONEER[(nb - 0xD5) as usize][hammer.min(2)];
                        if f < to {
                            strip.push((nb, f, to));
                            f = to;
                        }
                        bank = nb;
                        blow = true;
                        hammer += 1;
                    }
                }
                None => {}
            }
            if hammer == 3 {
                return high.map(|h| (h, price));
            }
            tick += 1;
        }
    }

    /// Year-end auction step (§1.1, §1.3): holds the auction when the record's counter is 0.
    pub fn auction_settle(&mut self, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        let Some(ai) = self.heap.find(3, NOBODY, ANY) else { return };
        if self.heap.recs[ai][3] != 0 {
            return;
        }
        // dummy sellers
        let mut dummies = vec![];
        for _ in 0..6 {
            let Some(p) = self.pick_computer(None) else { continue };
            let hs: Vec<usize> = self.humans().collect();
            let c = if hs.is_empty() {
                self.r(10) as u8
            } else {
                let h = hs[self.r(hs.len() as u32) as usize];
                self.props_of(h, KIND_BRANCH).next().map_or(0, |b| self.props[b].loc())
            };
            let b = match self.branch_at(p, c as usize) {
                Some(b) => b,
                None => match self.property_create(KIND_BRANCH, c, p as u8) {
                    Some(b) => {
                        dummies.push(b);
                        b
                    }
                    None => continue,
                },
            };
            let res = (self.r(100) << 8) | (self.r(8) << 5) | self.r(10);
            self.props[b].set_residence(res as u16);
            if self.r(100) < 30 {
                let t = 1 + self.r(5) as u8;
                let n = 2 + self.r((self.year as u32 - 1600) / 5 + 5) as u8;
                let cond = self.r(100) as u8;
                if let Some(i) = self.heap.alloc(12, p as u8, b as u8, 0) {
                    self.heap.recs[i][4..7].copy_from_slice(&[t, n, cond]);
                }
            }
            if self.r(100) < 20 {
                self.heap.alloc(14, p as u8, b as u8, 0);
            }
        }
        let lots: Vec<usize> = [12u8, 14, 16].iter().flat_map(|&t| self.heap.all(t, ANY, ANY)).collect();
        // catalogue pages (rules-15 §1.2): one question per lot, answered by a click
        if self.humans().next().is_some() {
            for (n, &i) in lots.iter().enumerate() {
                let r = self.heap.recs[i].clone();
                let b = r[2] as usize;
                let (pic, a, bb, c, d, base) = match r[0] {
                    12 => {
                        let (t, cnt, cond) = (r[4] as usize, r[5], r[6]);
                        let city = self.props[b].loc() as usize;
                        let mut base = cond as i64 * self.cities[city.min(9)].tprice(t) as i64 * cnt as i64 / 100;
                        base = if t == KARREN { base * 9 / 10 } else { base * 12 / 10 };
                        ([0x15A, 0x15C, 0x15E, 0x15B, 0x15D, 0x15F][t.min(5)], cnt as i32, t as i32, cond as i32, 0, base)
                    }
                    14 => (0x160 + (self.props[b].residence() & 0x0F), self.props[b].residence() as i32, self.props[b].loc() as i32, 0, 0, ((self.props[b].residence() & 0x0F) as i64 * 4 + 80) * self.residence_value_of(b) / 100),
                    _ => {
                        let troops: u32 = (0..4).map(|k| self.props[b].troops(k) as u32).sum();
                        (0x15C + self.props[b].loc() as u16, self.props[b].b3() as i32 / 21, self.props[b].loc() as i32, troops as i32, self.props[b].b4() as i32, self.property_value(b) * 3 / 2)
                    }
                };
                let mut price = (base / 3) as i32;
                price -= price % increment(price);
                if price == 0 {
                    price = 5;
                }
                ch.ask(&Ask::AuctionCatalog { n: n as u8, kind: r[0], pic, seller: r[1], a, b: bb, c, d, price });
            }
        }
        let human_seller = lots.iter().any(|&i| (self.heap.recs[i][1] as usize) < PERSONS && self.persons[self.heap.recs[i][1] as usize].human());
        let attend = human_seller || self.humans().collect::<Vec<_>>().into_iter().any(|h| ch.ask(&Ask::AttendAuction { person: h as u8 }) == 0);
        if attend && !lots.is_empty() {
            let city = self.r(10) as i32; // the header names a random city (rules-12 §3.5)
            self.hall_line(ch, 0, 0xC1, 0, 0, 0, [self.year as i32, city, 0, 0]);
        }
        let mut offered = 0;
        for (n, &i) in lots.iter().enumerate() {
            let r = self.heap.recs[i].clone();
            let (seller, b) = (r[1] as usize, r[2] as usize);
            if attend {
                offered += 1;
                let (pic, v) = match r[0] {
                    12 => ([0x15A, 0x15C, 0x15E, 0x15B, 0x15D, 0x15F][(r[4] as usize).min(5)], [r[5] as i32, r[4] as i32, 0, 0]),
                    14 => (0x160 + (self.props[b].residence() & 0x0F), [self.props[b].residence() as i32, 0, self.props[b].loc() as i32, 0]),
                    _ => (0x15C + self.props[b].loc() as u16, [0, 0, self.props[b].loc() as i32, 0]),
                };
                self.hall_line(ch, 1, pic, r[0], seller as u8, 0, v);
            }
            match r[0] {
                12 => {
                    let (t, cnt, cond) = (r[4] as usize, r[5], r[6]);
                    let c = self.props[b].loc() as usize;
                    let mut base = cond as i64 * self.cities[c.min(9)].tprice(t) as i64 * cnt as i64 / 100;
                    base = if t == KARREN { base * 9 / 10 } else { base * 12 / 10 };
                    let need = if needs_water(t) { Need::Port } else { Need::Any };
                    let won = if attend { self.bid(base, need, n as u8, [0x15A, 0x15C, 0x15E, 0x15B, 0x15D, 0x15F][t.min(5)], ch) } else { None };
                    match won {
                        Some((w, price)) => {
                            self.pay(w, price);
                            self.add_cash(seller, price);
                            let dest = self.props_of(w, KIND_BRANCH).find(|&x| !needs_water(t) || navigable(self.props[x].loc() as usize));
                            if let Some(d) = dest {
                                self.add_transporters(d, t, cnt, cond);
                            }
                            let loc = dest.map_or(0xFF, |d| self.props[d].loc());
                            self.hall_line(ch, 2, [0x15A, 0x15C, 0x15E, 0x15B, 0x15D, 0x15F][t.min(5)], 12, seller as u8, w as u8, [cnt as i32, t as i32, loc as i32, price]);
                            ev.push(Event::AuctionSold { lot: n as u8, winner: w as u8, price, kind: 12, loc, count: cnt, typ: t as u8, word: 0 });
                        }
                        None if !dummies.contains(&b) => {
                            if attend {
                                self.hall_line(ch, 3, 0x15A, 12, seller as u8, 0, [0; 4]);
                            }
                            self.add_transporters(b, t, cnt, cond)
                        }
                        None => {}
                    }
                }
                14 => {
                    let c = self.props[b].loc() as usize;
                    let base = ((self.props[b].residence() & 0x0F) as i64 * 4 + 80) * self.residence_value_of(b) / 100;
                    if let Some((w, price)) = if attend { self.bid(base, Need::BranchIn(c), n as u8, 0x160 + (self.props[b].residence() & 0x0F), ch) } else { None } {
                        self.pay(w, price);
                        self.add_cash(seller, price);
                        let res = self.props[b].residence();
                        self.props[b].set_residence(0xFFFF);
                        let sold = Event::AuctionSold { lot: n as u8, winner: w as u8, price, kind: 14, loc: c as u8, count: 0, typ: 0, word: res };
                        if let Some(wb) = self.branch_at(w, c) {
                            if self.persons[w].human() && self.props[wb].has_residence() {
                                // [L, reads as a bug]: paid either way, the new residence installed anyway
                                let take_new = ch.ask(&Ask::ResidenceSwap { branch: wb as u8, new_word: res }) == 0;
                                let old = self.residence_value_of(wb);
                                self.props[wb].set_residence(res);
                                let new = self.residence_value_of(wb);
                                self.add_cash(w, if take_new { old } else { new } as i32);
                            } else {
                                self.props[wb].set_residence(res);
                            }
                        }
                        self.hall_line(ch, 2, 0x160 + (res & 0x0F), 14, seller as u8, w as u8, [res as i32, 0, c as i32, price]);
                        ev.push(sold);
                    } else if attend {
                        self.hall_line(ch, 3, 0x160, 14, seller as u8, 0, [0; 4]);
                    }
                }
                _ => {
                    let base = self.property_value(b) * 3 / 2;
                    if let Some((w, price)) = if attend { self.bid(base, Need::Any, n as u8, 0x15C + self.props[b].loc() as u16, ch) } else { None } {
                        self.pay(w, price);
                        self.add_cash(seller, price);
                        self.props[b].set_owner(w as u8);
                        self.hall_line(ch, 2, 0x15C + self.props[b].loc() as u16, 16, seller as u8, w as u8, [0, 0, self.props[b].loc() as i32, price]);
                        ev.push(Event::AuctionSold { lot: n as u8, winner: w as u8, price, kind: 16, loc: self.props[b].loc(), count: 0, typ: 0, word: 0 });
                    } else if attend {
                        self.hall_line(ch, 3, 0x15C, 16, seller as u8, 0, [0; 4]);
                    }
                }
            }
            self.heap.delete(i);
        }
        if !lots.is_empty() {
            self.hall_line(ch, 4, 0xC1, 0, 0, 0, [(attend && offered > 0) as i32, 0, 0, 0]);
        }
        self.heap.delete(ai);
        for b in dummies {
            self.props[b] = Property::free();
        }
        self.globals.set_b(ga::AUCTION_PAUSE, 4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rules-12: a type-3 record with counter 1 makes the herald call at year end, any other counter does not.
    #[test]
    fn rules12_herald() {
        for (c, herald) in [(1u8, true), (3, false)] {
            let mut st = State::new_game(1, 3, &[crate::world::HumanSetup::test(3)]).unwrap();
            st.heap.alloc(3, NOBODY, NONE, c);
            let ev = st.year_end(&mut crate::Auto);
            assert_eq!(ev.iter().any(|e| matches!(e, Event::Herald { .. })), herald, "counter {c}");
        }
    }

    struct Rec(Vec<Ask>);
    impl Choices for Rec {
        fn ask(&mut self, q: &Ask) -> u32 {
            self.0.push(q.clone());
            0
        }
    }

    /// rules-25 §1: every bid or hammer tick extends a strip of valid, ordered segments; the last tick of a sold lot has seen a blow.
    #[test]
    fn rules25_s1_strip() {
        for seed in 1..40 {
            let mut st = State::new_game(seed, 3, &[crate::world::HumanSetup::test(1)]).unwrap();
            let mut rec = Rec(vec![]);
            if st.bid(3000, Need::Any, 0, 0x15A, &mut rec).is_none() {
                continue;
            }
            let ticks: Vec<&Ask> = rec.0.iter().collect();
            let Some(Ask::AuctionTick { strip, fresh, blow, .. }) = ticks.last() else { panic!() };
            assert!(*fresh as usize <= strip.len());
            assert!(strip.iter().all(|&(b, a, z)| (0xD5..=0xD8).contains(&b) && a < z && z <= 72));
            assert!(*blow || strip.is_empty() || ticks.len() > 1);
            assert!(ticks.iter().any(|t| matches!(t, Ask::AuctionTick { blow: true, .. })));
            return;
        }
        panic!("no lot sold");
    }

    /// rules-39 §9: Esc in the third tick ends the lot unsold, with the same draws as a no-bid tick before it.
    #[test]
    fn rules39_s9_esc_withdraws() {
        struct Esc(u32);
        impl Choices for Esc {
            fn ask(&mut self, q: &Ask) -> u32 {
                if matches!(q, Ask::AuctionTick { .. }) {
                    self.0 += 1;
                    if self.0 == 3 {
                        return crate::AUCTION_ESC;
                    }
                }
                0
            }
        }
        for seed in 1..20 {
            let mut st = State::new_game(seed, 3, &[crate::world::HumanSetup::test(1)]).unwrap();
            assert_eq!(st.bid(3000, Need::Any, 0, 0x15A, &mut Esc(0)), None, "seed {seed}");
        }
    }

    /// rules-24 §2: a second addition joins the lot at the weighted condition; 100 or more are refused.
    #[test]
    fn rules24_s2_lot_merge() {
        let mut st = State::new_game(1, 3, &[crate::world::HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        let b = st.props_of(h, KIND_BRANCH).next().unwrap();
        st.heap.alloc(3, NOBODY, NONE, 1);
        st.props[b].set_trans(0, 50);
        st.props[b].set_cond(0, 100);
        st.lot_transporters(h, b, 0, 10);
        st.props[b].set_cond(0, 60);
        st.lot_transporters(h, b, 0, 10);
        let lots = st.heap.all(12, h as u8, b as u8);
        assert_eq!(lots.len(), 1);
        assert_eq!((st.heap.recs[lots[0]][5], st.heap.recs[lots[0]][6]), (20, 80));
        st.props[b].set_trans(0, 99);
        st.lot_transporters(h, b, 0, 90);
        assert_eq!(st.heap.recs[lots[0]][5], 20, "20 + 90 >= 100");
    }

    /// rules-08 §1.4 increment table.
    #[test]
    fn rules08_s1_4_increments() {
        assert_eq!(increment(0), 5);
        assert_eq!(increment(99), 5);
        assert_eq!(increment(100), 10);
        assert_eq!(increment(4999), 250);
        assert_eq!(increment(5000), 500);
        assert_eq!(increment(4_999_999), 250000);
        // rules-39 §10: from 5 000 000 the scan runs through the repeated entries to the last increment
        assert_eq!(increment(5_000_000), 1_000_000);
    }

    /// An auction held with computer bidders only cleans up its records and sets the pause to 4.
    #[test]
    fn rules08_s1_3_auction_day() {
        let mut st = State::new_world(7);
        st.year = 1620;
        st.heap.alloc(3, NOBODY, NONE, 0);
        let free = (0..PROPS).filter(|&i| st.props[i].kind() == KIND_FREE).count();
        let mut ev = vec![];
        st.auction_settle(&mut crate::Auto, &mut ev);
        assert!(!st.auction_pending());
        assert_eq!(st.globals.b(ga::AUCTION_PAUSE), 4);
        assert_eq!((0..PROPS).filter(|&i| st.props[i].kind() == KIND_FREE).count(), free);
    }
}
