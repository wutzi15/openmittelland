//! usage: economy_run orig GAMEn.DAT...            (a series of yearly saves of one game, in year order)
//!        economy_run ours [seeds] [years] [difficulty]      (REPLAY=1: replay a ruinous turn end from RNG state 1)
//! Per-year economy and career metrics of a passive one-human game: the original's autosaves against N seeds of
//! ours (`Passive` answers, no Kontor commands). Each row is the state at a year start; the turnover columns
//! compare it with the row before. `ours` prints the mean over the seeds.
use mad_sim::state::*;
use mad_sim::{Ask, Choices, Command};

/// The human's cheapest answer everywhere: no gift, no fee, no bribe, decline offers, stay out.
struct Passive;
impl Choices for Passive {
    fn ask(&mut self, q: &Ask) -> u32 {
        match q {
            Ask::Match { .. } | Ask::LoanOffer { .. } | Ask::AttendAuction { .. } | Ask::BuyCastle { .. } | Ask::FencingLesson { .. } => 1,
            Ask::AcceptDuel { .. } | Ask::ContinueSiege { .. } | Ask::ResidenceSwap { .. } => 1,
            Ask::Gift { .. } => 3,
            Ask::WeddingFeast { .. } => 4,
            Ask::Building { .. } => 5,
            Ask::PickDirector { candidates, .. } => candidates.len() as u32,
            Ask::Plea => 7,
            // yes/no sub-plot dialogs: 1 refuses (the original run answered every bullet menu with its last line)
            Ask::Plot { .. } => 1,
            _ => 0,
        }
    }
}

const COLS: &[&str] = &[
    "year", "nw_mean", "nw_med", "nw_p10", "nw_p90", "cash_med", "c_branch", "c_castle", "c_resid", "c_married", "pop", "price", "wealth",
    "fav_mean", "fav<=50", "h_cash", "h_nw", "h_title", "offices", "off_chg", "off_new", "dismiss", "replaced", "h_loans", "h_tower",
];

fn pct(v: &[i64], q: f64) -> f64 {
    v[((v.len() - 1) as f64 * q).round() as usize] as f64
}

/// Office holders by (location, index).
fn offices(st: &State) -> std::collections::BTreeMap<(u8, u8), usize> {
    st.computers().chain(st.humans()).filter(|&i| st.persons[i].has_office()).map(|i| ((st.persons[i].loc(), st.persons[i].idx()), i)).collect()
}

/// A slot holds a new person when its life went up or its gender or name changed (life only falls, rules-00 §4).
fn replaced(a: &Person, b: &Person) -> bool {
    a.used() && b.used() && (b.life() > a.life() || a.gender() != b.gender() || a.name() != b.name())
}

fn row(st: &State, prev: Option<&State>) -> Vec<f64> {
    let comp: Vec<usize> = st.computers().collect();
    let mut nw: Vec<i64> = comp.iter().map(|&x| st.net_worth(x) as i64).collect();
    nw.sort();
    let mut cash: Vec<i64> = comp.iter().map(|&x| st.persons[x].cash() as i64).collect();
    cash.sort();
    let owned = |k: u8| st.props.iter().filter(|p| p.kind() == k && st.persons.get(p.owner() as usize).is_some_and(|o| o.used() && !o.human())).count() as f64;
    let resid = st.props.iter().filter(|p| p.kind() == KIND_BRANCH && p.has_residence() && st.persons.get(p.owner() as usize).is_some_and(|o| o.used() && !o.human())).count() as f64;
    let married = comp.iter().filter(|&&x| st.persons[x].spouse() != NONE).count() as f64;
    let pop: f64 = st.cities.iter().map(|c| c.population() as f64).sum();
    let price = st.cities.iter().flat_map(|c| (0..GOODS).map(|g| c.price(g) as f64)).sum::<f64>() / (CITIES * GOODS) as f64;
    let wealth = st.cities.iter().map(|c| c.wealth() as f64).sum::<f64>() / CITIES as f64;
    let h = st.humans().next();
    let fav: Vec<u8> = h.map(|h| comp.iter().map(|&x| st.persons[x].favour(h)).filter(|&f| f <= 100).collect()).unwrap_or_default();
    let fav_mean = fav.iter().map(|&f| f as f64).sum::<f64>() / fav.len().max(1) as f64;
    let fav_low = 100.0 * fav.iter().filter(|&&f| f <= 50).count() as f64 / fav.len().max(1) as f64;
    let off = offices(st);
    let (mut chg, mut new, mut dis, mut rep) = (f64::NAN, f64::NAN, f64::NAN, f64::NAN);
    // only consecutive years: a human in a tower gets no turn, so ours can skip a year start
    if let Some(pv) = prev.filter(|pv| pv.year + 1 == st.year) {
        let po = offices(pv);
        chg = off.iter().filter(|(k, v)| po.get(k) != Some(v)).count() as f64;
        // changed seats whose new holder held no office a year ago (or is a new person)
        new = off.iter().filter(|&(k, &v)| po.get(k) != Some(&v) && (!pv.persons[v].has_office() || replaced(&pv.persons[v], &st.persons[v]))).count() as f64;
        rep = (0..PERSONS).filter(|&i| replaced(&pv.persons[i], &st.persons[i])).count() as f64;
        dis = po.values().filter(|&&i| st.persons[i].used() && !replaced(&pv.persons[i], &st.persons[i]) && !st.persons[i].has_office()).count() as f64;
    }
    vec![
        st.year as f64,
        nw.iter().sum::<i64>() as f64 / nw.len() as f64,
        pct(&nw, 0.5),
        pct(&nw, 0.1),
        pct(&nw, 0.9),
        pct(&cash, 0.5),
        owned(KIND_BRANCH),
        owned(KIND_CASTLE) + owned(KIND_CAMP),
        resid,
        married,
        pop,
        price,
        wealth,
        fav_mean,
        fav_low,
        h.map_or(f64::NAN, |h| st.persons[h].cash() as f64),
        h.map_or(f64::NAN, |h| st.net_worth(h) as f64),
        h.map_or(f64::NAN, |h| st.persons[h].title() as f64),
        off.len() as f64,
        chg,
        new,
        dis,
        rep,
        h.map_or(f64::NAN, |h| st.loans_of(h) as f64),
        // history entries with code 0: turns skipped in a tower (rules-04b §2)
        h.map_or(f64::NAN, |h| st.heap.recs.iter().filter(|r| r[0] == 57 && r[1] as usize == h && r[4] == 0).count() as f64),
    ]
}

fn print(rows: &[Vec<f64>]) {
    println!("{}", COLS.join("\t"));
    for r in rows {
        println!("{}", r.iter().map(|v| if v.is_nan() { "-".into() } else if v.fract() == 0.0 || v.abs() >= 1000.0 { format!("{v:.0}") } else { format!("{v:.1}") }).collect::<Vec<_>>().join("\t"));
    }
}

/// Yearly states of one passive game of ours, from the new game to `years` later. With `replay`, a turn end that
/// ruins the human is run again from RNG state 1, as longrun.sh REPLAY=1 does with the original (rules-69 §7: a
/// reloaded autosave starts at state 1; ours replays from the Kontor, the original from the year start).
fn ours(seed: u32, years: u16, difficulty: u8, replay: bool) -> Vec<State> {
    let players = [mad_sim::world::HumanSetup { name: "Passiv".into(), gender: 0, religion: 1, colour: 0, hotkey: b'a', city: NONE, mission: None }];
    let mut st = State::new_game(seed, difficulty, &players).expect("new game");
    st.options = 0xFFFF;
    let mut out = vec![st.clone()];
    st.run(&mut Passive);
    while st.year < 1600 + years && st.humans().next().is_some() {
        let y = st.year;
        let before = replay.then(|| st.clone());
        st.apply(Command::EndTurn, &mut Passive);
        if let Some(mut b) = before.filter(|_| st.humans().next().is_none()) {
            b.rng = Default::default();
            b.apply(Command::EndTurn, &mut Passive);
            st = b;
        }
        if st.year != y {
            out.push(st.clone());
        }
        if st.current >= NOBODY {
            break;
        }
    }
    out
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("orig") => {
            let sts: Vec<State> = a[2..].iter().map(|f| State::from_game_dat(&std::fs::read(f).expect("read"), 1).expect("parse")).collect();
            eprintln!("difficulty {}", sts[0].globals.b(ga::DIFFICULTY));
            let rows: Vec<Vec<f64>> = sts.iter().enumerate().map(|(i, s)| row(s, i.checked_sub(1).map(|j| &sts[j]))).collect();
            print(&rows);
        }
        Some("ours") => {
            let arg = |i: usize, d: u32| a.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
            let (seeds, years, diff) = (arg(2, 20), arg(3, 30) as u16, arg(4, 3) as u8);
            // by year: a human in the debtors' tower skips turns, so one EndTurn can pass several years
            let mut acc: std::collections::BTreeMap<u16, (Vec<f64>, Vec<f64>)> = Default::default();
            for seed in 1..=seeds {
                let sts = ours(seed, years, diff, std::env::var("REPLAY").is_ok());
                for (k, s) in sts.iter().enumerate() {
                    let r = row(s, k.checked_sub(1).map(|j| &sts[j]));
                    let (sum, cnt) = acc.entry(s.year).or_insert_with(|| (vec![0.0; r.len()], vec![0.0; r.len()]));
                    for (c, v) in r.iter().enumerate().filter(|(_, v)| !v.is_nan()) {
                        sum[c] += v;
                        cnt[c] += 1.0;
                    }
                }
            }
            let rows: Vec<Vec<f64>> = acc.values().map(|(s, n)| s.iter().zip(n).map(|(s, n)| if *n > 0.0 { s / n } else { f64::NAN }).collect()).collect();
            print(&rows);
            eprintln!("games per year: {:?}", acc.iter().map(|(y, (_, n))| (*y, n[0] as u32)).collect::<Vec<_>>());
        }
        _ => eprintln!("usage: economy_run orig GAMEn.DAT... | economy_run ours [seeds] [years] [difficulty]"),
    }
}
