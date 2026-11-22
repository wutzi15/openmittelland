//! Family (docs/re/rules-05-family.md): purge, matchmaker, courtship, children, mistress, life,
//! death and inheritance.

use crate::heap::{self, ANY};
use crate::state::*;
use crate::{Ask, Choices, Event};

/// Record purge (0x42F60, §7.3, rules-40 §8). Own records: mode 1 all, mode 2 a type list, mode 3 none (the wealth
/// dismissal). Other persons' records pointing at p go in every mode; p's own never through that rule.
pub fn purge(st: &mut State, p: usize, mode: u8) {
    const MODE2: [u8; 13] = [5, 6, 7, 18, 22, 23, 24, 28, 29, 30, 31, 50, 57];
    let pb = p as u8;
    for i in 0..st.heap.recs.len() {
        if !st.heap.live(i) {
            continue;
        }
        let r = &st.heap.recs[i];
        let t = r[0];
        if r[1] == pb {
            if mode == 1 || (mode == 2 && MODE2.contains(&t)) {
                st.heap.delete(i);
            }
            continue;
        }
        let at = |o: usize| r.get(o) == Some(&pb);
        let refs = match t {
            4 if mode != 3 => at(5) || (0x0B..0x0B + 30).any(at),
            6 | 7 | 18 | 22 | 36 | 43 | 46 | 50 => at(4),
            30 if mode != 3 => at(9),
            45 => at(5),
            _ => false,
        };
        if refs {
            st.heap.delete(i);
        }
    }
}

/// Person clear (0x432D0, §7.2): every slot owned by p, any kind, passes to its own random computer principality
/// person (any computer if none), one draw per slot in slot order (rules-40 §9); name slot freed, record zeroed.
pub fn person_clear(st: &mut State, p: usize) {
    for i in 0..PROPS {
        if st.props[i].owner() as usize != p {
            continue;
        }
        let mut pool: Vec<usize> = st.computers().filter(|&x| st.persons[x].loc() == 13).collect();
        if pool.is_empty() {
            pool = st.computers().collect();
        }
        if !pool.is_empty() {
            let heir = pool[st.r(pool.len() as u32) as usize] as u8;
            st.props[i].set_owner(heir);
        }
    }
    if st.persons[p].human() {
        let n = st.persons[p].name() as usize;
        st.names[n] = [0; 24];
    }
    st.persons[p] = Person::empty();
}

impl State {
    /// §4 pregnancy scheduling for human parent p.
    pub fn schedule_child(&mut self, p: usize) {
        let m = self.heap.all(31, p as u8, ANY).len() as u32;
        let y = self.year;
        if m >= 3 || !(y < 1670 || (y < 1770 && self.opt(OPT_END_1800))) {
            return;
        }
        let c = 47 + self.r(3 * m + 3) + self.r(3);
        let sex = if self.r(100) < 66 { self.persons[p].gender() } else { 1 - self.persons[p].gender().min(1) };
        if let Some(i) = self.heap.alloc(31, p as u8, NONE, c as u8) {
            self.heap.recs[i][4] = (sex & 1) | if m == 0 { 0x80 } else { 0 };
        }
    }

    /// §4 type-31 handler, after the decrement.
    pub fn child_tick(&mut self, ri: usize, ev: &mut Vec<Event>) {
        let p = self.heap.recs[ri][1] as usize;
        let v = self.heap.recs[ri][3];
        if v == 45 {
            let Some(slot) = (0..32).find(|&n| self.names[n][0] == 0) else {
                self.heap.delete(ri);
                self.schedule_child(p);
                return;
            };
            // the name is written by the UI (Command::NameChild); reserve the slot now
            self.names[slot][0] = b'?';
            let r = &mut self.heap.recs[ri];
            r[5] = slot as u8;
            r[3] = 44;
            let sex = r[4] & 1;
            // rules-19 §2.1 step 5: the two cradle sound rolls consume game RNG
            let cry = [self.r(5) as u8, self.r(4) as u8];
            ev.push(Event::ChildBorn { parent: p as u8, slot: slot as u8, female: sex != 0, cry });
            self.history(p, 4, sex, 0);
            self.schedule_child(p);
        } else if self.persons[p].life() >= 5 && v < 45 && (v == 0 || (45 - v < 10 && self.r(100) < 3)) {
            let slot = self.heap.recs[ri][5] as usize;
            let name = self.name_str(slot);
            let female = self.heap.recs[ri][4] & 1 != 0;
            self.names[slot] = [0; 24];
            self.heap.delete(ri);
            ev.push(Event::ChildDied { parent: p as u8, age: 46 - v, name, female });
        }
    }

    /// History entry, type 57 (rules-06 §7): +4 code, +5 u16 year, +7, +8 arguments, counter 25.
    pub fn history(&mut self, p: usize, code: u8, a: u8, b: u8) {
        if let Some(i) = self.heap.alloc(57, p as u8, NONE, 25) {
            let y = self.year;
            let r = &mut self.heap.recs[i];
            r[4] = code;
            heap::w16(r, 5, y);
            r[7] = a;
            r[8] = b;
        }
    }

    /// §5 acquire mistress/lover (0x20A00).
    pub fn take_mistress(&mut self, p: usize, ev: &mut Vec<Event>) {
        if self.heap.has(29, p as u8) {
            if self.persons[p].human() {
                ev.push(Event::Refused("already has a mistress"));
            }
            return;
        }
        // rules-32 §4: the gossip roll comes before the record's counter draw
        if !self.persons[p].human() {
            let l = self.persons[p].loc() as usize;
            let human_there = self.humans().any(|h| (0..PROPS).any(|i| self.props[i].kind() != KIND_FREE && self.props[i].owner() as usize == h && self.props[i].loc() as usize == l));
            if human_there && self.r(100) < 6 {
                let text = self.r(5) as u8;
                ev.push(Event::Gossip { person: p as u8, text });
            }
        }
        let c = 2 + self.r(5) as u8;
        self.heap.alloc(29, p as u8, NONE, c);
    }

    /// §5 type-29 handler.
    pub fn mistress_tick(&mut self, ri: usize, ev: &mut Vec<Event>) {
        let k = self.heap.recs[ri][1] as usize;
        if self.heap.recs[ri][3] == 0 {
            if self.persons[k].human() {
                ev.push(Event::MistressLeft { person: k as u8 });
            }
            self.heap.delete(ri);
            return;
        }
        let demand = self.standing(k, 10);
        if self.persons[k].human() {
            // quirk [C]: shown, never deducted
            ev.push(Event::MistressDemand { person: k as u8, amount: demand });
        }
        for i in self.heap.all(29, ANY, ANY) {
            let o = self.heap.recs[i][1] as usize;
            let x = 10 + self.ri(10);
            self.favour_increase(k, o, x);
        }
    }

    /// §2 matchmaker (0x20D30). Returns the fee offer (target, fee) or None.
    /// `Err(text)`: the refusal's EXE text (0xBA95C two proposals this turn, 0x9DB01 nobody).
    pub fn matchmaker_offer(&mut self, h: usize) -> Result<(usize, i32, u8), u32> {
        if self.persons[h].spouse() != NONE || self.heap.has(30, h as u8) {
            return Err(0x9DB01);
        }
        if self.globals.b(ga::PROPOSALS) >= 2 {
            return Err(0xBA95C);
        }
        let me = &self.persons[h];
        let tier = me.loc() >= 10 && me.has_office();
        let list: Vec<usize> = (0..PERSONS)
            .filter(|&j| {
                let q = &self.persons[j];
                q.used() && q.gender() != me.gender() && q.has_office() && (q.loc() >= 10) == tier && q.flags10() == 0 && q.spouse() == NONE
            })
            .collect();
        // formats §10.6: an empty list leaves the proposal counter alone
        if list.is_empty() {
            return Err(0x9DB01);
        }
        let pv = self.globals.b(ga::PROPOSALS);
        self.globals.set_b(ga::PROPOSALS, pv + 1);
        let c = list[self.r(list.len() as u32) as usize];
        let l = self.persons[c].loc();
        let w = match l {
            13 => 0,
            10..=12 => (l - 10) * 3 + 1,
            _ => l,
        };
        let fee = self.standing(h, 40);
        Ok((c, fee, w))
    }

    /// Accepting the matchmaker's offer.
    pub fn start_courtship(&mut self, h: usize, c: usize, fee: i32, w: u8) {
        self.pay(h, fee);
        let n = 4 + self.r(3) as u8;
        if let Some(i) = self.heap.alloc(30, h as u8, NONE, n) {
            let r = &mut self.heap.recs[i];
            heap::w32(r, 4, 0);
            r[8] = n - 2;
            r[9] = c as u8;
            r[10] = w;
        }
    }

    /// §3 courtship handler (0x212B0).
    pub fn courtship_tick(&mut self, ri: usize, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        let s = self.heap.recs[ri][1] as usize;
        let c = self.heap.recs[ri][9] as usize;
        let w = self.heap.recs[ri][10] as usize;
        if self.heap.recs[ri][3] != 0 {
            let prices = [self.standing(c, 25), self.standing(c, 25), self.standing(c, 25)];
            // rules-56 §1: one word per gift, drawn gift 3 (rand 6), gift 2, gift 1 (rand 7 each)
            let (g3, g2, g1) = (self.r(6) as u8, self.r(7) as u8, self.r(7) as u8);
            let names = [g1, g2, g3];
            // rules-63 §2: the menu is printed into the turn-end report window, after the lines so far
            ch.show(ev);
            let k = ch.ask(&Ask::Gift { target: c as u8, prices, names }) as usize;
            if k < 3 {
                // rules-39 §7: the reaction word 0xB5818[rand(15)], printed before paying
                let word = self.r(15) as u8;
                if self.persons[s].human() {
                    ev.push(Event::GiftReaction { target: c as u8, word });
                }
                self.pay(s, prices[k]);
                let r = &mut self.heap.recs[ri];
                let g = heap::r32(r, 4) as i32 + prices[k];
                heap::w32(r, 4, g as u32);
            }
            return;
        }
        let gifts = heap::r32(&self.heap.recs[ri], 4) as i32;
        let mine = gifts + 20 * self.persons[c].favour(s) as i32;
        for i in self.heap.all(30, ANY, ANY) {
            let r = &self.heap.recs[i];
            let theirs = heap::r32(r, 4) as i32 + 20 * self.persons[c].favour(r[1] as usize) as i32;
            if theirs > mine {
                if r[1] as usize != s {
                    ev.push(Event::CourtshipFailed { person: s as u8, target: c as u8, reason: 0 });
                    self.heap.delete(ri);
                    return;
                }
                break;
            }
        }
        let k = self.heap.recs[ri][8] as i64;
        if (gifts as i64) < k * self.net_worth(c) as i64 * 25 / 1000 {
            ev.push(Event::CourtshipFailed { person: s as u8, target: c as u8, reason: 1 });
            self.heap.delete(ri);
            return;
        }
        if let Some(d) = self.holder(w, 9) {
            let mut used = false;
            for i in 0..PERSONS {
                if self.persons[i].used() && self.persons[d].favour(i) > 100 && ch.ask(&Ask::BlackmailCanon { by: i as u8, canon: d as u8, suitor: s as u8, target: c as u8 }) == 0 {
                    used = true;
                    break;
                }
            }
            let veto = if self.persons[d].human() && !used {
                ch.ask(&Ask::CanonVeto { canon: d as u8, suitor: s as u8, target: c as u8 }) == 0
            } else {
                self.persons[d].favour(s) <= 50 && c != d && !used
            };
            if veto {
                ev.push(Event::Vetoed { person: s as u8, target: c as u8, canon: d as u8 });
                self.heap.delete(ri);
                return;
            }
        }
        let canon = self.holder(w, 9).map(|d| d as u8);
        loop {
            match ch.ask(&Ask::Vows { suitor: s as u8, target: c as u8, canon }) {
                0 => break,
                1 => {
                    for i in 0..PERSONS {
                        if self.persons[i].used() && self.persons[i].favour(c) > 50 {
                            let x = self.ri(5);
                            self.favour_decrease(s, i, x);
                        }
                    }
                    self.heap.delete(ri);
                    return;
                }
                _ => {}
            }
        }
        self.history(s, 1, 0, 0);
        let mut t = self.persons[c].title();
        if t == 4 || t == 7 {
            t -= 1;
            self.persons[c].set_title(t);
        }
        let raised = (self.persons[s].title() < t).then_some(t);
        if raised.is_some() {
            self.persons[s].set_title(t);
        } else {
            let st = self.persons[s].title();
            self.persons[c].set_title(st);
        }
        for i in 0..PERSONS {
            let m = self.persons[i].favour(s).max(self.persons[i].favour(c));
            self.persons[i].set_favour(s, m);
            self.persons[i].set_favour(c, m);
        }
        self.persons[s].set_spouse(c as u8);
        self.persons[c].set_spouse(s as u8);
        // rules-15 §7.1 step 8: the banquet music (group 0x1D) after the title, before the fees
        let song = self.song(0x1D);
        let fees: [i32; 4] = std::array::from_fn(|o| (3 - o as i32) * self.standing(s, 20) + self.ri(100) + 2);
        let o = ch.ask(&Ask::WeddingFeast { suitor: s as u8, spouse: c as u8, city: w as u8, prices: fees, title: raised, song }) as usize;
        if o < 4 {
            self.pay(s, fees[o]);
            for i in 0..PERSONS {
                let l = self.persons[i].loc() as usize;
                let q = match o {
                    0 => true,
                    1 => l as i32 == (w as i32 - 1) / 3, // bug [C]: C division, compares with 0..2
                    2 => l == w,
                    _ => false,
                };
                if q {
                    let x = self.ri(5);
                    self.favour_increase(s, i, x);
                    let y = self.ri(5);
                    self.favour_increase(c, i, y);
                }
            }
        }
        self.schedule_child(s);
        for i in self.heap.all(30, ANY, ANY) {
            let tg = self.heap.recs[i][9] as usize;
            if tg == c || tg == s {
                self.heap.delete(i);
            }
        }
        ev.push(Event::Wedding { person: s as u8, spouse: c as u8, title: raised, feast: o < 4 });
    }

    /// §6 life decrement and §7 death/inheritance at the end of a human's turn. Returns true when
    /// the player left the game.
    pub fn life_and_death(&mut self, h: usize, ch: &mut dyn Choices, ev: &mut Vec<Event>) -> bool {
        let _ = ch;
        let d = if self.year < 1620 { 1 } else { self.r(3) as u8 };
        let life = self.persons[h].life().wrapping_sub(d);
        self.persons[h].set_life(life);
        if life as i8 > 0 {
            return false;
        }
        // epitaph: up to 16 tries of rand(8) for a set deed bit (rules-15 §7.4); 8 = none found
        let deeds = self.persons[h].deeds();
        let mut epitaph = 8u8;
        for _ in 0..16 {
            let bit = self.r(8) as u8;
            if deeds & (1 << bit) != 0 {
                epitaph = bit;
                break;
            }
        }
        // rules-17 §4.3: mourners' heckles after notary steps 1..4 and, unless the only child inherits or everything
        // goes to the archbishopric, after the testament. Draws per line: alignment, line, voice, gender.
        let len = self.heap.all(31, h as u8, ANY).into_iter().filter(|&i| self.heap.recs[i][3] < 45).count();
        let heir_at = self.heap.all(31, h as u8, ANY).into_iter().filter(|&i| self.heap.recs[i][3] < 45).position(|i| self.heap.recs[i][4] & 0x80 != 0);
        let h_idx = heir_at.unwrap_or(len);
        let mut heckles = vec![];
        for step in 1..=5u8 {
            if step == 5 && (h_idx >= len || (len == 1 && h_idx == 0)) {
                break;
            }
            let k = if len == 0 { 1 } else { 1 + self.r(len as u32) };
            for _ in 0..k {
                let (a, line) = (self.r(3) as u8, self.r(if step < 5 { 14 } else { 5 }) as u8);
                let voice = [15, 16, 17, 1][self.r(4) as usize];
                heckles.push([step, a, line, voice, self.r(2) as u8]);
            }
        }
        let nw = self.net_worth(h);
        let name = self.name_str(self.persons[h].name() as usize);
        ev.push(Event::Died { person: h as u8, name, female: self.persons[h].gender() == 1, nw, epitaph, heckles });
        let kids: Vec<usize> = self.heap.all(31, h as u8, ANY).into_iter().filter(|&i| self.heap.recs[i][3] < 45).collect();
        let heir = kids.iter().position(|&i| self.heap.recs[i][4] & 0x80 != 0);
        let Some(hi) = heir else {
            let name = self.name_str(self.persons[h].name() as usize);
            ev.push(Event::PlayerOut { person: h as u8, name, no_kids: kids.is_empty() });
            purge(self, h, 1);
            person_clear(self, h);
            return true;
        };
        let r = self.heap.recs[kids[hi]].clone();
        let nw = self.net_worth(h);
        let old = self.persons[h].name() as usize;
        self.names[old] = [0; 24];
        let p = &mut self.persons[h];
        p.set_name(r[5]);
        p.set_gender(r[4] & 1);
        for g in 0..GOODS {
            p.set_skill(g, ((p.skill(g) as u32 + 100) / 2) as u8);
        }
        p.set_spouse(NONE);
        p.set_life(r[3]);
        p.set_insurance(0);
        if kids.len() >= 2 {
            self.pay(h, nw / 5);
        }
        purge(self, h, 2);
        ev.push(Event::Succession { person: h as u8, kids: kids.len() as u8 });
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::HumanSetup;

    /// rules-05 §4: first child counter 47 + rand(3) + rand(3) is born at counter 45, main heir bit set.
    #[test]
    fn rules05_s4_child_scheduling() {
        let mut st = State::new_game(4, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        st.schedule_child(h);
        let i = st.heap.find(31, h as u8, ANY).unwrap();
        let c = st.heap.recs[i][3];
        assert!((47..=51).contains(&c));
        assert_eq!(st.heap.recs[i][4] & 0x80, 0x80);
        st.year = 1670;
        st.options &= !OPT_END_1800;
        st.schedule_child(h);
        assert_eq!(st.heap.all(31, h as u8, ANY).len(), 1, "no births from 1670 without the option");
    }

    /// rules-05 §7: death without children ends the player; with an heir the slot continues.
    #[test]
    fn rules05_s7_death_and_succession() {
        let mut st = State::new_game(4, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        let i = st.heap.alloc(31, h as u8, NONE, 30).unwrap();
        st.heap.recs[i][4] = 0x81;
        st.heap.recs[i][5] = 5;
        st.persons[h].set_life(1);
        st.persons[h].set_skill(0, 150);
        if let Some(x) = st.holder(3, 6) {
            st.office_vacate(x);
        }
        st.appoint(h, 3, 6);
        assert_eq!(st.holder(3, 6), Some(h));
        let mut ev = vec![];
        assert!(!st.life_and_death(h, &mut crate::Auto, &mut ev));
        assert_eq!(st.holder(3, 6), Some(h), "the heir keeps the office (§7 step 8)");
        assert_eq!(st.persons[h].life(), 30);
        assert_eq!(st.persons[h].gender(), 1);
        assert_eq!(st.persons[h].skill(0), 125);
        assert!(!st.heap.has(31, h as u8));
        st.heap.compact();
        st.persons[h].set_life(1);
        assert!(st.life_and_death(h, &mut crate::Auto, &mut ev));
        assert!(!st.persons[h].used());
    }

    /// rules-05 §7.3 purge mode 2 keeps loans (type 56) and drops spy contracts on the person.
    #[test]
    fn rules05_s7_3_purge() {
        let mut st = State::blank(1);
        st.heap.alloc(56, 3, 0, 5);
        st.heap.alloc(29, 3, 0, 5);
        let s = st.heap.alloc(6, 9, 0, 5).unwrap();
        st.heap.recs[s][4] = 3;
        purge(&mut st, 3, 2);
        st.heap.compact();
        assert_eq!(st.heap.recs.len(), 1);
        assert_eq!(st.heap.recs[0][0], 56);
    }

    /// rules-56 §1: three prices, then the gift words drawn gift 3 (rand 6), gift 2, gift 1 (rand 7).
    #[test]
    fn rules56_gift_word_order() {
        struct Rec(Option<Ask>);
        impl Choices for Rec {
            fn ask(&mut self, q: &Ask) -> u32 {
                self.0.get_or_insert(q.clone());
                3
            }
        }
        let mut st = State::new_game(4, 3, &[HumanSetup::test(1)]).unwrap();
        let h = st.current as usize;
        let (c, fee, w) = st.matchmaker_offer(h).unwrap();
        st.start_courtship(h, c, fee, w);
        let i = st.heap.find(30, h as u8, ANY).unwrap();
        st.heap.recs[i][3] = 1;
        let mut ex = st.clone();
        let prices = [ex.standing(c, 25), ex.standing(c, 25), ex.standing(c, 25)];
        let (g3, g2, g1) = (ex.r(6) as u8, ex.r(7) as u8, ex.r(7) as u8);
        let mut rec = Rec(None);
        st.courtship_tick(i, &mut rec, &mut vec![]);
        assert_eq!(rec.0, Some(Ask::Gift { target: c as u8, prices, names: [g1, g2, g3] }));
    }
}
