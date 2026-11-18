//! usage: flc_dump <file.flc> <frame> <out.ppm>
use mad_assets::flc::Flc;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let mut f = Flc::new(&data).unwrap();
    let n: usize = a[2].parse().unwrap();
    println!("{}x{} frames={} speed={}ms", f.width, f.height, f.frames, f.speed_ms);
    for _ in 0..=n {
        if !f.step().unwrap() { break; }
    }
    let mut out = format!("P6\n{} {}\n255\n", f.width, f.height).into_bytes();
    for &i in &f.pixels { out.extend_from_slice(&f.palette[i as usize]); }
    std::fs::write(&a[3], out).unwrap();
}
