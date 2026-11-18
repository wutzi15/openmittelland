//! usage: icon_dump <MITTELL.EXE> <ICONxx.DAT> <out.ppm>
//! Decodes any icon file (8-bit if its directory is ICONS2*, else RGB555) and writes all frames
//! side by side; transparent pixels are magenta.
use mad_assets::{palette, sprite};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let pal = palette::from_exe(&std::fs::read(&a[1]).unwrap()).unwrap();
    let bpp = if a[2].to_ascii_uppercase().contains("ICONS3") { 2 } else { 1 };
    let frames = sprite::decode_icon_file(&std::fs::read(&a[2]).unwrap(), bpp).unwrap();
    let (w, h) = (frames.iter().map(|f| f.width).sum::<usize>().max(1), frames.iter().map(|f| f.height).max().unwrap_or(1));
    let mut rgb = vec![[255, 0, 255]; w * h];
    let mut x0 = 0;
    for f in &frames {
        for y in 0..f.height {
            for x in 0..f.width {
                let i = y * f.width + x;
                if f.opaque[i] {
                    rgb[y * w + x0 + x] = if bpp == 1 { pal[f.pixels[i] as usize] } else { palette::rgb555(f.pixels[i]) };
                }
            }
        }
        x0 += f.width;
    }
    println!("{} frame(s), sheet {w}x{h}", frames.len());
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    out.extend(rgb.iter().flatten());
    std::fs::write(&a[3], out).unwrap();
}
