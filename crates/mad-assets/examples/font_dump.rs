//! usage: font_dump <MITTELL.EXE> <ICONCS.DAT> <out.ppm>
//! Renders all 256 glyphs on a 16x16 grid of 40x32 cells (8-bit font if path contains ICONS2).
use mad_assets::{font::Font, palette};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let pal = palette::from_exe(&std::fs::read(&a[1]).unwrap()).unwrap();
    let bpp = if a[2].to_ascii_uppercase().contains("ICONS3") { 2 } else { 1 };
    let font = Font::parse(&std::fs::read(&a[2]).unwrap(), bpp).unwrap();
    let (w, h) = (16 * 40, 16 * 32);
    let mut rgb = vec![[40u8, 40, 60]; w * h];
    for (c, g) in font.glyphs.iter().enumerate() {
        let (cx, cy) = (c % 16 * 40, c / 16 * 32);
        for y in 0..g.height.min(32) {
            for x in 0..g.width.min(40) {
                let i = y * g.width + x;
                if g.opaque[i] {
                    rgb[(cy + y) * w + cx + x] = if bpp == 1 { pal[g.pixels[i] as usize] } else { palette::rgb555(g.pixels[i]) };
                }
            }
        }
    }
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    out.extend(rgb.iter().flatten());
    std::fs::write(&a[3], out).unwrap();
}
