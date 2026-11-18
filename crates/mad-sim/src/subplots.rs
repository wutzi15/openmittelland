//! Sub-plots ("Abenteuer", record types 32..49) and newspaper contests (record type 4)
//! (docs/re/rules-06-events.md §5, §6).
//!
//! Dialog answers come from `Ask::Plot { kind, step }`: for yes/no dialogs 0 = accept (the first
//! line), 1 = refuse; for spinners the answer is the amount; for menus the line index. Steps per
//! kind are numbered in the order the dialogs appear in the doc. Cosmetic random draws (names,
//! taverns, texts) are kept so the RNG sequence matches.

use crate::economy::*;
use crate::heap::{self, ANY};
use crate::state::*;
use crate::{Ask, Choices, Event};

fn ask(ch: &mut dyn Choices, kind: u8, step: u8) -> u32 {
    ch.ask(&Ask::Plot { kind, step, arg: 0, arg2: 0 })
}
fn ask_arg(ch: &mut dyn Choices, kind: u8, step: u8, arg: i32) -> u32 {
    ch.ask(&Ask::Plot { kind, step, arg, arg2: 0 })
}
fn ask_args(ch: &mut dyn Choices, kind: u8, step: u8, arg: i32, arg2: i32) -> u32 {
    ch.ask(&Ask::Plot { kind, step, arg, arg2 })
}
fn yes(ch: &mut dyn Choices, kind: u8, step: u8) -> bool {
    ask(ch, kind, step) == 0
}
/// A sub-plot text with values (rules-13 §2); the ids per kind are listed in the app's `describe`.
fn note(ev: &mut Vec<Event>, kind: u8, id: u8, a: i32, b: i32, c: i32) {
    ev.push(Event::PlotNote { kind, id, a, b, c });
}

impl State {
    /// `person_pick_random(loc, clergy, any, computer)`: uniform among computer persons at `loc`
    /// (None = anywhere) with the clergy flag `clergy` (None = either). None if nobody qualifies.
    fn pick_cp(&mut self, loc: Option<u8>, clergy: Option<bool>) -> Option<usize> {
        let pool: Vec<usize> = self
            .computers()
            .filter(|&x| loc.is_none_or(|l| self.persons[x].loc() == l) && clergy.is_none_or(|c| self.persons[x].church() == c))
            .collect();
        if pool.is_empty() {
            return None;
        }
        Some(pool[self.r(pool.len() as u32) as usize])
    }

    fn at_loc(&self, l: u8) -> Vec<usize> {
        (0..PERSONS).filter(|&p| self.persons[p].used() && self.persons[p].loc() == l).collect()
    }

    fn rec(&mut self, t: u8, h: usize, key: u8, counter: u8) -> Option<usize> {
        self.heap.alloc(t, h as u8, key, counter)
    }

    /// §5.1 starter k = 0..16 for human h.
    pub fn subplot_start(&mut self, h: usize, k: u32, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        // the dialogs print into the turn-end window after the lines so far (rules-13)
        ch.show(ev);
        let hb = h as u8;
        let t = 32 + k as u8 + (k >= 3) as u8; // 0..2 → 32..34, 3.. → 36..49 (35 unused)
        if t != 37 && self.heap.has(t, hb) {
            return;
        }
        if t == 46 && self.heap.find(46, ANY, ANY).is_some() {
            return;
        }
        let started = match t {
            32 => {
                let tavern = self.r(30) as i32;
                // EXE flow (rules-13 §2.1): dialog 1 has three lines, line 1 skips dialog 2, line 2 leaves
                let a0 = ask_arg(ch, t, 0, tavern);
                // the refusals: 0xA468F (1), 0xA46FC (2), 0xA5332 (3)
                let end = if a0 == 2 {
                    1
                } else if a0 != 1 && !yes(ch, t, 1) {
                    2
                } else if !yes(ch, t, 2) {
                    1
                } else if !yes(ch, t, 3) {
                    3
                } else {
                    0
                };
                if end != 0 {
                    return note(ev, 32, end, 0, 0, 0);
                }
                self.pay(h, 1000);
                self.rec(32, h, NONE, 4).is_some()
            }
            33 => {
                let mut v = [0i32; 4]; // name, profession, dress, season
                for (k, m) in [117, 32, 2, 4].into_iter().enumerate() {
                    v[k] = self.r(m) as i32;
                }
                if ask_arg(ch, t, 0, v[0] | v[1] << 7 | v[2] << 12 | v[3] << 13) != 0 {
                    return note(ev, 33, 3, 0, 0, 0);
                }
                let b = self.r(49) as u8;
                let price = self.fy(500);
                let a = self.r(11) as u8;
                let place = self.r(2) as i32;
                if ask_args(ch, t, 1, price, b as i32 | (a as i32) << 6 | place << 10 | v[0] << 11) != 0 {
                    return note(ev, 33, 4, 0, 0, 0);
                }
                // the shown price is paid, no second roll (rules-40 §17); 0x9CEBF names the profession
                self.pay(h, price);
                note(ev, 33, 5, v[1], 0, 0);
                let c = 5 + self.r(3) as u8;
                match self.rec(33, h, NONE, c) {
                    Some(i) => {
                        self.heap.recs[i][4] = b;
                        self.heap.recs[i][5] = a;
                        heap::w16(&mut self.heap.recs[i], 7, 0xFFFF);
                        true
                    }
                    None => false,
                }
            }
            34 => {
                if self.persons[h].certificate() <= 1 {
                    return;
                }
                let l = 10 + self.r(3) as u8;
                // quirk [C]: the code tests != 0, so "none" passes and person 0 fails
                let x = self.pick_cp(Some(l), Some(false));
                if x == Some(0) {
                    return;
                }
                let dest = self.r(7) as u8;
                let city = self.r(10) as i32;
                match self.rec(34, h, NONE, 5) {
                    Some(i) => {
                        self.heap.recs[i][4] = dest;
                        // the notice 0xA619E: city, the person (0xFF none), destination
                        note(ev, 34, 4, city, x.map_or(NONE as i32, |x| x as i32), dest as i32);
                        true
                    }
                    None => false,
                }
            }
            36 => {
                let l = 10 + self.r(4) as u8;
                let Some(x) = self.pick_cp(Some(l), None) else { return };
                // the raw office byte +0x11, 15 without office (rules-40 §18)
                let r = self.persons[x].idx() as i32 + self.ri(3) + 2 + self.persons[x].title() as i32;
                let q = 5 - self.ri(6) * r / 30;
                let place = self.r(10) as i32;
                // host | R << 8 | (quality + 16) << 16 | place << 22
                if ask_arg(ch, t, 0, x as i32 | (r & 255) << 8 | (q + 16) << 16 | place << 22) != 0 {
                    return note(ev, 36, 4, 0, 0, 0);
                }
                note(ev, 36, 3, 0, 0, 0);
                match self.rec(36, h, NONE, 2) {
                    Some(i) => {
                        let rr = &mut self.heap.recs[i];
                        rr[4] = x as u8;
                        rr[5] = r as u8;
                        rr[6] = q as i8 as u8;
                        true
                    }
                    None => false,
                }
            }
            37 => {
                let tav = self.r(30) as u8;
                let mut look = [0i32; 3]; // shoes, hair, drink
                for (k, m) in [8, 7, 5].into_iter().enumerate() {
                    look[k] = self.r(m) as i32;
                }
                let Some(i) = self.rec(37, h, NONE, 1) else { return };
                let w = self.r(3) as u8;
                self.heap.recs[i][4] = w;
                match w {
                    0 => {
                        let c = self.r(10) as u8;
                        let g = self.r(15) as u8;
                        let p = self.cities[c as usize].price(g as usize);
                        self.heap.recs[i][5..8].copy_from_slice(&[c, g, p]);
                    }
                    1 => self.heap.recs[i][6] = tav,
                    _ => {
                        self.heap.recs[i][5] = self.r(10) as u8;
                        self.heap.recs[i][6] = self.r(8) as u8;
                    }
                }
                let stake = self.standing(h, 50) as u16;
                heap::w16(&mut self.heap.recs[i], 8, stake);
                let greeting = self.r(4) as i32;
                let rc = &self.heap.recs[i];
                // tavern | drink | hair | shoes | greeting | kind | byte +5 | byte +6
                let packed = tav as i32 | look[2] << 5 | look[1] << 8 | look[0] << 11 | greeting << 14 | (w as i32) << 16 | (rc[5] as i32 & 15) << 18 | (rc[6] as i32 & 31) << 22;
                if ask_args(ch, t, 0, stake as i32, packed) != 0 {
                    self.heap.delete(i);
                    return note(ev, 37, 4, 0, 0, 0);
                }
                note(ev, 37, 3, 0, 0, 0);
                true
            }
            38 => {
                let pool: Vec<usize> = self.computers().filter(|&x| self.persons[x].gender() == 0).collect();
                if pool.is_empty() {
                    return;
                }
                let x = pool[self.r(pool.len() as u32) as usize];
                let Some(camp) = self.pick_property(KIND_CAMP, h) else { return };
                let Some(i) = self.rec(38, h, camp as u8, 2) else { return };
                let look = self.r(3) as i32;
                let law = self.globals.forbidden(1, 8);
                // 0xA1341: count name, confession, city
                let invite = |st: &mut State, i: usize, ev: &mut Vec<Event>| {
                    let count = st.r(117) as u8;
                    st.heap.recs[i][4] = count;
                    let city = st.r(10) as i32;
                    let conf = st.r(2) as i32;
                    st.heap.recs[i][5] = 1;
                    note(ev, 38, 11, count as i32, conf, city);
                };
                if ask_args(ch, t, 0, x as i32 | look << 8, camp as i32) != 0 {
                    // deny: 0xA106D (law off), 0xA0FAC (law on)
                    if !law {
                        note(ev, 38, 7, 0, 0, 0);
                        let v = self.ri(10);
                        self.favour_decrease(h, x, v);
                        self.heap.delete(i);
                    } else {
                        note(ev, 38, 6, x as i32, 0, 0);
                        self.heap.recs[i][5] = 0;
                    }
                } else if !law {
                    invite(self, i, ev);
                } else {
                    let b = ask(ch, t, 1) as i32;
                    if ask(ch, t, 2) == 1 {
                        note(ev, 38, 8, 0, 0, 0);
                        self.heap.recs[i][5] = 0; // threat
                    } else if b < self.standing(h, 40) {
                        note(ev, 38, 9, b, 0, 0);
                        let v = self.ri(10);
                        self.favour_decrease(h, x, v);
                        self.heap.delete(i);
                    } else {
                        note(ev, 38, 10, 0, 0, 0);
                        self.pay(h, b);
                        self.add_cash(x, b);
                        invite(self, i, ev);
                    }
                }
                true
            }
            39 => {
                let c = self.r(10) as u8;
                let figure = self.r(3) as i32;
                let time = self.r(3) as i32;
                if ask_arg(ch, t, 0, c as i32 | figure << 4 | time << 6) == 0 {
                    return note(ev, 39, 1, 0, 0, 0); // the first answer declines
                }
                let p = self.standing(h, 20) as u16;
                let y = ask_arg(ch, t, 1, p as i32).min(10) as u8;
                if y == 0 {
                    return note(ev, 39, 2, 0, 0, 0);
                }
                let Some(i) = self.rec(39, h, NONE, y) else { return };
                self.heap.recs[i][4] = c;
                let yr = self.year;
                heap::w16(&mut self.heap.recs[i], 5, yr);
                heap::w16(&mut self.heap.recs[i], 7, p);
                self.pay(h, p as i32 / 2);
                note(ev, 39, 3, p as i32 / 2, 0, 0);
                true
            }
            40 => {
                let Some(i) = self.rec(40, h, NONE, 2) else { return };
                for o in [4, 5, 6, 8] {
                    let m = [9, 2, 10, 0, 10][o - 4];
                    self.heap.recs[i][o] = self.r(m) as u8;
                }
                let name = self.r(117) as i32;
                let rep = self.r(7) as i32;
                let rc = &self.heap.recs[i];
                let packed = rc[4] as i32 | (rc[5] as i32) << 4 | (rc[6] as i32) << 5 | (rc[8] as i32) << 9 | rep << 13 | name << 16;
                if ask_arg(ch, t, 0, packed) != 0 {
                    self.heap.delete(i);
                    return note(ev, 40, 1, 0, 0, 0);
                }
                true
            }
            41 => {
                let Some(b) = self.pick_property(KIND_BRANCH, h) else { return };
                if !self.props[b].has_residence() {
                    return;
                }
                let cost = self.fy(1000);
                if ask_args(ch, t, 0, cost, b as i32) != 0 {
                    return;
                }
                self.pay(h, cost);
                let c = 2 + self.r(5) as u8;
                self.rec(41, h, b as u8, c).is_some()
            }
            42 => {
                let Some(i) = self.rec(42, h, NONE, 8) else { return };
                self.heap.recs[i][4] = self.r(4) as u8;
                self.heap.recs[i][5] = self.r(10) as u8;
                let fee = self.standing(h, 10);
                let season = self.r(4) as i32;
                let packed = self.heap.recs[i][4] as i32 | (self.heap.recs[i][5] as i32) << 4 | season << 8;
                if ask_args(ch, t, 0, fee, packed) != 0 {
                    let tav = self.r(30) as i32;
                    self.heap.delete(i);
                    return note(ev, 42, 0, tav, 0, 0);
                }
                self.pay(h, fee);
                true
            }
            43 => {
                let Some(x) = self.pick_cp(None, None) else { return };
                let Some(i) = self.rec(43, h, NONE, 1) else { return };
                self.heap.recs[i][4] = x as u8;
                self.heap.recs[i][5] = self.r(30) as u8;
                self.card_game(i, ch, ev);
                true
            }
            44 => {
                let idx = self.r(13) as u8;
                let l = 10 + self.r(3) as u8;
                let pool: Vec<usize> = self.computers().filter(|&x| self.persons[x].loc() == l && self.persons[x].idx() == idx).collect();
                if pool.is_empty() {
                    return;
                }
                let x = pool[self.r(pool.len() as u32) as usize];
                // draw order of the original (rules-13 §4): deadline, reward, figure
                let c = 3 + self.r(6) as u8;
                let reward = self.fy(5000);
                let figure = self.r(3) as i32;
                if ask_args(ch, t, 0, reward, x as i32 | figure << 8 | (c as i32) << 12) != 0 {
                    return note(ev, 44, 4, 0, 0, 0);
                }
                note(ev, 44, 5, x as i32, 0, 0);
                let Some(i) = self.rec(44, h, NONE, c) else { return };
                let (pl, pi) = (self.persons[x].loc(), self.persons[x].idx());
                let r = &mut self.heap.recs[i];
                r[4] = x as u8;
                r[5] = pl;
                r[6] = pi;
                heap::w32(r, 7, reward as u32);
                true
            }
            45 => {
                let county = 10 + self.r(3) as u8;
                // quirk: no check for "none"
                let x = self.pick_cp(Some(county), Some(false)).map_or(NONE, |x| x as u8);
                if ask_args(ch, t, 0, x as i32 | (county as i32) << 8, 0) != 0 {
                    return note(ev, 45, 3, x as i32, 0, 0);
                }
                note(ev, 45, 4, x as i32, 0, 0);
                let Some(i) = self.rec(45, h, NONE, 2) else { return };
                self.heap.recs[i][4] = county;
                self.heap.recs[i][5] = x;
                true
            }
            46 => {
                if self.props_of(h, KIND_CASTLE).next().is_none() {
                    return;
                }
                let Some(x) = self.pick_cp(Some(13), Some(false)) else { return };
                let k = self.r(3) as u8;
                if ask_args(ch, t, 0, x as i32 | (k as i32) << 8, 0) != 0 {
                    return note(ev, 46, 7, x as i32, 0, 0);
                }
                let Some(i) = self.rec(46, h, NONE, 4) else { return };
                self.heap.recs[i][4] = x as u8;
                self.heap.recs[i][5] = k;
                true
            }
            47 => {
                let c = 2 + self.r(3) as u8;
                let city = self.r(10) as u8;
                let bc = self.pick_property(KIND_BRANCH, h).map(|b| self.props[b].loc() as usize);
                // quirk: 15 if none (out of range)
                let g = bc.and_then(|bc| (0..GOODS).find(|&g| self.cities[bc].yield_(g) >= 140)).unwrap_or(15) as u8;
                let q = 200 + self.r(800);
                let price = if (g as usize) < GOODS { self.cities[city as usize].price(g as usize) as u32 } else { 0 };
                let y = q * price / 2;
                let pen = (c as u32 * y / 100) * 100 + 100;
                let name = self.r(117) as i32;
                let adj = self.r(4) as i32;
                let packed = c as i32 | (city as i32) << 3 | (g as i32) << 7 | name << 11 | adj << 18 | (q as i32) << 20;
                if ask_args(ch, t, 0, pen as i32, packed) != 0 {
                    return note(ev, 47, 2, 0, 0, 0);
                }
                note(ev, 47, 3, c as i32, q as i32, g as i32 | (city as i32) << 8);
                let Some(i) = self.rec(47, h, NONE, c) else { return };
                let r = &mut self.heap.recs[i];
                r[4] = city;
                r[5] = g;
                heap::w16(r, 6, q as u16);
                heap::w32(r, 8, y);
                heap::w32(r, 12, pen);
                true
            }
            48 => {
                let Some(b) = self.pick_property(KIND_BRANCH, h) else { return };
                let n = 2 + self.r(10) as u8;
                let c = self.props[b].loc() as usize;
                let p = self.cities[c].tprice(KARREN) as u32 * n as u32 / 2;
                let place = self.r(3) as i32;
                if ask_args(ch, t, 0, p as i32, n as i32 | place << 8 | (b as i32) << 16) == 0 {
                    return note(ev, 48, 4, 0, 0, 0); // answer 0 declines
                }
                let Some(i) = self.rec(48, h, b as u8, 5) else { return };
                self.heap.recs[i][4] = n;
                heap::w32(&mut self.heap.recs[i], 5, p);
                true
            }
            49 => {
                let tavern = self.r(30) as i32;
                // 0xA7AA1 / 0xA7AE4 (ids 4, 5), 0xA7C17 / 0xA7C58 (ids 6, 7)
                let v = if ask_arg(ch, t, 0, tavern) == 0 {
                    if ask(ch, t, 1) == 1 {
                        return note(ev, 49, 4, 0, 0, 0);
                    }
                    note(ev, 49, 5, 0, 0, 0);
                    0
                } else {
                    if ask(ch, t, 2) == 0 {
                        return note(ev, 49, 6, 0, 0, 0);
                    }
                    note(ev, 49, 7, 0, 0, 0);
                    1
                };
                let Some(i) = self.rec(49, h, NONE, 1) else { return };
                self.heap.recs[i][4] = v;
                true
            }
            _ => false,
        };
        if started {
            ev.push(Event::SubPlot { person: hb, kind: t, step: 0xFF });
        }
    }

    /// Handler for sub-plot record `i` (types 32..49) after the turn-end countdown.
    pub fn subplot_tick(&mut self, i: usize, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        ch.show(ev);
        let r = self.heap.recs[i].clone();
        let (t, h, c) = (r[0], r[1] as usize, r[3]);
        ev.push(Event::SubPlot { person: h as u8, kind: t, step: c });
        let del = |st: &mut State| st.heap.delete(i);
        match t {
            32 => match c {
                3 => {
                    // 0xAA202 (4), 0xAA580 (5), 0xAA5E4 (6)
                    if !yes(ch, t, 10) {
                        note(ev, 32, 4, 0, 0, 0);
                        return del(self);
                    }
                    let p = self.fy(800) as u16;
                    heap::w16(&mut self.heap.recs[i], 4, p);
                    if ask_arg(ch, t, 11, p as i32) != 0 {
                        note(ev, 32, 5, 0, 0, 0);
                        return del(self);
                    }
                    self.pay(h, p as i32);
                    note(ev, 32, 6, 0, 0, 0);
                }
                1 | 2 => {
                    let p = heap::r16(&r, 4) as i32;
                    self.pay(h, p);
                    ev.push(Event::PlotNote { kind: 32, id: 0, a: p, b: 0, c: 0 });
                }
                0 => {
                    let mut u = self.r(3) as usize;
                    let n = 1 + self.r([8, 2, 6][u]) as u8;
                    if let Some(b) = self.pick_property(KIND_BRANCH, h) {
                        if u == 1 && !navigable(self.props[b].loc() as usize) {
                            u = 0;
                        }
                        self.add_transporters(b, 3 + u, n, 100);
                    }
                    // the letter 0xABD7E: count and kind after the navigability fallback
                    note(ev, 32, 7, n as i32, u as i32, 0);
                    del(self);
                }
                _ => {}
            },
            33 => {
                let n = heap::r16(&r, 7);
                if n == 0xFFFF {
                    // quality bits: the first draw is the most significant (rules-13 §4)
                    let offers: Vec<u8> = (0..3).map(|_| (0..4).fold(0u8, |q, _| (q << 1) | self.r(2) as u8)).collect();
                    let packed = offers[0] as i32 | (offers[1] as i32) << 4 | (offers[2] as i32) << 8;
                    let q = offers[(ask_args(ch, t, 10, packed, r[4] as i32 | (r[5] as i32) << 6) as usize).min(2)];
                    let n = ask_args(ch, t, 11, q as i32, r[4] as i32).min(0xFFFE) as u16;
                    self.heap.recs[i][6] = q;
                    heap::w16(&mut self.heap.recs[i], 7, n);
                    self.pay(h, (q as i32 + 5) * n as i32);
                    note(ev, 33, 6, (q as i32 + 5) * n as i32, r[4] as i32, 0);
                } else if c == 0 {
                    ev.push(Event::PlotNote { kind: 33, id: 0, a: n as i32, b: r[4] as i32, c: 0 });
                    del(self);
                } else {
                    let (q, a) = (r[6] as i32, r[5] as i32);
                    let d = (q + 5) * (11 - a) + self.ri(10);
                    if (n as i32) < d {
                        self.add_cash(h, (q + 8) * n as i32 * 2);
                        ev.push(Event::PlotNote { kind: 33, id: 1, a: r[4] as i32, b: (q + 8) * n as i32 * 2, c: 0 });
                        del(self);
                    } else {
                        self.add_cash(h, (q + 5) * d * 2);
                        ev.push(Event::PlotNote { kind: 33, id: 2, a: r[4] as i32, b: d, c: (q + 5) * d * 2 });
                        heap::w16(&mut self.heap.recs[i], 7, n - d as u16);
                    }
                }
            }
            34 => match c {
                4 => {
                    let w = 10 + self.r(90);
                    let m = w - 9 + self.r(150);
                    let carts: u32 = self.props_of(h, KIND_BRANCH).map(|b| self.props[b].trans(KARREN) as u32).sum();
                    if carts == 0 {
                        note(ev, 34, 5, 0, 0, 0);
                        return del(self);
                    }
                    let n = ask_args(ch, t, 10, carts as i32, r[4] as i32 | (w as i32) << 4 | (m as i32) << 14).min(carts);
                    let mm = ask(ch, t, 11).min(0xFFFF);
                    if n == 0 {
                        note(ev, 34, 6, 0, 0, 0);
                        return del(self);
                    }
                    note(ev, 34, 7, 0, 0, 0);
                    let rr = &mut self.heap.recs[i];
                    rr[5] = n as u8;
                    heap::w16(rr, 7, mm as u16);
                    heap::w16(rr, 9, (w + n) as u16);
                    heap::w16(rr, 11, (m + mm) as u16);
                    self.pay(h, (100 * n + 200 * mm) as i32);
                    let mut left = n as u8;
                    for b in self.props_of(h, KIND_BRANCH).collect::<Vec<_>>() {
                        let x = left.min(self.props[b].trans(KARREN));
                        let v = self.props[b].trans(KARREN) - x;
                        self.props[b].set_trans(KARREN, v);
                        left -= x;
                    }
                }
                1..=3 => ev.push(Event::PlotNote { kind: 34, id: 3, a: r[4] as i32, b: 0, c: 0 }),
                0 => {
                    let (n, w, m) = (r[5] as u32, heap::r16(&r, 9) as u32, heap::r16(&r, 11) as u32);
                    let b = self.pick_property(KIND_BRANCH, h);
                    let (dest, city) = (r[4] as i32, b.map_or(0, |b| self.props[b].loc() as i32));
                    if w * self.r(100) >= 100 * m {
                        ev.push(Event::SubPlot { person: h as u8, kind: t, step: 0xFE });
                        ev.push(Event::PlotNote { kind: 34, id: 2, a: dest, b: city, c: 0 });
                    } else if let Some(b) = b {
                        let rich = m * self.r(100) >= 70 * w;
                        if rich {
                            let x = self.fy(1000);
                            self.pay(h, x);
                            ev.push(Event::PlotNote { kind: 34, id: 1, a: dest, b: city, c: x });
                        } else {
                            ev.push(Event::PlotNote { kind: 34, id: 0, a: dest, b: city, c: 0 });
                        }
                        let cond = 30 + self.r(50) as u8;
                        self.add_transporters(b, KARREN, n as u8, cond);
                        let (goods, base) = if rich { ([12, 13, 14], 4) } else { ([9, 10, 11], 3) };
                        for g in goods {
                            let q = base * n + self.r((3 * n).max(1));
                            self.add_goods(b, g, q as u16);
                        }
                    }
                    del(self);
                }
                _ => {}
            },
            36 => match c {
                1 => {
                    let (rr, q) = (r[5] as i32, r[6] as i8 as i32);
                    let mut opts = vec![];
                    for k in 0..3 {
                        let n = (60 + self.ri(50)) * rr / 100 + 2;
                        let s = if k == 0 { q + self.ri(3) } else { self.ri(11) };
                        opts.push((n, s, (30 - s) * n));
                    }
                    // taverns: the third group is drawn first (rules-13 §2.4)
                    let tav: Vec<i32> = (0..3).map(|_| self.r(10) as i32).collect();
                    let a1 = (opts[0].0 & 127) | (opts[1].0 & 127) << 7 | (opts[2].0 & 127) << 14 | (r[4] as i32) << 21;
                    let a2 = (opts[0].1 + 8) | (opts[1].1 + 8) << 5 | (opts[2].1 + 8) << 10 | tav[2] << 15 | tav[1] << 19 | tav[0] << 23;
                    let (n, s, f) = opts[(ask_args(ch, t, 10, a1, a2) as usize).min(2)];
                    let rec = &mut self.heap.recs[i];
                    rec[7] = n as u8;
                    rec[8] = s as i8 as u8;
                    heap::w16(rec, 9, f as u16);
                    self.pay(h, f);
                    note(ev, 36, 5, 0, 0, 0);
                }
                0 => {
                    let (rr, q, n, s, f) = (r[5] as i32, r[6] as i8 as i32, r[7] as i32, r[8] as i8 as i32, heap::r16(&r, 9) as i32);
                    // rules-39 §6: music(0x1D) draws first; the favour loop runs over places 0..3 (the loop index, the
                    // first four cities)
                    let song = self.song(0x1D);
                    let people: Vec<usize> = (0..4).flat_map(|l| self.at_loc(l)).collect();
                    let pay = if rr < n && s < q {
                        for p in &people {
                            let v = 2 + self.ri(10);
                            self.favour_increase(h, *p, v);
                        }
                        (100 + self.ri(100)) * f / 100
                    } else if n < rr && q < s {
                        for p in &people {
                            let v = 2 + self.ri(10);
                            self.favour_decrease(h, *p, v);
                        }
                        0
                    } else {
                        (80 + self.ri(40)) * f / 100
                    };
                    self.add_cash(h, pay as u16 as i32);
                    let outcome = if rr < n && s < q { 2 } else if n < rr && q < s { 0 } else { 1 };
                    ev.push(Event::PlotNote { kind: 36, id: outcome, a: r[4] as i32, b: pay as u16 as i32, c: song as i32 });
                    self.history(h, 3, n as u8, s as u8);
                    del(self);
                }
                _ => {}
            },
            37 => {
                let mut other = 0;
                let win = match r[4] {
                    0 => r[7] <= self.cities[r[5] as usize].price(r[6] as usize),
                    1 => self.persons[h].life() > 14,
                    _ => {
                        let w = self.r(50) < 100;
                        other = self.r(9) as i32;
                        w
                    }
                };
                let s = heap::r16(&r, 8) as i32;
                if win { self.add_cash(h, s) } else { self.pay(h, s) }
                ev.push(Event::PlotNote { kind: 37, id: r[4], a: win as i32 | other << 1, b: r[5] as i32 | (r[6] as i32) << 8, c: s });
                del(self);
            }
            38 => {
                let camp = r[2] as usize;
                match (c, r[5]) {
                    (1, 0) => {
                        // the arrest (0x885C6): submit or flee, the same fine either way (rules-13 §2.6)
                        let a = ask(ch, t, 11).min(1) as i32;
                        let f = self.standing(h, 50);
                        self.pay(h, f);
                        ev.push(Event::PlotNote { kind: 38, id: 2, a, b: f, c: 0 });
                        del(self);
                    }
                    (1, _) => {
                        let tot: u32 = (0..4).map(|k| self.props[camp].troops(k) as u32).sum();
                        if tot == 0 {
                            note(ev, 38, 3, camp as i32, 0, 0);
                            return del(self);
                        }
                        let n = ask_args(ch, t, 10, tot.min(99) as i32, r[4] as i32).min(tot);
                        if n == 0 {
                            note(ev, 38, 4, 0, 0, 0);
                            return del(self);
                        }
                        note(ev, 38, 5, 0, 0, 0);
                        let mut moved = 0;
                        for k in 0..n as usize {
                            let ty = k % 4;
                            if self.props[camp].troops(ty) > 0 {
                                let v = self.props[camp].troops(ty) - 1;
                                self.props[camp].set_troops(ty, v);
                                self.heap.recs[i][7 + ty] += 1;
                                moved += 1;
                            }
                        }
                        // +6 stores the requested r; +7..+10 are write-only (rules-40 §19)
                        let _ = moved;
                        self.heap.recs[i][6] = n as u8;
                    }
                    (0, _) => {
                        let n = r[6] as usize;
                        let mut lost = 0u32;
                        for k in 0..n {
                            let ty = k % 4;
                            if self.r(100) < 65 {
                                let v = (self.props[camp].troops(ty) as u32 + 1).min(150);
                                self.props[camp].set_troops(ty, v as u8);
                            } else {
                                lost += 1;
                            }
                        }
                        let place = self.r(3) as i32;
                        let mut qs = [0i32; 3];
                        for (k, g) in [11, 12, 13].into_iter().enumerate() {
                            let q = lost * (4 + self.r(18)) + self.r(25);
                            self.add_goods(camp, g, q as u16);
                            qs[k] = q as i32;
                        }
                        let epithet = self.r(18) as i32;
                        let nick = self.r(10) as i32;
                        ev.push(Event::PlotNote { kind: 38, id: 0, a: camp as i32, b: r[4] as i32, c: lost as i32 | place << 8 | nick << 12 | epithet << 16 });
                        ev.push(Event::PlotNote { kind: 38, id: 1, a: qs[0], b: qs[1], c: qs[2] });
                        note(ev, 38, 12, nick, epithet, 0); // the signature 0xA8A77
                        del(self);
                    }
                    _ => {}
                }
            }
            39 => {
                let p = heap::r16(&r, 7) as i32;
                self.pay(h, p);
                ev.push(Event::PlotNote { kind: 39, id: 0, a: heap::r16(&r, 5) as i32, b: r[4] as i32 | (p << 8), c: (c == 0) as i32 });
                if c == 0 {
                    del(self);
                }
            }
            40 => match c {
                1 => {
                    // step 10: arg = hours + 2 (0xA63D8 on the first ask, 0xA66AD on a repeat),
                    // arg2 = profession | title << 4 | surname << 5 | city << 9 | repeat << 13
                    let who = r[4] as i32 | (r[5] as i32) << 4 | (r[6] as i32) << 5 | (r[8] as i32) << 9;
                    let mut hours = self.r(10) as i32 + 2;
                    // ponytail: the dialog repeats until the player picks line 2; capped at 50
                    // repeats so an always-0 `Choices` cannot hang the sim.
                    for k in 0..50 {
                        if ask_args(ch, t, 10, hours, who | ((k > 0) as i32) << 13) >= 2 {
                            break;
                        }
                        hours = self.r(10) as i32 + 2;
                    }
                    let g = self.r(15) as u8;
                    self.heap.recs[i][7] = g;
                    if ask_arg(ch, t, 11, g as i32) != 0 {
                        note(ev, 40, 2, r[4] as i32, 0, 0);
                        return del(self);
                    }
                    let a = self.fy(1500);
                    let o = if ask_args(ch, t, 12, a, g as i32) == 0 { a } else { ask(ch, t, 13) as i32 };
                    heap::w16(&mut self.heap.recs[i], 9, o as u16);
                    // 0xA6AA9 accepted (3), 0xA6A23 refused (4, the title)
                    if (a as i64) * (self.r(100) as i64) >= 100 * o as i64 {
                        note(ev, 40, 4, r[5] as i32, 0, 0);
                        del(self);
                    } else {
                        note(ev, 40, 3, 0, 0, 0);
                    }
                }
                0 => {
                    let o = heap::r16(&r, 9) as i32;
                    self.pay(h, o);
                    ev.push(Event::PlotNote { kind: 40, id: 0, a: r[7] as i32, b: o, c: 0 });
                    let x = 5 + self.r(10);
                    self.skill_up(h, r[7] as usize, x);
                    del(self);
                }
                _ => {}
            },
            41 => {
                let b = r[2] as usize;
                if c == 0 || !self.props[b].has_residence() || self.props[b].owner() as usize != h {
                    return del(self);
                }
                let cost = self.fy(1000);
                // step 11: arg2 = -1 without a find, else find | answer << 4 | branch << 8 (the reaction
                // 0x9E8DF / 0x9E90E opens that dialog)
                let mut find = -1;
                if self.r(100) < 60 {
                    let f = self.r(15) as i32;
                    let a = ask_args(ch, t, 10, f, b as i32) == 1;
                    if a {
                        let x = self.r(8);
                        let city = self.props[b].loc() as usize;
                        self.wealth_up(city, x);
                    }
                    find = f | (a as i32) << 4 | (b as i32) << 8;
                }
                if ask_args(ch, t, 11, cost, find) != 0 {
                    return del(self);
                }
                self.pay(h, cost);
            }
            42 => match c {
                7 => {
                    // 0x97480: seat | greeting << 4 | attendees << 8, title | city << 4; then quote and reaction
                    // pairs, then 0x9844B
                    let greet = self.r(5) as i32;
                    let att = self.r(20) as i32;
                    let seat = self.r(3) as i32;
                    note(ev, 42, 1, seat | greet << 4 | att << 8, r[4] as i32 | (r[5] as i32) << 4, 0);
                    let n = 1 + self.r(3);
                    for _ in 0..n {
                        let q = self.r(10) as i32;
                        let re = self.r(11) as i32;
                        note(ev, 42, 2, q, re, 0);
                    }
                    note(ev, 42, 3, 0, 0, 0);
                }
                6 => {
                    note(ev, 42, 4, r[4] as i32, 0, 0);
                    // rules-46 §3: J among the computer clergy (one rand(n), none without any), then rand(100) always;
                    // the invitation names the next title (title + rand(3)) mod 4, then one rand(4) per person there
                    let clergy: Vec<usize> = self.computers().filter(|&x| self.persons[x].church()).collect();
                    let j = (!clergy.is_empty()).then(|| clergy[self.r(clergy.len() as u32) as usize]);
                    let roll = self.r(100);
                    if let Some(j) = j.filter(|_| roll < 40) {
                        let tt = (r[4] as u32 + self.r(3)) % 4;
                        note(ev, 42, 5, tt as i32, j as i32, 0);
                        for p in self.at_loc(r[5]) {
                            let v = self.ri(4);
                            self.favour_increase(h, p, v);
                        }
                        del(self);
                    } else {
                        note(ev, 42, 6, 0, 0, 0);
                    }
                }
                0 => del(self),
                _ => {}
            },
            43 => self.card_game(i, ch, ev),
            44 => {
                let x = r[4] as usize;
                if c == 0 {
                    if r[6] == 15 {
                        self.add_cash(h, heap::r32(&r, 7) as i32);
                    }
                    ev.push(Event::PlotNote { kind: 44, id: if r[6] == 15 { 2 } else { 3 }, a: x as i32, b: heap::r32(&r, 7) as i32, c: 0 });
                    del(self);
                } else if self.persons[x].loc() != r[5] || self.persons[x].idx() != r[6] {
                    // toppled: the office is named as it was (+5 location, +6 index)
                    ev.push(Event::PlotNote { kind: 44, id: 1, a: x as i32, b: r[5] as i32, c: r[6] as i32 });
                    self.heap.recs[i][6] = 15;
                    self.heap.recs[i][3] = 1;
                } else {
                    ev.push(Event::PlotNote { kind: 44, id: 0, a: x as i32, b: 0, c: 0 });
                }
            }
            45 => match c {
                1 => {
                    let mut opts = vec![];
                    let (mut fees, mut looks) = (0i32, r[5] as i32);
                    for k in 0..3 {
                        let rank = self.r(3) as i32;
                        let g = self.ri(5);
                        let f = self.ri(10) + (4 - g) * 50 + 100;
                        opts.push((g, f));
                        fees |= f << (9 * k);
                        looks |= (rank | g << 2) << (8 + 5 * k);
                    }
                    let (g, f) = opts[(ask_args(ch, t, 10, fees, looks) as usize).min(2)];
                    self.heap.recs[i][6] = g as u8;
                    heap::w16(&mut self.heap.recs[i], 7, f as u16);
                    self.pay(h, f);
                    note(ev, 45, 5, f, 0, 0);
                }
                0 => {
                    let (g, f) = (r[6] as i32, heap::r16(&r, 7) as i32);
                    let people = self.at_loc(r[4]);
                    let pay = match g * 3 / 5 {
                        0 => {
                            let p = (110 + self.ri(190)) * f / 100;
                            for x in &people {
                                let v = 2 + self.ri(10);
                                self.favour_increase(h, *x, v);
                            }
                            p
                        }
                        1 => (100 + self.ri(20)) * f / 100,
                        _ => {
                            let p = (20 + self.ri(100)) * f / 100;
                            for x in &people {
                                let v = 2 + self.ri(10);
                                self.favour_decrease(h, *x, v);
                            }
                            p
                        }
                    };
                    self.add_cash(h, pay);
                    ev.push(Event::PlotNote { kind: 45, id: (g * 3 / 5).min(2) as u8, a: r[5] as i32, b: pay, c: 0 });
                    del(self);
                }
                _ => {}
            },
            46 => self.revolt_tick(i, ch, ev),
            47 => {
                let (city, g, q) = (r[4] as usize, r[5] as usize, heap::r16(&r, 6) as u32);
                let sold = if g < GOODS { self.cities[city].sold(g) } else { 0 };
                if sold < q {
                    self.pay(h, heap::r32(&r, 12) as i32);
                    ev.push(Event::PlotNote { kind: 47, id: 0, a: g as i32, b: city as i32, c: heap::r32(&r, 12) as i32 });
                    del(self);
                } else {
                    self.add_cash(h, heap::r32(&r, 8) as i32);
                    ev.push(Event::PlotNote { kind: 47, id: 1, a: heap::r32(&r, 8) as i32, b: 0, c: (c == 0) as i32 });
                    if c == 0 {
                        del(self);
                    }
                }
            }
            48 => {
                let (b, n) = (r[2] as usize, r[4]);
                let city = self.props[b].loc() as i32;
                if c == 4 {
                    if self.props[b].trans(KARREN) < n {
                        ev.push(Event::PlotNote { kind: 48, id: 0, a: self.props[b].trans(KARREN) as i32, b: city, c: 0 });
                        return del(self);
                    }
                    let v = self.props[b].trans(KARREN) - n;
                    self.props[b].set_trans(KARREN, v);
                    self.add_cash(h, heap::r32(&r, 5) as i32);
                    ev.push(Event::PlotNote { kind: 48, id: 1, a: n as i32, b: city, c: heap::r32(&r, 5) as i32 });
                } else if self.r(100) < 10 {
                    ev.push(Event::PlotNote { kind: 48, id: 2, a: n as i32, b: city, c: 0 });
                    del(self);
                } else {
                    ev.push(Event::PlotNote { kind: 48, id: 3, a: heap::r32(&r, 5) as i32, b: n as i32 | city << 8, c: (c == 0) as i32 });
                    if c == 0 {
                        if self.props[b].kind() == KIND_BRANCH && self.props[b].owner() as usize == h {
                            self.add_transporters(b, KARREN, n, 100);
                        }
                        del(self);
                    }
                }
            }
            49 => {
                if r[4] == 0 {
                    // draw order of the original (rules-13 §2.17): prize, second place, first place
                    let x = self.fy(200);
                    let d1 = self.r(3) as i32;
                    let d2 = self.r(3) as i32;
                    self.add_cash(h, x);
                    ev.push(Event::PlotNote { kind: 49, id: 0, a: 1 + d2, b: 4 + d1, c: x });
                } else if let Some(x) = self.pick_cp(Some(0), Some(false)) {
                    let f = (self.persons[x].favour(h) as i32 / 21).min(4);
                    let b = (ask_args(ch, t, 10, x as i32, f) as i32).clamp(0, 9999);
                    let ok = 2000 * f + b >= 10000;
                    ev.push(Event::PlotNote { kind: 49, id: if ok { 3 } else { 2 }, a: x as i32, b, c: 0 });
                    if ok {
                        self.pay(h, b);
                        self.add_cash(x, b);
                        let v = 5 + self.r(10);
                        self.skill_up(h, 1, v);
                        for idx in 0..5 {
                            if let Some(p) = self.holder(0, idx) {
                                let v = self.ri(10);
                                self.favour_increase(h, p, v);
                            }
                        }
                    }
                } else {
                    ev.push(Event::PlotNote { kind: 49, id: 1, a: 0, b: 0, c: 0 });
                }
                del(self);
            }
            _ => {
                if c == 0 {
                    del(self);
                }
            }
        }
    }

    /// 17+4 (type 43), start and every later year.
    fn card_game(&mut self, i: usize, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        let (h, x, c) = (self.heap.recs[i][1] as usize, self.heap.recs[i][4] as usize, self.heap.recs[i][3]);
        // 0xAF981 no-show (0), 0xA2841 declined (1), the deal (2), 0xA2ACB rematch agreed (3)
        if c == 0 && self.r(100) < 50 {
            note(ev, 43, 0, x as i32, 0, 0);
            return self.heap.delete(i);
        }
        let tavern = self.heap.recs[i][5] as i32;
        let s = ask_args(ch, 43, 10, tavern, x as i32 | (c as i32) << 8) as i32;
        if s == 0 {
            note(ev, 43, 1, x as i32, 0, 0);
            return self.heap.delete(i);
        }
        let mut deck: Vec<i32> = (0..52).map(|i| if i % 13 < 9 { i % 13 + 2 } else { 11 }).collect();
        for _ in 0..200 {
            let (a, b, cc) = (self.r(52) as usize, self.r(52) as usize, self.r(52) as usize);
            let old = deck[a];
            deck[a] = deck[b];
            deck[b] = deck[cc];
            deck[cc] = old;
        }
        let pair = |a: i32, b: i32| if a + b == 22 { 21 } else { a + b };
        let mut xt = pair(deck[0], deck[1]);
        let mut ht = pair(deck[2], deck[3]);
        let mut next = 4;
        while (21 - xt) * 13 > 95 && next < 52 {
            xt += deck[next];
            next += 1;
        }
        note(ev, 43, 2, x as i32, next as i32 - 4, 0);
        while ht < 21 && next < 52 && ask_args(ch, 43, 11, ht, x as i32) == 0 {
            ht += deck[next];
            next += 1;
        }
        let human_won = !((xt > 21 && ht > 21) || xt == ht) && !(xt <= 21 && (ht > 21 || ht < xt));
        if (xt > 21 && ht > 21) || xt == ht {
        } else if xt <= 21 && (ht > 21 || ht < xt) {
            self.add_cash(x, s);
            self.pay(h, s);
        } else {
            self.add_cash(h, s);
            self.pay(x, s);
        }
        ev.push(Event::Cards { mine: ht as u8, theirs: xt as u8, x: x as u8 });
        if ask_args(ch, 43, 12, human_won as i32, x as i32) == 0 {
            note(ev, 43, 3, x as i32, 0, 0);
            self.heap.recs[i][3] = 1;
        } else {
            note(ev, 43, 1, x as i32, 0, 0);
            self.heap.delete(i);
        }
    }

    /// Peasant revolt (type 46).
    fn revolt_tick(&mut self, i: usize, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        let r = self.heap.recs[i].clone();
        let (h, x, k, c) = (r[1] as usize, r[4] as usize, r[5] as u32, r[3]);
        let castles: Vec<usize> = self.props_of(h, KIND_CASTLE).collect();
        match c {
            3 => {
                let s: u32 = castles.iter().map(|&p| (0..4).map(|t| self.props[p].troops(t) as u32).sum::<u32>()).sum();
                let gather = 3 * k as i32 + 1 + self.r(3) as i32;
                ev.push(Event::PlotNote { kind: 46, id: if s == 0 { 4 } else { 0 }, a: gather, b: 0, c: 0 });
                let n = if s == 0 { 0 } else { ask_args(ch, 46, 10, s as i32, x as i32 | (r[5] as i32) << 8).min(s) };
                // 0xAB16C (5; also after 0xAB0CE, the S = 0 case shares the n = 0 path), 0xAB199 (6)
                if n == 0 {
                    note(ev, 46, 5, x as i32, 0, 0);
                    self.favour_decrease(h, x, 10);
                    return self.heap.delete(i);
                }
                note(ev, 46, 6, k as i32, 0, 0);
                // +6 is a u16; +8..+11 cleared; per castle the shares, then extras from that castle while the
                // signed remainder is positive, one rand(4) each (rules-40 §20)
                heap::w16(&mut self.heap.recs[i], 6, n as u16);
                self.heap.recs[i][8..12].fill(0);
                let mut rem: i64 = 0;
                for &p in &castles {
                    for t in 0..4 {
                        let cnt = self.props[p].troops(t) as u32;
                        let take = cnt * n / s;
                        rem += (cnt * n % s) as i64;
                        self.props[p].set_troops(t, (cnt - take) as u8);
                        self.heap.recs[i][8 + t] += take as u8;
                    }
                    while rem > 0 {
                        let t0 = self.r(4) as usize;
                        let Some(t) = (0..4).map(|q| (t0 + q) % 4).find(|&t| self.props[p].troops(t) > 0) else { break };
                        let v = self.props[p].troops(t) - 1;
                        self.props[p].set_troops(t, v);
                        self.heap.recs[i][8 + t] += 1;
                        rem -= s as i64;
                    }
                }
            }
            2 => {
                if self.r(100) < 66 {
                    let place = self.r(7) as i32;
                    ev.push(Event::PlotNote { kind: 46, id: 1, a: place, b: x as i32, c: 0 });
                } else {
                    let city = 3 * k as i32 + 1 + self.r(3) as i32;
                    ev.push(Event::PlotNote { kind: 46, id: 2, a: x as i32, b: city, c: 0 });
                    for t in 0..4 {
                        let v = self.heap.recs[i][8 + t] as u32 * (60 + self.r(30)) / 100;
                        self.heap.recs[i][8 + t] = v as u8;
                    }
                }
                note(ev, 46, 8, 0, 0, 0); // 0xAE2AD
                if self.r(100) < 60 {
                    self.heap.recs[i][3] = 3;
                }
            }
            _ => {
                let n = heap::r16(&r, 6) as u32;
                let back: u32 = (0..4).map(|t| self.heap.recs[i][8 + t] as u32).sum();
                for &p in &castles {
                    for t in 0..4 {
                        while self.heap.recs[i][8 + t] > 0 && (0..4).map(|q| self.props[p].troops(q) as u32).sum::<u32>() < 150 {
                            self.heap.recs[i][8 + t] -= 1;
                            let v = self.props[p].troops(t) + 1;
                            self.props[p].set_troops(t, v);
                        }
                    }
                }
                // bug [C]: the reward 100n + 50R is shown but never paid
                let mut new_city = 0;
                if n > 19 {
                    let city = (3 * k + 1 + self.r(3)) as u8;
                    if !(0..PROPS).any(|p| self.props[p].kind() == KIND_BRANCH && self.props[p].loc() == city) {
                        self.property_create(KIND_BRANCH, city, h as u8);
                        new_city = city as i32 + 1;
                    }
                }
                ev.push(Event::PlotNote { kind: 46, id: 3, a: n as i32, b: back as i32, c: x as i32 | new_city << 8 });
                self.heap.delete(i);
            }
        }
    }

    // ---- §6 newspaper contests ----

    /// Year-end contest announcement (0x19480).
    pub fn contest_start(&mut self, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        if !self.opt(OPT_ADVENTURE) || self.r(5) + 1605 >= self.year as u32 || self.r(100) >= 4 || self.heap.find(4, NOBODY, ANY).is_some() {
            return;
        }
        let Some(j) = self.pick_cp(None, None) else { return };
        let Some(i) = self.heap.alloc(4, NOBODY, NONE, 2) else { return };
        self.heap.recs[i][4] = self.r(10) as u8;
        self.heap.recs[i][5] = j as u8;
        for d in 0..5 {
            self.heap.recs[i][6 + d] = (3 * d as u32 + self.r(3)) as u8;
        }
        let fee = self.fy(1000) as u32;
        heap::w32(&mut self.heap.recs[i], 0x47, fee);
        let paper = self.r(6) as u8;
        let goods: [u8; 5] = std::array::from_fn(|d| self.heap.recs[i][6 + d] % 15);
        let mut slot = 0;
        for h in self.humans().take(6).collect::<Vec<_>>() {
            let mask = (ch.ask(&Ask::ContestEntry { person: h as u8, fee: fee as i32, goods, paper }) & 0x1F) as u8;
            // 0x9A3FA (0) / 0x9A437 (1)
            note(ev, 4, (mask != 0) as u8, h as i32, 0, 0);
            if mask == 0 {
                continue;
            }
            self.heap.recs[i][0x0B + slot] = h as u8;
            self.heap.recs[i][0x29 + slot] = mask;
            self.pay(h, (fee * mask.count_ones()) as i32);
            slot += 1;
        }
        if slot == 0 {
            ev.push(Event::ContestStart { city: 0, judge: 0, cancelled: true });
            return self.heap.delete(i);
        }
        ev.push(Event::ContestStart { city: self.heap.recs[i][4], judge: self.heap.recs[i][5], cancelled: false });
        // every remaining slot: the pick (0xFF if none) and its rand(32), no break (rules-40 §21)
        for s in slot..30 {
            self.heap.recs[i][0x0B + s] = self.pick_cp(None, None).map_or(NONE, |p| p as u8);
            self.heap.recs[i][0x29 + s] = self.r(32) as u8;
        }
    }

    /// Contest resolution at the next year end (0x19890).
    pub fn contest_resolve(&mut self, ev: &mut Vec<Event>) {
        let Some(i) = self.heap.find(4, NOBODY, ANY) else { return };
        let r = self.heap.recs[i].clone();
        let j = r[5] as usize;
        let fee = heap::r32(&r, 0x47) as i32;
        for d in 0..5 {
            let g = r[6 + d] as usize;
            let mut scores: Vec<(i64, usize)> = vec![];
            for s in 0..30 {
                let e = r[0x0B + s] as usize;
                if e >= PERSONS || r[0x29 + s] & (1 << d) == 0 {
                    continue;
                }
                let best = self
                    .props_of(e, KIND_BRANCH)
                    .map(|b| self.cities[self.props[b].loc() as usize].yield_(g) as i64 * self.effective_skill(b, g) as i64)
                    .max()
                    .unwrap_or(0);
                scores.push((self.persons[j].favour(e) as i64 * best, s));
            }
            // judged when a human entered the discipline, whatever the scores (rules-13 §3.2)
            if !scores.iter().any(|&(_, s)| self.persons[r[0x0B + s] as usize].human()) {
                continue;
            }
            // strictly greater wins, ties to the lower slot
            scores.sort_by_key(|&(v, s)| (-v, s));
            let places: Vec<usize> = scores.iter().filter(|x| x.0 > 0).take(3).map(|x| r[0x0B + x.1] as usize).collect();
            for (k, &p) in places.iter().enumerate().rev() {
                self.add_cash(p, [4, 3, 2][k] * fee);
            }
            ev.push(Event::Contest { discipline: d as u8, good: g as u8, fee, winners: places.iter().map(|&p| p as u8).collect(), city: r[4], judge: j as u8, first: ev.iter().all(|e| !matches!(e, Event::Contest { .. })), last: false });
        }
        if let Some(Event::Contest { last, .. }) = ev.iter_mut().rev().find(|e| matches!(e, Event::Contest { .. })) {
            *last = true;
        }
        self.heap.delete(i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::HumanSetup;

    /// Every starter and handler runs with `Auto` answers for decades without panicking.
    #[test]
    fn rules06_s5_all_subplots_run() {
        let mut st = State::new_game(5, 3, &[HumanSetup::test(4), HumanSetup::test(5)]).unwrap();
        st.year = 1640;
        let h = st.current as usize;
        let camp = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CAMP).unwrap();
        st.props[camp].set_owner(h as u8);
        st.persons[h].set_certificate(2);
        let mut ev = vec![];
        for k in 0..17 {
            st.subplot_start(h, k, &mut crate::Auto, &mut ev);
        }
        for _ in 0..10 {
            for i in 0..st.heap.recs.len() {
                let live = st.heap.live(i);
                let r = &mut st.heap.recs[i];
                if live && (32..=49).contains(&r[0]) && r[1] as usize == h {
                    r[3] = r[3].wrapping_sub(1);
                    st.subplot_tick(i, &mut crate::Auto, &mut ev);
                }
            }
        }
        assert!(ev.iter().filter(|e| matches!(e, Event::SubPlot { step: 0xFF, .. })).count() >= 8);
    }

    /// rules-06 §5.2 type 47: a sales contract pays the penalty when too little was sold.
    #[test]
    fn rules06_s5_sales_contract_penalty() {
        let mut st = State::new_game(5, 3, &[HumanSetup::test(4)]).unwrap();
        let h = st.current as usize;
        let i = st.heap.alloc(47, h as u8, NONE, 1).unwrap();
        st.heap.recs[i][4] = 2;
        st.heap.recs[i][5] = 0;
        heap::w16(&mut st.heap.recs[i], 6, 300);
        heap::w32(&mut st.heap.recs[i], 12, 5000);
        let cash = st.persons[h].cash();
        let mut ev = vec![];
        st.subplot_tick(i, &mut crate::Auto, &mut ev);
        assert_eq!(st.persons[h].cash(), cash - 5000);
        assert!(!st.heap.live(i));
    }
}
