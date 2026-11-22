//! usage: mkcastle <out GAMEn.DAT> [camp]   new game, one human who owns a castle (or camp) that a computer person wants to buy.
use mad_sim::{state::*, world::HumanSetup, State};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut st = State::new_game(1, 3, &[HumanSetup { name: "anna".into(), gender: 0, religion: 1, colour: 0, hotkey: b'a', city: 3, mission: None }]).unwrap();
    let h = st.humans().next().unwrap();
    let kind = if a.get(2).map(String::as_str) == Some("camp") { KIND_CAMP } else { KIND_CASTLE };
    let p = (0..PROPS).find(|&p| st.props[p].kind() == kind).unwrap();
    let x = st.computers().next().unwrap();
    st.props[p].set_owner(h as u8);
    st.props[p].set_stock(0, 123);
    st.props[p].set_stock(3, 4567);
    st.heap.alloc(17, x as u8, p as u8, 4);
    eprintln!("prop {p} in city {} buyer {x}", st.props[p].loc());
    std::fs::write(&a[1], st.to_game_dat()).unwrap();
}
