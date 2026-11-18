# After-battle report (rules-59)

Source: `battle_main` `0x3EE60`, round-end count `0x40C61..0x40D4E`, report `0x40D53..0x41E3B`. Helpers: `print` `0x322C0` (window text engine, `$W` = wait for a click, `$P` = new page, `$C`/`$L`/`$R` alignment, `$Z` line break, `$E` tab, `$n<` column n), `choose` `0x36850` (returns the clicked `%a` item, 0-based), `win_open` `0x2FA80`, `win_close` `0x30190`, `wait_click` `0x36720`, `rand(n)` `0x36FF0`, `pay(p, amt)` `0x43860`, `cash_add(p, amt)` `0x43790`, `add_evidence(p, mask)` `0x44E00`, `retaliate(victim, culprit, code)` `0x5B140` (`rules-09` §4), `branch_add_goods` `0x439F0`, record find `0x426B0(type, owner, key)`, record new `0x424A0`, record delete `0x42550`, icon bank `0x2DD70(id)`. `A` = `0xDB7F4`, `D` = `0xDB7F5`, participant p = 0 (A), 1 (D), 2.. (helpers, n = `[EBP−0x6C]`), person of p = `0xDB7F4[p]`, side of p = `0xD9794[p]` (0 attacker, 1 defender, 2 stayed out), colour `0xDB7FC[p]`, base unit type `0xD9798[p]` (0 Söldner row, 5 Räuber row; helpers at `0x40936`), participant record `0xD6A44[p]`, captured markers `0xDB800[p]`, prisoners `0xDB804[p]`, losses `L[p][c]` = byte `0xD751C + 6p + c` (c = unit class, `rules-07` §5.2), destroyed markers `0xDC83B`, destroyed wagons `0xDC83D`, bombardment hits `0xDC839`, idle counter `0xDC840`, `S` = shipment, `P` = target property (`[EBP−0x60]`), `H` = attacker home property, `value` = `[EBP−0x70]`, `markers` = `[EBP−0x68]`. Tags: **[C]** instruction level, **[L]** decompiler or partial trace, **[UNVERIFIED]** inferred. Everything is [C] unless marked.

## 0. End of the fight and what is behind the window

- After each round (`0x40C61`): `men0` = living men (type < 12, man +0x0C HP ≠ 0 is implied by presence on the grid) of side 0, `men1` of side 1, `ent` = entrance cells (flag 2) holding a side-0 man. End if `ent ≥ 3` or `men0 = 0` or `men1 = 0`. Otherwise `idle −= 1`; `idle = 0` ends the fight undecided (`0x40D53..0x40D68`; `idle` is reset to 10 by any damage, `rules-07` §5.6).
- If ESC mode was on (`0xDC83F = 1`): one draw-only `activate` and a present of the field so the last frame is visible (`0x40D6E..0x40DCE`).
- Wait until the sound driver is idle (`0x12C71` loop), unload the 36 battle samples `0x858E0[0..35]` (`0x12730`), present the field once more (`0x40E13`), `0x84D6C −= 1`.
- **The field window stays open.** `win_open(80, 80, 480, 320, 0x8810)` (`0x40E6A`) opens the report over it; nothing else is drawn behind it, no banner, no Kontor. At the very end (`0x41E18`): `wait_click`, `win_close` (report), free the field buffer (`0x1331E`), `win_close` (field), return. So the last battle frame is visible until the report closes, and the caller draws whatever comes next (turn-end bill).
- Everything in §1..§6 is printed into that one window with `print`; the pages are separated by `$W$P` inside the texts. There is no human test anywhere in the report except the two occupation menus (§2). Whoever sits at the keyboard clicks through it, also for a computer attacker vs a human victim at year end (`rules-57` §2: a battle only happens when A or D is human, so every report has a human party).

## 1. Page 1: title and verdict

1. `0xBBB20` `[EXE text]`.
2. Side effects, no text: if law flag `0x85A2E` (Überfälle forbidden) ≠ 0, `add_evidence(person(p), 0x400)` for every p with side 0 (`0x40E7D..0x40ED8`). Then **`rand(100) < 50`** → `retaliate(D, A, 4)` (`0x40EDA..0x40EFE`). These happen for every outcome, undecided included.
3. Verdict (one of):
   - `men0 = 0` → `0xBBB4C` `[EXE text]` with (A, D).
   - else `men1 = 0` → `0xBBBC4` `[EXE text]` with (A, D).
   - else `idle > 0` (ended by three entrance squares) → `0xBBD0C` `[EXE text]`.
   - else (idle ran out) → `0xBBC64` `[EXE text]`.
   The order of tests is `men0`, `men1`, `idle`; both sides empty reads as repelled.

## 2. Page 2: conquest (`0x40F78..0x4115B`)

Condition: `(ent ≥ 3 or men1 = 0) and idle > 0`. Otherwise nothing is printed here and the next `$W$P` comes from the Gewinne header.

| code | text | effect |
|---|---|---|
| 0, 1, 2 | none (no page 2) | none |
| 3 | `0xBBD50` `[EXE text]` with (D, A) | shipment record deleted (`0x40FBE`) |
| 4 | human A: `0xBBE08` `[EXE text]` with (A, `P+1`), `choose`; computer A: `0xBBE54` `$W`, answer := 0 | answer 0: `0xBBE58` `[EXE text]` with (`P+1`, A, pigs, musicians, adjective, wine), then `P+2 := A` |
| 5 | human A: `0xBBF4C` `[EXE text]` with (A, `P+1`), `choose`; computer A: `$W`, answer := 0 | answer 0: `0xBBF90` `$P$N$N$Z$CNie wieder wird vom %o das Böse seinen räuberischen Lauf nehmen; die neuen Herren, ihnen voran %1N0, geloben einander feierlich, niemals mehr friedlichen Durchreisenden etwas anderes als Gerechtigkeit widerfahren zu lassen.$N` with (`P+1`, A), then `P+2 := A` |

- Castle feast numbers, drawn in this order (`0x41039..0x41085`): `wine = rand(10) + 2`, `adjective = 0xB646C[rand(11)]`, `musicians = rand(30) + 2`, `pigs = rand(20) + 2`. Adjectives: gänzlich hervorragender, begnadeter, außergewöhnlicher, recht guter, durchschnittlicher, gewöhnlicher, mieser, erbarmungsloser, durstiger, tölpelhafter, dilletantischer (original spelling).
- A computer attacker always occupies. A human who answers "Nein." (or right-clicks) leaves the owner unchanged and gets no text; the `$W` of the question text has already been consumed, so the Gewinne header's `$W$P` follows directly.
- Goods and wagon bookkeeping, no text (`0x4115B..0x411C7`): if `markers > 0`: `lost = qty · 0xDC83B / markers`, `qty −= lost`, `0xDC83B := lost` (now in units, used by §4). Wagons: if `0xDC83D = S[+8]` and > 0, `0xDC83D −= 1` (one cart is always spared); `S[+8] −= 0xDC83D`.

## 3. Page 3: Gewinne (`0x411CA..0x415CF`)

Header `0xBC07C` `$W$P$N$C^ Gewinne... _$N$N$N$L`. Then one row per participant p = 0, 1, 2.. (in that order) with **side < 2** (helpers that stayed out are skipped):

`0xBC09C` `$L%1N0$E$30<$R%d%I%I$E$2<%d%I$|$N` with
1. person(p);
2. `%d` at column 30, right-aligned: **prisoners** `0xDB804[p]`;
3. `%I` (width 0, kind 0x12, frame 691 of `ICONBF1`, ramp `0x80 + 5·colour(p)`): a zero-width sprite, over-drawn by the next one;
4. `%I` (width 40, kind 0x12, frame 1444 of `ICONBF1`, same ramp): the **banner** in the participant's colour (the flag icon of the capture; same frame as the planted banner and the prisoner sprite, `rules-43`);
5. `%d` at column 2: **captured goods markers** `0xDB800[p]` (printed as 0 for p = 1, the defender, whatever the counter says);
6. `%I` (width 46, kind 6, icon `0x2C` raw, 0): the goods crate.

Frame pointers: kind-0x12 elements take `bank + bank[4·frame]` (`0x2DD70(0xF1)`); the table starts at offset 0, so `0xACC/4 = 691` and `0x1690/4 = 1444`. Frame 691 is the last frame of type 5 action 4 (fallen Räuber, left view) [L: by the BASE table, `rules-51` §2.2; its role at width 0 is cosmetic].

Per row, after printing (`0x4130D..0x41539`), no text:
- Prisoners: if `0xDB804[p] > 0` and person(p) is human, and not (p = 1 and code < 4): key = home property index of participant p (p = 1: index of P). Find type-55 record (owner person(p), key) or create it with +4 = 0; `+4 += prisoners`.
- Captured goods (p ≠ 1, `0xDB800[p] > 0`): `amount = qty · 0xDB800[p] / markers`; `branch_add_goods(home property of p, S.good, amount)`; `gain[p] = amount · (good + 8)`; if person(p)'s type-0 mission record has kind 0x13 or 0x1D, `+5 += gain[p]`; `qty −= amount`; `captured_total += amount`.

Then for every p with `gain[p] > 0` (`0x4153E..0x415CF`), **each on its own page**: `0xBC0C0` `[EXE text]` with (person(p), `0xB6618[good]`, `gain[p]`). The capture (all zeros) had none.

## 4. Page 4: Beschädigungen (`0x415D1..0x41687`)

`0xBC0F8` `$W$P$C` always. If `0xDC83B > 0` or `0xDC83D > 0`:
- `0xBC100` `$N^ Beschädigungen... _$N$N`; if only one of the two is non-zero, an extra `0xBC11C` `$N$N`;
- `0xDC83B > 0`: `0xBC124` `[EXE text]` with (`0xDC83B` in units, see §2, `0xB6618[good]`);
- `0xDC83D > 0`: `0xBC15C` `[EXE text]` with (`0xDC83D`, after the spare-one rule);
- then `0xBC0F8` `$W$P$C` again.
With neither counter set there is no Beschädigungen page: the first `$W$P$C` ends page 3 and opens the Verluste page directly. Castle and camp attacks (no markers, no wagons) therefore never show it.

## 5. Page 5: Verluste (`0x4168A..0x419DB`)

Carter/sailor line first. `k = L[1][5]` = class-5 losses of the defender (types 10 and 11 are class 5, so the one byte `0xD7527` serves both):
- `k = 0`: `0xBB86C` `$N` before the header, `0xBC11C` `$N$N` after it;
- header `0xBC1AC` `^ Verluste... _$N$Z$L`;
- `k = 1`: code < 3 `0xBC1C4` `[EXE text]`, else `0xBC204` `[EXE text]`;
- `k > 1`: code < 3 `0xBC23C` `[EXE text]`, else `0xBC278` `[EXE text]`, with (k).
`k` counts fallen class-5 men of participant 1 during the fight (the kill counter per participant and class). There is no separate formula: it is the body count.

Rows, one per participant p with side < 2, in participant order: `0xBC2B8` `$L%1N0$E$R$38<%d%I$E$26<%d%I$E$14<%d%I$E$2<%d%I$|$N` with person(p) and four pairs, column 38 → class 0, 26 → class 1, 14 → class 2, 2 → class 3:
- `%d` = `L[p][c]` (men of class c lost by p);
- `%I` (width 40, kind 0x12, frame `BASE[base(p) + c][3] + 2`, ramp `0x80 + 5·colour(p)`): the front view (DIR4 = 2) of the walking frame of that unit type in the participant's colour. `base(p)` = `0xD9798[p]` (0 or 5), so the icons are the four Söldner types (0..3) or the four Räuber types (5..8). Frame = `bank + bank[8 + 4·BASE]`, which is `BASE + 2`.
The leader (class 4) and class 5 are not in the table.

Per row, no text (`0x4190D..0x419DB`): losses are subtracted from the owner's troops: p = 1 on field ≥ 4 → garrison `P+0x58+c −= L[1][c]`; otherwise `record(p)+5+c −= L[p][c]` (the attacker's or helper's order record; for the transport defender record(1) is the shipment and the subtraction hits bytes +5..+8 of it [C that it is written; the defender of a raid normally has no class 0..3 losses unless escorted, in which case the escort counts are lost in the shipment's qty/good/transporter bytes: quirk, [UNVERIFIED] whether it is reachable in practice]).

## 6. Page 6: Bezahlung, castle damage, insurance (`0x419E0..0x41E18`)

1. `R` = sum of bytes `record(0)+5..+8` (attacker's order record **after** the loss subtraction), `premium = value · R / 80` (`0x41A28`). Header `0xBC2EC` `$W$P$C$Z^ Bezahlung... _$N$N`.
   - `premium > 0`: `0xBC30C` `[EXE text]` with (A, premium), then `pay(A, premium)`.
   - else `0xBC34C` `[EXE text]` with (A).
   This page is printed for every outcome (repelled, undecided, won).
2. Bombardment (code ≥ 4 and `0xDC839 > 0`, `0x41A96..0x41BD7`): `a = min(hits/4, P+3 high byte)`, `b = min(hits/3, P+3 low byte)`, `a = 0` if `a = 1`; `P+3 −= (a << 8) + b`. Castle (P kind 2): `0xBC378` `[EXE text]` (a); camp: `0xBC3AC` `[EXE text]` (a). Then `0xBC3E0` `[EXE text]` with (`P+1`, `0xB6814[(P+3 low byte)/11]`): erbärmlich, erschreckend, schlecht, mäßig, durchschnittlich, akzeptabel, gut, ausgezeichnet, vorbildlich, überragend.
3. Castle/camp taken (code ≥ 4 and `P+2 = A`, `0x41BDA..0x41D7D`), no text: garrison `P+0x58..+0x5B := 0`, training `P+0x5C..+0x5F := H+0x5C..+0x5F`; then up to `P+3 high byte` men move from the attacker record into the garrison round-robin over slots 1, 2, 3, 0, … (skipping empty record slots, stopping when all four are empty); delete the type-17 record (owner A, key = P index) and the type-16 record (any owner, key = P index).
4. Raid insurance (code < 4, `0x41D82..0x41E15`): `amount = captured_total · (V[good] + 4)`. If D has insurance bit 2 (`0xE3A83 + 0xF8·D` & 4) and `amount > 0`: escorted (`S[+4] & 1`) → `0xBC400` `%2N0 Versicherung zahlt zur Schadensdeckung %d Taler.$N` with (D, amount), `cash_add(D, amount)`; else `0xBC43C` `[EXE text]` with (D). Captured_total is the sum over the capturing participants of §3; destroyed goods are not insured.
5. `wait_click`, close both windows, return (§0).

## 7. Page sequence by case

| case | pages |
|---|---|
| raid 0..2, attacker won, nothing captured, no markers/wagons destroyed | 1 verdict, 3 Gewinne, 5 Verluste, 6 Bezahlung |
| raid 3, attacker won (capture) | 1, 2 fleet burned, 3, (gains pages), 4 if destruction, 5, 6 |
| raid, repelled or undecided | 1, 3, (gains), (4), 5, 6 |
| castle/camp, won | 1, 2 occupation question/feast, 3, 5, 6 (+ damage lines, no insurance) |
| castle/camp, repelled or undecided | 1, 3, 5, 6 (+ damage lines) |
| called off (`rules-50` §2) or ransom accepted (`rules-52` §2) | no report at all; `battle_main` returns before the fight |

A conquest page (2) needs `idle > 0`; a fight that ended by the idle counter never conquers, even with the defender's men all gone at that moment.

Answer: the report is one `print` stream in `win_open(80, 80, 480, 320, 0x8810)` over the still-open battle field. Page 1 verdict (`0xBBB4C`/`0xBBBC4`/`0xBBD0C`/`0xBBC64`, chosen by men0 = 0, men1 = 0, idle > 0), page 2 only on conquest with idle > 0 (ship: `0xBBD50`, castle/camp: occupy question for a human A, auto-occupy for a computer A, feast text with four `rand`s), page 3 Gewinne rows for every participant on a side (prisoners `0xDB804`, banner frame 1444 in team colour, captured markers `0xDB800`, crate icon `0x2C`) plus one "erbeutete" page per participant with gains, page 4 Beschädigungen only if markers or wagons were destroyed (units = `qty · destroyed / markers`, wagons minus the spared one), page 5 Verluste (class-5 body count line, then per participant four class counts with the front walking frame `BASE[base+c][3] + 2`), page 6 Bezahlung (`value · R / 80`, else "Niemand fordert"), then castle damage/condition or raid insurance lines on the same page, `wait_click`, both windows closed. The 50 % retaliation roll and the evidence happen right after the title, before the verdict text.

## 8. The `%I` element and the 48 px rows (follow-up)

Source: `print` `0x322C0` handler `0x33E3F` (dispatch `0x32AA8[0x24]`), line painter `0x31610` branch `0x31959..0x319C3`, `$N` handler `0x326EA`, `$E` `0x326F4`, `$n>` `0x32801`, `$n<` `0x3286D`. Window state as in `rules-47` (`d501A` cursor y, `d5026` line height, `d6A30` line width, `d502C` box width, `0x84D9A` entry count, `0xC25C0` entry words). All [C].

**Creation (`0x33E3F`).** `%I` takes four stack arguments and appends five u16 entries to the line list without any wrap or overflow test:

| arg | stored | meaning |
|---|---|---|
| 1 `width` | entry +3 (`0xC25C6`), and `d6A30 += width` | pen advance |
| 2 `k` | entry +0 = `(k << 8) | 4` | element code 4, **k is a vertical raise in pixels**, not a kind |
| 3 `ptr` | entry +1..+2 (u32 at `0xC25C2`) | raw sprite data pointer, passed to the blitter unchanged |
| 4 `flags` | entry +4 (`0xC25C8`) | blitter flags: `0x80 + 5·colour` for a team-ramp bank frame, 0 for a plain icon |

There is no height argument and the sprite header is never read. `rules-12` §1's "width, kind, pointer, height" should read "width, raise, pointer, flags".

**Placement (`0x31959`).** When the line is flushed, the walker reaches the code-4 entry with the pen at `x` and calls `0x13AE0(frame, x, lineY − k, ptr, flags)`, then `pen += width` and skips the five entries. So:
- horizontal origin = the pen x (left edge), exactly where the next glyph would start; no centring inside `width`, no XOFF, no frame x-offset. `width` only moves the pen and counts in `d6A30` for `$R`/`$C` alignment.
- vertical: `0x13AE0` is top-left anchored (`rules-33`, `rules-38` §10), so **sprite top = lineY − k** where lineY is the y the line's glyphs are drawn at (`0x3191B`: glyph y = lineY, the top of the 24 px line cell). The sprite's bottom is `lineY − k + h` with h from the frame data. For the report: bank frames with k = 0x12 and h = 40 run from lineY − 18 to lineY + 22, the crate (k = 6, icon `0x2C`, 40 px) from lineY − 6 to lineY + 34, overlapping the next row.
- a zero-width `%I` leaves the pen where it is, so the next element (the banner, frame 1444) is blitted at the **same x**, over frame 691. A later element with a larger width then advances past both.

**Line advance.** `0x31610` never looks at element heights: after the walk it sets `d5018 := d502A` and `d501A += d5026` (`0x31DB6..0x31DC8`), 24 px for these windows, unless `0x84D9E` (`$E`) is raised. A line with `%I` therefore advances 24 like any other.

The 48 px in the capture come from the row format, not from `%I`: `…%d%I$|$N`. `$|` expands to `$2>$2<`; `$2>` (`0x32801`) flushes the pending line because it holds entries (advance 1). Then **`$N` (`0x326EA`) calls `0x31610` unconditionally**, also on an empty line, which advances again (advance 2). Rows written as `…$|$N` are 48 px apart, the first row at the normal y. This corrects `rules-47` §2's note that `$N` flushes only when the line holds entries: that holds for `$L`, `$R`, `$C`, `$B`, `$Z`, `$V`, `$W` and the margin codes (each tests `0x84D9A` first), but `$N` and `$E` do not test.

Answer: `%I(width, k, ptr, flags)` is a code-4 line entry: pen advance = width (also counted for alignment), blit at (pen x, lineY − k) with the raw pointer and the given flags, no measurement, no wrap, no height. Line height stays `d5026` = 24. The doubled row pitch is `$|` + `$N`: the margin code flushes the non-empty line and `$N` always flushes once more. A zero-width `%I` is drawn at the same x as the following element.
