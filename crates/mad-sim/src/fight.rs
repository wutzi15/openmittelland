//! Battle setup and outcome (docs/re/rules-07-castles-bandits-combat.md §5.3 - §5.6).

use crate::battle::Battle;
use crate::economy::value;
use crate::heap::{self, ANY};
use crate::state::*;
use crate::{Ask, Choices, Event};

const FIELD: [usize; 6] = [0, 1, 1, 2, 3, 4];
const CELLS: usize = 46 * 30;

struct Part {
    person: usize,
    home: Option<usize>,
    rec: Option<usize>,
}

/// The picture of the last battle with a human taking part; the app takes it after the sim call.
/// Presentation only, not saved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replay {
    /// battle code 0..5 (rules-07 §5.1)
    pub code: u8,
    pub field: u8,
    /// player colour 0..5 of the attacking and the defending side
    pub colours: [u8; 4],
    pub frames: Vec<crate::battle::Snap>,
    /// the battle song drawn at setup (group 0x1B, rules-39 §5)
    pub song: u8,
}

/// The after-battle report (rules-59): what the six pages print, in participant order.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Report {
    /// page 1 (§1): 0 repelled (`0xBBB4C`), 1 won (`0xBBBC4`), 2 three entrances held (`0xBBD0C`), 3 idle ran out (`0xBBC64`)
    pub verdict: u8,
    /// page 2 (§2): the conquest page was printed (fleet burned, or the occupy question/auto-occupy)
    pub conquest: bool,
    /// a castle/camp was taken; for a castle the feast numbers (pigs, musicians, adjective index, wine)
    pub occupied: Option<[u8; 4]>,
    /// the target's place `P+1` (castles and camps)
    pub loc: u8,
    /// the shipment's good (raids)
    pub good: u8,
    /// page 3 (§3): one row per participant on a side
    pub rows: Vec<ReportRow>,
    /// page 4 (§4): destroyed goods in units and destroyed carts (after the spared one)
    pub destroyed: (u32, u32),
    /// page 5 (§5): the defender's class-5 losses (carters or sailors)
    pub class5: u32,
    /// page 6 (§6): the premium paid by the attacker (0 = "Niemand fordert")
    pub premium: i64,
    /// castle damage line: (casemates lost, security after), only after a bombardment
    pub damage: Option<(u8, u8)>,
    /// raid insurance: Some(Some(amount)) paid, Some(None) refused for want of an escort
    pub insurance: Option<Option<i32>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ReportRow {
    pub person: u8,
    /// participant colour `0xDB7FC[p]` and base unit type `0xD9798[p]` (0 Söldner, 5 Räuber)
    pub colour: u8,
    pub base: u8,
    pub prisoners: u32,
    /// captured goods markers (printed as 0 for the defender)
    pub markers: u32,
    /// the goods' value captured (`amount · (good + 8)`), its own page when > 0
    pub gain: i64,
    /// men lost per class 0..3
    pub losses: [u32; 4],
}

/// The arguments of the intro captions of a castle/camp attack (rules-50 §1.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Intro {
    /// `P+1`, the target's location; for a raid (codes 0..3) the shipment's source branch location
    pub loc: u8,
    /// a raid's second `%o` in line 1: the shipment's destination (`rec+10`); unused for castles and camps
    pub to: u8,
    pub attacker: u8,
    pub defender: u8,
    /// troop word `0xB601C[k]`: 1 Räuber (attacker's home kind byte != 2), 0 Söldner
    pub troop: u8,
    /// the call-off roll stopped the attack (rules-50 §2): its text replaces line 4
    pub called_off: bool,
}

/// The question of a pre-fight window page (rules-52); see [`crate::Ask::PreFight`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parley {
    /// lines only; the answer is ignored (a click closes the window)
    None,
    /// 2881: the human defender's amount 0..99999 (0: "Niemals, ich gewinne!" or a right click)
    Ransom { defender: u8, attacker: u8 },
    /// 3069: 0 "Gewiss, he he!" fights (nothing paid), anything else withdraws and takes the offer
    Attack { attacker: u8 },
    /// 2888..2890: one row per helper (owner, troop word 0 Söldner / 1 Räuber), the briber's own without an item.
    /// Answer `1 + (h << 20 | amount)` pays row h; 0 ("Genug bestochen!", right click) ends
    Bribe { briber: u8, helpers: Vec<(u8, u8)> },
    /// 3142: 0 attacker, 1 defender, anything else stays out
    Side { helper: u8, attacker: u8, defender: u8 },
}

/// What is attacked.
pub enum Target {
    /// a shipment record (codes 0..3)
    Shipment(usize),
    /// a castle or camp property (codes 4, 5)
    Property(usize),
}

impl State {
    /// rules-51 §1.4: humans use their own colour (+0xF4), then in participant order each computer person the
    /// lowest colour 0..5 not held yet, one colour per person across slots (live A/B: Hohenfels' owner, red)
    fn part_colours(&self, parts: &[Part]) -> [u8; 4] {
        let own = |p: usize| self.persons[p].human().then(|| self.persons[p].colour());
        let mut col: Vec<Option<u8>> = parts.iter().map(|q| own(q.person)).collect();
        for i in 0..parts.len() {
            if col[i].is_none() {
                let v = (0..6).find(|c| !col.contains(&Some(*c))).unwrap_or(0);
                for j in 0..parts.len() {
                    if parts[j].person == parts[i].person {
                        col[j] = Some(v);
                    }
                }
            }
        }
        let mut out = [0u8; 4];
        for (o, c) in out.iter_mut().zip(col) {
            *o = c.unwrap_or(0);
        }
        out
    }

    fn troop_base(&self, prop: usize) -> u8 {
        if self.props[prop].kind() == KIND_CASTLE { 0 } else { 5 }
    }

    /// `battle(code, record)` for the order record `rec` (type 51 or 54) against `target`.
    pub fn battle(&mut self, code: u8, rec: usize, target: Target, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        let field = FIELD[code as usize];
        if self.szenario.len() < 5 * CELLS {
            ev.push(Event::BattlePending { code, order: rec as u16 });
            return;
        }
        let att = self.heap.recs[rec][1] as usize;
        let ahome = self.heap.recs[rec][2] as usize;
        let (def, dhome, value) = match target {
            Target::Shipment(s) => {
                let r = &self.heap.recs[s];
                (r[1] as usize, None, heap::r16(r, 5) as i64 * (r[7] as i64 + 8))
            }
            Target::Property(p) => (self.props[p].owner() as usize, Some(p), self.property_value(p)),
        };
        // rules-50 §1.3 step 2: the call-off roll right after the target lookup, before the helper draws
        let called_off = matches!(target, Target::Property(p) if self.r(200) <= self.props[p].b3() as u32);
        let mut parts = vec![Part { person: att, home: Some(ahome), rec: Some(rec) }, Part { person: def, home: dhome, rec: None }];
        // §5.3 helpers
        let pool: Vec<usize> = [53u8, 54]
            .iter()
            .flat_map(|&t| self.heap.all(t, ANY, ANY))
            .filter(|&i| i != rec && (code >= 4 || self.heap.recs[i][4] == self.heap.recs[rec][4]))
            .collect();
        let mut listed = vec![];
        for i in pool {
            if self.r(100) < 80 && listed.len() < 20 {
                listed.push(i);
            }
        }
        let mut helpers = vec![];
        if !listed.is_empty() {
            let n = listed.len() as u32;
            let first = self.r(n) as usize;
            helpers.push(listed[first]);
            if n > 1 {
                let second = (first + 1 + self.r(n - 1) as usize) % n as usize;
                if self.heap.recs[listed[second]][1] != self.heap.recs[listed[first]][1] {
                    helpers.push(listed[second]);
                }
            }
        }
        let mut b = Battle::new(field, &self.szenario[field * CELLS..(field + 1) * CELLS]);
        b.record = self.persons[att].human() || self.persons[def].human();
        // helpers joining change the table (one Cell: the pre-fight closure reads it)
        let colours = std::cell::Cell::new(self.part_colours(&parts));
        // deployment: defender first
        let (mut qty, g, carts, markers, escorted) = match target {
            Target::Shipment(s) => {
                let r = &self.heap.recs[s];
                let (qty, g, carts) = (heap::r16(r, 5) as u32, r[7] as usize, r[8] as u32);
                // rules-58 §1: no minimum on the deployed count; the max(1) only guards our loss divisions
                b.deploy(20, qty.div_ceil(100), 1, 1, 0xFF, 0);
                let markers = qty.div_ceil(100).max(1);
                if code <= 2 {
                    b.deploy(10, carts, 1, 1, 0xFF, 50);
                    b.deploy(21, carts, 1, 1, 0xFF, 0);
                } else {
                    b.deploy(11, (5 * carts) & 0xFF, 1, 1, 0xFF, 50);
                }
                let esc = r[4] & 1 != 0;
                if esc {
                    for k in 0..4u8 {
                        if k != 2 || code == 3 {
                            b.deploy(k, qty.div_ceil(250), 1, 1, 0xFF, 50);
                        }
                    }
                }
                // rules-58 §1: the leader is placed for every shipment, escorted or not
                b.deploy(4, 1, 1, 1, 4, 87);
                (qty, g, carts, markers, esc)
            }
            Target::Property(p) => {
                let base = self.troop_base(p);
                for k in 0..4 {
                    b.deploy(base + k as u8, self.props[p].troops(k) as u32, 1, 1, k as u8, self.props[p].training(k) as i32);
                }
                b.deploy(base + 4, 1, 1, 1, 4, 87);
                (0, 0, 0, 1, false)
            }
        };
        // the captions show over the defender deployment (rules-50 §1.2). A raid's line 1 reads
        // (`props[rec+2]+1`, `rec+10`) of the shipment (rules-50 §1.1, the line-1 push at 0x3F947).
        b.snap();
        let troop = (self.props[ahome].kind() != KIND_CASTLE) as u8;
        let (loc, to) = match target {
            Target::Property(p) => (self.props[p].loc(), self.props[p].loc()),
            Target::Shipment(s) => (self.props[self.heap.recs[s][2] as usize].loc(), self.heap.recs[s][10]),
        };
        let intro = Intro { loc, to, attacker: att as u8, defender: def as u8, troop, called_off };
        // music(0x1B) after the defender deployment, before the captions and the attacker (rules-50 §1.3 step 7)
        b.music(&mut self.rng);
        // the field and captions 1..3, then line 4 or the call-off text; no draws (rules-50 §1.3 step 9). The
        // ransom questions come after them.
        if b.record {
            ch.ask(&Ask::BattleIntro { code, colours: colours.get(), song: b.song, frames: b.take_new_frames(), intro });
        }
        if called_off {
            // rules-50 §2: nothing else happens; the record stays
            if let Target::Property(p) = target {
                ev.push(Event::AttackDetected { prop: p as u8, attacker: att as u8 });
            }
            return;
        }
        let abase = self.troop_base(ahome);
        let cap = if self.persons[att].human() { u32::MAX } else { ((value / 350).max(1)) as u32 };
        for k in 0..4 {
            let n = (self.heap.recs[rec][5 + k] as u32).min(cap);
            b.deploy(abase + k as u8, n, 0, 0, k as u8, self.props[ahome].training(k) as i32);
        }
        // bug [C]: the leader's training is read from +0x60, the next property's kind byte
        let lt = self.props.get(ahome + 1).map_or(0, |p| p.kind()) as i32;
        b.deploy(abase + 4, 1, 0, 0, 4, lt);
        // rules-52: from here to the fight everything prints into one window over the deployed field. The computer
        // sides' lines travel with the next question, the rest with a closing `Parley::None`.
        use crate::WArg::{Num, P, Word};
        let mut lines: Vec<crate::Line> = vec![];
        if let Target::Shipment(s) = target {
            // §0.2: the raid caption "Information" first
            let g = self.heap.recs[s][7];
            lines.push((if g != 14 { 0xBBA04 } else { 0xBBA48 }, vec![P(def as u8), crate::WArg::Good(g), Num(value as i32)]));
        }
        b.snap();
        let mut pre = |b: &mut Battle, lines: &mut Vec<_>, q: Parley| ch.ask(&Ask::PreFight { code, colours: colours.get(), song: b.song, frames: b.take_new_frames(), lines: std::mem::take(lines), q });
        let (pa, pd) = (P(att as u8), P(def as u8));
        // §1 the defender's offer
        let s = b.strength_of(1);
        let mut offer = if self.persons[def].human() {
            pre(&mut b, &mut lines, Parley::Ransom { defender: def as u8, attacker: att as u8 }).min(99_999) as i32
        } else if s >= 3 {
            // factor s − 1 (rules-52 §1 corrects rules-07 §5.5's 2)
            let o = ((80 + self.r(40)) as i64 * value * (s as i64 - 1) / 400 + 1) as i32;
            lines.push((0xA9B11, vec![pd.clone(), pa.clone(), Num(o)]));
            o
        } else {
            lines.push(if s < 2 { (0xA9B64, vec![pd.clone(), Word(0xB6F68, self.persons[def].gender() & 1)]) } else { (0xA9BA5, vec![pd.clone()]) });
            0
        };
        // §2 the attacker's decision: a human fights on item 0 (nothing paid); a computer prints 3072/3071/3070
        let fight = if self.persons[att].human() {
            let f = pre(&mut b, &mut lines, Parley::Attack { attacker: att as u8 }) == 0;
            if f {
                offer = 0;
            }
            f
        } else if offer == 0 {
            lines.push((0xAD549, vec![pa.clone()]));
            true
        } else if value * self.r(100) as i64 > offer as i64 * 200 {
            lines.push((0xAD4F6, vec![pa.clone()]));
            true
        } else {
            lines.push((0xAD497, vec![pa.clone(), Word(0xB601C, (self.props[ahome].kind() != KIND_CASTLE) as u8)]));
            false
        };
        self.pay(def, offer);
        self.add_cash(att, offer);
        if !fight {
            // history code 8: a human attacker let mercy go before right ("Gnade vor Recht", rules-41 §2)
            if self.persons[att].human() {
                self.history(att, 8, field as u8, 0);
            }
            if b.record && !lines.is_empty() {
                pre(&mut b, &mut lines, Parley::None);
            }
            ev.push(Event::RansomPaid { attacker: att as u8, amount: offer });
            return;
        }
        // §3 helper bribes and sides
        let n = helpers.len();
        let owner = |st: &State, h: usize| st.heap.recs[helpers[h]][1] as usize;
        // troop word: a type-53 record's men are Söldner (0), else Räuber (1)
        let word = |st: &State, h: usize| (st.heap.recs[helpers[h]][0] != 53) as u8;
        if n > 0 {
            lines.push((0xA9BE3, vec![]));
        }
        let mut bribes = vec![[0i64; 2]; n];
        for (side, briber) in [att, def].into_iter().enumerate() {
            if n == 0 {
                break;
            }
            let mut commit = |st: &mut State, h: usize, x: i64| {
                let ho = owner(st, h);
                bribes[h][side] += x;
                st.pay(briber, x as i32);
                st.add_cash(ho, x as i32);
                let f = (x * 400 / st.net_worth(ho).max(100) as i64).min(100) as i32;
                // doubled for a human in byte arithmetic (`ADD AL,AL`): wraps above 127
                st.favour_increase(briber, ho, if st.persons[briber].human() { (2 * f) as u8 as i8 as i32 } else { f });
            };
            if self.persons[briber].human() {
                // one menu: per helper "… Taler für …" (commit) and its spinner; "Genug bestochen!" or a right click ends
                let rows: Vec<(u8, u8)> = (0..n).map(|h| (owner(self, h) as u8, word(self, h))).collect();
                loop {
                    let a = pre(&mut b, &mut lines, Parley::Bribe { briber: briber as u8, helpers: rows.clone() });
                    let h = (a.wrapping_sub(1) >> 20) as usize;
                    if a == 0 || h >= n {
                        break;
                    }
                    if owner(self, h) != briber {
                        commit(self, h, ((a - 1) & 0xF_FFFF).min(99_999) as i64);
                    }
                }
            } else {
                for h in 0..n {
                    let ho = owner(self, h);
                    let x = value / (4 * n as i64) + self.r(100) as i64;
                    // rules-07 §5.5 quirk: tests participant slot h (attacker, defender), not the helper's owner
                    let slot = [att, def].get(h).copied();
                    let x = if b.strength_of(side as u8) <= 1 || x < 500 || slot == Some(briber) || slot.is_some_and(|p| self.persons[p].human()) { 0 } else { x };
                    let (bw, hw, tw) = (P(briber as u8), P(ho as u8), Word(0xB601C, word(self, h)));
                    lines.push(match (ho == briber, x != 0) {
                        (true, _) => (0xBBA98, vec![bw]),
                        (_, true) => (0xA9C9C, vec![bw, hw, tw]),
                        _ => (0xA9CDC, vec![bw, hw, tw]),
                    });
                    commit(self, h, x);
                }
            }
        }
        for (hi, &hr) in helpers.iter().enumerate() {
            let ho = self.heap.recs[hr][1] as usize;
            let side = if self.persons[ho].human() {
                // 3142: no outcome line follows
                match pre(&mut b, &mut lines, Parley::Side { helper: ho as u8, attacker: att as u8, defender: def as u8 }) {
                    0 => Some(0),
                    1 => Some(1),
                    _ => None,
                }
            } else {
                let side = if ho == att {
                    Some(0)
                } else if ho == def {
                    Some(1)
                } else {
                    let (fa, fd) = (self.persons[ho].favour(att) as i64, self.persons[ho].favour(def) as i64);
                    if fa > fd && fa > 50 && bribes[hi][0] > value / 7 {
                        Some(0)
                    } else if fd > fa && fd > 45 && bribes[hi][1] > value / 7 {
                        Some(1)
                    } else {
                        None
                    }
                };
                let hw = P(ho as u8);
                lines.push(match side {
                    Some(k) if [att, def][k] == ho => (0xBBAE0, vec![hw]),
                    Some(k) => (0xAF323, vec![hw, P([att, def][k] as u8)]),
                    None => (0xAF36D, vec![hw]),
                });
                side
            };
            if let Some(s) = side {
                let p = parts.len() as u8;
                let home = self.heap.recs[hr][2] as usize;
                let base = self.troop_base(home);
                for k in 0..4 {
                    b.deploy(base + k as u8, self.heap.recs[hr][5 + k] as u32, p, s as u8, k as u8, self.props[home].training(k) as i32);
                }
                parts.push(Part { person: ho, home: Some(home), rec: Some(hr) });
                colours.set(self.part_colours(&parts));
            }
        }
        if b.record && !lines.is_empty() {
            pre(&mut b, &mut lines, Parley::None);
        }
        // §5.6 fight
        b.human[0] = self.persons[att].human();
        b.human[1] = self.persons[def].human();
        let mut told = [false; 2];
        b.fight_with(&mut self.rng, &mut |b, round, start| {
            for part in 0..2u8 {
                let groups = b.groups(part);
                if start && b.human.get(part as usize).copied().unwrap_or(false) && !b.leader_alive(part & 1) && !groups.is_empty() && !told[part as usize] {
                    told[part as usize] = true;
                    ch.ask(&Ask::LeaderLost { person: if part == 0 { att } else { def } as u8 });
                }
                // the hotkey is polled during each activation and its menu opens when the activation ends; before
                // round 0 only the deployment draws poll it (rules-44 §2). The app answers these without a dialog
                // unless the hotkey was pressed.
                if !b.controls(part) || groups.is_empty() || (start && round > 0) {
                    continue;
                }
                let frames = b.take_new_frames();
                let a = ch.ask(&Ask::Tactics { part, person: if part == 0 { att } else { def } as u8, round, code, colours: colours.get(), groups: groups.clone(), frames, song: b.song });
                if a == u32::MAX {
                    b.human = [false; 4];
                    return;
                }
                for (k, g) in groups.iter().enumerate().take(8) {
                    if let Some(t) = ((a >> (4 * k)) & 0xF).checked_sub(1) {
                        b.set_tactic(part, g.typ, t as u8);
                    }
                }
            }
        });
        if b.record {
            self.replays.push(Replay { code, field: field as u8, colours: colours.get(), frames: std::mem::take(&mut b.trace), song: b.song });
        }
        let conquest = (b.held > 2 || !b.defender_alive()) && !b.undecided;
        if self.globals.forbidden(1, 0) {
            // rules-59 §1: every participant on side 0, whatever is left of his men
            for (pi, p) in parts.iter().enumerate() {
                if pi == 0 || b.men.iter().any(|m| m.part as usize == pi && m.side == 0) {
                    crate::laws::add_evidence(self, p.person, 0x400);
                }
            }
        }
        if self.r(100) < 50 {
            self.retaliate(def, att, 4);
        }
        let alive = b.alive();
        let mut rep = Report {
            verdict: if alive[0] == 0 { 0 } else if alive[1] == 0 { 1 } else if !b.undecided { 2 } else { 3 },
            conquest: conquest && (code >= 3),
            good: g as u8,
            ..Report::default()
        };
        let mut occupy = false;
        match target {
            Target::Shipment(s) if conquest && code == 3 => self.heap.delete(s),
            Target::Property(p) if conquest => {
                rep.loc = self.props[p].loc();
                occupy = !self.persons[att].human() || ch.ask(&Ask::OccupyCastle { code, attacker: att as u8, defender: def as u8, loc: rep.loc, verdict: rep.verdict }) == 0;
                if occupy {
                    // rules-59 §2: the castle feast draws wine, adjective, musicians, pigs
                    rep.occupied = Some(if code == 4 {
                        let wine = self.r(10) as u8 + 2;
                        let adj = self.r(11) as u8;
                        let mus = self.r(30) as u8 + 2;
                        [self.r(20) as u8 + 2, mus, adj, wine]
                    } else {
                        [0; 4]
                    });
                }
            }
            Target::Property(p) => rep.loc = self.props[p].loc(),
            _ => {}
        }
        if let Target::Shipment(_) = target {
            rep.destroyed.0 = qty * b.markers_lost as u32 / markers;
            qty -= rep.destroyed.0;
        }
        let mut wl = b.wagons_lost as u32;
        if carts > 0 && wl >= carts {
            wl = carts - 1;
        }
        rep.destroyed.1 = wl;
        let mut captured_total = 0u32;
        for (pi, p) in parts.iter().enumerate() {
            let pr = b.prisoners[pi.min(3)];
            let transport_def = pi == 1 && matches!(target, Target::Shipment(_));
            if pr > 0 && self.persons[p.person].human() && !transport_def {
                let key = p.home.unwrap_or(0) as u8;
                let i = match self.heap.find(55, p.person as u8, key) {
                    Some(i) => i,
                    None => self.heap.alloc(55, p.person as u8, key, 1).unwrap_or(usize::MAX),
                };
                if i != usize::MAX {
                    self.heap.recs[i][4] = self.heap.recs[i][4].saturating_add(pr as u8);
                }
            }
            let c = b.banners[pi.min(3)];
            let mut row = ReportRow {
                person: p.person as u8,
                colour: colours.get()[pi.min(3)],
                base: p.home.map_or(0, |h| self.troop_base(h)),
                prisoners: pr.max(0) as u32,
                markers: if pi == 1 { 0 } else { c.max(0) as u32 },
                ..ReportRow::default()
            };
            if pi != 1 && c > 0 && g < GOODS {
                let amount = (c as u32 * qty / markers).min(qty);
                if let Some(h) = p.home {
                    self.add_goods(h, g, amount as u16);
                }
                if let Some(i) = self.heap.find(0, p.person as u8, ANY)
                    && matches!(self.heap.recs[i][4], 19 | 29)
                {
                    let v = heap::r32(&self.heap.recs[i], 5).wrapping_add(amount * (g as u32 + 8));
                    heap::w32(&mut self.heap.recs[i], 5, v);
                }
                qty -= amount;
                captured_total += amount;
                row.gain = amount as i64 * (g as i64 + 8);
            }
            for (cl, l) in row.losses.iter_mut().enumerate() {
                *l = b.men.iter().filter(|m| m.part as usize == pi && crate::battle::class(m.typ) as usize == cl && m.typ < 12 && (m.fallen || m.captured)).count() as u32;
            }
            if pi == 1 {
                rep.class5 = b.men.iter().filter(|m| m.part == 1 && m.typ < 12 && crate::battle::class(m.typ) == 5 && (m.fallen || m.captured)).count() as u32;
            }
            rep.rows.push(row);
            // losses per troop slot
            for k in 0..4u8 {
                let lost = b.men.iter().filter(|m| m.part as usize == pi && m.slot == k && (m.fallen || m.captured)).count() as u8;
                if lost == 0 {
                    continue;
                }
                if let Some(r) = p.rec {
                    let v = self.heap.recs[r][5 + k as usize].saturating_sub(lost);
                    self.heap.recs[r][5 + k as usize] = v;
                } else if let Some(h) = p.home {
                    let v = self.props[h].troops(k as usize).saturating_sub(lost);
                    self.props[h].set_troops(k as usize, v);
                }
            }
        }
        if let Target::Shipment(s) = target
            && self.heap.live(s)
        {
            heap::w16(&mut self.heap.recs[s], 5, qty as u16);
            self.heap.recs[s][8] = (carts - wl) as u8;
        }
        // robber premium (rules-07 §5.6, re-checked by RE): the attacker pays value * R / 80, R =
        // troops left in his order after losses (undeployed ones included), every held battle,
        // undecided too; the money leaves the game (H p.33 warns about it).
        let left: i64 = (5..9).map(|k| self.heap.recs[rec][k] as i64).sum();
        rep.premium = value * left / 80;
        self.pay(att, rep.premium as i32);
        if let Target::Property(p) = target {
            let bb = b.bombard as u32;
            let c4 = self.props[p].b4() as u32;
            let mut l4 = (bb / 4).min(c4);
            if l4 == 1 {
                l4 = 0;
            }
            self.props[p].set_b4((c4 - l4) as u8);
            let c3 = self.props[p].b3() as u32;
            self.props[p].set_b3((c3 - (bb / 3).min(c3)) as u8);
            if code >= 4 && bb > 0 {
                rep.damage = Some((l4 as u8, self.props[p].b3()));
            }
            if occupy {
                self.take_castle(p, att, ahome, rec);
            }
        }
        let insured = (captured_total as i32) * (value_of(g) + 4);
        if code < 4 && insured > 0 && self.persons[def].insurance() & 4 != 0 {
            // rules-59 §6.4: paid only with an escort, else the refusal line
            rep.insurance = Some(escorted.then_some(insured));
            if escorted {
                self.add_cash(def, insured);
            }
        }
        ev.push(Event::Battle { code, attacker: att as u8, defender: def as u8, conquest: conquest && (occupy || code == 3), undecided: b.undecided, report: Box::new(rep) });
    }

    fn take_castle(&mut self, p: usize, att: usize, ahome: usize, rec: usize) {
        self.props[p].set_owner(att as u8);
        for k in 0..4 {
            self.props[p].set_troops(k, 0);
            let t = self.props[ahome].training(k);
            self.props[p].set_training(k, t);
        }
        let cap = self.props[p].b4() as u32;
        let mut total = 0;
        let mut k = 0;
        while total < cap && (5..9).any(|j| self.heap.recs[rec][j] > 0) {
            if self.heap.recs[rec][5 + k] > 0 {
                self.heap.recs[rec][5 + k] -= 1;
                let v = self.props[p].troops(k) + 1;
                self.props[p].set_troops(k, v);
                total += 1;
            }
            k = (k + 1) % 4;
        }
        for i in self.heap.all(17, att as u8, p as u8) {
            self.heap.delete(i);
        }
        if let Some(i) = self.heap.find(16, ANY, p as u8) {
            self.heap.delete(i);
        }
    }
}

fn value_of(g: usize) -> i32 {
    if g < GOODS { value(g) } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rules-51 §1.4: humans keep their colour; computers take the lowest free colour in participant order, one per person.
    #[test]
    fn rules51_s1_4_part_colours() {
        let mut st = State::new_world(2);
        let all: Vec<usize> = st.computers().take(3).collect();
        let (h, c) = (vec![all[0]], vec![all[1], all[2]]);
        st.persons[h[0]].set_flags10(0x10);
        st.persons[h[0]].set_colour(0);
        let part = |p| Part { person: p, home: None, rec: None };
        // human attacker 0; computer defender → 1; a helper of the defender's person shares 1; another computer → 2
        assert_eq!(st.part_colours(&[part(h[0]), part(c[0]), part(c[0]), part(c[1])]), [0, 1, 1, 2]);
        st.persons[h[0]].set_colour(1);
        assert_eq!(st.part_colours(&[part(c[0]), part(h[0])]), [0, 1, 0, 0]);
    }

    /// rules-07 §5: a raid on a shipment runs to an end with a test field and changes the shipment.
    #[test]
    fn rules07_s5_raid_battle() {
        let mut st = State::new_world(2);
        let mut f = vec![0u8; 5 * CELLS];
        let base = crate::battle::tests::field();
        // field 0: attacker anchors at code 8 (class 0) and defender markers/carters: add slots
        let mut f0 = base.clone();
        f0[12 * 30 + 20] = 4; // goods marker slot
        f0[14 * 30 + 20] = 5; // wagon slot
        f0[12 * 30 + 5] = 19; // carters anchor (class 5, side 1)
        f0[16 * 30 + 20] = 18; // leader anchor (class 4, side 1)
        f[..CELLS].copy_from_slice(&f0);
        st.szenario = f;
        let camp = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CAMP).unwrap();
        let att = st.props[camp].owner();
        let r = st.heap.alloc(54, att, camp as u8, 1).unwrap();
        st.heap.recs[r][5] = 5;
        let s = st.heap.alloc(9, 3, 0, 1).unwrap();
        heap::w16(&mut st.heap.recs[s], 5, 100);
        st.heap.recs[s][7] = 0;
        st.heap.recs[s][8] = 1;
        st.heap.recs[s][10] = 7;
        let mut ev = vec![];
        st.battle(0, r, Target::Shipment(s), &mut crate::Auto, &mut ev);
        assert!(ev.iter().any(|e| matches!(e, Event::Battle { .. } | Event::RansomPaid { .. })), "{ev:?}");
        // rules-50 §1.1: with a human side the raid opens with its captions; line 1 names (source branch, destination)
        let s = st.heap.alloc(9, 3, 0, 1).unwrap();
        heap::w16(&mut st.heap.recs[s], 5, 100);
        st.heap.recs[s][8] = 1;
        st.heap.recs[s][10] = 7;
        st.persons[3].set_flags10(0x10);
        let mut ch = Script { answers: vec![], asked: vec![] };
        st.battle(0, r, Target::Shipment(s), &mut ch, &mut vec![]);
        let Some(crate::Ask::BattleIntro { code: 0, intro, frames, .. }) = ch.asked.first() else { panic!("{:?}", ch.asked) };
        assert_eq!((intro.loc, intro.to, intro.defender, intro.called_off), (st.props[0].loc(), 7, 3, false));
        assert!(frames[0].men.iter().all(|m| m.side == 1), "defender only");
        assert!(frames[0].men.iter().any(|m| m.typ == 4), "an unescorted shipment has its leader (rules-58 §1)");
    }

    /// `$MAD_DATA`: castle attacks and raids on every real SZENARIO.DAT field run to an end over
    /// several years of play.
    #[test]
    fn real_fields_battles() {
        let Some(d) = std::env::var("MAD_DATA").ok().and_then(|p| std::fs::read(std::path::Path::new(&p).join("SZENARIO.DAT")).ok()) else { return };
        let mut st = State::new_game(9, 3, &[crate::world::HumanSetup::test(4)]).unwrap();
        st.szenario = d;
        let h = st.current as usize;
        // give the human a camp and attack every castle from it
        let camp = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CAMP).unwrap();
        st.props[camp].set_owner(h as u8);
        let mut n = 0;
        for target in (0..PROPS).filter(|&p| matches!(st.props[p].kind(), KIND_CASTLE | KIND_CAMP) && p != camp).collect::<Vec<_>>() {
            let r = st.heap.alloc(51, h as u8, camp as u8, 1).unwrap();
            st.heap.recs[r][4] = st.props[target].loc() - 14;
            for k in 5..9 {
                st.heap.recs[r][k] = 6;
            }
            let code = if st.props[target].kind() == KIND_CASTLE { 4 } else { 5 };
            let mut ev = vec![];
            st.battle(code, r, Target::Property(target), &mut crate::Auto, &mut ev);
            n += ev.iter().filter(|e| matches!(e, Event::Battle { .. } | Event::AttackDetected { .. })).count();
        }
        assert_eq!(n, 6);
        for _ in 0..30 {
            st.apply(crate::Command::EndTurn, &mut crate::Auto);
        }
    }

    /// rules-50 §1.3, §2: the call-off roll comes right after the target lookup, before the helper `rand(100)`s; a
    /// called-off attack still draws the helper picks and the music `rand(3)`, and keeps the defender-only frame.
    #[test]
    fn rules50_calloff_roll_before_helpers() {
        let mut st = State::new_game(5, 3, &[crate::world::HumanSetup::test(4)]).unwrap();
        st.szenario = std::env::var("MAD_DATA").ok().and_then(|p| std::fs::read(std::path::Path::new(&p).join("SZENARIO.DAT")).ok()).unwrap_or(vec![0; 5 * CELLS]);
        let h = st.current as usize;
        let camp = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CAMP).unwrap();
        st.props[camp].set_owner(h as u8);
        let castle = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CASTLE).unwrap();
        for k in 0..4u8 {
            st.heap.alloc(53, 20 + k, castle as u8, 1).unwrap();
        }
        let r = st.heap.alloc(51, h as u8, camp as u8, 1).unwrap();
        st.heap.recs[r][4] = st.props[castle].loc() - 14;
        st.heap.recs[r][5] = 4;
        let pool = [53u8, 54].iter().map(|&t| st.heap.all(t, ANY, ANY).len()).sum::<usize>();
        let mut rng = st.rng;
        let roll = rng.n(200);
        let listed = (0..pool).filter(|_| rng.n(100) < 80).count().min(20) as u32;
        if listed > 0 {
            rng.n(listed);
        }
        if listed > 1 {
            rng.n(listed - 1);
        }
        let song = [0x10, 6, 3][rng.n(3) as usize];
        for b3 in [roll, roll.wrapping_sub(1)].into_iter().filter(|&b| b < 200) {
            let mut s = st.clone();
            s.props[castle].set_b3(b3 as u8);
            let mut ev = vec![];
            let mut ch = Script { answers: vec![], asked: vec![] };
            s.battle(4, r, Target::Property(castle), &mut ch, &mut ev);
            let off = b3 == roll;
            assert_eq!(ev.iter().any(|e| matches!(e, Event::AttackDetected { .. })), off, "b3 {b3} roll {roll}");
            if off {
                assert_eq!(s.rng.s, rng.s, "roll, helper draws and music, nothing else");
                assert!(s.heap.live(r), "the record stays");
                let Some(crate::Ask::BattleIntro { frames, song: sg, intro, .. }) = ch.asked.first() else { panic!("{:?}", ch.asked) };
                assert_eq!((frames.len(), *sg, intro.called_off, intro.troop), (1, song, true, 1));
                assert!(frames[0].men.iter().all(|m| m.side == 1), "defender only");
                assert_eq!(ch.asked.len(), 1, "no other question");
            }
        }
    }

    /// Records every question and answers from a script (then 0).
    struct Script {
        answers: Vec<u32>,
        asked: Vec<crate::Ask>,
    }
    impl crate::Choices for Script {
        fn ask(&mut self, q: &crate::Ask) -> u32 {
            self.asked.push(q.clone());
            self.answers.get(self.asked.len() - 1).copied().unwrap_or(0)
        }
    }

    fn castle_attack(seed: u32, answers: Vec<u32>) -> (State, Vec<Event>, Script) {
        attack_as(seed, answers, true)
    }

    /// A castle attack on a weak garrison (`human_attacks`: the human attacks from a camp, else a computer camp owner
    /// attacks the human's castle); `None` without `$MAD_DATA`.
    fn attack_setup(seed: u32, human_attacks: bool) -> Option<(State, usize, usize)> {
        let d = std::env::var("MAD_DATA").ok().and_then(|p| std::fs::read(std::path::Path::new(&p).join("SZENARIO.DAT")).ok())?;
        let mut st = State::new_game(seed, 3, &[crate::world::HumanSetup::test(4)]).unwrap();
        st.szenario = d;
        let h = st.current as usize;
        let camp = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CAMP).unwrap();
        let castle = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CASTLE).unwrap();
        if human_attacks {
            st.props[camp].set_owner(h as u8);
        } else {
            st.props[castle].set_owner(h as u8);
            if st.persons[st.props[camp].owner() as usize].human() {
                st.props[camp].set_owner(((h + 1) % 4) as u8);
            }
        }
        // a weak castle so the defender is outnumbered (strength 3)
        for k in 0..4 {
            st.props[castle].set_troops(k, 1);
        }
        st.props[castle].set_b3(0); // no detection: rand(200) <= 0 only on 0
        let r = st.heap.alloc(51, st.props[camp].owner(), camp as u8, 1).unwrap();
        st.heap.recs[r][4] = st.props[castle].loc() - 14;
        for k in 5..9 {
            st.heap.recs[r][k] = 3;
        }
        Some((st, r, castle))
    }

    fn attack_as(seed: u32, answers: Vec<u32>, human_attacks: bool) -> (State, Vec<Event>, Script) {
        let mut ch = Script { answers, asked: vec![] };
        let Some((mut st, r, castle)) = attack_setup(seed, human_attacks) else {
            let mut st = State::new_game(seed, 3, &[crate::world::HumanSetup::test(4)]).unwrap();
            st.szenario = vec![];
            return (st, vec![], ch);
        };
        let mut ev = vec![];
        st.battle(4, r, Target::Property(castle), &mut ch, &mut ev);
        (st, ev, ch)
    }

    fn parleys(ch: &Script) -> Vec<(Vec<crate::Line>, Parley)> {
        ch.asked.iter().filter_map(|q| if let crate::Ask::PreFight { lines, q, .. } = q { Some((lines.clone(), q.clone())) } else { None }).collect()
    }

    /// rules-52 §1, §2: a computer defender at strength 3 offers `(80 + rand(40)) · value · 2 / 400 + 1` (text 2882
    /// before 3069); "Gewiss, he he!" (0) fights and pays nothing, "Gnade" (1) withdraws, takes the offer and writes
    /// history code 8.
    #[test]
    fn rules52_offer_and_human_attacker() {
        for seed in 1..40 {
            let Some((st, r, castle)) = attack_setup(seed, true) else { return };
            let (att, def) = (st.heap.recs[r][1] as usize, st.props[castle].owner() as usize);
            let (_, ev, ch) = attack_as(seed, vec![0, 1], true);
            if ev.iter().any(|e| matches!(e, Event::AttackDetected { .. })) {
                continue;
            }
            let ps = parleys(&ch);
            let Some((lines, Parley::Attack { attacker })) = ps.first() else { panic!("{ps:?}") };
            assert_eq!(*attacker as usize, att);
            let [(0xA9B11, args)] = lines.as_slice() else { continue };
            let crate::WArg::Num(offer) = args[2] else { panic!() };
            assert_eq!(args[..2], [crate::WArg::P(def as u8), crate::WArg::P(att as u8)]);
            let v = st.property_value(castle);
            assert!((80 * v * 2 / 400 + 1..=119 * v * 2 / 400 + 1).contains(&(offer as i64)), "{offer} {v}");
            // withdraw: paid, history, no fight
            let (w, ev, _) = attack_as(seed, vec![0, 1], true);
            assert!(ev.contains(&Event::RansomPaid { attacker: att as u8, amount: offer }), "{ev:?}");
            assert_eq!(w.persons[att].cash(), st.persons[att].cash() + offer);
            assert_eq!(w.persons[def].cash(), st.persons[def].cash() - offer);
            assert!(!ev.iter().any(|e| matches!(e, Event::Battle { .. })));
            // fight: nothing paid
            let (_, ev, _) = attack_as(seed, vec![0, 0], true);
            assert!(ev.iter().any(|e| matches!(e, Event::Battle { .. })), "{ev:?}");
            assert!(!ev.iter().any(|e| matches!(e, Event::RansomPaid { .. })));
            return;
        }
        if std::env::var("MAD_DATA").is_ok() {
            panic!("no strength-3 offer in 39 attacks");
        }
    }

    /// rules-52 §1, §2: a human defender names the amount; the computer attacker's outcome is 3072 (offer 0, fight),
    /// 3071 (takes the money, fights) or 3070 (withdraws), printed after the question; the offer always moves.
    #[test]
    fn rules52_human_defender_computer_attacker() {
        let mut seen = [false; 3];
        for seed in 1..40 {
            for amount in [0u32, 1, 99_999] {
                let Some((st, r, castle)) = attack_setup(seed, false) else { return };
                let (att, def) = (st.heap.recs[r][1] as usize, st.props[castle].owner() as usize);
                let (w, ev, ch) = attack_as(seed, vec![0, amount], false);
                if ev.iter().any(|e| matches!(e, Event::AttackDetected { .. })) {
                    break;
                }
                let ps = parleys(&ch);
                assert!(matches!(ps.first(), Some((l, Parley::Ransom { .. })) if l.is_empty()), "{ps:?}");
                let (lines, _) = &ps[1];
                let (addr, args) = &lines[0];
                assert_eq!(args[0], crate::WArg::P(att as u8));
                let withdrew = ev.iter().any(|e| matches!(e, Event::RansomPaid { .. }));
                let k = match (*addr, amount) {
                    (0xAD549, 0) => 0,
                    (0xAD4F6, 1..) => 1,
                    (0xAD497, 1..) => {
                        assert_eq!(args[1], crate::WArg::Word(0xB601C, 1), "a camp's Räuber");
                        2
                    }
                    _ => panic!("{addr:#x} {amount}"),
                };
                assert_eq!(withdrew, k == 2);
                if k == 2 {
                    assert_eq!(w.persons[def].cash(), st.persons[def].cash() - amount as i32);
                }
                seen[k] = true;
            }
        }
        if std::env::var("MAD_DATA").is_ok() {
            assert_eq!(seen, [true, true, true], "3072, 3071, 3070");
        }
    }

    /// rules-52 §3: with helpers the header 2887 comes before the human briber's menu; a commit pays the row's owner;
    /// the computer defender's bribe lines (2891/2892/0xBBA98) and the computer owners' side lines
    /// (0xBBAE0/3143/3144) close the window, one per helper each.
    #[test]
    fn rules52_helpers() {
        for seed in 1..40 {
            let Some((mut st, r, castle)) = attack_setup(seed, true) else { return };
            for k in 0..4u8 {
                st.heap.alloc(53, 20 + k, castle as u8, 1).unwrap();
            }
            let att = st.heap.recs[r][1] as usize;
            // intro, fight, pay row 0 500 Taler, then "Genug"
            let mut ch = Script { answers: vec![0, 0, 1 + 500, 0], asked: vec![] };
            let mut ev = vec![];
            let mut w = st.clone();
            w.battle(4, r, Target::Property(castle), &mut ch, &mut ev);
            let ps = parleys(&ch);
            let Some(k) = ps.iter().position(|p| matches!(p.1, Parley::Bribe { .. })) else { continue };
            let Parley::Bribe { briber, helpers } = &ps[k].1 else { unreachable!() };
            assert_eq!((*briber as usize, ps[k].0.as_slice()), (att, [(0xA9BE3, vec![])].as_slice()));
            assert_eq!(ps[k + 1].1, ps[k].1, "the menu comes back after a commit");
            let h0 = helpers[0].0 as usize;
            assert_eq!(w.persons[h0].cash() - st.persons[h0].cash(), 500, "paid on commit");
            let (lines, q) = ps.last().unwrap();
            assert_eq!(*q, Parley::None);
            let n = helpers.len();
            assert_eq!(lines.len(), 2 * n, "{lines:x?}");
            assert!(lines[..n].iter().all(|l| [0xA9C9C, 0xA9CDC, 0xBBA98].contains(&l.0)), "{lines:x?}");
            assert!(lines[n..].iter().all(|l| [0xBBAE0, 0xAF323, 0xAF36D].contains(&l.0)), "{lines:x?}");
            return;
        }
        if std::env::var("MAD_DATA").is_ok() {
            panic!("no helpers in 39 attacks");
        }
    }

    /// rules-07 §5.5.1: "Ransom (raids and undetected attacks)": a castle attack that passed the
    /// detection roll (§5.3) is an undetected attack, so a human attacker is asked (3069).
    #[test]
    fn rules07_s5_5_ransom_on_undetected_castle_attack() {
        for seed in 1..40 {
            let (_, ev, ch) = castle_attack(seed, vec![]);
            if ev.iter().any(|e| matches!(e, Event::AttackDetected { .. })) || ev.is_empty() {
                continue;
            }
            if parleys(&ch).iter().any(|p| matches!(p.1, Parley::Attack { .. })) {
                return;
            }
        }
        if std::env::var("MAD_DATA").is_ok() {
            panic!("no attack question in 39 undetected castle attacks against a weak garrison");
        }
    }

    /// The app re-runs a command from a state copy for each question: the same answers must give
    /// the same questions, events and resulting state, and a different answer may only change
    /// what follows it.
    #[test]
    fn deterministic_with_respect_to_answers() {
        let (a, ea, ca) = castle_attack(7, vec![0, 1, 0]);
        let (b, eb, cb) = castle_attack(7, vec![0, 1, 0]);
        assert_eq!(ea, eb);
        assert_eq!(ca.asked, cb.asked);
        assert!(a == b);
        // replaying a recorded prefix reproduces the same first questions
        let (_, _, cc) = castle_attack(7, vec![0]);
        assert_eq!(ca.asked.first(), cc.asked.first());
        // whole turns too
        let run = |answers: Vec<u32>| {
            let mut st = State::new_game(3, 3, &[crate::world::HumanSetup::test(1), crate::world::HumanSetup::test(2)]).unwrap();
            let mut ch = Script { answers, asked: vec![] };
            st.run(&mut ch);
            let mut ev = vec![];
            for _ in 0..20 {
                ev.extend(st.apply(crate::Command::EndTurn, &mut ch));
            }
            (st.to_snapshot(), ev, ch.asked)
        };
        assert_eq!(run(vec![1, 0, 2, 1]), run(vec![1, 0, 2, 1]));
    }
}
