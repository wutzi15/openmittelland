//! usage: favour_drift [seeds] [years] [cmds_per_turn]
//! How the computer persons' favour toward the humans moves when the humans do little: for each year k, plays
//! every seed for k years (`soak::play`, random answers) and prints the mean favour(x→human) over all computer
//! persons x and humans, and the share at or below 50 (a canon with that favour vetoes the wedding, rules-05 §3).
use mad_sim::soak::{Setup, play};


fn main() {
    let a: Vec<String> = std::env::args().collect();
    let arg = |i: usize, d: u32| a.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let (seeds, years, cmds) = (arg(1, 20), arg(2, 12) as u16, arg(3, 0));
    println!("year  mean  <=50  <=30   (favour of computers toward humans, {seeds} games, {cmds} commands per turn)");
    for k in 0..=years {
        let (mut sum, mut n, mut low, mut vlow) = (0u64, 0u64, 0u64, 0u64);
        for seed in 1..=seeds {
            let setup = Setup { humans: 1, missions: false, options: 0xFFFF, sustain: true, immortal: true, ..Setup::from_seed(seed) };
            let Ok((_, st)) = play(&setup, k, cmds, &[], &mut |_, _| {}) else { continue };
            for h in st.humans() {
                for x in st.computers() {
                    let f = st.persons[x].favour(h) as u64;
                    if f > 100 {
                        continue;
                    }
                    sum += f;
                    n += 1;
                    low += (f <= 50) as u64;
                    vlow += (f <= 30) as u64;
                }
            }
        }
        println!("{:4}  {:4.1}  {:3.0}%  {:3.0}%", 1600 + k, sum as f64 / n.max(1) as f64, 100.0 * low as f64 / n.max(1) as f64, 100.0 * vlow as f64 / n.max(1) as f64);
    }
}
