//! Privilege actions (rules-04b §4.3, menu 0x5F080) and the testament editor (rules-05 §7.1).

use crate::heap::{self, ANY};
use crate::laws::privilege;
use crate::state::*;
use crate::{Ask, Choices, Event};

const BUILDINGS: [i32; 5] = [100000, 50000, 25000, 10000, 5000];

impl State {
    /// Run privilege `code` held by `holder` (the player or someone the player blackmails), paid by
    /// `player`. Arguments by code: 9 a = city (the building is asked); 14..17, 22, 31, 32, 51, 61, 71
    /// a = delta; 25 a = insurance bit value (1, 2, 4, 8); 28, 29 a = city, b = army size;
    /// 48/74/79, 58, 80, 81, 85 a = target person (58: b = amount; 80: b = second person);
    /// 48/74/79, 53, 54/84, 55/86, 85 b = the cost `Command::PrivilegeView` drew; 54/84 a = religion.
    pub fn use_privilege(&mut self, player: usize, holder: usize, code: u8, a: i32, b: i32, ch: &mut dyn Choices) -> Vec<Event> {
        let mut ev = vec![];
        if holder != player && self.persons[holder].favour(player) <= 100 {
            return vec![Event::Refused("not blackmailed")];
        }
        if privilege(self, holder, code) == 0 {
            return vec![Event::Refused("no such privilege")];
        }
        let loc = self.persons[holder].loc() as usize;
        let county = if loc < CITIES { crate::economy::COUNTY[loc] as usize } else { loc };
        let target = (a as usize).min(PERSONS - 1);
        match code {
            1 => ev.push(Event::Health { level: 11 - (self.persons[player].life() / 2).min(11) }),
            9 => {
                // rules-38 §2.3: the five prices are drawn once the city is chosen, then the building is asked
                let c = (a as usize).min(9);
                let prices: [i32; 5] = std::array::from_fn(|k| self.ri(100) + BUILDINGS[k]);
                let k = ch.ask(&Ask::Building { city: c as u8, prices }) as usize;
                if k >= 5 {
                    return ev;
                }
                let price = prices[k];
                // rules-38 §4.1: the flag follows the menu entry's index (< 2 church), not the building: the player's own
                // menu always has Medicus and Fest geben before it, a blackmailed holder's list has Bauwerk at index 1
                self.turn_flag(if holder != player { 0x80000 } else { 0x2000000 });
                self.pay(player, price);
                self.mission_add(player, &[18, 28], price as u32);
                self.wealth_up(c, (5 - k as u32) * 3);
                // rules-38 §4.7: the city's computer persons only
                for x in 0..PERSONS {
                    if self.persons[x].used() && !self.persons[x].human() && self.persons[x].loc() as usize == c {
                        let v = self.ri((5 - k as i32) * 2 + 1);
                        self.favour_increase(player, x, v);
                    }
                }
            }
            14..=17 if loc < CITIES => {
                let o = 0x7F + (code as usize - 14);
                let d = a.signum() * 21;
                let v = (self.cities[loc].0[o] as i32 + d).clamp(0, 100);
                self.cities[loc].0[o] = v as u8;
            }
            22 if loc < CITIES => {
                let v = (self.cities[loc].tax() as i32 + a).clamp(5, 33);
                self.cities[loc].set_tax(v as u8);
            }
            31 | 32 | 51 if (10..13).contains(&county) => {
                let (base, max) = match code {
                    31 => (ga::MAX_INTEREST, 60),
                    32 => (ga::TOLL_SHARE, 33),
                    _ => (ga::TITHE, 25),
                };
                let addr = base + county - 10;
                let old = self.globals.b(addr) as i32;
                let v = (old + a).clamp(0, max);
                self.globals.set_b(addr, v as u8);
                if code == 51 && v != old {
                    self.turn_flag(0x08);
                }
            }
            61 => {
                let v = (self.globals.b(ga::HARSHNESS) as i32 + a.signum() * 21).clamp(0, 100);
                self.globals.set_b(ga::HARSHNESS, v as u8);
            }
            64 => {
                let v = self.globals.b(ga::INQUISITION) ^ 1;
                self.globals.set_b(ga::INQUISITION, v);
            }
            71 => {
                let v = (self.globals.b(ga::STATE_RELIGION) as i32 + a).rem_euclid(3);
                self.globals.set_b(ga::STATE_RELIGION, v as u8);
                self.turn_flag(0x100000);
            }
            25 => {
                let bit = [1u8, 2, 4, 8].into_iter().find(|&x| x as i32 == a).unwrap_or(0);
                let v = self.persons[player].insurance() ^ bit;
                self.persons[player].set_insurance(v);
            }
            27 => {
                let f = self.persons[player].fencing() as i32;
                let cost = 2 * f * f + 302 + self.ri(100);
                if ch.ask(&Ask::FencingLesson { cost }) == 0 {
                    self.pay(player, cost);
                    if f < 95 {
                        let v = f + 1 + self.ri(5);
                        self.persons[player].set_fencing(v as u8);
                    }
                }
            }
            28 | 29 => {
                let c = (a as usize).min(9) as u8;
                let n = b.clamp(0, 50000);
                let sieges = self.heap.all(25, ANY, ANY);
                let besieged = sieges.iter().any(|&i| self.heap.recs[i][4] == c);
                let mine = sieges.iter().filter(|&&i| self.heap.recs[i][1] as usize == player).count();
                let ok = if code == 28 {
                    sieges.len() < 10 && mine < 3 && !besieged
                } else {
                    sieges.iter().any(|&i| self.heap.recs[i][4] == c && self.heap.recs[i][1] as usize != player)
                };
                if !ok {
                    ev.push(Event::Refused("siege not possible"));
                } else if let Some(i) = self.heap.alloc(if code == 28 { 25 } else { 26 }, player as u8, NONE, 4) {
                    self.pay(player, n * 10);
                    self.heap.recs[i][4] = c;
                    heap::w16(&mut self.heap.recs[i], 5, n as u16);
                }
            }
            40 | 69 => self.rotate_advisers(loc),
            48 | 74 | 79 => {
                if self.globals.b(ga::WORKERS_GONE) != 0 {
                    ev.push(Event::Refused("once per turn"));
                } else {
                    self.globals.set_b(ga::WORKERS_GONE, 1);
                    // the cost shown when the dialog opened (`Command::PrivilegeView`)
                    self.pay(player, b);
                    let keep = match code {
                        48 => 80,
                        74 => 70,
                        _ => 60,
                    };
                    for i in self.heap.all(8, target as u8, ANY) {
                        let w = self.heap.recs[i][5] as u32 * keep / 100;
                        self.heap.recs[i][5] = w as u8;
                    }
                    self.turn_flag(0x08);
                }
            }
            53 if (10..13).contains(&county) => {
                self.pay(player, b);
                match self.heap.all(23, ANY, ANY).into_iter().find(|&i| self.heap.recs[i][4] as usize == county) {
                    Some(i) => self.heap.recs[i][3] = 5,
                    None => {
                        if let Some(i) = self.heap.alloc(23, holder as u8, NONE, 5) {
                            self.heap.recs[i][4] = county as u8;
                        }
                    }
                }
                self.turn_flag(0x08);
            }
            54 | 84 => {
                self.pay(player, b);
                if let Some(i) = self.heap.alloc(24, holder as u8, NONE, 5) {
                    self.heap.recs[i][4] = loc as u8;
                    self.heap.recs[i][5] = (a as u8).min(1);
                }
                self.turn_flag(0x08);
            }
            55 | 86 => {
                self.pay(player, b);
                self.divine(&mut ev);
            }
            58 if target != player => {
                self.pay(player, b / 5);
                self.add_cash(target, b);
                let l = self.persons[target].loc();
                for x in 0..PERSONS {
                    if x != target && x != player && self.persons[x].used() && self.persons[x].loc() == l {
                        let v = self.ri(5);
                        self.favour_decrease(player, x, v);
                    }
                }
            }
            // rules-38 §4.3: the menu shows standing(holder, 30) for 80 and 81 but charges nothing
            80 => {
                self.sow_discord(player, target, (b as usize).min(PERSONS - 1));
            }
            // mode 7 (rules-04 §5.4): no office check; an open one is dropped (the app asked 0xBCCD4 first)
            81 => match self.open_trial(player, target, 1) {
                Some(i) => self.heap.delete(i),
                None => {
                    if let Some(i) = self.heap.alloc(18, player as u8, NONE, 1) {
                        self.heap.recs[i][4] = target as u8;
                        self.heap.recs[i][5] = 1;
                    }
                    self.turn_flag(0x08);
                    ev.push(Event::MsgBox { text: 0xBCD6C, args: vec![crate::WArg::P(target as u8)], rect: [160, 180, 320, 120] });
                }
            },
            85 => {
                self.pay(player, b);
                if target != player {
                    for x in 0..PERSONS {
                        if self.persons[x].used() && self.persons[x].church() {
                            // rules-38 §4.2: 10 + rand(15)
                            let v = 10 + self.ri(15);
                            self.favour_increase(target, x, v);
                        }
                    }
                }
                self.turn_flag(0x08);
            }
            _ => ev.push(Event::Refused("not a menu action here")),
        }
        ev
    }

    fn mission_add(&mut self, p: usize, kinds: &[u8], x: u32) {
        if let Some(i) = self.heap.find(0, p as u8, ANY)
            && kinds.contains(&self.heap.recs[i][4]) {
                let v = heap::r32(&self.heap.recs[i], 5).wrapping_add(x);
                heap::w32(&mut self.heap.recs[i], 5, v);
            }
    }

    /// rules-05 §7.1: move the main heir by `d` (±1); index len = Erzbistum.
    pub fn testament_step(&mut self, h: usize, d: i32) {
        let kids: Vec<usize> = self.heap.all(31, h as u8, ANY).into_iter().filter(|&i| self.heap.recs[i][3] < 45).collect();
        let len = kids.len() as i32;
        let cur = kids.iter().position(|&i| self.heap.recs[i][4] & 0x80 != 0).map_or(len, |x| x as i32);
        let new = (len + cur + 1 + d).rem_euclid(len + 1);
        if cur < len {
            self.heap.recs[kids[cur as usize]][4] &= 0x7F;
        }
        if new < len {
            self.heap.recs[kids[new as usize]][4] |= 0x80;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::HumanSetup;

    /// rules-04b §4.3 privilege 22: Bürgermeister sets the city tax within 5..33.
    #[test]
    fn rules04b_s4_3_tax_setting() {
        let mut st = State::new_game(1, 3, &[HumanSetup::test(2)]).unwrap();
        let h = st.current as usize;
        st.persons[h].set_loc(2);
        st.persons[h].set_idx(6);
        st.use_privilege(h, h, 22, 100, 0, &mut crate::Auto);
        assert_eq!(st.cities[2].tax(), 33);
        assert_eq!(st.use_privilege(h, h, 56, 0, 0, &mut crate::Auto), vec![Event::Refused("no such privilege")]);
    }

    /// rules-05 §7.1 testament cycle with two children: 0 → 1 → Erzbistum → 0.
    #[test]
    fn rules05_s7_1_testament() {
        let mut st = State::new_game(1, 3, &[HumanSetup::test(2)]).unwrap();
        let h = st.current as usize;
        for f in [0x80, 0x00] {
            let i = st.heap.alloc(31, h as u8, NONE, 30).unwrap();
            st.heap.recs[i][4] = f;
        }
        let heir = |st: &State| st.heap.all(31, h as u8, ANY).into_iter().position(|i| st.heap.recs[i][4] & 0x80 != 0);
        // (len + h + 1 + d) mod (len + 1) = h + d mod (len + 1)
        st.testament_step(h, 1);
        assert_eq!(heir(&st), Some(1));
        st.testament_step(h, 1);
        assert_eq!(heir(&st), None);
        st.testament_step(h, 1);
        assert_eq!(heir(&st), Some(0));
        st.testament_step(h, -1);
        assert_eq!(heir(&st), None);
    }
}
