//! usage: mkscene <out GAMEn.DAT> <first kind 2..7> [city]   new game, one human, four transport orders on its branch
//! (city 3 by default; the harness reaches Bergstedt's town on the map, city 5).
use mad_sim::{state::KIND_BRANCH, world::HumanSetup, State};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let first: u8 = a[2].parse().unwrap();
    let city: u8 = a.get(3).map_or(3, |c| c.parse().unwrap());
    let mut st = State::new_game(1, 0, &[HumanSetup { name: "anna".into(), gender: 0, religion: 1, colour: 0, hotkey: b'a', city, mission: None }]).unwrap();
    let h = st.humans().next().unwrap();
    let b = st.props_of(h, KIND_BRANCH).next().unwrap();
    for slot in 0..4u8 {
        let kind = 2 + (first - 2 + slot) % 6;
        let dest = match (kind < 6, city == 5) { (true, false) => 5, (true, true) => 3, (false, _) => city };
        // written straight into the order book (rules-02 §3.1): `SetOrder` is refused before the first Kontor phase.
        // 100 units of good `slot` on one Karren; status 0 (no stock yet)
        st.props[b].set_order(slot as usize, [kind, 100, 0, slot, 1, 0, dest, 0]);
    }
    std::fs::write(&a[1], st.to_game_dat()).unwrap();
}
