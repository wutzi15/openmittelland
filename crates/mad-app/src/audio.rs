//! Soundtrack (OGG) and sample (WAV, `SMPL_*.DAT`) playback via rodio. Silently does nothing
//! without an output device or with `--mute`.

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::{io::Cursor, path::Path};

/// Soundtrack track number for MIDI song `n` (index 0 unused), docs/re/rules-14 §4.2. The rows rules-14 marks
/// ambiguous by length are confirmed by rhythm (`examples/song_match.rs`: MIDI note onsets against the OGG's onset
/// strength); songs 0x0F and 0x13 (same MIDI length) stay close, 0x13 prefers 04 "Der Kaufmann" at both frame sizes
/// and fits the Kontor by title, so 0x0F gets 05 "Parks und Ländereien" [UNVERIFIED].
const OGG_OF_SONG: [u8; 25] = [0, 1, 13, 14, 2, 16, 9, 22, 24, 21, 3, 20, 18, 19, 23, 5, 10, 15, 11, 4, 12, 7, 17, 8, 6];

pub struct Audio {
    out: Option<MixerDeviceSink>,
    music: Option<Player>,
    song: Option<u8>,
    /// master levels of the Musik screen, CONFIG 0x195 / 0x194 over 127 (rules-44 §1.2: the SB mixer's FM and voice)
    music_vol: f32,
    fx_vol: f32,
}

impl Audio {
    pub fn new(enabled: bool) -> Self {
        let out = enabled.then(DeviceSinkBuilder::open_default_sink).and_then(|r| r.map_err(|e| eprintln!("audio: {e}")).ok());
        Self { out, music: None, song: None, music_vol: 1.0, fx_vol: 1.0 }
    }

    /// Music and digital (samples and speech) master levels, 0..127 as stored in CONFIG.DAT.
    pub fn set_volumes(&mut self, music: u8, digital: u8) {
        (self.music_vol, self.fx_vol) = (music.min(127) as f32 / 127.0, digital.min(127) as f32 / 127.0);
        if let Some(p) = &self.music {
            p.set_volume(self.music_vol);
        }
    }

    /// Original MIDI song `n` (`MIDSONGS/SONGn.MID`), played as its soundtrack file `NN - *.ogg` from `dir`
    /// (the mapping by duration is in docs/re/rules-14 §4.2, several rows [UNVERIFIED]); MIDI itself is
    /// not supported. Random groups (0x19..0x1E) are resolved by the caller. Does nothing if that song
    /// is already playing (the original's mode 1).
    pub fn play_song(&mut self, dir: Option<&Path>, n: u8) {
        if self.song == Some(n) {
            return;
        }
        let Some(&track) = OGG_OF_SONG.get(n as usize).filter(|&&t| t != 0) else { return };
        let prefix = format!("{track:02} - ");
        let file = dir.and_then(|d| std::fs::read_dir(d).ok()).into_iter().flatten().flatten().map(|e| e.path()).find(|p| {
            p.file_name().and_then(|f| f.to_str()).is_some_and(|f| f.starts_with(&prefix) && f.ends_with(".ogg"))
        });
        if let Some(f) = file {
            self.play_music(&f);
            self.song = Some(n);
        }
    }

    /// Mode 2 of `music` (rules-14 §4.1): start song `n` only when no song plays.
    pub fn play_song_if_idle(&mut self, dir: Option<&Path>, n: u8) {
        if self.song.is_none() {
            self.play_song(dir, n);
        }
    }

    /// Loops an OGG track until replaced.
    fn play_music(&mut self, path: &Path) {
        self.music = None;
        let Some(out) = &self.out else { return };
        let src = match std::fs::File::open(path).map_err(|e| e.to_string()).and_then(|f| Decoder::try_from(f).map_err(|e| e.to_string())) {
            Ok(s) => s,
            Err(e) => return eprintln!("music {}: {e}", path.display()),
        };
        let p = Player::connect_new(out.mixer());
        p.set_volume(self.music_vol);
        p.append(src.repeat_infinite());
        self.music = Some(p);
    }

    /// Fire-and-forget WAV (speech files are plain RIFF; samples via `Sample::to_wav`).
    pub fn play_wav(&self, wav: Vec<u8>) {
        let Some(out) = &self.out else { return };
        match Decoder::try_from(Cursor::new(wav)) {
            Ok(s) => out.mixer().add(s.amplify(self.fx_vol)),
            Err(e) => eprintln!("wav: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The soundtrack and a converted sample decode. Needs `MAD_DATA` (skipped without it); works
    /// with either install layout (`disk/MITTELL` ships `SMP22KHZ`, `install/MITTELL` only
    /// `SMP44KHZ`) and names the missing path when the directory is not a Mittelland install.
    /// Every song number the game uses maps to an existing soundtrack file name pattern, and the
    /// effect samples of rules-14 exist as files (needs `MAD_DATA`).
    #[test]
    fn real_cue_samples_exist() {
        let Ok(root) = std::env::var("MAD_DATA") else { return };
        let assets = crate::assets::Assets::load(root.into(), None, None).unwrap();
        for n in [0x42u8, 0x4C, 0x49, 0x4A, 0x4B, 0x4E, 0x84, 0x45] {
            assert!(assets.sample_wav(n).is_some(), "SMPL_{n:02X}0.DAT");
        }
        assert!(OGG_OF_SONG.iter().skip(1).all(|&t| (1..=24).contains(&t)));
        let mut seen = OGG_OF_SONG[1..].to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 24, "each track is used by exactly one song");
    }

    #[test]
    fn real_files_decode() {
        let Ok(root) = std::env::var("MAD_DATA") else { return };
        let root = Path::new(&root);
        let track = root.join("../../../soundtrack/01 - Unsere Zeit.ogg");
        let ogg = std::fs::File::open(&track).unwrap_or_else(|e| panic!("MAD_DATA={}: soundtrack {}: {e}", root.display(), track.display()));
        assert_eq!(Decoder::try_from(ogg).unwrap().take(1000).count(), 1000);
        let (path, rate) = [("SMP22KHZ", 22050), ("SMP44KHZ", 44100)]
            .into_iter()
            .map(|(dir, r)| (root.join(dir).join("SMPL_2D0.DAT"), r))
            .find(|(p, _)| p.is_file())
            .unwrap_or_else(|| panic!("MAD_DATA={}: no SMP22KHZ/ or SMP44KHZ/SMPL_2D0.DAT (not a MITTELL directory?)", root.display()));
        let d = std::fs::read(&path).unwrap();
        let wav = mad_assets::smpl::Sample::parse(&d).unwrap().to_wav(rate);
        assert_eq!(Decoder::try_from(Cursor::new(wav)).unwrap().count(), d.len() - 20);
    }
}
