# Open questions, round 7 (rules-46)

Three follow-up questions: the `%j` in the Kreditbuch loan line, the music call of the privileges menu, and the RNG order of the seminar re-invitation. Conventions as in `rules-45`: `rand(n)` = `0x36FF0` (0..n−1), `print` = `0x322C0`. Person records are 0xF8 bytes at `0xE3A7C`. Year word = `0xEF4B8`. `pick_random(place, church, office, ctrl)` = `0x43560` and the iterator `0x433E0`/`0x434E0` as in `rules-24` §2 and `rules-27` (0xFF = wildcard; one `rand(count)` over all matches, no draw when none match). Tags: **[C]** instruction level, **[L]** decompiler or xref only, **[UNVERIFIED]** inferred. Everything is [C] unless marked. Each section ends with an "Answer:" paragraph.

## 1. Kreditbuch `%j`

### 1.1 The print-code handler

`print` reads the character after `%` (and after an optional digit), subtracts 0x25 and jumps through the table at `0x32AA8` (83 entries for 0x25..0x77, `0x32C20`). Entry `'j'` (index 0x45) = **`0x33139`**. That handler:
1. Takes the next argument as the icon id.
2. Creates an item (`0x34130`) at (0, 0) with style **`0x1090`** and data = icon id. Its width and height are the icon's `ICONIN` nominal size.
3. If line width (`0xD6A30`) + item width (`0xD6A04`) > box width (`0xD502C`), or the line's marker count (`0x84D9A`) > 0xF0: it pops the item (`0x34930`), flushes the line (`0x30FB0(0)`) and creates the item again on the new line.
4. Line width += item width. Appends a **kind-2** marker `(item << 8) | 2` to the line list `0xC25C0`.

Placement at flush follows `formats.md` §11.2 (kind-2 marker): top-left at (pen x, **line y − 6**), pen then advances by the item width. Style `0x1090` is clickable (bit 0x400 clear, `rules-38` §0).

### 1.2 The Kreditbuch call (`0x5C450`, loan loop `0x5C779..0x5CA20`)

Loans are the player's type-56 (0x38) records (`rec_find(0x38, player, 0xFF)` / `rec_next`). For each record r, one `print` with arguments in print order:

| code | argument |
|---|---|
| `%1N3` | r+4 (lender) |
| `%qe` | r+4 again |
| `%d` | u32 r+5 (amount) |
| `%d` | r+9 (interest %) |
| `%l` | year + r+3 − 1 |
| `%j` | **0x6A** |

Format `0xBFE44` (year wrapped in `$h`) when `r+3 − 1 < 2` as a signed int, i.e. counter 0, 1 or 2. Format `0xBFEA0` otherwise. After the second loan the right page starts (`0xBFEF8` `$|$4H$56>$10<$Z`).

Icon `0x6A` = `ICON6A`, **32 × 32** in both `ICONS2` and `ICONS3` `ICONIN.DAT` (width table +0xA00, height table +0xE00; flags 0, so not in the `B` directory). It is the same icon as the residence "Kaufen" line (`rules-35`) and "Kaufinteresse zurückziehen" (`rules-26`).

Line layout: `$Z` ends the text line. The next line starts with the icon at the left margin (top at line y − 6). Then one normal space and the plain glyphs "Kredit tilgen". Only the icon is an item. The text is not clickable.

### 1.3 Click handling

`base` = the return of the banner helper `0x49D60(player, "Kreditbuch", 0)` (called after `draw_icon(0, 40, 0xD4)`). `i = menu(base, 0, &d) − base − 4`. The loan lines create no item except the `%j`, so loan k's icon is item base + 4 + k [L for what items base..base+3 are]. If `i < n` (n = loans listed): `pay(player, u32 r_i+5)` (`0x43860`) and delete r_i (`0x42550`), then the book is redrawn (back to `0x5C779`). If `i ≥ n` (right click or a banner item), the loop ends. The amount paid is the printed amount r+5. No interest is added here.

Answer: `%j` is handled at **`0x33139`** (print jump table `0x32AA8`). It draws one clickable inline icon item, style `0x1090`, taken from the next print argument. In the Kreditbuch that argument is **icon 0x6A** (`ICON6A`, 32 × 32). The item sits at the text cursor's pen x with its top at the line's y − 6. The pen then advances 32 px, followed by the space before "Kredit tilgen". The icon is the whole menu item for that loan: a click on loan k's icon (item `base + 4 + k`) repays it in full (`pay(player, r+5)`) and deletes the record. The words "Kredit tilgen" are plain text.

## 2. Privileges menu music

At the entry of `0x5F080(player)`, before the window opens:
- `CMP dword [0x86BC8], 0xC3500` / `JA` skip. So the call runs when the **unsigned** dword `0x86BC8 ≤ 0xC3500` (800 000). This is the "4-MB-Version" test (`rules-14` §1.4). In the large-memory build there is no music call at all.
- Then `music(0x15, 1)` (`0x41E50`, AL = 0x15, DL = 1). Mode 1 starts `SONGL.MID` unless song 0x15 is already the current song (`0x86C58`). If the sound driver is inactive, the front end first clears `current` to 0 (`rules-14` §4), so the start runs, but it plays nothing audible.
- 0x15 is not one of the random group codes 0x19..0x1E. `0x41E50` draws `rand` only for those codes, so this call makes **no RNG draw**. The test is a plain read. Ghidra shows no direct write xref to `0x86BC8`, only reads (`0x41E50`, `0x41FF0`, `0x45C30`, `0x5F080`, `0x4E810`, `0x15630`, `0x17930`). `rules-44` puts its setting in the start-up code [L].

Answer: music 0x15 is requested on every entry to the privileges menu when the memory dword `0x86BC8` is ≤ 0xC3500 (unsigned), i.e. the low-memory "4-MB" build. It is requested with mode 1 (start unless 0x15 is already current). In the large-memory build no music call is made. No RNG draw happens in either case.

## 3. Seminar re-invitation `0xADE02` (type 42 handler `0x1CCE0`, counter 6)

Branch condition: the handler tests the record counter r+3. 7 → sermon (`rules-13` §2.10). **6 → this branch** (`0x1CE10`). 0 → the `0xB3C1E` loss text and delete. Any other counter prints nothing.

Order in the counter-6 branch (r+1 = owner, r+4 = title, r+5 = seminar city):
1. `print(0xADD87, 0xB4E98[r+4])` (the letter opening).
2. **J = `pick_random(0xFF, 1, 0xFF, 0)`**. This lists every computer person (controller nibble 0) whose church-office flag (low nibble of +0x10) is 1, any place, any office, in slot order. If the list is non-empty it draws **`rand(count)`**, even when count = 1. If it is empty, J = 0xFF and nothing is drawn.
3. **`rand(100)`**, always drawn, also when J = 0xFF.
4. If `rand(100) ≥ 40` or J = 0xFF: `print(0xADF4B)` and return. The record stays, so the bonus lasts until counter 0. No further draws.
5. Otherwise (invitation):
   1. **`rand(3)`**. Next title t' = `(r+4 + rand(3)) mod 4`. This corrects `rules-13`'s "`rand(4)`". t' can equal the current title.
   2. `print(0xADE02, 0xB4E98[t'], J)`: `%s` = title string t', `%1N3` = J.
   3. Favour loop: for every person q from the iterator `0x433E0(r+5, 0xFF, 0xFF, 0xFF)` (slot in use, office place +0x0F = seminar city, any church flag, any office, **any controller**, so humans count too and the owner counts if their place is that city), in slot order: draw **`rand(4)`**, then `favour_increase(r+1, q, rand(4))` (`0x43620`, raises favour(q→owner) by 0..3, halved for a different religion, `rules-03` §1).
   4. Delete the record (`0x42550`).

RNG sequence: `[rand(count_J)]`, `rand(100)`, then in the invitation branch only `rand(3)`, then `rand(4)` × (number of persons at the seminar city). The favour draws all come after the text.

Answer: the branch runs at counter 6. J is a random computer clergy person (`0x43560(0xFF, 1, 0xFF, 0)`, one `rand(count)` over all matching persons in slot order, no draw when none exist). It is drawn **before** `rand(100)`, and `rand(100)` is always drawn. The invitation needs J ≠ 0xFF and `rand(100) < 40`. Then `rand(3)` picks the next title as `(title + rand(3)) mod 4` (`rand(3)`, not `rand(4)`). `0xADE02` prints with (title string `0xB4E98[t']`, J), in that order. After the text, one `rand(4)` per person whose office place is the seminar city (all controllers, slot order) feeds `favour_increase(owner, q, ·)`. Then the record is deleted. The failure branch prints `0xADF4B`, draws nothing more and keeps the record.
