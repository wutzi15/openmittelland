# OpenMittelland

An open-source reimplementation of *Mittelland AD* (1996, DOS; originally *Die Fugger 2*), written in Rust.
It runs on the game files of **your own copy**; no original assets or code are included.
Not affiliated with the original's authors or rights holders.

Status: playable, far from finished. The simulation, the Kontor and its rooms, trade, auctions, battles, sub-plots, courtship, trials and the event scenes are built from the original's data, texts and pictures; sound effects, scene music and speech are partial, MIDI is not implemented. `docs/FIDELITY.md` lists what was compared against the original, `docs/IMPL-GAPS.md` every guess and missing piece.

## Why

*Mittelland AD* is a great game: a trading and career simulation where you build a merchant house, buy your way into offices, court, scheme, bribe, sue and go to war, with a whole small world that keeps living around you. It deserves to be played today.

Today that means DOSBox, and DOSBox is a poor way to play it: the mouse is captured and drifts, the window is either tiny or blurry, sound needs setting up, saves sit inside an emulated drive, and the whole thing runs a 1990s DOS extender just to show a 640×480 picture. OpenMittelland runs the same game natively on Windows, macOS and Linux, from the files of your own copy, as a normal program you double-click.

## Running

You need your own copy of *Mittelland AD*: either the installed game (its folder holds `game/disk/MITTELL`, and `soundtrack` with the music) or a DOS install with the `MITTELL` folder (`MITTELL.EXE`, `ICONS2`, `ICONS3`, ...). OpenMittelland only reads these files; it never changes them.

Download the archive for your system from the [releases page](../../releases):

- **Windows** (x86_64): unzip, start `OpenMittelland.exe`. SmartScreen may warn about an unknown publisher: "More info" → "Run anyway".
- **macOS** (Apple silicon and Intel, macOS 11+): unzip and move `OpenMittelland.app` where you like. The app is not notarized, so the first start is refused ("Apple could not verify..."): click "Done", then open System Settings → Privacy & Security and click "Open Anyway" next to OpenMittelland. Or in Terminal: `xattr -dr com.apple.quarantine OpenMittelland.app`.
- **Linux** (x86_64, glibc 2.34 or newer, e.g. Ubuntu 22.04, Debian 12): unpack and run `./openmittelland`. Needs ALSA (`libasound2`).

On the first start it looks for the game in the usual places (next to the program, your home, Games, Spiele, Downloads, Desktop and Documents folders, Steam and GOG libraries, and on Windows `C:\`, `C:\Games`, `C:\GOG Games`). If it finds nothing it asks for the folder: pick the game's install folder or the `MITTELL` folder itself. The choice is remembered; to change it, delete `data-path.txt` in the saves folder below.

The music plays when the `soundtrack` folder sits where the installed game keeps it (three levels above `MITTELL`); `--music <folder>` points elsewhere.

Saves and options go to `%APPDATA%\mittelland-ad` (Windows) or `~/.local/share/mittelland-ad` (macOS, Linux), `MAD_SAVE_DIR` overrides it. Saves of the original install are offered for loading too.

### Extras

OpenMittelland adds optional features the original does not have. All are off by default, so a fresh install plays like the original; they live in `prefs.txt` next to the saves and never in `CONFIG.DAT` or the `GAMEn.DAT` saves, which stay readable by the original. Open them with **Tab** or Spieloptionen → Optionen → "Extras...":

| page | option | what it does |
|---|---|---|
| Anzeige und Komfort | Vollbild | fullscreen (also F11, Alt+Enter) |
| | Ganzzahlig skalieren | whole-number scaling with black borders (on by default, as before) |
| | Tempo | 1x to 4x speed for scrolling, waits and animations |
| | Sprache | German or English for the Extras' own texts (the game's texts come from your copy) |
| | Hinweise | tooltips after resting the pointer |
| | Tasten | keyboard shortcuts, list with `?` |
| | Autosave alle | history saves every 1, 5 or 10 years ("Verlauf..." in the load dialog) |
| | Rückgängig | take the last turn end back once (`U`) |
| | Spielnummer wählen | type a game number or play the daily game |
| Hilfen und Hausregeln | Schuldenwarnung | warns before debt seizure, due loans, orders the cash can't cover |
| | Produktionshilfe | best worker:means ratio, means needed, expected yield and profit (`H` on the branch screen) |
| | Bilanz | net worth, cash and branch results per year (`B`) |
| | Preistabelle | every good in every city, best price marked (`P`) |
| | Alles verkaufen | sell every branch's stock at once (`V`) |
| | Chronik | all report texts of this session (`L`) |
| | Aufträge kopieren | copy a branch's orders (`C`) and paste them into another (`E`) |
| | Sicherheitsfragen | asks before bribes, denunciations, blackmail, spies, dismissing troops, selling a castle |
| | Gnade bei Schulden | house rule: debt never takes your last branch |
| | Bis 255 Arbeiter | house rule: production orders up to 255 workers (the original: 40) |

The house rules change the game: a save played with them may not behave the same in the original.

Command-line options:

| option | effect |
|---|---|
| `--data <folder>` (or `MAD_DATA`) | the `MITTELL` folder; skips the search |
| `--music <folder>` | the soundtrack folder |
| `--mode 2` / `--mode 3` | 8-bit `ICONS2` or 15-bit `ICONS3` art (3 is the default, as in the original) |
| `--start intro\|menu\|missions\|players` | the first screen (default: the intro) |
| `--mute` | no sound |
| `--save-dir <folder>` | where saves and options are written |

## Building

Rust 1.100 or newer. Linux needs `libasound2-dev` (ALSA).

    cargo build --release -p mad-app
    target/release/mad-app [--data <MITTELL folder>] [--mode 2|3]

    MAD_DATA=<MITTELL folder> cargo test --workspace   # tests needing the original files skip without MAD_DATA
    cargo clippy --workspace --all-targets -- -D warnings

`tools/release/release.sh VERSION` builds the release archives (macOS here, Linux and Windows on build hosts over SSH). `ARCHITECTURE.md` describes the crates, `AGENTS.md` the clean-room rules the work follows.

## Contributing

Issues and pull requests are welcome, in English or German.

- **Bugs**: say what you did, what the original game does there and what OpenMittelland does instead; a screenshot of both helps most. Saves (`GAME*.DAT` from the folder above) make a report reproducible.
- **Pull requests**: keep `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` green, and run the tests with `MAD_DATA` pointing at your copy if you can (CI has no game files, so those tests skip there).
- **Never include** files of the original game (pictures, sounds, texts, saves of its install) or code taken from it, also not in issues. The project rebuilds the game from its behaviour; `AGENTS.md` explains the rules.
- Questions about how something works in the original belong in an issue too.

## License

GPL-3.0-or-later, see `LICENSE`. The game data you supply stays under its own copyright.
