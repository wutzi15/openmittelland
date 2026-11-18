//! usage: mkbattle <out GAMEn.DAT> [fight|conquer|defend|raid|ship]
//! New game, one human who owns a camp and has one attack order (type 51, due at the next turn end) against the
//! first castle, 250 men of each kind; rich and healthy enough to watch it.
//!   `fight`: castle security 0, so the call-off roll almost never fires (rules-50 §2)
//!   `conquer`: as `fight` with an empty garrison
//!   `defend`: the human owns the castle (empty garrison); the camp's computer owner attacks it
//!   `raid`: four raid orders from the camp on county 0 and a computer shipment by cart into city 1 (rules-07 §4;
//!   each order raids with 30 %); `ship`: the same with one ship into city 3 (code 3)
use mad_sim::{state::*, world::HumanSetup, State};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut st = State::new_game(1, 3, &[HumanSetup { name: "anna".into(), gender: 0, religion: 1, colour: 0, hotkey: b'a', city: 3, mission: None }]).unwrap();
    let h = st.humans().next().unwrap();
    let camp = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CAMP).unwrap();
    let robber = st.props[camp].owner();
    let castle = (0..PROPS).find(|&p| st.props[p].kind() == KIND_CASTLE).unwrap();
    st.props[camp].set_owner(h as u8);
    // `fight`: castle security 0, so the call-off roll rand(200) <= 0 almost never fires (rules-50 §2)
    if a.get(2).map(String::as_str) == Some("defend") {
        st.props[camp].set_owner(robber);
        st.props[castle].set_owner(h as u8);
        st.props[castle].set_b3(0);
        for k in 0..4 {
            st.props[castle].set_troops(k, 0);
        }
        let r = st.heap.alloc(51, robber, camp as u8, 1).unwrap();
        st.heap.recs[r][4] = st.props[castle].loc() - 14;
        st.heap.recs[r][5..9].fill(250);
        st.persons[h].set_cash_raw(50_000_000);
        st.persons[h].set_life(60);
        eprintln!("computer {robber}'s camp {camp} attacks anna's castle {castle}");
        std::fs::write(&a[1], st.to_game_dat()).unwrap();
        return;
    }
    if matches!(a.get(2).map(String::as_str), Some("fight" | "conquer")) {
        st.props[castle].set_b3(0);
    }
    // `conquer`: an empty garrison, so the attack wins and asks to occupy (rules-59 §2)
    if a.get(2).map(String::as_str) == Some("conquer") {
        for k in 0..4 {
            st.props[castle].set_troops(k, 0);
        }
    }
    let ship = a.get(2).map(String::as_str) == Some("ship");
    if ship || a.get(2).map(String::as_str) == Some("raid") {
        let src = (0..PROPS).find(|&p| st.props[p].kind() == KIND_BRANCH && !st.persons[st.props[p].owner() as usize].human()).unwrap();
        let s = st.heap.alloc(9, st.props[src].owner(), src as u8, 2).unwrap();
        let r = &mut st.heap.recs[s];
        r[4] = 2;
        r[5..7].copy_from_slice(&(if ship { 1500u16 } else { 500 }).to_le_bytes());
        r[7] = 1;
        r[8] = if ship { 1 } else { 5 };
        r[9] = ship as u8;
        r[10] = if ship { 3 } else { 1 };
        r[11] = 90;
        for _ in 0..4 {
            let r = st.heap.alloc(54, h as u8, camp as u8, 1).unwrap();
            st.heap.recs[r][4] = 0;
            st.heap.recs[r][5..9].fill(250);
        }
        st.persons[h].set_cash_raw(50_000_000);
        st.persons[h].set_life(60);
        eprintln!("camp {camp}: raids on county 0, shipment {s} from prop {src} (loc {}) to city 1", st.props[src].loc());
        std::fs::write(&a[1], st.to_game_dat()).unwrap();
        return;
    }
    let r = st.heap.alloc(51, h as u8, camp as u8, 1).unwrap();
    st.heap.recs[r][4] = st.props[castle].loc() - 14;
    st.heap.recs[r][5..9].fill(250);
    st.persons[h].set_cash_raw(50_000_000);
    st.persons[h].set_life(60);
    eprintln!("camp {camp} (loc {}) attacks castle {castle} (loc {})", st.props[camp].loc(), st.props[castle].loc());
    std::fs::write(&a[1], st.to_game_dat()).unwrap();
}
