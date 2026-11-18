# City info: goods rows, Katastrophe, Lager price (rules-48)

Source: city info window `0x4F3C0(player, c)` (from the trade map `0x4CC90`). `C` = city record `0x85F04 + 0x8C·c`. `print` = `0x322C0`, `rec_find`/`rec_next` = `0x426B0`/`0x427B0`, `property_create` = `0x42870`, `net_worth_goods` = `0x44340`, `clamp0` = `0x441F0`, `V_res` = `0x44230`. Tags: **[C]** instruction level, **[L]** decompiler or xref only, **[UNVERIFIED]** inferred. Everything is [C] unless marked. This confirms and extends `formats.md` §11.2. Goods order: 0 Korn, 1 Fisch, 2 Bier, 3 Wolle, 4 Holz, 5 Rind, 6 Erz, 7 Wein, 8 Fell, 9 Glas, 10 Salz, 11 Tuch, 12 Silber, 13 Pfeffer, 14 Waffen.

## 1. Hauptproduktion and Begehrte Güter

### 1.1 Selection (`0x4F409..0x4F504`, before any print)

One loop over three groups k = 0, 1, 2. Group k covers goods g = 5k … 5k+4. Each group yields exactly **one** good per row, so each row shows one good from 0..4, one from 5..9 and one from 10..14, in that order.

- `P[k]` (Hauptproduktion): running best starts at value 0 and good 5k. For g = 5k..5k+4 in ascending order: if byte `C+0x13+g` (yield `E[c][g]`, `rules-02` §1.4) > best, take g. Strict `>`.
- `D[k]` (Begehrte Güter): running best starts at 0 and good 5k. For the same g: `r = C[+0x22+g]·100 / V[g]`, a signed integer division (`IDIV`). `C+0x22+g` is the current price of g (`rules-00` §6), `V[g]` = goods-table byte 12 at `0x8653E + 15g` = 7 + g. If r > best, take g. Strict `>`.
- Tie-break: ties keep the lower g. If no value beats 0 (all zero), the result is g = 5k.
- No RNG draw. No other city field, demand counter or stock is read.

Our rule (three highest / three lowest yields over all 15 goods) is wrong in two ways. The original picks one good per group of five. Begehrte Güter uses price relative to value weight, not yield.

### 1.2 Check against `GAME5.orig`

These values come from the save's city block (file `0x12C1C`) and goods block (`0x13194`), with the rule above applied offline. They are not a live read:

| c | city | P | D |
|---|---|---|---|
| 4 | Greifenheim | Bier, Glas, Waffen (2, 9, 14) | Wolle, Fell, Tuch (3, 8, 11) |

Greifenheim's D ratios are: group 0: 128, 87, 88, **150**, 118. Group 1: 150, 107, 128, **166**, 100. Group 2: 147, **172**, 110, 130, 100.

In this save, our rule happens to give the same six goods in the same order. So the selection alone does not explain a difference in the Greifenheim capture. The likely cause is icon position (§1.3). Our code draws at `right edge + 14 + 40k`, `y − 8`. The original uses 42 px spacing, `y − 6`, starting at the label's pen x including trailing spaces [UNVERIFIED: not compared against the capture here]. Other cities and later prices will diverge from our rule.

### 1.3 Sprites and positions

- Each icon is printed with `%j ` and arg **`0x14A + g`**: three times with `P[0..2]` after `Hauptproduktion: `, then `$Z`. Three times with `D[0..2]` after `Begehrte Güter:    `, then `$N$N$Z`.
- `%j` makes an item of style `0x1090`, 36 × 36 (ICONIN size of `0x14A..0x158`). The item sits at pen x, top = line y − 6. Pen advances 36, then the space adds 6, so the icons are 42 px apart (`rules-46` §1.1, `formats.md` §11.2).
- The line is centred: `x0 = 12 + (456 − L)/2`, with `L = W(label) + 126`. Label widths from `formats.md` §11.2: 202 and 192.
- Window-relative positions (window at (80,20)) from those formulas. Not besieged:
  - Hauptproduktion: line y 240. Icons at x 278 / 320 / 362, top y 234. Screen: (358,254), (400,254), (442,254).
  - Begehrte Güter: line y 276. Icons at x 273 / 315 / 357, top y 270. Screen: (353,290), (395,290), (437,290).
  - When the city is besieged, the siege line pushes both rows down 24 px.
- These icons are clickable items. The menu result for them is not handled (only results 6 and 7, the Lager icon, act).
- The ±9 on cursor y (`0xD501A`) around each label has no visible effect (`formats.md` §11.2) [L].

## 2. Katastrophe

- `rec = rec_find(2, 0xFF, c)` (`0x4F504`: type 2, owner wildcard, key = c). This is the first disaster-aftermath record of the city (`rules-06` §aftermath, `rules-19`). There is no `rec_next` loop.
- `k = rec ? rec[+4] + 1 : 0`.
- If k ≠ 0, the label is wrapped in `$h` … `$h`. Only `Katastrophe: ` is highlighted.
- Then `print(T[0xB7054][k], u16 rec+6)`, with table `0xB7054`:

| k | text |
|---|---|
| 0 | `keine%v` (the arg is read through a null record pointer and swallowed by `%v`) |
| 1 | `der Brand von %l` |
| 2 | `die Sturmflut von %l` |
| 3 | `die Pest von %l` |
| 4 | `die Lawine von %l` |

`rec+4` = disaster kind d (0 Brand, 1 Sturmflut, 2 Pest, 3 Lawine). `rec+6` = the year it struck. `%l` prints the year without separators. The line shows the disaster for as long as the type-2 record lives: two year ends, plus 20 % per year for an extension (`rules-06`).

## 3. Lager line price

This is the "no branch here" case. `P = property_create(0, c, player)` is created at window open. It is a fresh kind-0 record in the first free property slot (kind 3). Its fields: +0x23 = 15 (wood m²), +0x25 = 0 (stone m²), +0x27 = 1 (one Karren), conditions +0x2D..+0x32 = 100, residence +0x03/+0x04 = 0xFFFF, stock 0.

Displayed price = `clamp0(net_worth_goods(P))`. For this record the kind-0 branch of `0x44340` reduces to:

`price = i32 C+0x84 + u16 C+0x6F · 15 + u16 C+0x71 · 0 + V_res(0xFFFF) (= 0) + 0 (stock) + 100·1·u16 C+0x73 / 100 + 0 (transport) + i32 C+0x0E`

In fields: rights base + 15·level A + TP[c][Karren] + population, then `max(0, ·)`.

This equals `branch_price` (`kontor.rs`) except for the `max(0, ·)` clamp. The clamp has no effect while all terms are non-negative. `formats.md` §11.2 also gives the conditions for the variant lines and the close refund `clamp0(value(B))·2/3`.

If no slot is free, P = 0 and the line reads "Dieses Lager ist bereits vergeben." The temporary P is freed (`0x429A0`) on every exit except a successful purchase. Any price difference against the original therefore comes from the inputs (rights `+rand(200)` per year, transporter price, population), not from the formula [UNVERIFIED for the specific capture].

Answer: Hauptproduktion and Begehrte Güter each show exactly three goods, one per group of five (0..4, 5..9, 10..14), left to right. Hauptproduktion takes the good with the highest yield byte `C+0x13+g` in each group. Begehrte Güter takes the good with the highest `price C+0x22+g · 100 / (7+g)` (goods byte 12, integer division). Both use a strict `>` from a start of 0, so ties go to the lower good and an all-zero group gives its first good. There is no RNG. Sprites are `0x14A + g`, 36 × 36, 42 px apart, top at line y − 6. Unbesieged, they sit at window (278/320/362, 234) and (273/315/357, 270), i.e. screen +(80,20); a siege pushes both rows down 24. On the `GAME5.orig` data this rule gives Greifenheim Bier/Glas/Waffen and Wolle/Fell/Tuch, the same as our yield rule, so the capture mismatch is probably icon placement. "Katastrophe:" shows `keine` only when the city has no type-2 record. Otherwise it shows `[EXE text]` from `rec+4` and `rec+6`, with the label highlighted. The Lager price is `max(0, value of a fresh branch)`, i.e. `branch_price` plus a zero clamp.
