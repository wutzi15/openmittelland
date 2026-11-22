//! Runtime access to the user's original install (never copied into the repo).

use mad_assets::{config::Config, font::Font, iconin, le::LeExe, palette, sprite};
use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

type Error = Box<dyn std::error::Error>;

pub struct Assets {
    pub root: PathBuf,
    pub palette: palette::Palette,
    pub font: Font,
    /// Live `CONFI2/CONFIG.DAT` if present, else the shipped default `CONFIG/CONFIG.DAT`.
    /// Loaded CONFIG.DAT; option changes are written back to [`Self::user_saves`] (`write_config`).
    pub config: RefCell<Option<Config>>,
    /// Original `CONFI2` directory holding `GAMEn.DAT`; only ever read.
    pub saves: PathBuf,
    /// Where this app writes its saves (`--save-dir`, `MAD_SAVE_DIR`, default
    /// `~/.local/share/mittelland-ad`). Never inside the original install, see [`Assets::write_save`].
    pub user_saves: PathBuf,
    /// Video mode 3 (`ICONS3`, 15-bit colour): the original's default. Mode 2 is the 8-bit `ICONS2`.
    pub hicolor: bool,
    /// Modern options (`prefs.txt` in [`Self::user_saves`]).
    pub prefs: RefCell<crate::prefs::Prefs>,
    exe: Vec<u8>,
    index: iconin::IconIndex,
    index3: Option<iconin::IconIndex>,
    icons: RefCell<HashMap<u16, Rc<Vec<sprite::Image>>>>,
    icons8: RefCell<HashMap<u16, Rc<Vec<sprite::Image>>>>,
}

/// Icons whose `ICONS3*` file is a byte-identical copy of the 8-bit file (`formats.md` §1): `ICONP6`
/// and `ICONBF1`. They are converted through the palette instead.
const COPY_OF_8BIT: [u16; 2] = [0x196, 0xF1];

impl Assets {
    pub fn load(root: PathBuf, config: Option<PathBuf>, saves: Option<PathBuf>) -> Result<Self, Error> {
        let read = |rel: &str| std::fs::read(root.join(rel)).map_err(|e| format!("{}: {e}", root.join(rel).display()));
        let exe = read("MITTELL.EXE")?;
        // The shipped layout keeps saves in `install/MITTELL/CONFI2`, next to `disk/MITTELL`.
        let saves = saves.unwrap_or_else(|| {
            [root.join("CONFI2"), root.join("../../install/MITTELL/CONFI2")].into_iter().find(|p| p.is_dir()).unwrap_or_else(|| root.join("CONFI2"))
        });
        let user_saves = crate::setup::user_dir();
        let config = [config, Some(user_saves.join("CONFIG.DAT")), Some(saves.join("CONFIG.DAT")), Some(root.join("CONFIG/CONFIG.DAT"))]
            .into_iter()
            .flatten()
            .find_map(|p| std::fs::read(p).ok())
            .and_then(|d| Config::parse(&d).map_err(|e| eprintln!("CONFIG.DAT: {e}")).ok());
        let hicolor = config.as_ref().is_none_or(|c| c.video_mode() == 3) && root.join("ICONS3/ICONIN.DAT").is_file();
        Ok(Self {
            palette: palette::from_exe(&exe)?,
            font: Self::load_font(&root, hicolor)?,
            hicolor,
            index: iconin::IconIndex::parse(&read("ICONS2/ICONIN.DAT")?)?,
            index3: std::fs::read(root.join("ICONS3/ICONIN.DAT")).ok().and_then(|d| iconin::IconIndex::parse(&d).ok()),
            icons8: RefCell::default(),
            exe,
            config: RefCell::new(config),
            saves,
            prefs: RefCell::new(crate::prefs::Prefs::load(&user_saves)),
            user_saves,
            icons: RefCell::default(),
            root,
        })
    }

    fn load_font(root: &std::path::Path, hicolor: bool) -> Result<Font, Error> {
        let (dir, bpp) = if hicolor { ("ICONS3", 2) } else { ("ICONS2", 1) };
        let p = root.join(dir).join("ICONCS.DAT");
        let mut f = Font::parse(&std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?, bpp)?;
        crate::text::fix_font(&mut f);
        Ok(f)
    }

    /// Switches between video mode 3 (hicolor) and 2 (8-bit): font and icon caches are reloaded.
    pub fn set_hicolor(&mut self, on: bool) -> Result<(), Error> {
        let on = on && self.index3.is_some();
        self.font = Self::load_font(&self.root, on)?;
        self.hicolor = on;
        self.icons.borrow_mut().clear();
        Ok(())
    }

    /// Icon `id` in the current video mode: RGB555 pixels in hicolor, palette indices otherwise.
    pub fn icon(&self, id: u16) -> Rc<Vec<sprite::Image>> {
        if !self.hicolor {
            return self.icon8(id);
        }
        if let Some(i) = self.icons.borrow().get(&id) {
            return i.clone();
        }
        let loaded = (!COPY_OF_8BIT.contains(&id)).then(|| self.read_icon3(id)).flatten();
        let imgs = loaded.unwrap_or_else(|| {
            self.icon8(id)
                .iter()
                .map(|im| sprite::Image { pixels: im.pixels.iter().map(|&p| crate::gfx::rgb555(self.palette[p as usize & 255])).collect(), ..im.clone() })
                .collect()
        });
        let rc = Rc::new(imgs);
        self.icons.borrow_mut().insert(id, rc.clone());
        rc
    }

    fn read_icon3(&self, id: u16) -> Option<Vec<sprite::Image>> {
        let idx = self.index3.as_ref()?;
        let dir = if idx.entries[id as usize].flags & iconin::FLAG_B_DIR != 0 { "ICONS3B" } else { "ICONS3" };
        let d = std::fs::read(self.root.join(dir).join(iconin::file_name(id))).ok()?;
        match sprite::decode_icon_file(&d, 2) {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("{dir}/{}: {e}", iconin::file_name(id));
                None
            }
        }
    }

    /// An 8-bit sprite drawn with colour ramp `k`: every pixel `p` becomes entry `16·k + p` of the ramp tables,
    /// read linearly with no range test, so values ≥ 16 run into the following ramps (rules-38 §10); the result
    /// is in the current mode.
    pub fn remap(&self, img: &sprite::Image, k: u8) -> sprite::Image {
        let base = k as u32 * 16;
        let (lo, hi) = if self.hicolor { (self.exe_bytes(0x83D59 + base, 256), self.exe_bytes(0x83F39 + base, 256)) } else { (self.exe_bytes(0x83B79 + base, 256), vec![]) };
        let pixels = img.pixels.iter().map(|&p| if self.hicolor { lo[p as usize & 255] as u16 | (hi[p as usize & 255] as u16) << 8 } else { lo[p as usize & 255] as u16 }).collect();
        sprite::Image { pixels, ..img.clone() }
    }

    /// All frames of 8-bit icon `id` (from `ICONS2` or `ICONS2B` per the index). Cached; panics on a
    /// missing or broken file since every 8-bit icon decodes in a complete install.
    pub fn icon8(&self, id: u16) -> Rc<Vec<sprite::Image>> {
        self.icons8
            .borrow_mut()
            .entry(id)
            .or_insert_with(|| {
                let dir = if self.index.entries[id as usize].flags & iconin::FLAG_B_DIR != 0 { "ICONS2B" } else { "ICONS2" };
                let p = self.root.join(dir).join(iconin::file_name(id));
                let d = std::fs::read(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
                Rc::new(sprite::decode_icon_file(&d, 1).unwrap_or_else(|e| panic!("{}: {e}", p.display())))
            })
            .clone()
    }

    /// NUL-terminated CP437 string at linear address `addr` of this EXE build (see `docs/re/strings.md`).
    pub fn text(&self, addr: u32) -> Vec<u8> {
        let le = LeExe::parse(&self.exe).expect("EXE parsed at load");
        let mut out = vec![];
        while let Some(&[b]) = le.read(addr + out.len() as u32, 1) {
            if b == 0 {
                break;
            }
            out.push(b);
        }
        out
    }

    /// Entry `n` of the EXE's text pointer table at 0xB4258 (docs/re/rules-12 §1.3).
    pub fn text_id(&self, n: u32) -> Vec<u8> {
        let p = u32::from_le_bytes(self.exe_bytes(0xB4258 + 4 * n, 4).try_into().unwrap());
        self.text(0x80000 + p)
    }

    /// Sample effect `n` (`SMPL_<n hex>0.DAT`, rules-14 §1.2) as a WAV, or None when the file does not exist
    /// (the original stays silent then too). 22 kHz is preferred, 44 kHz is the fallback.
    pub fn sample_wav(&self, n: u8) -> Option<Vec<u8>> {
        [("SMP22KHZ", 22050), ("SMP44KHZ", 44100)].into_iter().find_map(|(dir, rate)| {
            let d = self.read(&format!("{dir}/SMPL_{n:02X}0.DAT")).ok()?;
            Some(mad_assets::smpl::Sample::parse(&d).ok()?.to_wav(rate))
        })
    }

    /// The strings of a pointer table as raw EXE bytes (markup codes kept), see `name_table`.
    pub fn text_table(&self, (addr, n): (u32, usize)) -> Vec<Vec<u8>> {
        self.exe_bytes(addr, 4 * n).chunks(4).map(|p| self.text(0x80000 + u32::from_le_bytes(p.try_into().unwrap()))).collect()
    }

    /// A speech line (`SPC22KHZ/<name>.WAV`, rules-14 §1.3 names such as `0229906`), or None if the file is missing.
    pub fn speech_wav(&self, name: &str) -> Option<Vec<u8>> {
        ["SPC22KHZ", "SPC44KHZ"].into_iter().find_map(|dir| self.read(&format!("{dir}/{name}.WAV")).ok())
    }

    /// Table of `n` string pointers at `addr`. The pointers are stored relative to linear 0x80000
    /// (observed: every table in `rules-00-map.md` resolves to its documented names that way).
    pub fn name_table(&self, (addr, n): (u32, usize)) -> Vec<String> {
        let raw = self.exe_bytes(addr, 4 * n);
        raw.chunks(4)
            .map(|p| {
                let a = 0x80000 + u32::from_le_bytes(p.try_into().unwrap());
                self.text(a).iter().filter_map(|&b| crate::input::from_cp437(b)).collect()
            })
            .collect()
    }

    /// `len` raw bytes at linear address `addr` of the EXE (zeros where unmapped).
    pub fn exe_bytes(&self, addr: u32, len: usize) -> Vec<u8> {
        let le = LeExe::parse(&self.exe).expect("EXE parsed at load");
        le.read(addr, len).map_or_else(|| vec![0; len], <[u8]>::to_vec)
    }

    /// `GAMEn.DAT` for slot n (`confi2\\game0.dat` with the digit replaced, `formats.md` §3.2): this
    /// app's save if there is one, else the original install's.
    pub fn save_path(&self, slot: u8) -> PathBuf {
        let name = format!("GAME{slot}.DAT");
        let own = self.user_saves.join(&name);
        if own.is_file() { own } else { self.saves.join(name) }
    }

    /// Writes slot n into [`Self::user_saves`]. Refuses any directory inside the original data tree
    /// (the install root's grandparent covers both `disk/MITTELL` and `install/MITTELL`).
    pub fn write_save(&self, slot: u8, data: &[u8]) -> std::io::Result<PathBuf> {
        self.write_user(&format!("GAME{slot}.DAT"), data)
    }

    /// Patches the loaded config image and writes it to [`Self::user_saves`] (the original writes the whole
    /// 0x279-byte image back, rules-43 §2). Nothing happens without a loaded config.
    pub fn write_config(&self, patch: impl FnOnce(&mut [u8])) -> std::io::Result<()> {
        let mut c = self.config.borrow_mut();
        let Some(c) = c.as_mut() else { return Ok(()) };
        patch(&mut c.raw);
        self.write_user("CONFIG.DAT", &c.raw).map(drop)
    }

    fn write_user(&self, name: &str, data: &[u8]) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(&self.user_saves)?;
        let dir = self.user_saves.canonicalize()?;
        let install = self.root.canonicalize()?;
        // the install tree two levels up, but never the filesystem root (data mounted at /disk/MITTELL)
        let tree = install.ancestors().nth(2).filter(|t| t.parent().is_some()).unwrap_or(&install);
        // the original's save folder only counts when it exists (an empty path would match every directory)
        let saves = self.saves.canonicalize().ok();
        if dir.starts_with(tree) || saves.is_some_and(|s| dir.starts_with(s)) {
            return Err(std::io::Error::other(format!("refusing to write saves into the original install ({})", dir.display())));
        }
        let p = dir.join(name);
        std::fs::write(&p, data)?;
        Ok(p)
    }

    pub fn read(&self, rel: &str) -> std::io::Result<Vec<u8>> {
        std::fs::read(self.root.join(rel))
    }
}

#[cfg(test)]
mod tests {
    /// Menu strings come from the user's EXE at the addresses in `docs/re/strings.md`.
    #[test]
    fn real_exe_text() {
        let Ok(root) = std::env::var("MAD_DATA") else { return };
        let a = super::Assets::load(root.into(), None, None).unwrap();
        assert_eq!(a.text(0xBD434), b"Ein neues Spiel beginnen");
        assert_eq!(a.text(0xBDBB8), b"Ja");
        assert_eq!(a.icon(0x31)[0].width, 640);
        let locs = a.name_table((0xB6194, 21));
        assert_eq!((locs[0].as_str(), locs[20].as_str()), ("Fürstenbrück", "Säbeltal"));
        assert_eq!(a.name_table((0xB6A80, 10))[9], "Fürstin");
    }

    /// Saves never land in the original data tree.
    #[test]
    fn write_save_guard() {
        let Ok(root) = std::env::var("MAD_DATA") else { return };
        let mut a = super::Assets::load(root.clone().into(), None, None).unwrap();
        // the original's save folder is refused, wherever it lives
        let orig = std::env::temp_dir().join(format!("mad-app-orig-{}", std::process::id()));
        std::fs::create_dir_all(&orig).unwrap();
        (a.saves, a.user_saves) = (orig.clone(), orig.clone());
        assert!(a.write_save(0, b"x").is_err());
        std::fs::remove_dir_all(&orig).unwrap();
        // a missing original save folder blocks nothing
        let tmp = std::env::temp_dir().join(format!("mad-app-test-{}", std::process::id()));
        a.user_saves = tmp.clone();
        assert_eq!(std::fs::read(a.write_save(3, b"x").unwrap()).unwrap(), b"x");
        assert_eq!(a.save_path(3), tmp.join("GAME3.DAT"));
        a.write_config(|raw| raw[0xF3] = 5).unwrap();
        assert_eq!(std::fs::read(tmp.join("CONFIG.DAT")).unwrap()[0xF3], 5);
        std::fs::remove_dir_all(tmp).unwrap();
    }
}
