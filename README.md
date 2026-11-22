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

OpenMittelland adds optional features the original does not have. All are off by default, so a fresh install plays like the original. They are stored in `prefs.txt` next to the saves, never in `CONFIG.DAT` or the `GAMEn.DAT` saves, which stay readable by the original. Open the Extras window with **Tab** (wherever no other window is open) or Spieloptionen → Optionen → "Extras...". It has two pages; the last line switches between them. Click an option's bullet or value to change it; changes apply at once.

| page | option | key | what it does |
|---|---|---|---|
| Anzeige und Komfort | Vollbild | F11, Alt+Enter | fullscreen |
| | Ganzzahlig skalieren | | sharp whole-number scaling (on by default) |
| | Tempo | | game speed 1x to 4x |
| | Sprache | | German or English for the Extras' own texts |
| | Hinweise | | tooltips |
| | Tasten | `?` | keyboard shortcuts |
| | Autosave alle | | history saves every 1, 5 or 10 years |
| | Rückgängig | `U` | take the last turn end back |
| | Spielnummer wählen | | choose the game number or play the daily game |
| | Szenen überspringen | Space | skip court, funeral, election and dismissal scenes |
| | Posten überspringen | Tab | let an auction lot run without you, at once |
| Hilfen und Hausregeln | Schuldenwarnung | | warnings at the start of your turn |
| | Produktionshilfe | `H` | production advice on the branch screen |
| | Bilanz | `B` | your yearly accounts |
| | Preistabelle | `P` | all prices in all cities |
| | Alles verkaufen | `V` | sell the stock of every branch at once |
| | Chronik | `L` | every report of this session |
| | Aufträge kopieren | `C`, `E` | copy orders from one branch to another |
| | Amtsinfo | | who fills an office and how the vote stands |
| | Sicherheitsfragen | | ask before actions that can't be taken back |
| | Gnade bei Schulden | | house rule: debt never takes your last branch |
| | Bis 255 Arbeiter | | house rule: up to 255 workers per production order |

The keys work in the Kontor (some also on the Handel map or the branch screen) and only while their option is on.

#### Display and comfort

- **Vollbild, Ganzzahlig skalieren.** The game is drawn at 640×480 and scaled to the window. Whole-number scaling keeps every pixel sharp and leaves black borders; with it off the picture fills the largest 4:3 area of the window.
- **Tempo.** Runs the game clock 2, 3 or 4 times faster: report scrolling, waits, animations, films and battles. The music plays at normal speed.
- **Sprache.** Only the texts OpenMittelland adds (Extras, tooltips, the new windows) switch to English. Everything else is the original's German text, read from your copy of the game.
- **Hinweise.** Rest the pointer for about half a second on a spot in the Kontor, a room, the Handel map, the branch screen or a menu line, and a label says what a click does. In the Extras window it explains each option.
- **Tasten.** In the Kontor: `M` Handel map, `S` Schreibstube, `R` Spielstand, `G` wages, `Z` Hinterzimmer, `K` Kirche, `T` Söldner & Räuber, `D` Kreditbuch, `O` Spieloptionen; Space or Enter end the turn (with the usual confirmation), Esc closes a window or leaves a room, `1`–`9` pick a line of an open menu, `?` lists all keys.
- **Autosave alle.** At every year start that is a multiple of 1, 5 or 10, the game is also saved to `HISTORY/<year>.DAT` in the saves folder, in the same format as a normal save. The newest ten are kept. Load them from the start menu: "Spiel laden" → "Verlauf...".
- **Rückgängig.** Before you end a turn, the game keeps a copy of it. In the next turn, `U` in the Kontor (or "Rückgängig" in Optionen) goes back to that moment, once. The dice are rolled fresh after an undo, so even the same decisions can turn out differently.
- **Spielnummer wählen.** A new game normally starts from the clock (one of 6,000 worlds, as in the original). With this on, a screen after the player count offers "Zufall" (the clock), "Tagesspiel" (the same world for everyone on the same day) or a number from 0 to 15203: the same number and player count give the same world. The running game's number is shown in the Extras window until you load another game.
- **Szenen überspringen.** In a court scene (the charge on the stairs, the court room with witnesses, pleas and the judges' votes, the creditors' jury) Space jumps to the verdict; a hint at the bottom says so. Only the pictures and waits go: every question (bribes, the plea, your vote or sentence as judge) was already asked before the scene, and the dice were already rolled, so the outcome is exactly what watching it would give. The verdict animation and the costs still show. At a death, Space skips the whole funeral (candle film, gravestone, mourners, the testament and the notary's last words); who inherits is decided before it starts. Elections and dismissals skip the same way, straight to the next screen; an election where you still have to vote stays until you have voted. A speech already playing finishes.
- **Posten überspringen.** In the auction hall, Tab or the line "Tab: Posten überspringen" lets the current lot run without you: the computers bid against each other as if you kept silent, with no waits between the calls, and the sold or unsold line follows. Tab on a lot's introduction skips the lot that follows. The dice and the result are those of sitting through it without pressing your key. Esc in the original is different: it ends the lot unsold at once, computer bids dropped.

#### Helpers

- **Schuldenwarnung.** At the start of your turn a "Warnung" window appears if your debts will trigger a seizure at the year end (and says so when only one branch is left, which can then be lost), if a loan falls due next year, or if your confirmed orders cost more than your cash.
- **Produktionshilfe.** On the branch screen press `H` or click "Hilfe (H)". For the branch's first production order it shows: the best ratio of workers to means for that good (for example Korn 5:1, Fisch 3:1), how many means your workers can use, the expected yield at average weather times the local price, the cost and the profit. Each year a limit between 30 and 39 is drawn; every worker above it adds only half as much. Below that, a table lists every good of the city with its ratio, means price and local price.
- **Bilanz.** `B` in the Kontor. Page one: net worth and cash per year with bars, last ten years. Page two ("Niederlassungen"): per city, the sales of the shipments from that branch minus its wages, means and transport. Not counted per branch: goods sold directly on the branch screen, tolls and director fees. A year appears at the start of the next turn. The accounts are kept while the game runs and start empty after loading.
- **Preistabelle.** `P` in the Kontor: all 15 goods in all ten cities; the best price of each good is in gold.
- **Alles verkaufen.** `V` in the Kontor or on the Handel map, or the gold "Verkaufen" on the map's sea. After a Ja/Nein question it sells the whole stock of every branch at local prices, exactly as if you clicked the empty-warehouse icon in each branch, and shows what was sold where.
- **Chronik.** `L` in the Kontor: the text of every report window of this session (turn ends, year ends, fate), newest first, page by page. It starts empty after loading.
- **Aufträge kopieren.** On a branch screen `C` copies its four order slots, `E` pastes them into the branch you are looking at. The game checks each order at the new place; an order that is impossible there shows as impossible. Confirmed orders already carry over to the next year by themselves.
- **Amtsinfo.** In the bribery office trees (gold display → map → a city, county, the principality or the church level), rest the pointer on an office. The box shows who holds it (a held seat frees up only by dismissal, death or promotion) or that it is filled at the year end; who appoints it and your favour with each of them (Vogt and Regent are drawn by lot; someone who blackmails an elector votes in his place); the applicants, or, if nobody applied, that three random eligible persons and the players' rivals stand, and that the seat stays empty one more year if you are eligible; whether you are eligible or have applied; and each elector's current favourite with the likely winner. The forecast uses the game's own voting rule but today's favours: the year end changes favours, dismisses and replaces people before the vote, and a rival's score gets a random 0 to 19 on top.
- **Sicherheitsfragen.** A Ja/Nein question before bribes, denunciations, blackmail, hiring spies, dismissing troops and selling a castle. Saboteurs and selling a branch or residence already ask in the original.

#### House rules

These change the game. A save played with them can behave differently when loaded in the original.

- **Gnade bei Schulden.** When debt reaches the last stage, the original rolls between a court case and seizing your last branch, which ends the game. With this on it is always the court case. The dice are still rolled, so the rest of the game stays the same.
- **Bis 255 Arbeiter.** Production orders take up to 255 workers instead of 40. Workers above the yearly limit of 30 to 39 add half as much each, so large crews pay off less per worker.

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
