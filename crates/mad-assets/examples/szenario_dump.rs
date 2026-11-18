//! usage: szenario_dump <SZENARIO.DAT>   prints each 46x30 layout as hex cells ('..' = 00).
use mad_assets::szenario::{COLS, ROWS, SCENARIOS, Szenario};
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let s = Szenario::parse(&std::fs::read(path).unwrap()).unwrap();
    for k in 0..SCENARIOS {
        println!("scenario {k}");
        for r in 0..ROWS {
            let row: Vec<String> = (0..COLS).map(|c| match s.cell(k, r, c) { 0 => "..".into(), v => format!("{v:02x}") }).collect();
            println!("{}", row.join(" "));
        }
    }
}
