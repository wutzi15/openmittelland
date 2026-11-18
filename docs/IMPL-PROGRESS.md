# mad-sim implementation progress

Implementer role: built only from `docs/spec/` and `docs/re/`. Gaps are marked `// SPEC-GAP:` in code and listed in `docs/IMPL-GAPS.md` (section mad-sim).

## Design
- `State` (`state.rs`) holds the 12 original save blocks as raw records with named accessors, so field widths, sign and wrapping match the docs and `GAMEn.DAT` round-trips byte-identically. The record heap (`heap.rs`) is a `Vec` of records in heap order. Size table: rules-00 §8.1.
- `State::run(&mut Choices)` advances the turn/year state machine until the current human is in the Kontor (phase 4) or the game ends. `State::apply(Command, &mut Choices)` executes Kontor actions; `Command::EndTurn` runs the settlement, then the next turns and the year end. Questions the original asks mid-processing go through `Choices::ask(&Ask) -> u32` (`Auto` always answers 0). Output: `Vec<Event>`.
- Snapshot (`save.rs`): the original 79703-byte image followed by a trailer (RNG state, options, weather, last fate k, church prices, castle table).
- The RNG is seeded explicitly. The original's clock reseed points (program start, every battle) are not reproduced.

## Status
| module | spec | status |
|---|---|---|
| `rng.rs` | rules-01 §1 | done |
| `turn.rs` loop, prison, city growth | rules-01 §3 | done (the autosave is the app's job) |
| `state.rs`, `heap.rs`, `save.rs` | rules-00 §2, §4-§8, §10 | done; the real-save round trip passes on GAME5.DAT |
| `world.rs` | rules-00 §11, rules-02 §9, rules-10 §2 | done |
| `economy.rs` + turn start/settlement in `turn.rs` | rules-02 | done except castle troop pay (undocumented) |
| `career.rs` | rules-03 | done |
| `backroom.rs` | rules-04 | done |
| `laws.rs` | rules-04b §1, §2, §4.1, §4.2 | done |
| `trials.rs` | rules-04b §3 | done |
| `privileges.rs` | rules-04b §4.3, rules-05 §7.1 | done (mistress, feast, trial and insult have their own `Command`s) |
| `castles.rs` | rules-04b §4b, rules-07 §2-§3 | sabotage, computer castles, recruiting, manoeuvre, toll rate, settlement_pre done; `battle.rs` / `fight.rs`: the tactical battle and its setup/outcome (§5) done. Needs `State::szenario` = SZENARIO.DAT bytes, else `Event::BattlePending` |
| `siege.rs` | rules-07 §6 | done (siege, plunder, tribute, relief) |
| `debt.rs` | rules-02 §10 | done (seizure stages, jury court) |
| `family.rs` | rules-05 | done |
| `events.rs` | rules-06 §2-§4 | done (disasters, divine events, world fate, 100 fate events); `subplots.rs`: 17 sub-plots and the newspaper contests done |
| `auction.rs`, `social.rs` | rules-08 | done |
| `ai.rs` | rules-09 §2-§4 | npc_act and revenge done (all 10 cases) |
| `director.rs` | rules-09 §5 | done except the flag-1 wood/stone swap |
| `missions.rs` | rules-10 | done |
| `kontor.rs` | rules-11 | done: branch rights and sale, start cities, residences (buy/build/sell/extend/renovate), local sale, money lender and loan book, Kircheneintritt, giving up, director hire/fire/budget |

Tests: `cargo test -p mad-sim` (73 tests; test names cite spec file and section). `MAD_SAVE=<GAMEn.DAT> cargo test -p mad-sim real_save` checks the round trip of an original save and then simulates 12 turns from it.

## Next steps, in order
1. (done 2026-11-06) human castle/camp order book and troop pay, `castles.rs` `troop_*`.
