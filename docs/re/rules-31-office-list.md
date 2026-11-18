# Office list: the person list opened from the plot map (rules-31)

Source: the list part of `0x45C30(player, mode)` (`0x46390..0x46A55`, exit `0x47BC0`); item creation `item(x, y, w, h, style, data)` = `0x34130`; item painter `0x34BF0` (kind 0xB0 branch `0x3557A`); `menu` = `0x35B60`; `print` = `0x322C0` (`%*` handler `0x33D0F`); line painter `0x31610` (element kind 3 at `0x318C1`); `draw_icon` = `0x2E310`; OK item `0x348D0`; `human_rival_of` = `0x44F10`; `rec_find`/`rec_next` = `0x426B0`/`0x427B0`. Continues `rules-29` §5. Tags: **[C]** instruction level, **[L]** decompiler-level or taken from another doc, **[O]** matches the team lead's live capture, **[UNVERIFIED]** not confirmed.

Coordinates: the window is (20, 40, 600, 400), style `0x8910` (`rules-29` §1). "window" = relative to its top left; screen = window + (20, 40).

## 1. Drawing order per (re)display [C]

1. `draw_icon(0, 0, 0x1B9)`: the background picture (§4).
2. Title: `0xBC568` `$H$C^ %s in %o _$N` (verb, place) or, mode 12, `0xBC57C` `$H$C^ %s bei... _$N` (verb). `$H` = line 0, window y 12.
3. `r = human_rival_of(player)` (§3.3). Then one item per row (§2).
4. Level entry (§6), then the OK item `0x348D0`, then `flip`.
5. Wait until no button is pressed, then `menu`.

## 2. Entries [C]

### 2.1 Rows per place and level
`n` rows; row `i` (0..n−1) shows office `obase + i` of the place, labelled with name index `k = nbase + i`. Holder = `0x433E0(place, 0xFF, obase + i, 0xFF)`; castles/camps: byte +2 of `0x42A50(0xFF, place, 0xFF)` (0xFF when no property).

| place | level | n | obase | nbase |
|---|---|---|---|---|
| city 0..9 | worldly | 7 | 0 | 0 |
| city 0..9 | church | 3 | 7 | 7 |
| county 10..12 | worldly | 8 | 0 | 10 |
| county 10..12 | church | 5 | 8 | 18 |
| Mittelland 13 | worldly | 8 | 0 | 23 |
| Mittelland 13 | church | 3 | 8 | 31 |
| castle/camp 14..20 | (only) | 1 | – | 34 |

### 2.2 Position
`item(x = COL[k]·100, y = ROW[k]·36 + 48, w = 200, h = 72, style 0x10B0, data = &args[i])`, window coordinates. `COL` = u16 at `0x86D1C + 4k`, `ROW` = u16 at `0x86DA8 + 4k` (tables of 32-bit entries, low word read). Text is centred (`$C`) in the 200 px item, so the entry's centre x is `COL·100 + 100` (screen `COL·100 + 120`).

| k | office label | COL | ROW | item (window) | item (screen) |
|---|---|---|---|---|---|
| 0, 1, 2 | Ratsherr | 0, 2, 4 | 5 | (0/200/400, 228) | (20/220/420, 268) |
| 3, 4, 5 | Baumeister, Richter, Kämmerer | 0, 2, 4 | 3 | y 156 | y 196 |
| 6 | Bürgermeister | 2 | 1 | (200, 84) | (220, 124) |
| 7, 8 | Priester | 1, 3 | 4 | (100/300, 192) | (120/320, 232) |
| 9 | Domherr | 2 | 2 | (200, 120) | (220, 160) |
| 10, 11, 12 | Geheimrat | 0, 2, 4 | 6 | y 264 | y 304 |
| 13, 14, 15 | Hauptmann, Justizberater, Finanzberater | 0, 2, 4 | 4 | y 192 | y 232 |
| 16 | Kanzler | 2 | 2 | (200, 120) | (220, 160) |
| 17 | Vogt | 2 | 0 | (200, 48) | (220, 88) |
| 18, 19 | Kellermeister, Sakristan | 0, 2 | 5 | y 228 | y 268 |
| 20 | Diakon | 3 | 3 | (300, 156) | (320, 196) |
| 21 | Abt | 1 | 3 | (100, 156) | (120, 196) |
| 22 | Bischof | 2 | 1 | (200, 84) | (220, 124) |
| 23, 24, 25 | Hofrat | 0, 2, 4 | 6 | y 264 | y 304 |
| 26, 27, 28 | Marschall, Justizminister, Finanzminister | 0, 2, 4 | 4 | y 192 | y 232 |
| 29 | Großkanzler | 2 | 2 | (200, 120) | (220, 160) |
| 30 | Regent | 2 | 0 | (200, 48) | (220, 88) |
| 31, 32 | Erzdiakon, Inquisitor | 1, 3 | 4 | (100/300, 192) | (120/320, 232) |
| 33 | Erzbischof | 2 | 2 | (200, 120) | (220, 160) |
| 34 | Besitzer | 2 | 3 | (200, 156) | (220, 196) |

Shapes: city worldly 1 + 3 + 3 (top to bottom) [O]; city church 1 + 2; county and Mittelland worldly 1 + 1 + 3 + 3; county church 1 + 2 (Abt left, Diakon right) + 2 (Kellermeister col 0, Sakristan col 2); Mittelland church 1 + 2; castle one entry.

### 2.3 Text
Item kind 0xB0 prints `print(args[0], args[1], …, args[13])` with the cursor at the item's top left and margins = item x, item width (`0x3557A`). `args` is a 13-dword block per row in the caller (`EBP−0x6AC + 0x34·i`); the painter reads 14 dwords, the last one belongs to the next row and is never consumed.

Holder = 0xFF ("unbesetzt"): `args = (0x86C94, male office name)`, format `$0s$C%s$N(unbesetzt)$N$1s`.

Otherwise `args = (fmt, office, c1, name, c2, c3, fav, 0x1D, 0, c4)` with:
- `office` = `0xB4378[k]` (male) when holder byte +0 == 0, else `0xB5854[k]` (female).
- `name` = dword at holder +0x09, a string pointer [L: field not in `rules-00` §4].
- `fav` = holder's favour toward `player` (byte holder + 0x20 + player).
- `fmt` (first match wins):

| condition | fmt | line 3 |
|---|---|---|
| `fav > 100` (holder is blackmailed by `player`, `rules-03`) | `0x86CF4` `$0s$C%s$N%c %s %c$N%c =%v%v%v %c$N$1s` | glyph `=` (§3.4) instead of the bar |
| holder is human (byte +0x10 & 0xF0 ≠ 0) | `0x86CCF` `$0s$C%s$N%c %s %c$N%c %v%v%v %c$N$1s` | no bar |
| else (computer) | `0x86CAE` `$0s$C%s$N%c %s %c$N%c %* %c$N$1s` | favour bar (§5) |

So line 1 = office, line 2 = `c1 name c2`, line 3 = `c3 [bar] c4`; line tops at item y + 0, 24, 48 (line height 24). `$0s`/`$1s` set the item spacing to 0 and back to 1 (`formats.md` §8.3).

## 3. Marker glyphs [C for the conditions; glyph shapes from ICONCS.DAT]

`%c` prints one font code. In the default text mode (flag 0x40 set, dark-only) the painter draws the glyph of code `c + 0x80` (`rules-16b` §3 [L]); the low codes 0x22, 0x23, 0x2A, 0x2F, 0x3D, 0x40, 0x60 are empty in the gold half, and at font load they get the advance of their dark twin (`formats.md` §8.3). 0x20 = blank, advance 5.

| slot | condition | code | glyph (dark code, advance) |
|---|---|---|---|
| c1, before the name | a type-29 record (mistress/lover) with owner = holder exists (`rec_find(29, holder, 0xFF)`) | 0x2F, else 0x20 | 0xAF, 17 px: a heart |
| c2, after the name | holder religion (byte +0x03): 0 → 0x22, 1 → 0x23, 2 → 0x20 | 0x22 / 0x23 | 0xA2, 14 px: thin cross with cross-bar ends (evangelisch); 0xA3, 14 px: heavier cross on a stepped base (katholisch) |
| c3, line 3 left | `player` holds a saboteur contract (type 7) whose +4 target = holder: `rec_find(7, player, 0xFF)`, `rec_next` until +4 == holder | 0x2A, else 0x20 | 0xAA, 16 px: a round bomb with a fuse |
| c4, line 3 right | `player` holds a spy contract (type 6) with +4 == holder (same search): none → 0x20; found → 0x40 if its evidence dword +0x0B == 0, else 0x60 | 0x40 / 0x60 | 0xC0, 11 px: outlined figure; 0xE0, 11 px: filled figure |
| line 3, middle (fmt `0x86CF4`) | `fav > 100` | literal `=` (0x3D) | 0xBD, 14 px: a funnel shape over a block with an arch [UNVERIFIED what it depicts; meaning is "blackmailed by you"] |

The glyph after every name is therefore the **religion** marker, present for every holder with religion 0 or 1; the same glyph on all names means they share one religion [O]. Religion value 2 (no glyph) was not seen in other docs [UNVERIFIED whether it occurs].

### 3.3 The rival [C]
`r = human_rival_of(player)` (`rules-03` §4.1: the computer person with the lowest favour toward `player`, preferring the player's office location). For the row whose holder == r, bit 0x80 of the text-flags high byte (`0xD5025`) is flipped around the `item` call; the item stores the flags at creation (+0x0C), so the whole entry is drawn **highlighted** (§5.2). Side effect: when `player` has no office and no branch, `human_rival_of` calls `rand(10)`; it runs on every (re)display of the list, including level toggles. Keep for RNG parity.

## 4. Background and watermarks [C for the call; geometry measured from the extracted icon]

`draw_icon(0, 0, 0x1B9)` = `ICONR9`, a single 600 × 400 parchment picture with torn edges (the edge pixels are the transparent key, magenta in the extracted PPM). The two shields with crossed swords and the large coat of arms are **painted into this picture**; there are no separate icons and no blending. Measured bounds (pixels that differ from the parchment colour (247, 239, 206) by more than 40 summed RGB):

| element | window | screen |
|---|---|---|
| left shield + swords | 90, 72 – 185, 204 | 110, 112 – 205, 244 |
| right shield + swords | 412, 72 – 507, 204 | 432, 112 – 527, 244 |
| coat of arms | 215, 130 – 388, 341 | 235, 170 – 408, 381 |

The capture's x range for the coat of arms (235..410) matches; its y range (230..415) does not match the icon drawn at window y 0 [UNVERIFIED: recheck the capture; the emblem's faint top (fleur-de-lis, helmet) may have been missed].

The map is not erased first: the picture is drawn over the window frame, then the title and the items over it.

## 5. The black bar under an entry (`%*`) [C]

`%*` takes (width, glyph, mode) = (`fav`, 0x1D, 0) and stores a line element; when the line is painted, the glyph `0x1D` of the first font table (`0xC41A0`) is blitted `width` times at x, x+1, …, x+width−1, with blitter mode 0. Glyph 0x1D is a 1 px wide vertical stroke in rows 6..14 of the 24-row cell (9 px), colour index 216 in `ICONS2`, RGB555 0 (black) in `ICONS3`. So:

- **Bar = the holder's favour toward the viewing player**, `width = fav` px (0..100), height 9 px, top at line-3 top + 6 = item y + 54. Start values 48..52 explain the observed ~48 px [O].
- Only for computer holders with `fav ≤ 100`; humans get no bar, blackmailed holders get the `=` glyph.
- The line is `c3 ␠ bar ␠ c4` centred in the 200 px item; with blanks for c3/c4 the line is about `fav + 20` px wide [UNVERIFIED: exact x, glyph spacing of 1 px per glyph not checked].
- The bar is drawn from the font table directly and ignores the highlight flag, so it stays black on hover [C for the code path].
- Spielstand's bar (`rules-28` §3) is the same element with glyph 0x1C (rows 3..17, 15 px).

### 5.2 Hover [C]
`menu` hit-tests items in creation order on the screen rect `[sx, sx + w) × [sy, sy + h)` (item +0x14/+0x16 = window origin + item x/y; +4/+6 = w/h), skipping items with flag +9 & 4. The item under the pointer gets +9 bit 0x40, the previous one has it cleared; both are repainted. The painter flips text bit 0x80 while an item has +9 bit 0x40, so **the whole entry (office, name, glyphs) is re-printed highlighted**.

- Hover rect per entry: screen `(20 + 100·COL, 88 + 36·ROW, 200, 72)`.
- Highlight colour: in dark-only text mode a highlighted run is drawn in solid colour 1 (`rules-16b` §3, `formats.md` §8.4) [L]; red in the capture [O].
- The rival's entry is created with bit 0x80 already flipped, so it shows highlighted at rest and flips back to normal under the pointer.

## 6. Level entry and closing [C]

- Places < 14: `item(300, 364, 0, 0, 0x41030, text)`, a menu line (kind 0x30, size from the text, height 24) centred on window x 300 = **screen x 320, top y 404**. Text `0xBC590` "Zur kirchlichen Ebene..." on the worldly level, `0xBC5AC` "Zur weltlichen Ebene..." on the church level. Hover highlight as §5.2 (rect = measured text width × 24).
- Castles/camps: `print("%M$E")` (`0xBC5C4`) instead, an invisible placeholder that keeps the index layout.
- Then the OK item `0x348D0` (icon 0xCD, 1 × 1 px, at window (w − 32, h − 32) = (568, 368), screen (588, 408); `rules-26` §1). Practically invisible.
- Result `idx = menu − first item`:
  - `idx < n`: the entry; dispatch on `mode` (`rules-29` §3, `rules-04`). If nothing was done (empty office, own person in a mode ≠ 0, action flag unset), the list stays and waits again without redraw.
  - `idx == n`: level toggle, items freed, list redrawn (§1).
  - `idx > n` (OK item or **right click**, which returns last item + 1): items freed and the plot map is redrawn (`rules-29` §5).
  - Entries that ran an action also free the items and redraw the list.

## Open
1. Exact x of the glyphs and bar inside line 3 (per-glyph spacing) [UNVERIFIED].
2. Coat-of-arms y range vs the capture (§4).
3. What glyph 0xBD depicts.
