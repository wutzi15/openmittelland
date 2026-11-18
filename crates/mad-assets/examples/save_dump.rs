//! usage: save_dump <CONFIG.DAT | GAMEn.DAT>   prints the known fields / block layout.
use mad_assets::{config::Config, save};
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let d = std::fs::read(&path).unwrap();
    if let Ok(c) = Config::parse(&d) {
        println!("video mode {} cd path {:?}", c.video_mode(), c.cd_path());
        for k in 0..5 {
            println!("slot {k}: {:?}", c.slot_name(k));
        }
        return;
    }
    let s = save::Save::parse(&d).unwrap();
    let mut off = 0;
    for (i, (name, len, addr)) in save::BLOCKS.iter().enumerate() {
        let b = s.block(i);
        let head: Vec<String> = b.iter().take(16).map(|x| format!("{x:02x}")).collect();
        println!("{off:#07x} {name:<12} len {len:#07x} -> mem {addr:#x}: {}", head.join(" "));
        off += len;
    }
    for i in 0..save::NAMES {
        let n = s.name(i);
        if !n.is_empty() {
            println!("name[{i}] = {n:?}");
        }
    }
}
