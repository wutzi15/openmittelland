# Church: Hochzeitsglocken click (rules-62)

Source: church screen `0x5CCA0(p)` (items `0x5CD38..0x5CE28`, dispatch `0x5CE70..0x5CEA4`, case 4 `0x5CED3..0x5CF49`, loop tail `0x5D15F..0x5D1D0`), `menu` `0x35B60` (hit test `0x35C10..0x35CAA`, hotspot → label `0x35D00..0x35D26`, click acceptance `0x365A0..0x36672`), matchmaker `0x20D30` (entry `0x20D41..0x20F33`). Save fields read from `CONFI2/GAME5.orig` with the layout of `formats.md` §3.2. Tags: **[C]** instruction level, **[L]** decompiler or partial trace, **[UNVERIFIED]** inferred. This extends `formats.md` §10.6.

## 1. Dispatch chain [C]

1. Items: banner base B (`0x49D60`), then label 4 `0x404F0` "Hochzeitsglocken" (318, 108) and hotspot 5 `0x12E0` (250, 130, 145, 111). The pair is created unconditionally; no state hides it (`0x5CD38..0x5CD71`).
2. `menu(B, 0, &v)`: the hit test takes the first item from B whose rectangle holds the pointer and whose flag byte `+9` has bit 0x04 clear (`0x35C9B`). A hit on a kind-0xE0 item (style `& 0x1F0`) walks back to the previous non-0xE0 item (`0x35D00..0x35D26`), so hotspot 5 resolves to label 4, which is highlighted (`+9 |= 0x40`) and `0x84D50 := 0xA0`. The hover highlight the capture shows is this step.
3. Click acceptance, the only input-side gates: button state from INT 33h/3 each poll; a press is dropped only if (a) `0x84D50 < 0xA0` (never after a resolve) or (b) the same item is still hovered, `[EBP−0x50] ≠ 0` and `clock < [EBP−0x50]` (the auto-repeat hold-off; `[EBP−0x50]` is zeroed by any poll with the button up on that item or by a hover change). An accepted left press returns the label index at once (sound 0x42); right or both buttons return count + 1.
4. `i = result − B = 4` → jump table `0x5CE44[4] = 0x5CED3`:
   - player `+4` (spouse) ≠ 0xFF → message box `0x368E0(120, 180, 400, 120)` `0xC0034` / `0xC0074` (by gender byte). Visible.
   - else `0x49EB0` (drops the banner items) and `0x20D30(p)`, then redraw flag.
5. Loop tail: close only if i = 3, i ≥ 12, `0xEF4C5` ≠ 4, the options result ≠ 0, or the leave flag. Otherwise rebuild the screen (redraw flag) or poll again.

## 2. Gates inside `0x20D30` before anything is drawn [C]

Only one: `rec_find(30, p, 0xFF)` (`0x20D50`). A courtship record → message box `(160, 100, 320, 280)` `0x9D96C` (visible). Otherwise `win_open(0, 0, 640, 480, 0x8500)`, `draw_icon(0, 0, 0xBD)` (ICONS2B/ICONBD, present in the install), `music(0x11, 1)`, `flip`. After the picture:
- `0x85E07 ≥ 2` → caption menu `0xBA95C` + `wait_click`;
- candidate list empty → caption `(0, 390, 640, 90, 0)` `0x9DB01`;
- else the offer (`rules-15` §7.2, `rules-61`).
Candidate filter (`0x20E58..0x20EF9`), i = 0..191: player byte 0 ≠ 0xFF; `gender(i) ≠ gender(p)`; **`+0x11 ≠ 15` (i holds an office)**; `(+0x0F < 10) = (p+0x0F < 10)` (both city-level or both county-level); `+0x10 = 0` (computer, no church office); `+4 = 0xFF`. No age, cash, year, turn-flag or confession test anywhere on the path.

**There is no silent return.** Every path from an accepted click draws a window or a box and waits for a click.

## 3. The reference save [C for the values]

`GAME5.orig`: year 1600, current person 160 "anna", gender 0 (male), confession 0, spouse 0xFF, location 0, `+0x10 = 0x10` (human, no church office), `+0x11 = 15`, cash 750, no type-30 record, `0x85E07 = 0`, turn flags 0, phase byte in the file 0 (set at run time). The filter of §2 yields **13 candidates** (persons 5, 10, 23, 35, 36, 40, 65, 70, 76, 83, 86, 91, …), so the code path for this save is: Kupplerin picture, `rand(13)`, offer caption `0x9D9F9`, price menu `0x9DB78`.

## 4. What the capture contradicts

The code has no state in which the label lights but a click on the same spot does nothing. The remaining explanations are outside this screen's logic [UNVERIFIED, in order of plausibility]:
1. The click was accepted and `0x20D30` ran, but its window was not presented, so the church picture stayed and the next clicks were eaten by the matchmaker's own waits (offer caption, price menu, then redraw of the same church). Test: after one click, press the right button or count clicks until the screen rebuilds; or check whether the hover highlight still follows the pointer (it does not while `0x20D30` waits).
2. The harness delivers button state that INT 33h/3 does not report at poll time (press and release both between two polls). The 0.4 s hold rules this out unless polls stop.
3. The press arrived while `[EBP−0x50]` was armed (within `0x84D58` hundredths of menu entry or of a hover change without an intervening button-up poll). Only a press that starts before the pointer settles can hit this; a second click later cannot.

Answer: the item is always created; the dispatch is hotspot 5 → label 4 → case `0x5CED3` → spouse test → `0x20D30`; the only pre-draw gate inside is the type-30 record. No path returns silently. The save passes every gate and has 13 candidates, so the original must show the Kupplerin and an offer; what the capture saw is not reproducible from the code and points at presentation or input delivery, not at game state.
