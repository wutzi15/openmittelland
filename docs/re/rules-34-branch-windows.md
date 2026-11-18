# Branch screen sub-windows: Wohnsitz, Lagerraum, Direktor, Transporter

Source: branch screen dispatch `0x50F02..0x512AF` (in `0x4FA70`), residence menu `0x52240`, action preconditions `0x51EB0`, storage `0x531A0`, director hire `0x53680`, director fire `0x53990`, transporters `0x53A60`. Primitives: `win(x, y, w, h, style)` = `0x2FA80`, `print` = `0x322C0` (returns the index of the first item it creates), `draw_icon(x, y, id)` = `0x2E310` (window-relative), `box(x, y, w, h, fmt, …)` = `0x368E0`, `item_range(item, min, max)` = `0x34B00`, `item_set_icon(item, id)` = `0x34B80`, `menu` = `0x35B60`, `items_free` = `0x34AC0`, `clear_page` = `0x30F10`, `cash_add` = `0x43790`, `pay` = `0x43860`, `rec_find(type, owner, branch)` = `0x426B0` / next `0x427B0`, `rec_new(type, owner, branch, counter)` = `0x424A0`. Tags: [C] instruction level, [UNVERIFIED] inferred. Text codes: `rules-12` §1, `formats.md` §8.3; item kinds and the spinner click rule: `rules-16b` §6, `rules-26` §1.

Notation: `b` = branch record (0x60 bytes at `0xDC86C`, index `bi = (b − 0xDC86C)/0x60`), `c` = city (`b+1`), `o` = owner (`b+2`), `r` = residence word (`b+3`), `C` = city record `0x85F04 + 0x8C·c`. Window coordinates are screen pixels; item/text coordinates are window-relative.

## 0. Dispatch from the branch screen [C]

`menu` index on the branch screen:

| index | item (`formats.md` §11.3 bottom row) | action |
|---|---|---|
| 0x24 | residence/building icon (x 38) | `0x52240(b)`; afterwards the item's icon is set to `0x51 + (r & 0x0F)`, or `0x51` without a residence |
| 0x25, 0x26 | `ICON5B` (x 138) and the text next to it | `0x531A0(b)` storage |
| 0x27 | `ICON5C` (x 238) | `b+0x53 == 0xFF` → hire `0x53680(b)`, else fire `0x53990(b)` |
| 0x28..0x30 | transporter icon / name / count, 3 items per type | `0x53A60(b, t)`, `t = (index − 0x28) / 3` (0 Karren, 1 Schiffe, 2 Ballons) |

## 1. Wohnsitz (`0x52240`) [C]

**Window** `win(60, 40, 520, 400, 0x8810)`: parchment with frame tiles, erased on close. Each redraw starts with `clear_page`.

**Text, in order:**
1. `0xBE8E4` `$C^ Wohnsitz _$Z`.
2. With a residence (`r ≠ 0xFFFF`):
   - picture `draw_icon(186, 144, 0x60 + (r & 0x1F))` (ICON60..ICON69, 320×240, one per level Kate..Schloß);
   - if none of the extension bits `r & 0xE0` is set: `0xBDED0` `$N` (one extra line);
   - `0xBE8F8` `%1W in %o$N$N` with (r, c).
   
   Without: `0xBE908` `[EXE text]` with (c). No picture.
3. `0xBE934` `$H$L$N$N$N$N$N$N$Z`: cursor home, left-aligned, 6½ lines down.
4. `0xBE948` `%j Kaufen$Z%j Bauen$Z%j Verkaufen$Z%j Versteigern$Z%j Erweitern$Z%j Renovieren$Z` with icons (0x6A, 0x6E, 0x6F, 0x70, 0x71, 0x72). Each `%j` is an icon item, style `0x1090`, 32×32; the label follows on the same line.

The item list is the same in every state; only lines 2 change.

**Items and actions** (index relative to the first item). Each action first calls the precondition check `0x51EB0(b, flags)`; on failure it shows one box and does nothing.

| idx | label | flags | handler | rules |
|---|---|---|---|---|
| 0 | Kaufen | 3 | `0x52490` | `rules-11` §2 buy, `rules-23` §7.2 |
| 1 | Bauen | 3 | `0x52790` | `rules-11` §2 build |
| 2 | Verkaufen | 7 | `0x52AA0` | `rules-11` §2 sell |
| 3 | Versteigern | 6 | `0x52B50` | `rules-08`, `rules-23` §7.2 |
| 4 | Erweitern | 7 | `0x52D00` | `rules-11` §2 extensions |
| 5 | Renovieren | 7 | `0x53060` | `rules-11` §2 renovate |

Precondition boxes (`0x51EB0`), checked in this order:
- flag 1: a type-14 record (o, bi) exists → `box(160, 140, 320, 200, 0xBE804 $Z$C%1W ist schon so gut wie unter dem Hammer. Eine Änderung ist nicht mehr möglich.$N, r)`.
- flag 2: a type-13 record (o, bi) exists → `box(160, 160, 320, 160, 0xBE864 $Z$CBautrupps bevölkern Euer Gelände in %o.$ZHabt Geduld$Nbis zum Jahr %l.$N, c, year + rec+3)`.
- flag 4: no residence → `box(160, 200, 320, 120, 0xBE8B8 $N$CIhr besitzt keinen Wohnsitz in %o.$N, c)`.

**Loop.** After an action with index < 6: if the action was Kaufen, or the residence word changed, the items are freed and the window redrawn from step 1; otherwise the same items stay and `menu` runs again. Any index ≥ 6 (no item hit) frees the items and closes the window.

## 2. Lagerraum (`0x531A0`) [C]

**Window** `win(60, 100, 520, 320, 0x8910)`. The title's `$P` repaints the frame (`clear_page` → frame painter `0x30880`, which does not test style bit 0x100), so the window shows parchment tiles despite 0x100. [UNVERIFIED: not compared with a capture pixel-for-pixel.]

**Text, in order:**
1. `0xBEE28` `$P$C$Z^ Lagerraum in %o _$N$L` with (c).
2. Two columns, k = 0 (Holz), 1 (Stein). For each: left margin and cursor x = `12 + 248k`, text width 248, cursor y = 72; then `0xBEE48` `%slager: %w m$18c$ZPreis: %w Taler$N$N%j %m m$18c verkaufen$Z` with:
   - `0x87122 + 6k` ("Holz" / "Stein");
   - area `b+0x23+2k` (u16);
   - price per m² `C+0x6F+2k` (u16);
   - icon 0x6F (Verkaufen);
   - spinner style `0x13` (3 digits, shown `000`), data = the column's sell amount (starts 0 each time the window opens).
   `$18c` is the ² glyph.
3. `0xBEE88` `$|$B`.
4. Offers: for each of the 3 slots `v = offers[c][s]` (u16 at `0x85D01 + 6c + 2s`): column `k = 1` and `v −= 10000` if `v > 10000`, else `k = 0`. If `v > 0`: cursor x = `12 + 248k`, y = `192 + 36·row[k]`, then `0xBEE90` `%j %d m$18c kaufen$N` with (icon `0x78 + k`, v); `row[k]++`. If `v = 0`: `0xBDB40` `%M$E` (invisible placeholder, keeps the item numbering).
5. If no offer was printed: `0xBEEA8` `[EXE text]`.

Wood offers stack in the left column under "verkaufen", stone offers in the right column, in slot order. Icons: 0x78 buy wood, 0x79 buy stone.

**Items**: 0 sell-wood icon, 1 wood spinner, 2 sell-stone icon, 3 stone spinner, 4..6 offer slots 0..2. Before every `menu` call the spinner ranges are set to `0 .. min(area_k, 999)`.

**Actions:**
- 0 / 2, sell column `k = idx/2`: `cash_add(o, amount_k · price_k)`, `area_k −= amount_k`, `amount_k = 0`. Full price, same as buying (`rules-02` §5.2).
- 1 / 3: the spinner delta is added to `amount_k`; no redraw.
- 4..6, buy slot `s = idx − 4`, decoded `(k, v)` as above: if `area_k + v > 4999`: `v = 4999 − area_k` and the raw offer word is reduced by v; else the offer word := 0. `area_k += v`; `pay(o, price_k · v)` (no cash check; cash may go negative); then wait until the mouse button is released.
- After 0, 2, 4..6: items freed, text redrawn from step 1 (window stays open). After 1, 3: `menu` again. Index ≥ 7: close.

Offer generation: `rules-02` §5.2. Note `rules-09` §8: the director reads the same offer table without consuming it.

## 3. Direktor

### 3.1 Hire (`0x53680`, no director) [C]

**Candidates** (`rules-11` §7): place = owner's office place byte (`0xE3A8B + 0xF8·o`) if ≥ 10, else c. Three draws `0x43560(place, 0, 0xFF, 0)`; a draw equal to an earlier one becomes 0xFF (empty). `n` = number of non-empty slots.

**Window** `h = ((48n + 39) / 40)·40 + 120` (integer division): n = 0 → 120, 1 → 200, 2 → 240, 3 → 280. `win(80, (480 − h)/2, 480, h, 0x8810)`.

**Text:**
1. `0xBEED8` `$Z$C^ Direktor einstellen _$Z$L`.
2. n = 0: `0xBEEF8` `[EXE text]`. No further items.
3. n > 0: `0xBEF3C` `[EXE text]`; then per slot: candidate → `0xBEF54` `%a$L%1N3 für %d%% des Einsatzes$|` with (p, `director_fee_pct(p)` = `0x458F0`); empty → `0xBDB40` `%M$E`. Then `0xBEF78` `$Z%j Direktor einstellen$N` with icon 0x7A.

`%a` makes the bullet item (style 0x1090, icon 0x73, 18×18); the name line follows it. `%1N3` prints office title and name ("Richter Hermann").

**Items**: 0..2 the three slots (bullet or placeholder), 3 the hire icon.

**Selection.** `sel` starts at 3 (none). Every pass sets each bullet's icon to `0x74` if it is `sel`, else `0x73`, refreshes, waits for the mouse button release, then `menu`:
- idx 0..2: `sel = idx`; clicking the selected one again clears it (`sel = 10`).
- idx 3 with a selection: leave the loop and hire. idx 3 without a selection: ignored.
- any other index: close without hiring.

**Hire**: window closed, then `box(160, 160, 320, 160, 0xBEF94 $Z$CHerzlichen Glückwunsch!$ZIhr konntet %4N1 als Direktor gewinnen.$N, p)`; `b+0x53 = p`, `b+0x54 = 1000`. No fee (`rules-11` §7).

### 3.2 Fire (`0x53990`, director d present) [C]

No own window; one box: `box(80, 120, 480, 240, 0xBEFE0 $Z$C^ Direktor entlassen _$Z$LEure Lagerstätten werden derzeit von %3N5, für %d%% Eures Einsatzes geführt. %1N0 wirtschaftet zur Zeit mit %d Talern.$Z%j %4N1 entlassen$N, d, director_fee_pct(d), d, b+0x54, 0x7B, d)`. Icon 0x7B. Result 0 (the icon) → `box(160, 180, 320, 120, 0xBF094 $N$C%1N1 ist gefeuert.$N, d)` and `b+0x53 = 0xFF`. Anything else: nothing.

## 4. Transporters (`0x53A60(b, t)`) [C]

Kinds `k = t + 3j`: page j = 0 is Karren/Schiffe/Ballons (k 0..2), page j = 1 Dampfkarren/Tauchboote/Flugmaschinen (k 3..5).

**Waterway check.** If `0x85A00[c] == 0` and t = 1: only `box(160, 160, 320, 160, 0xBF0B0 $N$C%o liegt an keiner Wasserstraße. Daher nützen Euch Schiffe hier nichts.$N, c)`, return.

**Counts before the window** (both pages): `inbuild[j]` = sum of byte +5 over type-10 records (o, bi) with +4 = k; `inrepair[j]` = the same over type-11 records.

**Second page available** (`more`) when `b+0x2A+t ≠ 0` (stock of kind t+3) or `inbuild[1] ≠ 0` or `inrepair[1] ≠ 0`.

**Window** `win(100, 20, 440, 440, 0x8810)`. The page starts with j = 0.

**Page text, in order:**
1. Title: with `more` `0xBF100` `$P$C^ %s & %s _$N$N` with (`0xB4954[t]`, `0xB4960[t]`) ("Karren & Dampfkarren", "Schiffe & Tauchboote", "Ballons & Flugmaschinen"); without `0xBF114` `$P$C^ %s _$N$N` with (`0xB4954[t]`). The title stays the same on both pages.
2. Picture `draw_icon(12, 48, pic[k])`, `pic` (u16 stride 4 at `0x87130`) = 0x7C, 0x7E, 0x80, 0x7D, 0x7F, 0x81 for k = 0..5. All 192×192.
3. `0xBF124` `$34>$CIm Lager: %m$N` with (0x402, &`b+0x27+k`). `$34>` = left margin 204, so lines 3..5 form the right column next to the picture.
4. Stock > 0: `0xBF13C` `Zustand: %d%%$Z` with (`b+0x2D+k`); stock 0: `0xBF14C` `Zustand: --$Z`.
5. `0xBF15C` `[EXE text]` with:
   - (0x402, &inbuild[j]), `P_k` = `C+0x73+2k` (the build price per unit, `rules-02` §6);
   - (0x402, &inrepair[j]), `(100 − cond_k)·P_k / 100`;
   - icon 0x6F, spinner style 2, &sell[j], `cond_k·P_k / 300`.
   `$2>` resets the left margin to 12: the action rows span the window below the picture.
6. j = 0: `0xBF1BC` `%j %m Bauauftrag$Z` with (0x82, 2, &build[j]); j = 1: `0xBE5D4` `%M%M$E`.
7. `0xBF1D0` `%j %m Reparatur$Z%j %m Versteigern$N` with (0x83, 2, &repair[j], 0x70, 2, &auction[j]).
8. `more`: item `0x34130(200, 404, 0, 0, 0x41030, "Mehr...")` (`0xBF1F8`, centred text item); otherwise `0xBDB40` `%M$E`.

The `0x402` fields are 2-digit numeric items that show the pointed byte. [UNVERIFIED: flag 0x400 = display-only, by analogy with `0x415` in `rules-13` §1.] The four spinners are style 2 (2 digits, `00`); their amounts start at 0 when the window opens and are kept per page.

**Items**: 0 Im Lager, 1 Im Bau, 2 In Reparatur, 3 sell icon, 4 sell spinner, 5 build icon, 6 build spinner, 7 repair icon, 8 repair spinner, 9 auction icon, 10 auction spinner, 11 Mehr.

**Before every `menu` call**: `sell`, `repair`, `auction` are clamped to the stock `b+0x27+k`; ranges: sell 0..stock, build 0..(99 − inbuild[j]), repair 0..stock, auction 0..stock.

**Actions** (amount n of the current page; each resets its amount to 0):

| idx | effect | rules |
|---|---|---|
| 3 Verkaufen | `stock_k −= n`; `cash_add(o, n·P_k·cond_k / 200)` | `rules-02` §6 |
| 5 Bauauftrag | if n > 0: find a type-10 record (o, bi) with +4 = k and counter (+3) = `BT[k]`, else `rec_new(10, o, bi, BT[k])` with +4 = k, +5 = 0; `+5 += n`; `pay(o, n·P_k)`; `inbuild[j] += n`. `BT` (u8 stride 4 at `0x86758`) = 1, 3, 1, 2, 3, 2 | `rules-02` §6 |
| 7 Reparatur | if n > 0: type-11 record (o, bi) with +4 = k and counter 1, else new one with counter 1; `+5 += n`; `pay(o, n·P_t·(100 − cond_t) / 100)` (t, not k: quirk); `stock_k −= n`; `inrepair[j] += n` | `rules-02` §6 |
| 9 Versteigern | no type-3 record (0xC0, 0xFF) → `box(140, 180, 360, 120, 0xBF200 $N$CDerzeit ist keine Versteigerung angesetzt.$N)`. Else if n > 0: type-12 record (o, bi) with +4 = k, else `rec_new(12, o, bi, 100)` with +4 = k, +5 = 0, +6 = 100. If `+5 + n ≥ 100`: `box(160, 160, 320, 160, 0xBF234 $Z$LDie Auktionsleitung nimmt keine weiteren %8s aus Eurer hiesigen Niederlassung mehr an.$N, 19 + k)`. Else `+6 = (cond_k·n + +6·+5) / (n + +5)`, `+5 += n`, `stock_k −= n` | `rules-08`, `rules-23` §7.3 |
| 4, 6, 8, 10 | add the spinner delta to the amount | |
| 11 Mehr... | `j ^= 1` | |

**Loop.** idx < 11: the items stay, the clamps and ranges are reapplied, `menu` again. The `%d` texts (prices) are not reprinted then; they do not change inside the window. idx = 11: items freed, the page redrawn from step 1 with the other j. idx > 11: items freed, window closed.

**Display quirks [C]:**
- "Verkaufen zu %d Talern" shows `cond·P/300` per unit, but the sale pays `cond·P/200` per unit (at 100 % and P = 100: shows 33, pays 50).
- On page 2 the repair line shows the cost from kind k, but the charge uses kind t's price and condition (`rules-02` §6 quirk).

## 5. Corrections to other docs
- `rules-11` §2 "buy and build are refused while a type-14 (renovation/extension) or type-13 (construction) record exists": type 14 is the **residence auction lot** (box text `[EXE text]`), type 13 the construction ("Bautrupps …"). Versteigern (flags 6) skips the type-14 check: sell, extend and renovate need all three checks; auction needs only "no construction" and "has a residence".
- `formats.md` §11.3 "Sub-actions … transporter purchase, loading and route set-up. Their layouts are not traced": `0x52240` residence, `0x531A0` storage, `0x53680`/`0x53990` director hire/fire, `0x53A60` transporters; layouts above.
- `rules-02` §6 "Total built plus pending is limited to 99 per type": the window's build spinner is limited to `99 − inbuild` (pending only; the stock is not counted).

## 6. Open
- Exact pixel y of the item rows in §1 and §4 (depends on `$Z` line height and the item height); not computed.
- Whether `menu` returns an index ≥ the item count for a right click or a click outside every item (all four windows close on such an index).
- Window placement vs. the live captures: the captured rectangles look like half-scale screen coordinates ((60, 40, 520, 400) → about (30, 20)–(290, 220)). [UNVERIFIED]
