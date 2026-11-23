# Open questions, round 9 (rules-71)

Answers to `docs/IMPL-GAPS.md` §1 "For the RE role" (state of 2026-11-22) and the two "Needs a live check" items that the code settles without a capture. Conventions as in `rules-66`: `rand(n)` = `0x36FF0` (it calls the generator and takes `mod n`, so `rand(1)` is a draw too), `draw_icon(x, y, id)` = `0x2E310`, `strip_copy` = `0x13539`, `sample(id, 3)` = `0x12698`, `print` = `0x322C0`, `menu` = `0x35B60`, `items_free` = `0x34AC0` (pops items with `0x34930`), `clear_page` = `0x30F10`. Person record `0xE3A7C + 248·i`; battle man record `0xD7DCC + 33·i` with the fields of `rules-07` §5.9 / `rules-43`; unit record `0xD7534 + 22·u` (slot bytes +0x04..+0x0D, tactic +0x0E). Tags: **[C]** instruction level, **[L]** decompiler only, **[G]** guess.

## 1. Jury markers of the creditors' court (`0x64AF9` in `turn_end_settlement`) [C]

The marker table `0x87310` is a table of **words**, read as `word[0x87310 + 2·state]` (`0x650B9..0x650C9`): `[0x30, 0xA7, 0xB3]`. The earlier reading `[0x48, 0xA7, 0x179]` in `rules-18` §12 and `rules-15` §6.1 was wrong (corrected there).

Per juror i = 0..11 after every vote (`0x65087..0x6513E`), at x = 62 + 44·i, y = 420:
- juror state 2 (not yet voted) → `draw_icon(x, 420, 0xB3)`: ICONB3, the framed Justice figure (32×32);
- state 0 (guilty) → `draw_icon(x, 420, 0x30)`: ICON30, the framed prison cell with a figure behind bars (32×32, the same icon the `%j` format code uses, `formats.md` §11.2);
- state 1 (not guilty) → no icon: the 32×32 cell at (x, 420) is copied back from the strip saved before the first vote (`strip_copy(saved, x, 420, screen, x, 420, 32, 32)`), so the Justice figure disappears and the backdrop shows. Table entry 1 (0xA7, a framed landscape) is never drawn; the state-1 branch skips the table.

State 0/1 is set per vote by `rand(100) < threshold` (guilty, counter +1, speech slot `s`) / else (not guilty, speech slot `s + 4`), as in `rules-15` §6.1 step 4. 7 or more guilty → tower (`0x65176`, `counter > 6`).

The live check of 2026-11-22 named the guilty marker "ICONB2". The code draws 0x30; on the rendered sheets ICONB2 is a framed barrel and ICON30 the barred cell, so the "prison door" seen live is ICON30. If ours draws 0xB2, switch it to 0x30 and recheck one row.

**Trial scene** (`trial_main` 0x289E0): no juror markers. Its only `draw_icon` calls are the hall 0xC1 (`0x28D8A`), the court room 0xC6 (`0x28F12`) and the rebuild pictures 0xB8/0xB7 (`0x2C00F`/`0x2C02D`); the table 0x87310 is referenced only from `0x650C9`. The seats' verdicts are the speech captions of `rules-18` §9.

## 2. Dismissal picture: the whole byte +0x10 is passed [C]

The dismissal call (`0x666A3..0x666E4`) is `0x66160(EAX = byte +0x0F of p, EDX = byte +0x10 of p, EBX = byte +0x03 of p)`: office location, the **whole** flag byte +0x10 (low nibble church office, high nibble controller, `rules-00-map` §4), religion. `0x66160` stores DL and tests `DL ≠ 0` for "church" (`0x66175`); it never masks the nibble. So:

| dismissed p | +0x10 | picture |
|---|---|---|
| human, any office | ≥ 0x10 (controller nibble 1) | church variant: city **0xC0** (portal), county **0xC2**/**0xC3** by religion byte +0x03 (0 → 0xC2 small church, 1 → 0xC3 large church), principality **0xBE** (altar) |
| computer, secular office | 0 | secular variant: city **0xC1** (hall stairs), county **0x193 + location** (0x19D..0x19F castle landscapes), principality **0xBF** (niche) |
| computer, church office | low nibble ≠ 0 | church variant as for a human |

A human Baumeister therefore gets 0xC0, as the capture showed. Religion matters only for county offices (10..12). The other three callers (election header `0x6767B`, the two no-election messages `0x6839D`/`0x684BD`) pass a computed church flag, not the byte, so they are unaffected (`rules-15` §3.1 stands for them).

## 3. Battle hit sound: same frame as the damage roll, in the object scan [C]

Within one frame of a unit's activation (`rules-44` §2 order): (1) the step logic of the unit's men in slot order, (2) the object scan row by row, (3) display only: status strip and tick wait, (4) key poll.

The strike resolution (`rules-07` §5.9 step 1, `0x39680..0x3975E`) runs in phase 1: `rand(100)` hit roll, `rand(HP + 1)` damage roll; with damage > 0 the victim's hit flag +0x20 := 1 and `0xDC840 := 10`; then HP, state (0 idle with the idle frame count, or 4 with the fall frame count), frame index 0.

The sound is in phase 2 of the **same frame**, in the man branch of the object scan, when the scan reaches the victim's record (`0x3B74E..0x3B82C`): if +0x20 ≠ 0: +0x20 := 0, frame index −1, frame counter +1; if displayed (`0xDC83F == 0`): `draw_icon(pixel x + xoff[type][state], pixel y + 6, 0xF8)` (the hit splash), then `sample(0x39 + rand(9), 3)`. In ESC mode neither the icon nor the `rand(9)` happens. The branch is reached for every man record the scan visits, whatever its counter.

So the `rand(9)` sits in the one battle RNG stream, after **all** step-logic draws of that frame (the hit/damage rolls of every man of the active unit whose animation ended in that frame, in slot order) and at the victim's position in the scan among the other scan-phase presentation draws (walk/strike/parry/fall sounds of records scanned earlier in the frame come first; the fall sound at `0x3B6F4` is `0x58 + rand(1)`, a draw of its own when displayed). Nothing is deferred to the frame in which the victim's counter runs out.

For ours: playing it at the damage frame is the right frame. The draw order only matters for a port that reproduces displayed battles.

## 4. Prisoners [C]

**Capture attempt** (unit tactic 7, `0x397A5..0x39828`), when the captor's strike animation ends and the target t is neither in state 4 nor of type ≥ 12:
- `rand(5·condition_t + 1) < condition_captor` → captured: `0xDB804[captor +0x10] += 1`; **t.+0x1E := captor's man index**; t.state +0x12 := 4; frame index +0x14 := 0; frame counter +0x15 := fall frames `word[0x855D0 + 22·type_t]`. Nothing else of t changes here: HP, participant +0x10, side +0x11, type, cell, unit slot all stay.
- otherwise t.state := 0, frame 0, counter := idle frames `word[0x855C8 + 22·type_t]` (the parry is interrupted).
- either way the captor's moves +0x19 := 0.

**End of the fall** (`rules-43` §1, `0x3D084..0x3D15B`): with HP > 0 the record is converted in place: +0x10 := captor's +0x10 (colour/participant), +0x11 ^= 1, +0x1E := old type, type +0x0E := 12, state 0, facing 0, frame 0, counter 6, pixel x += row mod 4; sample 0x4C when displayed. The record is never moved and his old unit's slot byte keeps his index.

**His old unit afterwards.** At every activation of that unit the reset loop (`0x3907A..0x390F6`) runs over all slots ≠ 0xFF: moves +0x19 := `0x857AC[type]`, which is **0** for type 12 (the table has 12 entries followed by zero bytes), counter := 0, **+0x1E := 0xFF**. In the per-man loop (`0x39461..`) a slot with counter 0 and moves 0 only gets "state := 0 if it was not" (`0x3AE7B`), so a prisoner never acts, and in the tactic dispatch types > 10 are skipped anyway (`0x398A8`). Consequences: (a) `rules-43`'s "+0x1E = old type" survives only until his old unit's next activation, then it reads 0xFF like a banner (nothing reads it either way); (b) the unit's "has men left" test (`0x38AB5..0x38B5B`) sums the HP of its slots, and a prisoner keeps his HP, so a unit whose men are all prisoners is still activated (and does nothing); (c) the end-of-battle counts (`rules-07` §5.11, type < 12 and not fallen) no longer count him for either side; the only durable record of the capture is `0xDB804`.

**Strike on a falling or fallen man.** Step 1 tests t first (`0x39660..0x39678`): `t.state == 4` or `t.type ≥ 12` → the striker goes idle with moves 0 and **no roll at all** (no hit roll, no damage roll, no capture roll). This covers a target that was captured or killed by another man of the same unit earlier in the activation, and a converted prisoner. The adjacent-enemy tests of the tactic fallbacks (`0x39D6C`, `0x39DC1`) exclude state 4 and type ≥ 12 as well, so nobody starts a strike at one. Ours ("participant kept, strikes ignored") is right on the strike; the participant and side do change at the end of the fall.

## 5. Tactics hotkeys of two humans (`0x3D5F7..0x3D794`) [C]

Per frame of an activation, after the draw and tick wait (ESC mode: right after the scan), **one** key is taken from the BIOS buffer (`0x6B081` test, `0x6AC77` read; a 0 code is read a second time for the extended key):
1. key 27 → ESC mode (`0xDC83F := 1`); from then on the compare below is skipped for this and every later frame (`0x3D627`).
2. The key is compared with the attacker's hotkey (person `0xDB7F4`, byte +0xF5): equal and `q & 5 == 0` → `q := (q << 2) | 1`.
3. Then, independently of 2, with the defender's hotkey (`0xDB7F5`, +0xF5): equal and `q & 0xA == 0` → `q := (q << 2) | 2`.

`q` is a local byte cleared at activation entry (`0x389D5`). Encoding: bit 1/2 = the most recent side queued (attacker/defender), bits 4/8 = the side queued before it. A repeat press of an already-queued side is dropped (the `& 5` / `& 0xA` tests). If both humans use the same key, one press queues both, attacker first.

When the frame loop ends (no object animated, `0x3D68C`) and `q ≠ 0`: the screen is saved, then `while q ≠ 0: if q & 4: menu(side 0); if q & 8: menu(side 1); q := (q << 2) & 0xC`. Traced: attacker then defender → `q = 6` → side 0, then `q = 8` → side 1. Defender then attacker → `q = 9` → side 1, then `q = 4` → side 0. One side only → `q = 1` or `2` → `q = 4`/`8` → that menu once. So within one activation each side gets at most one menu, both menus open back to back after the activation in press order, and `0xDC83A := 1` (strip redraw) follows. Two presses in the same frame: the second key stays in the BIOS buffer and is read at the next frame's poll; if that frame was the activation's last, it is read by the first poll of the **next** activation and queued there (the buffer is not drained between activations [L]). A hotkey pressed after ESC in the same activation is ignored; a hotkey queued before ESC still opens its menu.

The replay model of ours (one `Ask::Tactics` per activation) needs two answers in the rare both-humans case; the original's observable order is attacker/defender by press order. No RNG.

## 6. Pre-fight bribe spinner adds the step (`0x3FE2E..0x3FEBA`) [C]

`offer` (`[EBP−0x58]`) is preset 0 and its address is the `%m` argument of text 2881, so the field displays `offer`. `menu` never writes a spinner's variable: for a numeric item it computes the digit step `±10^k` from the click position, clamps it so that `value + step` stays in the item's range, stores the clamped step in its out-parameter (`0x35B60`, `*EBX := step`) and returns the item. The caller then does `if r == 1: offer += step` and loops while `r == 1`. So every click on the spinner adds (or subtracts) one digit step to the running amount, like every other spinner (`rules-13` §1.3); nothing is "committed" twice. Item 0 ends the loop with `offer` as shown; item 2 or a right click sets `offer := 0`. `rules-52` §1's [UNVERIFIED] "adds" is confirmed; ours (final field value) is equivalent.

## 7. Donation window: the question stays above the price list [C for the flow, L for the pixels]

`0x5F6C2..0x5F823`: `win(120, 120, 400, 240, 0xA810)`; `print(T[2525])` with its `$Z` tail; the city loop; `items_free(base)`; then straight into the five `print(T[2526])` lines and `print(T[2527])`, `flip`, `choose`. There is no `clear_page`, no cursor move and no window close between the city choice and the price list, so the prices are printed at the text cursor left by T[2525], under `[EXE text]`. Popping an item (`0x34930`) repaints its background only for items with style bit 0x80000; `%a` bullets are created with style 0x1090 and the `%m` city item with the caller's 0xA5, neither has it, so the bullet and the last shown city name are left on the window surface as painted [L: the repaint rule read in the decompiler]. The window scrolls if the list overflows it (style 0xA810).

## Summary

| item | answer | section |
|---|---|---|
| jury markers | word table `0x87310` = [0x30, 0xA7, 0xB3]: not voted ICONB3, guilty ICON30 (barred cell), not guilty = strip restored; trial scene has none | §1 |
| dismissal picture | whole byte +0x10 passed: humans always get the church variant, computers only for church offices | §2 |
| hit sound | same frame as the damage roll, in the object scan at the victim's record; `rand(9)` after that frame's step-logic draws, displayed only; icon 0xF8 drawn with it | §3 |
| prisoners | at capture only +0x1E (captor index), state 4, frames change; at the end of the fall +0x10/+0x11/type 12; slot kept; his unit's next activation sets +0x1E := 0xFF and moves 0; strikes on state 4 / type ≥ 12 skipped with no roll | §4 |
| two humans' hotkeys | one key per frame, one queue slot per side, both menus after the activation in press order | §5 |
| bribe spinner | `offer += clamped digit step` per click; menu never writes the variable | §6 |
| donation window | question text, bullet and city name stay; prices continue below | §7 |

## Corrections to other docs

- `rules-18` §12, `rules-15` §6.1 step 4 and the icon table rows 0x48 / 0x179: the markers are 0xB3 and 0x30 (§1).
- `rules-07` §5.9 step 1: a target in state 4 or of type ≥ 12 is skipped with no roll; the capture sets +0x1E, state 4 and the fall frames only (§4).
- `rules-43` §1: +0x1E = old type only until the prisoner's old unit is next activated (§4).
- `rules-14` §2.3: the fall sound is `0x58 + rand(1)` (one draw when displayed); the hit sound comes with icon 0xF8 in the scan of the same frame (§3).
- `rules-44` §2: a shared hotkey queues both sides from one press; one key per frame (§5).
- `rules-52` §1: "adds" confirmed (§6). `rules-38` §2.3: the printed text stays (§7).
