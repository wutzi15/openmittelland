# Branch screen order editor (transport panel)

Source: branch screen `0x4FA70`, panel drawing `0x502C0..0x50E98`, click handling `0x512B9..0x51E2B`; feasibility refresh `0x45820`, `order_feasible` `0x45360`, reserve `0x49370`, unreserve `0x49430`, property lookup `0x42A50(kind, location, owner)` (location 0xFF = any) and its "next" `0x42B10`. Primitives as in `rules-34` header (`print` `0x322C0`, `item_range` `0x34B00`, `menu` `0x35B60`, `items_free` `0x34AC0`). Item kinds and the spinner/cycling click rule: `rules-26` §1 (upper half of the line = delta +1 / +10^d, lower half = −1 / −10^d, clamped to the item range). Slot layout: `rules-02` §3.1. Tags: [C] instruction level, [L] inferred from code, [UNVERIFIED] not checked. Corrects `formats.md` §11.3 (see §8).

Notation: `b` branch record, `c = b+1` city, `o = b+2` owner, `stock[g]` = u16 `b+5+2g`, `owned[t]` = `b+0x27+t`, slot `s = b+0x33+8k`: `s0` kind, `s1` u16 qty, `s3` good, `s4` workers/count, `s5` means/transporter type, `s6` destination, `s7` status. `CAP` (`0x86710`) = 100, 1000, 50, 80, 700, 30; `TC` (`0x86728`) = 200, 1800, 250, 250, 2200, 150 (t = Karren, Schiff, Ballon, Dampfkarren, Tauchboot, Flugmaschine). `NAV` (`0x85A00`, u8 per location) = 1,0,0,1,1,1,0,0,1,1 for cities 0..9, 0 for 10..13, 1 for 14..16, 0 for 17..20. "Ship" = t ∈ {1 Schiff, 4 Tauchboot}.

## 1. Rows shown [C]

- Exactly **two rows**, slots 0 and 1. Slots 2 and 3 are never drawn or editable. Slots with kind > 7 (castle/camp orders 0x11.., `rules-26`) produce **no row at all** (no items; the indices below then shift [L: the click handler assumes 7 items per row, so a castle kind in slot 0 would misroute slot-1 clicks; branch slots never hold those kinds in practice]).
- Every row creates exactly **7 items** (`%J` is an inline icon, not an item). Panel item `i` (relative to the branch screen's banner base) = `0x31 + 7·row + col`, `col` 0..6. Indices `0x31..0x3E` go to the order handler (§4).
- Panel: `print("$L$t$|")`, then the cursor y is set to 296 (`0xD501A := 0x128`), so row 0 occupies y 296..319 and row 1 y 320..343 (line height 24) [L; matches the A/B: (615,300) hit slot 0, y = 328 is slot 1's upper half]. After the rows: `$Z$LAuslastung: %d%%$E$RKosten: %d Taler$N$B` (`0xBE7CC`), then `$g$|`.
- The last clicked panel item (if its index lies in the 14 panel items) gets bit 0x40 of item byte +9 set before redraw [C; visual meaning UNVERIFIED].

## 2. Row layouts [C]

Format strings (`print` args in order, `%m` = (style, data)). Styles: `0x8A0` cycling string (kind 0xA0), `0x802` 2-digit spinner, `0x814` 4-digit spinner, `0xC02` 2-digit spinner with flag 0x400, `0x1830` menu line (status glyph). Flag 0x800 on all of them: meaning [UNVERIFIED]. Status glyph data = 2-byte string at `0x86770 + 2·s7`: status 0 → char 0x0D, 1 → 0x0E, 2 → 0x0F, 3 → empty string. Observed: a new feasible unconfirmed line shows an empty circle (0x0E); 0x0D/0x0F shapes [UNVERIFIED, presumably "impossible" / filled].

| kind | string | col 0 | col 1 | col 2 | col 3 | col 4 | col 5 | col 6 |
|---|---|---|---|---|---|---|---|---|
| 0 | `0xBE640` `$4<$C%m%M%M%M%M%M%M$\|$L` | `Kein Auftrag` padded with 32 spaces each side (`0xBE5F0`), 0x8A0 | %M | %M | %M | %M | %M | %M |
| 1 | `0xBE658` `$78<$R%m%M %m$E$\|$30>$Lmit %m %s$E$\|$62>$7=$C%s$E$\|$70>$L%m%M %s$E$\|$R%m$\|$L` | verb `0xB65A0[g]`, 0x8A0 | %M | good `0xB6618[g]`, 0x8A0 | workers `&s4`, 0x802; then `%s` "Arbeiter" if s4 = 1 else "Arbeitern" | means `&s5`, 0x802; then `%s` means noun, singular `0xB465C[g]` if s5 = 1 else plural `0xB4D0C[g]` | %M | status |
| 2 | `0xBE6A8` `%m %m$E$30>$14=$C%m$E$\|$44>$Lauf %m %m in %m$E$\|$R%m$\|$L` | `Verkauft` | qty `&s1`, 0x814 | good | count `&s4`, 0xC02 | transporter | destination | status |
| 3 | `0xBE720` `%m %J %m…auf…` | `Verk.` then icon 0x1B6 | qty | good | count | transporter | destination | status |
| 4 | `0xBE6E4` `%m %m$E$30>$14=$C%m$E$\|$44>$Lmit %m %m n. %m$E$\|$R%m$\|$L` | `Bringt` | qty | good | count | transporter | destination | status |
| 5 | `0xBE75C` `%m %J %m…mit…n.…` | `Bringt` then icon 0x1B6 | qty | good | count | transporter | destination | status |
| 6, 7 | `0xBE798` `$78<$R%m%M%M$E$\|$30>$Lmit %m %m aus %m$E$\|$R%m$\|$L` | `Holt Beute` (same for 6 and 7, no escort icon) | %M | %M | count `&s4`, 0x802 | transporter | location | status |

- Kind names: table `0xB64E0` = Kein Auftrag, Produziert, Verkauft, Verk., Bringt, Bringt, Holt Beute, Holt Beute. Production shows the per-good verb instead (Erntet, Fangt, Braut, Schert, Schlagt, Züchtet, Fördert, Keltert, Gerbt, Blast, Fördert, Webt, Schürft, Holt, Baut); preposition col `[EXE text]` = `0xB65DC[g]` (plain `%s`, not an item).
- Transporter (kinds 2..5): singular `0xB4614[t]` if s4 = 1 else `0xB4CC4[t]` (Karren/Karren, Schiff/Schiffen, Ballon/Ballons, Dampfk., Tauchb., Flugm.). Kinds 6/7: `0xB45FC[t]` / `0xB4CAC[t]` (full names: …/Dampfkarren, Tauchboot(en), Flugmaschine(n)).
- Destination: kinds 2..5 short city name `0xB4AF4[cityrec(s6).+0]` (Fürstenbr., Finsterw., Moortal, Lichterh., Greifenh., Bergstedt, Falkenh., Waldstein, Lauenb., Kronsbach); kinds 6/7 location name `0xB6194[s6]` (Hohenfels … Säbeltal).
- Example (A/B): kind 1, g = 1, s4 = s5 = 0 → `[EXE text]`.

## 3. Per-redraw normalisation and spinner ranges [C]

Before the panel is drawn, `0x45820` (§6) runs, then per row:
- **Kind 1**: `item_range(workers, 0, 40)`. Means keeps the default range of a 2-digit spinner (0..99).
- **Kinds 2..5**: `n = ceil(s1 / CAP[s5])` (as `(s1 + CAP − 1) / CAP`, unsigned; s1 = 0 → 0). If s7 = 2: `owned[s5] += s4 − n`. Then `s4 := n` (always). So the count is derived from qty; the count item (0xC02) has no lasting effect even if clicked [flag 0x400 likely makes it inert: UNVERIFIED]. `r = (s7 == 2)`; `L = CAP[s5] · (owned[s5] + r·s4)`; `A = stock[s3] + r·s1`; `item_range(qty, 0, min(L, A))`.
- **Kinds 6/7**: `item_range(count, 0, owned[s5] + (s7 == 2)·s4)`.

## 4. Clicks [C]

`delta` = the menu delta (cycling string ±1; spinner ±10^d, clamped by `menu` to the item range). Clicks on `%M` placeholders and plain text do nothing. After any panel click the handler waits for the mouse button release (status and kind clicks only), frees the panel items and redraws the panel (§3, §6).

### 4.1 Col 0: kind
1. If s7 = 2: unreserve (status byte keeps 2 for now).
2. `S = Σ stock[g]`, `T = Σ owned[t]` (after the unreserve).
3. `k = s0`; repeat up to 8 times: `k = (k + 8 + delta) mod 8`; stop at the first acceptable k:
   - 0, 1: always;
   - 2, 3: T > 0 and S > 0;
   - 4, 5: T > 0, S > 0, and the owner has a branch found by `0x42A50(0, any, o)` **and** a second one (`0x42B10`) [L: "at least two branches"];
   - 6, 7: T > 0 and the owner owns a castle (`0x42A50(2, any, o)`) or a camp (`0x42A50(1, any, o)`).
4. Defaults for the new k (other bytes are kept, including s1, s4, s5 left over from the previous kind; a never-used slot is all zeros):
   - k = 1: `s3` = first good g (0..14) with city `E[c][g] > 100` (`0x85F17 + 0x8C·c + g`), else 0.
   - k = 2..5: if s3 ≥ 15 → 0; then up to 15 times: if `stock[s3] == 0`, `s3 = (s3 + 1) mod 15`.
   - k = 2..7: if s5 ≥ 6 → 0; up to 6 times: if `owned[s5] == 0`, `s5 = (s5 + 1) mod 6`.
   - k = 2, 3: if s6 ≥ 10 → 0; up to 10 times: advance `s6 = (s6 + 1) mod 10` while `s6 == c` or (s5 is a ship and `NAV[s6] == 0`).
   - k = 4, 5: as 2/3, and additionally advance while the owner has no branch at s6 (`0x42A50(0, s6, o)`).
   - k = 6, 7: if s6 ∉ 14..20 → 14; up to 7 times: advance `s6 = (s6 − 13) mod 7 + 14` while `s6 == c` or the owner has neither castle nor camp at s6. (No ship rule here.)
5. `s0 := k`. Status: if it was 2 → `feasible ? reserve (stays 2) : s7 := 0`; else `s7 := feasible` (0/1).

Consequences: kinds 0 and 1 are always reachable, so "Kein Auftrag" is reached by stepping over 0 (1 → 0 on −1, 7 → 0 on +1, or skipping rejected kinds). In a branch with no stock or no transporters the kind toggles only between Kein Auftrag and production. This matches the A/B: (320,328) is the upper half of row 1 (+1), 0 → 1, new good = first producible (Fisch), s4 = s5 = 0, feasible → status 1 (empty circle), cost unchanged (only status 2 is costed); the second click at (320,328) hit no item on the production row; a +1 click on the production verb went 1 → 2..7 (all rejected) → 0. A click at x = 40 on the "Kein Auftrag" row misses the centred padded item [L; glyph widths not measured].

### 4.2 Col 1: quantity (kinds 2..5 only; col 1 is %M otherwise)
`s1 += delta`; if s7 = 2: `stock[s3] −= delta`. `n = ceil(s1 / CAP[s5])`; if s7 = 2: `owned[s5] += s4 − n`; `s4 := n`. No status change here (the redraw refresh may change it).

### 4.3 Col 2: good
- Kind 1: up to 15 times `s3 = (s3 + 15 + delta) mod 15` until `E[c][s3] > 100`.
- Kinds 2..5: `g = s3`; up to 15 times `g = (g + 15 + delta) mod 15` until `stock[g] > 0` (current stock, i.e. without this slot's reservation). If s7 = 2: `stock[s3] += s1`; if `stock[g] ≥ s1` then `stock[g] −= s1`, else `s7 := 0` and `owned[s5] += s4`. `s3 := g`.
- Kinds 6/7: no item.

### 4.4 Col 3: workers / count
`s4 += delta` (any kind); if s7 = 2 and kind ≥ 2: `owned[s5] −= delta`. Production: workers 0..40. Loot: count 0..owned + reserved. Transport: overwritten by §3.

### 4.5 Col 4: means / transporter
- Kind 1: `s5 += delta` (means, 0..99).
- Kinds 2..7: `t = s5`; up to 6 times `t = (t + 6 + delta) mod 6` until `owned[t] > 0`. New count `n = ceil(s1 / CAP[t])` for kinds 2..5, `n = s4` for 6/7. If s7 = 2: `owned[s5] += s4`; if `owned[t] ≥ n` then `owned[t] −= n`, else `s7 := 0` and (kinds 2..5) `stock[s3] += s1`. `s4 := n`, `s5 := t`. The destination is **not** re-validated against the ship rule (the refresh catches it through feasibility).

### 4.6 Col 5: destination (no status change at the click)
- Kinds 2, 3: repeat **without limit** `s6 = (s6 + 10 + delta) mod 10` until `s6 ≠ c` and (s5 not a ship or `NAV[s6]`).
- Kinds 4, 5: up to 10 times the same step, until `s6 ≠ c`, owner has a branch at s6, and (not a ship or `NAV[s6]`).
- Kinds 6, 7: up to 7 times `s6 = (s6 − 7 + delta) mod 7 + 14`, until `s6 ≠ c`, owner owns a castle or camp at s6, and (not a ship or `NAV[s6]`; castles 14..16 count as navigable, camps not).
- If no target passes, the loop ends back at the start value.

### 4.7 Col 6: status glyph
- s7 = 1 → reserve, `s7 := 2` (no feasibility call here; status 1 is current because every redraw refreshes).
- s7 = 2 → unreserve, `s7 := feasible` (normally 1). A/B: (615,300) unconfirmed slot 0, cost line 0.
- s7 = 0 or 3 → nothing.

Reserve (`0x49370`): kinds 2..5 `stock[s3] −= s1`; kinds 2..7 `owned[s5] −= s4`; production reserves nothing. Unreserve is the inverse (`rules-02` §4.1).

## 5. Feasibility `0x45360` as used here [C]
- Kind 0 (and any kind not handled) → **1** (feasible). A "Kein Auftrag" slot therefore carries status 1 after a refresh; it is never drawn and never executed.
- Kind 1: (owner certificate level `person[o]+5` ≥ g/5, or director's ≥ g/5) and `E[c][g] > 100`. Workers and means are not checked (0 workers is feasible).
- Kinds 2..7: `rules-02` §4.1 (qty > 0, counts, ship/navigable rule, branch/castle at destination).

## 6. Refresh `0x45820` (branch entry and every redraw) [C]
For all **4** slots: status 2 → unreserve; feasible ? reserve (2) : 0. Otherwise: infeasible → 0; feasible → 1, except status 3 stays 3.

## 7. Auslastung and Kosten [C]

```
cap  = (u16 b+0x23 + u16 b+0x25) * 17 + 1          storage m² wood + stone
Auslastung = (Σ_{g<15} stock[g]) * 100 / cap        unsigned; reserved goods are not in stock
Kosten = Σ over slots 0,1 with s7 == 2 of:
  kind 1:    t = WB[g]*w                              WB u16 at 0x8653C + 15g = 50,55,…,120
             t = WF[W_c/10] * t / 100                 WF 0x8668C = 92,94,…,110; W_c = city +0x12
             t = (wage_c + 450) * t / 500             wage_c = city +0x7F
             t = (pay_o + 150) * t / 250              pay_o = person +0xF0
             t = I * t / 1000                         I = dword 0x85E21
             cost = t + m * MC[g]                     MC dword 0x86650 = 100,200,150,50,500,100,500,100,100,300,700,300,1000,1000,1000
  kinds 2..5: (TC[t] * s4 * (2 + (kind & 1))) >> 1
  kinds 6, 7: (TC[t] * s4 * (2 + (kind == 7))) / 2
```
Signed divisions for kind 1; no random terms and no toll. This is a forecast; the bill uses `rules-02` §3.2 / §4.2 (different wage formula, `rand` terms, toll). Status 1 and 3 slots cost 0 in the display.

## 8. Corrections to `formats.md` §11.3
- The "buy" row (`$78<$R%m%M%M…aus %m`) is the **loot fetch** (kinds 6/7, "Holt Beute … aus <castle/camp>"); there is no buy order.
- "one-way" is the production row; "route" rows are kinds 2..5 (`%J` = escort icon 0x1B6 for 3 and 5).

## 9. Open
- Item flags 0x800 and 0x400 (count spinner), item byte +9 bit 0x40 [UNVERIFIED].
- Pixel extents of row items (glyph widths, padded "Kein Auftrag" item) not measured.
- Shapes of status glyphs 0x0D and 0x0F [UNVERIFIED].
- `0x42B10` semantics ("second branch") [L].
