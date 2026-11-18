//! Trials (docs/re/rules-04b-crime-trials-privileges.md §3).

use crate::heap::{self, ANY};
use crate::state::*;
use crate::{Ask, Choices, Event, WArg};

/// Next-city table `0x84CEC`.
const N: [usize; 10] = [1, 2, 3, 0, 5, 6, 4, 8, 9, 7];

impl State {
    /// The open type-18 record of `pl` against `d` of `kind` (0 trial, 1 inquisition).
    pub fn open_trial(&self, pl: usize, d: usize, kind: u8) -> Option<usize> {
        self.heap.all(18, pl as u8, ANY).into_iter().find(|&i| self.heap.recs[i][4] as usize == d && self.heap.recs[i][5] == kind)
    }

    /// Office-tree mode 5 (privilege 12), texts rules-04 §5.4: an open trial is dropped (the app asked
    /// 0xBCB04 first); office byte +0x0F check → 0xBCB7C; else a new kind-0 record and 0xBCBE0.
    pub fn file_trial(&mut self, pl: usize, d: usize) -> Option<Event> {
        if let Some(i) = self.open_trial(pl, d, 0) {
            self.heap.delete(i);
            return None;
        }
        if !crate::laws::has(self, pl, 12) {
            return Some(Event::Refused("cannot sue"));
        }
        let (t, p) = (self.persons[d].loc(), self.persons[pl].loc());
        if (t >= 13 && p < 13) || (t >= 10 && p < 10) {
            return Some(Event::MsgBox { text: 0xBCB7C, args: vec![WArg::P(d as u8)], rect: [160, 160, 320, 160] });
        }
        if let Some(i) = self.heap.alloc(18, pl as u8, NONE, 1) {
            self.heap.recs[i][4] = d as u8;
            self.heap.recs[i][5] = 0;
        }
        Some(Event::MsgBox { text: 0xBCBE0, args: vec![WArg::P(d as u8)], rect: [160, 180, 320, 120] })
    }

    /// §3.2 court seats (judge, assessor, assessor) with the vacant-seat rule.
    fn court(&mut self, d: usize, pl: usize) -> [usize; 3] {
        let l = self.persons[d].loc() as usize;
        let base: [Option<usize>; 3] = match l {
            0..=9 => [self.holder(l, 4), self.holder(N[l], 4), self.holder(N[N[l]], 4)],
            10..=12 => {
                let c = N[3 * (l - 10) + 1];
                [self.holder(l, 4), self.holder(c, 4), self.holder(N[c], 4)]
            }
            _ => {
                // c is the location 10..12, so (rand(2) + c) mod 3 = (k + 1 + rand(2)) mod 3 != k
                let c = self.r(3) as usize;
                let c2 = (c + 1 + self.r(2) as usize) % 3;
                [self.holder(13, 4), self.holder(10 + c, 4), self.holder(10 + c2, 4)]
            }
        };
        std::array::from_fn(|i| base[i..].iter().flatten().next().copied().unwrap_or(pl))
    }

    fn evidence_of(&self, pl: usize, d: usize) -> u32 {
        self.spy_contract(pl, d).map_or(0, |i| heap::r32(&self.heap.recs[i], 0x0B))
    }

    /// `trial_main(r)`: runs only when the counter is 0; deletes the record.
    pub fn trial(&mut self, ri: usize, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        let r = self.heap.recs[ri].clone();
        if r[3] != 0 {
            return;
        }
        let (pl, d, kind) = (r[1] as usize, r[4] as usize, r[5]);
        let seats = self.court(d, pl);
        let bribe = self.opt(OPT_BRIBE_COURT);
        let sides = [pl, d];
        let n0 = self.evidence_of(pl, d).count_ones() as i32;
        let (mut w, mut b) = ([0i32; 2], [0i32; 2]);
        let mut invested = vec![];
        if bribe {
            for s in 0..2 {
                if s == 1 && kind == 1 {
                    continue;
                }
                let p = sides[s];
                if self.persons[p].human() {
                    w[s] = ch.ask(&Ask::TrialWitnesses { person: p as u8 }) as i32;
                    b[s] = ch.ask(&Ask::TrialBribe { person: p as u8 }) as i32;
                    self.pay(p, w[s] + b[s]);
                } else {
                    let mut x = if s == 0 { 1000 + 500 * self.ri(10) + self.ri(1000) - 100 * n0 } else { self.ri(1000) + 1000 * n0 };
                    match self.persons[p].loc() {
                        0..=9 => x /= 3,
                        10..=12 => x = x * 2 / 3,
                        _ => {}
                    }
                    w[s] = x.clamp(0, 9999);
                    invested.push(p as u8);
                }
            }
        }
        let k0 = (w[0] / 1500).clamp(1, 6);
        let mut e = self.evidence_of(pl, d);
        let without = e == 0;
        if without {
            // table 0x84B4C: entry i = 1 << i (rules-39 §10)
            e = 1 << self.r(30);
        }
        let n = e.count_ones() as i32;
        let mut groups = vec![];
        for g in 0..3 {
            if (e >> (10 * g)) & 0x3FF != 0 {
                let c = self.r(5) as u8;
                let t = self.r(3) as u8;
                groups.push((g as u8, t, c));
            }
        }
        let q = if self.persons[d].human() {
            (ch.ask(&Ask::Plea) as i32).min(8)
        } else if without {
            5 + self.ri(4)
        } else {
            self.ri(9)
        };
        // rules-18 §6, §7 (option "Zeugen & Plädoyers"): presentation rolls that consume game RNG. Witnesses alternate
        // from the plaintiff's side (line, voice, head per witness), then the prosecution's three lines and the
        // defence's three lines.
        let mut bribes = vec![];
        let mut talk = None;
        if self.opt(OPT_WITNESSES) {
            let mut left = [k0 as usize, if kind == 1 { 1 } else { (w[1] / 1500).clamp(1, 6) as usize }];
            let mut side = 0;
            let mut witnesses = vec![];
            while left[0] > 0 || left[1] > 0 {
                if left[side] > 0 {
                    let line = self.r(if side == 0 { 8 } else { 9 }) as u8;
                    let voice = 3 + self.r(8) as u8;
                    let head = self.r(8) as u8;
                    witnesses.push((side as u8, voice, line, head));
                    left[side] -= 1;
                }
                side ^= 1;
            }
            let mut plea = [0u8; 6];
            for (slot, k) in [7, 6, 4, 3, 2, 2].into_iter().enumerate() {
                plea[slot] = self.r(k) as u8;
            }
            talk = Some(Event::TrialTalk { witnesses, plea });
        }
        if bribe {
            for s in 0..2 {
                let p = sides[s];
                let x = if self.persons[p].human() {
                    b[s]
                } else if s == 0 {
                    if n < 6 { k0 * 1300 + self.ri(1000) } else { 0 }
                } else if without {
                    0
                } else {
                    k0 * 1300 + self.ri(1000)
                };
                bribes.push((p as u8, x));
                self.pay(p, x);
                for &j in &seats {
                    self.add_cash(j, x / 3);
                }
                w[s] += x;
            }
        }
        let r1 = self.ri(500);
        let r2 = self.ri(3000);
        let pscore = w[0] + (1500 + r1) * n;
        let dscore = w[1] + if without { 7000 + r2 } else { 0 } + 2000 - 200 * q;
        let mut votes = [false; 3]; // true = innocent
        for j in 0..3 {
            if let Some(k) = (0..j).find(|&k| seats[k] == seats[j]) {
                votes[j] = votes[k];
                continue;
            }
            let judge = seats[j];
            let (mut a, mut bb) = (0, 0);
            for x in 0..PERSONS {
                if self.persons[x].used() && self.persons[judge].favour(x) > 100 {
                    // every blackmailer, computer ones too, gets the question (rules-40 §7)
                    match ch.ask(&Ask::BlackmailVerdict { by: x as u8, judge: judge as u8 }) {
                        0 => a += 1,
                        1 => bb += 1,
                        _ => {}
                    }
                }
            }
            votes[j] = if a != bb {
                a < bb
            } else if !self.persons[judge].human() {
                if judge == d {
                    true
                } else if judge == pl {
                    false
                } else {
                    self.persons[judge].favour(pl) as i32 * 100 + pscore < dscore + self.persons[judge].favour(d) as i32 * 100
                }
            } else {
                ch.ask(&Ask::JudgeVote { judge: judge as u8 }) == 1
            };
            if !self.persons[judge].human() {
                self.r(4);
            }
        }
        let guilty = votes.iter().filter(|&&v| v).count() < 2;
        if bribe {
            ev.push(Event::TrialMoney { invested, bribes });
        }
        ev.push(Event::TrialCharge { evidence: e, groups, plea: q as u8 });
        ev.extend(talk);
        ev.push(Event::Trial { plaintiff: pl as u8, defendant: d as u8, judges: [seats[0] as u8, seats[1] as u8, seats[2] as u8], kind, loc: self.persons[d].loc().min(13), votes });
        let harsh = self.globals.b(ga::HARSHNESS) as u32;
        let f = ((self.net_worth(d) as u32).wrapping_mul(n as u32).wrapping_mul(harsh + 150) / 4000) as i32;
        if guilty {
            let s = if self.persons[seats[0]].human() {
                ch.ask(&Ask::Sentence { defendant: d as u8 }).min(6) as i32
            } else {
                let mut s = self.ri((harsh / 21 + 1) as i32) + n - 1;
                if self.persons[d].title() == 0 && s == 3 {
                    s = 2;
                }
                s.min(6)
            };
            let s = if s == 3 && self.persons[d].title() == 0 { 2 } else { s };
            if !self.persons[seats[0]].human() {
                self.r(4); // the presiding judge's voice for the announcement (rules-18 §10 step 2)
            }
            let office = (self.persons[d].loc(), self.persons[d].idx());
            match s {
                0 => {
                    let l = self.persons[d].life() as u32 * 9 / 10 + 1;
                    self.persons[d].set_life(l as u8);
                }
                1 => self.pay(d, f),
                2 => {
                    if let Some(i) = self.heap.alloc(19, d as u8, NONE, 3) {
                        heap::w32(&mut self.heap.recs[i], 4, (f * 2 / 3) as u32);
                    }
                }
                3 => {
                    let t = self.persons[d].title() - 1;
                    self.persons[d].set_title(t);
                }
                4 => {
                    self.heap.alloc(20, d as u8, NONE, 1);
                }
                5 => {
                    self.persons[d].set_idx(15);
                    self.persons[d].set_loc(0);
                }
                _ => {
                    self.heap.alloc(21, d as u8, NONE, 1);
                }
            }
            ev.push(Event::Verdict { defendant: d as u8, guilty: true, sentence: s as u8, fine: f, office });
        } else {
            ev.push(Event::Verdict { defendant: d as u8, guilty: false, sentence: 0, fine: 0, office: (0, 15) });
        }
        let payer = if guilty { d } else { pl };
        if guilty
            && let Some(i) = self.heap.find(0, pl as u8, ANY)
                && self.heap.recs[i][4] == 7 {
                    let v = heap::r32(&self.heap.recs[i], 5) + 1;
                    heap::w32(&mut self.heap.recs[i], 5, v);
                }
        let c = (f / 5).max(100) + self.ri(100);
        self.pay(payer, c);
        for &j in &seats {
            self.add_cash(j, 3 * c / 10);
        }
        let insured = self.persons[payer].insurance() & 2 != 0;
        if insured {
            self.add_cash(payer, c);
        }
        ev.push(Event::TrialCosts { plaintiff: pl as u8, defendant: d as u8, judges: [seats[0] as u8, seats[1] as u8, seats[2] as u8], payer: payer as u8, amount: c, insured });
        for i in self.heap.all(6, ANY, ANY) {
            let r = &mut self.heap.recs[i];
            if r[4] as usize != d {
                continue;
            }
            let left = heap::r32(r, 0x0B) & !e;
            heap::w32(r, 0x0B, left);
            if left == 0 && (r[9] == 0 || r[1] as usize == pl) {
                self.heap.delete(i);
            }
        }
        if self.r(100) < 80 {
            self.retaliate(d, pl, 3);
        }
        self.heap.delete(ri);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rules-04b §3.2: an empty bench seats the plaintiff; §3.5 fines and record deletion.
    #[test]
    fn rules04b_s3_trial_runs_and_cleans_up() {
        let mut st = State::new_world(3);
        st.options &= !OPT_BRIBE_COURT;
        let (pl, d) = (0usize, 1usize); // city-0 Ratsherren
        for p in 0..PERSONS {
            if st.persons[p].used() && st.persons[p].has_office() && st.persons[p].idx() == 4 && st.persons[p].loc() < 10 {
                st.persons[p].set_idx(15);
            }
        }
        assert_eq!(st.court(d, pl), [pl, pl, pl]);
        st.spies(pl, d, 0);
        let c = st.spy_contract(pl, d).unwrap();
        heap::w32(&mut st.heap.recs[c], 0x0B, 0b111);
        let ri = st.heap.alloc(18, pl as u8, NONE, 0).unwrap();
        st.heap.recs[ri][4] = d as u8;
        let mut ev = vec![];
        st.trial(ri, &mut crate::Auto, &mut ev);
        assert!(!st.heap.live(ri));
        // the plaintiff judges alone: guilty
        assert!(ev.iter().any(|e| matches!(e, Event::Verdict { guilty: true, .. })));
        assert!(st.spy_contract(pl, d).is_none(), "evidence used up, plaintiff's contract deleted");
    }

    /// rules-18 §13: the option "Zeugen & Plädoyers" adds one witness per side (3 draws each) and six lines.
    #[test]
    fn rules18_s13_presentation_draws() {
        let steps = |on: bool| {
            let mut st = State::new_world(3);
            st.options &= !OPT_BRIBE_COURT;
            if on {
                st.options |= OPT_WITNESSES;
            } else {
                st.options &= !OPT_WITNESSES;
            }
            let (pl, d) = (0usize, 1usize);
            st.spies(pl, d, 0);
            let ri = st.heap.alloc(18, pl as u8, NONE, 0).unwrap();
            st.heap.recs[ri][4] = d as u8;
            let start = st.rng;
            st.trial(ri, &mut crate::Auto, &mut vec![]);
            let mut r = start;
            (0..10_000)
                .find(|_| {
                    if r == st.rng {
                        return true;
                    }
                    r.rand();
                    false
                })
                .unwrap()
        };
        assert_eq!(steps(true) - steps(false), 12);
    }

    /// rules-04 §5.4: the office byte check, the new record with 0xBCBE0, dropping an open one; no RNG.
    #[test]
    fn rules04_s5_4_file_trial() {
        let mut st = State::new_world(3);
        let (pl, d) = (0usize, 1usize);
        assert!(st.persons[pl].has_office() && st.persons[pl].loc() < 10);
        let rng = st.rng;
        let text = |e: Option<Event>| match e { Some(Event::MsgBox { text, rect, .. }) => (text, rect), e => panic!("{e:?}") };
        for (t, p, refused) in [(13, 12, true), (13, 13, false), (10, 9, true), (12, 10, false), (9, 0, false)] {
            st.persons[d].set_loc(t);
            st.persons[pl].set_loc(p);
            let e = text(st.file_trial(pl, d));
            assert_eq!(e, if refused { (0xBCB7C, [160, 160, 320, 160]) } else { (0xBCBE0, [160, 180, 320, 120]) }, "{t} {p}");
            assert_eq!(st.open_trial(pl, d, 0).is_some(), !refused);
            if !refused {
                assert_eq!(st.file_trial(pl, d), None, "an open trial is dropped");
                assert!(st.open_trial(pl, d, 0).is_none());
            }
        }
        assert_eq!(st.rng, rng);
    }
}
