//! usage: exe_strings <MITTELL.EXE> [min_len]
//! Lists text strings in the data object (linear 0x80000..) as `address<TAB>text`, decoding CP437.
//! Strings end at NUL; the game's `$`-escapes (`$N`, `$Z`, ...) are kept verbatim.
use mad_assets::le::LeExe;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let exe = std::fs::read(&a[1]).unwrap();
    let min: usize = a.get(2).map_or(4, |s| s.parse().unwrap());
    let le = LeExe::parse(&exe).unwrap();
    let obj = &le.objects[1];
    let data = le.read(obj.base, obj.file_len()).unwrap();
    let cp437 = |b: u8| match b {
        0x81 => 'ü', 0x84 => 'ä', 0x8E => 'Ä', 0x94 => 'ö', 0x99 => 'Ö', 0x9A => 'Ü', 0xE1 => 'ß', 0x0A => '\n',
        0x20..=0x7E => b as char,
        _ => '\0',
    };
    let mut start = 0;
    for (i, &b) in data.iter().enumerate() {
        if cp437(b) != '\0' {
            continue;
        }
        let s: String = data[start..i].iter().map(|&c| cp437(c)).collect();
        let letters = s.chars().filter(|c| c.is_alphabetic()).count();
        if b == 0 && s.chars().count() >= min && letters * 2 >= s.chars().count() {
            println!("{:#07x}\t{}", obj.base as usize + start, s.replace('\n', "\\n"));
        }
        start = i + 1;
    }
}
