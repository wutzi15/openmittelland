//! Side-by-side check of one turn end + year end against the original (docs/FIDELITY.md "Year-end parity").
//!
//!   yearend_parity cg BEFORE AFTER                 brute-force the RNG state before the original's city_growth
//!                                                  from the city deltas (30 draws, rules-01 §3.3) over all 2^32 states
//!   yearend_parity scan BEFORE AFTER S_CG K0 K1    for k in K0..K1: load BEFORE with S_CG stepped back k draws, run
//!                                                  the path, score against AFTER; prints the best candidates
//!   yearend_parity diff BEFORE AFTER S0            run the path from RNG state S0 and print every difference
//!
//! Path = what the app does after loading a phase-4 save: `KontorView`, then `EndTurn` (settlement, year end,
//! compaction, year + 1, city_growth), stopped where the original autosaves. `MAD_NO_KONTOR=1` skips the KontorView
//! draws. Answers: Gift = `MAD_GIFT` (default 2), JuryBribe = 0, everything else 0.
use mad_sim::heap;
use mad_sim::state::*;
use mad_sim::{Ask, Choices, Event};

const A: u32 = 1103515245;
const C: u32 = 12345;

fn inv(a: u32) -> u32 {
    let mut x = a; // Newton: a*a == 1 mod 8, so x = a is correct to 3 bits
    for _ in 0..5 {
        x = x.wrapping_mul(2u32.wrapping_sub(a.wrapping_mul(x)));
    }
    x
}

fn back(mut s: u32, k: u64) -> u32 {
    let ai = inv(A);
    for _ in 0..k {
        s = s.wrapping_sub(C).wrapping_mul(ai);
    }
    s
}

struct Log {
    asks: Vec<String>,
    gift: u32,
}
impl Choices for Log {
    fn ask(&mut self, q: &Ask) -> u32 {
        let a = match q {
            Ask::Gift { .. } => self.gift,
            _ => 0,
        };
        self.asks.push(format!("{q:?} -> {a}"));
        a
    }
}

struct Run {
    st: State,
    asks: Vec<String>,
    events: Vec<Event>,
    /// RNG state right before city_growth
    s_cg: u32,
    draws_before_cg: u64,
}

fn run_path(before: &[u8], s0: u32, gift: u32, kontor: bool) -> Run {
    let mut st = State::from_game_dat(before, s0).expect("parse");
    let mut ch = Log { asks: vec![], gift };
    let mut ev = vec![];
    if kontor {
        ev.extend(st.apply(mad_sim::Command::KontorView, &mut ch));
    }
    let h = st.current as usize;
    // State::run from phase 5, stopped after city_growth (the original's autosave point)
    st.phase = 5;
    ev.extend(st.settlement(h, &mut ch));
    st.phase = 6;
    st.heap.compact();
    st.current = st.next_human(h);
    assert!(st.current >= NOBODY, "other humans move after this one");
    let first = st.humans().next().expect("a human") as u8;
    st.current = first;
    st.phase = 0;
    ch.show(&ev);
    // `MAD_RESEED=hex`: the original's RNG state at the start of the year end (after the debtors' tower wait loop,
    // rules-02 §10.3 parity hazard), measured from the stream
    if let Some(x) = std::env::var("MAD_RESEED").ok().and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok()) {
        st.rng.s = x;
    }
    ev.extend(st.year_end(&mut ch));
    st.heap.compact();
    st.year += 1;
    let s_cg = st.rng.s;
    st.city_growth();
    // draw count: steps from s0 to s_cg
    let mut n = 0u64;
    let mut s = s0;
    while s != s_cg && n < 50_000_000 {
        s = s.wrapping_mul(A).wrapping_add(C);
        n += 1;
    }
    Run { st, asks: ch.asks, events: ev, s_cg, draws_before_cg: n }
}

fn blk(d: &[u8], i: usize) -> &[u8] {
    use mad_assets::save::BLOCKS;
    let o: usize = BLOCKS[..i].iter().map(|b| b.1).sum();
    &d[o..o + BLOCKS[i].1]
}

/// (differing bytes, total compared) over persons, properties, cities, goods, globals, year/current/phase and the
/// used part of the heap.
fn score(ours: &[u8], theirs: &[u8]) -> (usize, usize) {
    let mut diff = 0;
    let mut total = 0;
    for i in [1, 2, 4, 5, 6, 9, 10, 11] {
        let (a, b) = (blk(ours, i), blk(theirs, i));
        // person bytes 0x09..0x0B hold a runtime pointer the original writes (not game state)
        diff += a.iter().zip(b).enumerate().filter(|(k, (x, y))| x != y && !(i == 1 && (9..12).contains(&(k % 0xF8)))).count();
        total += a.len();
    }
    let used = rd32(blk(theirs, 8), 0) as usize;
    let (a, b) = (&blk(ours, 3)[..used.min(0x2710)], &blk(theirs, 3)[..used.min(0x2710)]);
    diff += a.iter().zip(b).filter(|(x, y)| x != y).count();
    total += used;
    (diff, total)
}

fn cities(d: &[u8]) -> Vec<City> {
    blk(d, 9).chunks(0x8C).map(|c| City(c.try_into().unwrap())).collect()
}

fn find_cg(before: &[u8], after: &[u8]) -> Vec<u32> {
    let (cb, ca) = (cities(before), cities(after));
    let want: Vec<(u32, u32, u32, bool)> = (0..CITIES)
        .map(|c| {
            let dr = ca[c].rights().wrapping_sub(cb[c].rights());
            assert!(dr < 200, "city {c}: rights delta {dr} is not a rand(200)");
            (cb[c].population(), ca[c].population(), dr, ca[c].extra88() == cb[c].extra88() + 1)
        })
        .collect();
    eprintln!("city deltas (pop before, pop after, rights +, extra88 +1): {want:?}");
    let threads = std::thread::available_parallelism().map_or(8, |n| n.get()) as u64;
    let want = std::sync::Arc::new(want);
    let mut hs = vec![];
    for t in 0..threads {
        let want = want.clone();
        hs.push(std::thread::spawn(move || {
            let mut found = vec![];
            let lo = (t << 32) / threads;
            let hi = ((t + 1) << 32) / threads;
            for s0 in lo..hi {
                let mut s = s0 as u32;
                let mut ok = true;
                for &(p0, p1, dr, e) in want.iter() {
                    s = s.wrapping_mul(A).wrapping_add(C);
                    let g = 1000 + ((s >> 16) & 0x7FFF) % 50;
                    if (p0 as u64 * g as u64 / 1000) as u32 != p1 {
                        ok = false;
                        break;
                    }
                    s = s.wrapping_mul(A).wrapping_add(C);
                    if ((s >> 16) & 0x7FFF) % 200 != dr {
                        ok = false;
                        break;
                    }
                    s = s.wrapping_mul(A).wrapping_add(C);
                    if (((s >> 16) & 0x7FFF) % 100 < 7) != e {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    found.push(s0 as u32);
                }
            }
            found
        }));
    }
    hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
}

fn pname(st: &State, i: usize) -> String {
    let p = &st.persons[i];
    format!("{i}{}", if p.human() { "H" } else { "" })
}

fn diff_full(before: &[u8], after: &[u8], s0: u32, gift: u32, kontor: bool) {
    let r = run_path(before, s0, gift, kontor);
    for a in &r.asks {
        println!("ask: {a}");
    }
    for e in &r.events {
        let s = format!("{e:?}");
        println!("event: {}", &s[..s.len().min(300)]);
    }
    let ours = r.st.to_game_dat();
    if let Ok(p) = std::env::var("MAD_OUT") {
        std::fs::write(p, &ours).expect("write MAD_OUT");
    }
    let theirs = State::from_game_dat(after, 1).expect("parse after");
    let st = &r.st;
    println!("\nrng before city_growth: ours {:#010x} after {} draws", r.s_cg, r.draws_before_cg);
    println!("year/current/phase: ours {}/{}/{} theirs {}/{}/{}", st.year, st.current, st.phase, theirs.year, theirs.current, theirs.phase);
    let (d, t) = score(&ours, after);
    println!("score: {d} differing bytes of {t}");
    // persons
    for i in 0..PERSONS {
        let (a, b) = (&st.persons[i], &theirs.persons[i]);
        if a.0 == b.0 {
            continue;
        }
        let mut parts = vec![];
        let mut fav = vec![];
        for o in 0..0xF8 {
            if a.0[o] == b.0[o] || (9..12).contains(&o) {
                continue;
            }
            if (0x20..0x20 + PERSONS).contains(&o) {
                fav.push(format!("→{}:{}/{}", pname(st, o - 0x20), a.0[o], b.0[o]));
            } else if o == 0x18 {
                parts.push(format!("cash {}/{}", a.cash(), b.cash()));
            } else if o == 0x1C {
                parts.push(format!("nw_cache {}/{}", a.nw_cache(), b.nw_cache()));
            } else if (0x19..0x1C).contains(&o) || (0x1D..0x20).contains(&o) {
            } else {
                let name = match o {
                    0x00 => "gender",
                    0x02 => "life",
                    0x03 => "religion",
                    0x04 => "spouse",
                    0x05 => "certificate",
                    0x06 => "cert_due",
                    0x07 => "insurance",
                    0x08 => "name",
                    0x0D => "title",
                    0x0F => "loc",
                    0x10 => "flags10",
                    0x11 => "idx",
                    0x12 => "lvl_sec",
                    0x13 => "lvl_church",
                    0xEF => "pay_dir",
                    0xF0 => "pay_work",
                    0xF1 => "pay_merc",
                    0xF2 => "pay_rob",
                    0xF3 => "deeds",
                    0xF6 => "fencing",
                    0xF7 => "denounced",
                    _ => "",
                };
                parts.push(format!("{name}@{o:#04x} {}/{}", a.0[o], b.0[o]));
            }
        }
        println!("person {:>4}: {} | favour ours/theirs: {}", pname(st, i), parts.join(", "), fav.join(" "));
    }
    for i in 0..PROPS {
        let (a, b) = (&st.props[i], &theirs.props[i]);
        if a.0 != b.0 {
            let d: Vec<String> = (0..0x60).filter(|&o| a.0[o] != b.0[o]).map(|o| format!("@{o:#04x} {}/{}", a.0[o], b.0[o])).collect();
            println!("prop {i} (kind {} loc {} owner {}): {}", b.kind(), b.loc(), b.owner(), d.join(" "));
        }
    }
    for c in 0..CITIES {
        let (a, b) = (&st.cities[c], &theirs.cities[c]);
        if a.0 != b.0 {
            let d: Vec<String> = (0..0x8C).filter(|&o| a.0[o] != b.0[o]).map(|o| format!("@{o:#04x} {}/{}", a.0[o], b.0[o])).collect();
            println!("city {c}: {}", d.join(" "));
        }
    }
    let d: Vec<String> = (0..0xE1).filter(|&o| st.goods[o] != theirs.goods[o]).map(|o| format!("@{o:#04x} {}/{}", st.goods[o], theirs.goods[o])).collect();
    if !d.is_empty() {
        println!("goods: {}", d.join(" "));
    }
    let d: Vec<String> = (0..0x4E2).filter(|&o| st.globals.0[o] != theirs.globals.0[o]).map(|o| format!("{:#07x} {}/{}", 0x85A20 + o, st.globals.0[o], theirs.globals.0[o])).collect();
    if !d.is_empty() {
        println!("globals: {}", d.join(" "));
    }
    let (ha, hb) = (&st.heap.recs, &theirs.heap.recs);
    println!("heap: ours {} records, theirs {}", ha.len(), hb.len());
    for i in 0..ha.len().max(hb.len()) {
        let (a, b) = (ha.get(i), hb.get(i));
        if a != b {
            println!("  rec {i}: ours {:?} theirs {:?}", a.map(fmt_rec), b.map(fmt_rec));
        }
    }
}

fn fmt_rec(r: &heap::Rec) -> String {
    format!("t{} o{} k{} c{} {:02x?}", r[0], r[1], r[2], r[3], &r[4..])
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rd = |p: &String| std::fs::read(p).expect("read");
    let gift: u32 = std::env::var("MAD_GIFT").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    let kontor = std::env::var("MAD_NO_KONTOR").is_err();
    let hex = |s: &String| u32::from_str_radix(s.trim_start_matches("0x"), 16).expect("hex state");
    match args.get(1).map(String::as_str) {
        Some("cg") => {
            let f = find_cg(&rd(&args[2]), &rd(&args[3]));
            for s in &f {
                println!("s_cg {s:#010x}");
            }
            if f.is_empty() {
                println!("no state reproduces the city growth of AFTER from BEFORE");
            }
        }
        Some("scan") => {
            let (before, after) = (rd(&args[2]), rd(&args[3]));
            let s_cg = hex(&args[4]);
            let (k0, k1): (u64, u64) = (args[5].parse().unwrap(), args[6].parse().unwrap());
            let mut best: Vec<(usize, u64, u32, bool, String)> = vec![];
            let mut s = back(s_cg, k0);
            let ai = inv(A);
            for k in k0..k1 {
                let r = run_path(&before, s, gift, kontor);
                let (d, _) = score(&r.st.to_game_dat(), &after);
                let gifts = r.asks.iter().find(|a| a.starts_with("Gift")).cloned().unwrap_or_default();
                let means = r.events.iter().find_map(|e| if let Event::Bill { means, .. } = e { Some(*means) } else { None }).unwrap_or(-1);
                // `MAD_ANCHOR="prices: [234, 252, 184]"`: print every k whose gift question shows these prices
                if let Ok(a) = std::env::var("MAD_ANCHOR")
                    && gifts.contains(&a)
                {
                    println!("anchor k {k} s0 {s:#010x} diff {d} rng_at_cg_matches {} means {means} {gifts}", r.s_cg == s_cg);
                }
                best.push((d, k, s, r.s_cg == s_cg, format!("means {means} {gifts}")));
                s = s.wrapping_sub(C).wrapping_mul(ai);
            }
            best.sort();
            for (d, k, s, same, info) in best.iter().take(15) {
                println!("k {k:6} s0 {s:#010x} diff {d:6} rng_at_cg_matches {same} {info}");
            }
        }
        Some("diff") => diff_full(&rd(&args[2]), &rd(&args[3]), hex(&args[4]), gift, kontor),
        _ => eprintln!("usage: see source"),
    }
}
