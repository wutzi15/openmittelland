//! Finding the original game files when no `--data` is given (the downloaded app): the folder chosen last
//! time, then the usual install places, then the player picks it in a folder dialog. The choice is kept in
//! the user folder, next to the saves.

use std::path::{Path, PathBuf};

/// Where this app keeps saves, options and the chosen data folder: `%APPDATA%\mittelland-ad` on Windows,
/// `~/.local/share/mittelland-ad` elsewhere. `MAD_SAVE_DIR` overrides it.
pub fn user_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("MAD_SAVE_DIR") {
        return d.into();
    }
    match std::env::var_os("APPDATA").filter(|_| cfg!(windows)) {
        Some(a) => PathBuf::from(a).join("mittelland-ad"),
        None => home().join(".local/share/mittelland-ad"),
    }
}

fn home() -> PathBuf {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_default()
}

fn remembered() -> PathBuf {
    user_dir().join("data-path.txt")
}

fn is_data(p: &Path) -> bool {
    p.join("MITTELL.EXE").is_file() && p.join("ICONS2/ICONIN.DAT").is_file()
}

/// The `MITTELL` folder at `p` or below it: the folder itself, the DOS install (`MITTELL`), the re-release
/// layout (`game/disk/MITTELL`), each also one folder further down (a Steam or GOG library, a download folder).
pub fn resolve(p: &Path) -> Option<PathBuf> {
    let here = |d: &Path| ["", "MITTELL", "disk/MITTELL", "game/disk/MITTELL"].into_iter().map(|s| d.join(s)).find(|c| is_data(c));
    here(p).or_else(|| {
        let mut subs: Vec<PathBuf> = std::fs::read_dir(p).ok()?.flatten().map(|e| e.path()).filter(|d| d.is_dir()).collect();
        subs.sort();
        subs.iter().find_map(|d| here(d))
    })
}

/// The remembered folder, or the first of the usual places that holds the game.
pub fn find() -> Option<PathBuf> {
    if let Some(p) = std::fs::read_to_string(remembered()).ok().map(|s| PathBuf::from(s.trim())).filter(|p| is_data(p)) {
        return Some(p);
    }
    let h = home();
    // next to the program (also the folder holding a macOS .app: Contents/MacOS is three levels down)
    let mut roots: Vec<PathBuf> = std::env::current_exe().ok().map(|e| e.ancestors().skip(1).take(4).map(Path::to_path_buf).collect()).unwrap_or_default();
    roots.extend(std::env::current_dir().ok());
    roots.extend(["", "Games", "Spiele", "Downloads", "Desktop", "Documents"].map(|s| h.join(s)));
    roots.extend(
        [
            "Library/Application Support/Steam/steamapps/common",
            ".steam/steam/steamapps/common",
            ".local/share/Steam/steamapps/common",
            ".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common",
            "GOG Games",
        ]
        .map(|s| h.join(s)),
    );
    if cfg!(windows) {
        roots.extend(
            ["C:\\", "C:\\Program Files (x86)\\Steam\\steamapps\\common", "C:\\Program Files\\Steam\\steamapps\\common", "C:\\GOG Games", "C:\\Program Files (x86)\\GOG Galaxy\\Games", "C:\\Games", "C:\\Spiele"]
                .map(PathBuf::from),
        );
    }
    roots.iter().find_map(|r| resolve(r))
}

/// Asks for the folder until it holds the game or the player cancels; keeps the answer.
pub fn ask() -> Option<PathBuf> {
    use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
    let intro = "Die Spieldaten von Mittelland AD wurden nicht gefunden. Bitte wählt den Ordner Eures Spiels \
                 (den Installationsordner oder den Ordner MITTELL mit MITTELL.EXE).\n\n\
                 The Mittelland AD game files were not found. Please choose the folder of your copy of the game \
                 (its install folder or the MITTELL folder holding MITTELL.EXE).";
    let mut text = intro.to_string();
    loop {
        let go = MessageDialog::new().set_level(MessageLevel::Info).set_title("OpenMittelland").set_description(&text).set_buttons(MessageButtons::OkCancel).show();
        if !matches!(go, MessageDialogResult::Ok | MessageDialogResult::Yes) {
            return None;
        }
        let picked = rfd::FileDialog::new().set_title("Mittelland AD: Spielordner / game folder").pick_folder()?;
        if let Some(p) = resolve(&picked) {
            let _ = std::fs::create_dir_all(user_dir()).and_then(|_| std::fs::write(remembered(), p.to_string_lossy().as_bytes()));
            return Some(p);
        }
        text = format!("In {} ist kein MITTELL.EXE mit ICONS2.\nNo MITTELL.EXE with ICONS2 in that folder.\n\n{intro}", picked.display());
    }
}

/// A fatal start error: on stderr, and in a box for a program started without a terminal (not for the
/// headless `--screenshot` runs of the A/B scripts).
pub fn fatal(msg: &str, code: i32) -> ! {
    eprintln!("mad-app: {msg}");
    if std::env::args().any(|a| a == "--screenshot") {
        std::process::exit(code)
    }
    let _ = rfd::MessageDialog::new().set_level(rfd::MessageLevel::Error).set_title("OpenMittelland").set_description(msg).show();
    std::process::exit(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The re-release layout is found from its install folder and from the library folder above it.
    #[test]
    fn resolve_finds_nested_layouts() {
        let t = std::env::temp_dir().join(format!("mad-setup-{}", std::process::id()));
        let data = t.join("Mittelland AD/game/disk/MITTELL");
        std::fs::create_dir_all(data.join("ICONS2")).unwrap();
        std::fs::write(data.join("MITTELL.EXE"), b"").unwrap();
        std::fs::write(data.join("ICONS2/ICONIN.DAT"), b"").unwrap();
        assert_eq!(resolve(&data), Some(data.clone()));
        assert_eq!(resolve(&t.join("Mittelland AD")), Some(data.clone()));
        assert_eq!(resolve(&t), Some(data.clone()));
        assert_eq!(resolve(&t.join("Mittelland AD/game/disk/MITTELL/ICONS2")), None);
        let _ = std::fs::remove_dir_all(&t);
    }
}
