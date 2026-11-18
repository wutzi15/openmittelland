# Trade-map hover outlines and the courtship gift menu (rules-63)

Source: trade map `0x4CC90` (items `0x4CD69..0x4D010`), `menu` `0x35B60` (hover `0x35D28..0x35D37`, repaint rule `0x365A0..0x365DC`, `formats.md` §8.7), item painter `0x34BF0` (kind 0x100 branch `0x353F9`, tail `0x35985..0x35AA6`, backup/restore `0x357D0..0x3595D`), courtship handler `0x212B0` (gift branch `0x212C1..0x2141A`), turn end `0x61A70` (report window `0x62A6B..0x62A8C`, record dispatch `0x63680`, type-30 call `0x638CC`). Tags: **[C]** instruction level, **[L]** decompiler or partial trace, **[UNVERIFIED]** inferred. Everything is [C] unless marked.

## 1. Trade map: what hover draws

### 1.1 Items per city c = 0..9 (`0x4CD69..0x4D010`)

| order | condition | `0x34130(x, y, w, h, style, data)` | kind |
|---|---|---|---|
| first loop | `0x85E08[c] ≠ 0xFF` | at P3 (city +10), 40×20, style `0x510 | colour(owner)` (colour = person +0xF4 of `0x85E08[c]`), data `0x196` (ICONP6) | 0x110 animated owner flag |
| | else | `print("%M$E")` placeholder item | |
| second loop | all c | at P2 (city +6), w = h = 0 (c = 6 Falkenhorst: 50×40), style `0x100`, data **`0x120 + c`** | 0x100 icon button |
| | all c | at P1 (city +2), w = h = 0, style `0x100`, data **`0x135 + c`** | 0x100 icon button |

`0x120..0x129` are ICONI0..I9, `0x135..0x13E` are ICONJ5..JE: the "yellow outline masks" and "ribbon/label outlines" of `formats.md` §1.7. With w = h = 0 the item takes the icon's own size from the ICONIN tables (`rules-16b` §6), so the click/hover rectangle is the outline's bounding box with its top-left at P2 (town) or P1 (label). The own-branch flag item is the only thing that differs for a city with the player's branch; its hover does nothing visible (§1.3).

### 1.2 Painter rule for kind 0x100 (`0x353F9`, `0x35985`)

- An **unlit** kind-0x100 item draws nothing at all: the branch falls through to the common tail, which only handles the background backup.
- A **lit** item (byte +9 bit 0x40, set by `menu` for the hovered item) is remembered (`[EBP−0x28] := index`, last one wins [L: the initial value is "none"]). After the item loop the painter calls `draw_icon(item.x, item.y, data)` for that one item, with the highlight colour bit `0xD5025 & 0x80` turned off during the blit (`0x359A4..0x359DE`), and presents the item rectangle (`0x13539`). So the outline is blitted as stored (yellow), top-left at the item position, on top of everything else.
- Every repaint first restores the item's saved background when it has one (`+9 & 0x08`, allocated on the first paint for styles without flag 0x1000; `0x357D0..0x3595D`). Style `0x100` has no 0x1000, so each town and label item owns a backup, and repainting it unlit erases its outline.

### 1.3 Behaviour

- Hover over the town drawing (item at P2) lights **ICONI0+c** at P2; hover over the name ribbon (item at P1) lights **ICONJ5+c** at P1. They are separate items; one hover never lights both. The first item in creation order under the pointer wins, so where a flag item (P3) overlaps a town outline the flag wins and nothing lights (kind 0x110 ignores the lit bit: `0x356F3` draws its frame regardless).
- Lighting happens only when the hovered index changes (`formats.md` §8.7): the old item loses bits 0x40|0x80, all items repaint (backgrounds restored, the newly lit outline drawn once), `0x84D4C` updated. Moving within one item redraws nothing.
- Removal: on the next hover change (restore + repaint), or when the screen is rebuilt. A click returns from `menu` with the outline still on screen (only bit 0x80 is cleared at `0x366A1`); the branch screen that opens replaces it.
- No outline for castles/camps: the map creates no items for them; the swords markers (`0x1B5`/`0x1B6`) are plain `draw_icon`s.

Answer: hovering a town lights ICONI0+c (town outline) at P2, hovering its name lights ICONJ5+c (ribbon outline) at P1, drawn by the item painter only while that item is the hovered one and erased from the saved background on the next hover change. Own-branch cities differ only by the animated flag item at P3, which never lights. Our port must create the town/label items as kind-0x100 icon buttons whose data is the outline id and draw that icon only for the lit item.

## 2. Courtship gift menu

### 2.1 Window and position

- The human's turn end (`0x61A70`) runs `settlement_pre`, then opens the full-screen backdrop (`0x8100`), draws ICONB7, the banner "Ereignisse" (`0x49D60`), and **`win_open(30, 110, 520, 290, 0xA810)`** (`0x62A84`), the report text window. Every report line is `print`ed into it; the window scrolls as `rules-42` §1 describes (a wait for a click once a window of unread text has built up, then a 24 px scroll; `$V`/`$W` waits where the texts have them).
- Order inside the window: `^ Eure Ausgaben _` block (`0x62AA1..0x62E90`), confiscation / title income / toll income lines (`0x62FC1..0x63190`), the Gehilfe tips (`0x6332D..0x63512`), blackmail expiry (`0x635CE`), **then the record pass** (jump table `0x63680`, records in heap order). A type-30 record of the player calls `0x212B0(record)` at `0x638CC`. So the gift menu appears inside the Ereignisse window, after the tips, at the heap position of the courtship record relative to the other record-driven lines (births, deaths, loans, sub-plots). There is no dedicated window, caption or message box.

### 2.2 The menu (`0x212C1..0x2141A`, counter `+3 > 0`)

1. Prices `P[i] = standing(target, 25)` ×3, then `rand(6)`, `rand(7)`, `rand(7)` (`rules-56` §1).
2. `print(0x963C3)`: `[EXE text]`. The four `%a` lines are ordinary menu items of the text window (`%a` = `$L$2>%m$E$B$6>` with bullet icon `0x73`, `rules-47` §1), left-aligned, one per line, as part of the scrolled text. `print` returns the first item index.
3. `choose(base)` (`0x36850`): flips, waits for a click on one of the four items (the bullet of the hovered item swaps to `0x74`, `0x35424..0x3547C`), removes the items, returns 0..3 (right click: 4).
4. Result 0..2: `print(0x96657 "$Z$L%1N0 ist %s.$N$N", target, 0xB5818[rand(15)])`, `pay(player, P[result])`, `record+4 += P[result]`. The reaction line follows directly under the menu text in the same window.
5. Result ≥ 3 (`[EXE text]` or right click): `print("$N")` only. No text, no payment, no favour change.
Then control returns to the record pass, which continues printing into the same window.

### 2.3 Geometry

Window `(30, 110, 520, 290)` style `0xA810` (frameless text window on the ICONB7 desk; the banner is above it). Text margins are the window's `$H`/`$|` defaults of `rules-47` §2; the menu lines are indented by `$2>` (12 px) with the bullet at the margin. Whether the menu starts on a fresh page depends only on how much text precedes it: the print engine never pages for a menu, it scrolls when the cursor passes the bottom and waits for a click before scrolling if a full window of unread text has built up.

Answer: the gift question is printed into the turn-end Ereignisse window `win_open(30, 110, 520, 290, 0xA810)` as four `%a` menu lines, in heap order of the type-30 record after the expense list, income lines and tips; `choose` waits for the click; the reaction `%1N0 ist <word>.` or a bare `$N` prints right below it in the same window. No separate window, no caption, no message box, no forced page.
