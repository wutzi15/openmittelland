//! Kontor actions (docs/re/rules-11-kontor-actions.md): branch rights, residences, local sales,
//! money lender and loan book, church entry, giving up, directors.

use crate::economy::*;
use crate::heap::{self, ANY};
use crate::state::*;
use crate::{Ask, Choices, Event};

/// Build time in years by residence level (`0x866C0`, §2).
const BUILD_TIME: [u8; 10] = [1, 1, 1, 1, 2, 2, 2, 3, 4, 4];
/// Start-city sequence (`0x870C2`, §1).
const START: [u8; 12] = [3, 5, 8, 9, 3, 5, 8, 9, 2, 6, 2, 6];
/// Residence buy offers, three u16 words per city (`0x85CC5 + 6c`).
const RES_OFFERS: usize = 0x85CC5;
/// Money lender mood (`0x85D45`).
const LENDER_MOOD: usize = 0x85D45;

impl State {
    /// Rights holder of a location (`0x85E08`, §1): the last human (index order) owning any
    /// property there. Computed on demand instead of cached.
    pub fn rights_holder(&self, loc: usize) -> Option<usize> {
        self.humans()
            .filter(|&h| (0..PROPS).any(|p| self.props[p].kind() != KIND_FREE && self.props[p].owner() as usize == h && self.props[p].loc() as usize == loc))
            .last()
    }

    /// §1 start city of a new human (setup 0x4DBC0).
    pub fn pick_start_city(&mut self) -> Option<u8> {
        let mut i = self.r(4) as usize;
        while i < 12 {
            if i == 8 {
                i += self.r(2) as usize;
            }
            let c = START[i];
            i += 1;
            if self.rights_holder(c as usize).is_none() {
                return Some(c);
            }
        }
        None
    }

    /// §1 price of the branch rights in `city`: the value of a fresh branch there.
    pub fn branch_price(&self, city: usize) -> i32 {
        let c = &self.cities[city];
        (c.rights() as i64 + 15 * c.level_a() as i64 + c.tprice(KARREN) as i64 + c.population() as i64) as i32
    }

    /// §1 buy the branch rights of `city`.
    pub fn buy_branch(&mut self, h: usize, city: usize) -> Option<Event> {
        if city >= CITIES || self.branch_at(h, city).is_some() || self.rights_holder(city).is_some() {
            return Some(Event::Refused("branch rights taken"));
        }
        let price = self.branch_price(city);
        // quirk [C]: unsigned comparison, a player in debt passes
        if (self.persons[h].cash() as u32) < price as u32 {
            return Some(Event::Refused("not enough cash"));
        }
        let b = self.property_create(KIND_BRANCH, city as u8, h as u8)?;
        self.pay(h, price);
        Some(Event::BranchBought { prop: b as u8, price })
    }

    /// §1 sell a branch for 2/3 of its value; needs another branch.
    pub fn sell_branch(&mut self, h: usize, b: usize) -> Option<Event> {
        if self.branches(h) < 2 {
            return Some(Event::Refused("last branch"));
        }
        let v = (self.property_value(b) * 2 / 3) as i32;
        self.add_cash(h, v);
        self.free_property(b);
        Some(Event::BranchSold { prop: b as u8, price: v })
    }

    /// 0x429A0: free the slot and delete every heap record keyed (byte 2) to it. Only kind and owner are written;
    /// the rest stays until `property_create` zero-fills the slot (rules-68 §1).
    pub fn free_property(&mut self, b: usize) {
        self.props[b].set_kind(KIND_FREE);
        self.props[b].set_owner(NONE);
        for i in 0..self.heap.recs.len() {
            if self.heap.live(i) && self.heap.recs[i][2] as usize == b {
                self.heap.delete(i);
            }
        }
    }

    /// §3 local sale: `qty` units of g, or (good None) everything.
    pub fn sell_local(&mut self, h: usize, b: usize, good: Option<usize>, qty: u16) -> i32 {
        let c = self.props[b].loc() as usize;
        if c >= CITIES {
            return 0;
        }
        let bonus = if self.heap.all(42, h as u8, ANY).into_iter().any(|i| self.heap.recs[i][3] < 7) { 2 } else { 0 };
        let goods: Vec<usize> = match good {
            Some(g) if g < GOODS => vec![g],
            Some(_) => vec![],
            None => (0..GOODS).collect(),
        };
        let mut total = 0;
        for g in goods {
            let q = if good.is_none() { self.props[b].stock(g) } else { qty.min(self.props[b].stock(g)) };
            if q == 0 {
                continue;
            }
            let rev = q as u32 * (self.cities[c].price(g) as u32 + bonus);
            let left = self.props[b].stock(g) - q;
            self.props[b].set_stock(g, left);
            let s = self.globals.sales(g, c).wrapping_add(rev);
            self.globals.set_sales(g, c, s);
            let sold = self.cities[c].sold(g) + q as u32;
            self.cities[c].set_sold(g, sold);
            total += rev as i32;
        }
        self.add_cash(h, total);
        total
    }

    // ---- §2 residences ----

    pub fn res_value(&self, b: usize, word: u16) -> i64 {
        self.residence_value_word(self.props[b].loc() as usize, word)
    }

    fn score(word: u16) -> i32 {
        if word == 0xFFFF {
            return 0;
        }
        (word & 0x0F) as i32 + (word & 0x20 != 0) as i32 + (word & 0x40 != 0) as i32 + (word & 0x80 != 0) as i32
    }

    /// `set_residence` (0x520A0): favour change in city, county, principality, then the word.
    pub fn set_residence(&mut self, b: usize, new: u16) {
        let h = self.props[b].owner() as usize;
        let mut d = (Self::score(new) - Self::score(self.props[b].residence())) * 3;
        let c = self.props[b].loc() as usize;
        for loc in [c, COUNTY[c.min(9)] as usize, 13] {
            // every person whose place byte equals the location (office-less persons have place 0)
            for x in 0..PERSONS {
                if h < PERSONS && self.persons[x].used() && self.persons[x].loc() as usize == loc {
                    if d < 0 { self.favour_decrease(h, x, -d) } else { self.favour_increase(h, x, d) }
                }
            }
            d /= 3;
        }
        self.props[b].set_residence(new);
    }

    fn res_busy(&self, b: usize) -> bool {
        self.heap.has_key(13, b as u8) || self.heap.has_key(14, b as u8)
    }

    fn credit_old(&mut self, h: usize, b: usize) {
        if self.props[b].has_residence() {
            let v = (self.residence_value_of(b) * 2 / 3) as i32;
            self.add_cash(h, v);
        }
    }

    /// Turn start: residence offer k of city c (0x4B900, rules-37 §1 #19).
    pub fn roll_residence_offer(&mut self, c: usize, k: usize) {
        let w = (self.r(100) << 8) | (self.r(8) << 5) | self.r(10);
        wr16(&mut self.globals.0, G(RES_OFFERS) + 6 * c + 2 * k, w as u16);
    }

    pub fn residence_offer(&self, city: usize, k: usize) -> u16 {
        rd16(&self.globals.0, G(RES_OFFERS) + 6 * city + 2 * k)
    }

    /// §2 buy offer k outright (immediate; the offer stays [quirk]).
    pub fn buy_residence(&mut self, h: usize, b: usize, k: usize) -> Option<Event> {
        if self.res_busy(b) || k > 2 {
            return Some(Event::Refused("construction pending"));
        }
        let w = self.residence_offer(self.props[b].loc() as usize, k);
        self.credit_old(h, b);
        let cost = (self.res_value(b, w) * 11 / 10) as i32;
        self.pay(h, cost);
        self.set_residence(b, w);
        None
    }

    fn order_residence(&mut self, h: usize, b: usize, kind: u8, word: u16, years: u8, cost: i64) -> Option<Event> {
        let i = self.heap.alloc(13, h as u8, b as u8, years)?;
        heap::w16(&mut self.heap.recs[i], 4, word);
        self.pay(h, cost as i32);
        Some(Event::ResidenceOrdered { branch: b as u8, cost: cost as i32, kind, word, years })
    }

    /// §2 build level k (condition 80).
    pub fn build_residence(&mut self, h: usize, b: usize, level: u8) -> Option<Event> {
        if self.res_busy(b) || level > 9 {
            return Some(Event::Refused("cannot build"));
        }
        let w = 0x5000 | level as u16;
        self.credit_old(h, b);
        let cost = self.res_value(b, w);
        self.order_residence(h, b, 0, w, BUILD_TIME[level as usize], cost)
    }

    /// §2 sell for V/3.
    pub fn sell_residence(&mut self, h: usize, b: usize) -> i32 {
        if !self.props[b].has_residence() || self.res_busy(b) {
            return 0;
        }
        let v = (self.residence_value_of(b) / 3) as i32;
        self.add_cash(h, v);
        self.set_residence(b, 0xFFFF);
        v
    }

    /// §2 extensions: `bits` = the wanted set of 0x20 / 0x40 / 0x80.
    pub fn extend_residence(&mut self, h: usize, b: usize, bits: u8) -> Option<Event> {
        let r = self.props[b].residence();
        if !self.props[b].has_residence() || self.res_busy(b) || r & 0xE0 == 0xE0 {
            return Some(Event::Refused("cannot extend"));
        }
        let new = (r & !0xE0) | (bits as u16 & 0xE0);
        if new == r {
            return None;
        }
        let cost = self.res_value(b, new) - self.residence_value_of(b);
        self.order_residence(h, b, 1, new, 1, cost)
    }

    /// §2 renovate to condition 100.
    pub fn renovate_residence(&mut self, h: usize, b: usize) -> Option<Event> {
        let r = self.props[b].residence();
        if !self.props[b].has_residence() || self.res_busy(b) {
            return Some(Event::Refused("cannot renovate"));
        }
        let target = (r & 0xFF) | 0x6400;
        let cost = (self.res_value(b, target) - self.residence_value_of(b)).max(100);
        let years = 1 + ((r >> 8) < 25) as u8;
        self.order_residence(h, b, 2, target, years, cost)
    }

    /// Turn start delivery of type-13 records with counter 0 or bit 7 set.
    pub fn deliver_residences(&mut self, h: usize) {
        for i in self.heap.all(13, h as u8, ANY) {
            let c = self.heap.recs[i][3];
            if c != 0 && c & 0x80 == 0 {
                continue;
            }
            let b = self.heap.recs[i][2] as usize;
            if self.props[b].kind() == KIND_BRANCH && self.props[b].owner() as usize == h {
                let w = heap::r16(&self.heap.recs[i], 4);
                self.set_residence(b, w);
            }
            self.heap.delete(i);
        }
    }

    // ---- §4 money lender ----

    /// Turn start: lender mood.
    pub fn roll_lender_mood(&mut self) {
        let m = 50 + self.r(20) as u8;
        self.globals.set_b(LENDER_MOOD, m);
    }

    /// §4 one inquiry at the money lender; the offer is accepted with answer 0.
    pub fn ask_lender(&mut self, h: usize, ch: &mut dyn Choices) -> Option<Event> {
        let m = self.globals.b(LENDER_MOOD);
        if m > 100 {
            return Some(Event::Notice { text: 0xC0438 });
        }
        let n = self.heap.all(56, h as u8, ANY).len();
        // the original crashes without a branch [bug]; refused here
        let x = self.pick_property(KIND_BRANCH, h)?;
        let city = self.props[x].loc() as usize;
        let pl = self.persons[h].loc();
        let pool = if pl < 10 { city as u8 } else if pl < 13 { COUNTY[city] } else { 13 };
        let lender = self.pick_computer(Some(pool));
        if n >= 4 {
            return Some(Event::Notice { text: 0xC03F4 });
        }
        let Some(lender) = lender else { return Some(Event::Notice { text: 0xC0498 }) };
        let k = 30 + self.ri(50);
        let amount = self.standing(h, k).max(100);
        let years = 3 + self.r(8) as u8;
        let maxrate = self.globals.b(ga::MAX_INTEREST + COUNTY[city] as usize - 10) as i32;
        let rate = (maxrate * m as i32 / 100 + years as i32 / 2).min(maxrate).max(1) as u8;
        if ch.ask(&Ask::LoanOffer { lender: lender as u8, amount, rate, years }) == 0
            && let Some(i) = self.heap.alloc(56, h as u8, NONE, years)
        {
            let r = &mut self.heap.recs[i];
            r[4] = lender as u8;
            heap::w32(r, 5, amount as u32);
            r[9] = rate;
            self.add_cash(h, amount);
        }
        let mut nm = (m as i32 - 5 + self.ri(25)) as u8;
        if nm > 200 {
            nm = 1;
        }
        self.globals.set_b(LENDER_MOOD, nm);
        None
    }

    /// §4 loan book: repay a loan (no interest refund).
    pub fn repay_loan(&mut self, i: usize) {
        let (h, amount) = (self.heap.recs[i][1] as usize, heap::r32(&self.heap.recs[i], 5) as i32);
        self.pay(h, amount);
        self.heap.delete(i);
    }

    /// §4 type-56 expiry (counter 0 after the countdown).
    pub fn loan_expiry(&mut self, i: usize, ev: &mut Vec<Event>) {
        let r = self.heap.recs[i].clone();
        let (h, lender, amount) = (r[1] as usize, r[4] as usize, heap::r32(&r, 5) as i32);
        if lender < PERSONS {
            self.favour_decrease(h, lender, 8);
        }
        self.pay(h, amount);
        if self.r(100) < 10 && lender < PERSONS {
            self.retaliate(lender, h, 2);
        }
        self.heap.delete(i);
        ev.push(Event::LoanDue { lender: lender as u8, amount });
    }

    // ---- §5, §6 ----

    /// §5 Kircheneintritt: free.
    pub fn join_church(&mut self, h: usize, religion: u8) {
        if self.persons[h].religion() == 2 && religion < 2 {
            self.persons[h].set_religion(religion);
        }
    }

    /// §6 giving up: purge mode 1, person clear (every property to a computer person), phase 6.
    pub fn give_up(&mut self, h: usize) {
        crate::family::purge(self, h, 1);
        crate::family::person_clear(self, h);
        self.phase = 6;
    }

    // ---- §7 directors ----

    /// §7 hire: up to three distinct candidates, the player picks one (or none).
    pub fn hire_director(&mut self, h: usize, b: usize, ch: &mut dyn Choices) {
        let pl = self.persons[h].loc();
        let place = if pl < 10 { self.props[b].loc() } else { pl };
        let mut cands: Vec<u8> = vec![];
        for _ in 0..3 {
            let pool: Vec<usize> = self.computers().filter(|&x| self.persons[x].loc() == place && !self.persons[x].church()).collect();
            if pool.is_empty() {
                break;
            }
            let x = pool[self.r(pool.len() as u32) as usize] as u8;
            if !cands.contains(&x) {
                cands.push(x);
            }
        }
        if cands.is_empty() {
            return;
        }
        let fees: Vec<u8> = cands.iter().map(|&c| self.director_fee_pct(c as usize) as u8).collect();
        let k = ch.ask(&Ask::PickDirector { candidates: cands.clone(), fees }) as usize;
        if let Some(&d) = cands.get(k) {
            self.props[b].set_director(d);
            self.props[b].set_budget(1000);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::HumanSetup;

    /// rules-11 §1: price = rights + 15A + Karren price + population; computer branches do not block.
    #[test]
    fn rules11_s1_branch_rights() {
        let mut st = State::new_game(1, 3, &[HumanSetup::test(2)]).unwrap();
        let h = st.current as usize;
        let c = &st.cities[5];
        let p = c.rights() as i32 + 15 * c.level_a() as i32 + c.tprice(0) as i32 + c.population() as i32;
        assert_eq!(st.branch_price(5), p);
        st.add_cash(h, p);
        let cash = st.persons[h].cash();
        assert!(matches!(st.buy_branch(h, 5), Some(Event::BranchBought { .. })));
        assert_eq!(st.persons[h].cash(), cash - p);
        let b = st.branch_at(h, 5).unwrap();
        assert!(matches!(st.sell_branch(h, b), Some(Event::BranchSold { .. })));
        let last = st.branch_at(h, 2).unwrap();
        assert_eq!(st.sell_branch(h, last), Some(Event::Refused("last branch")));
        // quirk: negative cash passes the unsigned check
        st.persons[h].set_cash_raw(-5);
        assert!(matches!(st.buy_branch(h, 7), Some(Event::BranchBought { .. })));
    }

    /// rules-11 §1 start cities: the first four players get ports, then Moortal / Falkenhorst.
    #[test]
    fn rules11_s1_start_cities() {
        let mut st = State::new_world(3);
        let mut got = vec![];
        for _ in 0..6 {
            let c = st.pick_start_city().unwrap();
            st.setup_human(&HumanSetup::test(c)).unwrap();
            got.push(c);
        }
        let mut first: Vec<u8> = got[..4].to_vec();
        first.sort();
        assert_eq!(first, vec![3, 5, 8, 9]);
        let mut rest: Vec<u8> = got[4..].to_vec();
        rest.sort();
        assert_eq!(rest, vec![2, 6]);
    }

    /// rules-11 §2 build: pays V(level, condition 80), delivered after BT[k] turn ends.
    #[test]
    fn rules11_s2_build_and_deliver() {
        let mut st = State::new_game(1, 3, &[HumanSetup::test(2)]).unwrap();
        let h = st.current as usize;
        let b = st.branch_at(h, 2).unwrap();
        let cost = st.residence_value_word(2, 0x5004);
        let cash = st.persons[h].cash();
        st.build_residence(h, b, 4);
        assert_eq!(st.persons[h].cash() as i64, cash as i64 - cost);
        let i = st.heap.find(13, h as u8, b as u8).unwrap();
        assert_eq!(st.heap.recs[i][3], 2);
        st.heap.recs[i][3] = 0;
        st.deliver_residences(h);
        assert_eq!(st.props[b].residence(), 0x5004);
        assert_eq!(st.sell_residence(h, b) as i64, st.residence_value_word(2, 0x5004) / 3);
    }

    /// rules-11 §3 and §4: local sale, lender offer, repayment.
    #[test]
    fn rules11_s3_s4_sale_and_loans() {
        let mut st = State::new_game(1, 3, &[HumanSetup::test(2)]).unwrap();
        let h = st.current as usize;
        let b = st.branch_at(h, 2).unwrap();
        st.props[b].set_stock(0, 10);
        assert_eq!(st.sell_local(h, b, Some(0), 4), 4 * st.cities[2].price(0) as i32);
        assert_eq!(st.cities[2].sold(0), 4);
        st.roll_lender_mood();
        let cash = st.persons[h].cash();
        st.ask_lender(h, &mut crate::Auto);
        let i = st.heap.find(56, h as u8, ANY).unwrap();
        let (amt, rate, years) = (heap::r32(&st.heap.recs[i], 5) as i32, st.heap.recs[i][9], st.heap.recs[i][3]);
        assert!(amt >= 100 && (3..=10).contains(&years) && (1..=35).contains(&rate));
        assert_eq!(st.persons[h].cash(), cash + amt);
        st.repay_loan(i);
        assert_eq!(st.persons[h].cash(), cash);
    }
}
