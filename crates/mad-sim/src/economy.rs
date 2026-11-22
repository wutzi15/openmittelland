//! Economy (docs/re/rules-02-economy.md): prices, production, storage, transport, net worth.

use crate::heap::{self, ANY};
use crate::state::*;

// ---- tables (rules-02 §1) ----

pub const GOOD_NAMES: [&str; GOODS] = [
    "Korn", "Fisch", "Bier", "Wolle", "Holz", "Rind", "Erz", "Wein", "Fell", "Glas", "Salz", "Tuch", "Silber",
    "Pfeffer", "Waffen",
];
pub const HOLZ: usize = 4;
pub const WEIN: usize = 7;
pub const WAFFEN: usize = 14;

/// Demand weight D[g][wealth decile] (§1.1).
pub fn demand(g: usize, k: usize) -> i32 {
    if g < 5 { 120 - 4 * k as i32 } else { 84 + 4 * k as i32 }
}
/// Wage base WB[g].
pub fn wage_base(g: usize) -> i32 {
    50 + 5 * g as i32
}
/// Value weight V[g].
pub fn value(g: usize) -> i32 {
    7 + g as i32
}
pub const OUTPUT_BASE: [i32; GOODS] = [155, 221, 218, 239, 186, 211, 200, 192, 280, 177, 182, 209, 179, 164, 256];
/// Workers per 10 means divisor R[g].
pub const MEANS_DIV: [i32; GOODS] = [50, 30, 20, 5, 70, 10, 50, 10, 5, 30, 60, 20, 80, 90, 40];
pub const MEANS_COST: [i32; GOODS] = [100, 200, 150, 50, 500, 100, 500, 100, 100, 300, 700, 300, 1000, 1000, 1000];

/// Transporters (§1.2).
pub const KARREN: usize = 0;
pub const SCHIFF: usize = 1;
pub const BALLON: usize = 2;
pub const DAMPFKARREN: usize = 3;
pub const TAUCHBOOT: usize = 4;
pub const FLUGMASCHINE: usize = 5;
pub const CAP: [i32; TRANSPORTERS] = [100, 1000, 50, 80, 700, 30];
pub const TRIP_COST: [i32; TRANSPORTERS] = [200, 1800, 250, 250, 2200, 150];
pub const PRICE_BASE: [i32; TRANSPORTERS] = [100, 1000, 300, 300, 3000, 500];
pub const BUILD_YEARS: [u8; TRANSPORTERS] = [1, 3, 1, 2, 3, 2];
pub fn needs_water(t: usize) -> bool {
    t == SCHIFF || t == TAUCHBOOT
}
/// Navigable locations (`0x85A00`): cities 0, 3, 4, 5, 8, 9 and the three toll castles.
pub fn navigable(loc: usize) -> bool {
    matches!(loc, 0 | 3 | 4 | 5 | 8 | 9 | 14 | 15 | 16)
}
/// City → county (`0x868D4`).
pub const COUNTY: [u8; CITIES] = [10, 10, 10, 10, 11, 11, 11, 12, 12, 12];
/// Wage factor WF[wealth decile] (§1.3).
pub fn wage_factor(k: usize) -> i32 {
    92 + 2 * k as i32
}

// The three worker formulas below run in i64: at the original's w <= 40 none of their 32-bit products
// overflows, so they equal the EXE's results bit for bit (test `house_i64_matches_i32`); the house rule's
// up to 255 workers would wrap in i32.

/// §3.2 wage cost terms: `((pay + 100) * (((wl + 450) * (((inf - adj) * (WF * w * WB / 100)) / 10)) / 500 / 10)) / 200 / 10`.
pub fn wage_terms(wf: i32, w: i32, g: usize, inf_adj: i32, wage_level: i32, pay: i32) -> i32 {
    let t1 = (wf as i64 * (w * wage_base(g)) as i64) / 100;
    let t2 = (inf_adj as i64 * t1) / 10;
    let t3 = ((wage_level as i64 + 450) * t2) / 500 / 10;
    (((pay as i64 + 100) * t3) / 200 / 10) as i32
}

/// §3.2 forecast terms (without the means): `I * ((pay + 150) * ((wl + 450) * (WF * w * WB / 100) / 500) / 250) / 1000`.
pub fn forecast_terms(wf: i32, w: i32, g: usize, wage_level: i32, pay: i32, inf: i32) -> i32 {
    let t1 = (wf as i64 * (w * wage_base(g)) as i64) / 100;
    let t2 = ((wage_level as i64 + 450) * t1) / 500;
    let t3 = ((pay as i64 + 150) * t2) / 250;
    (inf as i64 * t3 / 1000) as i32
}

/// §3.3 output before the means and the crowding cut.
pub fn output_terms(skill: i32, year_f: i32, w: i32, g: usize, pay: i32, city_f: i32, weather: i32) -> i32 {
    let t1 = ((skill + 250) as i64 * (year_f * w * OUTPUT_BASE[g]) as i64) / 350000;
    let t2 = ((pay + 150) as i64 * t1 * city_f as i64) / 75000;
    (((weather as i64 + 200) * t2) / 300) as i32
}
/// Uncle's starting order by start city: (good, workers, means) (§1.3).
pub const UNCLE: [(u8, u8, u8); CITIES] =
    [(3, 4, 8), (4, 7, 1), (0, 10, 2), (1, 6, 2), (2, 6, 3), (3, 4, 8), (2, 6, 3), (4, 7, 1), (1, 6, 2), (0, 10, 2)];
/// Initial city data (§1.4): population, wealth, yields.
pub const CITY_START: [(u32, u8, [u8; GOODS]); CITIES] = [
    (10000, 70, [120, 50, 75, 140, 25, 120, 75, 50, 25, 140, 50, 75, 25, 120, 140]),
    (5000, 60, [25, 50, 120, 75, 140, 25, 75, 120, 140, 50, 25, 140, 75, 50, 120]),
    (2500, 40, [140, 75, 25, 120, 50, 120, 50, 140, 75, 25, 120, 140, 25, 50, 75]),
    (2500, 40, [120, 140, 25, 50, 75, 140, 25, 75, 50, 120, 75, 120, 50, 140, 25]),
    (5000, 60, [50, 120, 140, 25, 75, 50, 120, 75, 25, 140, 50, 25, 120, 75, 140]),
    (2500, 40, [25, 120, 75, 140, 50, 25, 140, 50, 75, 120, 140, 25, 120, 75, 50]),
    (2500, 40, [50, 25, 140, 75, 120, 50, 140, 25, 120, 75, 120, 50, 140, 25, 75]),
    (5000, 60, [75, 25, 120, 50, 140, 75, 50, 120, 140, 25, 140, 50, 75, 25, 120]),
    (2500, 40, [75, 140, 50, 25, 120, 75, 25, 140, 120, 50, 75, 120, 50, 140, 25]),
    (2500, 40, [140, 75, 50, 120, 25, 140, 120, 25, 50, 75, 25, 50, 140, 120, 75]),
];
/// Castle and camp troop wages (`0x85A4E`, `0x85A56`).
pub const MERC_WAGE: [i32; 4] = [100, 125, 125, 150];
pub const ROBBER_WAGE: [i32; 4] = [75, 100, 100, 125];

pub const CASH_LIMIT: i32 = 9_999_999;

// ---- cash ----

impl State {
    /// 0x43790 add cash: only the upper bound is clamped (rules-60).
    pub fn add_cash(&mut self, p: usize, x: i32) {
        let v = self.persons[p].cash().wrapping_add(x).min(CASH_LIMIT);
        self.persons[p].set_cash_raw(v);
    }
    /// 0x43860 pay cash: only the lower bound is clamped, so a value above the cap survives a payment (rules-60).
    pub fn pay(&mut self, p: usize, x: i32) {
        let v = self.persons[p].cash().wrapping_sub(x).max(-CASH_LIMIT);
        self.persons[p].set_cash_raw(v);
    }
}

// ---- prices (§2) ----

impl State {
    pub fn wealth_decile(&self, c: usize) -> usize {
        (self.cities[c].wealth() / 10).min(9) as usize
    }

    /// §2.1 prices_init.
    pub fn prices_init(&mut self) {
        for c in 0..CITIES {
            let k = self.wealth_decile(c);
            for g in 0..GOODS {
                let e = self.cities[c].yield_(g) as i32;
                let p = (value(g) * demand(g, k) * 3) / (e + 200) + 1;
                self.cities[c].set_price(g, p as u8);
            }
        }
    }

    /// §2.2 inflation index for the current year.
    pub fn inflation(&self) -> u32 {
        let mut i: u32 = 1000;
        for _ in 1600..=self.year as u32 {
            i = i.wrapping_mul(1004) / 1000;
        }
        i + 200
    }

    /// §2.2 prices_update, plus storage levels A/B (§5.2) and transporter prices (§6).
    pub fn prices_update(&mut self) {
        let inf = self.inflation();
        self.globals.set_d(ga::INFLATION, inf);
        for c in 0..CITIES {
            let k = self.wealth_decile(c);
            let city = &mut self.cities[c];
            let pop = city.population();
            let s = (city.siege() != 0) as i32;
            let f = (city.fire() & 0x80 != 0) as i32;
            for g in 0..GOODS {
                if city.sold(g) > pop {
                    city.set_sold(g, pop);
                }
                let rm = (pop as i64 - city.sold(g) as i64 * 5 / 3).max(1) as u32;
                let boost = 100 + (g == HOLZ) as i32 * f * 10 + (g == WAFFEN) as i32 * s * 15;
                let div = 100 - 20 * s - 20 * f;
                let e = city.yield_(g) as i32;
                let base = (value(g) * demand(g, k) * 20) / ((e + 100) * 10);
                let t1 = (inf as i32).wrapping_mul(city.price(g) as i32).wrapping_mul(boost) / div;
                let t2 = (base as u32).wrapping_mul(1000).wrapping_mul(rm) / pop.max(1);
                let new = (t1 as u32).wrapping_add(t2) / 2000;
                let b = (new as u8) as i8;
                city.set_price(g, b.clamp(1, 99) as u8);
            }
        }
        // rules-55 §1: per city after its prices, A0 B0 … A9 B9; the price loop draws nothing, so running
        // it first for all cities gives the same draws
        let y = self.year as u32;
        for c in 0..CITIES {
            let lvl = self.cities[c].storage_level() as u32 / 11;
            let extra = self.cities[c].extra88();
            let a = (y - 1600) / 7 + lvl + 23 + extra + self.r(12);
            let b = (y - 1600) / 7 + lvl + 30 + extra + self.r(12);
            self.cities[c].set_level_a(a as u16);
            self.cities[c].set_level_b(b as u16);
            for t in 0..TRANSPORTERS {
                let tp = (y as i32 - 1550) * PRICE_BASE[t] / 50;
                self.cities[c].set_tprice(t, tp as u16);
            }
        }
    }
}

// ---- production (§3) ----

/// Order slot (§3.1): byte 0 type, 1..2 qty, 3 good, 4 workers/count, 5 means/transporter, 6 dest, 7 status.
pub const ORDER_PRODUCE: u8 = 1;
/// The EXE bytes `branch_add_goods` reads as "prices" for loc 14..20 (rules-40 §26).
const PAST_CITIES: [[u8; GOODS]; 7] = [
    [0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 3],
    [0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3],
    [0, 0, 91, 8, 0, 0, 169, 9, 0, 0, 124, 12, 0, 0, 46],
    [0, 0, 170, 12, 0, 0, 162, 11, 0, 0, 172, 12, 0, 0, 174],
    [16, 25, 28, 28, 28, 29, 29, 29, 30, 30, 28, 28, 30, 8, 8],
    [114, 32, 95, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [32, 76, 101, 101, 114, 32, 95, 0, 0, 0, 0, 0, 0, 0, 0],
];

pub const ST_IMPOSSIBLE: u8 = 0;
pub const ST_POSSIBLE: u8 = 1;
pub const ST_CONFIRMED: u8 = 2;
pub const ST_DONE: u8 = 3;

pub fn production_order(g: u8, w: u8, m: u8, status: u8) -> [u8; 8] {
    [ORDER_PRODUCE, 0, 0, g, w, m, 0, status]
}
pub fn transport_order(kind: u8, qty: u16, g: u8, count: u8, t: u8, dest: u8, status: u8) -> [u8; 8] {
    let q = qty.to_le_bytes();
    [kind, q[0], q[1], g, count, t, dest, status]
}

impl State {
    /// §3.2 wage cost of one production order (no RNG).
    pub fn wage_cost(&self, owner: usize, city: usize, g: usize, w: i32) -> i32 {
        let y = self.year as i32;
        let adj = if y < 1630 { (1630 - y) * 4 } else { -(y - 1630) };
        let inf = self.globals.d(ga::INFLATION) as i32;
        wage_terms(wage_factor(self.wealth_decile(city)), w, g, inf - adj, self.cities[city].wage_level() as i32, self.persons[owner].pay_workers() as i32)
    }

    /// The order book's cost forecast for a production order (rules-02 §3.2, last paragraph):
    /// `I * ((pay + 150) * ((wage_level + 450) * (WF * WB * w / 100) / 500) / 250) / 1000 + m * MC`.
    /// It is what the panel prints; the settlement charges `wage_cost` instead.
    pub fn order_forecast(&self, owner: usize, city: usize, g: usize, w: i32, m: i32) -> i32 {
        let inf = self.globals.d(ga::INFLATION) as i32;
        forecast_terms(wage_factor(self.wealth_decile(city)), w, g, self.cities[city].wage_level() as i32, self.persons[owner].pay_workers() as i32, inf) + m * MEANS_COST[g]
    }

    /// §3.3 branch_effective_skill.
    pub fn effective_skill(&self, branch: usize, g: usize) -> i32 {
        let b = &self.props[branch];
        let owner = b.owner() as usize;
        if g == WEIN && crate::laws::privilege(self, owner, 44) != 0 {
            return 150;
        }
        let mut s = self.persons[owner].skill(g);
        let d = b.director();
        if d != NONE && self.persons[d as usize].skill(g) > s {
            s = self.persons[d as usize].skill(g);
        }
        s as i32
    }

    /// §3.3 bonus/blocked terms for city c.
    pub fn city_blocked(&self, c: usize) -> bool {
        self.heap.all(25, ANY, ANY).into_iter().any(|i| self.heap.recs[i][4] as usize == c)
    }
    pub fn county_bonus(&self, c: usize) -> i32 {
        let county = COUNTY[c];
        if self.heap.all(23, ANY, ANY).into_iter().any(|i| self.heap.recs[i][4] == county) { 10 } else { 0 }
    }

    /// §3.3 output of one production record (good g, workers w, means m) at `branch`. Rolls T.
    pub fn production_output(&mut self, branch: usize, g: usize, w: i32, m: i32) -> i32 {
        let c = self.props[branch].loc() as usize;
        let y = self.year as i32;
        let year_f = if y < 1620 { 155 - (y - 1600) } else { 135 };
        let city_f = self.cities[c].yield_(g) as i32 + 200 + self.county_bonus(c) - 60 * self.city_blocked(c) as i32;
        let skill = self.effective_skill(branch, g);
        let pay = self.persons[self.props[branch].owner() as usize].pay_workers() as i32;
        let mut out = output_terms(skill, year_f, w, g, pay, city_f, self.weather[g] as i32);
        let opt = (w * 10 / MEANS_DIV[g]).max(1);
        if m < opt {
            out = m * out / opt;
        }
        if m > opt {
            out += MEANS_COST[g] * (m - opt) / (2 * self.cities[c].price(g) as i32);
        }
        let t = 30 + self.ri(10);
        if w > t {
            out = out * t / w + out * (w - t) / (2 * w);
        }
        out
    }

    /// §3.1 production feasibility.
    pub fn production_feasible(&self, branch: usize, g: usize) -> bool {
        let b = &self.props[branch];
        let c = b.loc() as usize;
        let need = (g / 5) as u8;
        let d = b.director();
        let cert = self.persons[b.owner() as usize].certificate() >= need
            || (d != NONE && self.persons[d as usize].certificate() >= need);
        cert && c < CITIES && self.cities[c].yield_(g) >= 101
    }

    /// §4.1 transport feasibility (types 2..7).
    pub fn transport_feasible(&self, branch: usize, o: [u8; 8]) -> bool {
        let b = &self.props[branch];
        let (kind, qty, g, count, t, dest) =
            (o[0], u16::from_le_bytes([o[1], o[2]]) as i32, o[3] as usize, o[4] as i32, o[5] as usize, o[6] as usize);
        if g >= GOODS || t >= TRANSPORTERS {
            return false;
        }
        let owner = b.owner() as usize;
        let water = !needs_water(t) || navigable(dest);
        let cnt = 0 < count && count <= b.trans(t) as i32;
        match kind {
            2..=5 => {
                let base = 0 < qty
                    && qty <= b.stock(g) as i32
                    && qty <= CAP[t] * count
                    && cnt
                    && dest != b.loc() as usize
                    && water;
                base && (kind < 4 || self.branch_at(owner, dest).is_some())
            }
            6 | 7 => {
                let own = (0..PROPS).any(|i| {
                    let p = &self.props[i];
                    (p.kind() == KIND_CASTLE || p.kind() == KIND_CAMP) && p.owner() as usize == owner && p.loc() as usize == dest
                });
                own && cnt && water
            }
            _ => false,
        }
    }

    pub fn order_feasible(&self, branch: usize, k: usize) -> bool {
        if self.props[branch].kind() != KIND_BRANCH {
            return self.troop_feasible(branch, k);
        }
        let o = self.props[branch].order(k);
        match o[0] {
            ORDER_PRODUCE => self.production_feasible(branch, o[3] as usize),
            2..=7 => self.transport_feasible(branch, o),
            _ => false,
        }
    }

    /// Slot `k` of property `p` holds an order the book knows (branch kinds 1..7, castle/camp troop orders).
    pub fn has_order(&self, p: usize, k: usize) -> bool {
        if self.props[p].kind() == KIND_BRANCH { self.props[p].order(k)[0] != 0 } else { self.is_troop_order(p, k) }
    }

    /// 0x45820 refresh (rules-36 §6): status 2 is unreserved, then reserved again (2) if feasible, else 0;
    /// any other status: infeasible 0, feasible 1, except 3 stays 3.
    pub fn refresh_orders(&mut self, p: usize) {
        for k in 0..4 {
            if !self.has_order(p, k) {
                continue;
            }
            let mut o = self.props[p].order(k);
            if o[7] == ST_CONFIRMED {
                self.reserve(p, k, false);
            }
            let ok = self.order_feasible(p, k);
            o[7] = match (o[7], ok) {
                (_, false) => ST_IMPOSSIBLE,
                (ST_CONFIRMED | ST_DONE, true) => o[7],
                _ => ST_POSSIBLE,
            };
            self.props[p].set_order(k, o);
            if o[7] == ST_CONFIRMED {
                self.reserve(p, k, true);
            }
        }
    }

    /// 0x49370 / 0x49430: confirming a transport order reserves goods and transporters.
    pub fn reserve(&mut self, branch: usize, k: usize, on: bool) {
        if self.props[branch].kind() != KIND_BRANCH {
            return self.troop_reserve(branch, k, on);
        }
        let o = self.props[branch].order(k);
        if !(2..=7).contains(&o[0]) {
            return;
        }
        let (qty, g, count, t) = (u16::from_le_bytes([o[1], o[2]]), o[3] as usize, o[4], o[5] as usize);
        let b = &mut self.props[branch];
        if on {
            if o[0] <= 5 {
                b.set_stock(g, b.stock(g) - qty);
            }
            b.set_trans(t, b.trans(t) - count);
        } else {
            if o[0] <= 5 {
                b.set_stock(g, b.stock(g) + qty);
            }
            b.set_trans(t, b.trans(t) + count);
        }
    }
}

// ---- storage (§5) ----

impl State {
    /// §5.1 branch_add_goods. Returns the amount given away. Castles and camps (4999 + 4999 m², rules-55 §2) never
    /// give away, but still draw the rand(100) and clamp to 9999 with the 0x85CC1 count.
    pub fn add_goods(&mut self, branch: usize, g: usize, n: u16) -> u32 {
        let mut given = 0u32;
        let b = &mut self.props[branch];
        let s = b.stock(g) as u32 + n as u32;
        if s < 10000 {
            b.set_stock(g, s as u16);
        } else {
            b.set_stock(g, 9999);
            // quirk: counts 9999 - n, not the real excess
            given = given.wrapping_add(9999u32.wrapping_sub(n as u32));
        }
        let cap = (b.wood() as u32 + b.stone() as u32) * 17 + self.rng.n(100);
        let c = self.props[branch].loc() as usize;
        loop {
            let b = &self.props[branch];
            let total: u32 = (0..GOODS).map(|g| b.stock(g) as u32).sum();
            if total <= cap {
                break;
            }
            // castles and camps read past the city table into constant bytes (rules-40 §26)
            let price = |g: usize| match c {
                0..CITIES => self.cities[c].price(g),
                14..=20 => PAST_CITIES[c - 14][g],
                _ => 100,
            };
            let pick = (0..GOODS).filter(|&g| b.stock(g) > 0 && price(g) < 100).min_by_key(|&g| price(g)); // first minimum
            let Some(g) = pick else { break };
            let x = (total - cap).min(b.stock(g) as u32);
            let b = &mut self.props[branch];
            b.set_stock(g, b.stock(g) - x as u16);
            given += x;
        }
        let ga_v = self.globals.d(ga::GIVEN_AWAY).wrapping_add(given);
        self.globals.set_d(ga::GIVEN_AWAY, ga_v);
        given
    }

    /// §6 branch_add_transporters: weighted condition, cap 99.
    pub fn add_transporters(&mut self, branch: usize, t: usize, n: u8, cond: u8) {
        let b = &mut self.props[branch];
        let old = b.trans(t) as u32;
        let tot = old + n as u32;
        if let Some(c) = (b.cond(t) as u32 * old + cond as u32 * n as u32).checked_div(tot) {
            b.set_cond(t, c as u8);
        }
        b.set_trans(t, tot.min(99) as u8);
    }

    /// §5.2 storage offer k for one city: (m², stone?). `cur` = wood + stone m² of the player's branch
    /// there, 15 without a branch (rules-37 §1 #19).
    pub fn storage_offer(&mut self, s: u32, k: usize) -> (u16, bool) {
        let size = ((20 * k as u32 + 15 + self.r(10)) * s / 100 + 1).min(4999);
        (size as u16, self.r(100) < 40)
    }
}

// ---- net worth (§8) ----

impl State {
    pub fn property_value(&self, i: usize) -> i64 {
        let p = &self.props[i];
        match p.kind() {
            KIND_BRANCH => {
                let c = p.loc() as usize;
                let city = &self.cities[c];
                let mut v = city.rights() as i64
                    + city.level_a() as i64 * p.wood() as i64
                    + city.level_b() as i64 * p.stone() as i64
                    + self.residence_value_of(i);
                for g in 0..GOODS {
                    v += city.price(g) as i64 * p.stock(g) as i64;
                }
                for t in 0..TRANSPORTERS {
                    v += p.cond(t) as i64 * p.trans(t) as i64 * city.tprice(t) as i64 / 100;
                }
                for k in 0..4 {
                    let o = p.order(k);
                    if (2..=7).contains(&o[0]) && o[7] == ST_CONFIRMED {
                        let (qty, g, count, t) = (u16::from_le_bytes([o[1], o[2]]) as i64, o[3] as usize, o[4], o[5] as usize);
                        if o[0] <= 5 && g < GOODS {
                            v += qty * city.price(g) as i64;
                        }
                        if t < TRANSPORTERS {
                            v += p.cond(t) as i64 * count as i64 * city.tprice(t) as i64 / 100;
                        }
                    }
                }
                v + city.population() as i64
            }
            KIND_CASTLE => {
                4000 + p.b3() as i64 * 250
                    + p.b4() as i64 * 200
                    + (0..4).map(|k| p.troops(k) as i64 * MERC_WAGE[k] as i64).sum::<i64>()
            }
            KIND_CAMP => {
                500 + p.b3() as i64 * 100
                    + p.b4() as i64 * 80
                    + (0..4).map(|k| p.troops(k) as i64 * ROBBER_WAGE[k] as i64).sum::<i64>()
            }
            _ => 0,
        }
    }


    /// How much of `amount` a castle or camp can take: security up to 100, casemates up to 150 (rules-26 §5.1).
    pub fn fortify_amount(&self, i: usize, which: u8, amount: u8) -> u8 {
        let p = &self.props[i];
        if !matches!(p.kind(), KIND_CASTLE | KIND_CAMP) {
            return 0;
        }
        let (cur, cap) = if which == 0 { (p.b3() as u32, 100u32) } else { (p.b4() as u32, 150u32) };
        (amount as u32).min(cap.saturating_sub(cur)) as u8
    }

    /// Price of adding `amount` to security (`which` 0) or casemates (1): the difference of the property value.
    pub fn fortify_cost(&self, i: usize, which: u8, amount: u8) -> i32 {
        let add = self.fortify_amount(i, which, amount) as i64;
        let (a, b) = if self.props[i].kind() == KIND_CASTLE { (250, 200) } else { (100, 80) };
        (add * if which == 0 { a } else { b }) as i32
    }

    pub fn loans_of(&self, p: usize) -> i64 {
        self.heap.all(56, p as u8, ANY).into_iter().map(|i| heap::r32(&self.heap.recs[i], 5) as i32 as i64).sum()
    }

    /// 0x44640 net worth (i32 like the original).
    pub fn net_worth(&self, p: usize) -> i32 {
        let props: i64 = (0..PROPS).filter(|&i| self.props[i].owner() as usize == p && self.props[i].kind() != KIND_FREE).map(|i| self.property_value(i)).sum();
        (self.persons[p].cash() as i64 + props - self.loans_of(p)) as i32
    }

    /// rules-04 §1 standing(p, k); one RNG call.
    pub fn standing(&mut self, p: usize, k: i32) -> i32 {
        let nw = self.net_worth(p).clamp(100, 999_999);
        (nw * k / 1000) * (70 + self.ri(60)) / 100 + 1
    }

    pub fn branches(&self, p: usize) -> usize {
        self.props_of(p, KIND_BRANCH).count()
    }
}

// ---- transport (§4) ----

impl State {
    /// §4.3 shipment_mishaps on type-9 record `ri`. Layout: +4 kind, +5 u16 qty, +7 good,
    /// +8 count, +9 transporter, +10 dest, +11 condition.
    /// Layout confirmed by the creation code (rules-37 §4).
    pub fn shipment_mishaps(&mut self, ri: usize) -> Vec<crate::Event> {
        let mut ev = vec![];
        let mut r = self.heap.recs[ri].clone();
        let cond = r[11] as u32 * (91 + self.r(9)) / 100;
        r[11] = cond as u8;
        if cond <= self.r(80) {
            let q = heap::r16(&r, 5) as u32 * (70 + self.r(30)) / 100;
            heap::w16(&mut r, 5, q as u16);
            ev.push(crate::Event::CargoDamaged { owner: r[1] });
        }
        if cond <= self.r(90) && r[8] > 1 {
            r[8] -= 1;
            ev.push(crate::Event::TransporterLost { owner: r[1], kind: r[9], dest: r[10] });
        }
        if self.r(100) < 5 && (r[4] == 2 || r[4] == 3) && r[8] > 1 && (r[9] as usize == BALLON || r[9] as usize == DAMPFKARREN) {
            let old = r[10];
            r[10] = ((r[10] as u32 + 1 + self.r(9)) % 10) as u8;
            ev.push(crate::Event::OffCourse { owner: r[1], kind: r[9], old, new: r[10] });
        }
        self.heap.recs[ri] = r;
        ev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rules-60: cash_add clamps only the top, pay only the bottom (50 000 000 − 771 stays, the next income snaps).
    #[test]
    fn rules60_one_sided_cash_clamp() {
        let mut st = State::new_world(2);
        st.persons[0].set_cash_raw(50_000_000);
        st.pay(0, 771);
        assert_eq!(st.persons[0].cash(), 49_999_229);
        st.add_cash(0, 10);
        assert_eq!(st.persons[0].cash(), CASH_LIMIT);
        st.persons[0].set_cash_raw(-50_000_000);
        st.add_cash(0, 5);
        assert_eq!(st.persons[0].cash(), -49_999_995);
        st.pay(0, 1);
        assert_eq!(st.persons[0].cash(), -CASH_LIMIT);
    }

    /// rules-55 §2: a castle never gives away, but add_goods still draws rand(100) and clamps to 9999,
    /// counting 9999 - n into 0x85CC1.
    #[test]
    fn rules55_s2_add_goods_castle() {
        let mut st = State::new_world(3);
        let c = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CASTLE).unwrap();
        st.props[c].set_stock(3, 9990);
        st.globals.set_d(ga::GIVEN_AWAY, 0);
        let mut r = st.rng;
        r.n(100);
        assert_eq!(st.add_goods(c, 3, 20), 9979);
        assert_eq!((st.props[c].stock(3), st.globals.d(ga::GIVEN_AWAY), st.rng), (9999, 9979, r));
    }

    /// rules-02 §1.1: optimal means w*10/R reproduces Appendix II (docs/spec/08 Appendix II).
    #[test]
    fn rules02_s1_1_appendix_ii_ratios() {
        // (workers, means) from Appendix II
        let ratio = [(5, 1), (3, 1), (2, 1), (1, 2), (7, 1), (1, 1), (5, 1), (1, 1), (1, 2), (3, 1), (6, 1), (2, 1), (8, 1), (9, 1), (4, 1)];
        for g in 0..GOODS {
            let (w, m) = ratio[g];
            assert_eq!(w * 10 * 10 / MEANS_DIV[g], m * 10, "{}", GOOD_NAMES[g]);
        }
    }

    /// rules-02 §2.2 inflation: 1600 → 1204.
    #[test]
    fn rules02_s2_2_inflation() {
        let mut s = State::blank(1);
        assert_eq!(s.inflation(), 1204);
        // The doc's "about 1429 / 1690" are real-valued; the specified integer loop truncates.
        s.year = 1650;
        assert_eq!(s.inflation(), 1404);
        s.year = 1700;
        assert_eq!(s.inflation(), 1642);
    }

    /// The i64 worker formulas equal the EXE's i32 wrapping arithmetic wherever that does not overflow:
    /// every w 0..=40 over the input ranges, so `House::default()` stays bit-exact.
    #[test]
    fn house_i64_matches_i32() {
        let wage_old = |wf: i32, w: i32, g: usize, ia: i32, wl: i32, pay: i32| {
            let t1 = (wf * (w * wage_base(g))) / 100;
            let t2 = ia.wrapping_mul(t1) / 10;
            let t3 = (wl + 450).wrapping_mul(t2) / 500 / 10;
            (pay + 100).wrapping_mul(t3) / 200 / 10
        };
        let fc_old = |wf: i32, w: i32, g: usize, wl: i32, pay: i32, inf: i32| {
            let t1 = (wf * (w * wage_base(g))) / 100;
            let t2 = (wl + 450).wrapping_mul(t1) / 500;
            let t3 = (pay + 150).wrapping_mul(t2) / 250;
            inf.wrapping_mul(t3) / 1000
        };
        let out_old = |skill: i32, yf: i32, w: i32, g: usize, pay: i32, cf: i32, wx: i32| {
            let t1 = (skill + 250).wrapping_mul(yf * w * OUTPUT_BASE[g]) / 350000;
            let t2 = (pay + 150).wrapping_mul(t1).wrapping_mul(cf) / 75000;
            (wx + 200) * t2 / 300
        };
        for w in 0..=40 {
            for g in [0, 7, 14] {
                for (wf, ia, wl, pay, inf) in [(92, 1200 - 120, 0, 0, 1200), (110, 3400, 255, 255, 3400), (100, 1500, 60, 100, 1500)] {
                    assert_eq!(wage_terms(wf, w, g, ia, wl, pay), wage_old(wf, w, g, ia, wl, pay));
                    assert_eq!(forecast_terms(wf, w, g, wl, pay, inf), fc_old(wf, w, g, wl, pay, inf));
                }
                for (skill, yf, pay, cf, wx) in [(0, 155, 0, 140, -100), (350, 155, 255, 465, 155), (100, 135, 100, 300, 0)] {
                    assert_eq!(output_terms(skill, yf, w, g, pay, cf, wx), out_old(skill, yf, w, g, pay, cf, wx));
                }
            }
        }
        // the extremes at 255 workers would wrap in i32; i64 keeps them sane
        assert!(output_terms(350, 155, 255, 8, 255, 465, 155) > output_terms(350, 155, 99, 8, 255, 465, 155));
        assert!(wage_terms(110, 255, 14, 3400, 255, 255) > wage_terms(110, 99, 14, 3400, 255, 255));
    }

    /// House rule `max_workers` 255: output grows with the workers and nothing overflows at the extremes.
    #[test]
    fn house_255_workers_output() {
        let mut s = State::new_game(1, 3, &[crate::world::HumanSetup::test(1)]).unwrap();
        let h = s.current as usize;
        let b = s.branch_at(h, 1).unwrap();
        s.persons[h].set_pay_workers(255);
        for g in 0..GOODS {
            let rng = s.rng;
            let mut last = 0;
            for w in [1, 10, 40, 99, 180, 255] {
                s.rng = rng;
                // exactly the means the workers need (§3.3: fewer cut the output, more add their resale value)
                let opt = (w * 10 / MEANS_DIV[g]).max(1);
                let out = s.production_output(b, g, w, opt.min(255));
                let enough = opt <= 255;
                assert!((out >= last || !enough) && out < 30000, "g {g} w {w}: {out} after {last}");
                last = out;
            }
        }
    }

    /// rules-02 §3.2 worked example: 1600, wealth 40, wage level 50, pay 100 %: 10 Korn workers cost 542.
    #[test]
    fn rules02_s3_2_wage_example() {
        let mut s = State::new_world(1);
        s.year = 1600;
        s.globals.set_d(ga::INFLATION, s.inflation());
        s.cities[2].set_wealth(40);
        s.cities[2].set_wage_level(50);
        s.persons[0].set_pay_workers(100);
        assert_eq!(s.wage_cost(0, 2, 0, 10), 542);
    }

    /// rules-02 §2.1 initial price, Fürstenbrück Korn: (7*92*3)/(120+200)+1 = 7.
    #[test]
    fn rules02_s2_1_prices_init() {
        let mut s = State::blank(1);
        for (c, (pop, w, e)) in CITY_START.iter().enumerate() {
            s.cities[c].set_population(*pop);
            s.cities[c].set_wealth(*w);
            s.cities[c].0[0x13..0x22].copy_from_slice(e);
        }
        s.prices_init();
        assert_eq!(s.cities[0].price(0), (7 * 92 * 3 / 320 + 1) as u8);
        assert_eq!(s.cities[0].price(14), (21 * 112 * 3 / 340 + 1) as u8);
    }

    /// rules-02 §2.2 signed-byte wrap: a raw result 128..255 becomes 1.
    #[test]
    fn rules02_s2_2_signed_wrap() {
        let mut s = State::blank(1);
        s.cities[0].set_population(10000);
        s.cities[0].0[0x13..0x22].copy_from_slice(&CITY_START[0].2);
        s.year = 1900;
        for g in 0..GOODS {
            s.cities[0].set_price(g, 99);
        }
        s.prices_update();
        // I(1900) ≈ 3500: T1 ≈ 3500*99 = 346k, /2000 ≈ 173 → wraps to negative → 1.
        assert_eq!(s.cities[0].price(0), 1);
    }

    /// rules-02 §5.1: stock caps at 9999, the counter quirk adds 9999 - n.
    #[test]
    fn rules02_s5_1_add_goods_cap() {
        let mut s = State::new_world(5);
        let b = 0;
        s.props[b] = Property::free();
        s.props[b].set_kind(KIND_BRANCH);
        s.props[b].set_wood(4999);
        s.props[b].set_stone(4999);
        s.props[b].set_stock(0, 9000);
        s.globals.set_d(ga::GIVEN_AWAY, 0);
        s.add_goods(b, 0, 2000);
        assert_eq!(s.props[b].stock(0), 9999);
        assert_eq!(s.globals.d(ga::GIVEN_AWAY), 7999);
    }

    /// rules-02 §5.1: overflow above (m²*17 + rand(100)) gives away the cheapest good first.
    #[test]
    fn rules02_s5_1_overflow_cheapest_first() {
        let mut s = State::new_world(5);
        let b = 0;
        s.props[b] = Property::free();
        s.props[b].set_kind(KIND_BRANCH);
        s.props[b].set_wood(5); // cap 85..184
        s.cities[0].set_price(0, 3);
        s.cities[0].set_price(1, 9);
        s.props[b].set_stock(1, 200);
        s.add_goods(b, 0, 100);
        assert_eq!(s.props[b].stock(0), 0, "Korn (cheaper) goes first");
        assert!((85..=184).contains(&s.props[b].stock(1)));
    }
}
