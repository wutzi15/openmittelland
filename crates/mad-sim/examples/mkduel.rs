//! usage: mkduel <out GAMEn.DAT>   new game, one human with a pending duel against the first computer person.
use mad_sim::{state::NONE, world::HumanSetup, State};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut st = State::new_game(1, 3, &[HumanSetup { name: "anna".into(), gender: 0, religion: 1, colour: 0, hotkey: b'a', city: 3, mission: None }]).unwrap();
    let h = st.humans().next().unwrap();
    let x = st.computers().next().unwrap();
    if let Some(i) = st.heap.alloc(22, h as u8, NONE, 1) {
        st.heap.recs[i][4] = x as u8;
    }
    std::fs::write(&a[1], st.to_game_dat()).unwrap();
}
