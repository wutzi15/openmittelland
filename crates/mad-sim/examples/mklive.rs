//! usage: mklive <out GAMEn.DAT> <mode>   new game in 1601, one human "anna" in city 3, rich; the next turn end shows:
//!   gift: a courtship gift menu (rules-63 §2)        siege-won: a won siege of city 4 (plunder/tribute question)
//!   siege-go: a siege of city 4 that most likely goes on (continue question)   long: siege-go and gift in one window
//!   apply: anna is a citizen (title 1) and Ratsherr seat 0 of city 3 is vacant (Bewerbung; with an application,
//!     an election at the year end where only computers vote)
//!   apply2: as apply, anna has applied and so have two eligible computers (an election only computers vote in)
//!   vote: anna holds seat 3 of city 3 and Ratsherr seat 0 is vacant (an election anna votes in)
//!   contract: gift plus a type-47 sales contract anna cannot keep (penalty line 0xAC911, no `$N` at its end)
//!   contract2: two such contracts, no gift (two penalty lines in a row)
//!   priv: anna is Sakristan of county 10 and has applied for the vacant Abt seat (privileged candidate line)
//!   solo: as apply, anna has applied alone (no election: 0xC1998 on the place picture)
//!   dismiss: anna holds seat 3 of city 3 with a huge cached net worth (dismissed at the year end)
use mad_sim::{heap, state::NONE, world::HumanSetup, State};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mode = a[2].as_str();
    let mut st = State::new_game(1, 0, &[HumanSetup { name: "anna".into(), gender: 0, religion: 1, colour: 0, hotkey: b'a', city: 3, mission: None }]).unwrap();
    st.year = 1601;
    let h = st.humans().next().unwrap();
    st.persons[h].set_cash_raw(5_000_000);
    st.persons[h].set_life(60);
    if matches!(mode, "siege-won" | "siege-go" | "long") {
        // counter 3: the turn end decrements to 2 and rolls (rules-21 §2.2); army = P/5 gives a 10 % win chance
        let p = st.cities[4].population();
        let army = if mode == "siege-won" { 60_000 } else { (p / 5 + 1).min(60_000) as u16 as u32 };
        let i = st.heap.alloc(25, h as u8, NONE, 3).unwrap();
        st.heap.recs[i][4] = 4;
        heap::w16(&mut st.heap.recs[i], 5, army as u16);
        eprintln!("siege of city 4 (population {p}) by {army}");
    }
    if matches!(mode, "contract" | "contract2") {
        // contract2: two penalty lines in a row (does the second start on the first one's line?)
        for g in 0..1 + (mode == "contract2") as u8 {
            let i = st.heap.alloc(47, h as u8, NONE, 1).unwrap();
            st.heap.recs[i][4] = 3;
            st.heap.recs[i][5] = g;
            heap::w16(&mut st.heap.recs[i], 6, 300);
            heap::w32(&mut st.heap.recs[i], 8, 1000);
            heap::w32(&mut st.heap.recs[i], 12, 5000);
        }
    }
    if mode == "priv" {
        for idx in [11, 9] {
            if let Some(x) = st.holder(10, idx) {
                st.office_vacate(x);
            }
        }
        st.appoint(h, 10, 9);
        st.apply_for(h, 10, 11, true);
    }
    if mode == "dismiss" {
        let y = st.holder(3, 3).unwrap();
        st.office_vacate(y);
        st.appoint(h, 3, 3);
        st.persons[h].set_nw_cache(500_000_000);
    }
    if matches!(mode, "gift" | "long" | "contract") {
        let (c, fee, w) = st.matchmaker_offer(h).unwrap();
        st.start_courtship(h, c, fee, w);
        eprintln!("courting person {c}");
    }
    if matches!(mode, "apply" | "apply2" | "vote" | "solo") {
        let x = st.holder(3, 0).unwrap();
        st.office_vacate(x);
        if mode == "vote" {
            let y = st.holder(3, 3).unwrap();
            st.office_vacate(y);
            st.appoint(h, 3, 3);
        } else {
            st.persons[h].set_title(1);
            eprintln!("applicable: {:?}", st.applicable_offices(h));
            if mode == "solo" {
                st.apply_for(h, 3, 0, true);
            }
            if mode == "apply2" {
                st.apply_for(h, 3, 0, true);
                let xs: Vec<usize> = st.computers().filter(|&x| st.eligibility(x, 3, 0) == 2).take(2).collect();
                for &x in &xs {
                    st.apply_for(x, 3, 0, true);
                }
                eprintln!("computer applicants {xs:?}");
            }
        }
    }
    std::fs::write(&a[1], st.to_game_dat()).unwrap();
}
