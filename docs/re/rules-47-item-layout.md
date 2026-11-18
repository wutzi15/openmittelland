# Item layout inside printed lines (rules-47)

How `%m`, `%M`, `%a` and `%j` items are measured, placed and painted inside a line, and where plain text continues after them. Code: `print` = `0x322C0`, glyph append = `0x30FB0`, line painter = `0x31610`, item create = `0x34130`, string width = `0x33F60`, item painter = `0x34BF0`. Window state at `0xD5000` as in `formats.md` §8.6: `d5014` = text origin x (0 for windows opened with flag 0x8000, which draw into a buffer), `d5018` = left margin of the current line, `d502A` = left margin restored at each new line, `d502C` = box width, `d5022` = alignment (1 L, 2 R, 3 C, 4 B), `d502E` = letter spacing `sp`, `d6A30` = line width so far. `adv[c]` = font advance after the load-time overrides (`formats.md` §8.3). Tags: **[C]** instruction level, **[L]** decompiler or xref only, **[UNVERIFIED]** inferred. Everything is [C] unless marked.

## 1. The line buffer

Each `print` character goes to `0x30FB0`, which appends one 16-bit entry to the line list `0xC25C0` and adds to `d6A30`:
- glyph: entry `flags | c`, width `adv[c] + sp`;
- space: entry `(adv[0x20] + sp) << 8 | 0x20` (the width rides in the high byte, no flags), width 6 with `sp = 1`. The space also records the word-break point.

Items enter the same list as markers `(item index << 8) | kind`:

| code | handler | arguments | style | marker | adds to `d6A30` |
|---|---|---|---|---|---|
| `%m` | `0x331EA` | **two**: style, data | caller's | kind 1 | item width `w` |
| `%M` | `0x3329D` | none | `0x1630`, data `0xBB5C8` (empty string) | kind 1 | nothing (and `w` = 0) |
| `%j` | `0x33139` | one: icon id | `0x1090` | kind 2 (y − 6) | `w` |
| `%a` | `0x33CCB` | none | expands to `print("$L$2>%m$E$B$6>", 0x1090, 0x73)` | via `%m` | via `%m` |

`%m` and `%j` first create the item (`0x34130` at x = 0, y = 0, preset width 0), then test `d6A30 + w > d502C` or more than 0xF0 entries. If either is true they delete the item (`0x34930`), flush with `0x30FB0(0)` (word wrap) and create it again. The item's text is **not** put into the line. Only the marker is.

`w` is computed once at creation from the **style the caller passed** (`formats.md` §8.6 table). For string classes 0x30/0xA0/0xF0 with `n = style & 0xF = 0`, `w` = `0x33F60(data)` = Σ(`adv[c] + sp`), with the trailing glyph's `sp` included. With `n > 0`, `w` = `sp + 2n·(adv['M'] + sp)`. That is a fixed slot, whatever the string is. `sp` is the window's value at creation and is stored in the item (`+0x0E`).

## 2. Measuring and placing the line (`0x31610`)

Start pen `x0 = d5014 + d5018`. If `d6A30 < d502C`: alignment 2 adds `d502C − d6A30`; alignment 3 adds `(d502C − d6A30) / 2` (signed, truncated). If the line is as wide as the box or wider, there is no shift. Alignments 1 and 4 add nothing here. Justification happens only in `0x30FB0`, on a wrapped line or a `$M` flush (§4).

Then the entries are walked in order:
- glyph: draw at pen, pen += `adv[c] + sp`;
- space: pen += its stored width (6, or 6 + stretch on a justified line);
- item marker (kind 1 or 2): `item.x += pen − d5014`, `item.y += line y − d5016` (kind 2: line y − 6). Then **pen += item.+4 (`w`)**.

So the width used for alignment is exactly `L = Σ glyphs (adv + sp) + Σ spaces 6 + Σ item w`. No padding is added around items. The space before or after an item is an ordinary space entry: it is measured as 6 and advances 6. Text after an item starts at `item.x + w`, whatever the item draws.

`$N`, `$E`, `$L`, `$R`, `$C`, `$B` and the margin codes call `0x31610` directly, so they never justify. `$E` raises `0x84D9E` around the call. The y and `d5018` then stay put, so the next segment starts again at `d5014 + d5018`. Margin codes (`0x32801`, `0x3286D`, `0x32965`, `0x3269D`):
- `$n>`: `d502A = d5018 = n·6`, and `d502C` is set so that the right edge `d502A + d502C` stays where it was.
- `$n<`: `d502C = window width − d502A − n·6`.
- `$n=`: `d502C = n·6`.
- `$|` = `print("$2>$2<")`.
- `$H`: `d5018 = d502A = 12`, `d502C = width − 24`.

A new line resets `d5018 := d502A`, so margins carry over from row to row.

## 3. Painting the item text (`0x34BF0`)

For each item not yet painted, the painter loads the item's flags (`+0x0C`) and `sp` (`+0x0E`). It flushes any pending line. Then it sets **`d5018 := item.x`, `d501A := item.y`, `d5022 := 1`** (left) and raises `0x84D9E`. It writes the content and flushes with `0x31610`:
- classes 0x30, 0x80, 0xA0, 0xF0 (0xF0 only while highlighted): `print(item.+0x10)`;
- 0x00/0x10/0x20/0xC0: number writer `0x32050`;
- 0x40/0x50/0x60/0xD0: `0x32130`;
- 0x90/0x100/0x110/0x130: icon blit at (item.x, item.y).

So the text starts at **exactly the x the layout stored** (`d5014 + item.x`), left-aligned, with the same `adv + sp` per glyph. It is never right-aligned or centred inside its slot, and it is never re-measured. The window's `d502C` at paint time is still the wrap limit inside that nested `print`, and the slot width `w` plays no part in it. A string item narrower than `w` leaves `w − text width` blank on its right. A string item with `n > 0` reserves the fixed slot of §1.

## 4. Justified lines

`0x30FB0` stretches only entries with low byte 0x20 and non-zero width, i.e. real spaces. It runs on a wrap, and on `0x30FB0(0)` (the `$M` code at `0x3270A`, and the `%m`/`%j` overflow path), when alignment is 4. Free width `d502C − width up to the break` is spread as `free / count`, plus 1 for the first `free mod count` spaces. Items and glyphs never stretch, and spaces next to items stretch like any other.

## 5. Case a): Spieloptionen (`0x4EFC0`)

Window `0x2FA80(100, 80, 440, 320, 0x8810)`, so `d5014 = 0` (buffered) and text width 416. Call: `print(0xBE260, 0x32, "", 0x32, "", …)`. Ten `%m` items, each with **style 0x32** (class 0x30, n = 2) and data = the empty string `0xBDECC`. The loop sets the data to `"Ein"`/`"Aus"` (`0xB7390[bit]`) with `0x34B80` (item `+0x10` := data, clears the painted bit) before each repaint. That does not change `w`. There is no `$8<` in the EXE string. The row is `%aLabel:$E$R%m$N`:
1. `%a`: `$L`, `$2>` (left 12, box 416). Bullet icon 0x73 (18 × 18 in both sets) at window x + 12, then `$E`. `$B`, then `$6>`: left 36, box 392.
2. The label from x 36, then `$E`.
3. `$R`, `%m` with `w = 1 + 2·2·(adv['M'] + 1) = 1 + 4·19 = 77`, then `$N`: pen = 36 + (392 − 77) = **351** (window-relative), screen x = **451**.

"Ein" and "Aus" are both drawn from x 451 (screen), left-aligned in a 77-px slot that ends at 528 (window right − 12). Check: with the 16-bit font (`E` = 14, `ICONS3`), "Ein" spans 15 + 8 + 16 = 39 px including the trailing 1 px, so it ends at 490. That is "2 px before" the port's assumed margin 492 (`$8<` after `$6>`). The port's "+2 for right-aligned items" rule only matches "Ein". "Aus" (15 + 15 + 13 = 43 in `ICONS3`) also starts at 451, so a right-aligned port draws it 4 px left of the original [UNVERIFIED: no capture of "Aus"].

## 6. Case b): branch production row (`0x4FA70`, call at `0x504BD`)

Order panel window `0x2FA80(0, 278, 640, 120, 0x8010)`, `d5014 = 0`. Format `0xBE658` = `$78<$R%m%M %m$E$|$30>$Lmit %m %s$E$|$62>$7=$C%s$E$|$70>$L%m%M %s$E$|$R%m$|$L`. There is a `%M` after the verb, which the brief's quote omits. Arguments in order:

| slot | style | data | `w` |
|---|---|---|---|
| verb `%m` | `0x8A0` (class 0xA0, n 0) | `0xB65A0[good]` ("Schert") | text width |
| `%M` | `0x1630` | `""` | 0 |
| good `%m` | `0x8A0` | `0xB6618[good]` ("Wolle") | text width |
| workers `%m` | `0x802` (u8, n 2, zero-padded) | `&order[4]` | `6/3 + 2·15 = 32` |
| `%s` | | `0xB462C` / `0xB4CDC` (Arbeiter / Arbeitern, by count = 1) | glyphs |
| `%s` | | `0xB65DC[good]` (preposition) | glyphs |
| means `%m` | `0x802` | `&order[5]` | 32 |
| `%s` | | `0xB465C` / `0xB4D0C` (means sg/pl, by count = 1) | glyphs |
| status `%m` | `0x1830` | `0x86770 + 2·order[7]` | string width |

`$78<` with `d502A = 12`: `d502C = 640 − 12 − 468 = 160`, right edge x = 172. `L = w(Schert) + 0 + 6 + w(Wolle) = 72 + 6 + 57 = 135`, using `S17 c10 h11 e10 r11 t7` and `W21 o9 l6 l6 e10`, each + 1 (same in both font sets). Result: **Schert at x = 172 − 135 = 37, Wolle at 37 + 72 + 6 = 115** (ending at 172 with its trailing 1 px).

The code has no mechanism that moves the second item 4 px relative to the first. The first item's advance is its `w`, the space is 6, and both texts are painted at their stored x. The reported "Wolle 4 px left, Schert exact" does not follow from this code. The port's +2 rule for right-aligned items puts Schert at 33 and Wolle at 113. [UNVERIFIED] Re-measure against 37/115 with that rule removed.

## 7. Notes for c)

- Number items reserve `w` from the style, not from the value. The text is left-aligned in the slot, so the gap before the next word is `w − drawn` (`formats.md` §8.6). Sell row (`0x50779`, format `0xBE6A8`): quantity `0x814` (u16, n 4, padded: `0.000`, `w` = 66, no gap). Count `0xC02` (n 2, bit 0x400 = no padding: `w` = 32, so "0" leaves 17 px). The transporter word is item `0x8A0` from `0xB4614`/`0xB4CC4` (by count = 1), the destination item `0x8A0` from `0xB4AF4[…]`.
- Font sets differ: `ICONS2` and `ICONS3` `ICONCS.DAT` advances differ for A (15/14), E (13/14), F (10/15), H (15/20), I (8/10), X (11/15) and their 0xC0+ copies. A word-width residue on words with these letters ("Arbeitern") points at the wrong font set [UNVERIFIED as the cause].

Answer: an item is one marker in the line. Its width `w` is fixed at creation from the **caller's style** (`%m` takes style and data as two arguments). For alignment the line measures `Σ(adv+sp)` of glyphs + 6 per space + Σ`w`, with no padding around items. The pen advances by `w` at the marker. The text after it starts at `item.x + w`, and a space before or after an item is an ordinary 6-px space (stretched only on justified wrapped lines). The item painter `0x34BF0` later draws the item's text **left-aligned at the stored item x** with the item's own `sp`. It never re-aligns it inside the slot. Formula: `item.x = d5014 + d5018 + shift + Σ(widths of the entries before it)`, with `shift = d502C − L` for `$R` and `(d502C − L)/2` for `$C`, and 0 if `L ≥ d502C`. Case a): the Optionen value item has style 0x32, a **77-px slot**, so "Ein"/"Aus" both start at screen x 451 (window 100 + 351), and the EXE has no `$8<` here. Case b): style 0x8A0 items are as wide as their text, so Schert is at 37 and Wolle at 115. The reported 4-px shift of Wolle is not explained by the code and needs a re-measurement without the port's +2 right-alignment rule.

## 8. Word wrap in `0x30FB0`

### 8.1 The test

Every character `c` that `print` emits, glyph or space, is first mapped (CP437 umlauts → font codes, `\` → 0x1D). Then, **before** it is appended, `0x30FB0` tests:

`d502C < d6A30 + adv[c]`, or more than 0xF0 entries in the line.

- `d6A30` = the line width so far. Every glyph already in the line counts `adv + sp`, the last one's trailing `sp` included, and every space counts 6.
- The new character counts **`adv[c]` only, without its `sp`**. For a space that is `adv[0x20]` = **5**, not 6.
- The comparison is strict: a character whose right edge lands exactly on `d502C` still fits.

If the test fails and the line has a recorded break point (the index just after the last space or hyphen), the line is cut there and `c` then goes onto the new line. The remembered space becomes a zero-width entry and the line width drops by 6, so a wrapped line's trailing space is neither measured nor drawn. With alignment 4 the remaining free width is spread over the spaces (§4). If there is no break point (a single word wider than the box), the pending entries are flushed as they are with `0x31610` (new line) and `c` starts the next line.

A word therefore ends up on the next line in two cases:
1. One of its own glyphs fails the test: `width before the glyph + adv[glyph] > box`.
2. The **space after it** fails: `width including the word's last sp + 5 > box`. The break point is still the space *before* the word, so the whole word moves down.

At the end of a paragraph (`$N`, `$Z`, end of string) no space follows. The last word then only needs `width − 1 ≤ box`.

### 8.2 What does not change the test

- `$L`, `$R`, `$C`, `$B`, `$N` and `$E` call `0x31610` only when the line holds entries (`0x84D9A` > 0). `0x31610` resets `d6A30 = 0` and the entry count to 0. On an empty line these codes only set `d5022`.
- Alignment never changes `d502C`. Only `$n>`, `$n<`, `$n=`, `$k/m`, `$|`, `$H` and the window open do.
- `%s` without a digit (`0x32C27`) pushes the current format pointer and continues inside the argument string. When it reaches the argument's terminating 0 it pops back (`0x33EFB`). So codes inside the argument act exactly as if they were written in the format.
- In the Austreten format `$Z$L%s…`, the argument's leading `$L` meets an empty line, because `$Z` has just flushed it. It only sets alignment 1 again.

### 8.3 Hyphens (from the same routine)

`-` (0x2D) records a break point **after** itself (it stays on the upper line, width unchanged). `~` (0x7E) is a soft hyphen. If `width + adv['-'] + sp ≤ box`, it records a break point that, if used, turns into a visible `-` (+`adv['-'] + sp`). It is otherwise invisible and has zero width.

[L] With text flag 0x800 (`$(`), hyphens record no break point. An overflow cuts at the last space, appends three `.` glyphs (each `adv['.'] + sp`) and sets flag 0x400, which drops every further character until `$)` clears 0x800 and 0x400 (`$)` = AND 0xF3 on the high flag byte). So `$(`…`$)` is "truncate with an ellipsis", not "keep together" (`formats.md` §8.3 says keep-together [UNVERIFIED]).

### 8.4 The two captures

Both boxes come from the generic text box `0x368E0(x, y, w, h, fmt, args…)`. It opens `0x2FA80(x, y, w, h, 0x8810)` (so box = `w − 24`, text from `x + 12`), prints, adds the `0x1690`/`0xCD` item, and waits in the menu loop.

- **Austreten** (`0x5D034`): window **(140, 120, 360, 240)**, box **336**, text x 152..488. `%s` = `0xB70C0[faith]` (`0xAB6B2` "$LDurch Euren Austritt …"), `%d` = cost. With the rule above, both font sets give: `[EXE text]` / `[EXE text]` / "Kirchenämter versperrt." (274) / `[EXE text]` … At the space after "Wollt" the width is 274 + 6 + 22 + 10 + 7 + 7 + 8 = 334, and 334 + 5 = 339 > 336. Case 2 applies and "Wollt" wraps, as captured. A test without the following space would keep it (the last glyph: 326 + 7 = 333 ≤ 336).
- **Spy** (`0x4711F`, Hinterzimmer `0x45C30`): window **(120, 140, 400, 200)**, box **376**, text x 132..508. `%3N1` = the person, `%d` = the yearly cost. "informieren. Sie verlangen dafür" measures 371 including its last sp (both font sets). The following space tests 371 + 5 = **376**, which is not > 376, so it fits and "dafür" stays, as captured. A rule that adds the space as 6 would get 377 and wrap. (Lines 1–3 depend on the name's width. "informieren" starts line 4 for the captured name.)

Answer: a character is appended to the current line when `line width + adv[c] ≤ d502C`. The line width includes every earlier glyph's `adv + sp` (the last glyph's trailing `sp` too) and 6 per space. The new character counts `adv` without `sp`, so a space counts 5. Otherwise the line is cut at the last space (or after the last `-`/used `~`), and that space is dropped from the wrapped line's width. In practice a word moves down if one of its glyphs overflows, or if the space after it does: `word end (incl. last sp) + 5 > box`. A paragraph's last word only needs `end − 1 ≤ box`. Alignment codes neither change the box nor matter mid-print except as flushes of a non-empty line, which reset the width to 0. An argument's leading `$L` after `$Z` is a no-op. The Austreten box is (140, 120, 360, 240) with box 336. The spy box is (120, 140, 400, 200) with box 376. With these, the rule reproduces both captures: "Wollt" wraps (339 > 336) and "dafür" stays (376 ≤ 376).
