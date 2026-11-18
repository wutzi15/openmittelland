//! usage: mkauction <out GAMEn.DAT>   new game, one human, an auction held at the next year end.
use mad_sim::{state::{NOBODY, NONE}, world::HumanSetup, State};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut st = State::new_game(1, 3, &[HumanSetup { name: "anna".into(), gender: 0, religion: 1, colour: 0, hotkey: b'a', city: 3, mission: None }]).unwrap();
    let h = st.humans().next().unwrap();
    st.add_cash(h, 5000);
    st.heap.alloc(3, NOBODY, NONE, 0);
    std::fs::write(&a[1], st.to_game_dat()).unwrap();
}
