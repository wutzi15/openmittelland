//! Long-play soak: random but valid answers and Kontor commands for many game-years, with invariant checks after
//! every command. Every range below cites the `docs/re` section it comes from. The example `soak` runs this at
//! scale (threads, catch_unwind, watchdog); `play_game` itself has no wall clock.

use crate::heap::{ANY, HEAP_BYTES, SIZE};
use crate::state::*;
use crate::world::HumanSetup;
use crate::{Ask, Choices, Command, Event};

/// Bot RNG, independent of the game's Watcom RNG (xorshift64*).
#[derive(Clone)]
pub struct Rnd(pub u64);
impl Rnd {
    pub fn step(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub fn n(&mut self, n: u32) -> u32 {
        if n == 0 { 0 } else { (self.step() >> 33) as u32 % n }
    }
    pub fn pick<'a, T>(&mut self, v: &'a [T]) -> Option<&'a T> {
        if v.is_empty() { None } else { Some(&v[self.n(v.len() as u32) as usize]) }
    }
}

/// Answers every `Ask` with a seeded random answer inside the documented range (ranges: `Ask` doc comments in
/// `lib.rs`, the app's menus). Counts the questions of the current command: a runaway question loop trips `cap`.
pub struct Bot {
    pub rnd: Rnd,
    pub asks: u32,
    pub cap: u32,
    pub humans: u32,
}

impl Choices for Bot {
    fn ask(&mut self, q: &Ask) -> u32 {
        self.asks += 1;
        assert!(self.asks <= self.cap, "question cap {} hit at {q:?}", self.cap);
        let r = &mut self.rnd;
        match q {
            Ask::Vote { candidates, .. } => r.n(candidates.len() as u32),
            Ask::Gift { .. } => r.n(4),
            Ask::Vows { .. } => r.n(3),
            Ask::WeddingFeast { .. } => r.n(5),
            Ask::Building { .. } => r.n(6),
            Ask::Plea => r.n(9),
            Ask::Sentence { .. } => r.n(7),
            // 0 nobody, k-th human presses, rarely Esc
            Ask::AuctionTick { .. } => match r.n(20) {
                0 => crate::AUCTION_ESC,
                1..=4 => 1 + r.n(self.humans),
                _ => 0,
            },
            Ask::TrialWitnesses { .. } | Ask::TrialBribe { .. } | Ask::JuryBribe { .. } => r.n(5000),
            Ask::ContestEntry { .. } => r.n(32),
            Ask::PickDirector { candidates, .. } => r.n(candidates.len() as u32 + 1),
            // nibble per group: 0 keep, else tactic + 1 (`set_tactic` ignores tactics > 10); sometimes ESC
            Ask::Tactics { .. } => {
                if r.n(10) == 0 {
                    u32::MAX
                } else {
                    (0..8).fold(0, |a, k| a | r.n(12) << (4 * k))
                }
            }
            // menus of at most 3 lines are clamped by the sim; spinners take amounts
            Ask::Plot { .. } => {
                if r.n(8) == 0 { r.n(10000) } else { r.n(3) }
            }
            Ask::SiegeWon { .. } => r.n(3),
            // 1 would quit the game
            Ask::ContinueAfterMission { .. } => 0,
            _ => r.n(2),
        }
    }
}

/// Game setup varied per seed.
pub struct Setup {
    pub seed: u32,
    pub humans: u8,
    pub difficulty: u8,
    pub missions: bool,
    pub options: u16,
    /// Keep the humans solvent: cash topped up through `add_cash` when low. Random play goes broke within years;
    /// the late game is what the soak is for. Off (every 8th seed) = plain rules.
    pub sustain: bool,
    /// Keep the humans alive too (life kept above 7); off in half of the sustained games so deaths and successions run.
    pub immortal: bool,
}

impl Setup {
    pub fn from_seed(seed: u32) -> Self {
        let mut r = Rnd((seed as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let difficulty = r.n(3) as u8;
        Self {
            seed,
            humans: 1 + r.n(4) as u8,
            difficulty,
            missions: r.n(2) == 0,
            options: if r.n(3) == 0 { 0xFFFF } else { r.n(0x800) as u16 },
            sustain: !seed.is_multiple_of(8),
            immortal: seed % 2 == 1,
        }
    }
}

#[derive(Debug)]
pub struct Failure {
    pub seed: u32,
    pub year: u16,
    pub last: String,
    pub what: String,
}

pub struct Stats {
    pub years: u32,
    pub commands: u32,
    /// Event variant counts (coverage of the systems exercised).
    pub events: std::collections::BTreeMap<String, u64>,
    pub over: bool,
    /// The events that ended the game (deaths, bankruptcies, player-out), for the report.
    pub end: String,
}

/// Play one game for `years` (or until it is over) with `cmds` random commands per human turn.
/// `progress(year, last command)` is called before each command for an outside watchdog.
pub fn play_game(setup: &Setup, years: u16, cmds: u32, szenario: &[u8], progress: &mut dyn FnMut(u16, &str)) -> Result<Stats, Failure> {
    play(setup, years, cmds, szenario, progress).map(|(s, _)| s)
}

/// `play_game` that also hands back the final state (the app soak starts its screens from it).
pub fn play(setup: &Setup, years: u16, cmds: u32, szenario: &[u8], progress: &mut dyn FnMut(u16, &str)) -> Result<(Stats, State), Failure> {
    let mut r = Rnd(setup.seed as u64 ^ 0xD1B5_4A32_D192_ED03);
    let mut cities: Vec<u8> = (0..10).collect();
    let players: Vec<HumanSetup> = (0..setup.humans)
        .map(|i| {
            let c = cities.remove(r.n(cities.len() as u32) as usize);
            HumanSetup {
                name: format!("P{i}"),
                gender: r.n(2) as u8,
                religion: r.n(2) as u8,
                colour: i,
                hotkey: b'a' + i,
                // the game's own pick only for the last player (it could collide with a later explicit city)
                city: if i + 1 == setup.humans && r.n(3) == 0 { NONE } else { c },
                mission: setup.missions.then(|| setup.difficulty * 10 + r.n(10) as u8),
            }
        })
        .collect();
    let fail = |st: &State, last: &str, what: String| Failure { seed: setup.seed, year: st.year, last: last.into(), what };
    let Some(mut st) = State::new_game(setup.seed, setup.difficulty, &players) else {
        return Err(Failure { seed: setup.seed, year: 1600, last: "new_game".into(), what: "new_game returned None".into() });
    };
    st.szenario = szenario.to_vec();
    st.options = setup.options;
    let mut bot = Bot { rnd: Rnd(r.step() | 1), asks: 0, cap: 100_000, humans: setup.humans as u32 };
    let start = st.year;
    let mut stats = Stats { years: 0, commands: 0, over: false, end: String::new(), events: Default::default() };
    let mut last = String::from("run");
    progress(st.year, &last);
    let ev = st.run(&mut bot);
    count(&mut stats, &ev);
    check(&st).map_err(|e| fail(&st, &last, e))?;
    if over(&ev) {
        stats.over = true;
        stats.end = ending(&ev);
        return Ok((stats, st));
    }
    while st.year < start + years {
        let n = r.n(cmds + 1);
        let h = st.current as usize;
        if setup.sustain {
            if st.persons[h].cash() < 20_000 {
                st.add_cash(h, 200_000);
            }
            if setup.immortal && st.persons[h].life() < 8 {
                st.persons[h].set_life(10 + r.n(10) as u8);
            }
        }
        for _ in 0..n {
            let h = st.current as usize;
            let cmd = random_command(&st, h, &mut r, !setup.sustain);
            last = format!("{cmd:?}");
            progress(st.year, &last);
            bot.asks = 0;
            let ev = st.apply(cmd, &mut bot);
            count(&mut stats, &ev);
            stats.commands += 1;
            check(&st).map_err(|e| fail(&st, &last, e))?;
            if over(&ev) {
                stats.over = true;
                stats.end = ending(&ev);
                stats.years = (st.year - start) as u32;
                return Ok((stats, st));
            }
            if st.current >= NOBODY || !(1..=4).contains(&st.phase) {
                // GiveUp ends the turn
                break;
            }
        }
        last = "EndTurn".into();
        progress(st.year, &last);
        bot.asks = 0;
        let ev = st.apply(Command::EndTurn, &mut bot);
        count(&mut stats, &ev);
        st.replays.clear();
        check(&st).map_err(|e| fail(&st, &last, e))?;
        if over(&ev) {
            stats.over = true;
            stats.end = ending(&ev);
            break;
        }
        // every so often continue from a GAMEn.DAT reload, as a player would (the original keeps no RNG, weather
        // or castle table in the save, rules-00 §2)
        if r.n(9) == 0 {
            last = "reload".into();
            let d = st.to_game_dat();
            let mut back = State::from_game_dat(&d, r.n(1 << 30)).map_err(|e| fail(&st, &last, e))?;
            back.szenario = std::mem::take(&mut st.szenario);
            back.options = st.options;
            st = back;
            check(&st).map_err(|e| fail(&st, &last, e))?;
        }
    }
    stats.years = (st.year - start) as u32;
    Ok((stats, st))
}

fn ending(ev: &[Event]) -> String {
    ev.iter()
        .filter(|e| matches!(e, Event::PlayerOut { .. } | Event::Died { .. } | Event::Bankrupt { .. } | Event::Succession { .. } | Event::MissionDone { .. } | Event::GameOver | Event::AllPlayersOut))
        .map(|e| format!("{e:?}"))
        .collect::<Vec<_>>()
        .join("; ")
}

fn over(ev: &[Event]) -> bool {
    ev.iter().any(|e| matches!(e, Event::GameOver | Event::AllPlayersOut))
}

fn count(stats: &mut Stats, ev: &[Event]) {
    for e in ev {
        let s = format!("{e:?}");
        let n = s.find([' ', '{', '(']).unwrap_or(s.len());
        *stats.events.entry(s[..n].to_string()).or_default() += 1;
    }
}

/// A random Kontor command of human `h`, valid in form (the sim refuses what the rules forbid).
/// `wild`: amounts far beyond the cash too (ruins the player within a few years).
pub fn random_command(st: &State, h: usize, r: &mut Rnd, wild: bool) -> Command {
    let branches: Vec<u8> = st.props_of(h, KIND_BRANCH).map(|b| b as u8).collect();
    let forts: Vec<u8> = (0..PROPS).filter(|&i| st.props[i].owner() as usize == h && matches!(st.props[i].kind(), KIND_CAMP | KIND_CASTLE)).map(|i| i as u8).collect();
    let own: Vec<u8> = branches.iter().chain(&forts).copied().collect();
    let persons: Vec<u8> = (0..PERSONS).filter(|&i| st.persons[i].used()).map(|i| i as u8).collect();
    let all_props: Vec<u8> = (0..PROPS).filter(|&i| st.props[i].kind() != KIND_FREE).map(|i| i as u8).collect();
    let any_branch = *r.pick(&branches).unwrap_or(&0xFF);
    let any_own = *r.pick(&own).unwrap_or(&0xFF);
    let any_fort = *r.pick(&forts).unwrap_or(&any_own);
    let person = *r.pick(&persons).unwrap_or(&0);
    let prop = *r.pick(&all_props).unwrap_or(&0);
    let cash = st.persons[h].cash().max(0);
    let money = |r: &mut Rnd| match r.n(8) {
        0 => r.n(100) as i32,
        1 => r.n(10_000) as i32,
        2 if wild => r.n(1_000_000) as i32,
        3 if wild => r.n(20_000_000) as i32 - 10_000_000,
        _ => (cash as i64 * r.n(100) as i64 / 400) as i32,
    };
    match r.n(60) {
        0..=7 => {
            // rules-02 §3.1 / §4.1: production (1), transports 2..7, castle/camp orders 0x11..0x13 / 0x21..0x23
            let branch = if r.n(6) == 0 { any_own } else { any_branch };
            let fort = branch != any_branch || branches.is_empty();
            let order = match (fort, r.n(4)) {
                (true, _) => {
                    let kind = [0x11, 0x12, 0x13, 0x21, 0x22, 0x23][r.n(6) as usize];
                    [kind, r.n(8) as u8, 0, r.n(100) as u8, r.n(100) as u8, r.n(100) as u8, r.n(100) as u8, 0]
                }
                (false, 0) => crate::economy::production_order(r.n(15) as u8, r.n(60) as u8, r.n(12) as u8, 0),
                _ => {
                    let qty = if r.n(3) == 0 { r.n(10_000) } else { r.n(200) } as u16;
                    crate::economy::transport_order(2 + r.n(6) as u8, qty, r.n(15) as u8, 1 + r.n(5) as u8, r.n(6) as u8, r.n(21) as u8, 0)
                }
            };
            Command::SetOrder { branch, slot: r.n(4) as u8, order }
        }
        8..=11 => Command::ConfirmOrder { branch: any_own, slot: r.n(4) as u8, on: r.n(4) != 0 },
        12 => Command::BuyStorage { branch: any_branch, k: r.n(3) as u8 },
        13 => Command::SellStorage { branch: any_branch, m2: r.n(200) as u16, stone: r.n(2) == 0 },
        14 => Command::BuildTransporters { branch: any_branch, t: r.n(6) as u8, n: r.n(5) as u8 },
        15 => Command::SellTransporters { branch: any_branch, t: r.n(6) as u8, n: r.n(3) as u8 },
        16 => Command::RepairTransporters { branch: any_branch, t: r.n(6) as u8, n: r.n(3) as u8 },
        17 => Command::SetPay { which: r.n(4) as u8, pct: r.n(200) as u8 },
        18 => Command::Apply { loc: r.n(14) as u8, idx: r.n(15) as u8, on: r.n(3) != 0 },
        19 => Command::OfficeTree,
        20 => Command::Bribe { target: person, amount: money(r) },
        21 => Command::Spies { target: person, price: r.n(5000) },
        22 => Command::Saboteur { target: person, price: r.n(5000) },
        23 => Command::Blackmail { target: person },
        24 => Command::Denounce { victim: person, listener: *r.pick(&persons).unwrap_or(&0) },
        25 => Command::Matchmaker,
        26 => Command::TakeMistress,
        27 => Command::Recruit { prop: any_fort, money: r.n(12_000) as u16 },
        28 => Command::Manoeuvre { prop: any_fort },
        29 => Command::TollRate { prop: any_fort, rate: r.n(16) as u8 },
        30 => Command::Dismiss { prop: any_fort, slot: r.n(4) as u8, amount: r.n(20) as u8 },
        31 => Command::Fortify { prop: any_fort, which: r.n(2) as u8, amount: r.n(160) as u8 },
        32 => Command::FileTrial { target: person },
        33 => Command::PlanFeast { branch: any_branch, size: r.n(4) as u8, musicians: r.n(5) as u8, years: r.n(4) as u8 },
        34 => Command::Insult { target: person },
        35 => Command::AuctionTransporters { branch: any_branch, t: r.n(6) as u8, n: r.n(3) as u8 },
        36 => Command::AuctionProperty { prop: any_own },
        37 => Command::CastleOffers { prop: any_fort },
        38 => Command::Interest { prop, on: r.n(3) != 0 },
        39 => {
            let buyer = st.heap.all(17, ANY, any_fort).first().map_or(person, |&i| st.heap.recs[i][1]);
            Command::SellCastle { prop: any_fort, buyer }
        }
        40 => {
            const CODES: [u8; 31] = [1, 9, 14, 15, 16, 17, 22, 25, 27, 28, 29, 31, 32, 40, 48, 51, 53, 54, 55, 58, 61, 64, 69, 71, 74, 79, 80, 81, 84, 85, 86];
            let holder = if r.n(3) == 0 { person } else { h as u8 };
            let a = match r.n(3) {
                0 => r.n(10) as i32,
                1 => person as i32,
                _ => r.n(30) as i32 - 15,
            };
            Command::Privilege { holder, code: CODES[r.n(31) as usize], a, b: money(r) }
        }
        41 => Command::EditLaw { book: r.n(3) as u8, i: r.n(10) as u8, delta: r.n(21) as i32 - 10 },
        42 => Command::Testament { d: r.n(5) as i8 - 2 },
        43 => Command::HireDirector { branch: any_branch },
        44 => Command::FireDirector { branch: any_branch },
        45 => Command::SetBudget { branch: any_branch, budget: r.n(50_000) },
        46 => Command::BuyBranch { city: r.n(10) as u8 },
        47 if r.n(4) == 0 => Command::SellBranch { branch: any_branch },
        48 => Command::SellStock { branch: any_branch, good: if r.n(5) == 0 { 0xFF } else { r.n(15) as u8 }, qty: r.n(500) as u16 },
        49 => Command::BuyResidence { branch: any_branch, k: r.n(3) as u8 },
        50 => Command::BuildResidence { branch: any_branch, level: r.n(10) as u8 },
        51 => match r.n(3) {
            0 => Command::SellResidence { branch: any_branch },
            // only adding extensions: removal is unreachable in the UI (rules-35 §5) and refunds past the cash cap
            1 => {
                let have = st.props.get(any_branch as usize).map_or(0, |p| if p.has_residence() { (p.residence() & 0xE0) as u8 } else { 0 });
                Command::ExtendResidence { branch: any_branch, bits: have | (r.n(8) as u8) << 5 }
            }
            _ => Command::RenovateResidence { branch: any_branch },
        },
        52 => Command::AskLender,
        53 => {
            let loans = st.heap.all(56, h as u8, ANY);
            Command::RepayLoan { loan: *r.pick(&loans).unwrap_or(&0) as u16 }
        }
        54 => match r.n(4) {
            0 => Command::JoinChurch { religion: r.n(2) as u8 },
            1 => Command::Convert,
            2 => Command::LeaveChurch,
            _ => Command::Indulgence,
        },
        55 => match r.n(3) {
            0 => Command::Candles,
            1 => Command::Pray,
            _ => Command::Alms,
        },
        56 => Command::WithdrawLot { prop: any_branch },
        57 => Command::CancelSpies { target: person },
        58 => Command::NameChild { slot: r.n(32) as u8, name: "Kind".into() },
        _ => Command::KontorView,
    }
}

/// State invariants from `docs/re`. Returns the first violation.
pub fn check(st: &State) -> Result<(), String> {
    let used = |i: u8| (i as usize) < PERSONS && st.persons[i as usize].used();
    // rules-00 §2 blocks 6, 7: current person 0xC0 = none, phase 0..6
    if st.current != NOBODY && !(used(st.current) && st.persons[st.current as usize].human()) {
        return Err(format!("current {} is not a human", st.current));
    }
    if st.phase > 6 {
        return Err(format!("phase {}", st.phase));
    }
    for (i, p) in st.persons.iter().enumerate() {
        if !p.used() {
            continue;
        }
        // rules-00 §4
        if p.gender() > 1 {
            return Err(format!("person {i} gender {}", p.gender()));
        }
        // rules-05 §7.1 quirk: a widow's spouse byte keeps pointing at the dead slot, so only the index is checked
        if p.spouse() != NONE && p.spouse() as usize >= PERSONS {
            return Err(format!("person {i} spouse {}", p.spouse()));
        }
        if p.certificate() > 2 {
            return Err(format!("person {i} certificate {}", p.certificate()));
        }
        if p.idx() > 15 || (p.has_office() && p.loc() > 13) {
            return Err(format!("person {i} office loc {} idx {}", p.loc(), p.idx()));
        }
        if p.flags10() >> 4 > 1 {
            return Err(format!("person {i} controller nibble {:#x}", p.flags10()));
        }
        // rules-60 §1: cash_add caps the top, pay the bottom; nothing else writes cash, so the band holds.
        // Computer persons: the yearly wealth drift (rules-03 §4.4, -7000 at most) is a cash_add with a negative
        // amount here, which the floor does not catch; which writer the original uses is open (IMPL-GAPS), so
        // only their top is checked.
        let floor = if p.human() { -9_999_999 } else { i32::MIN };
        if !(floor..=9_999_999).contains(&p.cash()) {
            return Err(format!("person {i} cash {}", p.cash()));
        }
        if p.human() && (p.colour() > 5 || p.name() > 31) {
            return Err(format!("person {i} colour {} name {}", p.colour(), p.name()));
        }
        // rules-00 §3 titles: ten names
        if p.title() > 9 {
            return Err(format!("person {i} title {}", p.title()));
        }
        // rules-40 §4: skill_up caps at 150
        if (0..GOODS).any(|g| p.skill(g) > 150) {
            return Err(format!("person {i} skill over 150"));
        }
        // rules-00 §3: one holder per (level location, index within level)
        if p.has_office() && st.persons[..i].iter().any(|q| q.used() && q.has_office() && q.loc() == p.loc() && q.idx() == p.idx()) {
            return Err(format!("person {i} shares office {}/{}", p.loc(), p.idx()));
        }
    }
    for (i, q) in st.props.iter().enumerate() {
        // rules-00 §5
        let kind = q.kind();
        if kind > KIND_FREE {
            return Err(format!("prop {i} kind {kind}"));
        }
        if kind == KIND_FREE {
            continue;
        }
        if !used(q.owner()) {
            return Err(format!("prop {i} kind {kind} owner {} unused", q.owner()));
        }
        let loc = q.loc() as usize;
        // rules-00 §3: cities 0..9; castles 14..16, camps 17..20
        let loc_ok = match kind {
            KIND_BRANCH => loc < CITIES,
            KIND_CASTLE => (14..17).contains(&loc),
            _ => (17..21).contains(&loc),
        };
        if !loc_ok {
            return Err(format!("prop {i} kind {kind} loc {loc}"));
        }
        for g in 0..GOODS {
            if q.stock(g) > 9999 {
                return Err(format!("prop {i} stock[{g}] {}", q.stock(g)));
            }
        }
        for t in 0..TRANSPORTERS {
            if q.trans(t) > 99 || q.cond(t) > 100 {
                return Err(format!("prop {i} transporters[{t}] {} cond {}", q.trans(t), q.cond(t)));
            }
        }
        for k in 0..4 {
            if q.training(k) > 100 {
                return Err(format!("prop {i} training[{k}] {}", q.training(k)));
            }
            if q.order(k)[7] > 3 {
                return Err(format!("prop {i} order {k} status {}", q.order(k)[7]));
            }
        }
        if q.director() != NONE && !used(q.director()) {
            return Err(format!("prop {i} director {} unused", q.director()));
        }
        // rules-26 §5.1: casemates / straw at most 150. Security / camouflage would be at most 100, but fate 99
        // wraps the byte to 246..255 (rules-06 §4, a documented bug of the original), so it is not checked.
        if kind != KIND_BRANCH && q.b4() > 150 {
            return Err(format!("prop {i} kind {kind} casemates {}", q.b4()));
        }
    }
    for (c, city) in st.cities.iter().enumerate() {
        // rules-00 §6
        if city.wealth() > 99 {
            return Err(format!("city {c} wealth {}", city.wealth()));
        }
        for g in 0..GOODS {
            if !(1..=99).contains(&city.price(g)) {
                return Err(format!("city {c} price[{g}] {}", city.price(g)));
            }
        }
    }
    // rules-00 §8
    if st.heap.used_bytes() > HEAP_BYTES {
        return Err(format!("heap {} bytes", st.heap.used_bytes()));
    }
    let mut missions = [0u8; PERSONS];
    let mut courting = [0u8; PERSONS];
    let mut interest = [0u8; PROPS];
    let (mut emperor, mut auction, mut contest) = (0, 0, 0);
    for (i, rec) in st.heap.recs.iter().enumerate() {
        let t = (rec[0] & 0x7F) as usize;
        if t >= SIZE.len() || rec.len() != SIZE[t] as usize {
            return Err(format!("heap rec {i} type {t} size {}", rec.len()));
        }
        if rec[0] & 0x80 != 0 {
            continue;
        }
        // byte 1: owner person or 0xC0 = world
        if rec[1] != NOBODY && rec[1] as usize >= PERSONS {
            return Err(format!("heap rec {i} type {t} owner {}", rec[1]));
        }
        match t {
            // type 35: no creator or handler
            35 => return Err(format!("heap rec {i} type 35")),
            0 => {
                if rec[4] > 29 || !used(rec[1]) {
                    return Err(format!("heap rec {i} mission {} of {}", rec[4], rec[1]));
                }
                missions[rec[1] as usize] += 1;
            }
            1 => emperor += 1,
            3 => auction += 1,
            4 => contest += 1,
            5 if rec[4] > 13 => return Err(format!("heap rec {i} application place {}", rec[4])),
            2 if rec[2] as usize >= CITIES => return Err(format!("heap rec {i} disaster city {}", rec[2])),
            25 | 26 if rec[4] as usize >= CITIES => return Err(format!("heap rec {i} type {t} city {}", rec[4])),
            31 if rec[5] > 31 => return Err(format!("heap rec {i} child name slot {}", rec[5])),
            57 if rec[4] > 9 => return Err(format!("heap rec {i} history code {}", rec[4])),
            8..=16 | 51..=55 if rec[2] as usize >= PROPS => return Err(format!("heap rec {i} type {t} prop {}", rec[2])),
            17 => {
                if rec[2] as usize >= PROPS {
                    return Err(format!("heap rec {i} interest prop {}", rec[2]));
                }
                interest[rec[2] as usize] += 1;
            }
            30 => courting[rec[1] as usize] += 1,
            _ => {}
        }
    }
    // one mission per human (rules-10 §2), one emperor / auction / contest record (rules-00 §8.2 owner 0xC0),
    // one courtship per suitor (rules-05 §2.1), at most six interested per castle (rules-26 §2)
    if let Some(p) = missions.iter().position(|&n| n > 1) {
        return Err(format!("person {p} has {} mission records", missions[p]));
    }
    if emperor > 1 || auction > 1 || contest > 1 {
        return Err(format!("world records: emperor {emperor} auction {auction} contest {contest}"));
    }
    if let Some(p) = courting.iter().position(|&n| n > 1) {
        return Err(format!("person {p} has {} courtships", courting[p]));
    }
    if let Some(p) = interest.iter().position(|&n| n > 6) {
        return Err(format!("prop {p} has {} interested", interest[p]));
    }
    // save round trip (rules-00 §2): to_game_dat -> from_game_dat -> to_game_dat byte-identical
    let a = st.to_game_dat();
    let back = State::from_game_dat(&a, 1).map_err(|e| format!("save does not load: {e}"))?;
    let b = back.to_game_dat();
    if a != b {
        let o = a.iter().zip(&b).position(|(x, y)| x != y).unwrap_or(0);
        return Err(format!("save round trip differs at byte {o:#x}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A few short games; `MAD_SOAK_SEEDS` / `MAD_SOAK_YEARS` lengthen it.
    #[test]
    fn soak_short() {
        let seeds: u32 = std::env::var("MAD_SOAK_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
        let years: u16 = std::env::var("MAD_SOAK_YEARS").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
        let szenario = std::env::var("MAD_DATA").ok().and_then(|p| std::fs::read(std::path::Path::new(&p).join("SZENARIO.DAT")).ok()).unwrap_or_default();
        for seed in 1..=seeds {
            let setup = Setup::from_seed(seed);
            play_game(&setup, years, 6, &szenario, &mut |_, _| {}).unwrap_or_else(|f| panic!("{f:?}"));
        }
    }

    /// The checker sees a broken record.
    #[test]
    fn check_catches_corruption() {
        let mut st = State::new_game(5, 0, &[HumanSetup::test(2)]).unwrap();
        assert_eq!(check(&st), Ok(()));
        let b = st.props_of(st.current as usize, KIND_BRANCH).next().unwrap();
        st.props[b].set_stock(0, 10_000);
        assert!(check(&st).unwrap_err().contains("stock"));
        st.props[b].set_stock(0, 0);
        st.heap.alloc(35, 0, 0, 1);
        assert!(check(&st).unwrap_err().contains("type 35"));
    }
}
