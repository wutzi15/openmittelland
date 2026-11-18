# Kontor "Spielstand" window (rules-28)

Source: `0x5DC60(viewer)`, called once from the Kontor screen (`0x5D9E1`) with the Kontor's player as `viewer`. Helpers: `human_rival_of` = `0x44F10` (`rules-03` §4.1), `net_worth` = `0x44640` (`rules-02` §8), `clamp0(x)` = `0x441F0` = `max(0, x)`, `tax_tithe(p, &tax, &tithe)` = `0x618C0` (`rules-02` §4.5), `win` = `0x2FA80`, `print` = `0x322C0`, `menu` = `0x35B60`, `msg_box(x, y, w, h, fmt, …)` = `0x368E0`. Tags: **[C]** read at instruction level, **[O]** observed live in the original (new game 1600, one human "anna", difficulty 3), **[UNVERIFIED]** not confirmed. This corrects `formats.md` §10.4 (height is not fixed at 240; rows are not "one per player").

## 1. Rows: who and in which order [C]

1. Scan person slots 0..191 in ascending order. A slot qualifies if byte +0x00 ≠ 0xFF (in use) and the controller nibble (+0x10 & 0xF0) ≠ 0 (human). Stop after **12** matches. Let `n` = number of humans found.
2. Append, for each of those humans in the same order, `human_rival_of(h)`. Rows = `2n`: first all humans, then their rivals. A rival shared by two humans appears twice. No computer "players" other than rivals are listed.
3. Live: rows "anna", "Agnes" (Agnes = anna's rival, confirmed by the click text in §5) [O].

## 2. Values per row [C]

For each row person `p` two values are computed:
- `cached = clamp0(person[p] +0x1C)` (the cached net worth, refreshed at the year end, `rules-03` career section).
- `live = clamp0(net_worth(p))`.

Only for the row whose person == `viewer`:
- A loop over the viewer's branches (`0x42A50`/`0x42B10`) and the 4 eight-byte sub-entries at branch +0x33 exists, but its condition compares a pointer with 1 and is never true, so its sum stays 0. Ignore it.
- `tax_tithe(viewer, &a, &b)` is called and `live = clamp0(live − a − b)`.
- **Side effect:** `tax_tithe` adds this turn's sales tax to each city treasury (`0x85E25[c]`) and the tithe to `0x85E4D[c]` (`rules-02` §4.5). So every opening of Spielstand credits the city and church treasuries again with the current turn's tax and tithe amounts. The function consumes no RNG. Reproduce this for parity.

## 3. Score, bar, scale [C]

- Scale: `M` = maximum of `cached` and `live` over all rows (start 0, signed compare). `scale` = the first entry of `{10 000, 25 000, 50 000, 100 000, 250 000, 500 000}` (table `0x87248`) with `M ≤ entry` (unsigned); if none, `1 000 000` (entry 6).
- Per row `i` (0..2n−1), in row order: `print("%a%1N0$E$36>%*$E$R%d$|", p, width, 0x1C, 0, cached / 100)` with `width = min(199, cached · 200 / scale)` (unsigned division).
  - `%d` = `cached / 100` (cached net worth in hundreds of Taler, integer division). `live` only feeds the scale, it is never shown.
  - `%*` takes (width, 0x1C, 0) and stores a line element `(0x1C << 8) | 3` with the width and the value 0; the x cursor advances by `width`. How element kind 3 renders the 0x1C argument was not traced [UNVERIFIED]; live it is a **solid black** (RGB 0,0,0) rectangle, **15 px high**, top 3 px below the row's text line top [O].
  - Bar left edge: `$36>` = left margin 216 px from the window's left edge → x = 256 [C, O: measured 256..275 for width 20].
  - Score: `$R` right-aligned to the window's right inset (window right − 12 = x 588) [C, O: "0", "50" end at x ≈ 586].
- Live check: anna (cash 750) shows 0, so her cached +0x1C was < 100 at that point; Agnes shows 50 (cached 5000..5099) with a 20 px bar, which implies `scale` = 50 000, i.e. some row had `live` or `cached` in 25 001..50 000 [O; the individual live values were not read from memory].

## 4. Gem icon [C]

`%a` expands to `$L$2>%m$E$B$6>` with item style `0x1090` and data `0x73`: a clickable icon item showing icon id **0x73 (`ICON73`)**, the generic bullet used in all `%a` lists (its "checked" variant 0x74 is used in multi-selects, `rules-13`). Same icon on every row; it does not encode the player or a colour. Position: left margin `$2>` = 12 px; measured gem pixels x 55..66, 18 px high from the line top [O]. Name starts at `$6>` = 36 px → x 76 [C, O].

## 5. Geometry [C, O]

- `R = 2n` rows. `h = (R · 24 + 99) / 40` (integer). `win(40, 240 − 20h, 560, 40h, 0x8810)`: vertically centred on y = 240.
  - R = 2: h = 3 → window (40, 180, 560, 120) [O: frame 180..300].
  - With 12 humans (24 rows) h = 16 gives y = −80; the code does not guard this. Realistic maxima: R = 12 → (40, 60, 560, 360).
- Title `$C^ Spielstand _$Z` (0xC0378) on line 0 (y = window + 12), then a half line, so row `i` text line top = window y + 48 + 24·i [O: rows at 228 and 252 for window y 180].

## 6. Clicks [C, O]

`menu` loop until a result index ≥ R (the item index is the `%a` gem; clicking the name or bar does nothing [O]):
- Row index `i < n` (a human): `msg_box(160, 180, 320, 120, "$N$C%1N0 ist zur Zeit %1N4.$N", p, p)` (0xC03A4). Live: "anna ist zur Zeit ohne Amt." in a box x 160..480, y 180..300 [O].
- Row index `i ≥ n` (a rival `r` of human `h = row[i − n]`): `msg_box(140, 160, 360, 160, "$N$C%1N1 ist zur Zeit %1N4 und %f von %3N0.$N", r, r, r, h)` (0xC03C4). `%f` = "Gegenspieler"/"Gegenspielerin". Live: `[EXE text]` in a box x 140..500, y 160..320 [O].
- Live, the message box closed on a **right** click; left clicks inside or outside it did not close it (two tries) [O; the wait rule of `0x368E0` was not read, UNVERIFIED].
- A right click in the Spielstand window (no row) closes it [O]. Then `0x34AC0` (free items) and `win_close`.
- No RNG draws anywhere in this window [C for 0x5DC60, 0x618C0; net_worth per `rules-02` §8].
