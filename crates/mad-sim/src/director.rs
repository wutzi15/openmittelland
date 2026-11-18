//! Branch directors (docs/re/rules-09-ai.md §5, 0x59640 / 0x58410).

use crate::economy::*;
use crate::state::*;

/// Order slots the director writes: 0 and 1, transport first (rules-40 §22).
const SLOTS: [usize; 2] = [0, 1];

/// (value, Some((destination, transporter, count, quantity, missing transporters)))
type Sale = (i64, Option<(usize, usize, i64, i64, i64)>);

impl State {
    /// 0x59640: run every director of human h. `flag` 0 at turn start, 1 at turn end.
    pub fn run_directors(&mut self, h: usize, flag: u8) {
        for b in self.props_of(h, KIND_BRANCH).collect::<Vec<_>>() {
            let d = self.props[b].director();
            if d == NONE || !self.persons[d as usize].used() {
                continue;
            }
            let pay = self.persons[h].pay_directors() as i32;
            let base = self.props[b].budget();
            let r = self.ri(151 - pay) as u32;
            let fav = self.persons[d as usize].favour(h) as u32;
            let budget = (base.wrapping_mul(150 + r) / 150).wrapping_mul(600 - fav) / 500;
            self.director_ai(b, d as usize, budget as i64, flag);
        }
    }

    /// Best sale option of good g at branch b: (value, Some((dest, t, n, q, missing))).
    fn best_sale(&self, b: usize, g: usize, budget: i64, blocked: &[bool]) -> Sale {
        let p = &self.props[b];
        let c = p.loc() as usize;
        let stock = p.stock(g) as i64;
        let local = |cc: usize, q: i64| (100 - self.cities[cc].tax() as i64) * q * self.cities[cc].price(g) as i64 / 100;
        let mut best = (local(c, stock), None);
        // a besieged source ships nothing (rules-40 §23)
        for d in (0..CITIES).filter(|&d| d != c && !blocked[c] && !blocked[d]) {
            for t in 0..TRANSPORTERS {
                if needs_water(t) && !(navigable(c) && navigable(d)) {
                    continue;
                }
                let cap = CAP[t] as i64;
                let mut n = (stock + cap - 1) / cap;
                let usable = p.trans(t) as i64 * p.cond(t) as i64 / 100;
                let mut missing = 0;
                if n > usable {
                    missing = (n - p.trans(t) as i64).max(0);
                    n = usable;
                }
                n = n.min(budget * 2 / 10 / TRIP_COST[t] as i64).min(60);
                if n <= 0 {
                    continue;
                }
                let q = stock.min(n * cap);
                let v = local(c, stock - q) + local(d, q) - n * TRIP_COST[t] as i64;
                if v > best.0 {
                    best = (v, Some((d, t, n, q, missing)));
                }
            }
        }
        best
    }

    /// `director_ai(branch, budget, flag)` (0x58410).
    pub fn director_ai(&mut self, b: usize, d: usize, mut budget: i64, flag: u8) {
        let h = self.props[b].owner() as usize;
        let c = self.props[b].loc() as usize;
        let blocked: Vec<bool> = (0..CITIES).map(|x| self.city_blocked(x)).collect();
        // 1. a besieged source city only loses its shipment options (rules-40 §23)
        // 2. release the confirmed slots 0 and 1, any type (rules-40 §22)
        for k in SLOTS {
            let mut o = self.props[b].order(k);
            if o[7] == ST_CONFIRMED {
                self.reserve(b, k, false);
                o[7] = if self.order_feasible(b, k) { ST_POSSIBLE } else { ST_IMPOSSIBLE };
                self.props[b].set_order(k, o);
            }
        }
        // 3. fee
        budget = budget * (100 - self.director_fee_pct(d) as i64) / 100;
        let mut orders = 0;
        let mut transport_done = false;
        // 4./5. sales
        let mut todo: Vec<usize> = (0..GOODS).filter(|&g| self.props[b].stock(g) > 0).collect();
        while !todo.is_empty() {
            let opts: Vec<(usize, Sale)> =
                todo.iter().map(|&g| (g, self.best_sale(b, g, budget, &blocked))).collect();
            let (g, (v, ship)) = *opts.iter().max_by_key(|(g, (v, _))| (*v, -(*g as i64))).unwrap();
            if v <= 0 {
                break;
            }
            todo.retain(|&x| x != g);
            if let Some((dest, t, n, q, missing)) = ship.filter(|_| !transport_done) {
                transport_done = true;
                // rand(6000) drawn for every transporter type (rules-40 §22)
                let kind = if 4000 + (self.r(6000) as i64) < v && t < 2 { 3 } else { 2 };
                let k = SLOTS[orders];
                orders += 1;
                if self.props[b].order(k)[7] == ST_CONFIRMED {
                    self.reserve(b, k, false);
                }
                self.props[b].set_order(k, transport_order(kind, q as u16, g as u8, n as u8, t as u8, dest as u8, 0));
                // not feasible: status 1 (rules-40 §22)
                let ok = self.order_feasible(b, k);
                if ok {
                    self.reserve(b, k, true);
                }
                let mut o = self.props[b].order(k);
                o[7] = if ok { ST_CONFIRMED } else { ST_POSSIBLE };
                self.props[b].set_order(k, o);
                budget -= kind as i64 * TRIP_COST[t] as i64 * n / 2;
                if flag == 0 && missing > 0 {
                    let cost = missing * self.cities[c].tprice(t) as i64;
                    if cost <= budget
                        && let Some(i) = self.heap.alloc(10, h as u8, b as u8, BUILD_YEARS[t])
                    {
                        self.heap.recs[i][4] = t as u8;
                        self.heap.recs[i][5] = missing.min(99) as u8;
                        self.pay(h, cost as i32);
                        budget -= cost;
                    }
                }
            }
            // the remaining stock of g is sold locally
            let s = self.props[b].stock(g) as u32;
            if s > 0 {
                let rev = s * self.cities[c].price(g) as u32;
                self.props[b].set_stock(g, 0);
                let a = self.globals.sales(g, c).wrapping_add(rev);
                self.globals.set_sales(g, c, a);
                let sold = self.cities[c].sold(g) + s;
                self.cities[c].set_sold(g, sold);
                self.add_cash(h, rev as i32);
            }
        }
        // 6. repairs at turn end
        if flag == 1 {
            for t in 0..TRANSPORTERS {
                let (n, cond) = (self.props[b].trans(t), self.props[b].cond(t));
                if n == 0 || cond >= 80 {
                    continue;
                }
                let cost = (100 - cond as i64) * n as i64 * self.cities[c].tprice(t) as i64 / 100;
                if cost < budget
                    && let Some(i) = self.heap.alloc(11, h as u8, b as u8, 1)
                {
                    self.pay(h, cost as i32);
                    budget -= cost;
                    self.heap.recs[i][4] = t as u8;
                    self.heap.recs[i][5] = n;
                    self.props[b].set_trans(t, 0);
                }
            }
        }
        // 7. production
        let y = self.year as i64;
        let l = if y < 1620 { 155 - (y - 1600) } else { 135 };
        let cert = self.persons[h].certificate().max(self.persons[d].certificate()) as usize;
        let mut used: Vec<usize> = vec![];
        let mut fill = 0i64; // storage the new orders will fill: sum of expected_output / 17
        while orders < 2 {
            let mut best: Option<(i64, usize, i64)> = None;
            for g in 0..(5 * cert + 5).min(GOODS) {
                let e = self.cities[c].yield_(g) as i64;
                if e <= 100 || used.contains(&g) {
                    continue;
                }
                let skill = self.effective_skill(b, g) as i64;
                let st = if skill >= 140 { 350 } else { skill + 250 };
                let score = (e + 200) * (l * OUTPUT_BASE[g] as i64 * st / 3500) / 300;
                let key = score * 200 / OUTPUT_BASE[g] as i64;
                // the original scans 14..0 with `>`: the highest good wins a tie (rules-40 §22)
                if best.is_none_or(|(k, _, _)| key >= k) {
                    best = Some((key, g, score));
                }
            }
            let Some((_, g, score)) = best else { break };
            used.push(g);
            let wf = wage_factor(self.wealth_decile(c)) as i64;
            let wage = ((self.cities[c].wage_level() as i64 + 450) * (wf * wage_base(g) as i64 / 100)) / 500;
            let cw = wage + MEANS_COST[g] as i64 * 10 / MEANS_DIV[g] as i64;
            let t = 30 + self.r(10) as i64;
            let mut w = (budget / cw.max(1)).min(t);
            // free storage: wood + stone area minus this call's earlier orders, stock not counted, no clamp (rules-40 §24)
            let p = &self.props[b];
            let free = p.wood() as i64 + p.stone() as i64 - fill;
            if score > 0 && w * score / 100 / 17 > free {
                w = free * 1700 / score;
            }
            let w = w.min(255);
            let m = w * 10 / MEANS_DIV[g] as i64;
            // nothing to order: try the next good
            if w <= 0 || m <= 0 {
                continue;
            }
            let k = SLOTS[orders];
            orders += 1;
            if self.props[b].order(k)[7] == ST_CONFIRMED {
                self.reserve(b, k, false);
            }
            // bytes 0..6 rewritten; an infeasible order keeps the old status byte (rules-40 §22)
            let old = self.props[b].order(k)[7];
            self.props[b].set_order(k, production_order(g as u8, w as u8, m.min(255) as u8, 0));
            let ok = self.order_feasible(b, k);
            if ok {
                self.reserve(b, k, true);
            }
            let mut o = self.props[b].order(k);
            o[7] = if ok { ST_CONFIRMED } else { old };
            self.props[b].set_order(k, o);
            // means are charged once (rules-40 §22)
            budget -= w * wage + m * MEANS_COST[g] as i64;
            fill += w * score / 100 / 17;
        }
        // 8. storage, turn end only (rules-09 §5 step 8)
        if flag != 1 {
            return;
        }
        let (pa, pb) = (self.cities[c].level_a() as i64, self.cities[c].level_b() as i64);
        let price = [pa, pb];
        // offers summed by type; the offer table itself is not reduced [quirk]
        let mut offers = [0i64; 2];
        for k in 0..3 {
            let v = crate::state::rd16(&self.globals.0, G(0x85D01) + 6 * c + 2 * k) as i64;
            if v > 10000 { offers[1] += v - 10000 } else { offers[0] += v }
        }
        let area = |st: &State, t: usize| if t == 1 { st.props[b].stone() } else { st.props[b].wood() } as i64;
        let set_area = |st: &mut State, t: usize, v: i64| {
            if t == 1 { st.props[b].set_stone(v as u16) } else { st.props[b].set_wood(v as u16) }
        };
        let mut need = fill * 140 / 100 - (area(self, 0) + area(self, 1));
        let first = if pb < pa + 7 { 1 } else { 0 };
        for t in [first, 1 - first] {
            if need > 0 && price[t] > 0 {
                let q = (budget / price[t]).min(offers[t]).min(need);
                if q > 0 {
                    self.pay(h, (q * price[t]) as i32);
                    budget -= q * price[t];
                    let v = area(self, t) + q;
                    set_area(self, t, v);
                    offers[t] -= q;
                    need -= q;
                }
            }
        }
        if budget >= 0 {
            let d = pb - pa;
            let (t, cap) = if d < 7 {
                (1, if d < 1 { offers[1] } else { (budget / d).min(offers[1]) })
            } else {
                (0, offers[0])
            };
            let src = 1 - t;
            let q = cap.min(area(self, src)).max(0);
            if q > 0 {
                let v = area(self, src) - q;
                set_area(self, src, v);
                self.add_cash(h, (q * price[src]) as i32);
                let v = area(self, t) + q;
                set_area(self, t, v);
                self.pay(h, (q * price[t]) as i32);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::HumanSetup;

    /// rules-09 §5: a director sells stock and places a feasible production order.
    #[test]
    fn rules09_s5_director_runs() {
        let mut st = State::new_game(1, 3, &[HumanSetup::test(2)]).unwrap();
        let h = st.current as usize;
        let b = st.branch_at(h, 2).unwrap();
        st.props[b].set_director(150);
        st.props[b].set_budget(10000);
        st.props[b].set_stock(0, 500);
        let cash = st.persons[h].cash();
        st.run_directors(h, 0);
        assert_eq!(st.props[b].stock(0), 0);
        assert!(st.persons[h].cash() > cash || st.props[b].order(0)[0] >= 2);
        let orders: Vec<[u8; 8]> = (0..2).map(|k| st.props[b].order(k)).collect();
        assert!(orders.iter().any(|o| o[0] == ORDER_PRODUCE && o[7] == ST_CONFIRMED && st.cities[2].yield_(o[3] as usize) > 100));
    }

    /// rules-09 §5 fee: certificate 0 → (skill 0..4) / 45.
    #[test]
    fn rules09_s5_fee() {
        let mut st = State::new_world(1);
        for g in 0..5 {
            st.persons[150].set_skill(g, 100);
        }
        assert_eq!(st.director_fee_pct(150), 11);
    }
}
