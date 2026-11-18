# Residence sub-dialogs: Kaufen, Bauen, Verkaufen, Versteigern, Erweitern, Renovieren

Source: `0x52490` buy, `0x52790` build, `0x52AA0` sell, `0x52B50` auction, `0x52D00` extend, `0x53060` renovate. Called from the Wohnsitz window (`rules-34` §1) after the precondition check `0x51EB0`. Mechanics: `rules-11` §2; confirmations: `rules-23` §7.2 (this doc supersedes its auction row). Primitives as in `rules-34` (header). Additional helpers: `V(r)` = residence value `0x44230(r, c)` (`rules-06`), returns 0 for r = 0xFFFF [C]; `set_res` = `0x520A0` (`rules-11` §2); `rec_del` = `0x42550` (sets bit 7 of the type byte); `ok_item` = `0x348D0` (invisible OK item, `rules-26` §1); `items_paint(first)` = `0x34BF0`; `mouse` = `0x12DE4`; `flip` = `0x30400`; `win_close` = `0x30190`. `year` = u16 at `0xEF4B8`. Text codes: `rules-12` §1 (`%a` bullet item, `%M` placeholder item, `%ku` noun with article, `%kW` residence phrase). Tags: [C] instruction level, [UNVERIFIED] inferred.

Notation as in `rules-34`: `b` branch record, `c = b+1`, `o = b+2`, `r = b+3` (u16), `bi` = branch index, `C` = city record. `BT` = u32 table `0x866C0` = 1, 1, 1, 1, 2, 2, 2, 3, 4, 4 (build years per level).

Common pattern of the three selection dialogs (Kaufen, Bauen, Erweitern) [C]: after printing, `ok_item` is added (one item index after the last printed item) and the window is flipped. Then a loop: every bullet item gets icon `0x74` (selected) or `0x73` (not selected) via `item_set_icon`; `items_paint`; wait until no mouse button is down; `menu`. A right click returns "last item + 1" (`rules-26` §1), i.e. past the OK item. No dialog checks the player's cash; `pay` may drive cash negative.

## 1. Kaufen (`0x52490`) [C]

**Window** `win(60, 40, 520, 400, 0x8010)`: own frame, parchment tiles, **no erase on close** (bit 0x800 absent). It covers the Wohnsitz window exactly; the Wohnsitz loop always redraws itself after Kaufen (`rules-34` §1).

**Text, in order:**
1. `0xBE99C` `[EXE text]`.
2. For s = 0, 1, 2 (all three, no filter): `0xBE9E0` `%a$L%5W für %d Taler$Z` with (`w_s`, `V(w_s)·11/10`), `w_s` = u16 at `0x85CC5 + 6c + 2s`. `%5W` = indefinite nominative, e.g. "ein verfallenes Bürgerhaus" (`rules-12` §1.5). The price shown is the price paid.
3. Only if `V(r) > 0` (i.e. the player has a residence): `0xBE9F8` `$LGutschrift: %d Taler$N` with (`V(r)·2/3`).
4. `0xBEA18` `$Z$L%j Kaufen$N` with icon `0x6A`.

**Items**: 0..2 offer bullets, 3 Kaufen icon, 4 OK item.

**Selection** (`sel` starts at 3 = none):
- idx 0..2: `sel = idx`; clicking the selected bullet again clears it (`sel = 3`). Loop.
- idx 3 with `sel < 3`: buy. idx 3 without a selection: ignored, loop.
- idx ≥ 4 (OK item, right click): close without buying.

**Buy**: `cash_add(o, V(r)·2/3)` (0 without a residence); `pay(o, V(w_sel)·11/10)`; `set_res(b, w_sel)`; then the icon loader `0x2DD70` is called with `0x60 + (r_new & 0x0F)` (loads the residence picture for the redraw). Items freed, window closed. No confirmation text. The offer table is not changed (`rules-11` §2 quirk).

## 2. Bauen (`0x52790`) [C]

**Window** `win(120, 20, 400, 440, 0x8810)`.

**Text, in order:**
1. `0xBEA28` `[EXE text]`.
2. For k = 0..9, **all ten levels, no affordability filter**: `0xBEA60` `%a$L%4u$E$R%d Taler$|` with (1, `0x3A + k`, `(C+0x81 + 150) · V(0x5000 | k) / 200`) (signed division). `%4u` with count 1 and noun id 58..67 prints the accusative with indefinite article: "eine Kate", "eine Hütte", "ein Haus", "ein Bürgerhaus", "ein Landhaus", "eine Villa", "einen Landsitz", "eine Burg", "eine Wasserburg", "ein Schloß". `$E$R` right-aligns the price on the same line; `$|` ends the line.
3. Only if `V(r) > 0`: `0xBEA78` `$Z$LGutschrift:$E$R%d Taler$N` with (`V(r)·2/3`), price right-aligned.
4. `0xBEA98` `$Z$L%j Bauen$N` with icon `0x6E`.

**Items**: 0..9 level bullets, 10 Bauen icon, 11 OK item.

**Selection** (`sel` starts at 10 = none):
- idx 0..9: `sel = idx`, again on the selected one clears it (`sel = 10`). Loop.
- any idx ≥ 10: items freed, **window closed**. Then build only if idx = 10 and a level is selected. Bauen without a selection closes the window without effect (unlike Kaufen).

**Build** (`k = sel`, `r_new = 0x5000 | k`):
1. `box(160, 140, 320, 160, 0xBEAA8 $N$Z$C%1W wird im Jahr %l errichtet sein.$N, r_new, year + BT[k])`. Shown before any money moves; it has no choice.
2. `cash_add(o, V(r)·2/3)`.
3. `pay(o, V(r_new))`. Note: the paid amount is the full `V(r_new)`, not the listed price; the list uses the `(build_level + 150)/200` factor (`rules-11` §2 "display only").
4. `rec_new(13, o, bi, BT[k])`, record +4 (u16) = `r_new`. The old residence stays until delivery.

## 3. Verkaufen (`0x52AA0`) [C]

No window; one box: `box(80, 120, 480, 240, 0xBEAD8 $Z$C^ Wohnsitz verkaufen _$Z$LWollt Ihr %4W für %d Taler verkaufen?$Z%j Ja$Z%j Nein$N, r, V(r)/3, 0x6F, 0x30)`. Items: 0 "Ja" (icon 0x6F), 1 "Nein" (icon 0x30).

Result 0: `cash_add(o, V(r)/3)`, `set_res(b, 0xFFFF)`. Anything else: nothing. No further text.

## 4. Versteigern (`0x52B50`) [C]

Before the window: `A` = `rec_find(3, 0xFF, 0xFF)` (the announced auction, `rules-08` §1.1); `L` = `rec_find(14, o, bi)` (this branch's lot).

**Window** `win(80, 120, 480, 240, 0x8810)`.

**Text, in order:**
1. `0xBEB34` `$C^ Wohnsitz versteigern _$Z$L`.
2. One of:
   - no `A`: `0xBEB54` `[EXE text]`. Items 0..2 are invisible placeholders; no Zurück item.
   - `A` and `L`: `0xBEB90` `[EXE text]` with (r, `year + A.counter`, icon `0x70`). Item 0 = withdraw icon, 1 = placeholder.
   - `A`, no `L`: `0xBEC04` `[EXE text]` with (`year + A.counter`, r, icon `0x70`). Item 0 = placeholder, 1 = register icon.
3. With `A` only: `0xBEC6C` `%j Zurück.$N` with icon `0x30` (item 2).
4. `ok_item` (item 3), flip.

`A.counter` = byte +3 of the type-3 record.

**One `menu` call, then the window closes** (items freed, `win_close`) whatever was clicked:
- idx 0: `rec_del(L)` (withdraw the lot). In the "no auction" layout item 0 is a placeholder; if it could be hit, `L` might be null. [UNVERIFIED: placeholders are not hittable.]
- idx 1: `rec_new(14, o, bi, A.counter)` (register; type-14 counter = auction counter). Fields beyond the header are zero.
- else (Zurück, OK, right click): nothing.

No confirmation box after withdrawing or registering. Once registered, the Wohnsitz precondition flag 1 blocks Kaufen/Bauen/Verkaufen/Erweitern/Renovieren (`rules-34` §1); Versteigern (flags 6) stays open so the lot can be withdrawn.

## 5. Erweitern (`0x52D00`) [C]

Extension bits `X[i]` = u16 table `0x8711C` = 0x20 (Park), 0x40 (Nebengebäude), 0x80 (Ländereien).

**Refusal**: if `(r & 0xE0) == 0xE0`: `box(160, 140, 320, 200, 0xBEC7C $Z$CEs gibt nichts, was Ihr %3W noch hinzufügen könntet.$N, r)`, return. No window.

**Costs** (before the window): `cost[i] = V(r | X[i]) − V(r)`, i = 0..2. An already built extension has cost 0.

**Window** `win(80, 120, 480, 240, 0x8810)`.

**Text, in order:**
1. `0xBECBC` `[EXE text]` with (r).
2. i = 0: `cost[0] ≠ 0` → `0xBED00` `%a%s für %d Taler$|` with (string `*0xB486C` = "Park", cost[0]); else `0xBDB40` `%M$E`.
3. i = 1: same with (string `*0xB4F20` = "Nebengebäude", cost[1]); else `%M$E`.
4. i = 2: `cost[2] ≠ 0` → `0xBED14` `%aLändereien für %d Taler$|` with (cost[2]); else `%M$E`.
5. `0xBED30` `$Z%j Erweitern lassen$N` with icon `0x71`.

So only missing extensions are listed; built ones leave an invisible placeholder and keep the indices.

**Items**: 0..2 extension bullets (or placeholders), 3 "Erweitern lassen" icon, 4 OK item.

**Toggling**: working word `w = r`. Each pass sets item i's icon to `0x74` if `w & X[i]`, else `0x73` (also on placeholder items). `menu`:
- idx 0..2: `w ^= X[i]`. Loop. The toggles are independent (checkboxes, any combination).
- **any idx ≥ 3** (Erweitern lassen, OK item, right click): items freed, window closed, and the change is **committed**. There is no cancel path except toggling back to the original state. [C] quirk: a right click confirms.

The XOR could remove a built extension, but built extensions show as placeholders. [UNVERIFIED: placeholders cannot be clicked, so removal is not reachable through the UI; `rules-11` §2's "removing an extension gives money back" describes the code path only.]

**Commit**, if `w ≠ r` and `w ≠ 0xFFFF`:
1. `pay(o, V(w) − V(r))`.
2. `box(160, 180, 320, 120, 0xBED48 $N$CEure Wünsche werden im nächsten Jahr erfüllt sein.$N)`.
3. `rec_new(13, o, bi, 1)`, +4 = `w`.

Unchanged word: nothing, no text.

## 6. Renovieren (`0x53060`) [C]

No window; one box.
- target `t = (r & 0xFF) | 0x6400` (same level and extensions, condition 100);
- `cost = V(t) − V(r)`, raised to 100 if smaller (signed compare);
- `d = 1 + [cond < 25]`, `cond = r >> 8`.

`box(80, 100, 480, 280, 0xBED84 $Z$C^ Wohnsitz renovieren _$N$N$L$LDie Renovierung %2W kostet Euch %d Taler und würde bis zum Jahre %l dauern.$Z%j Renovieren$Z%j Nichts dergleichen tun$N, r, cost, year + d, 0x72, 0x30)`. Items: 0 "Renovieren" (icon 0x72), 1 "Nichts dergleichen tun" (icon 0x30).

Result 0: `pay(o, cost)`; `rec_new(13, o, bi, d)`, +4 = `t`. No confirmation. Anything else: nothing.

The dialog is offered at any condition, also at 100 (cost then 100). [C]

## 7. Summary of boxes and windows

| dialog | rect (x, y, w, h) | style | closes after |
|---|---|---|---|
| Kaufen | 60, 40, 520, 400 | 0x8010 window | buy, OK item, right click |
| Bauen | 120, 20, 400, 440 | 0x8810 window | any index ≥ 10 |
| Bauen confirmation | 160, 140, 320, 160 | box | click |
| Verkaufen | 80, 120, 480, 240 | box | answer |
| Versteigern | 80, 120, 480, 240 | 0x8810 window | the first `menu` result |
| Erweitern refusal | 160, 140, 320, 200 | box | click |
| Erweitern | 80, 120, 480, 240 | 0x8810 window | any index ≥ 3 (commits) |
| Erweitern confirmation | 160, 180, 320, 120 | box | click |
| Renovieren | 80, 100, 480, 280 | box | answer |

Precondition refusals before any of these: `rules-34` §1.

## Open

- Whether `%M` placeholder items (style 0x1630) can be hit by `menu`, and whether `item_set_icon` on them makes a glyph visible (Erweitern sets icon 0x74 on placeholders of built extensions). [UNVERIFIED]
- Icon 0x30 is used for "Nein", "Zurück." and "Nichts dergleichen tun"; its picture is not described here.
- `rules-12` §1 dates the auction announcement as `year + c + 1` at the year end that creates it, while this dialog prints `year + A.counter`; both are consistent if the year has advanced and the counter been decremented once in between. [UNVERIFIED]
