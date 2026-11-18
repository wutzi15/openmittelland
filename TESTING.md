# Testing

Run everything: `MAD_DATA=<...>/game/disk/MITTELL MAD_SAVE=<...>/game/install/MITTELL/CONFI2/GAME5.DAT cargo test --workspace`. Without the env vars, the tests that need original files skip.

- **mad-assets**: hand-built fixtures per format; real-file sanity when MAD_DATA is set (all 442 8-bit icons decode, index matches). FLC verified against ffmpeg (PSNR inf on INTRO frame 900).
- **mad-sim**: tests cite spec file and section. Includes original-save round trip (byte-identical write-back, then 12 turns), determinism with respect to answers (the app replays commands), real battles on all five SZENARIO fields, ransom and inflation values.
- **mad-app**: pure logic (scaling, text layout, input), a two-player game through one year, save-directory guard (never writes into the original install). Headless screenshots via `--screenshot`; intro/outro frames compared pixel for pixel with decoded FLC.
- Not verified: menu/Kontor layouts beyond the documented geometry, audio (decode only), anything against a live original run.

- Lint: `cargo clippy --workspace --all-targets -- -D warnings` is clean and runs in CI on Linux.
- App dialogs (2026-11-07): `real_new_question_dialogs` builds every sim question dialog with the expected number of answers (`MAD_SHOT_DIR` writes screenshots), `real_courtship_gift_dialog`, `real_matchmaker_follow_up`, `real_child_naming` (typed name reaches the sim state), `real_mission_success_lines`, `real_exit_lines`, `real_chronicle_table`. `real_debug_fallbacks` (ignored) runs 5 seeds x 40 years through the app and lists report lines still in debug form; 29 seeds ran without a panic.
- CI: `.github/workflows/ci.yml` (tests + release build on Linux/macOS/Windows). Not yet run anywhere; only `cargo test --workspace` and `cargo build --release -p mad-app` were run locally on macOS, plus a headless `--screenshot` smoke run of the release binary.
- Soak: `world::tests::soak_idle_humans` runs two idle humans plus the computer world for years (default 2 seeds x 10 years; `MAD_SOAK=400:60` ran clean on 2026-11-06 in a debug build with overflow checks, with SZENARIO.DAT so human battles run). Covers panics and the save round trip only; human actions are not fuzzed.
- Scenes and dialogs (2026-11-08): `real_event_scenes`, `real_office_scenes`, `real_concert_scene`, `real_report_window` (with the bill page), `real_film_popup`, `real_mission_success_scenes`, `real_new_question_dialogs` (every question dialog builds and has the expected number of answers) all write screenshots to `MAD_SHOT_DIR` for a visual check; `speech_marker`, `real_cue_samples_exist` check the sound plumbing; the battle test asserts that a replayed fight yields effect cues.
