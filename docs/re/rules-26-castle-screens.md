# Castle and camp screens (rules-26)

Sources: map screen `0x54570`, info window `0x549C0`, castle screen `0x55090`, expansion `0x56080`, recruiting `0x56760`, manoeuvre `0x56950`, toll rate `0x56CF0`, camp screen `0x56E20` (its order editor starts at `0x579A1`), order refresh `0x45820`, item creation `0x34130`, item limits `0x34B00`, `menu` `0x35B60`. Tags: [C] instruction level, [L] decompiler only, [UNVERIFIED] inferred. Live check: screenshots `cs2`/`cs5`/`cs10` (info window owner/foreign, sell list) match §2 and rules-23 §4.2.

Conventions used below:
- `print` = `0x322C0`. It returns the index of the **first item it will create**, which is the item count before the call. Menu results are always "clicked item − that base".
- `win(x, y, w, h, style)` = `win_open` (rules-16b). All coordinates inside a window are window-relative. `pic(x, y, id)` = `0x2E310`, draws icon `id` at window-relative (x, y).
- `box(x, y, w, h, fmt, …)` = message box `0x368E0` (rules-15 §0.3).
- `OK` = `0x348D0`: an item of style `0x1690` with icon **0xCD** at (w − 32, h − 32). ICONCD is a **1×1** sprite, so this item is invisible and practically unclickable. These windows are closed with the **right mouse button**: `menu` returns "last item + 1" on a right click (0x35B60, `local_18 == 2`), which falls past every handled case.
- Icon names: id < 0x100 → `ICONxx` (hex), id ≥ 0x100 → two base-36 digits (0x12E = `ICONIE`, 0x143 = `ICONK3`, 0x199 = `ICONP9`). Sizes below were measured with `icon_dump`.
- Item kinds and the click rule of numeric items (rules-16b §6, completed here from 0x35B60 [C]): for kinds 0x00/0x10/0x20/0x40/0x50/0x60/0xC0/0xD0 a click in the **upper half** adds, the **lower half** subtracts; the step is `10^d`, d = digit position counted from the right end of the item (table `0x84DA4` = 0, 1, 10, 100, …). The result is clamped to the item's [min, max] (+0x1C, +0x20, set by `0x34B00(item, min, max)`; default min 0, max `10^digits − 1` for kinds 0x00/0x10, 255/65535 when the nibble is ≥ 3/5). `menu` writes the clamped delta to the caller's variable; **the caller applies it**. Kind 0xA0 (cycling string): upper half delta +1, lower half −1. Kind 0xE0 (hotspot, style `0x12E0`): `menu` walks back to the preceding item and reports **its** index, so a hotspot reports its label (this answers the open question in `formats.md` §9.2).

## 1. Entry from the "Söldner & Räuber" map (0x54570) [C]

Map layout is `formats.md` §10.7. Item indices relative to the banner base (`0x49D60`): 0 gold, 1 banner, 2/3 placeholders, 4..10 the seven owner flags (or `%M` placeholders), then per place k = 0..6 (records `0x8647C + 26k`, rules-07 §1; k 0..2 castles Hohenfels/Seefels/Waldenfels, 3..6 camps Meucheltann/Henkerswald/Dunkelforst/Säbeltal):

| index | item | position |
|---|---|---|
| 11 + 2k | icon `0x12E + k` (ICONIE..ICONJ4, the castle/camp drawing, e.g. ICONIE 55×59), style `0x100` | record +6, +8 (second point) |
| 12 + 2k | icon `0x143 + k` (ICONK3..ICONK9, the name label, e.g. ICONK3 127×38), style `0x100` | record +2, +4 (first point) |

Click handling, i = index − 11, k = i / 2:
- **Label** (odd i): always the info window `0x549C0(player, k)`.
- **Drawing** (even i): look up the property of kind **2** (k < 3) or **1** (k ≥ 3) at location `14 + k` whose owner is the current player (`0x42A50(kind, 14 + k, player)`). Found → castle screen `0x55090(property)` (k < 3) or camp screen `0x56E20(property)` (k ≥ 3). Not found → info window.
- i ≥ 14 → turn phase byte `0xEF4C5 := 4` (leave).

After the castle/camp screen returns, the pointer is reset (`0x2EC40(0)`) and the map is redrawn. The info window returns 1 if the castle changed owner; the map is then redrawn too. The screen's return value (non-zero = the options screen ended the game/loaded) is passed up.

## 2. Info window (0x549C0(player, k)) [C]

Data: static record R = `0x8647C + 26k`; property P = first property at location `14 + k` of any kind and owner; `value = net_worth_goods(P)` (`0x44340`, formulas rules-07 §1); `lot` = type-16 record (owner = viewer, key = P's index); `auction` = type-3 world record (owner 0xC0). Sound: `0x41E50(0x1A, 2)` for k < 3, `0x41E50(6, 2)` for camps (rules-14).

**Window** `win(80, 20, 480, 440, 0x8810)`: parchment with frame tiles; on screen x 80..560, y 20..460. Opened once; every pass of the loop below deletes the items (`0x34AC0`), clears the window (`0x30F10`: margins and cursor reset, frame repainted) and redraws.

**Per pass:**
1. Interest pool roll: rules-24 §3 (runs on every pass, also for the owner).
2. Picture, then the text block:

| place | picture | text (`print`) | args |
|---|---|---|---|
| castle (k < 3) | `pic(80, 102, 0x8B + k)` (ICON8B/8C/8D, 320×240) | `0xBF2B8` `$C^ Information Burg %o _$ZDiese Burg gehört %3N5.$N$14H$Z$/Wert: %d Taler$E$1/Wege-Zollsatz: %d%%$|$Z` | location (P+1), owner (P+2), value, toll rate R+0x14 |
| camp (k ≥ 3) | `pic(80, 102, 0x8E)` (ICON8E, forest, 320×240, same for all four camps) | `0xBF36C` `[EXE text]` | location, owner, value |

   Result on screen: ornamented title on line 0; the owner line under it (wraps when long, as in `cs2`); the picture at screen (160, 122)..(480, 362); `$14H$Z` puts the value line at window y = 12 + 14·24 + 12 = 360. Castles split that line into two half-width columns (`$/` column 0 = "Wert: …", `$1/` column 1 = "Wege-Zollsatz: …%"); camps centre "Wert: …" only. There is **no separate string for computer owners**: the line depends only on castle vs camp; `%3N5` renders any owner (name with rank/office, rules-12). An unowned place (owner 0xFF) is [UNVERIFIED].
3. Action line (32×32 icon items; `%M` = invisible placeholder that keeps the indices fixed):

| viewer | place | text | items 0..3 |
|---|---|---|---|
| owner | castle, no lot | `0xBF320` `$/%j Verkaufen$E$1/%j%M%M Versteigern$|` (icons **0x6F**, **0x70**) | 0 Verkaufen (ICON6F, hand with coin), 1 Versteigern (ICON70, gavel), 2, 3 placeholders |
| owner | castle, lot exists | `0xBF348` `%M%M%M%M%s wird %l versteigert.$N` (castle name `T[R+0]`, year = `auction+3 + year`) | none active |
| owner | camp | `0xBF3B0` `%j%M%M%M Verkaufen$N` (icon **0x30**) | 0 Verkaufen |
| other | any, no own interest, < 6 interested | `0xBF3C8` `%M%M%j%M Kaufinteresse bekunden$|` (icon **0x6A**, coin stack) | 2 |
| other | any, 6 interested | `0xBF3EC` `%M%M%M%M$E` | none |
| other | any, own interest record exists | `0xBF3F8` `%M%M%M%j Kaufinteresse zurückziehen$|` (icon 0x6A) | 3 |

   Ownership test: `P+2 == player`. Then `OK` (item 4) and `flip`.
4. `menu`; result r:
   - **0 Verkaufen**: rules-23 §4.2 (box `(140, 180, 360, 120)` when nobody is interested, else the "Verkaufen" window `(140, 140, 360, 200)` with one `%a` bullet per interested person and "Niemandem"; confirmation box for a human buyer; sale). After a sale the function will return 1.
   - **1 Versteigern**: no type-3 auction record → `box(140, 180, 360, 120, 0xBF4E4 $Z$CDerzeit ist keine Versteigerung angesetzt.$N)`. Otherwise `box(140, 140, 360, 200, 0xBF51C $Z$C^ Versteigern _$Z$LDie nächste Auktion findet %l statt. Wollt Ihr %s versteigern lassen?$Z%j Ja!$Z%j Nein.$N, auction+3 + year, castle name, 0x70, 0x70)`; answer 0 ("Ja!") → `rec_alloc(16, player, P index, 100)` (the lot; auctions are rules-08). Both icons are the gavel.
   - **2 Kaufinteresse bekunden**: `rec_alloc(17, player, P index, 100)`. No message.
   - **3 Kaufinteresse zurückziehen**: the viewer's type-17 record is deleted, heap compacted (`0x42550`, `0x42580`). No message.
   - r ≥ 4 (OK or right click): close (`win_close`) and return.
   
   After 0..3 the loop repeats (step 1), so the window stays open and shows the new state.

Note: `Zollburg versteigern` (`0x565B0`, strings `0xBF8E0`..`0xBF9C0`, cited in rules-23 §4.2) has **no caller and no pointer reference** in the image; it is dead code. The live auction route is the `0xBF51C` question above.

## 3. Castle screen (0x55090(P)) [C]

Reached only by the owner (§1). Full screen.

### 3.1 Static layer (drawn once)
1. `0x45820(P)`: order refresh (rules-02: confirmed orders re-checked; infeasible → status 0, else 1 unless 3).
2. `win(0, 0, 640, 480, 0x8500)`; `pic(0, 0, 0x101)` = **ICONG1** (640×480: four figures, left to right Söldner, Musketier, Kanonier, Offizier).
3. Banner `0x49D60(owner P+2, title = location name 0xB6194[P+1], 0)`: "Hohenfels, A.D. 1600" right, owner name centred. Returns base B; all indices below are relative to B.
4. `print("$g")`, then for k = 0..3 set the text cursor and left margin to (`0x87148[k]`, `0x87158[k]`) = **(60, 76), (184, 76), (320, 76), (464, 76)** and print `$L$g%s$N` with the troop name `0xB6BE0[k]` = Söldner, Musketiere, Kanoniere, Offiziere (gold outlined text over the picture).
5. Items:

| index | style | data | x, y, w, h | action |
|---|---|---|---|---|
| 0 | gold display | | banner | `0x4F370(owner)` (finances) |
| 1 | banner | | | options `0x4D1E0`; a non-zero result leaves the screen and is returned |
| 2 | placeholder | | | nothing |
| 3 | placeholder | | | leave |
| 4 | `0x1091` icon | **0x87** (castle with scaffold) | 32, 408 (64×64) | expansion `0x56080` (§5.1) |
| 5 | `0x1091` | **0x88** (tavern sign) | 160, 408 | recruiting `0x56760` (§5.2) |
| 6 | `0x1091` | **0x89** (cannon) | 288, 408 | manoeuvre `0x56950` (§5.3) |
| 7 | `0x1091` | **0x8A** (scales) | 416, 408 | toll rate `0x56CF0` (§5.4) |
| 8 | `0x1091` | **0x5C** (seal ring) | 544, 408 | toggle the order book (§3.3) |
| 9, 11, 13, 15 | `0x404F0` label "Info" (`0xBF5A0`), centred (flag 0x40000), size from text | | x = **103, 246, 369, 515**, y = 180 | morale box, slot (index − 9)/2 |
| 10, 12, 14, 16 | `0x12E0` hotspot (reports the label before it) | | (43, 110, 120, 170), (186, 110, 120, 170), (309, 110, 120, 170), (455, 110, 120, 170) | same |

6. Bottom strip buffers: two 640×80 off-screen copies of y = 320..400: S0 = the picture as is; S1 = the same area after `win(0, 320, 640, 80, 0x8010)` painted the parchment frame tiles over it (that window is closed again at once). S1 is the order-book background.
7. Sound `0x41E50(0x1A, 2)`.

### 3.2 Per pass
1. `0x45820(P)` again, `print("$g")`.
2. Troop counts, items 0x11..0x14 (k = 0..3): style **`0x80803`** (byte number, 3 digits), data `&P[0x58 + k]`, at x = `0x87168[k]` = **87, 230, 353, 499**, y = **370** with the order book hidden, **290** with it shown (`0x87178[k] + 80 − 80·toggle`). Limits `(0, P[0x58 + k])`.
3. Order book, toggle byte = static record **R+0x19** (`0x86495 + 26(loc − 14)`; initial value 0 for castles, 1 for camps, rules-07 §1):
   - **Hidden** (0): S0 copied back to y = 320; `print("$H$L$t$|")` then twice `%M%M%M%M%M%M%M%M$|` (16 invisible items, indices stay fixed).
   - **Shown** (1): S1 copied to y = 320; text cursor y := **338**; `print("$L$t$|")` (dark text); then one line per order slot s = 0, 1 (`P+0x33 + 8s`, layout rules-07 §3a) as in §3.4.
4. First pass: `flip`. Later passes: only the strip y = 320..400 is re-presented (the rest of the screen is unchanged); the last clicked order item keeps its hover flag.
5. `menu` with a delta variable; dispatch on r = clicked − B:
   - 0..8 as in the table above; 8 flips R+0x19 and marks the four troop-count items moved (they are recreated at the other y).
   - 9..16: `box(140, 180, 360, 120, 0xBF6DC $N$CDie Moral Eurer %s ist %s.$N, troop name 0xB6BE0[s], 0xB6814[P[0x5C + s] / 11])`, s = (r − 9)/2; the morale words are erbärmlich, erschreckend, schlecht, mäßig, durchschnittlich, akzeptabel, gut, ausgezeichnet, vorbildlich, überragend (training 0..100 / 11). This is the slot's own training, not the weighted average of §5.3.
   - 0x11..0x14: `P[0x58 + (r − 0x11)] += delta`. With max = current count only the **lower half** has an effect: it **dismisses** 1/10/100 men (clamped at 0), immediately, without confirmation or refund [L for intent; mechanics C].
   - 0x15..0x24: order editor (§3.4), s = (r − 0x15)/8, j = (r − 0x15) mod 8.
   - r = 3, r ≥ 0x25 (right click), phase byte `0xEF4C5 ≠ 2`, or a non-zero options result: free the two buffers, `0x49EB0` (banner cleanup), `win_close`, return.

### 3.3 Order line layout (shown view)
Every line creates exactly 8 items (j = 0..7). Style `0x8A0` = cycling string (kind 0xA0); `0x802` = 2-digit byte number; `0x1830` = text item whose data is a 2-byte string from `0x86770 + 2·status`: status 0 → glyph **0x0D**, 1 → **0x0E**, 2 → **0x0F** (the radio/checkbox glyphs, `formats.md` font map), 3 → empty string (nothing shown).

| type | text | items |
|---|---|---|
| 0x10, or 0 (0 is first rewritten to 0x10) | `0xBF600` `$C%m%M%M%M%M%M%M%M$|$L` with `(0x8A0, 0xBF5B0)`: "Kein Auftrag" padded with 32 spaces on each side, centred | j = 0 only |
| 0x11 | `0xBF618` `$R$25<%m%M %m mit %m+%m+%m+%m$E$2<$L$82>%s$E$R%m$|$L` with `(0x8A0, "Überfallt" 0xB6504[0])`, `(0x8A0, name T[0x8647C + 26·target])`, four `(0x802, &slot[3..6])`, noun, `(0x1830, status glyph)` | 0, 2..7 |
| 0x12 | `0xBF650` (same without "mit"), word "Schickt" `0xB6504[1]` | 0, 2..7 |
| 0x13 | `0xBF684` (as 0x11), word "Überwacht" `0xB6504[2]`, target = county name `0xB61BC[target]` (Mooringen, Bergenhöh, Auenthal) | 0, 2..7 |
| other | nothing printed (items missing; does not occur) | |

Noun after the numbers: if the four counts sum to exactly 1, the singular, else the plural; 0x11/0x13 dative (`T[247]` "Söldner" / `T[675]` "Söldnern"), 0x12 accusative (`T[354]` / `T[782]`, both "Söldner"). The noun is always "Söldner" whatever the slot mix.

Geometry: the left part is right-aligned against a right margin of 25·6 = 150 px ("Überfallt Seefels mit 5+0+0+1"), the noun starts at x = 82·6 = 492, the status glyph is right-aligned at the 12-px margin. Slot 0 is on the line at y = 338, slot 1 on the next line (y = 362) [L: line advance of `$|` inferred from usage].

After each line, troop spinners j = 3..6 get limits `(0, P[0x58 + k] + (status == 2 ? slot[3 + k] : 0))`.

### 3.4 Order editor (r = 0x15 + 8s + j) [C]
Let O = slot s. Rules for feasibility and reservation are rules-07 §3a (`0x45360` feasible, `0x49370` reserve, `0x49430` unreserve).
- **j = 0, order word** (delta ±1): if O.status = 2, unreserve. Then up to 4 times: `t = ((t − 0x0C) + delta) mod 4 + 0x10` (forward on the upper half, backward on the lower half); stop at 0x10; if the garrison is empty keep cycling (so only "Kein Auftrag" is reachable); 0x11 is skipped when the owner already owns 7 castles/camps (all of them). Store t. Target fix-up: 0x11/0x12 → target ≥ 7 becomes 0; if it is this castle, `+1 mod 7`; for 0x11 additionally step `+1 mod 7` while the owner owns that place (≤ 7 steps). 0x13 → target > 2 becomes 0. Status: if it was 2 → still feasible ? re-reserve (stays 2) : 0; else status := feasible ? 1 : 0. Waits for the mouse button to be released.
- **j = 1**: placeholder, nothing.
- **j = 2, target** (delta ±1): 0x13 → `target = (target + 3 − delta) mod 3` (note the **reversed** direction). Others: up to 7 times `target = (target + delta + 7) mod 7` until target ≠ this castle and (type ≠ 0x11 or the owner does not own it). For 0x12 ("Schickt") **no** ownership filter applies: any other castle or camp can be chosen. Correction to rules-07 §3a, which says [L] "among indices where the player owns a castle or camp". Status is not re-checked here.
- **j = 3..6, troop numbers**: if status 2, unreserve and set 1; `slot[j] += delta` (clamped by the limits of §3.3); if it was 2, reserve again and set 2. Feasibility is not re-checked; a total of 0 is caught by the next pass's `0x45820` refresh (status 0).
- **j = 7, status glyph**: 1 → reserve, 2. 2 → unreserve, status := feasible ? 1 : 0. 0 and 3 → nothing. Waits for release.

## 4. Camp screen (0x56E20(P)) [C]

Same structure as §3 with these differences:
- Background `pic(0, 0, 0x100)` = **ICONG0** (640×480). Sound `0x41E50(6, 2)`.
- **Display order** of the four slots: position i shows slot `perm[i]`, `perm = 0x8721C` = **0, 3, 2, 1** (Räuber, Bombenleger, Kanoniere, Schützen from left to right). Names from `0xB6BF0` = Räuber, Schützen, Kanoniere, Bombenleger. The permutation is used everywhere: labels, counts, Info boxes and the four numbers of an order line.
- Name labels at (`0x871DC`, `0x871EC`) = **(35, 76), (154, 76), (320, 76), (484, 76)**, text `T_camp[perm[i]]`.
- Four icons (no toll button), style `0x1091`, y = 408: **0xA5** at x 48 (expansion), **0x88** at 208 (recruiting), **0xA6** at 368 (manoeuvre), **0x5C** at 528 (toggle).
- "Info" labels at x = **78, 221, 379, 535**, y = 180; hotspots identical to the castle (43/186/309/455, 110, 120, 170).
- Item indices: 4 expansion, 5 recruiting, 6 manoeuvre, 7 toggle, 8..15 Info pairs (position (r − 8)/2), **0x10..0x13** troop counts (position i, data `&P[0x58 + perm[i]]`, x = `0x871FC[i]` = **62, 205, 363, 519**, y = 290 + 80·(1 − toggle)), **0x14..0x23** order items; leave on r = 3 or r ≥ 0x24.
- Morale box `box(160, 180, 320, 120, 0xBFC8C …)` (narrower than the castle's) with `T_camp[perm[p]]` and training of slot `perm[p]` (`0x8722C` = same permutation).
- Order types 0x20..0x23: "Kein Auftrag" `0xBFB98`, words `0xB6514` "Überfallt", `0xB6518` "Schickt", `0xB651C` "Plündert" (county target, with "mit"); formats `0xBFBE8`, `0xBFC20`, `0xBFC54` (identical texts to §3.3); noun "Räuber"/"Räubern" (`T[246]`, `T[674]`, `T[353]`, `T[781]`). The four numbers are printed in display order (slot 0, 3, 2, 1); position q edits slot `perm[q]` (`0x87210 + 4j`), limits likewise.
- Order cycling `t = ((t − 0x1C) + delta) mod 4 + 0x20`; 0x21 plays the role of 0x11 and 0x23 of 0x13 in every rule of §3.4.
- The toggle starts at 0 (order book hidden): R+0x19 = 0 for camps in the image (rules-00 §10; the byte read as 1 there is +0x15). Live A/B: the camp screen opens without the book.

## 5. Sub-windows (all `win(100, 20, 440, 440, 0x8810)`, picture at window (60, 92) = screen (160, 112), 320×240) [C]

### 5.1 Expansion (0x56080)
- Castle: `pic(60, 92, 0x7D + location)` (= ICON8B..8D, as the info window); title `0xBF700` `$Z$C^ Zustand Burg %o _$Z`. Camp: `pic(60, 92, 0x8E)`; title `0xBF71C` `$Z$C^ Zustand im %o _$Z`.
- Second line: castle with no goods in stock → `0xBF748` `Baujahr:%l$N` (R+0x12); otherwise `0xBF734` `Waren vor Ort: %d$N` (sum of the 15 stock words).
- Item 0: a live text item, style `0x204B0`, at (0, 354, 440, 24), showing `0x87188` `$C$/Sicherheit: %d%%$E$1/Kasematten: %d$|` (camp `0x871B2` `…Tarnung: %d%%…Strohmatten: %d…`) with P+3, P+4.
- `$15H$Z` line (y = 384): castle `0xBF758` `$/%j Sicherheit$E$1/%j Erweitern$|` (icons **0x8F**, **0x90**); camp `0xBF784` `$/%j Tarnung$E$1/%j Erweitern$|` (icons **0xA7**, 0x90). Items 1, 2; OK item 3.
- Item 1 → `win(120, 160, 400, 160, 0x8910)` (the `$P` in the text repaints it as parchment); text `0xBF84C` `$P$C^ Sicherheit verbessern _$N$N%j %m%% verbessern$N$NKosten: %d Taler$N` (camp `0xBF898` "Tarnung verbessern") with icon 0x8F/0xA7, spinner style `0x12` (2 digits) on amount a, cost c. Spinner limits `(0, 100 − P+3)`. Loop: spinner click → `a += delta`, `c = value(P with P+3 += a) − value(P)`; icon click → `P+3 += a`, `cash_pay(owner, c)` (`0x43860`, no affordability check), a = c = 0, item 0 redrawn; OK/right click → close.
- Item 2 → same window, `0xBF7AC` `$P$C^ Zollburg erweitern _$N$N%j %m Kasematten bauen$N$NKosten: %d Taler$N` (camp `0xBF7F8` `^ Räuberlager erweitern _ … %m Strohmatten stopfen …`), icon **0x90**, spinner style `0x13` (3 digits), limits `(0, 150 − P+4)`, applies to P+4.
- r ≥ 3 closes. Prices are rules-07 §2.1.

### 5.2 Recruiting (0x56760)
- Castle title `0xBFA2C` `$P$H$Z$C^ Söldner anwerben _$Z`, `pic(60, 92, 0x91)`; camp `0xBFA4C` `$P$H$Z$C^ Räuber dingen _$Z`, `pic(60, 92, 0xA8)`.
- `0xBFA68` `Werbegelder: %d Taler$N$15H%j %m Taler investieren$N` with (already invested this turn = type-15 record +4, or 0), icon **0x6A**, spinner style `0x14` (u16, 4 digits) on amount a. Items: 0 icon, 1 spinner, 2 OK.
- Spinner click → `a += delta` (window not redrawn). Icon click with a > 0 → `cash_pay(owner, a)`; create `rec_alloc(15, owner, P index, 1)` with +4 = 0 if missing; `+4 += a`; redraw. Settlement is rules-07 §2.2.
- Quirk [C]: the code calls `0x34B00(base + 2, 0, 9999 − invested)`, which is the invisible OK item, not the spinner. The spinner keeps its default range 0..9999, so **each** investment may be up to 9999 and the per-turn total is not capped. Correction to rules-07 §2.2 ("total per castle up to 9999 per turn").

### 5.3 Manoeuvre (0x56950)
- On opening: picture id `0x199 + rand(3)` (ICONP9/PA/PB, one RNG draw per opening, before anything else).
- Per pass: `$Z$C^ Manöver _$Z` (`0xBFAA0`); morale (confirmed orders 0x11..0x13 / 0x21..0x23 unreserved for the computation and reserved again) `0xBFAB4` `Die Moral ist %s.$N` with `0xB6814[avg / 11]`, avg = Σ count·training / Σ count, 50 without troops; picture; `0xBFAC8` `[EXE text]` with cost `15·troops + (year·277) mod 20`, icon **0x92**. Items: 0 icon, 1 OK.
- Icon: m = 15 if the owner has privilege 37 or 66 (`0x5E600`), else 1; `cash_pay(owner, cost)` (no check); `k = rand(4)`, `training[k] += 5 + rand(10) + rand(m)`, cap 100. Box `(140, 180, 360, 120)`: castle (location < 17) `0xBFB04` `[EXE text]` with `0xB6BE0[k]`; camp `0xBFB34` `[EXE text]` with `0xB6BF0[k]`. Window cleared, loop. Works with 0 troops too (pays `(year·277) mod 20`, trains an empty slot).
- RNG order per manoeuvre: rand(4), rand(10), rand(m).

### 5.4 Toll rate (0x56CF0, castles only)
- Per pass: window cleared; `pic(60, 92, 0x93)` (ICON93, 320×240), then the overlay `pic(60, 92, 0xA2 − rate)` (ICON94..ICONA2, 275/276×240; rate 14 → ICON94, rate 0 → ICONA2), so the picture changes with the rate.
- `0xBFB64` `$Z$C^ Zollsatz Burg %o _$N$15H$CZollsatz: %m%%$N` with (location, style `0x02`, data &R+0x14): a 2-digit byte spinner, limits `(0, 14)`. Item 0 spinner, 1 OK.
- Spinner click → `R+0x14 += delta`, redraw; OK/right click → close. The rate is a static-table byte shared by whoever owns the castle.

## 6. Implementer notes
- The present `Menu::Castle` window has no counterpart. Build: info window (§2) from the label and from a foreign or computer place's drawing; full-screen castle/camp screen (§3, §4) from an owned place's drawing; four/three sub-windows (§5) from the icon row.
- Everything closes with the right mouse button; the OK items are invisible.
- Persistence of R+0x14 and R+0x19 (static table bytes) across save/load is [UNVERIFIED]; check whether the save block covers `0x8647C`.

## 7. Order book toggle and clicks after toggling [C]

Sources: camp loop `0x571B1`..`0x57E2B`, castle loop `0x5541E`..`0x56041`, `menu` `0x35B60`, item create `0x34130`, item pop `0x34930`, delete-from `0x34AC0`, mouse poll `0x12DE4` (INT 33h AX=3, current state, no latched presses), clock `0x6AC50` (C `clock()`, 1/100 s; wraps at 8 640 000).

### 7.1 Item list per pass (identical before and after a toggle)
Item records are 0x24 bytes at `0xD5030`; `0x84D48` holds the index of the **last** item, and `0xD6A00` is a working copy of that last record. `0x34AC0(n)` pops until the last index is below n; a pop restores the item's saved background.

Every pass rebuilds the same index layout. Only the geometry of the order items depends on the toggle byte. Indices are relative to the banner base B.

| index (camp) | index (castle) | item | position, book hidden → shown |
|---|---|---|---|
| 0..3 | 0..3 | banner, gold, placeholders | unchanged |
| 4..7 | 4..8 | icons; **toggle = 7** (camp, ICON5C at 528,408) / **8** (castle, at 544,408) | unchanged |
| 8..15 | 9..16 | Info label / hotspot pairs | unchanged (hotspots y 110..280) |
| 0x10..0x13 | 0x11..0x14 | troop counts | y 370 → 290 (x unchanged; table `0x8720C`/`0x87178` = 290, + 80 when hidden) |
| 0x14..0x23 | 0x15..0x24 | order items, 8 per slot | hidden: 16 zero-width `%M` (`$H` puts them on window lines 0 and 1, y 12/36; width 0, so they can never be hit). Shown: slot lines from text y 338 (slot 0) and the next line (slot 1), §3.3 |

Rule for (re)creation: after **every** `menu` result that does not leave the screen, the dispatcher falls through to `0x57DFC` (camp) / `0x56012` (castle). That code deletes every item from the first troop count onwards (`0x34AC0(order_base − 4)`, where order_base is the value `print("$L$t$|")` / `print("$H$L$t$|")` returned in that pass). It then jumps back to the pass head (`0x571B1` / `0x5541E`), which recreates the four counts, then the 16 order items for the current toggle value. Items below the counts are created once and never rebuilt. So the toggle handler needs no rebuild code of its own. It XORs R+0x19 with 1 (`0x578B0` camp, `0x55AF9` castle) and sets bit 0x10 of byte +0x0A on the four count records, so that their pop also restores the strip under the old position. The next pass then builds the other layout under the **same** indices.

Camp and castle differ only by the one-index shift (the castle has the extra toll icon). The camp has no geometric or index difference after a toggle.

### 7.2 What a click at (320, 345) returns
`menu` hit test (`0x35C10`): the scan runs from B upward to the last item, and the **lowest** index whose rectangle contains the point (x in [+0x14, +0x14 + w), y in [+0x16, +0x16 + h)) and whose byte +9 bit 0x04 is clear wins. A kind-0xE0 hit walks back to the preceding item. If nothing is hit and the keyboard-focus item `0x84D50` (< 0xA0 only for text-entry items, kind 0x80) is in range, that item is used instead. A left press on a hit item returns at once. Upper half means y < item_y + h/2 − 2.
- Book shown (castle at entry, or camp after an odd number of toggles): no item with a lower index covers (320, 345). The hotspots end at y 280, the counts occupy 290..307 (h = 0x12), and the icons start at y 408. The hit is the slot-0 order word, "Kein Auftrag" (kind 0xA0, centred, about 440 px wide). Upper half gives delta +1. The result is **0x14** (camp) / **0x15** (castle), the editor j = 0 (§3.4). With an empty garrison it cycles back to 0x20/0x10, so the line does not change. The editor then waits for the button to be released (`0x57C4D`).
- Book hidden: nothing is hit (the `%M` items have width 0). The press is ignored and `menu` keeps polling.
- Only the toggle case writes R+0x19 (the only writers in the image are `0x55AF9` and `0x578B0`). A click on an order line **cannot** hide the book. If the book disappeared, `menu` returned 7 (camp) or 8 (castle).

### 7.3 Auto-repeat (`menu`, `0x35BA4`, `0x36605`..`0x36692`)
`menu` repeats a held button on the item under the pointer:
- On entry it sets deadline = clock + `0x84D58`. A change of hovered item (`0x84D4C`), a change of the delta, or a poll with no button down sets the deadline to 0 (no wait). A held button is ignored until clock ≥ deadline.
- On return: if the deadline was 0, `0x84D58` := 0x11 (17 cs). Otherwise `0x84D58` −= 4 while it is > 1. The repeat intervals are therefore **170, 130, 90, 50, 10, 10, … ms**, plus the time each pass takes to rebuild.
- Handlers that wait for the release themselves stop the repeat: order editor j = 0 and j = 7 (camp `0x57C4D`, `0x57DEC`; castle `0x55E71`, `0x56002`). The toggle, the troop counts, the target (j = 2) and the order numbers (j = 3..6) **do** repeat.

Consequence: a press held on the seal ring toggles the book once at the press, then again after 0.17 s, 0.30 s, 0.39 s, … The harness `orig.sh click` holds the button for 0.4 s (`mousedown; sleep 0.4; mouseup`). One harness click on the toggle therefore flips R+0x19 three or four times. The exact count depends on how long each rebuild takes, so the end state after one "click" is a race.

### 7.4 The live observation
- From the code, the click at (320, 345) on the camp after the toggle returns 0x14 and cannot hide the book (§7.2). Positions and indices after a toggle are the same for camp and castle (§7.1).
- The castle comparison is not like-for-like: on the castle no toggle preceded the click.
- [UNVERIFIED] Most likely explanation: a held-button artefact. The 0.4 s hold on the seal ring produced an even or odd number of toggles (§7.3), and the screenshot taken after it did not reflect the final state seen by the second click. I did not find a code path that maps (320, 345) to index 7. Decisive re-test: click the seal ring with a short press (`mousedown 1 sleep 0.05 mouseup 1`) and take a screenshot. Then `move` to (320, 345), wait 1 s, take a screenshot, and click with a short press. Repeat the same sequence on the castle with two short toggles first.
- Implementer note: reproduce the auto-repeat (first repeat after 170 ms, then 40 ms faster each time, floor 10 ms; reset when the hovered item changes or the button is up). Otherwise holding a spinner or the seal ring behaves differently from the original.

## 8. Corrections to other docs
- rules-07 §3a: "Schickt" target cycling skips only the castle's own location; there is no ownership filter (§3.4).
- rules-07 §2.2: the 9999 limit is set on the wrong item; the per-turn total is uncapped (§5.2).
- rules-23 §4.2: the "Zollburg versteigern" window `0x565B0` is unreachable; the live route is `0xBF4E4`/`0xBF51C` (§2).
- `formats.md` §9.2: hotspots (kind 0xE0) report the preceding label's index (introduction).

## Corrections (rules-27)
- §5.1: `Waren vor Ort` (sum of all 15 stock words) also shows for camps with 0 stock; a castle with 0 stock shows `Baujahr`.
- Manoeuvre window opening draws exactly one `rand(3)` for its picture (rules-27 §4).
