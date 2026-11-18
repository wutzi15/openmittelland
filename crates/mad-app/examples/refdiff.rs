//! Compares a DOSBox-X window capture with a mad-app screenshot (docs/DOSBOX.md).
//! Usage: refdiff <capture.png> <mine.png> <diff.png> [top=64] [scale=2]
//! The capture is cropped below the title bar and reduced by `scale` (nearest) to 640x480.
use std::{fs::File, io::BufWriter};

fn read(p: &str) -> (Vec<u8>, usize, usize) {
    let mut r = png::Decoder::new(std::io::BufReader::new(File::open(p).unwrap())).read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    let i = r.next_frame(&mut buf).unwrap();
    let ch = i.color_type.samples();
    let rgb: Vec<u8> = buf[..i.buffer_size()].chunks(ch).flat_map(|c| [c[0], c[1], c[2]]).collect();
    (rgb, i.width as usize, i.height as usize)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let top: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(64);
    let k: usize = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(2);
    let (dx, dy): (i64, i64) = (a.get(6).and_then(|s| s.parse().ok()).unwrap_or(0), a.get(7).and_then(|s| s.parse().ok()).unwrap_or(0));
    // REGION=x0,y0,x1,y1 limits the counted pixels (to fit one text run's offset).
    let reg: Vec<usize> = std::env::var("REGION").map(|r| r.split(',').filter_map(|v| v.parse().ok()).collect()).unwrap_or_default();
    let inside = |x: usize, y: usize| reg.len() != 4 || (x >= reg[0] && x < reg[2] && y >= reg[1] && y < reg[3]);
    let (cap, cw, _) = read(&a[1]);
    let (mine, mw, mh) = read(&a[2]);
    assert_eq!((mw, mh), (640, 480));
    let mut out = vec![0u8; 640 * 480 * 3];
    let (mut differ, mut sum, mut worst) = (0usize, 0u64, 0u8);
    for y in 0..480 {
        for x in 0..640 {
            let (sx, sy) = ((x as i64 + dx).clamp(0, 639) as usize, (y as i64 + dy).clamp(0, 479) as usize);
            let c = ((top + sy * k + k / 2) * cw + sx * k + k / 2) * 3;
            let m = (y * 640 + x) * 3;
            let d = (0..3).map(|i| cap[c + i].abs_diff(mine[m + i])).max().unwrap();
            if !inside(x, y) {
                continue;
            }
            worst = worst.max(d);
            sum += d as u64;
            if d > 24 {
                differ += 1;
                out[m..m + 3].copy_from_slice(&[255, 0, 0]);
            } else {
                out[m..m + 3].iter_mut().zip(&mine[m..m + 3]).for_each(|(o, v)| *o = v / 3);
            }
        }
    }
    let mut e = png::Encoder::new(BufWriter::new(File::create(&a[3]).unwrap()), 640, 480);
    e.set_color(png::ColorType::Rgb);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(&out).unwrap();
    println!("pixels differing by >24: {differ} of {} ({:.2} %), mean abs diff {:.2}, worst {worst}", 640 * 480, differ as f64 * 100.0 / 307200.0, sum as f64 / 307200.0);
}
