# Economy: prices, production, storage, transport, costs

Confidence tags as in `rules-00-map.md`. Record, person, property and city field offsets are in `rules-00-map.md`. All arithmetic is 32-bit integer; "/" truncates toward zero; where the original divides unsigned this is noted, but all operands here are non-negative so signed and unsigned division agree unless stated.

Notation: `Y` = current year. `c` = city 0..9, `g` = good 0..14, `t` = transporter type 0..5. `rand(n)` = `random_n(n)` from `rules-01-rng-calendar.md`.

## 1. Constant tables

### 1.1 Goods (`0x86532`, 15 bytes each) [C]

| g | good | demand weights by wealth decile 0..9 | wage base `WB[g]` | value `V[g]` | output base `OB[g]` | means per 10 workers divisor `R[g]` | means cost `MC[g]` |
|---|---|---|---|---|---|---|---|
| 0 | Korn | 120 116 112 108 104 100 96 92 88 84 | 50 | 7 | 155 | 50 | 100 |
| 1 | Fisch | same as Korn | 55 | 8 | 221 | 30 | 200 |
| 2 | Bier | same | 60 | 9 | 218 | 20 | 150 |
| 3 | Wolle | same | 65 | 10 | 239 | 5 | 50 |
| 4 | Holz | same | 70 | 11 | 186 | 70 | 500 |
| 5 | Rind | 84 88 92 96 100 104 108 112 116 120 | 75 | 12 | 211 | 10 | 100 |
| 6 | Erz | same as Rind | 80 | 13 | 200 | 50 | 500 |
| 7 | Wein | same | 85 | 14 | 192 | 10 | 100 |
| 8 | Fell | same | 90 | 15 | 280 | 5 | 100 |
| 9 | Glas | same | 95 | 16 | 177 | 30 | 300 |
| 10 | Salz | same | 100 | 17 | 182 | 60 | 700 |
| 11 | Tuch | same | 105 | 18 | 209 | 20 | 300 |
| 12 | Silber | same | 110 | 19 | 179 | 80 | 1000 |
| 13 | Pfeffer | same | 115 | 20 | 164 | 90 | 1000 |
| 14 | Waffen | same | 120 | 21 | 256 | 40 | 1000 |

`D[g][k]` = demand weight for wealth decile k. `R[g]` is at `0x86614` (dword), `MC[g]` at `0x86650` (dword). Optimal means for `w` workers: `w*10 / R[g]`. This reproduces Appendix II exactly (Korn 5:1, Fisch 3:1, Bier 2:1, Wolle 1:2, Holz 7:1, Rind 1:1, Erz 5:1, Wein 1:1, Fell 1:2, Glas 3:1, Salz 6:1, Tuch 2:1, Silber 8:1, Pfeffer 9:1, Waffen 4:1).

### 1.2 Transporters [C]

| t | type | capacity `CAP[t]` (`0x86710`) | trip cost `TC[t]` (`0x86728`) | price base `PB[t]` (`0x86740`) | build time years (`0x86758`) | needs water |
|---|---|---|---|---|---|---|
| 0 | Karren | 100 | 200 | 100 | 1 | no |
| 1 | Schiff | 1000 | 1800 | 1000 | 3 | yes |
| 2 | Ballon | 50 | 250 | 300 | 1 | no |
| 3 | Dampfkarren | 80 | 250 | 300 | 2 | no |
| 4 | Tauchboot | 700 | 2200 | 3000 | 3 | yes |
| 5 | Flugmaschine | 30 | 150 | 500 | 2 | no |

Capacities match the handbook table (H p.14).

Navigable locations (`0x85A00`) [C]: Fürstenbrück, Lichterheide, Greifenheim, Bergstedt, Lauenburg, Kronsbach and the three toll castles. Not navigable: Finsterwalde, Moortal, Falkenhorst, Waldstein and all four robber camps. Matches H p.14.

### 1.3 Other constants [C]
- Wage factor by city wealth decile (`0x8668C`): `WF[k]` = 92, 94, 96, 98, 100, 102, 104, 106, 108, 110.
- Uncle's starting order by start city (`0x87070`, `0x8707A`, `0x87084`): good, workers, means = Fürstenbrück Wolle 4/8; Finsterwalde Holz 7/1; Moortal Korn 10/2; Lichterheide Fisch 6/2; Greifenheim Bier 6/3; Bergstedt Wolle 4/8; Falkenhorst Bier 6/3; Waldstein Holz 7/1; Lauenburg Fisch 6/2; Kronsbach Korn 10/2. Status "confirmed". Falkenhorst Bier 6/3 matches the handbook example (H p.13).
- Initial city data (data object, `0x85F04`): population Fürstenbrück 10000; Finsterwalde, Greifenheim, Waldstein 5000; the other six 2500. Wealth level: 70 / 60 / 40 for the same three groups. Per-good yields are fixed per city (table in section 1.4).

### 1.4 City start data and yields `E[c][g]` (city +0x0E, +0x12, +0x13) [C]

Dumped from the data object. Each city has the yield values 25, 50, 75, 120, 140 three times each. Only goods with yield >= 101 (the six goods with 120 or 140) can be produced there (H p.11: six goods per city).

| city | pop | wealth | Korn | Fisch | Bier | Wolle | Holz | Rind | Erz | Wein | Fell | Glas | Salz | Tuch | Silber | Pfeffer | Waffen |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Fürstenbrück | 10000 | 70 | 120 | 50 | 75 | 140 | 25 | 120 | 75 | 50 | 25 | 140 | 50 | 75 | 25 | 120 | 140 |
| Finsterwalde | 5000 | 60 | 25 | 50 | 120 | 75 | 140 | 25 | 75 | 120 | 140 | 50 | 25 | 140 | 75 | 50 | 120 |
| Moortal | 2500 | 40 | 140 | 75 | 25 | 120 | 50 | 120 | 50 | 140 | 75 | 25 | 120 | 140 | 25 | 50 | 75 |
| Lichterheide | 2500 | 40 | 120 | 140 | 25 | 50 | 75 | 140 | 25 | 75 | 50 | 120 | 75 | 120 | 50 | 140 | 25 |
| Greifenheim | 5000 | 60 | 50 | 120 | 140 | 25 | 75 | 50 | 120 | 75 | 25 | 140 | 50 | 25 | 120 | 75 | 140 |
| Bergstedt | 2500 | 40 | 25 | 120 | 75 | 140 | 50 | 25 | 140 | 50 | 75 | 120 | 140 | 25 | 120 | 75 | 50 |
| Falkenhorst | 2500 | 40 | 50 | 25 | 140 | 75 | 120 | 50 | 140 | 25 | 120 | 75 | 120 | 50 | 140 | 25 | 75 |
| Waldstein | 5000 | 60 | 75 | 25 | 120 | 50 | 140 | 75 | 50 | 120 | 140 | 25 | 140 | 50 | 75 | 25 | 120 |
| Lauenburg | 2500 | 40 | 75 | 140 | 50 | 25 | 120 | 75 | 25 | 140 | 120 | 50 | 75 | 120 | 50 | 140 | 25 |
| Kronsbach | 2500 | 40 | 140 | 75 | 50 | 120 | 25 | 140 | 120 | 25 | 50 | 75 | 25 | 50 | 140 | 120 | 75 |

Handbook cross-check of the best (140) locations (H p.43-46): Korn Moortal, Kronsbach ✓; Fisch Lichterheide, Lauenburg ✓; Bier Greifenheim, Falkenhorst ✓; Wolle Fürstenbrück, Bergstedt ✓; Holz Finsterwalde, Waldstein, handbook says Finsterwalde, Greifenheim ✗; Rind Lichterheide, Kronsbach ✓; Erz Bergstedt, Falkenhorst ✓; Wein Moortal, Lauenburg ✓; Fell Finsterwalde, Waldstein, handbook says Finsterwalde, Greifenheim ✗; Glas Fürstenbrück, Greifenheim ✓; Salz Bergstedt, Waldstein ✓; Tuch Finsterwalde, Moortal ✓; Silber Falkenhorst, Kronsbach ✓; Pfeffer Lichterheide, Lauenburg ✓; Waffen Fürstenbrück, Greifenheim (handbook names none). Holz and Fell disagree with the handbook; the code values are authoritative for parity.

Other start values (same for all cities unless noted): +0x6D siege 0; +0x6E: 7 (Fürstenbrück, Lichterheide, Lauenburg, Kronsbach), 5 (Finsterwalde, Moortal, Waldstein), 13 (Greifenheim, Bergstedt, Falkenhorst); +0x6F level A 20; +0x71 level B 25; transporter prices 100 each until the first update; +0x7F..+0x82 50 each; +0x83 sales tax 15; +0x84 12000 (Fürstenbrück), 8000 (Finsterwalde, Greifenheim, Waldstein), 5000 (others); +0x88 10 / 5 / 0 for the same groups.

## 2. Prices

### 2.1 Initial prices (`prices_init`, 0x49EF0) [C]
For every city c and good g:

```
price[c][g] = (V[g] * D[g][W_c / 10] * 3) / (E[c][g] + 200) + 1
```
`W_c` = city wealth level (+0x12). Stored as a byte.

### 2.2 Price update (`prices_update`, 0x49FE0) [C, instruction-level check]

Called once by `world_init` and then at the start of every human's `turn_end_settlement`, i.e. several times per year.

Inflation index (global `0x85E21`):
```
I = 1000
for y in 1600 ..= Y:  I = I * 1004 / 1000        (integer, Y-1600+1 iterations)
I = I + 200
```
Exact values of this loop (each step truncates): 1600: 1204; 1650: 1404; 1700: 1642; 1747: 1912; 1761: 2002 (first year with I >= 2000); 1800: 2286; 1900: 3245; 1999: 4660. (Earlier versions of this doc quoted the untruncated 1.004^n values; those were wrong.)

Then for each city c and good g:
```
P = population[c]                                   (u32 at +0x0E)
if sold[c][g] > P: sold[c][g] = P                   (sold = u32 at +0x31+4g)
Rm = P - sold[c][g] * 5 / 3;  if Rm < 1: Rm = 1
s = (siege flag +0x6D != 0) ? 1 : 0
f = (bit 7 of +0x6E set)   ? 1 : 0
boost = 100 + (g == Holz and f) * 10 + (g == Waffen and s) * 15
div   = 100 - 20*s - 20*f
base  = (V[g] * D[g][W_c/10] * 20) / ((E[c][g] + 100) * 10)        (signed)
T1    = (I * price[c][g] * boost) / div                            (signed)
T2    = (base * 1000 * Rm) / P                                     (unsigned)
new   = (T1 + T2) / 2000                                           (unsigned)
price[c][g] = new mod 256, then read as a signed byte:
    if < 1:  price = 1
    if > 99: price = 99
```
Notes and quirks:
- The siege and fire modifiers raise all prices of the city through `div` (each by 25 %), and additionally Holz after a fire (+10 %) and Waffen during a siege (+15 %). The siege byte +0x6D is never written by the code, so in practice only the fire modifier is live (`rules-07` §6.7).
- Because the stored byte is reinterpreted as signed, a raw result of 128..255 becomes 1 instead of 99. With prices capped at 99 and the formula weights below, the raw result exceeds 127 only after late-game inflation or under siege; implement the wrap exactly.
- The old price is weighted by `I/2000`. Early in the game this is about 0.6, so prices relax toward `base * Rm / (2P)`. `I` first reaches 2000 in 1761; from then on the old price is kept or grows every update and prices drift to the cap regardless of sales. This is a property of the original, not a decoding error.
- `sold[c][g]` is reset to 0 for all cities at the start of every human's turn (in `turn_start_reports`) and then increased by that human's arriving sales. So the saturation term mainly reflects the most recent human's deliveries.

### 2.3 Other price writers [L]
- Sieges (0x26640) and disasters (0x18BD0) modify `price` directly (siege: clamped to 1..99). Covered in the events doc.

## 3. Production

### 3.1 Placing an order [C]
Order book slot (8 bytes, property +0x33 + 8k):

| byte | production (type 1) | transport (types 2..7) | castle/camp (0x11..0x13, 0x21..0x23) |
|---|---|---|---|
| 0 | 1 | type, see 4.1 | type |
| 1..2 | - | u16 quantity | 1: target index |
| 3 | good | good | troops type 0 |
| 4 | workers | transporter count | troops type 1 |
| 5 | means | transporter type | troops type 2 |
| 6 | - | destination location | troops type 3 |
| 7 | status: 0 impossible, 1 possible not confirmed, 2 confirmed, 3 executed this year | same | same |

Branches show 2 order rows (handbook: two orders per branch per year); the record has room for 4. The slot persists across years. After execution at turn end the status becomes 3 for production (type 1) and transports (types 2..5), and 1 for loot runs (6, 7) and castle attacks/transfers (rules-07 §3a). At the next turn start every status-3 slot is re-reserved (goods and transporters taken out of the branch again) and set to 2, so these orders repeat every year without re-confirmation; then each turn start re-checks feasibility: a confirmed order that is no longer feasible drops to status 0, a feasible unconfirmed order shows status 1 (`branch_yearly_wear` 0x45820 with `order_feasible` 0x45360).

Production order feasible iff [C]:
- the owner's certificate level (person +0x05) `>= g / 5`, or the branch director's certificate level `>= g / 5`. Directors are computer persons, whose level is always 0 (`rules-00` §4), so in practice only the owner's level counts. So goods 0-4 need level 0, goods 5-9 level 1, goods 10-14 level 2 (Handelszertifikat I, II, III of Appendix II).
- `E[c][g] >= 101`.

### 3.2 Order execution and wage cost (`turn_end_settlement`) [C, instruction-level check]
For each confirmed production slot (status 2) of every branch of the player, a record of type 8 is created with (good, workers `w`, means `m`) and the slot is set to status 3. Costs:

```
adj = (Y < 1630) ? (1630 - Y) * 4 : -(Y - 1630)
t1 = (WF[W_c / 10] * (w * WB[g])) / 100
t2 = ((I - adj) * t1) / 10
t3 = ((wage_level_c + 450) * t2) / 5000          wage_level_c = city byte +0x7F (start 50)
wages += ((pay_workers + 100) * t3) / 2000        pay_workers = person +0xF0, percent
means_cost += m * MC[g] + rand(20)
```
(`/5000` and `/2000` are each done as two successive divisions, /500 then /10 and /200 then /10, which gives the same result for non-negative values.)

Example, 1600, wealth 40, wage level 50, pay 100 %: 10 Korn workers cost 542, two fields 200..219.

The order book screen shows a forecast with a different formula (0x4FA70): `I * ((pay+150) * ((wage_level+450) * (WF*WB*w/100) / 500) / 250) / 1000 + m * MC[g]`. The forecast is not what is charged. [L]

### 3.3 Output (`turn_start_reports` 0x4B900, next turn of the same player) [C, instruction-level check]

Executed at the start of the owner's next turn, for each type-8 record:

```
yearF = (Y < 1620) ? 155 - (Y - 1600) : 135        Y = the new year
weather[g] = 70 + rand(60)                        rolled once per good per human turn start,
                                                  before any record is processed, g = 0..14 in order
cityF = E[c][g] + 200 + bonus10[c] - 60 * blocked[c]
    blocked[c] = 1 if any type-25 record has byte 4 == c (disaster/siege, see events doc)
    bonus10[c] = 10 if any type-23 record has byte 4 == county of c
skill = branch_effective_skill(branch, g)
t1 = ((skill + 250) * (yearF * w * OB[g])) / 350000
t2 = ((pay_workers + 150) * t1 * cityF) / 75000
out = ((weather[g] + 200) * t2) / 300
opt = w * 10 / R[g];  if opt == 0: opt = 1
if m < opt:   out = m * out / opt
if m > opt:   out = out + MC[g] * (m - opt) / (2 * price[c][g])
T = 30 + rand(10)                                  one roll per production record
if w > T:     out = out * T / w + out * (w - T) / (2 * w)
```
Then `branch_add_goods(branch, g, out mod 65536)` (the amount is passed as 16 bits).

`branch_effective_skill(branch, g)` [C]: the owner's skill `person[owner].0xE0+g`; if a director is employed and his skill for g is higher, the director's. For g = Wein, if the owner holds ability record code 44 (checked through 0x5E600; likely a sub-plot reward), the result is 150 regardless.

Skill gain [C]: after the production screen, for each good with total output > 0 this turn, the owner's skill for that good rises by `1 + rand(3)`, capped at 150. Only the owner learns, not the director. Quirk [L]: when more than 13 (11 if goods were given away) different goods were produced, the goods beyond the displayed list are handled by a second loop that passes the index plus one, so the next good's skill rises instead, and index 15 would hit person byte 0xEF (director pay level).

Harvest message: quality text index = `(59 - (weather - 70)) / 12` into ausgezeichnet, gut, normal, mäßig, schlecht.

Handbook cross-check: the handbook staffing prose (H p.43-46) gives ranges such as "Korn: 2 fields at most 12 men, fewer than 8 weak". In the code the only staffing rules are the means ratio and the per-order 30..39 worker threshold; there is no lower limit. [handbook prose is flavour]

## 4. Transport and sales

### 4.1 Transport order types [C]
| type | meaning | handbook |
|---|---|---|
| 2 | sell at destination | Verkauft |
| 3 | sell at destination, escorted | Verkauft (X) |
| 4 | deliver to own branch at destination | Schafft |
| 5 | deliver, escorted | Schafft (X) |
| 6 | fetch loot from own castle/camp | Holt Beute |
| 7 | fetch loot, escorted | Holt Beute (X) |

Feasibility (0x45360) [C]:
- types 2/3: `0 < qty <= stock[g]`, `qty <= CAP[t] * count`, `0 < count <= owned[t]`, destination != own city, and if t is Schiff or Tauchboot the destination must be navigable.
- types 4/5: as 2/3 and the player must own a branch at the destination.
- types 6/7: the player must own a castle or camp at the destination, `0 < count <= owned[t]`, water rule for ships. The loot amount is not checked.
- The UI sets `count = ceil(qty / CAP[t])` automatically (0x4FA70).

Confirming an order reserves its goods and transporters (removed from the branch) (0x49370); unconfirming returns them (0x49430).

### 4.2 Execution at turn end [C]
For each confirmed transport slot a type-9 record is created: (kind, qty, good, count, t, destination, condition = branch condition of t). Costs added to the player's bill:

- types 2..5: `transport += TC[t] * count + rand(10)`; escorted (odd types) additionally `transport += (TC[t] * count + rand(10)) / 2` (second, separate roll).
- types 6/7: the good with the largest `stock * V[g]` at the castle/camp is chosen; `qty = min(stock, TC[t] * count)`. Quirk [C]: the cap uses the trip-cost table, not the capacity table, so loot runs carry far more than the transporters' capacity (Karren 200 instead of 100, Flugmaschine 150 instead of 30). Cost `((type == 7 ? 3 : 2) * (TC[t] * count + rand(10))) / 2`. The record is created with kind 4/5 (deliver) to the branch city.
- Toll: if destination != Fürstenbrück and t is not Ballon, Tauchboot or Flugmaschine: `county = 0x868D4[dest]`, `k = county - 10`, `toll = rate[k] * qty * V[g] / 100` where `rate[k]` = toll rate of castle k (castle table +0x14). The player pays `toll`; castle k's toll income accumulator (+0x15, u32) receives `(100 - share[k]) * toll / 100`; the county's Kanzler (office index 6 in that county, if present) receives `share[k] * toll / 100` directly, `share` = `0x85A48` = 25, 25, 25. Karren, Schiff and Dampfkarren pay toll.

### 4.3 In transit (same turn end, after the bill) [C]
Each of the player's type-9 records goes through `shipment_mishaps` (0x174C0), in this order:
1. `cond = cond * (91 + rand(9)) / 100`.
2. if `cond <= rand(80)`: `qty = qty * (70 + rand(30)) / 100`.
3. if `cond <= rand(90)` and `count > 1`: `count -= 1` (one transporter lost, with message).
4. if `rand(100) < 5`, kind is sell (2 or 3) and `count > 1`: for Ballon "Ungünstige Winde", for Dampfkarren "Die Lenkung", the destination becomes `(dest + 1 + rand(9)) mod 10`. Other types roll but are unaffected. Quirk: a single transporter is never blown off course.

Robber attacks on shipments happen in `settlement_pre` (0x61410) of the robber camp owner; see the bandits doc.

### 4.4 Arrival (`turn_start_reports`, the shipper's next turn) [C]
For each type-9 record of the player, in heap order:
- If the destination city or the source city is blocked (type-25 record), goods and transporters return to the source branch.
- Sell (kinds 2, 3): `revenue = qty * (price[dest][g] + b)`, `b = 2` if the player has a type-42 record with byte 3 < 7, else 0. Revenue is added to `sales[g][dest]` (`0x85A62`, reset each turn start) and to the player's cash at the end of the arrival loop; `sold[dest][g] += qty`; transporters return to the source branch with the shipment's condition (weighted average with the branch's existing condition, `branch_add_transporters`).
- Deliver (kinds 4, 5): if the player still has a branch at the destination, goods and transporters go there; otherwise back to the source branch.
- Kinds 0, 1 do nothing (deleted).

The sale price is the destination price at the time of arrival, i.e. after other humans' turns in between have moved it.

### 4.5 Taxes and tithe on sales (`turn_end_settlement` 0x618C0) [C]
Computed from this turn's `sales[g][c]` (revenue of the arrivals at the start of the same turn):
- Sales tax: unless the player has ability code 57 (0x39), for each city c: `tax_c = sum_g (rate_c * sales[g][c]) / 100` (division per good), `rate_c` = city byte +0x83 (start 15 everywhere). The amount also goes into the city treasury `0x85E25[c]`.
- Church tithe: unless the player is without religion (person +0x03 == 2) or has ability code 11, `tithe_c = sum_g (tithe[k] * sales[g][c]) / 100`, `k = (c - 1) / 3` for c >= 1 and 0 for Fürstenbrück, `tithe` = `0x85A42` = 10, 10, 10. Added to `0x85E4D[c]`.

Handbook disagreement: the law text says tax is "the seventh part" (14.3 %, H p.53). The code starts every city at 15 %. The tithe of 10 % matches.

## 5. Storage

### 5.1 Capacity and overflow (`branch_add_goods` 0x439F0) [C]
Adding `n` units of g:
1. If `stock[g] + n < 10000`: `stock[g] += n`. Otherwise `stock[g] = 9999` and the given-away counter (`0x85CC1`) increases by `9999 - n` (quirk: not by the real excess).
2. `cap = (wood_m2 + stone_m2) * 17 + rand(100)` (one roll per call).
3. While total stock > cap: find the held good with the lowest current local price among goods whose price is below 100 (strictly `< 100`, all prices are <= 99 so in practice all held goods); give away `min(excess, stock)` of it (counter increases). If no good qualifies, stop.

Handbook says 15 to 20 units per m² depending on the good (H p.15). The code uses 17 per m² for all goods plus 0..99 slack. [code wins]

### 5.2 Buying and selling storage (0x531A0, offers in 0x4B900) [C]
- Every turn start, for each city, 3 offers are rolled for the player: size `k = 0,1,2`: `((20k + 15 + rand(10)) * S) / 100 + 1`, `S` = player's current storage m² in that city (15 if he has no branch there), capped at 4999. With `rand(100) < 40` the offer is stone, else wood.
- Price per m²: wood = city level A (+0x6F), stone = city level B (+0x71); buying and selling use the same price. A purchase is capped so that the type's m² stays <= 4999.
- Levels, recomputed in `prices_update` [C]:
  `A = (Y-1600)/7 + lvl82/11 + 23 + extra88 + rand(12)`, `B = (Y-1600)/7 + lvl82/11 + 30 + extra88 + rand(12)` with `lvl82` = city byte +0x82 (start 50) and `extra88` = city dword +0x88 (start 10 / 5 / 0, +1 with 7 % per year).

## 6. Transporters (0x53A60) [C]
- Price `TP[c][t] = ((Y - 1550) * PB[t]) / 50`, recomputed in `prices_update` (1600: Karren 100, Schiff 1000, Ballon 300, Dampfkarren 300, Tauchboot 3000, Flugmaschine 500). Ships and submarines cannot be built or used in non-navigable cities.
- Build n: pay `TP * n`; a type-10 record with byte 3 = build time delivers them at a later turn start with condition 100. Total built plus pending is limited to 99 per type.
- Sell n: receive `cond * n * TP / 200` (half value at full condition).
- Repair n: pay `(100 - cond) * n * TP / 100`; the transporters leave and return at the next turn start (type-11 record) with condition 100, or `100 - rand(30)` if any type-39 record of another owner exists [G: sub-plot "wheelwright cartel"]. Quirk: in the combined menu for type t+3 (Dampfkarren, Tauchboot, Flugmaschine) the repair cost uses the condition and price of type t.
- Auction (needs an announced auction): moves n to a type-12 record (auction doc).
- `branch_add_transporters` caps each type at 99; the condition is the count-weighted average of old and new, computed before the cap.

## 7. Other yearly costs and income in `turn_end_settlement` [C unless marked]

Order of evaluation (one bill, paid at once with `person_pay`, cash may go negative):
1. `prices_update`; title awards (career doc); castle/camp troop pay and training (castles doc).
2. Order execution (sections 3.2, 4.2): wages, means, transport, toll.
3. Robber/escort battles (`settlement_pre`).
4. Directors: for each branch with a director: `fee = ((director_fee_pct * budget) / 100) * pay_directors / 100` where `director_fee_pct` comes from 0x458F0 (director's ability, [L]) and `budget` = property +0x54.
5. Sales tax and tithe (4.5).
6. Mercenary and robber pay (castles doc).
7. Spies: sum of `amount` (+5) over the player's type-6 records with byte 9 != 0. Saboteurs: sum over type-7 records.
8. Loan interest: unless ability code 57, `sum over type-56 records of amount * rate / 100`.
9. Insurance: `popcount(person +0x07 & 0x0F) * net_worth / 100`, i.e. 1 % of net worth per insurance.
10. Pay the bill. Then, if the number of the player's branches exceeds `MaxBr`, an extra charge of `(branches - MaxBr) * 7 * net_worth / 100`. `MaxBr` (`0x85A2C`) = `(n + 9) / n` for n starting players: 1 player 10, 2: 5, 3: 4, 4: 3, 5 or 6: 2. [C; the purpose text is not yet tied, L: administration cost]
11. Office income `office_income` (0x5E500): `salary[o] * Y * L / 300000` with `L = 2 * city byte +0x80 + 300` for city offices and 400 otherwise. `salary` (`0x87264`) by global office index: Ratsherr 0, Baumeister/Richter/Kämmerer 250, Bürgermeister 500, Priester/Domherr 0, Geheimrat 400, Hauptmann/Justiz-/Finanzberater 750, Kanzler 1000, Vogt 2000, all county church offices 0, Hofrat 1000, Marschall/Justiz-/Finanzminister 2500, Großkanzler 5000, Regent 10000, principality church offices 0. Church offices earn nothing here.
12. Toll income of owned castles: `acc[k] += (rate[k] * 7 * F1 * I) / 400000` (64-bit product, low 32 bits used; `F1` = global `0x85E1D`, start 100); then the castle's county Vogt (if any and not the player) gets `acc * 30 / 100`; the player gets `acc - acc * share[k] / 100 - vogt_part`; `acc` is reset. [L: the Kanzler share was already paid per shipment in 4.2; here it is subtracted again but not paid to anyone.]
13. Tips (option value 0x008 "Spieltipps"): one random check of five (`rand(5)`): any owned transporter type with condition <= 50 %; no residence at any branch and year >= 1610; cash above half of net worth; remaining life (person +0x02) < 15 and no spouse (+0x04 == 0xFF) and no type-31 record (H p.9: warning when the screen life nears its end without children); a production order with a bad ratio (`R[g] * 14 * m < 100 * w`, or `140 * w < R[g] * 10 * m`, or more than 49 workers).
14. Fate events (option value 0x004 "Das Schicksal"; function 0x226F0, see `rules-06-events.md` §4): one, then with `rand(100) < 50` a second. Sub-plots (option 0x002 "Abenteuer") start elsewhere (rules-06 §5).
15. Blackmail expiry: for each person whose favour toward the player is above 100, `rand(100) < 33`: favour = `rand(30)`, and with `rand(100) < 50` that person retaliates (0x5B140, code 3). [L: favour > 100 encodes "being blackmailed by"]
16. Record countdown: all records owned by the player get byte 3 decremented, then per-type handlers run (loans due, fines, sieges, feasts, courtship, births, sub-plots, troop arrivals). Unknown types are deleted when byte 3 reaches 0.
17. Debt seizure check (0x64108).

## 8. Net worth (0x44640, 0x44340) [C]
`net_worth = cash + sum over owned properties of value(property) - sum of loan amounts (type-56 records)`.

Branch value:
```
rights[c] (+0x84) + A * wood_m2 + B * stone_m2 + residence_value + sum_g price[c][g] * stock[g]
+ sum_t cond[t] * count[t] * TP[c][t] / 100
+ for confirmed orders: goods reserved by transport orders at local price, transporters reserved at condition value
+ population[c]
```
Quirk: every branch adds the full city population to the owner's net worth.

Castle value: `4000 + security * 250 + casemates * 200 + sum_k troops[k] * merc_wage[k]`. Camp value: `500 + camouflage * 100 + straw * 80 + sum_k troops[k] * robber_wage[k]`. Wages: mercenary 100, 125, 125, 150; robber 75, 100, 100, 125 (`0x85A4E`, `0x85A56`).

## 9. Starting position [C]
- Every new person, human or computer, starts with 750 cash (handbook p.38 says 750, p.47 says 500: the handbook disagrees with itself; code: 750).
- A human starts with one branch (15 m² wood storage, 1 Karren at condition 100) and the uncle's confirmed production order (1.3).
- A player joining after 1600 receives `average net worth of the other humans - own net worth` as cash if positive (H p.8 ✓).

## 10. Turn-end tail: debt seizure and jury court (0x64108 .. 0x6540A) [C unless marked]

Runs at the very end of every human's `turn_end_settlement`, after the record countdown. Order:
1. Sub-plot starter (option Abenteuer, see `rules-06` §5).
2. Rival grudge: `r = human_rival_of(p)` (career doc 4.1); with `rand(100) < 25`: `retaliate(r, p, 3)` (`rules-09` §4).
3. Every duel record (type 22) of p: duel (`rules-08`). Then 0x447D0 / 0x44E00 (crime evidence update, `rules-04b`). Then every trial record (type 18) of p: trial (`rules-04b`).
4. Debt check (below).
5. Death and inheritance check (`rules-05`), unless the jury court ran (the court returns directly, see 4.6).

### 10.1 Trigger
`NW = net_worth(p)` (cash included, so negative cash lowers it). Seizure starts if `cash < 0` and `cash < -NW / 20` (C division, truncating toward zero). Then:
```
debt   = -NW / 20 - cash          (the amount by which cash is below the allowed overdraft of 5 % of net worth)
debt0  = max(0, debt)             (frozen for the jury formula)
NW0    = max(0, NW)               (frozen for the jury formula)
```
Message `[EXE text]`.

### 10.2 Stages
Six stages s = 0..5. In every stage the player's branches are visited in property-slot order (`property_find_first(kind 0, any city, p)` then next). A per-stage counter `n` (shown in the summary) starts at 0. `B` = number of branches (set in stage 0). Proceeds are added to p's cash and subtracted from `debt`. The stages run even when `debt` is already <= 0; each action checks `debt > 0` itself.

- **Stage 0**: `B += 1` per branch.
- **Stage 1, goods**: for g = 0..14 while `debt > 0`: `q = debt * 2 / price[c][g] + 1`, `q = min(q, stock[g])`; proceeds `price[c][g] * q / 2` (half price); `stock[g] -= q`; `n += q`. Summary: "Insgesamt %d Waren zum halben Preis".
- **Stage 2, transporters**: t from 5 down to 0, while `debt > 0`: if the branch has no transporter of type t, go to t-1; else take one: proceeds `TP[c][t] / 2` (condition is ignored), `count[t] -= 1`, `n += 1`. Summary "%d Transportmittel zum halben Preis".
- **Stage 3, storage**: for wood then stone while `debt > 0`: `q = debt / level + 1` (level = storage price A for wood, B for stone, section 5.2), `q = min(q, area * 2 / 5)` (at most 40 % of that area per branch); proceeds `level * q / 2`; `area -= q`; `n += q`. Summary "%d m² Lagerraum zum halben Preis".
- **Stage 4, residence**: if `debt > 0`: if a residence construction record (type 13) exists for the branch index `F` (see quirk): proceeds = half the value of the building under construction, record deleted, residence = none, message `[EXE text]`; else if the branch has a residence: a pending type-14 record for `F` is deleted, proceeds = half the residence value (0x44230), residence = none, message `[EXE text]`.
- **Stage 5, whole branch or jury**:
  1. If `debt <= 0`: `rand(50)`; if it is 0, skip this branch (no jury for it); otherwise continue.
  2. `x` = the branch with the lowest `net_worth_goods` value among this branch and all branches after it in slot order; `m` = that value (clamped >= 0).
  3. If `B <= 1` and `debt <= floor(m / 2)`: jury court.
  4. Else `rand(100)`: if `>= 50`, jury court; if `< 50` and `debt > 0`: seize x; if `< 50` and `debt <= 0`: jury court.
  - Seize x: proceeds `m` (the full value, not half); shipments in transit (type 9) of p whose source byte equals `F` are redirected to the first other branch y (first in slot order with y != x); message "Eure Niederlassung in %o" and, if any non-delivery shipments were redirected, `[EXE text]`; the branch slot is freed (0x429A0); `B -= 1`.

Quirk [C]: `F` is the property index of the player's **first** branch, computed once at the start of each stage and never updated inside the loop. So stage 4 only finds construction/renovation records of the first branch (for other branches it falls back to the "has a residence" case), and stage 5 redirects only the first branch's shipments. A port aiming at parity must copy this.

Consequence: once seizure starts, the jury court is reached in stage 5 at the first branch with probability about 1 - 1/50 if the debt is already covered, and otherwise 50 % per branch (100 % for a last branch whose half value covers the debt). The handbook says the creditors "always" follow a seizure with a jury trial (H p.55); in the code it is almost always. If the loop ends without a court, the summaries of stages 1..3 are shown and the turn continues with the death check.

### 10.3 Jury court
Court of the county of p's office location (Fürstenbrück pool → county 10). p may enter a bribe X (step input, paid immediately with `person_pay`, cash may go further negative).
```
T = max(1, NW0)
pg = (debt0 - X) * 100 / T + 50;  if pg < 0: pg = 0
```
12 jurors vote one after another. RNG order: first `rand(4)` (speaker voice), then for each vote: `rand(12)` picks a starting juror, the next juror that has not voted (cyclic +1) votes; `rand(100) < pg` → guilty, else not guilty; then `rand(3)` (next speaker voice). (The game also waits 1000/200 ms between steps, no RNG.)
- Fewer than 7 guilty votes: acquitted (text `[EXE text]`).
- 7 or more: debtors' tower: a type-20 record with counter 1 is created (the next turn is skipped, `rules-01` §3.1) and person byte +0xF3 gets bit 0x10 (remembered for the chronicle and the gravestone texts). Text variant: with `B < 2` `[EXE text]`.
- RNG in the waiting loop [C, parity hazard]: while the tower screen waits for a key, every frame calls `rand(100)` (and `rand(4)` if < 2) and `rand(100)` (and `rand(2)` if 0) for ambient sounds. The number of draws depends on how long the player waits. A deterministic port must not draw these, and will diverge from any recorded original session here.
- Either verdict: phase := 6 and the settlement returns immediately. The death and inheritance check is skipped for this turn [C, quirk].

Handbook cross-check: H p.55 (seizure of goods, transporters, storage, whole branches, then the jury, bribery possible, one year in the debtors' tower) ✓. The handbook does not give the 5 % overdraft rule or the half-price rule; the residence and goods texts in the game (`[EXE text]`, "zum halben Preis") match the code.

## Open questions
1. Meaning of record types 23, 25, 39, 42 (named from usage only).
2. Director fee function 0x458F0 and director AI 0x58410 (to be documented with the AI).
3. Toll-share double subtraction in 7.12.
4. Which humans' sales count toward `sold[]`: confirm by test that the reset happens per turn start, not per year (code reading is unambiguous; behaviour check only).
