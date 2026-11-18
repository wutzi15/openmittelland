# RE rules progress, agent B (family, events, missions/scoring)

Ghidra project copy: `<scratch>`. Helpers in `<scratch>` (run with `<scratch>`):
- `d/<addr>.c` full decompile of all 446 functions (names applied).
- `dcr.py addr...` raw decompile `r_<addr>.txt` + listing `a_<addr>.txt`.
- `ann.py addr [width]` (plain python3) resolves string and pointer-to-string operands in `a_<addr>.txt`.

| doc | status |
|---|---|
| rules-05-family.md | done |
| rules-06-events.md | done (disasters, divine, world fate, 100 fate events, 17 sub-plots, contests, chronicle); spinner limits open |
| rules-10-missions-scoring.md | done |

Corrections for other docs: `rules-01` §3 difficulty order is wrong (0 Leicht, 1 Mittel, 2 Schwer, 3 none).

## Notes (now written up, kept for reference)
- 0x64108 is not a function start: it is the phase-6 tail of turn_end_settlement. It begins with the sub-plot launcher: option Abenteuer and `year > 1605 + rand(6)`: k = rand(100) (debug override `0xEF4A0 >= 500` gives k = value-500); k 0..16 call start functions 0x19C30, 0x1A000, 0x1A680, 0x1ACC0, 0x1B400, 0x1B8D0, 0x1C040, 0x1C350, 0x1C820, 0x1CB90, 0x1CF60, 0x1D4E0, 0x1D840, 0x1DD70, 0x1E580, 0x1E910, 0x1EC50. Then rival reaction 25 % (0x5B140 code 3), pending duels (type 22), crime scan 0x447D0/0x44E00, trials (type 18), debt seizure (cash < 0 and < -NW/20), then 0x654A9.
- 0x226F0 ("adventures_main") is the Schicksal (fate) event roller: option bit 0x004, once per human turn end plus a second time with rand(100) < 50.
- Turn-end record handlers for sub-plot types 32..49 at 0x61A70 dispatch: 0x20→0x19DC0, 0x21→0x1A240, 0x22→0x1A770, 0x24→0x1AEA0, 0x25→0x1B660, 0x26→0x1BC00, 0x27→0x1C270, 0x28→0x1C4C0, 0x29→0x1C990, 0x2A→0x1CCE0, 0x2B→0x1D000, 0x2C→0x1D6A0, 0x2D→0x1D9A0, 0x2E→0x1DE90, 0x2F→0x1E7F0, 0x30→0x1EA80, 0x31→0x1ED90 (0x23 none).
- Year end: record types 2 and 4 owned by 0xC0 (world) handled by 0x19360 / 0x19890 (contests); contest start 0x19480 if Abenteuer and `rand(5) + 1605 < year` and rand(100) < 4. Schicksal option: 15 % 0x26640 (siege price effects?). Auction announcement 0x269D0 20 % when counter 0x85E06 = 0.
- npc_misfortunes 0x59940 called at year end when `rand(50) < 0x85CBF` (then 0x85CBF := 0) and an Erzbischof (computer) exists; also from npc_actions_a and privileges menu.
