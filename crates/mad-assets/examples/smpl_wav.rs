//! usage: smpl_wav <SMPL_xxx.DAT> <out.wav>   (rate from the directory name: SMP44KHZ -> 44100, else 22050)
use mad_assets::smpl::Sample;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = std::fs::read(&a[1]).unwrap();
    let s = Sample::parse(&d).unwrap();
    let rate = if a[1].to_ascii_uppercase().contains("SMP44KHZ") { 44100 } else { 22050 };
    println!("header {:x?}, {} PCM bytes, {rate} Hz", s.header, s.pcm.len());
    std::fs::write(&a[2], s.to_wav(rate)).unwrap();
}
