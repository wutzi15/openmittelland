//! World state in the original byte layout (docs/re/rules-00-map.md §2, §4-§7).
//!
//! Persons, properties, cities and the misc-globals block are kept as the raw records of
//! `GAMEn.DAT`, with named accessors. This keeps the documented field widths (wrapping, sign) exact
//! and makes save round-trips byte-identical. Unknown bytes survive untouched.

use crate::heap::Heap;
use crate::rng::Rng;

pub const PERSONS: usize = 192;
pub const PROPS: usize = 192;
pub const CITIES: usize = 10;
pub const GOODS: usize = 15;
pub const TRANSPORTERS: usize = 6;
/// "none" marker for person bytes (spouse, director, current person).
pub const NONE: u8 = 0xFF;
/// `current_person` value meaning "no human due" (rules-00 §2 block 6).
pub const NOBODY: u8 = 0xC0;

pub fn rd16(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
pub fn wr16(b: &mut [u8], o: usize, v: u16) {
    b[o..o + 2].copy_from_slice(&v.to_le_bytes());
}
pub fn rd32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
pub fn wr32(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

/// `name, set_name: type @ offset` accessors over a `.0` byte array.
macro_rules! fields {
    ($($get:ident, $set:ident : $t:tt @ $o:expr;)*) => { $(
        pub fn $get(&self) -> $t { fields!(@rd $t, &self.0, $o) }
        pub fn $set(&mut self, v: $t) { fields!(@wr $t, &mut self.0, $o, v) }
    )* };
    (@rd u8, $b:expr, $o:expr) => { $b[$o] };
    (@wr u8, $b:expr, $o:expr, $v:expr) => { $b[$o] = $v };
    (@rd u16, $b:expr, $o:expr) => { rd16($b, $o) };
    (@wr u16, $b:expr, $o:expr, $v:expr) => { wr16($b, $o, $v) };
    (@rd u32, $b:expr, $o:expr) => { rd32($b, $o) };
    (@wr u32, $b:expr, $o:expr, $v:expr) => { wr32($b, $o, $v) };
    (@rd i32, $b:expr, $o:expr) => { rd32($b, $o) as i32 };
    (@wr i32, $b:expr, $o:expr, $v:expr) => { wr32($b, $o, $v as u32) };
}

/// Person record, 248 bytes (rules-00 §4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Person(pub [u8; 0xF8]);

impl Person {
    fields! {
        gender, set_gender: u8 @ 0x00;
        life, set_life: u8 @ 0x02;
        religion, set_religion: u8 @ 0x03;
        spouse, set_spouse: u8 @ 0x04;
        certificate, set_certificate: u8 @ 0x05;
        certificate_due, set_certificate_due: u8 @ 0x06;
        insurance, set_insurance: u8 @ 0x07;
        name, set_name: u8 @ 0x08;
        title, set_title: u8 @ 0x0D;
        loc, set_loc: u8 @ 0x0F;
        flags10, set_flags10: u8 @ 0x10;
        idx, set_idx: u8 @ 0x11;
        level_secular, set_level_secular: u8 @ 0x12;
        level_church, set_level_church: u8 @ 0x13;
        cash, set_cash_raw: i32 @ 0x18;
        nw_cache, set_nw_cache: i32 @ 0x1C;
        pay_directors, set_pay_directors: u8 @ 0xEF;
        pay_workers, set_pay_workers: u8 @ 0xF0;
        pay_mercs, set_pay_mercs: u8 @ 0xF1;
        pay_robbers, set_pay_robbers: u8 @ 0xF2;
        deeds, set_deeds: u8 @ 0xF3;
        colour, set_colour: u8 @ 0xF4;
        hotkey, set_hotkey: u8 @ 0xF5;
        fencing, set_fencing: u8 @ 0xF6;
        denounced, set_denounced: u8 @ 0xF7;
    }
    pub fn empty() -> Self {
        let mut p = Self([0; 0xF8]);
        p.0[0] = NONE;
        p
    }
    pub fn used(&self) -> bool {
        self.0[0] != NONE
    }
    pub fn human(&self) -> bool {
        self.flags10() >> 4 != 0
    }
    pub fn church(&self) -> bool {
        self.flags10() & 0x0F != 0
    }
    pub fn has_office(&self) -> bool {
        self.idx() != 15
    }
    /// favour(self → j)
    pub fn favour(&self, j: usize) -> u8 {
        self.0[0x20 + j]
    }
    pub fn set_favour(&mut self, j: usize, v: u8) {
        self.0[0x20 + j] = v;
    }
    pub fn skill(&self, g: usize) -> u8 {
        self.0[0xE0 + g]
    }
    pub fn set_skill(&mut self, g: usize, v: u8) {
        self.0[0xE0 + g] = v;
    }
}

/// Property record, 96 bytes (rules-00 §5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Property(pub [u8; 0x60]);

pub const KIND_BRANCH: u8 = 0;
pub const KIND_CAMP: u8 = 1;
pub const KIND_CASTLE: u8 = 2;
pub const KIND_FREE: u8 = 3;

impl Property {
    fields! {
        kind, set_kind: u8 @ 0x00;
        loc, set_loc: u8 @ 0x01;
        owner, set_owner: u8 @ 0x02;
        b3, set_b3: u8 @ 0x03;
        b4, set_b4: u8 @ 0x04;
        residence, set_residence: u16 @ 0x03;
        wood, set_wood: u16 @ 0x23;
        stone, set_stone: u16 @ 0x25;
        director, set_director: u8 @ 0x53;
        budget, set_budget: u32 @ 0x54;
    }
    pub fn free() -> Self {
        let mut p = Self([0; 0x60]);
        p.0[0] = KIND_FREE;
        p
    }
    pub fn stock(&self, g: usize) -> u16 {
        rd16(&self.0, 5 + 2 * g)
    }
    pub fn set_stock(&mut self, g: usize, v: u16) {
        wr16(&mut self.0, 5 + 2 * g, v)
    }
    pub fn trans(&self, t: usize) -> u8 {
        self.0[0x27 + t]
    }
    pub fn set_trans(&mut self, t: usize, v: u8) {
        self.0[0x27 + t] = v;
    }
    pub fn cond(&self, t: usize) -> u8 {
        self.0[0x2D + t]
    }
    pub fn set_cond(&mut self, t: usize, v: u8) {
        self.0[0x2D + t] = v;
    }
    pub fn order(&self, k: usize) -> [u8; 8] {
        self.0[0x33 + 8 * k..0x3B + 8 * k].try_into().unwrap()
    }
    pub fn set_order(&mut self, k: usize, o: [u8; 8]) {
        self.0[0x33 + 8 * k..0x3B + 8 * k].copy_from_slice(&o);
    }
    pub fn troops(&self, k: usize) -> u8 {
        self.0[0x58 + k]
    }
    pub fn set_troops(&mut self, k: usize, v: u8) {
        self.0[0x58 + k] = v;
    }
    pub fn training(&self, k: usize) -> u8 {
        self.0[0x5C + k]
    }
    pub fn set_training(&mut self, k: usize, v: u8) {
        self.0[0x5C + k] = v;
    }
    pub fn has_residence(&self) -> bool {
        self.residence() != 0xFFFF
    }
}

/// City record, 140 bytes (rules-00 §6).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct City(pub [u8; 0x8C]);

impl City {
    fields! {
        population, set_population: u32 @ 0x0E;
        wealth, set_wealth: u8 @ 0x12;
        siege, set_siege: u8 @ 0x6D;
        fire, set_fire: u8 @ 0x6E;
        level_a, set_level_a: u16 @ 0x6F;
        level_b, set_level_b: u16 @ 0x71;
        wage_level, set_wage_level: u8 @ 0x7F;
        income_level, set_income_level: u8 @ 0x80;
        building_level, set_building_level: u8 @ 0x81;
        storage_level, set_storage_level: u8 @ 0x82;
        tax, set_tax: u8 @ 0x83;
        rights, set_rights: u32 @ 0x84;
        extra88, set_extra88: u32 @ 0x88;
    }
    pub fn yield_(&self, g: usize) -> u8 {
        self.0[0x13 + g]
    }
    pub fn price(&self, g: usize) -> u8 {
        self.0[0x22 + g]
    }
    pub fn set_price(&mut self, g: usize, v: u8) {
        self.0[0x22 + g] = v;
    }
    pub fn sold(&self, g: usize) -> u32 {
        rd32(&self.0, 0x31 + 4 * g)
    }
    pub fn set_sold(&mut self, g: usize, v: u32) {
        wr32(&mut self.0, 0x31 + 4 * g, v)
    }
    pub fn tprice(&self, t: usize) -> u16 {
        rd16(&self.0, 0x73 + 2 * t)
    }
    pub fn set_tprice(&mut self, t: usize, v: u16) {
        wr16(&mut self.0, 0x73 + 2 * t, v)
    }
}

/// Misc-globals block, linear `0x85A20 .. 0x85F02` (rules-00 §2 block 12). Addresses below are the
/// original linear addresses; `G` converts.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Globals(pub [u8; 0x4E2]);

pub const G_BASE: usize = 0x85A20;
#[allow(non_snake_case)]
pub const fn G(addr: usize) -> usize {
    addr - G_BASE
}

impl Globals {
    pub fn b(&self, addr: usize) -> u8 {
        self.0[G(addr)]
    }
    pub fn set_b(&mut self, addr: usize, v: u8) {
        self.0[G(addr)] = v;
    }
    pub fn d(&self, addr: usize) -> u32 {
        rd32(&self.0, G(addr))
    }
    pub fn set_d(&mut self, addr: usize, v: u32) {
        wr32(&mut self.0, G(addr), v)
    }
    /// Law flag/number byte: book 0 finance, 1 criminal, 2 church; law 0..9 (rules-04b §1).
    /// Finance law 7 (cash cap) is the u32 at 0x85A28 and law 8 sits at 0x85A2C, so finance laws
    /// 8 and 9 are shifted by 3.
    pub fn law(&self, book: usize, i: usize) -> u32 {
        match (book, i) {
            (0, 7) => self.d(0x85A28),
            (0, 8 | 9) => self.b(0x85A21 + i + 3) as u32,
            (0, _) => self.b(0x85A21 + i) as u32,
            (1, _) => self.b(0x85A2E + i) as u32,
            _ => self.b(0x85A38 + i) as u32,
        }
    }
    pub fn set_law(&mut self, book: usize, i: usize, v: u32) {
        match (book, i) {
            (0, 7) => self.set_d(0x85A28, v),
            (0, 8 | 9) => self.set_b(0x85A21 + i + 3, v as u8),
            (0, _) => self.set_b(0x85A21 + i, v as u8),
            (1, _) => self.set_b(0x85A2E + i, v as u8),
            _ => self.set_b(0x85A38 + i, v as u8),
        }
    }
    pub fn forbidden(&self, book: usize, i: usize) -> bool {
        self.law(book, i) != 0
    }
    /// sales[g][c] of the current turn, u32 at 0x85A62 (rules-02 §4.4).
    pub fn sales(&self, g: usize, c: usize) -> u32 {
        self.d(0x85A62 + 4 * (g * 10 + c))
    }
    pub fn set_sales(&mut self, g: usize, c: usize, v: u32) {
        self.set_d(0x85A62 + 4 * (g * 10 + c), v)
    }
    pub fn office_wealth(&self, o: usize) -> u32 {
        self.d(0x85E7A + 4 * o)
    }
    pub fn set_office_wealth(&mut self, o: usize, v: u32) {
        self.set_d(0x85E7A + 4 * o, v)
    }
}

/// Global addresses used by the rules.
pub mod ga {
    pub const MAX_BRANCHES: usize = 0x85A2C;
    pub const TITHE: usize = 0x85A42;
    pub const MAX_INTEREST: usize = 0x85A45;
    pub const TOLL_SHARE: usize = 0x85A48;
    pub const STATE_RELIGION: usize = 0x85A4B;
    pub const INQUISITION: usize = 0x85A4C;
    pub const HARSHNESS: usize = 0x85A4D;
    pub const TURN_FLAGS: usize = 0x85CBA;
    pub const DIFFICULTY: usize = 0x85CBE;
    pub const MIRACLE_CHANCE: usize = 0x85CBF;
    pub const GIVEN_AWAY: usize = 0x85CC1;
    pub const DENOUNCE: usize = 0x85D46;
    pub const AUCTION_PAUSE: usize = 0x85E06;
    pub const PROPOSALS: usize = 0x85E07;
    pub const AVG_SHIPMENT: usize = 0x85E1D;
    pub const INFLATION: usize = 0x85E21;
    pub const TAX_TREASURY: usize = 0x85E25;
    pub const TITHE_TREASURY: usize = 0x85E4D;
    pub const INDULGENCES: usize = 0x85E75;
    pub const WORKERS_GONE: usize = 0x85E79;
}

/// Castle/camp static record (`0x8647C`, 26 bytes, index = location - 14; rules-07 §1). This table
/// lies outside the 12 saved blocks (cities end at 0x8647C), so the original loses toll rates and
/// accumulators on reload; we keep them in our snapshot.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CastleInfo {
    pub toll_rate: u8,
    pub acc: u32,
}

pub const OPT_HISTORY: u16 = 0x001;
pub const OPT_ADVENTURE: u16 = 0x002;
pub const OPT_FATE: u16 = 0x004;
pub const OPT_TIPS: u16 = 0x008;
pub const OPT_WITNESSES: u16 = 0x040;
pub const OPT_ANNOUNCE: u16 = 0x100;
pub const OPT_BRIBE_COURT: u16 = 0x200;
pub const OPT_END_1800: u16 = 0x400;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct State {
    pub names: [[u8; 24]; 32],
    pub persons: Vec<Person>,
    pub props: Vec<Property>,
    pub heap: Heap,
    pub year: u16,
    pub current: u8,
    pub phase: u8,
    pub cities: Vec<City>,
    pub goods: [u8; 0xE1],
    pub globals: Globals,
    pub castles: [CastleInfo; 3],
    /// Option word `0x86AD1` (rules-05 §8; default all on). Lives in the config, not the save.
    pub options: u16,
    pub rng: Rng,
    /// Weather roll per good of the current human turn start (rules-02 §3.3).
    pub weather: [u8; GOODS],
    /// Previous fate event k (`0xC25AC`, not saved by the original; rules-06 §4).
    pub last_fate: u8,
    /// location of the property last returned by `pick_property` (report aid, not saved)
    pub last_pick: u8,
    /// cosmetic draws of the last fate event (tavern / book index, good), report aid, not saved
    pub last_aux: [u8; 2],
    /// Informant film gate of this turn end (rules-21 §4.1): one roll per human turn end, at most one film
    pub informant_gate: bool,
    /// the person last drawn for a fate event (report aid, not saved)
    pub last_person: u8,
    /// amount a fate text shows that was not paid (kinds 50, 54); -1 when unused (report aid, not saved)
    pub last_amount: i32,
    /// `SZENARIO.DAT` (5 x 46 x 30 bytes) from the user's install; empty = battles not simulated.
    /// Game data, not state: not saved.
    pub szenario: Vec<u8>,
    /// Pictures of the battles with a human attacker or defender, in event order (`fight::Replay`);
    /// the app takes them. Not saved.
    pub replays: Vec<crate::fight::Replay>,
    /// House rules (modern options, ours only; off = the original's rules). Not saved: the app sets them.
    pub house: House,
}

/// Optional rule changes; `House::default()` is the original game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct House {
    /// Debt stage 5 always goes to the jury instead of rolling for the last branch (rules-02 §10.2).
    pub debt_mercy: bool,
    /// Largest worker count of a production order (the original: 40). Above 40 the workers past the yearly cap no
    /// longer count half (`production_output`).
    pub max_workers: u8,
}

impl Default for House {
    fn default() -> Self {
        Self { debt_mercy: false, max_workers: 40 }
    }
}

impl State {
    pub fn blank(seed: u32) -> Self {
        Self {
            names: [[0; 24]; 32],
            persons: vec![Person::empty(); PERSONS],
            props: vec![Property::free(); PROPS],
            heap: Heap::default(),
            year: 1600,
            current: NOBODY,
            phase: 0,
            cities: vec![City([0; 0x8C]); CITIES],
            goods: [0; 0xE1],
            globals: Globals([0; 0x4E2]),
            castles: std::array::from_fn(|_| CastleInfo { toll_rate: 5, acc: 0 }),
            options: 0xFFFF,
            rng: Rng::default(),
            weather: [0; GOODS],
            last_fate: 0,
            last_pick: 0xFF,
            last_aux: [0xFF; 2],
            informant_gate: false,
            last_person: 0xFF,
            last_amount: -1,
            szenario: vec![],
            replays: vec![],
            house: House::default(),
        }
        .seeded(seed)
    }
    fn seeded(mut self, seed: u32) -> Self {
        self.rng.srand(seed);
        self
    }
    pub fn opt(&self, bit: u16) -> bool {
        self.options & bit != 0
    }
    pub fn r(&mut self, n: u32) -> u32 {
        self.rng.n(n)
    }
    pub fn ri(&mut self, n: i32) -> i32 {
        self.rng.ni(n)
    }
    pub fn p(&self, i: usize) -> &Person {
        &self.persons[i]
    }
    pub fn pm(&mut self, i: usize) -> &mut Person {
        &mut self.persons[i]
    }
    pub fn humans(&self) -> impl Iterator<Item = usize> + '_ {
        (0..PERSONS).filter(|&i| self.persons[i].used() && self.persons[i].human())
    }
    pub fn computers(&self) -> impl Iterator<Item = usize> + '_ {
        (0..PERSONS).filter(|&i| self.persons[i].used() && !self.persons[i].human())
    }
    pub fn props_of(&self, owner: usize, kind: u8) -> impl Iterator<Item = usize> + '_ {
        (0..PROPS).filter(move |&i| {
            let p = &self.props[i];
            p.kind() == kind && p.owner() as usize == owner
        })
    }
    pub fn branch_at(&self, owner: usize, city: usize) -> Option<usize> {
        self.props_of(owner, KIND_BRANCH).find(|&i| self.props[i].loc() as usize == city)
    }
    /// Holder of (location, index within level), if any.
    pub fn holder(&self, loc: usize, idx: usize) -> Option<usize> {
        (0..PERSONS).find(|&i| {
            let p = &self.persons[i];
            p.used() && p.has_office() && p.loc() as usize == loc && p.idx() as usize == idx
        })
    }
    pub fn name_str(&self, slot: usize) -> String {
        self.names[slot].iter().take_while(|&&b| b != 0).map(|&b| b as char).collect()
    }
    pub fn set_name_str(&mut self, slot: usize, s: &str) {
        self.names[slot] = [0; 24];
        for (d, b) in self.names[slot].iter_mut().zip(s.bytes().take(23)) {
            *d = b;
        }
    }
}
