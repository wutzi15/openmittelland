//! usage: inspect_save GAMEn.DAT
//! Favour and wealth facts behind dismissals (rules-03 §4.3) and wedding vetoes (rules-05 §3) for each human of a
//! save: office, net worth against last year's cache, the appointers' favour, every canon's favour, and how the
//! computer persons' favour toward him is spread.
use mad_sim::state::State;

fn main() {
    let path = std::env::args().nth(1).expect("GAMEn.DAT");
    let st = State::from_game_dat(&std::fs::read(&path).expect("read"), 1).expect("parse");
    println!("year {}", st.year);
    for h in st.humans().collect::<Vec<_>>() {
        let p = &st.persons[h];
        println!(
            "\nhuman {h}: title {} religion {} office loc {} idx {} | net worth {} last year {}",
            p.title(),
            p.religion(),
            p.loc(),
            p.idx(),
            st.net_worth(h),
            p.nw_cache()
        );
        if p.has_office() {
            for a in st.appointers(p.loc() as usize, p.idx() as usize) {
                let q = &st.persons[a];
                println!("  appointer {a:3} loc {:2} idx {:2} religion {} favour→me {:3}{}", q.loc(), q.idx(), q.religion(), q.favour(h), if q.human() { " (human)" } else { "" });
            }
        }
        for c in 0..10 {
            if let Some(d) = st.holder(c, 9) {
                println!("  canon of city {c}: {d:3} religion {} favour→me {:3}", st.persons[d].religion(), st.persons[d].favour(h));
            }
        }
        let mut hist = [0u32; 6];
        for x in st.computers() {
            let f = st.persons[x].favour(h) as usize;
            hist[if f > 100 { 5 } else { (f / 20).min(4) }] += 1;
        }
        println!("  computers' favour→me: 0-19 {} | 20-39 {} | 40-59 {} | 60-79 {} | 80-100 {} | >100 (blackmailed) {}", hist[0], hist[1], hist[2], hist[3], hist[4], hist[5]);
    }
}
