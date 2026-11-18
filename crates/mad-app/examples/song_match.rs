//! usage: song_match <MIDSONGS dir> <soundtrack dir> [hop ms, default 46]
//! Matches the original MIDI songs to the soundtrack OGGs by rhythm, not just length: the MIDI note-on times
//! (tempo map, rules-20 §1) are binned like the OGG's onset strength (rises of the log frame energy), and the best
//! normalized correlation over a 0..4 s lag wins. Prints, per song, the three best tracks with their scores.
use rodio::{Decoder, Source};
use std::path::Path;

static HOP_MS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(46);

fn hop() -> f64 {
    HOP_MS.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1000.0
}

fn vlq(d: &[u8], i: &mut usize) -> u32 {
    let mut v = 0u32;
    loop {
        let b = d[*i];
        *i += 1;
        v = (v << 7) | (b & 0x7F) as u32;
        if b & 0x80 == 0 {
            return v;
        }
    }
}

/// Note-on times in seconds of a format-1, one-track SMF (rules-20 §1: division 48, tempo metas inline).
fn midi_onsets(d: &[u8]) -> Vec<f64> {
    let div = u16::from_be_bytes([d[12], d[13]]) as f64;
    let mut i = 14;
    let mut out = vec![];
    while i + 8 <= d.len() {
        let len = u32::from_be_bytes([d[i + 4], d[i + 5], d[i + 6], d[i + 7]]) as usize;
        let (start, end) = (i + 8, i + 8 + len);
        if &d[i..i + 4] == b"MTrk" {
            let (mut j, mut t, mut tempo, mut status) = (start, 0.0f64, 500_000.0f64, 0u8);
            while j < end {
                t += vlq(d, &mut j) as f64 * tempo / 1e6 / div;
                let mut b = d[j];
                if b & 0x80 != 0 {
                    j += 1;
                    if b < 0xF0 {
                        status = b;
                    }
                } else {
                    b = status;
                }
                match b {
                    0xFF => {
                        let kind = d[j];
                        j += 1;
                        let l = vlq(d, &mut j) as usize;
                        if kind == 0x51 {
                            tempo = ((d[j] as u32) << 16 | (d[j + 1] as u32) << 8 | d[j + 2] as u32) as f64;
                        }
                        j += l;
                    }
                    0xF0 | 0xF7 => {
                        let l = vlq(d, &mut j) as usize;
                        j += l;
                    }
                    _ => {
                        let n = if matches!(b & 0xF0, 0xC0 | 0xD0) { 1 } else { 2 };
                        if b & 0xF0 == 0x90 && d[j + 1] > 0 {
                            out.push(t);
                        }
                        j += n;
                    }
                }
            }
        }
        i = end;
    }
    out
}

fn ogg_onsets(p: &Path) -> Vec<f64> {
    let dec = Decoder::try_from(std::fs::File::open(p).unwrap()).unwrap();
    let (ch, sr) = (dec.channels().get() as usize, dec.sample_rate().get() as f64);
    let hop = (hop() * sr) as usize * ch;
    let mut energies = vec![];
    let (mut acc, mut n) = (0.0f64, 0usize);
    for s in dec {
        acc += (s as f64) * (s as f64);
        n += 1;
        if n == hop {
            energies.push((acc / n as f64 + 1e-9).ln());
            (acc, n) = (0.0, 0);
        }
    }
    energies.windows(2).map(|w| (w[1] - w[0]).max(0.0)).collect()
}

fn bins(times: &[f64], len: usize) -> Vec<f64> {
    let mut v = vec![0.0; len];
    for &t in times {
        let k = (t / hop()) as usize;
        if k < len {
            v[k] += 1.0;
        }
    }
    v
}

/// Best Pearson correlation of `a` against `b` delayed by 0..max_lag frames.
fn best_corr(a: &[f64], b: &[f64], max_lag: usize) -> f64 {
    let mut best = -1.0f64;
    for lag in 0..max_lag {
        let n = a.len().min(b.len().saturating_sub(lag));
        if n < 50 {
            continue;
        }
        let (x, y) = (&a[..n], &b[lag..lag + n]);
        let (mx, my) = (x.iter().sum::<f64>() / n as f64, y.iter().sum::<f64>() / n as f64);
        let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
        for k in 0..n {
            let (dx, dy) = (x[k] - mx, y[k] - my);
            (sxy, sxx, syy) = (sxy + dx * dy, sxx + dx * dx, syy + dy * dy);
        }
        best = best.max(sxy / (sxx * syy).sqrt().max(1e-12));
    }
    best
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (mid, ogg) = (Path::new(&a[1]), Path::new(&a[2]));
    if let Some(ms) = a.get(3) {
        HOP_MS.store(ms.parse().unwrap(), std::sync::atomic::Ordering::Relaxed);
    }
    let mut tracks: Vec<_> = std::fs::read_dir(ogg).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "ogg")).collect();
    tracks.sort();
    let onsets: Vec<(String, Vec<f64>)> = tracks.iter().map(|p| (p.file_name().unwrap().to_string_lossy().into_owned(), ogg_onsets(p))).collect();
    for (k, c) in "123456789ABCDEFGHIJKLMNO".chars().enumerate() {
        let d = std::fs::read(mid.join(format!("SONG{c}.MID"))).unwrap();
        let times = midi_onsets(&d);
        // candidates by length first (renders run 0..3 s longer than the MIDI, rules-14 §4.2), rhythm decides
        let end = times.iter().cloned().fold(0.0, f64::max);
        let mut scores: Vec<(f64, &str)> = onsets.iter().filter(|(_, o)| (o.len() as f64 * hop() - end).abs() < 8.0).map(|(name, o)| (best_corr(&bins(&times, o.len()), o, (4.0 / hop()) as usize), name.as_str())).collect();
        scores.sort_by(|x, y| y.0.total_cmp(&x.0));
        let top: Vec<String> = scores.iter().take(3).map(|(s, n)| format!("{:.3} {}", s, &n[..n.len().min(22)])).collect();
        println!("song {:#04x}: {}", k + 1, top.join(" | "));
    }
}
