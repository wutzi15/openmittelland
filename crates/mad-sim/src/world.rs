//! World generation and player setup (rules-00 §4, rules-02 §1.4 / §9, rules-03 §2.2 / §4.4,
//! rules-05 §6, rules-07 §1). Large parts of `world_init` (0x4A490) are not documented; every
//! guess is marked SPEC-GAP.

use crate::career::{OFFICE_WEALTH, UNLOCK_C, UNLOCK_S, is_church_office};
use crate::economy::*;
use crate::state::*;

pub struct HumanSetup {
    pub name: String,
    pub gender: u8,
    pub religion: u8,
    pub colour: u8,
    pub hotkey: u8,
    /// Start city 0..9, or 0xFF to let the game pick it (rules-11 §1, `State::pick_start_city`).
    pub city: u8,
    /// Mission index 0..29 from the secret code (rules-10 §2), or None.
    pub mission: Option<u8>,
}

impl HumanSetup {
    pub fn test(city: u8) -> Self {
        Self { name: "Test".into(), gender: 0, religion: 1, colour: 0, hotkey: b'a', city, mission: None }
    }
}

/// +0x6E start values (rules-02 §1.4).
const CITY_6E: [u8; CITIES] = [7, 5, 5, 7, 13, 13, 13, 5, 7, 7];
/// Trade-map points per city (rules-30 §1): P1 label, P2 drawing, P3 owner flag, as x, y.
const CITY_POINTS: [[u16; 6]; CITIES] = [
    [323, 180, 329, 201, 284, 189],
    [81, 197, 58, 207, 36, 193],
    [119, 163, 144, 145, 134, 112],
    [115, 281, 106, 296, 74, 282],
    [352, 122, 410, 91, 374, 84],
    [497, 91, 549, 113, 583, 129],
    [503, 199, 553, 220, 569, 228],
    [471, 279, 500, 309, 551, 314],
    [302, 334, 346, 359, 385, 364],
    [410, 239, 449, 224, 387, 215],
];
const CITY_RIGHTS: [u32; CITIES] = [12000, 8000, 5000, 5000, 8000, 5000, 5000, 8000, 5000, 5000];
const CITY_88: [u32; CITIES] = [10, 5, 0, 0, 5, 0, 0, 5, 0, 0];
/// Persons created by the world generator (rules-00 §4).
pub const NPCS: usize = 160;

impl State {
    /// `pick_name` / person init (rules-00 §11 step 3, rules-05 §6): rand(10) fencing, life
    /// 25 + rand(15), 192 x rand(5) favours 48..52 (self 100), cash 750, skills 100, pay levels
    /// 100, no spouse, no office, religion = state religion, career levels 2/8 (rules-03 §3.1).
    pub fn person_init(&mut self, i: usize, gender: u8, name: u8, controller: u8) {
        let mut p = Person([0; 0xF8]);
        p.set_gender(gender);
        p.set_name(name);
        p.set_fencing(self.r(10) as u8);
        p.set_life(25 + self.r(15) as u8);
        for j in 0..PERSONS {
            p.set_favour(j, 48 + self.r(5) as u8);
        }
        p.set_favour(i, 100);
        p.set_spouse(NONE);
        p.set_idx(15);
        p.set_flags10(controller << 4);
        p.set_level_secular(2);
        p.set_level_church(8);
        p.set_cash_raw(750);
        for g in 0..GOODS {
            p.set_skill(g, 100);
        }
        p.set_pay_directors(100);
        p.set_pay_workers(100);
        p.set_pay_mercs(100);
        p.set_pay_robbers(100);
        p.set_religion(self.globals.b(ga::STATE_RELIGION));
        self.persons[i] = p;
    }

    /// `property_create`: first free slot. Branch 15 m² wood and 1 Karren (rules-00 §5); castle
    /// security 60 / casemates 40, camp camouflage 60 / straw 20, both 4999 m² wood.
    pub fn property_create(&mut self, kind: u8, loc: u8, owner: u8) -> Option<usize> {
        let i = (0..PROPS).find(|&i| self.props[i].kind() == KIND_FREE)?;
        let mut p = Property::free();
        p.set_kind(kind);
        p.set_loc(loc);
        p.set_owner(owner);
        p.set_director(NONE);
        for t in 0..TRANSPORTERS {
            p.set_cond(t, 100);
        }
        match kind {
            KIND_BRANCH => {
                p.set_residence(0xFFFF);
                p.set_wood(15);
                p.set_trans(KARREN, 1);
            }
            KIND_CASTLE => {
                p.set_b3(60);
                p.set_b4(40);
                p.set_wood(4999);
            }
            _ => {
                p.set_b3(60);
                p.set_b4(20);
                p.set_wood(4999);
            }
        }
        self.props[i] = p;
        Some(i)
    }

    /// Random computer person whose place byte (+0x0F) is `loc` (rand(count)), or 0xFF.
    fn random_at(&mut self, loc: u8) -> u8 {
        let pool: Vec<usize> = self.computers().filter(|&x| self.persons[x].loc() == loc).collect();
        if pool.is_empty() {
            return NONE;
        }
        pool[self.r(pool.len() as u32) as usize] as u8
    }

    /// `world_init` (0x4A490), rules-00 §11 in RNG call order.
    pub fn new_world(seed: u32) -> Self {
        let mut st = State::blank(seed);
        crate::laws::init_laws(&mut st.globals);
        for g in 0..GOODS {
            let r = &mut st.goods[15 * g..15 * g + 15];
            for k in 0..10 {
                r[k] = demand(g, k) as u8;
            }
            wr16(r, 10, wage_base(g) as u16);
            r[12] = value(g) as u8;
            wr16(r, 13, OUTPUT_BASE[g] as u16);
        }
        for c in 0..CITIES {
            let (pop, w, e) = CITY_START[c];
            let city = &mut st.cities[c];
            wr16(&mut city.0, 0, 1999 + c as u16);
            city.set_population(pop);
            city.set_wealth(w);
            city.0[0x13..0x22].copy_from_slice(&e);
            city.set_fire(CITY_6E[c]);
            city.set_level_a(20);
            city.set_level_b(25);
            for t in 0..TRANSPORTERS {
                city.set_tprice(t, 100);
            }
            for l in 0x7F..=0x82 {
                city.0[l] = 50;
            }
            city.set_tax(15);
            city.set_rights(CITY_RIGHTS[c]);
            city.set_extra88(CITY_88[c]);
            // rules-30 §1: static trade-map points (label item, drawing item, owner flag) as u16 x, y
            for (k, v) in CITY_POINTS[c].iter().enumerate() {
                city.0[2 + 2 * k..4 + 2 * k].copy_from_slice(&v.to_le_bytes());
            }
        }
        // rules-30 §2: static wage tables (mercenaries, robbers) and two unused words, as in the EXE image
        for (k, v) in [100u16, 125, 125, 150, 75, 100, 100, 125, 250, 500].into_iter().enumerate() {
            let a = 0x85A4E + 2 * k;
            st.globals.set_b(a, v as u8);
            st.globals.set_b(a + 1, (v >> 8) as u8);
        }
        // step 1: the emperor record (type 1, owner 0xC0, counter 1; rules-06 §7).
        // the emperor number starts at 0 (rules-42 §6)
        st.heap.alloc(1, NOBODY, NONE, 1);
        // step 2
        st.prices_init();
        st.prices_update();
        // step 3
        for i in 0..NPCS {
            let female = st.r(4) == 0;
            let name = if female { st.r(70) } else { st.r(117) };
            st.person_init(i, female as u8, name as u8, 0);
            let rel = st.r(2) as u8;
            st.persons[i].set_religion(rel);
            let (loc, idx, t, b0e) = match i {
                0..=99 => {
                    let t = 1 + st.r(2);
                    ((i / 10) as u8, (i % 10) as u8, t, (i / 10) as u8)
                }
                100..=138 => {
                    let t = 3 + st.r(4);
                    let c = (10 + (i - 100) / 13) as u8;
                    (c, ((i - 100) % 13) as u8, t, c + 64 * st.r(2) as u8)
                }
                139..=149 => {
                    let t = 6 + st.r(4);
                    (13, (i - 139) as u8, t, 13 + 64 * st.r(4) as u8)
                }
                _ => (0, 15, 1 + st.r(2), 0),
            };
            let p = &mut st.persons[i];
            p.set_loc(loc);
            p.set_idx(idx);
            p.set_title(t as u8);
            p.0[0x0E] = b0e;
            let o = crate::laws::global_office(p);
            let church = idx != 15 && is_church_office(o);
            p.set_flags10(church as u8);
            p.set_level_secular(UNLOCK_S[o]);
            p.set_level_church(UNLOCK_C[o]);
            for g in 0..GOODS {
                let s = (o as u32 + 2 * g as u32 + 53 + st.rng.n(34)) % 256;
                st.persons[i].set_skill(g, s as u8);
            }
        }
        // step 4: 16 rounds of three computer branches
        for _ in 0..16 {
            for which in 0..3 {
                let c = st.r(10) as u8;
                let place = match which {
                    0 => c,
                    1 => COUNTY[c as usize],
                    _ => 13,
                };
                let owner = st.random_at(place);
                if let Some(b) = st.property_create(KIND_BRANCH, c, owner) {
                    let n = 1 + st.r(5) as u8;
                    st.props[b].set_trans(KARREN, n);
                }
            }
        }
        // steps 5, 6: camps 17..20, then castles 14..16
        for (kind, base, n) in [(KIND_CAMP, 17u8, 4u8), (KIND_CASTLE, 14, 3)] {
            for k in 0..n {
                let place = 10 + st.r(4) as u8;
                let owner = st.random_at(place);
                let Some(pi) = st.property_create(kind, base + k, owner) else { continue };
                for slot in 0..4 {
                    let cnt = if kind == KIND_CAMP { 2 + st.r(4) } else { 5 + st.r(5) };
                    let tr = 30 + st.r(40);
                    st.props[pi].set_troops(slot, cnt as u8);
                    st.props[pi].set_training(slot, tr as u8);
                }
            }
        }
        // step 7
        for (o, w) in OFFICE_WEALTH.iter().enumerate() {
            st.globals.set_office_wealth(o, *w);
        }
        // step 8: every slot, empty ones too
        for i in 0..PERSONS {
            st.init_wealth(i, 100, 10_000_000);
            // written clamped to 0 (0x4AAF6, rules-55 §6)
            let nw = st.net_worth(i).max(0);
            st.persons[i].set_nw_cache(nw);
        }
        // step 9
        st.globals.set_b(ga::AUCTION_PAUSE, 5);
        st.globals.set_d(ga::AVG_SHIPMENT, 100);
        for c in 0..CITIES {
            let pop = st.cities[c].population();
            st.globals.set_d(ga::TAX_TREASURY + 4 * c, pop / 5);
            st.globals.set_d(ga::TITHE_TREASURY + 4 * c, pop / 10);
        }
        st.globals.set_d(ga::INDULGENCES, 5000);
        st
    }

    /// `setup_human_player` (0x4DBC0). Returns the person slot, or None when no slot / name slot is
    /// free, the city's branch rights already belong to a human, or it is after 1670 (rules-10 §6).
    pub fn setup_human(&mut self, s: &HumanSetup) -> Option<usize> {
        let city = if s.city == NONE { self.pick_start_city()? } else { s.city } as usize;
        // rules-11 §1: joining before 1671, at most 6 humans, city free in the rights table
        if city >= CITIES || self.humans().count() >= 6 || self.year >= 1671 || self.rights_holder(city).is_some() {
            return None;
        }
        // rules-30 §3: setup rebuilds the rights table (0x85E08) before the new branch exists
        for loc in 0..21 {
            self.globals.set_b(0x85E08 + loc, self.rights_holder(loc).map_or(0xFF, |h| h as u8));
        }
        let slot = (0..PERSONS).find(|&i| !self.persons[i].used())?;
        let name = (0..32).find(|&n| self.names[n][0] == 0)?;
        if !(0..PROPS).any(|i| self.props[i].kind() == KIND_FREE) {
            return None;
        }
        let others: Vec<i64> = self.humans().map(|h| self.net_worth(h) as i64).collect();
        self.set_name_str(name, &s.name);
        self.person_init(slot, s.gender, name as u8, 1);
        let p = &mut self.persons[slot];
        p.set_religion(s.religion);
        p.set_colour(s.colour);
        p.set_hotkey(s.hotkey);
        let b = self.property_create(KIND_BRANCH, city as u8, slot as u8)?;
        let (g, w, m) = UNCLE[city];
        self.props[b].set_order(0, production_order(g, w, m, ST_CONFIRMED));
        if self.year > 1600 && !others.is_empty() {
            let avg = others.iter().sum::<i64>() / others.len() as i64;
            let d = avg - self.net_worth(slot) as i64;
            if d > 0 {
                self.add_cash(slot, d as i32);
            }
        }
        if let Some(m) = s.mission
            && let Some(i) = self.heap.alloc(0, slot as u8, NONE, 100) {
                self.heap.recs[i][4] = m;
            }
        // rules-30 §2: the cached net worth is set at the end of setup (0x4E719)
        let nw = self.net_worth(slot).max(0);
        self.persons[slot].set_nw_cache(nw);
        Some(slot)
    }

    /// New game: players in order, MaxBr (rules-02 §7.10), difficulty (rules-10 §1), first human due.
    pub fn new_game(seed: u32, difficulty: u8, players: &[HumanSetup]) -> Option<Self> {
        let mut st = State::begin_game(seed, difficulty);
        for p in players {
            st.setup_human(p)?;
        }
        st.finish_game_setup(players.len());
        Some(st)
    }

    /// A new world without humans yet (the app creates the players one by one, `setup_human`).
    pub fn begin_game(seed: u32, difficulty: u8) -> Self {
        let mut st = State::new_world(seed);
        st.globals.set_b(ga::DIFFICULTY, difficulty);
        st
    }

    /// After the last `setup_human` of `n` players: MaxBr, first human due.
    pub fn finish_game_setup(&mut self, n: usize) {
        let n = n.max(1);
        self.globals.set_b(ga::MAX_BRANCHES, ((n + 9) / n) as u8);
        let first = self.humans().next().map_or(NOBODY, |h| h as u8);
        self.current = first;
        self.phase = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rules-02 §9 start position: 750 cash, one branch with 15 m² and 1 Karren, uncle order confirmed.
    #[test]
    fn rules02_s9_start_position() {
        let st = State::new_game(42, 3, &[HumanSetup::test(6)]).unwrap();
        let h = st.current as usize;
        assert!(st.persons[h].human());
        assert_eq!(st.persons[h].cash(), 750);
        let b = st.branch_at(h, 6).unwrap();
        assert_eq!(st.props[b].wood(), 15);
        assert_eq!(st.props[b].trans(KARREN), 1);
        // Falkenhorst: Bier 6/3 (H p.13)
        assert_eq!(st.props[b].order(0), production_order(2, 6, 3, ST_CONFIRMED));
        assert_eq!(st.globals.b(ga::MAX_BRANCHES), 10);
    }

    /// rules-02 §7.10 MaxBr = (n + 9) / n.
    #[test]
    fn rules02_s7_10_max_branches() {
        let ps: Vec<HumanSetup> = (0..4).map(HumanSetup::test).collect();
        let st = State::new_game(1, 3, &ps).unwrap();
        assert_eq!(st.globals.b(ga::MAX_BRANCHES), 3);
        assert!(State::new_game(1, 3, &[HumanSetup::test(1), HumanSetup::test(1)]).is_none(), "rights per city");
    }

    /// rules-05 §6: life 25..39; rules-03 §1: favour 48..52, self 100.
    #[test]
    fn rules05_s6_person_init_ranges() {
        let st = State::new_world(77);
        for i in 0..NPCS {
            let p = &st.persons[i];
            assert!((25..=39).contains(&p.life()));
            assert_eq!(p.favour(i), 100);
            assert!((48..=52).contains(&p.favour((i + 1) % NPCS)));
        }
        assert_eq!((0..NPCS).filter(|&i| st.persons[i].has_office()).count(), 150);
    }

    /// Soak: idle humans end their turns for many years with the whole computer world running
    /// (`MAD_SOAK=<seeds>:<years>` for a long run). Nothing may panic, and the state must stay
    /// loadable (save round trip) at the end.
    #[test]
    fn soak_idle_humans() {
        let (seeds, years) = std::env::var("MAD_SOAK")
            .ok()
            .and_then(|v| v.split_once(':').map(|(a, b)| (a.parse().unwrap_or(2), b.parse().unwrap_or(10))))
            .unwrap_or((2u32, 10u32));
        let szenario = std::env::var("MAD_DATA").ok().and_then(|p| std::fs::read(std::path::Path::new(&p).join("SZENARIO.DAT")).ok()).unwrap_or_default();
        for seed in 1..=seeds {
            let mut st = State::new_game(seed, 3, &[HumanSetup::test(4), HumanSetup::test(7)]).unwrap();
            st.szenario = szenario.clone();
            let start = st.year;
            st.run(&mut crate::Auto);
            while st.year < start + years as u16 {
                let ev = st.apply(crate::Command::EndTurn, &mut crate::Auto);
                if ev.iter().any(|e| matches!(e, crate::Event::GameOver | crate::Event::AllPlayersOut)) {
                    break;
                }
                st.replays.clear();
            }
            let back = State::from_game_dat(&st.to_game_dat(), 1).expect("snapshot loads");
            assert_eq!(back.year, st.year, "seed {seed}");
        }
    }
}
