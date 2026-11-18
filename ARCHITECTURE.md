# Architecture

Decisions
1. **Rust** (owner preference). 2D indexed-color game, so CPU framebuffer, no engine.
2. **Crates**: `mad-assets` (formats) / `mad-sim` (rules, deterministic) / `mad-app` (shell). Asset import stays outside the sim.
3. **Rendering**: software 8-bit indexed framebuffer at the original resolution (SVGA via VESA, 640x480; `ICONS2*` is 8-bit, `ICONS3*` is RGB555 hicolor, both 640x480), palette-mapped to RGB, shown via `winit` + `softbuffer`. Implemented.
4. **Audio plan**: `rodio` for WAV/OGG (implemented; MIDI not) (`SPC*`/`SMP*` voice+samples, soundtrack), MIDI (`MIDSONGS/SONG*.MID`) via a software synth. MIDI is not implemented (the OGG soundtrack plays instead). Sound effects are not mapped yet.
5. **Simulation**: turn-based hotseat game. Sim is a pure state machine: `State + Command -> State + Events`, seeded RNG, serializable, so replays and tests are trivial. Implemented (`mad-sim`, ~78 tests); the app drives it through `Command`s and answers its synchronous questions by re-running a command with the answers so far (`Scripted` in `screens/game.rs`).
6. **Original saves** (`CONFI2/GAME5.DAT`, `CONFIG.DAT`) are to be parsed in `mad-assets`; the format is unanalysed.

Reference data layout (from the DOS install)
- `MITTELL.EXE`: 32-bit LE (DOS4GW). `SZENARIO.DAT` 6900 B, `CONFIG/CONFIG.DAT`.
- `FLCFILME/*.FLC`: 13 Autodesk FLC cutscenes, 320x180/320x200. Decoded (see TESTING.md).
- `ICONS2`, `ICONS2B`, `ICONS3`, `ICONS3B`: `ICONxx.DAT` sprite/bitmap files, several sub-formats (headers seen: `w16 h16` raw, `fe`-RLE, `00 xx` ...). Unanalysed.
- `SPC22KHZ`/`SPC44KHZ`: ~1400 numbered WAV (speech). `SMP22KHZ`/`SMP44KHZ`: `SMPL_*.DAT` samples (header + unsigned 8-bit PCM-like). `MIDSONGS`: 24 SMF files.

Current structure (2026-11)
- `mad-app/src/screens/game.rs` is the in-game screen: `Game` holds the sim `State`, the popup queue (`Popup`), and turns sim `Event`s into report text (`describe`), questions (`Ask`) into dialogs (`menus.rs::question`), and sub-plot/fate/election texts from EXE strings. `Game::pref` implements the EXE person codes, `wform`/`transporters` the noun codes (docs/re/rules-12).
- Questions: the sim asks synchronously (`Choices::ask`). `Scripted` unwinds out of the sim when a question has no answer yet, the dialog is shown, and the command is re-run with the answers collected so far (exact because the sim is deterministic). A dialog may answer two questions at once (`Action::AnswerPair`).
- Text: EXE strings are read with `Assets::text(addr)` or `Assets::text_id(n)` (pointer table 0xB4258) and filled by `subst`/`subst_seq`; report lines are flattened with `plain`.
- Specs: `docs/re/` is written by the RE role (rules-00..15, formats, strings); the implementer role builds from it and from `docs/spec` (handbook) only. Every guess is listed in `docs/IMPL-GAPS.md`.
