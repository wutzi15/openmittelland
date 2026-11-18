# Branch screen goods row: price, arc gauge, stock

Source: branch screen `0x4FA70` (goods loop `0x4FD4C`..`0x50030`), item creation `0x34130`, font `ICONS2/ICONCS.DAT` decoded with `mad_assets::font` (glyph bounds and colours measured from the decoded pixels and the EXE palette at `0x8383D`). Tags: [C] instruction level, [M] measured from the asset, [UNVERIFIED] inferred. Extends `formats.md` §11.3, which calls the arc a "price-trend marker"; that name is wrong (§2).

Notation: `x`, `y` = slot anchor from `formats.md` §11.3 (x = slot centre; y = 176, or 125 / 227 with two rows). `g` = good, `c` = branch city (branch byte 1), `owner` = branch byte 2.

## 1. Draw order per good slot [C]

1. Good icon item.
2. Stock item.
3. Price text.
4. Arc gauge glyph, blitted straight into the window frame (drawn after the price, so it lies over the text where they overlap).

## 2. Arc with two blue beads = production-skill gauge [C][M]

- It is a **font glyph**, not an icon: code `0x80 + k` of `ICONCS.DAT`, taken from the second glyph pointer table `0xC43A0` (index `k` there = code `0x80 + k`, `rules-16b` §3).
- `k = (skill − 50) / 10` (integer division, truncating toward zero), clamped to 0..9, where `skill = branch_effective_skill(branch, g)` (`0x43CC0`, `rules-02` §3: owner's or director's skill for g, 150 for Wein with the code-44 ability). Skill range 0..150, so k = 0 for skill < 60, 9 for skill ≥ 140.
- It does not depend on price or price trend. There is no other variant and no colour change: the bead colour is part of the glyph.
- Blit position: top-left **(x − 18, y − 48)**, blitter `0x13AE0` mode 0 on the current window's frame (`[0xD500E]`).

Glyph geometry (8-bit font; pixel offsets inside the glyph, inclusive; "arc" includes its black outline) [M]:

| code | skill | advance | arc x | arc y | left bead x / y | right bead x / y | bead centre distance |
|---|---|---|---|---|---|---|---|
| 0x80 | < 60 | 27 | 10..26 | 18..25 | 11..16 / 19..24 | 20..25 / 19..24 | 9 |
| 0x81 | 60..69 | 32 | 5..31 | 16..23 | 6..11 / 17..22 | 25..30 / 17..22 | 19 |
| 0x82 | 70..79 | 34 | 3..33 | 14..23 | 4..9 / 15..20 | 27..32 / 15..20 | 23 |
| 0x83 | 80..89 | 37 | 0..36 | 12..23 | 1..6 / 13..18 | 30..35 / 13..18 | 29 |
| 0x84 | 90..99 | 37 | 0..36 | 10..27 | 1..6 / 11..16 | 30..35 / 11..16 | 29 |
| 0x85 | 100..109 | 37 | 0..36 | 7..23 | 1..6 / 8..13 | 30..35 / 8..13 | 29 |
| 0x86 | 110..119 | 37 | 0..36 | 5..23 | 1..6 / 6..11 | 30..35 / 6..11 | 29 |
| 0x87 | 120..129 | 36 | 1..35 | 3..23 | 2..7 / 4..9 | 29..34 / 4..9 | 27 |
| 0x88 | 130..139 | 36 | 1..35 | 1..23 | 5..10 / 2..7 | 26..31 / 2..7 | 21 |
| 0x89 | ≥ 140 | 36 | 1..35 | 0..23 | 11..16 / 0..5 | 20..25 / 0..5 | 9 |

- The arc is a gold ring segment that grows from a shallow dish (0x80) to a closed ring (0x89); the two beads sit on the arc ends and climb with it, meeting at the top in 0x89.
- Bead colours: palette 138..149 (blue ramp, (100,104,240) down to (8,8,120)) with a white highlight 186; arc: gold indices 1, 2, 3, 4, 9, 10, 12, 15; outline 216 (black).
- On screen the arc's bottom row is y − 25 (glyph row 23) for every code except 0x80 (y − 23) and 0x84 (one stray row to y − 21).
- The observation "about 40 px wide, 14 high, beads 29 apart" matches codes 0x83..0x86 (skill 80..119); a newly started player's skills fall in this band [UNVERIFIED: start skill values not checked here].
- The 16-bit font `ICONS3/ICONCS.DAT` was not measured; same geometry assumed [UNVERIFIED].

## 3. Price number [C]

- Text `$g$d$C%B$N` (`0xBE5DC`), printed through `0x322C0` with:
  - left margin and cursor x = **x − 39**, box width **80** (`0xD502C`), cursor y = **y − 41**;
  - `$g`: gold glyphs with a one-pixel dark outline (`rules-16b` §3); `$d`: text flag 0x20 (meaning [UNVERIFIED]); `$C`: centred in the 80 px box (centre x + 1).
- Value: `price[c][g] + b`, where `price` = city record `+0x22 + g` (1..99) and `b = 2` if `owner` has a type-42 record (church seminar) with counter < 7, else 0. `b` is computed once when the screen opens. This is the local sale price per unit, the same `u` as `rules-11` §3 / `rules-02` §4.4.
- `%B` prints the number with the small digit glyphs 0x10..0x19 (`rules-12` §1). Small digits: advance 8, ink 11 rows (glyph rows 0..10). With the price ≤ 101 no separator is printed.
- Vertical relation [UNVERIFIED: assumes glyph row 0 = cursor y]: digits occupy y − 41..y − 31; the arc ink spans y − 30..y − 25 for 0x80 and rises to y − 48 for 0x89, so from 0x83 up the digits sit inside the arc.

## 4. Stock number [C]

- Item at **(x, y + 27)**, w = h = 0 (computed), style `0x40054`, data = pointer to the branch's u16 stock of g (branch `+5 + 2g`). It shows the live value of that word.
- Style decode (`0x34130`, `rules-16b` §6): kind 0x50 = numeric field with the small-digit metrics, nibble 4 = **4 digits**, flag 0x40000 = centred (x −= w/2).
  - Width = `(4 − 1)·(adv[0x1F] + s) / 3 + 4·(adv[0x10] + s)`, s = item letter spacing (`0xD502E`); 0x1F is the small thousands dot. Advance table base `0xD4A38` (inferred from the `0x2E`/`0x30` offsets used for kind 0x00).
  - Height = **12** for kinds 0x40, 0x50, 0x60, 0xD0 and 18 for 0x00, 0x10, 0x20, 0xC0 [C]. `rules-16b` §6 states 18 for 0x40..0x60; that is wrong.
- Displayed as `0.000`: four small digits with a dot after the first. Zero padding and the dot are consistent with the width formula and the observed screen; the item draw routine itself was not traced [UNVERIFIED]. Colour of the digits not traced [UNVERIFIED].

## 5. Good icon [C]

- Item at (x, y − 24), style `0x41090` (icon button, centred, no background backup). Icon `0x32 + g`, or `0x41 + g` (dark variant, +15) when `g / 5 > L`, `L` = `0x43DD0(branch)` = the higher of owner's and director's certificate level (`rules-09`). Height and width from ICONIN.
- Slot 0 (warehouse, `ICON50`, style `0x1090`) is not centred: top-left (x − 24, y − 24). It has no price, arc or stock.

## 6. Open

- `$d` flag 0x20 semantics.
- Exact digit rendering of numeric items (padding, colour).
- 16-bit font glyph geometry.
