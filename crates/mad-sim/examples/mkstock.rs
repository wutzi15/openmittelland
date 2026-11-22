//! usage: mkstock <in GAMEn.DAT> <out GAMEn.DAT> <good> <units>   the first human's first branch gets `units` of `good`.
use mad_sim::{state::KIND_BRANCH, State};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut st = State::from_game_dat(&std::fs::read(&a[1]).unwrap(), 1).unwrap();
    let h = st.humans().next().unwrap();
    let b = st.props_of(h, KIND_BRANCH).next().unwrap();
    st.props[b].set_stock(a[3].parse().unwrap(), a[4].parse().unwrap());
    std::fs::write(&a[2], st.to_game_dat()).unwrap();
}
