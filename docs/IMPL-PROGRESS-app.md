# mad-app progress (resumable)

Built only from `docs/spec`, `docs/re` and `mad-assets`. Guesses and placeholders: `IMPL-GAPS.md`.

## Run
```
export MAD_DATA=…/MittellandAD/game/disk/MITTELL
cargo run -p mad-app                          # intro → start menu (Escape/click skips)
cargo run -p mad-app -- --start menu|missions|players [--mute] [--sample 2D0]
# headless: N ticks of 10 ms, then dump the 640x480 framebuffer
cargo run -p mad-app -- --start players --type 'Anna@0' --click 293,423@1 --frames 3 --mouse 300,250 --screenshot out.png
```
Saves: `--save-dir DIR` / `MAD_SAVE_DIR` (default `~/.local/share/mittelland-ad`; never the install). Music: `--music DIR` / `MAD_MUSIC`. The default is `$MAD_DATA/../../../soundtrack`. Config: `--config FILE`; the default is `$MAD_DATA/CONFI2/CONFIG.DAT`, then `CONFIG/CONFIG.DAT`.

## Layout (`crates/mad-app/src`)
- `gfx.rs`: 640×480 indexed `Frame` + palette, clipped/flipped blit, `fit`/`to_frame` window scaling (integer scale, centred; 4:3 fit below 1×), PNG/PPM screenshot.
- `input.rs`: `Event` (left/right click, keys), `Cursor` shapes `00-04`, Unicode→CP437.
- `text.rs`: §8.3 markup layout (all position/alignment codes, `%m %a %d %s %c %%`, skips `%1N1`-style refs), font fix-ups, word wrap + justification, choice hit-boxes.
- `assets.rs`: palette and strings from the EXE, ICONS2/2B icons (cached), font.
- `audio.rs`: rodio. Looped OGG music, fire-and-forget WAV (SMPL via `Sample::to_wav`).
- `screens/`: `Screen` trait (`update` per fixed tick, `draw` returns the pointer). `film` (FLC), `menu` (generic `Dialog` + start menu, missions, player count), `setup` (per-player form), `placeholder`.
- `main.rs`: args, winit + softbuffer window, 10 ms fixed tick with a 100 ms catch-up cap, headless screenshot path.

## Status
- [x] window, scaling, own pointer, fixed timestep, input abstraction
- [x] icons + font + EXE strings
- [x] intro/outro FLC, skippable
- [x] start menu → missions → player count → per-player form → placeholder; right click/Escape goes back
- [x] audio: menu OGG loop, `--sample` plays an SMPL file
- [x] menu art and layout per `formats.md` §7 (ICOND4 book at y=40, ICON22 banner, window rects, tile rule table, item visibility from CONFIG, INTRO/INTROB choice, SONG4 → OGG 04)
- [x] §8: intro at (0,10) in 320×200 mode, 10 fps, speech cues `SPCxxKHZ/0000000-0000012.WAV`; text layout per §8.3 (12 px inset, `$nH`, margins, columns, space = 5 px, justified default); player creation two-column layout per §8.2
- [x] mad-sim wired (2026-11-03): new game from the player screens (`State::new_game`), load original `GAMEn.DAT` (`State::from_game_dat`), turn reports, Kontor + banner, branch screen with order confirmation, end round (`Command::EndTurn`), year cycle. Difficulty 0 Leicht .. 3 none (rules-10 §1).
- [x] §9 geometry applied: banner, Kontor hotspots + labels, Nächster Spieler, Erträge spread, end-turn/give-up dialogs, load dialog (2026-11-03)
- [x] Kontor menus (own layouts): Handel + branch editing (orders, storage, transporters), Schreibstube (applications), Bezahlung, Spielstand (save/load/quit), Hinterzimmer (+ bribery), Kirche, Söldner & Räuber
- [x] Saves in a separate directory with a guard against the install; year-end autosave to slot 5
- [x] Battles: SZENARIO.DAT attached to every game, battle result windows, dialogs for the five fight questions via deterministic replay (2026-11-03)
- [x] §10 rooms: Handel map, Schreibstube, Löhne, Hinterzimmer, Kirche, Söldner & Räuber map, Spieloptionen, Kreditbuch; new sim commands wired (branch rights, local sales, residence, director, loans, join church, give up) (2026-11-03)
- [ ] battle screen; trade map; office tree; remaining `Ask` dialogs; event texts; ICONS3; MIDI

## Verification done (2026-11-02, superseded for films by the §8 section below)
- Start menu (pre-§7 version, icon `31` background): the frame equalled the decoded icon outside the window, byte for byte. The §7 version was checked by eye only: book, banner, frames with correct corners, wrapped text.
- Screens viewed by eye: start menu (with hover), missions, player form with name typed (umlaut `ö` renders), scripted 2-player flow ending at the placeholder.
- Windowed run on macOS: opens, runs 5 s with music and `--sample 2D0`, no errors. I could not listen to the audio output.
- Tests: scaling/mapping, blit clipping/flip, markup layout, CP437, cursor halves; with `MAD_DATA`: EXE strings, OGG + SMPL decode.
- No original screenshot of the start menu exists to compare against, so menu parity is **not** claimed.

## Verification (2026-11-03, after §8)
- Intro: framebuffer after 1000 ticks (10 s at 10 fps = frame 100) equals `flc_dump INTRO.FLC 100` mapped through (x/2, y·200/480 − 10): 0 of 276480 pixels differ; frame 99 differs in 2794.
- Outro (live config, right click in menu, 5 s): equals `flc_dump OUTRO.FLC 70` under the same mapping, 0 differing pixels.
- Player creation: viewed by eye; scripted 2-player run (`--type Anna@0 --click 430,213@1 …`) reaches the placeholder with both names; sex/faith/colour clicks change the selection.
- Intro speech cues: segment logic unit-tested (`film.rs`); playback not listened to.

## Verification (2026-11-03, sim wiring)
- Scripted run (`--start players --type Anna@0 --click 430,213@2 --type Bert@3 --click 430,213@4 --click 300,300@6 …`): reports → Kontor → globe → branch Lichterheide (6 goods, uncle's Fisch order confirmed, 15 qm, 1 Karren) → order toggled → right click → Ja → Bert's turn → both end → A.D. 1601 with production reported. Screenshots viewed.
- "Das letzte Spiel fortsetzen" loads the shipped `GAME5.DAT` (1712, Erzbischof in Mittelland) and shows paged reports.
- Test `real_new_game_year_cycle` (needs `MAD_DATA`): two `EndTurn`s reach 1601 back at the first player.

## Verification (2026-11-03, §9 + menus)
- Screenshots (scripted with `--click`/`--type`, `MAD_SAVE_DIR` in the scratchpad): Nächster Spieler on ICONBB, Kontor with §9.1 banner and gold hover label, Handel list → branch Lichterheide, worker digit stepped 06 → 09 with the "+" pointer and the confirmation dropping to `( )`, "(kein Auftrag)" cycled to a sell order, all seven menus open, save to slot 3 → start-menu load dialog lists "Anno 1600, Anna" → load returns to the Kontor, back-room target list.
- Tests: `form_inline_and_digits`, `real_order_editing`, `real_menus_open_and_pay` (every menu builds; pay step reaches the sim; building a cart costs cash), `real_new_game_year_cycle` (autosave lands in the app's dir), `write_save_guard` (refuses the install's CONFI2).

## Verification (2026-11-03, battles)
- `real_battle_questions_replay`: a human camp with 4×250 men attacks all castles and camps at turn end; the sim asks AcceptRansom several times, each answer re-runs EndTurn from the saved state, the turn completes, battle windows appear. `real_question_dialogs`: all five dialogs build with the right answer items; typed amount reaches `Answer`. Frames written with `MAD_SHOT_DIR` and viewed (questions, result window). OccupyCastle, Ransom, BribeHelper and HelperSide were seen only in the rendered dialogs, not reached in a sim run.

## Verification (2026-11-03, §10 rooms)
- Screenshots of every room with a hovered hotspot (gold label), the branch screen opened from the Handel map city Lichterheide, Spielstand window. `real_rooms`: hotspot counts per room match §10 (10, 7, 6, 5, 4, 7), own city on the map opens the branch, Fürstenbrück opens its info. `real_menus_open_and_pay` covers every popup menu incl. the new ones.
- `audio::tests::real_files_decode` passes with `MAD_DATA` at `disk/MITTELL` (SMP22KHZ) and at `install/MITTELL` (SMP44KHZ only) and names the missing path otherwise.

## Icon identification notes
- `AB`: Kontor room (§9.2 confirms). `BB`: lute (Nächster Spieler, §9.3). `C8`: bookkeeper (likely the Schreibstube). Branch control icons by eye: `51`-`5A` residences by rank, `5B` storage house, `5C` seal ring, `5D` wagon, `5E` ship, `5F` balloon, `50` warehouse.
- Start screens per RE §7: `D4` book, `22` banner, `05-11` frame tiles. My earlier guess (`31` wall) was wrong.
- `00-04`: pointers. The arrow tip is at (0,0).
- `P6`: 6 frames of one gold waving flag (an animation, not six colours).
- `F2-F8`: gold buttons/checkbox bits; `F9-FC`: 1×1 dark-brown dots.

## Requests to other roles
- re-formats: §7 and §8 answered. Still open: player-colour draw call, intro play modes 3/1/7, `$Z` overlap question, hotkey menu layout.
- mad-assets owner: clippy `explicit_counter_loop` warning in `flc.rs:178`. `Flc` borrowing `&'a [u8]` forces mad-app to leak film buffers; an owning constructor would remove that.

## 2026-11-06
- Kontor menus follow the rules-11 sim API: lender (`AskLender`, `Ask::LoanOffer`, loan repay list), residences (`BuildResidence{level}`), directors (`HireDirector`/`Ask::PickDirector`, `FireDirector`, `SetBudget`). Kreditbuch lists loans as text (not screenshot-checked; "Kredit tilgen" is in the lender menu only).
- Battle playback (`screens/game/battle.rs`): `mad-sim` records a frame trace of each fight with a human attacker or defender (`State::replays`, `Battle::record`); the app draws it on `ICONB26..2A` with `ICONBF1` sprites (tables and colour ramps read from the EXE), team colour ramp `5 * player colour`, 25 trace frames/s, click or Esc skips. Checked by screenshot only (`MAD_SHOT_DIR`), not against the original. Not interactive: tactics are still the computer's (ESC mode), no tactic menu, no status bar, no scrolling, no sound. Facing of idle units is the last move/strike direction (0 = N at start, placeholder). Marker/effect sprite mapping (types 12-19) is a guess from formats §11.4.
- Interactive battle: `Ask::Tactics` (one per human participant before every round) runs through the existing replay mechanism; the app plays the frames since the last menu, then shows the tactic menu on the battlefield (formats §11.5: status bar with unit count, type, strength % and tactic, corner frames on the hovered group, left click = next allowed tactic, right click = fight on, ESC = computer takes over). Screenshot-checked only. The opening message box (0xB4219) and, with config 0x17D bit 0x10, the "Seid Ihr zufrieden?" Ja/Nein box (0xBB744) are in. The leader-lost message (0xBB66C) is the sim's `Ask::LeaderLost`, asked once per human participant, shown as a plain question window with a Weiter item (not on the battlefield, untested in a live fight). Not done: hotkey opening mid-round, scrolling, sound.
- Castle/camp menu has the two order lines of rules-07 §3a (type word, target word, troop digits, confirm). Layout is ours (no RE layout for the castle screen 0x55090 yet).
- Gesetze (Schreibstube): read-only law book browser with the EXE's law texts (`0x895CF`, `0xA3DCE`, `0xA3EDB`), the strictness word per book (rules-04b §1.1) and book buttons as text; the original also shows three small icons; ministers edit values in the edit mode (`Menu::LawEdit`, rules-38 §3). Compared with a capture of the original: 6.1 % of pixels differ (icons, hover).

## Platforms (2026-11-18)
GitHub CI has not run since billing stopped its jobs (account payment / spending limit). Checked locally instead: Linux (Docker `rust:latest`, rustc 1.100): full workspace tests with the game data pass, the release build renders the start menu headless. Windows: `cargo check --all-targets --target x86_64-pc-windows-gnu` is clean and the MinGW release build links (`mad-app.exe`, 4.3 MB); not run: Wine 8 under x86 emulation on the Mac exits silently, so a run on real Windows (or CI, once billing is fixed) is still owed. rustc 1.91 is too old (`if let` match guards). Fix found on Linux: the save guard refused every write when the data sat near the filesystem root or the original save folder was missing (an empty path matched every directory).
