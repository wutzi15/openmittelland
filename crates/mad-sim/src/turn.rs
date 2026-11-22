//! Turn and year loop (rules-01 §3), turn start reports (rules-02 §3.3, §4.4, §5.2), turn-end
//! settlement (rules-02 §7) and year end (rules-01 §3.2 with the subsystem docs).

use crate::economy::*;
use crate::heap::{self, ANY};
use crate::laws::{self, has};
use crate::state::*;
use crate::{Ask, Choices, Event};

/// Kontor actions of the current human (phase 4) plus `EndTurn`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    EndTurn,
    /// Write an order slot (status is set by the sim: 1 if feasible, else 0).
    SetOrder { branch: u8, slot: u8, order: [u8; 8] },
    ConfirmOrder { branch: u8, slot: u8, on: bool },
    /// Buy storage offer `k` (0..2) for the branch's city (rules-02 §5.2).
    BuyStorage { branch: u8, k: u8 },
    SellStorage { branch: u8, m2: u16, stone: bool },
    BuildTransporters { branch: u8, t: u8, n: u8 },
    SellTransporters { branch: u8, t: u8, n: u8 },
    RepairTransporters { branch: u8, t: u8, n: u8 },
    /// 0 directors, 1 workers, 2 mercenaries, 3 robbers.
    SetPay { which: u8, pct: u8 },
    Apply { loc: u8, idx: u8, on: bool },
    /// Opens the office tree: rolls all prices (rules-04 §2), returned as `Event::TreePrices`.
    OfficeTree,
    Bribe { target: u8, amount: i32 },
    Spies { target: u8, price: u32 },
    CancelSpies { target: u8 },
    Saboteur { target: u8, price: u32 },
    CancelSaboteur { target: u8 },
    Blackmail { target: u8 },
    Denounce { victim: u8, listener: u8 },
    Matchmaker,
    TakeMistress,
    NameChild { slot: u8, name: String },
    Recruit { prop: u8, money: u16 },
    Manoeuvre { prop: u8 },
    /// Opening the manoeuvre window draws its picture from the game RNG (rules-27 §4).
    ManoeuvreView,
    /// Entering the Kontor (0x5D610) rolls its music: `rand(100) < 25`, then `rand(100) < 50` for 0x15 or 0x13 (rules-14 §4).
    KontorView,
    /// A screen's `music(song, mode)` call (rules-14 §4): codes 0x19..0x1E are random groups drawn from the game RNG;
    /// `coin`: the back room's `rand(100) < 50` picks mode 0, else mode 2.
    Music { song: u8, mode: u8, coin: bool },
    /// Opening the indulgence offer draws its city: one rand(10) after the price (rules-04b Ablass).
    IndulgenceView,
    /// Opening the Bewerbung window: `human_rival_of` for every human (rules-64 §2, draws).
    ApplyView,
    /// Opening Spielstand: tax and tithe are credited again (rules-28 §2) and reported for the bar scale.
    ViewScores,
    TollRate { prop: u8, rate: u8 },
    /// Improve security (`which` 0, at most 100) or build casemates (1, at most 150) of an owned castle or camp for the
    /// value difference (rules-26 §5.1; no affordability check).
    /// Send `amount` men of troop slot `slot` home without pay or refund (lower half of a count on the castle screen, rules-26 §3.2).
    Dismiss { prop: u8, slot: u8, amount: u8 },
    Fortify { prop: u8, which: u8, amount: u8 },
    /// Office tree mode 5: file (or withdraw) a trial (rules-04b §3.1).
    FileTrial { target: u8 },
    /// Plan or cancel a feast (rules-08 §2.1).
    PlanFeast { branch: u8, size: u8, musicians: u8, years: u8 },
    /// Office tree mode 6 / privilege 26 (rules-08 §3).
    Insult { target: u8 },
    /// Auction lots (rules-08 §1.2): transporters (type 12), a residence or castle (14 / 16).
    AuctionTransporters { branch: u8, t: u8, n: u8 },
    AuctionProperty { prop: u8 },
    /// Withdraw a residence lot (type 14) from the coming auction (rules-35 §4).
    WithdrawLot { prop: u8 },
    /// Opening the castle info screen: with nobody interested, `rand(100) < 50` makes a computer person interested (rules-08 §1.5).
    CastleOffers { prop: u8 },
    /// Sell an owned castle or camp to an interested person for its value (rules-23 §4.2).
    SellCastle { prop: u8, buyer: u8 },
    /// Express or withdraw interest in somebody else's castle or camp (rules-26 §2, at most six interested).
    Interest { prop: u8, on: bool },
    /// Privilege menu action (see `State::use_privilege` for a/b per code).
    Privilege { holder: u8, code: u8, a: i32, b: i32 },
    /// Opening a privilege's dialog draws the cost it shows (`standing`, rules-38 §2): `Event::PrivCost`.
    PrivilegeView { holder: u8, code: u8 },
    /// Law book edit mode (rules-38 §3): line `i` of `book` toggles (flag) or moves by `delta` (number, clamped to its
    /// range); needs privilege 56 / 59 / 82.
    EditLaw { book: u8, i: u8, delta: i32 },
    /// Testament editor: move the main heir by d (rules-05 §7.1); needs privilege 6.
    Testament { d: i8 },
    /// Kontor actions of rules-11 (kontor.rs).
    /// Directors §7: hire asks `Ask::PickDirector`; fire; set the yearly budget.
    HireDirector { branch: u8 },
    FireDirector { branch: u8 },
    SetBudget { branch: u8, budget: u32 },
    /// §1
    BuyBranch { city: u8 },
    SellBranch { branch: u8 },
    /// §3: good 0xFF = sell every good completely; otherwise `qty` units of `good`.
    SellStock { branch: u8, good: u8, qty: u16 },
    /// §2: buy offer k (0..2, `State::residence_offer`), build level 0..9, sell, extensions
    /// (wanted bit set of 0x20/0x40/0x80), renovate.
    BuyResidence { branch: u8, k: u8 },
    BuildResidence { branch: u8, level: u8 },
    SellResidence { branch: u8 },
    ExtendResidence { branch: u8, bits: u8 },
    RenovateResidence { branch: u8 },
    /// §4: one inquiry at the money lender (`Ask::LoanOffer`).
    AskLender,
    /// record index of the loan in `State::heap` (from the loan book)
    RepayLoan { loan: u16 },
    /// §5 Kircheneintritt (free)
    JoinChurch { religion: u8 },
    /// §6
    GiveUp,
    Convert,
    LeaveChurch,
    Indulgence,
    Candles,
    Pray,
    Alms,
}

/// Storage offers per city (`0x85D01`, three u16 per city, value + 10000 = stone; rules-09 §5.8).
const OFFERS: usize = 0x85D01;

impl State {
    /// `music(code, ..)` resolves the random groups 0x19..0x1E with one game-RNG draw (rules-14 §1); other codes
    /// are songs.
    pub fn song(&mut self, code: u8) -> u8 {
        match code {
            0x19 => [0x12, 0x13, 0x14, 0x15, 0x01, 0x04, 0x0A, 0x12, 0x15][self.r(9) as usize],
            0x1A => [2, 3][self.r(2) as usize],
            0x1B => [0x10, 6, 3][self.r(3) as usize],
            0x1C => [2, 3, 0x10, 6][self.r(4) as usize],
            0x1D => [1, 4, 0x12, 0x18][self.r(4) as usize],
            0x1E => [7, 0x0E][self.r(2) as usize],
            s => s,
        }
    }

    fn offer(&self, c: usize, k: usize) -> u16 {
        rd16(&self.globals.0, G(OFFERS) + 6 * c + 2 * k)
    }
    fn set_offer(&mut self, c: usize, k: usize, v: u16) {
        wr16(&mut self.globals.0, G(OFFERS) + 6 * c + 2 * k, v)
    }

    /// Next human after `i` (rules-01 §3): smallest j > i with controller nibble set.
    pub fn next_human(&self, i: usize) -> u8 {
        ((i + 1)..PERSONS).find(|&j| self.persons[j].used() && self.persons[j].human()).map_or(NOBODY, |j| j as u8)
    }

    /// Advance until the current human is in the Kontor (phase 4) or the game is over.
    pub fn run(&mut self, ch: &mut dyn Choices) -> Vec<Event> {
        let mut ev = vec![];
        loop {
            if self.year >= 2000 {
                ev.push(Event::GameOver);
                return ev;
            }
            if self.current >= NOBODY {
                let Some(first) = self.humans().next() else {
                    ev.push(Event::AllPlayersOut);
                    return ev;
                };
                self.current = first as u8;
                self.phase = 0;
                // the turn-end windows come before the year end and its battles (live A/B: bill, toll, then the
                // computer's castle attack)
                ch.show(&ev);
                ev.extend(self.year_end(ch));
                self.heap.compact();
                self.year += 1;
                self.city_growth();
                continue;
            }
            let h = self.current as usize;
            match self.phase {
                0 => {
                    ev.push(Event::TurnStart { person: h as u8, year: self.year });
                    if self.prison_check(h) {
                        ev.push(Event::TurnSkipped { person: h as u8 });
                        self.phase = 6;
                    } else {
                        ev.extend(self.turn_start(h));
                        self.phase = 4;
                    }
                }
                1..=4 => return ev,
                5 => {
                    ev.extend(self.settlement(h, ch));
                    if self.persons[h].used() && self.persons[h].human() {
                        ev.push(Event::TurnEndCash { person: h as u8, cash: self.persons[h].cash() });
                    }
                    self.phase = 6;
                }
                _ => {
                    self.heap.compact();
                    self.current = self.next_human(h);
                    self.phase = 0;
                }
            }
        }
    }

    /// rules-01 §3.1 step 1 / rules-04b §3.6: types 20, then 21 separately.
    /// The "Produktion" page (rules-38 §8): the listed goods (13, or 11 when some were given away) each draw one
    /// rand(3) and gain 1 + rand(3); when more goods were produced than fit, every index after the last listed one
    /// up to 14 draws rand(3) and adds it to the *next* index (quirk: 15 is person byte 0xEF, director pay).
    pub fn production_skill(&mut self, h: usize, total: &[u32], gave: bool, ev: &mut Vec<Event>) {
        for g in (0..GOODS).filter(|&g| total[g] > 0) {
            let q = ((59 - (self.weather[g] as i32 - 70)) / 12) as u8;
            ev.push(Event::Harvest { good: g as u8, quality: q });
        }
        let shown = if gave { 11 } else { 13 };
        let (mut listed, mut after) = (0, GOODS);
        for g in (0..GOODS).filter(|&g| total[g] > 0) {
            if listed == shown {
                after = g;
                break;
            }
            listed += 1;
            let s = (self.persons[h].skill(g) as u32 + 1 + self.r(3)).min(150) as u8;
            self.persons[h].set_skill(g, s);
            ev.push(Event::SkillGain { good: g as u8, skill: s });
        }
        for g in after..GOODS {
            let up = self.r(3);
            if g + 1 < GOODS {
                let s = (self.persons[h].skill(g + 1) as u32 + up).min(150) as u8;
                self.persons[h].set_skill(g + 1, s);
            } else {
                let v = (self.persons[h].pay_directors() as u32 + up).min(150) as u8;
                self.persons[h].set_pay_directors(v);
            }
        }
    }

    fn prison_check(&mut self, h: usize) -> bool {
        let mut skip = false;
        for t in [20, 21] {
            if let Some(i) = self.heap.find(t, h as u8, ANY) {
                if self.heap.recs[i][3] == 0 {
                    self.heap.delete(i);
                } else {
                    self.heap.recs[i][3] -= 1;
                    skip = true;
                    // rules-04b §2: the skipped turn's message writes history code 0 (debtors' tower)
                    self.history(h, 0, 0, 0);
                }
            }
        }
        skip
    }

    /// rules-01 §3.3 city growth, 3 RNG calls per city. The product is 32-bit unsigned: past about 4.09 million
    /// it wraps and the city drops to a few ten thousand (the original's long run, FIDELITY "Passive long run" 6).
    pub fn city_growth(&mut self) {
        for c in 0..CITIES {
            let p = self.cities[c].population();
            let g = 1000 + self.r(50);
            self.cities[c].set_population(p.wrapping_mul(g) / 1000);
            let r = self.cities[c].rights().wrapping_add(self.r(200));
            self.cities[c].set_rights(r);
            if self.r(100) < 7 {
                let e = self.cities[c].extra88() + 1;
                self.cities[c].set_extra88(e);
            }
        }
    }

    // ---- phase 0 ----

    /// `turn_start_reports` (0x4B900), steps in the order of rules-37 §1.
    pub fn turn_start(&mut self, h: usize) -> Vec<Event> {
        let mut ev = vec![];
        let hb = h as u8;
        self.globals.set_d(ga::TURN_FLAGS, 0);
        self.globals.set_b(ga::WORKERS_GONE, 0);
        // give-away counter 0x85CC1 = 0 (rules-37 §1 #1, rules-55 §2)
        self.globals.set_d(ga::GIVEN_AWAY, 0);
        for c in 0..CITIES {
            for g in 0..GOODS {
                self.cities[c].set_sold(g, 0);
                self.globals.set_sales(g, c, 0);
            }
        }
        if self.year > 1600 {
            ev.extend(self.arrivals(h));
            // troop orders 51/53/54 return to the home garrison (rules-07 §4)
            for t in [51u8, 53, 54] {
                for i in self.heap.all(t, hb, ANY) {
                    let home = self.heap.recs[i][2] as usize;
                    if self.props[home].owner() as usize == h {
                        for k in 0..4 {
                            let v = self.props[home].troops(k).saturating_add(self.heap.recs[i][5 + k]);
                            self.props[home].set_troops(k, v);
                        }
                    }
                    self.heap.delete(i);
                }
            }
            self.garrison_trim();
            for g in 0..GOODS {
                self.weather[g] = 70 + self.r(60) as u8;
            }
            // production (§3.3)
            let mut total = [0u32; GOODS];
            let mut gave = false;
            for i in self.heap.all(8, hb, ANY) {
                let r = self.heap.recs[i].clone();
                let (b, g, w, m) = (r[2] as usize, r[4] as usize, r[5] as i32, r[6] as i32);
                self.heap.delete(i);
                if self.props[b].kind() != KIND_BRANCH || self.props[b].owner() != hb || g >= GOODS {
                    continue;
                }
                let out = self.production_output(b, g, w, m);
                let given = self.add_goods(b, g, out as u16);
                total[g] += out as u16 as u32;
                ev.push(Event::Produced { branch: b as u8, good: g as u8, amount: out as u16 as u32 });
                if given > 0 {
                    gave = true;
                    ev.push(Event::GivenAway { branch: b as u8, amount: given });
                }
            }
            self.production_skill(h, &total, gave, &mut ev);
            // transporters built / repaired (types 10, 11) with counter exactly 0, after the report's click;
            // a repair always draws rand(30), the foreign cartel decides whether it counts (rules-37 §1 #15/16)
            let cartel = self.heap.all(39, ANY, ANY).into_iter().any(|i| self.heap.recs[i][1] != hb);
            for t in [10u8, 11] {
                for i in self.heap.all(t, hb, ANY) {
                    let r = self.heap.recs[i].clone();
                    if r[3] != 0 {
                        continue;
                    }
                    let b = r[2] as usize;
                    let cond = if t == 11 {
                        let x = self.r(30) as u8;
                        if cartel { 100 - x } else { 100 }
                    } else {
                        100
                    };
                    if self.props[b].kind() == KIND_BRANCH && self.props[b].owner() == hb {
                        self.add_transporters(b, r[4] as usize, r[5], cond);
                        ev.push(Event::TransportersBuilt { branch: b as u8, t: r[4], n: r[5] });
                    }
                    self.heap.delete(i);
                }
            }
            self.deliver_residences(h);
        }
        // certificate award 0x494F0 (rules-37 §1 #18, rules-55 §10): after the Erträge window and the
        // type-10/11/13 deliveries, before the offers; outside the year > 1600 block
        if self.persons[h].certificate_due() != 0 {
            self.persons[h].set_certificate_due(0);
            let c = self.persons[h].certificate() + 1;
            self.persons[h].set_certificate(c);
            ev.push(Event::Certificate { person: h as u8, level: c });
        }
        // residence and storage offers, interleaved per slot (rules-37 §1 #19)
        for c in 0..CITIES {
            let cur = self.branch_at(h, c).map_or(15, |b| self.props[b].wood() as u32 + self.props[b].stone() as u32);
            let mut offers = [(0u16, false); 3];
            for (k, o) in offers.iter_mut().enumerate() {
                self.roll_residence_offer(c, k);
                *o = self.storage_offer(cur, k);
                self.set_offer(c, k, o.0 + if o.1 { 10000 } else { 0 });
            }
            ev.push(Event::StorageOffer { city: c as u8, offers });
        }
        for l in 0..PERSONS {
            self.globals.set_b(ga::DENOUNCE + l, 0);
        }
        // church prices: 0x85D3D = standing(40) first, then 0x85D41 = standing(30) (rules-37 §1 #21)
        let alms = self.standing(h, 40);
        let candles = self.standing(h, 30);
        self.set_church_prices([candles, alms]);
        self.roll_lender_mood();
        self.globals.set_b(ga::PROPOSALS, 0);
        // order refresh (rules-37 §1 #24, rules-55 §12): every property of h, 0x45820; then each status-3 slot
        // (any kind: production, transports 2..5, patrol, raid) is reserved, set to 2 and refreshed again
        for p in (0..PROPS).filter(|&p| self.props[p].owner() as usize == h && self.props[p].kind() < KIND_FREE).collect::<Vec<_>>() {
            self.refresh_orders(p);
            for k in 0..4 {
                let mut o = self.props[p].order(k);
                if self.has_order(p, k) && o[7] == ST_DONE {
                    self.reserve(p, k, true);
                    o[7] = ST_CONFIRMED;
                    self.props[p].set_order(k, o);
                    self.refresh_orders(p);
                }
            }
        }
        self.run_directors(h, 0);
        ev
    }

    /// 0x446F0: every castle and camp sheds men above its capacity (byte +4), one at a time from slots
    /// 0, 1, 2, 3, 0, … (rules-37 §1 #8, [L]).
    fn garrison_trim(&mut self) {
        for p in 0..PROPS {
            if !matches!(self.props[p].kind(), KIND_CASTLE | KIND_CAMP) {
                continue;
            }
            let cap = self.props[p].b4() as u32;
            let mut k = 0;
            while (0..4).map(|j| self.props[p].troops(j) as u32).sum::<u32>() > cap {
                let v = self.props[p].troops(k);
                if v > 0 {
                    self.props[p].set_troops(k, v - 1);
                }
                k = (k + 1) % 4;
            }
        }
    }

    /// §4.4 arrivals of type-9 records of h, in heap order.
    fn arrivals(&mut self, h: usize) -> Vec<Event> {
        let mut ev = vec![];
        let hb = h as u8;
        let mut cash = 0i32;
        let bonus = if self.heap.all(42, hb, ANY).into_iter().any(|i| self.heap.recs[i][3] < 7) { 2 } else { 0 };
        for i in self.heap.all(9, hb, ANY) {
            let r = self.heap.recs[i].clone();
            self.heap.delete(i);
            let (src, kind, qty, g, cnt, t, dest, cond) =
                (r[2] as usize, r[4], heap::r16(&r, 5), r[7] as usize, r[8], r[9] as usize, r[10] as usize, r[11]);
            if self.props[src].kind() != KIND_BRANCH || self.props[src].owner() != hb {
                continue;
            }
            let src_city = self.props[src].loc() as usize;
            let back = |st: &mut State, ev: &mut Vec<Event>| {
                if kind <= 5 && g < GOODS {
                    st.add_goods(src, g, qty);
                }
                st.add_transporters(src, t, cnt, cond);
                ev.push(Event::Returned { branch: src as u8 });
            };
            if (dest < CITIES && self.city_blocked(dest)) || self.city_blocked(src_city) {
                back(self, &mut ev);
                continue;
            }
            match kind {
                2 | 3 if dest < CITIES => {
                    let rev = qty as i32 * (self.cities[dest].price(g) as i32 + bonus);
                    let s = self.globals.sales(g, dest).wrapping_add(rev as u32);
                    self.globals.set_sales(g, dest, s);
                    cash += rev;
                    let sold = self.cities[dest].sold(g) + qty as u32;
                    self.cities[dest].set_sold(g, sold);
                    self.add_transporters(src, t, cnt, cond);
                    ev.push(Event::Sold { good: g as u8, city: dest as u8, qty, revenue: rev });
                }
                4 | 5 => match self.branch_at(h, dest) {
                    Some(d) => {
                        self.add_goods(d, g, qty);
                        self.add_transporters(d, t, cnt, cond);
                        ev.push(Event::Arrived { branch: d as u8, good: g as u8, qty });
                    }
                    None => back(self, &mut ev),
                },
                _ => {}
            }
        }
        self.add_cash(h, cash);
        ev
    }

    // ---- phase 4 ----

    pub fn apply(&mut self, cmd: Command, ch: &mut dyn Choices) -> Vec<Event> {
        let h = self.current as usize;
        if self.current >= NOBODY || !(1..=4).contains(&self.phase) {
            return vec![Event::Refused("not in the Kontor")];
        }
        let own = |st: &State, b: u8| (b as usize) < PROPS && st.props[b as usize].owner() as usize == h && st.props[b as usize].kind() != KIND_FREE;
        let mut ev = vec![];
        match cmd {
            Command::EndTurn => {
                self.phase = 5;
                ev.extend(self.run(ch));
            }
            Command::SetOrder { branch, slot, mut order } if own(self, branch) && slot < 4 => {
                let (b, k) = (branch as usize, slot as usize);
                // rules-36 §4, §6: an edit leaves the status to the redraw refresh: a confirmed order is reserved
                // again and stays confirmed while feasible, else drops to 0; any other order shows its feasibility
                let was = self.props[b].order(k)[7];
                if was == ST_CONFIRMED {
                    self.reserve(b, k, false);
                }
                order[7] = 0;
                self.props[b].set_order(k, order);
                let ok = self.order_feasible(b, k);
                order[7] = match (was == ST_CONFIRMED, ok) {
                    (true, true) => ST_CONFIRMED,
                    (_, true) => ST_POSSIBLE,
                    (_, false) => ST_IMPOSSIBLE,
                };
                self.props[b].set_order(k, order);
                if order[7] == ST_CONFIRMED {
                    self.reserve(b, k, true);
                }
            }
            Command::ConfirmOrder { branch, slot, on } if own(self, branch) && slot < 4 => {
                let (b, k) = (branch as usize, slot as usize);
                let mut o = self.props[b].order(k);
                if on && o[7] == ST_POSSIBLE && self.order_feasible(b, k) {
                    self.reserve(b, k, true);
                    o[7] = ST_CONFIRMED;
                } else if !on && o[7] == ST_CONFIRMED {
                    self.reserve(b, k, false);
                    o[7] = ST_POSSIBLE;
                }
                self.props[b].set_order(k, o);
            }
            Command::BuyStorage { branch, k } if own(self, branch) && k < 3 => {
                let b = branch as usize;
                let c = self.props[b].loc() as usize;
                let v = self.offer(c, k as usize);
                let (size, stone) = if v > 10000 { (v - 10000, true) } else { (v, false) };
                let cur = if stone { self.props[b].stone() } else { self.props[b].wood() };
                let capped = cur + size > 4999;
                let size = size.min(4999u16.saturating_sub(cur));
                let price = if stone { self.cities[c].level_b() } else { self.cities[c].level_a() } as i32;
                if size > 0 && c < CITIES {
                    self.pay(h, size as i32 * price);
                    if stone { self.props[b].set_stone(cur + size) } else { self.props[b].set_wood(cur + size) }
                    // rules-34 §2: a capped purchase only reduces the offer, otherwise it is used up
                    self.set_offer(c, k as usize, if capped { v - size } else { 0 });
                }
            }
            Command::SellStorage { branch, m2, stone } if own(self, branch) => {
                let b = branch as usize;
                let c = self.props[b].loc() as usize;
                let cur = if stone { self.props[b].stone() } else { self.props[b].wood() };
                let m2 = m2.min(cur);
                let price = if stone { self.cities[c].level_b() } else { self.cities[c].level_a() } as i32;
                self.add_cash(h, m2 as i32 * price);
                if stone { self.props[b].set_stone(cur - m2) } else { self.props[b].set_wood(cur - m2) }
            }
            Command::BuildTransporters { branch, t, n } if own(self, branch) && (t as usize) < TRANSPORTERS => {
                // rules-34 §4: one type-10 record per kind with counter BT[k], reused; at most 99 pending
                // (the stock is not counted, correction to rules-02 §6)
                let b = branch as usize;
                let (t, c) = (t as usize, self.props[b].loc() as usize);
                let recs: Vec<usize> = self.heap.all(10, h as u8, branch).into_iter().filter(|&i| self.heap.recs[i][4] as usize == t).collect();
                let pending: u32 = recs.iter().map(|&i| self.heap.recs[i][5] as u32).sum();
                if n == 0 {
                    // rules-34 §4: "if n > 0", otherwise nothing
                } else if (needs_water(t) && !navigable(c)) || pending + n as u32 > 99 {
                    ev.push(Event::Refused("cannot build"));
                } else if let Some(i) = recs.into_iter().find(|&i| self.heap.recs[i][3] == BUILD_YEARS[t]).or_else(|| {
                    let i = self.heap.alloc(10, h as u8, branch, BUILD_YEARS[t])?;
                    self.heap.recs[i][4] = t as u8;
                    self.heap.recs[i][5] = 0;
                    Some(i)
                }) {
                    self.pay(h, self.cities[c].tprice(t) as i32 * n as i32);
                    self.heap.recs[i][5] += n;
                }
            }
            Command::SellTransporters { branch, t, n } if own(self, branch) && (t as usize) < TRANSPORTERS => {
                let b = branch as usize;
                let (t, c) = (t as usize, self.props[b].loc() as usize);
                let n = n.min(self.props[b].trans(t));
                let v = self.props[b].cond(t) as i32 * n as i32 * self.cities[c].tprice(t) as i32 / 200;
                self.add_cash(h, v);
                let left = self.props[b].trans(t) - n;
                self.props[b].set_trans(t, left);
            }
            Command::RepairTransporters { branch, t, n } if own(self, branch) && (t as usize) < TRANSPORTERS => {
                // rules-34 §4: one type-11 record per kind (counter 1), reused; the window bills page 1's kind
                // t % 3 at its condition and price, also for page-2 kinds (rules-02 §6 quirk)
                let b = branch as usize;
                let (k, c) = (t as usize, self.props[b].loc() as usize);
                let bill = k % 3;
                let n = n.min(self.props[b].trans(k));
                if n > 0 {
                    let rec = self.heap.all(11, h as u8, branch).into_iter().find(|&i| self.heap.recs[i][4] as usize == k && self.heap.recs[i][3] == 1).or_else(|| {
                        let i = self.heap.alloc(11, h as u8, branch, 1)?;
                        self.heap.recs[i][4] = k as u8;
                        self.heap.recs[i][5] = 0;
                        Some(i)
                    });
                    if let Some(i) = rec {
                        let cost = n as i32 * self.cities[c].tprice(bill) as i32 * (100 - self.props[b].cond(bill) as i32) / 100;
                        self.pay(h, cost);
                        self.heap.recs[i][5] = self.heap.recs[i][5].saturating_add(n);
                        let left = self.props[b].trans(k) - n;
                        self.props[b].set_trans(k, left);
                    }
                }
            }
            Command::SetPay { which, pct } => match which {
                0 => self.persons[h].set_pay_directors(pct),
                1 => self.persons[h].set_pay_workers(pct),
                2 => self.persons[h].set_pay_mercs(pct),
                _ => self.persons[h].set_pay_robbers(pct),
            },
            Command::Apply { loc, idx, on } => {
                if !on || self.eligibility(h, loc as usize, idx as usize) == 2 {
                    self.apply_for(h, loc as usize, idx as usize, on);
                }
            }
            Command::OfficeTree => {
                let p = self.office_tree_prices(h);
                ev.push(Event::TreePrices(p));
            }
            Command::Bribe { target, amount } => ev.extend(self.bribe(h, target as usize, amount)),
            Command::Spies { target, price } => self.spies(h, target as usize, price),
            Command::CancelSpies { target } => self.cancel_spies(h, target as usize),
            Command::Saboteur { target, price } => self.saboteur(h, target as usize, price),
            Command::CancelSaboteur { target } => self.cancel_saboteur(h, target as usize),
            Command::Blackmail { target } => ev.push(self.blackmail(h, target as usize)),
            Command::Denounce { victim, listener } => ev.extend(self.denounce(h, victim as usize, listener as usize)),
            Command::Matchmaker if self.heap.has(30, h as u8) => {
                let i = self.heap.find(30, h as u8, ANY).unwrap();
                let (target, city) = (self.heap.recs[i][9], self.heap.recs[i][10]);
                if ch.ask(&Ask::GiveUpCourtship { target, city }) == 0 {
                    self.heap.delete(i);
                }
            }
            Command::Matchmaker => match self.matchmaker_offer(h) {
                Ok((c, fee, w)) => {
                    if ch.ask(&Ask::Match { target: c as u8, fee }) == 0 {
                        self.start_courtship(h, c, fee, w);
                    }
                }
                Err(text) => ev.push(Event::Notice { text }),
            },
            Command::TakeMistress if has(self, h, 4) => self.take_mistress(h, &mut ev),
            Command::NameChild { slot, name } => {
                let ok = self.heap.all(31, h as u8, ANY).into_iter().any(|i| self.heap.recs[i][5] == slot);
                if ok && (slot as usize) < 32 {
                    self.set_name_str(slot as usize, &name);
                }
            }
            Command::Recruit { prop, money } if own(self, prop) => {
                let money = money.min(9999); // rules-26 §5.2: per investment, no per-turn total
                if money > 0
                    && let Some(i) = self.heap.alloc(15, h as u8, prop, 1) {
                        self.pay(h, money as i32);
                        heap::w16(&mut self.heap.recs[i], 4, money);
                    }
            }
            Command::Manoeuvre { prop } if own(self, prop) => self.manoeuvre(prop as usize),
            Command::ViewScores => {
                let (tax, tithe) = self.tax_tithe(h);
                ev.push(Event::Scores { tax, tithe });
            }
            Command::IndulgenceView => ev.push(Event::IndulgenceCity { city: self.r(10) as u8 }),
            Command::ApplyView => {
                let hs: Vec<usize> = self.humans().collect();
                let table = hs.into_iter().map(|x| (x as u8, self.rival_of(x).map_or(NONE, |r| r as u8))).collect();
                ev.push(Event::ApplyRivals(table));
            }
            Command::ManoeuvreView => ev.push(Event::ManoeuvrePic { pic: 0x199 + self.r(3) as u16 }),
            Command::Music { song, mode, coin } => {
                let song = self.song(song);
                let mode = if coin && self.r(100) < 50 { 0 } else { mode };
                ev.push(Event::Song { song, mode });
            }
            Command::KontorView => {
                if self.r(100) < 25 {
                    let song = if self.r(100) < 50 { 0x15 } else { 0x13 };
                    ev.push(Event::Song { song, mode: 2 });
                }
            }
            Command::Dismiss { prop, slot, amount } if own(self, prop) && slot < 4 => {
                let pr = &mut self.props[prop as usize];
                pr.set_troops(slot as usize, pr.troops(slot as usize).saturating_sub(amount));
            }
            Command::Fortify { prop, which, amount } if own(self, prop) && which < 2 => {
                let q = prop as usize;
                let cost = self.fortify_cost(q, which, amount);
                let add = self.fortify_amount(q, which, amount);
                if add > 0 {
                    let pr = &mut self.props[q];
                    if which == 0 { pr.set_b3(pr.b3() + add) } else { pr.set_b4(pr.b4() + add) }
                    self.pay(h, cost);
                }
            }
            Command::TollRate { prop, rate } if own(self, prop) && self.props[prop as usize].kind() == KIND_CASTLE => {
                let k = self.props[prop as usize].loc() as usize - 14;
                self.castles[k].toll_rate = rate.min(14);
            }
            Command::FileTrial { target } => ev.extend(self.file_trial(h, target as usize)),
            Command::PlanFeast { branch, size, musicians, years } if (branch as usize) < PROPS => {
                ev.extend(self.plan_feast(h, branch as usize, size, musicians, years))
            }
            Command::Insult { target } => ev.extend(self.insult(h, target as usize, ch)),
            Command::Convert if self.persons[h].religion() != 2 => self.convert(h),
            Command::LeaveChurch if self.persons[h].religion() != 2 => self.leave_church(h),
            Command::Indulgence => {
                self.indulgence(h);
            }
            Command::AuctionTransporters { branch, t, n } if own(self, branch) && (t as usize) < TRANSPORTERS => {
                self.lot_transporters(h, branch as usize, t as usize, n)
            }
            Command::AuctionProperty { prop } if own(self, prop) => self.lot_property(h, prop as usize),
            Command::WithdrawLot { prop } if own(self, prop) => {
                if let Some(i) = self.heap.find(14, h as u8, prop) {
                    self.heap.delete(i);
                }
            }
            Command::CastleOffers { prop } if own(self, prop) => self.castle_offers(prop as usize),
            Command::Interest { prop, on } if (prop as usize) < PROPS && !own(self, prop) => {
                let mine = self.heap.find(17, h as u8, prop);
                match (on, mine) {
                    (true, None) if self.heap.all(17, crate::heap::ANY, prop).len() < 6 => {
                        self.heap.alloc(17, h as u8, prop, 100);
                    }
                    (false, Some(i)) => self.heap.delete(i),
                    _ => {}
                }
            }
            Command::SellCastle { prop, buyer } if own(self, prop) => self.sell_castle(h, prop as usize, buyer as usize, ch),
            Command::Privilege { holder, code, a, b } if (holder as usize) < PERSONS => {
                ev.extend(self.use_privilege(h, holder as usize, code, a, b, ch))
            }
            Command::Testament { d } if has(self, h, 6) => self.testament_step(h, d as i32),
            Command::PrivilegeView { holder, code } if (holder as usize) < PERSONS => {
                let x = holder as usize;
                let cost = match code {
                    48 | 74 | 79 | 55 => self.standing(x, 20),
                    53 | 54 | 84 | 80 | 81 => self.standing(x, 30),
                    86 => self.standing(h, 10),
                    85 => self.standing(x, 35),
                    _ => 0,
                };
                ev.push(Event::PrivCost { code, cost });
            }
            Command::EditLaw { book, i, delta } if book < 3 && i < 10 && has(self, h, [56, 59, 82][book as usize]) => {
                let (b, i) = (book as usize, i as usize);
                let old = self.globals.law(b, i) as i64;
                let range = match (b, i) {
                    (0, 1) => Some((0, 33)),
                    (0, 5 | 6) => Some((50, 150)),
                    (0, 7) => Some((0, 9_999_999)),
                    (0, 8) => Some((2, 10)),
                    (2, 9) => Some((0, 25)),
                    _ => None,
                };
                let v = match range {
                    Some((lo, hi)) => (old + delta as i64).clamp(lo, hi),
                    None => old ^ 1,
                };
                self.globals.set_law(b, i, v as u32);
            }
            Command::HireDirector { branch } if own(self, branch) && self.props[branch as usize].kind() == KIND_BRANCH => {
                self.hire_director(h, branch as usize, ch)
            }
            Command::FireDirector { branch } if own(self, branch) => self.props[branch as usize].set_director(NONE),
            Command::SetBudget { branch, budget } if own(self, branch) => self.props[branch as usize].set_budget(budget),
            Command::BuyBranch { city } => ev.extend(self.buy_branch(h, city as usize)),
            Command::SellBranch { branch } if own(self, branch) && self.props[branch as usize].kind() == KIND_BRANCH => {
                ev.extend(self.sell_branch(h, branch as usize))
            }
            Command::SellStock { branch, good, qty } if own(self, branch) && self.props[branch as usize].kind() == KIND_BRANCH => {
                self.sell_local(h, branch as usize, (good != 0xFF).then_some(good as usize), qty);
            }
            Command::BuyResidence { branch, k } if own(self, branch) && self.props[branch as usize].kind() == KIND_BRANCH => {
                ev.extend(self.buy_residence(h, branch as usize, k as usize))
            }
            Command::BuildResidence { branch, level } if own(self, branch) && self.props[branch as usize].kind() == KIND_BRANCH => {
                ev.extend(self.build_residence(h, branch as usize, level))
            }
            Command::SellResidence { branch } if own(self, branch) => {
                self.sell_residence(h, branch as usize);
            }
            Command::ExtendResidence { branch, bits } if own(self, branch) => ev.extend(self.extend_residence(h, branch as usize, bits)),
            Command::RenovateResidence { branch } if own(self, branch) => ev.extend(self.renovate_residence(h, branch as usize)),
            Command::AskLender => ev.extend(self.ask_lender(h, ch)),
            Command::RepayLoan { loan }
                if (loan as usize) < self.heap.recs.len()
                    && self.heap.live(loan as usize)
                    && self.heap.recs[loan as usize][0] == 56
                    && self.heap.recs[loan as usize][1] as usize == h =>
            {
                self.repay_loan(loan as usize)
            }
            Command::JoinChurch { religion } => self.join_church(h, religion),
            Command::GiveUp => {
                self.give_up(h);
                ev.extend(self.run(ch));
            }
            Command::Candles => self.candles(h),
            Command::Pray => self.pray(),
            Command::Alms => self.alms(h),
            _ => ev.push(Event::Refused("not allowed")),
        }
        ev
    }

    // ---- castles (rules-07 §2) ----

    /// §2.2 add_troops (0x45240).
    /// Returns the capped count, negative on an over-full property; `rand(4)` only when a man fits (rules-41 §4).
    pub fn add_troops(&mut self, prop: usize, men: u32, training: u8) -> i32 {
        let p = &self.props[prop];
        let cur: i32 = (0..4).map(|k| p.troops(k) as i32).sum();
        let cap = p.b4() as i32 - cur;
        let men = if men as i32 + cur > p.b4() as i32 { cap } else { men as i32 };
        if men <= 0 {
            return men;
        }
        let mut k = self.r(4) as usize;
        let p = &mut self.props[prop];
        for _ in 0..men {
            let c = p.troops(k) as u32;
            p.set_training(k, ((p.training(k) as u32 * c + training as u32) / (c + 1)) as u8);
            p.set_troops(k, (c + 1) as u8);
            k = (k + 1) % 4;
        }
        men
    }

    /// rules-02 §4.5 `0x618C0`: this turn's sales tax and tithe of human `h`, credited to the city and
    /// church treasuries (also on every Spielstand opening, rules-28 §2).
    pub fn tax_tithe(&mut self, h: usize) -> (i32, i32) {
        let (mut tax, mut tithe) = (0i32, 0i32);
        let exempt = has(self, h, 57);
        for c in 0..CITIES {
            if !exempt {
                let rate = self.cities[c].tax() as i64;
                let t: i64 = (0..GOODS).map(|g| rate * self.globals.sales(g, c) as i64 / 100).sum();
                tax += t as i32;
                let a = ga::TAX_TREASURY + 4 * c;
                self.globals.set_d(a, self.globals.d(a).wrapping_add(t as u32));
            }
            if !exempt && self.persons[h].religion() != 2 && !has(self, h, 11) {
                let k = if c == 0 { 0 } else { (c - 1) / 3 };
                let rate = self.globals.b(ga::TITHE + k) as i64;
                let t: i64 = (0..GOODS).map(|g| rate * self.globals.sales(g, c) as i64 / 100).sum();
                tithe += t as i32;
                let a = ga::TITHE_TREASURY + 4 * c;
                self.globals.set_d(a, self.globals.d(a).wrapping_add(t as u32));
            }
        }
        (tax, tithe)
    }

    /// §2.3 manoeuvre.
    fn manoeuvre(&mut self, prop: usize) {
        let h = self.props[prop].owner() as usize;
        let total: i32 = (0..4).map(|k| self.props[prop].troops(k) as i32).sum();
        let cost = 15 * total + (self.year as i32 * 277) % 20;
        self.pay(h, cost);
        let k = self.r(4) as usize;
        let m = if has(self, h, 37) || has(self, h, 66) { 15 } else { 1 };
        let add = 5 + self.r(10) + self.r(m);
        let t = (self.props[prop].training(k) as u32 + add).min(100) as u8;
        self.props[prop].set_training(k, t);
    }

    // ---- phase 5 ----

    /// `turn_end_settlement` (0x61A70), rules-02 §7 order.
    pub fn settlement(&mut self, h: usize, ch: &mut dyn Choices) -> Vec<Event> {
        let mut ev = vec![];
        let hb = h as u8;
        // rules-37 §2: directors, prices, title award
        self.run_directors(h, 1);
        self.prices_update();
        ev.extend(self.title_award(h));
        // rules-07 §3a: troop pay and training come first, the cost joins the bill below
        let [troops, robbers] = self.troop_pay(h);
        // 2 orders
        let (mut wages, mut means, mut transport, mut toll) = (0i32, 0i32, 0i32, 0i32);
        let inf = self.globals.d(ga::INFLATION);
        let _ = inf;
        for b in self.props_of(h, KIND_BRANCH).collect::<Vec<_>>() {
            let c = self.props[b].loc() as usize;
            for k in 0..4 {
                let mut o = self.props[b].order(k);
                if o[7] != ST_CONFIRMED {
                    continue;
                }
                if o[0] == ORDER_PRODUCE {
                    let (g, w, m) = (o[3] as usize, o[4], o[5]);
                    if let Some(i) = self.heap.alloc(8, hb, b as u8, 1) {
                        self.heap.recs[i][4..7].copy_from_slice(&[g as u8, w, m]);
                        wages += self.wage_cost(h, c, g, w as i32);
                        means += m as i32 * MEANS_COST[g] + self.ri(20);
                        o[7] = ST_DONE;
                    }
                } else if (2..=7).contains(&o[0]) {
                    let (mut kind, mut qty, mut g, cnt, t, dest) = (o[0], u16::from_le_bytes([o[1], o[2]]), o[3] as usize, o[4], o[5] as usize, o[6]);
                    if kind >= 6 {
                        // loot run: the good with the largest stock*V at the castle/camp; cap uses TC (quirk)
                        let Some(src) = (0..PROPS).find(|&i| {
                            let p = &self.props[i];
                            (p.kind() == KIND_CASTLE || p.kind() == KIND_CAMP) && p.owner() == hb && p.loc() == dest
                        }) else { continue };
                        g = (0..GOODS).max_by_key(|&g| (self.props[src].stock(g) as i32 * value(g), -(g as i32))).unwrap();
                        qty = (self.props[src].stock(g) as i32).min(TRIP_COST[t] * cnt as i32) as u16;
                        let s = self.props[src].stock(g) - qty;
                        self.props[src].set_stock(g, s);
                        transport += ((if kind == 7 { 3 } else { 2 }) * (TRIP_COST[t] * cnt as i32 + self.ri(10))) / 2;
                        kind -= 2;
                        // delivered to the branch city
                        let _ = dest;
                    } else {
                        transport += TRIP_COST[t] * cnt as i32 + self.ri(10);
                        if kind & 1 == 1 {
                            transport += (TRIP_COST[t] * cnt as i32 + self.ri(10)) / 2;
                        }
                    }
                    let dest = if o[0] >= 6 { c as u8 } else { dest };
                    if o[0] < 6 && dest != 0 && !matches!(t, BALLON | TAUCHBOOT | FLUGMASCHINE) && (dest as usize) < CITIES {
                        let county = COUNTY[dest as usize] as usize;
                        let k = county - 10;
                        let tl = self.castles[k].toll_rate as i32 * qty as i32 * value(g) / 100;
                        toll += tl;
                        let share = self.globals.b(ga::TOLL_SHARE + k) as i32;
                        self.castles[k].acc = self.castles[k].acc.wrapping_add(((100 - share) * tl / 100) as u32);
                        if let Some(kz) = self.holder(county, 6) {
                            self.add_cash(kz, share * tl / 100);
                        }
                    }
                    if let Some(i) = self.heap.alloc(9, hb, b as u8, 1) {
                        let r = &mut self.heap.recs[i];
                        r[4] = kind;
                        heap::w16(r, 5, qty);
                        r[7] = g as u8;
                        r[8] = cnt;
                        r[9] = t as u8;
                        r[10] = dest;
                        r[11] = self.props[b].cond(t);
                    }
                    // rules-37 §2 #5: loot runs (6, 7) drop to 1, everything else waits as executed (3)
                    o[7] = if o[0] >= 6 { ST_POSSIBLE } else { ST_DONE };
                }
                self.props[b].set_order(k, o);
            }
        }
        self.troop_execute(h);
        // 3
        self.settlement_pre(h, ch, &mut ev);
        // 4 directors
        let mut directors = 0i32;
        for b in self.props_of(h, KIND_BRANCH).collect::<Vec<_>>() {
            let d = self.props[b].director();
            if d != NONE {
                let pct = self.director_fee_pct(d as usize) as i64;
                let fee = (pct * self.props[b].budget() as i64 / 100) * self.persons[h].pay_directors() as i64 / 100;
                directors += fee as i32;
            }
        }
        // 5 tax and tithe
        let exempt = has(self, h, 57);
        let (tax, tithe) = self.tax_tithe(h);
        // 6 mercenary and robber pay: computed at the start (`troop_pay`)
        // 7 spies and saboteurs
        let mut spies = 0i32;
        for i in self.heap.all(6, hb, ANY) {
            if self.heap.recs[i][9] != 0 {
                spies += heap::r32(&self.heap.recs[i], 5) as i32;
            }
        }
        let mut saboteurs = 0i32;
        for i in self.heap.all(7, hb, ANY) {
            saboteurs += heap::r32(&self.heap.recs[i], 5) as i32;
        }
        // 8 interest
        let mut interest = 0i32;
        if !exempt {
            for i in self.heap.all(56, hb, ANY) {
                let r = &self.heap.recs[i];
                interest += (heap::r32(r, 5) as i64 * r[9] as i64 / 100) as i32;
            }
        }
        // 9 insurance
        let nw = self.net_worth(h);
        let insurance = (self.persons[h].insurance() & 0x0F).count_ones() as i32 * nw / 100;
        // 10 the bill
        let bill = wages + troops + robbers + means + transport + toll + directors + tax + tithe + spies + saboteurs + interest + insurance;
        self.pay(h, bill);
        ev.push(Event::Bill { wages, troops, robbers, means, transport, toll, directors, tax, tithe, spies, saboteurs, interest, insurance, cash: self.persons[h].cash() });
        let maxb = self.globals.b(ga::MAX_BRANCHES) as i32;
        let nb = self.branches(h) as i32;
        if nb > maxb {
            let x = ((nb - maxb) as i64 * 7 * self.net_worth(h) as i64 / 100) as i32;
            self.pay(h, x);
            let emperor = self.heap.find(1, NOBODY, ANY).map_or(0, |i| self.heap.recs[i][4]);
            ev.push(Event::BranchSurcharge { emperor, percent: ((nb - maxb) * 7).min(255) as u8, amount: x });
        }
        // 11 office income
        if self.persons[h].has_office() {
            let x = self.office_income(h);
            if x > 0 {
                self.add_cash(h, x);
                ev.push(Event::OfficeIncome { amount: x });
            }
        }
        // 12 castle toll income
        let inf = self.globals.d(ga::INFLATION) as u64;
        let f1 = self.globals.d(ga::AVG_SHIPMENT) as u64;
        for pi in self.props_of(h, KIND_CASTLE).collect::<Vec<_>>() {
            let k = self.props[pi].loc() as usize - 14;
            let add = ((self.castles[k].toll_rate as u64 * 7 * f1 * inf) / 400000) as u32;
            let acc = self.castles[k].acc.wrapping_add(add);
            let mut vogt = 0;
            let mut vogt_by = 0xFF;
            if let Some(v) = self.holder(10 + k, 7)
                && v != h {
                    vogt = (acc as i64 * 30 / 100) as i32;
                    vogt_by = v as u8;
                    self.add_cash(v, vogt);
                }
            let share = self.globals.b(ga::TOLL_SHARE + k) as i64;
            // [L] the Kanzler share is subtracted again but paid to nobody (rules-02 §7.12)
            let mine = (acc as i64 - acc as i64 * share / 100 - vogt as i64) as i32;
            self.add_cash(h, mine);
            self.castles[k].acc = 0;
            ev.push(Event::CastleIncome { castle: k as u8, amount: mine, gross: acc as i32, share: (acc as i64 * share / 100) as i32, vogt, vogt_by });
        }
        // 13 tips
        if self.opt(OPT_TIPS) {
            let k = self.r(5) as u8;
            ev.extend(self.tip(h, k));
        }
        // 14 fate events (rules-06 §1; rules-02 §7.14 calls this step "sub-plots")
        if self.opt(OPT_FATE) {
            self.fate(h, &mut ev);
            if self.r(100) < 50 {
                self.fate(h, &mut ev);
            }
        }
        // 15 blackmail expiry
        for a in 0..PERSONS {
            if self.persons[a].used() && self.persons[a].favour(h) > 100 && self.r(100) < 33 {
                let v = self.r(30) as u8;
                self.persons[a].set_favour(h, v);
                ev.push(Event::BlackmailEnded { by: a as u8 });
                if self.r(100) < 50 {
                    self.retaliate(a, h, 3);
                }
            }
        }
        // 16 record countdown; rules-21 §4.1: the Informant film is drawn once per turn end (30 %), before the pass
        self.informant_gate = self.r(100) >= 70;
        self.record_pass(hb, ch, &mut ev);
        // phase-6 tail (rules-PROGRESS-b note, rules-04b §2.3)
        if self.opt(OPT_ADVENTURE) {
            let r = self.r(6) as u16;
            if self.year > 1605 + r {
                let k = self.r(100);
                if k < 17 {
                    self.subplot_start(h, k, ch, &mut ev);
                }
            }
        }
        if self.r(100) < 25
            && let Some(r) = self.rival_of(h) {
                self.retaliate(r, h, 3);
            }
        for i in self.heap.all(22, hb, ANY) {
            self.duel(i, &mut ev);
        }
        let w = laws::offences(self, h);
        laws::add_evidence(self, h, w);
        for i in self.heap.all(18, hb, ANY) {
            self.trial(i, ch, &mut ev);
        }
        if self.debt_check(h, ch, &mut ev) {
            return ev; // the court returns directly: no death check this turn [C, quirk]
        }
        // 0x654A9: bankruptcy, mission, life
        if self.branches(h) == 0 {
            let name = self.name_str(self.persons[h].name() as usize);
            ev.push(Event::Bankrupt { person: hb, name });
            crate::family::purge(self, h, 1);
            crate::family::person_clear(self, h);
            return ev;
        }
        if self.heap.has(0, hb) {
            ev.extend(crate::missions::check(self, h, ch));
        }
        self.life_and_death(h, ch, &mut ev);
        ev
    }

    /// rules-09 §5 director fee percent (0x458F0): sum of the director's skills over the goods he
    /// may trade / (9 * count); goods 0 .. 5*certificate + 4.
    pub fn director_fee_pct(&self, d: usize) -> i32 {
        let n = (5 * self.persons[d].certificate() as usize + 5).min(GOODS);
        let s: i32 = (0..n).map(|g| self.persons[d].skill(g) as i32).sum();
        s / (9 * n as i32)
    }

    /// Turn-end countdown of all records owned by `owner` (rules-02 §7.16) with type handlers.
    fn record_pass(&mut self, owner: u8, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        let mut i = 0;
        while i < self.heap.recs.len() {
            if !self.heap.live(i) || self.heap.recs[i][1] != owner {
                i += 1;
                continue;
            }
            let r = &mut self.heap.recs[i];
            r[3] = r[3].wrapping_sub(1);
            let (t, c) = (r[0], r[3]);
            // dispatch per rules-37 §2.1 (jump table 0x63680)
            match t {
                6 => self.spy_tick(i, ev),
                // shipments in transit meet their mishaps at their heap position, kept for the turn start
                9 => {
                    let e = self.shipment_mishaps(i);
                    ev.extend(e);
                }
                // resolved at a later turn start or by a buyer; never deleted here
                8 | 10 | 11 | 13 | 14 => {}
                15 => self.recruit_tick(i),
                19 => {
                    if c == 0 {
                        self.heap.delete(i);
                    } else {
                        let x = heap::r32(&self.heap.recs[i], 4) as i32;
                        self.pay(owner as usize, x);
                        ev.push(Event::Fine { person: owner, amount: x });
                    }
                }
                28 => {
                    if c == 0 {
                        self.hold_feast(i, ev);
                    }
                }
                29 => self.mistress_tick(i, ev),
                30 => self.courtship_tick(i, ch, ev),
                31 => self.child_tick(i, ev),
                // trials and duels are held in the phase-6 tail, which deletes them
                18 | 22 => {}
                25 => self.siege_tick(i, ch, ev),
                26 => self.relief_tick(i, ev),
                27 => self.tribute_tick(i, ev),
                32..=49 => self.subplot_tick(i, ch, ev),
                // troop orders live until the next turn start (rules-07 §4)
                51 | 53 | 54 => {}
                52 => self.troop_transfer(i),
                55 => self.prisoners(i, ev),
                56 => {
                    if c == 0 {
                        self.loan_expiry(i, ev);
                    }
                }
                // default (0..5, 7, 12, 16, 17, 20, 21, 23, 35, 50, ≥ 57): deleted at 0
                _ => {
                    if c == 0 {
                        self.heap.delete(i);
                    }
                }
            }
            i += 1;
        }
    }

    /// Yearly office income of `h` (0x5E500), also shown by the privilege entry "Einkommen" (rules-38 §1.2).
    pub fn office_income(&self, h: usize) -> i32 {
        let p = &self.persons[h];
        if !p.has_office() {
            return 0;
        }
        let o = laws::global_office(p);
        let l = if p.loc() < 10 { 2 * self.cities[p.loc() as usize].income_level() as i64 + 300 } else { 400 };
        (crate::career::SALARY[o] as i64 * self.year as i64 * l / 300000) as i32
    }

    /// rules-38 §7 type-55 handler: half the prisoners join the home castle or camp with training 30..59.
    fn prisoners(&mut self, i: usize, ev: &mut Vec<Event>) {
        let r = self.heap.recs[i].clone();
        self.heap.delete(i);
        let (n, prop) = (r[4] as u32, r[2] as usize);
        if n == 0 || prop >= PROPS {
            return;
        }
        let t = self.r(30) as u8;
        let joined = self.add_troops(prop, n / 2, 30 + t);
        ev.push(Event::Prisoners { person: r[1], prop: prop as u8, taken: n.min(255) as u8, joined: joined as i16 });
    }

    /// rules-07 §2.2 type-15 handler.
    fn recruit_tick(&mut self, i: usize) {
        let r = self.heap.recs[i].clone();
        let (o, prop) = (r[1] as usize, r[2] as usize);
        let m = heap::r16(&r, 4) as u32;
        let m = if has(self, o, 36) { m * 4 / 3 } else if has(self, o, 65) { m * 2 } else { m };
        let men = m / (150 + self.r(200));
        if men > 0 {
            self.add_troops(prop, men, 30);
        }
        self.heap.delete(i);
    }

    /// §7.13: the one hint checked this turn; `None` when its condition does not hold.
    fn tip(&self, h: usize, kind: u8) -> Option<Event> {
        let branches: Vec<usize> = self.props_of(h, KIND_BRANCH).collect();
        let tip = |a, b, loc| Some(Event::Tip { kind, a, b, loc });
        match kind {
            0 => branches.iter().find_map(|&p| (0..TRANSPORTERS).find(|&t| self.props[p].trans(t) > 0 && self.props[p].cond(t) <= 50).map(|t| (t as u8, self.props[p].loc()))).and_then(|(t, l)| tip(t, 0, l)),
            1 => (self.year >= 1610 && !branches.iter().any(|&p| self.props[p].has_residence())).then_some(Event::Tip { kind, a: 0, b: 0, loc: 0 }),
            2 => (self.persons[h].cash() > self.net_worth(h) / 2).then_some(Event::Tip { kind, a: 0, b: 0, loc: 0 }),
            3 => (self.persons[h].life() < 15 && self.persons[h].spouse() == NONE && !self.heap.has(31, h as u8)).then_some(Event::Tip { kind, a: 0, b: 0, loc: 0 }),
            _ => branches.iter().find_map(|&p| {
                (0..4).find_map(|k| {
                    let o = self.props[p].order(k);
                    if o[0] != ORDER_PRODUCE {
                        return None;
                    }
                    let (g, w, m) = (o[3] as usize % GOODS, o[4] as i32, o[5] as i32);
                    let r = MEANS_DIV[g];
                    let a = if r * 14 * m < 100 * w { 0 } else if 140 * w < r * 10 * m { 1 } else if w > 49 { 2 } else { return None };
                    Some((a, g as u8, self.props[p].loc()))
                })
            }).and_then(|(a, g, l)| tip(a, g, l)),
        }
    }

    // ---- year end ----

    /// `year_end_world` (0x66220), the documented parts in documented order.
    /// Year-end record pass (rules-04b §3.6): over the computer persons in slot order (`person_find_first(…,
    /// controller 0)`), each person's live records in heap order. Year-end parity run 1619 (docs/FIDELITY.md): the
    /// original's mistress ticks run keeper by keeper in slot order, not in heap order.
    pub fn computer_record_pass(&mut self, ch: &mut dyn Choices, ev: &mut Vec<Event>) {
        for o in self.computers().collect::<Vec<_>>() {
            let mut i = 0;
            while i < self.heap.recs.len() {
                if self.heap.live(i) && self.heap.recs[i][1] as usize == o && self.persons[o].used() && !self.persons[o].human() {
                    let r = &mut self.heap.recs[i];
                    r[3] = r[3].wrapping_sub(1);
                    match r[0] {
                        29 => self.mistress_tick(i, ev),
                        18 => self.trial(i, ch, ev),
                        50 => self.revenge_tick(i, ch, ev),
                        // rules-07 §3b: computer troop records 51..54 have no handler; they are deleted when the
                        // counter reaches 0 and their troops never return (they were never taken)
                        25 => self.siege_tick(i, ch, ev),
                        26 => self.relief_tick(i, ev),
                        27 => self.tribute_tick(i, ev),
                        22 => self.duel(i, ev),
                        28 => {
                            if self.heap.recs[i][3] == 0 {
                                self.hold_feast(i, ev);
                            }
                        }
                        _ => {
                            if self.heap.recs[i][3] == 0 {
                                self.heap.delete(i);
                            }
                        }
                    }
                }
                i += 1;
            }
        }
    }

    pub fn year_end(&mut self, ch: &mut dyn Choices) -> Vec<Event> {
        let mut ev = vec![Event::YearEnd { year: self.year }];
        ev.extend(self.year_end_offices(ch));
        if self.r(100) < 12 {
            self.disaster(&mut ev);
        }
        // rules-37 §3: disaster, settlement_pre for every computer person, auction settlement, window,
        // herald, sabotage, castle step, world fate, world records, contest start, auction announcement
        for p in (0..PERSONS).filter(|&p| self.persons[p].used() && !self.persons[p].human()).collect::<Vec<_>>() {
            self.settlement_pre(p, ch, &mut ev);
        }
        self.auction_settle(ch, &mut ev);
        // rules-12: at the start of the Ereignisse window, one year before the auction
        if self.heap.find(3, NOBODY, ANY).is_some_and(|i| self.heap.recs[i][3] == 1) {
            let song = [7, 0x0E][self.r(2) as usize];
            ev.push(Event::Herald { song });
        }
        self.sabotage(&mut ev);
        self.computer_castles(ch, &mut ev);
        if self.opt(OPT_FATE) && self.r(100) < 15 {
            self.world_fate(&mut ev);
        }
        // world record loop (owner 0xC0): disaster aftermath, contest resolution, auction counter;
        // then the contest announcement (rules-06 §1).
        self.disaster_aftermath();
        self.contest_resolve(&mut ev);
        if let Some(i) = self.heap.find(3, NOBODY, ANY) {
            self.heap.recs[i][3] = self.heap.recs[i][3].saturating_sub(1);
        }
        self.contest_start(ch, &mut ev);
        self.auction_announce(&mut ev);
        // rules-09 §2.1 / rules-05 §5: ten mistress events for computer persons drawn with
        // `person_pick_random(any, computer)` (year-end parity run 1619: the original's picks index the 160 computers,
        // not the 161 used persons, docs/FIDELITY.md).
        for _ in 0..10 {
            if let Some(p) = self.pick_computer(None) {
                self.take_mistress(p, &mut ev);
            }
        }
        self.npc_turns(&mut ev);
        // divine event (rules-06 §3.1 trigger a)
        let prayers = self.globals.b(ga::MIRACLE_CHANCE) as u32;
        if self.r(50) < prayers && self.holder(13, 10).is_some_and(|e| !self.persons[e].human()) {
            self.globals.set_b(ga::MIRACLE_CHANCE, 0);
            self.divine(&mut ev);
        }
        self.computer_record_pass(ch, &mut ev);
        // privilege office income for humans, then treasury reset (rules-04b §4.2)
        for h in self.humans().collect::<Vec<_>>() {
            // turnover, tithe, indulgences: each positive part its own line and credit (rules-40 §27)
            for (kind, x) in self.privilege_income(h).into_iter().enumerate() {
                if x > 0 {
                    self.add_cash(h, x as i32);
                    ev.push(Event::PrivilegeIncome { person: h as u8, kind: kind as u8, amount: x as i32 });
                }
            }
        }
        // "Im nächsten Jahr werden neu gewählt" (rules-44 §3): every year end, offices or seat groups a human could
        // be promoted to; places 13..0, offices high to low, grouped seats under their base index
        let mut next = vec![];
        for l in (0..14usize).rev() {
            let mut i = 9 + 3 * (l >= 10) as i32 - 2 * (l == 13) as i32;
            while i >= 0 {
                if self.humans().any(|h| self.eligibility(h, l, i as usize) == 2) {
                    while i == 1 || i == 2 || (l < 10 && i == 8) {
                        i -= 1;
                    }
                    next.push((l as u8, i as u8));
                }
                i -= 1;
            }
        }
        if !next.is_empty() {
            ev.push(Event::NextElections(next));
        }
        let mut a = 0u32;
        for c in 0..CITIES {
            let pop = self.cities[c].population();
            self.globals.set_d(ga::TAX_TREASURY + 4 * c, pop);
            self.globals.set_d(ga::TITHE_TREASURY + 4 * c, pop);
            a = a.wrapping_add(pop);
        }
        self.globals.set_d(ga::INDULGENCES, a / 2);
        // emperor record (rules-06 §7)
        if let Some(i) = self.heap.find(1, NOBODY, ANY) {
            let r = &mut self.heap.recs[i];
            r[3] = 1;
            if matches!(self.year, 1612 | 1619 | 1637 | 1657) {
                r[4] += 1;
            }
        }
        // evidence pass, last step (rules-04b §2.3)
        for i in self.heap.all(6, ANY, ANY) {
            let t = self.heap.recs[i][4] as usize;
            self.globals.set_d(ga::TURN_FLAGS, 0);
            if self.persons[t].used() {
                let w = laws::offences(self, t);
                laws::add_evidence(self, t, w);
            }
        }
        ev
    }

    /// rules-04b §4.2 year-end privilege income of a human.
    fn privilege_income(&self, h: usize) -> [i64; 3] {
        let p = &self.persons[h];
        let loc = p.loc() as usize;
        let t = |c: usize| self.globals.d(ga::TAX_TREASURY + 4 * c) as i64;
        let k = |c: usize| self.globals.d(ga::TITHE_TREASURY + 4 * c) as i64;
        let county = |f: &dyn Fn(usize) -> i64, pct: i64| -> i64 {
            (0..CITIES).filter(|&c| COUNTY[c] as usize == loc).map(|c| f(c) * pct / 100).sum()
        };
        let all = |f: &dyn Fn(usize) -> i64, pct: i64| -> i64 { (0..CITIES).map(|c| f(c) * pct / 100).sum() };
        let city = |f: &dyn Fn(usize) -> i64| if loc < CITIES { f(loc) / 10 } else { 0 };
        let hs = |c| has(self, h, c);
        let turnover = if hs(21) {
            city(&t)
        } else if hs(39) {
            county(&t, 15)
        } else if hs(41) {
            county(&t, 20)
        } else if hs(30) {
            county(&t, 10)
        } else if hs(68) {
            all(&t, 20)
        } else if hs(70) {
            all(&t, 40)
        } else {
            0
        };
        let tithe = if hs(24) {
            city(&k)
        } else if hs(45) {
            county(&k, 10)
        } else if hs(52) {
            county(&k, 20)
        } else if hs(83) {
            all(&k, 20)
        } else {
            0
        };
        let a = self.globals.d(ga::INDULGENCES) as i64;
        let ind = if hs(49) { a / 30 } else if hs(75) { a / 5 } else { 0 };
        [turnover, tithe, ind]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Auto;
    use crate::world::HumanSetup;

    /// rules-36 §4.4, §6: changing a confirmed production order's workers keeps it confirmed (playtest 2026-11-18).
    #[test]
    fn rules36_edit_keeps_confirmed() {
        let mut st = State::new_game(3, 3, &[HumanSetup::test(3)]).unwrap();
        st.run(&mut Auto);
        let h = st.current as usize;
        let b = st.props_of(h, KIND_BRANCH).next().unwrap();
        let k = (0..4).find(|&k| st.props[b].order(k)[0] == 1).expect("a production order");
        if st.props[b].order(k)[7] != ST_CONFIRMED {
            st.apply(Command::ConfirmOrder { branch: b as u8, slot: k as u8, on: true }, &mut Auto);
        }
        assert_eq!(st.props[b].order(k)[7], ST_CONFIRMED);
        let mut o = st.props[b].order(k);
        o[4] = o[4].saturating_add(1);
        st.apply(Command::SetOrder { branch: b as u8, slot: k as u8, order: o }, &mut Auto);
        assert_eq!((st.props[b].order(k)[4], st.props[b].order(k)[7]), (o[4], ST_CONFIRMED));
    }

    /// rules-02 §7.13: a hint is given only when its condition holds.
    /// rules-38 §8: 14 produced goods, 13 listed: 13 draws of 1 + rand(3), then the tail from the 14th produced
    /// index to 14 shifted by one, the last onto the director pay byte.
    #[test]
    fn production_skill_overflow() {
        let mut st = State::new_game(3, 3, &[HumanSetup::test(3)]).unwrap();
        let h = st.humans().next().unwrap();
        let mut total = [5u32; GOODS];
        total[2] = 0; // goods 0..14 without 2: the 14th produced is index 14
        for g in 0..GOODS {
            st.persons[h].set_skill(g, 50);
        }
        st.persons[h].set_pay_directors(7);
        let mut probe = st.clone();
        let mut ev = vec![];
        st.production_skill(h, &total, false, &mut ev);
        assert_eq!(ev.iter().filter(|e| matches!(e, Event::SkillGain { .. })).count(), 13);
        assert_eq!(st.persons[h].skill(2), 50, "unproduced inside the list: no gain");
        assert_eq!(st.persons[h].skill(14), 50, "the 14th produced good is not listed");
        // 14 draws in all: 13 listed + one tail draw for index 14 (onto byte 0xEF)
        let draws: Vec<u32> = (0..14).map(|_| probe.r(3)).collect();
        assert_eq!(st.persons[h].pay_directors() as u32, 7 + draws[13]);
        assert_eq!(st.r(1000), probe.r(1000), "same number of draws");
    }

    #[test]
    fn rules02_s7_13_tips() {
        let mut st = State::new_game(3, 3, &[HumanSetup::test(3)]).unwrap();
        let h = st.humans().next().unwrap();
        st.year = 1612;
        assert!(matches!(st.tip(h, 1), Some(Event::Tip { kind: 1, .. })), "no residence after 1610");
        st.year = 1605;
        assert!(st.tip(h, 1).is_none(), "too early");
        let b = st.props_of(h, KIND_BRANCH).next().unwrap();
        st.props[b].set_trans(0, 2);
        st.props[b].set_cond(0, 40);
        assert!(matches!(st.tip(h, 0), Some(Event::Tip { kind: 0, a: 0, .. })), "worn transporter");
        st.props[b].set_cond(0, 90);
        assert!(st.tip(h, 0).is_none());
        // good 0 has R = 50: 50 workers with 10 means are balanced but crowded; 60 with 60 are too few
        st.props[b].set_order(0, production_order(0, 50, 10, ST_CONFIRMED));
        assert!(matches!(st.tip(h, 4), Some(Event::Tip { kind: 4, a: 2, .. })));
        st.props[b].set_order(0, production_order(0, 60, 60, ST_CONFIRMED));
        assert!(matches!(st.tip(h, 4), Some(Event::Tip { kind: 4, a: 1, .. })));
        st.props[b].set_order(0, production_order(0, 10, 10, ST_CONFIRMED));
        assert!(st.tip(h, 4).is_none() || matches!(st.tip(h, 4), Some(Event::Tip { kind: 4, a: 0 | 1, .. })));
    }

    fn game(seed: u32) -> State {
        let mut st = State::new_game(seed, 3, &[HumanSetup::test(6), HumanSetup::test(3)]).unwrap();
        st.run(&mut Auto);
        st
    }

    /// rules-01 §3: humans play in slot order; the year advances after the last human.
    #[test]
    fn rules01_s3_turn_order_and_year() {
        let mut st = game(1);
        let (a, b) = (st.humans().next().unwrap(), st.humans().nth(1).unwrap());
        assert_eq!((st.current as usize, st.phase, st.year), (a, 4, 1600));
        st.apply(Command::EndTurn, &mut Auto);
        assert_eq!((st.current as usize, st.year), (b, 1600));
        st.apply(Command::EndTurn, &mut Auto);
        assert_eq!((st.current as usize, st.year), (a, 1601));
    }

    /// The whole loop is a pure function of the seed (no wall clock).
    #[test]
    fn deterministic_for_seed() {
        let play = |seed| {
            let mut st = game(seed);
            for _ in 0..20 {
                st.apply(Command::EndTurn, &mut Auto);
            }
            st.to_snapshot()
        };
        assert_eq!(play(5), play(5));
        assert_ne!(play(5), play(6));
    }

    /// rules-02 §3: the uncle's Bier order runs each year; goods arrive at the next turn start and
    /// wages are billed at turn end.
    #[test]
    fn rules02_s3_uncle_order_produces() {
        let mut st = game(2);
        let h = st.current as usize;
        let b = st.branch_at(h, 6).unwrap();
        let cash = st.persons[h].cash();
        st.apply(Command::EndTurn, &mut Auto);
        assert!(st.persons[h].cash() < cash, "wages paid");
        st.apply(Command::EndTurn, &mut Auto);
        assert_eq!(st.current as usize, h);
        assert!(st.props[b].stock(2) > 0, "Bier produced");
    }

    /// rules-04b §3.6: a prison record with counter 1 costs the next turn.
    #[test]
    fn rules04b_s3_6_prison_skips_turn() {
        let mut st = game(3);
        let h = st.current as usize;
        // counter 2: the own turn-end countdown takes it to 1, as for a sentence given in the tail
        st.heap.alloc(20, h as u8, NONE, 2);
        let other = st.humans().nth(1).unwrap();
        st.apply(Command::EndTurn, &mut Auto);
        st.apply(Command::EndTurn, &mut Auto);
        assert_eq!(st.current as usize, other, "h skipped in 1601");
        st.apply(Command::EndTurn, &mut Auto);
        assert_eq!(st.current as usize, h, "h plays again in 1602");
    }

    /// rules-00 §4: a pending certificate flag (+0x06) raises +0x05 at the next turn start.
    #[test]
    fn rules00_s4_certificate_award() {
        let mut st = game(4);
        let h = st.current as usize;
        st.persons[h].set_certificate_due(1);
        st.turn_start(h);
        assert_eq!((st.persons[h].certificate(), st.persons[h].certificate_due()), (1, 0));
    }

    /// rules-55 §10: the certificate award follows the type-10/11 deliveries and precedes the offers; it also
    /// runs in 1600.
    #[test]
    fn rules55_s10_certificate_after_deliveries() {
        let mut st = game(4);
        let h = st.current as usize;
        let b = st.props_of(h, KIND_BRANCH).next().unwrap();
        st.year = 1601;
        let i = st.heap.alloc(10, h as u8, b as u8, 0).unwrap();
        st.heap.recs[i][4] = 0;
        st.heap.recs[i][5] = 1;
        st.persons[h].set_certificate_due(1);
        let ev = st.turn_start(h);
        let at = |f: &dyn Fn(&Event) -> bool| ev.iter().position(f).unwrap();
        let (built, cert) = (at(&|e| matches!(e, Event::TransportersBuilt { .. })), at(&|e| matches!(e, Event::Certificate { .. })));
        assert!(built < cert && cert < at(&|e| matches!(e, Event::StorageOffer { .. })));
        st.year = 1600;
        st.persons[h].set_certificate_due(1);
        assert!(st.turn_start(h).iter().any(|e| matches!(e, Event::Certificate { .. })));
    }

    /// rules-55 §12: the refresh pass runs first, then every status-3 slot (transports too) is reserved, set to 2
    /// and refreshed again, so an earlier unconfirmed slot sees the reserved goods; infeasible ones drop to 0.
    #[test]
    fn rules55_s12_executed_orders_repeat() {
        let mut st = State::new_world(3);
        let h = st.setup_human(&HumanSetup::test(2)).unwrap();
        let b = st.props_of(h, KIND_BRANCH).next().unwrap();
        let dest = (st.props[b].loc() + 1) % CITIES as u8;
        st.props[b].set_stock(0, 10);
        st.props[b].set_trans(0, 2);
        st.props[b].set_order(0, transport_order(2, 6, 0, 1, 0, dest, ST_POSSIBLE));
        st.props[b].set_order(1, transport_order(2, 8, 0, 1, 0, dest, ST_DONE));
        st.props[b].set_order(2, transport_order(2, 20, 0, 1, 0, dest, ST_DONE));
        st.props[b].set_order(3, [0; 8]);
        st.year = 1600;
        st.turn_start(h);
        let s = |st: &State, k| st.props[b].order(k)[7];
        assert_eq!((s(&st, 0), s(&st, 1), s(&st, 2)), (ST_IMPOSSIBLE, ST_CONFIRMED, ST_IMPOSSIBLE));
        assert_eq!((st.props[b].stock(0), st.props[b].trans(0)), (2, 1), "slot 1 reserved again");
    }

    /// rules-01 §3.3 city growth 0.0 .. 4.9 %.
    #[test]
    fn rules01_s3_3_city_growth() {
        let mut st = State::new_world(8);
        let before: Vec<u32> = st.cities.iter().map(|c| c.population()).collect();
        st.city_growth();
        for c in 0..CITIES {
            let p = st.cities[c].population();
            assert!(p >= before[c] && p <= before[c] * 1049 / 1000);
        }
    }

    /// The original's 1873 autosaves: Lichterheide 4 243 380 grows by 1.4 % and wraps to 7 820.
    #[test]
    fn rules01_s3_3_growth_wraps() {
        assert_eq!(4_243_380u32.wrapping_mul(1014) / 1000, 7820);
    }

    /// Long run: 60 years with two players without panics.
    #[test]
    fn sixty_years() {
        let mut st = game(11);
        while st.year < 1660 && st.humans().count() > 0 {
            st.apply(Command::EndTurn, &mut Auto);
        }
    }

    /// rules-05 §5: the ten year-end mistress events draw from the computer persons only (year-end parity run 1619,
    /// docs/FIDELITY.md); a human never gets a lover from the lottery.
    #[test]
    fn rules05_s5_mistress_lottery_skips_humans() {
        for seed in 1..=30 {
            let mut st = State::new_game(seed, 3, &[HumanSetup::test(6)]).unwrap();
            st.run(&mut Auto);
            let h = st.humans().next().unwrap();
            for _ in 0..5 {
                st.apply(Command::EndTurn, &mut Auto);
                assert!(!st.heap.has(29, h as u8), "seed {seed}: the human got a mistress from the year-end lottery");
            }
        }
    }

    /// rules-04b §3.6: the year-end record pass visits the computer persons in slot order (a person's records in
    /// heap order), not the heap in heap order (year-end parity run 1619: the mistress ticks run keeper by keeper).
    #[test]
    fn rules04b_s3_6_record_pass_in_person_order() {
        let mut st = State::blank(7);
        st.person_init(5, 0, 0, 0);
        st.person_init(9, 0, 1, 0);
        // heap order: 9's record first
        st.heap.alloc(29, 9, NONE, 2);
        st.heap.alloc(29, 5, NONE, 2);
        let (f95, f59) = (st.persons[9].favour(5) as u32, st.persons[5].favour(9) as u32);
        // person 5 ticks first: standing, then x for every record in heap order (9's, then 5's own); then person 9
        let mut r = st.rng;
        r.n(60);
        let a = 10 + r.n(10);
        r.n(10);
        r.n(60);
        r.n(10);
        let b = 10 + r.n(10);
        st.computer_record_pass(&mut Auto, &mut vec![]);
        assert_eq!((st.persons[9].favour(5) as u32, st.persons[5].favour(9) as u32), (f95 + a, f59 + b));
    }

}
