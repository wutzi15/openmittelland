# Löhne (wage) screen input (rules-54)

Source: `0x57E70(p)` (`0x57E70..0x58402`), `menu` `0x35B60` (numeric-field branch `0x35D60..0x36257`), `item_range(item, min, max)` `0x34B00`, `draw_icon(x, y, id)` `0x2E310`, `blit` `0x13539`. `P` = person `0xE3A7C + 0xF8·p`. Tags as in `rules-50`. Everything is [C] unless marked. This replaces `formats.md` §10.3 "Exact row formulas: [UNVERIFIED]".

## 1. Layout

- `win_open(0, 0, 640, 480, 0x8500)`, picture `ICONA9` at (0, 0), `music(0x19, 2)` (a random group, so it draws `rand`, `rules-46`), banner "Löhne" (`0x49D60`, items 0 gold, 1 banner, 2/3 placeholders). The screen's own items start at base + 4.
- Rows k = 0..3: Direktoren, Arbeiter, Söldner, Räuber. The person byte is **`P[+0xEF + k]`**, a wage level in % (`rules-02`: +0xEF = director pay level).
  - Label: margin x = 100 + 100k, width 140, cursor y 90 (row 0: cursor x − 20), text `0xB6FA0[k]` = `$CDirektoren$N` / `$CArbeiter$N` / `$CSöldner$N` / `$CRäuber$N` (ids 2898..2901).
  - Value: cursor y 408, text 2902 `0xA9D89` `$C%m%%$N` with (style **3**, `&P[+0xEF+k]`). This is a 3-digit **u8** number item (kind 0x00), so it reads e.g. "100%". These are items base + 4 + k. Then `item_range(base+4+k, 50, 150)`.
- Coin stacks: a 48×300 backup of the picture at (146 + 100k, 100) per row. `coins = clamp((level − 50)·50 / 100, 1, 50)`, i.e. (level − 50)/2. Coin `ICONAA` at **x = 146 + 100k, y = 386 − 5i** for i = 0..coins−1. 100 % gives 25 coins (top coin at y 266), 150 % gives 50 (y 141), 50 % gives 1. When the count grows, only the new coins are drawn. When it shrinks, the strip is restored and the whole stack is redrawn. After the first pass, each changed strip is copied to the screen (`0x5822E..0x5827C`).
- The coin stacks are pictures, not items. Clicking them does nothing, which matches the live test.

## 2. Changing a value

The value item is a digit spinner handled inside `menu` (kind 0x00, u8):
- The column under the mouse x selects the step `0x84DA4[col]`: 1, 10, 100 for the units, tens and hundreds digit [L for the column formula `0x35E73..0x35F27`].
- Mouse y in the upper half of the item (`y < top + h/2 − 2`) means `+step`, clamped so that value + step ≤ max (150). The lower half means `−step`, clamped to ≥ min (50). The delta goes out through `menu`'s third argument (`0x36159`). The mouse pointer changes to the up/down shapes (pointer ids 2/3).
- The wage loop (`0x58333..0x58385`) does `v = P[+0xEF+k] + delta` (byte). If v is odd, it moves one more in the same direction (+1 if delta > 0, else −1), then stores v. Effective steps are therefore **±2** on the units digit, ±10 on tens and ±100 on hundreds (clamped to 50..150). Values stay even.
- No key handling of its own. Holding the button repeats through `menu`'s auto-repeat (`0x84D58` from 17, −4 per repeat) [L].
- Sound: only `menu`'s click samples (0x42 left, 0x4C right, `rules-14`) [L].

## 3. Leaving

`r = menu − base`:
- 0 (gold): `0x4F370(p)`, then stay.
- 1 (banner): options `0x4D1E0`; leave if it returns ≠ 0.
- 2 or 3: phase := 4.
- 4..7: change a row (§2).
- ≥ 8 (a right click returns count + 1 = 8): phase := 4 (back to the Kontor).

The loop exits on r = 3, r ≥ 8, phase ≠ 3, or a non-zero options result. On exit it frees the four strips, cleans up the banner and closes the window. It returns the options result.

Answer: wages change only by clicking the "NNN%" number at y 408 under each stack. The upper half of a digit adds and the lower half subtracts. The units digit steps by 2 (the code forces even values), tens by 10 and hundreds by 100, within 50..150 % per row. The value is stored in `P[+0xEF..+0xF2]`. The stack shows (level − 50)/2 coins (at least 1), 5 px apart, upward from y 386 at x 146 + 100k. A right click leaves the screen.

## 4. `menu` auto-repeat (follow-up)

All references are to `menu` `0x35B60`. Clock = `clock()` `0x6AC50` in 1/100 s (`rules-26`). Its real resolution under DOS is the BIOS tick (~55 ms) [UNVERIFIED: Watcom runtime].

1. **Entry (`0x35BA4..0x35BFF`).**
   - `deadline = clock() + R`, where R is the global `0x84D58` (image value 17).
   - Only if the requested pointer shape `0x84D68` is 0: set it to 1 and **wait until all buttons are released**. Pointer 0 is set by `win_open` `0x2FAB1`, `win_close` `0x301B7`, `0x30317`, the line painter `0x31739` (any `print` that emits a line) and `0x367D8`.
   - Inside `menu`, the pointer is set to 1 (plain item, `0x36259`), 2/3 (digit up/down, `0x36082`/`0x360EC`) or 4/5 (±1 kinds, `0x361D0`/`0x3620F`). So a caller that redraws only icons or blits and prints nothing re-enters **without** the release wait.
2. **Poll loop.** `state = mouse()` (`0x12DE4`: 0 none, 1 left, 2 right). The item under the cursor and, for value kinds, the delta `*out` are recomputed every poll. Then (`0x365A0..0x36627`):
   - the item differs from the last one (`0x84D4C`) → remember it, timer off (`[EBP−0x50] := 0`);
   - else `*out` differs from the last delta (`0x84D54`) → remember it, timer off;
   - else no button → timer off;
   - else (same item, same delta, button held) if the timer is on and `clock() < deadline` → treat as no click and keep polling.
   
   A press with the timer off is accepted at once.
3. **Step on press, not on release.** An accepted press returns the item immediately (`0x366C5`, with sample 0x42 for left and 0x4C for right). A release never returns anything.
4. **Acceleration (`0x36672..0x3668A`).** On return, if the timer was on (a held repeat), `R -= 4` while `R > 1`. Otherwise (a fresh press) `R := 17`. Each caller loop re-enters `menu`, so a held button produces steps at:

| step | delay since previous (1/100 s) | cumulative (ms) |
|---|---|---|
| 1 | 0 (on press) | 0 |
| 2 | 17 | 170 |
| 3 | 13 | 300 |
| 4 | 9 | 390 |
| 5 | 5 | 440 |
| 6+ | 1 each | +10 each |

   With BIOS-tick resolution, each delay rounds up to whole ~55 ms ticks [UNVERIFIED]. A 100–200 ms press gives 1–2 steps by this table. The live 3 steps need the button held for about 0.3 s, or a faster emulated clock [UNVERIFIED].
5. **Which items repeat.** The repeat is generic: any item kind repeats while it stays under the held button and its delta stays the same, provided the caller re-enters `menu` without printing text or opening/closing a window (otherwise the release wait of step 1 applies).
   - Digit spinners (item kinds 0x00 u8, 0x10/0x50 u16, 0x20/0x60/0xC0/0xD0 u32; `0x35D60..0x35E63`) use a 10^column step.
   - Kind 0xA0 (cycling word toggle, `0x36169`) uses ±1 (upper/lower half).
   - Plain bullets and lines return their index.
   
   On the wage screen the loop only blits coins, so held clicks repeat. Bullet menus whose callers then print or close a window do not repeat in practice [L: depends on each caller].

Answer: the first step happens on press. While the same item is held, it steps again after 17, 13, 9 and 5 hundredths of a second, then every 1/100 s (`0x84D58`, minus 4 per repeat, reset to 17 by a fresh press). This applies to every item kind whenever the caller re-enters `menu` without redrawing text.
