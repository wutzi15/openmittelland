# OpenMittelland

An open-source reimplementation of *Mittelland AD* (1996, DOS; originally *Die Fugger 2*), written in Rust.
It runs on the game files of **your own copy**; no original assets or code are included.
Not affiliated with the original's authors or rights holders.

Status: playable, far from finished. The simulation, the Kontor and its rooms, trade, auctions, battles, sub-plots, courtship, trials and the event scenes are built from the original's data, texts and pictures; sound effects, scene music and speech are partial, MIDI is not implemented. `docs/FIDELITY.md` lists what was compared against the original, `docs/IMPL-GAPS.md` every guess and missing piece.

## Playing

Download the archive for your system from the [releases page](../../releases) and start it. On the first start it looks for the game in the usual places (next to the program, your home, Games and Downloads folders, Steam and GOG libraries) and asks for the folder if it finds nothing: pick the game's install folder or the `MITTELL` folder that holds `MITTELL.EXE`. The choice is remembered.

Saves and options go to `%APPDATA%\mittelland-ad` (Windows) or `~/.local/share/mittelland-ad` (macOS, Linux); the original install is never written to.

## Building

Rust 1.100 or newer. Linux needs `libasound2-dev` (ALSA).

    cargo build --release -p mad-app
    target/release/mad-app [--data <MITTELL folder>] [--mode 2|3]

`--mode 3` (default, as in the original) draws the 15-bit `ICONS3` art, `--mode 2` the 8-bit `ICONS2` art. `--data` (or `MAD_DATA`) skips the search.

    MAD_DATA=<MITTELL folder> cargo test --workspace   # tests needing the original files skip without MAD_DATA
    cargo clippy --workspace --all-targets -- -D warnings

`tools/release/release.sh VERSION` builds the release archives (macOS here, Linux and Windows on build hosts over SSH). `ARCHITECTURE.md` describes the crates, `AGENTS.md` the clean-room rules the work follows.

## License

GPL-3.0-or-later, see `LICENSE`. The game data you supply stays under its own copyright.
