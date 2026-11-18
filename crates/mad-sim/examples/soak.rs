//! usage: soak [first_seed] [seeds] [years] [cmds_per_turn] [threads]
//! Plays `seeds` random games (`mad_sim::soak`) for `years` each, with invariant checks after every command.
//! Reports every panic, invariant violation and stall (no progress for 120 s) with seed, year and last command.
//! `MAD_DATA` points at the install (SZENARIO.DAT makes battles run).
use mad_sim::soak::{Setup, play_game};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let arg = |i: usize, d: u64| a.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let (first, seeds, years, cmds, threads) = (arg(1, 1) as u32, arg(2, 100) as u32, arg(3, 120) as u16, arg(4, 8) as u32, arg(5, 8) as usize);
    let szenario = std::env::var("MAD_DATA").ok().and_then(|p| std::fs::read(std::path::Path::new(&p).join("SZENARIO.DAT")).ok()).unwrap_or_default();
    if szenario.is_empty() {
        eprintln!("warning: no SZENARIO.DAT (set MAD_DATA); battles are not simulated");
    }
    let next = Arc::new(AtomicU32::new(first));
    let (total_years, total_cmds, failures) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)), Arc::new(Mutex::new(Vec::<String>::new())));
    let coverage = Arc::new(Mutex::new(std::collections::BTreeMap::<String, u64>::new()));
    // watchdog: per worker (last heartbeat, description)
    let beats: Arc<Vec<(AtomicU64, Mutex<String>)>> = Arc::new((0..threads).map(|_| (AtomicU64::new(u64::MAX), Mutex::new(String::new()))).collect());
    let t0 = Instant::now();
    let mut workers = vec![];
    for w in 0..threads {
        let (next, szenario, beats, ty, tc, failures, coverage) = (next.clone(), szenario.clone(), beats.clone(), total_years.clone(), total_cmds.clone(), failures.clone(), coverage.clone());
        workers.push(std::thread::spawn(move || {
            loop {
                let seed = next.fetch_add(1, Ordering::Relaxed);
                if seed >= first + seeds {
                    break;
                }
                let setup = Setup::from_seed(seed);
                let desc = format!("seed {seed} humans {} difficulty {} missions {} options {:#x}", setup.humans, setup.difficulty, setup.missions, setup.options);
                let last = Arc::new(Mutex::new((1600u16, String::new())));
                let last2 = last.clone();
                let beat = &beats[w];
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    play_game(&setup, years, cmds, &szenario, &mut |year, cmd| {
                        beat.0.store(t0.elapsed().as_secs(), Ordering::Relaxed);
                        let mut l = last2.lock().unwrap();
                        if l.0 != year || l.1 != cmd {
                            *l = (year, cmd.to_string());
                            *beat.1.lock().unwrap() = format!("{desc}: year {year}, {cmd}");
                        }
                    })
                }));
                beat.0.store(u64::MAX, Ordering::Relaxed);
                let l = last.lock().unwrap();
                match res {
                    Ok(Ok(s)) => {
                        ty.fetch_add(s.years as u64, Ordering::Relaxed);
                        tc.fetch_add(s.commands as u64, Ordering::Relaxed);
                        let mut cov = coverage.lock().unwrap();
                        for (k, v) in &s.events {
                            *cov.entry(k.clone()).or_default() += v;
                        }
                        println!("ok   {desc}: {} years, {} commands{}", s.years, s.commands, if s.over { format!(", game over: {}", s.end) } else { String::new() });
                    }
                    Ok(Err(f)) => {
                        let m = format!("FAIL {desc}: year {} after {}: {}", f.year, f.last, f.what);
                        println!("{m}");
                        failures.lock().unwrap().push(m);
                    }
                    Err(p) => {
                        let msg = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                        let m = format!("PANIC {desc}: year {} after {}: {msg}", l.0, l.1);
                        println!("{m}");
                        failures.lock().unwrap().push(m);
                    }
                }
            }
        }));
    }
    // watchdog thread: a worker without a heartbeat for 120 s is a hang; print and exit
    let beats2 = beats.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(5));
            let now = t0.elapsed().as_secs();
            for (b, d) in beats2.iter() {
                let t = b.load(Ordering::Relaxed);
                if t != u64::MAX && now.saturating_sub(t) > 120 {
                    println!("HANG {}", d.lock().unwrap());
                    std::process::exit(2);
                }
            }
        }
    });
    for w in workers {
        let _ = w.join();
    }
    let f = failures.lock().unwrap();
    let cov = coverage.lock().unwrap();
    println!("events: {}", cov.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(", "));
    println!(
        "\n{} games, {} game-years, {} commands, {} failures in {:.0} s",
        seeds,
        total_years.load(Ordering::Relaxed),
        total_cmds.load(Ordering::Relaxed),
        f.len(),
        t0.elapsed().as_secs_f64()
    );
    for m in f.iter() {
        println!("{m}");
    }
    if !f.is_empty() {
        std::process::exit(1);
    }
}
