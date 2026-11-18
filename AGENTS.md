# AGENTS.md

- Rust, stable, workspace in `crates/`. `cargo test --workspace` must pass before every commit.
- `mad-assets`: pure parsers for original file formats. No I/O policy, no game logic, no embedded data.
- `mad-sim`: deterministic simulation. No wall-clock, no floats where the original used integers, seeded RNG only. Must not depend on rendering/audio.
- `mad-app`: window, input, rendering, audio, UI. Depends on the other two.
- Original assets are never committed (`assets/` and `*.tar` are gitignored). Point tools at a local install path.
- Clean-room rule (owner approved disassembly of MITTELL.EXE on 2026-11-02, with a two-role split):
  - **RE role** disassembles and writes behavior specs in `docs/re/` (prose, tables, constants, formulas; no pasted decompiler output or assembly). Data-format parsers in `mad-assets` are interoperability work and may be written by the RE role.
  - **Implementer role** builds `mad-sim`/`mad-app` from `docs/spec/` (handbook) and `docs/re/` only, and does not open the disassembly.
  - Ghidra projects live outside the repo; each agent uses its own project directory.
- Every decoder is verified against an independent decoder or a hand-built fixture. Never claim parity without a measured comparison.
- Mark every unverified rule `[UNVERIFIED]` in FEATURES.md; never invent numbers.
