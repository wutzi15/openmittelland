# Toll castles, robber camps, raids and tactical combat

Confidence tags as in `rules-00-map.md`. "Property" is the 96-byte record (`rules-00-map.md` 5); castles have kind 2, camps kind 1. `human(p)`, `rand(n)` (0..n-1) as in the other docs. Handbook = `docs/spec/06-combat.md`, `17-bandits.md`.

## 1. Castle and camp data [C]

Static table `0x8647C`, 26 bytes per record, index = location - 14: records 0..2 are the toll castles Hohenfels, Seefels, Waldenfels (text ids 2013..2015 at +0), records 3..6 the camps Meucheltann, Henkerswald, Dunkelforst, Säbeltal (2016..2019). So castle record k belongs to location 14+k, and the toll code pairs county 10+k with castle k: Mooringen-Hohenfels, Bergenhöh-Seefels, Auenthal-Waldenfels. This settles the question in `rules-00-map.md`: the code uses this pairing; the handbook's pairing disagrees.

| offset | meaning | values |
|---|---|---|
| +0x00 | u16 text id of the name | |
| +0x02..+0x0D | six u16, likely three map points | |
| +0x0E | u16 | 1000 everywhere |
| +0x12 | u16 year of construction ("Baujahr") | castles 1210, 1300, 1240; camps 0 |
| +0x14 | toll rate percent | castles 5 |
| +0x15 | u32 toll income accumulator (castles) | 0 |
| +0x19 | | order book shown: castles 1, camps 0 (rules-00 §10 image bytes; live A/B: a camp screen opens with the book hidden) |

Property fields used: +0x03 security (castle) / camouflage (camp), percent, start 60; +0x04 casemates (castle) / straw mats (camp), = troop capacity, start 40 / 20; stock and storage like a branch; +0x58+k troop count, +0x5C+k training (0..100) for the four troop slots k.

Troop slots and battle unit types:

| k | castle | camp | unit type castle / camp | wage (economy doc) |
|---|---|---|---|---|
| 0 | Söldner | Räuber | 0 / 5 | 100 / 75 |
| 1 | Musketier | Flintenschütze | 1 / 6 | 125 / 100 |
| 2 | Kanonier | Kanonier | 2 / 7 | 125 / 100 |
| 3 | Offizier | Bombenleger | 3 / 8 | 150 / 125 |
| (leader) | General | Räuberhauptmann | 4 / 9 | |

Value (economy doc, `net_worth_goods`): castle `4000 + 250*security + 200*casemates + sum troops*wage`, camp `500 + 100*camouflage + 80*straw + sum troops*wage`.

## 2. Owner actions

### 2.1 Expansion (0x56080) [L]
Two editors: security/camouflage (+0x03, raise by up to `100 - current`) and casemates/straw (+0x04, raise by up to `150 - current`). The price is the increase of the property value, i.e. 250 per security point, 200 per casemate (camp: 100 per camouflage point, 80 per straw mat), paid by the owner.

### 2.2 Recruiting (0x56760, settled at turn end) [C]
The owner invests money M (total per castle up to 9999 per turn); it is paid at once and kept in a type-15 record (owner, key = property index, counter 1, +4 u16 money). In the owner's turn-end record pass:
1. `M' = M*4/3` with privilege 36 (Hauptmann), else `M' = 2*M` with privilege 65 (Marschall), else `M' = M`.
2. `men = M' / (150 + rand(200))`; 0 → message only.
3. `add_troops(property, men, 30)` (0x45240): men are limited to `capacity (+0x04) - current total`; starting at slot `rand(4)` and going round-robin over the four slots, each man raises the slot count by 1 and sets its training to `(training*count + 30) / (count + 1)`.
4. The record is deleted.

### 2.3 Manoeuvre (0x56950) [C]
Shows morale = average training weighted by troop counts (50 with no troops), named by value/11. Cost `15 * total_troops + (year * 277) % 20`. On "Durchführen": pay; `k = rand(4)`; `training[k] += 5 + rand(10) + rand(m)` capped at 100, with m = 15 if the owner has privilege 37 (Hauptmann) or 66 (Marschall), else m = 1 (adds 0). The screen repeats, so several manoeuvres per turn are possible. Order-book reservations of orders 0x11..0x13 / 0x21..0x23 in state 2 are refreshed around the display.

### 2.4 Toll rate (0x56CF0) [C]
Castle record +0x14, editor range 0..14 percent. Toll charging is in the economy doc.

## 3. Computer-owned castles and camps at year end (`year_end_world`, after the sabotage step) [C unless marked]
Order of steps per property of kind 1 or 2 whose owner is a computer person:
1. Average shipment size `0x85E1D = (avg qty of all shipments in transit + 0x85E1D) / 2` (computed once before the loop).
2. Castle only: `acc += rand(100) + 1 + rate * 7 * 0x85E1D * 0x85E21 / 400000` (64-bit product, `0x85E21` not decoded [G: number of shipments or traffic level]); then the human Vogt of that county (office index 7 at location 10+k, controller human) receives `acc * 30 / 100` (message); `acc = 0`. The computer owner itself receives nothing here.
3. Purchase interest: if there is no sale record (type 16) for this property, collect up to six type-17 records (purchase interest, byte 1 = interested person, key = property). The candidate is the interested person who owns no property of this kind and has the highest favour of the owner toward him (ties: later wins). Price `P = value * (100 + 10*count) / 100`. With `rand(100) < count*80/6`: a computer candidate buys at once; a human candidate is asked (yes/no). On sale the buyer pays P (computer buyers do not pay [C, quirk]), the owner receives P, ownership moves, all interest records on it are deleted.
4. If not sold:
   - security/camouflage < 50 and `rand(100) < 30`: `+= 20 + rand(10)`;
   - training of slot 0 < 70 and `rand(100) < 5`: every slot `+= 2 + rand(10)`, capped at 100;
   - capacity < 80 and `rand(100) < 50`: capacity + 1;
   - `rand(100) < 40`: `add_troops(property, rand(4), 40 + rand(40))`, the training argument drawn first (corrected, see 3b);
   - castle and `rand(100) < 60`: patrol order, type-53 record: +4 = county `rand(3)`, +5..+8 = `troops[k] * (20 + rand(40)) / 100`;
   - else camp, year > 1607 and `rand(100) < 50`: raid order, type-54 record: +4 = county `rand(3)`, +5..+8 = `troops[k] * (40 + rand(40)) / 100`;
   - attack orders: for target kind camp, then castle: find a property of that kind whose owner is human; if its troop total is below `10 + rand(10)`, create a type-51 record (+4 = target location - 14, +5..+8 = `troops[k] * (50 + rand(30)) / 100`), deleted again if those troops sum to less than 20 (camp target) or 30 (castle target). **Corrected (3b): this branch is dead code; the step only consumes two RNG draws.**
   - the owner pays the increase of the property value caused by these steps;
   - castle, an auction is pending (type-3 record with byte 1 = 0xC0), no sale record yet and `rand(100) < 5`: create a type-16 record (castle put up for auction).

## 3a. Human castle/camp order book and yearly troop pay (castle screen 0x55090 / editor 0x55BC8, camp screen 0x56E20 / editor 0x579A1, settlement 0x61A70) [C]

The order book uses slots 0 and 1 of the property's four 8-byte order slots (+0x33, +0x3B; layout economy doc §3.1): byte 0 = order type, byte 2 = target, bytes 3..6 = troops per slot k = 0..3, byte 7 = status (0 impossible, 1 possible, 2 confirmed, 3 executed and recurring).

| type castle / camp | text | target byte 2 | cycling of the target on click |
|---|---|---|---|
| 0x10 / 0x20 | Kein Auftrag | - | - |
| 0x11 / 0x21 | Überfallt | castle/camp index 0..6 (location 14 + t) | `t = (t + 1) mod 7`, skipping indices where the player owns the property; up to 7 tries |
| 0x12 / 0x22 | Schickt | index 0..6 | `t = (t + 1) mod 7` among indices where the player owns a castle or camp (other than this one) [L for the skip rule] |
| 0x13 / 0x23 | Überwacht / Plündert | county 0..2 | `t = (t + 1) mod 3` |

Clicking the order word cycles the type 0x10 → 0x11 → 0x12 → 0x13 → 0x10 (camp 0x20..0x23); a newly selected type resets the target to the first valid one. Troop digits change bytes 3..6 by ±1 / ±10 (the number control, two digits).

Feasibility (`order_feasible` 0x45360, also economy §3.1): types 0x10/0x20 and anything outside the troop types: always feasible (nothing happens). 0x11/0x21 (attack): not feasible if the player owns a property at location `target + 14`. 0x12/0x22 (transfer) and 0x13/0x23 have no target check, so troops can be sent to own and to foreign castles/camps (H p.23 ✓). All troop types: each troop count must be <= the garrison count of its slot, and the total must be > 0.

Status handling: clicking the status circle toggles 1 ↔ 2 for a feasible order (2 reserves the troops out of the garrison, `order_reserve`; back to 1 returns them). Editing a confirmed order unreserves, applies the change and reserves again; if it became infeasible the status drops to 0. Order execution at the owner's turn end (economy §7 step 2): every slot with status 2 creates its troop record and changes status:
- 0x11 / 0x21 → type 51 (attack), status := **1** (must be re-confirmed every year);
- 0x12 / 0x22 → type 52 (transfer), status := **1**;
- 0x13 → type 53 (patrol), 0x23 → type 54 (raid), status := **3**.
At the next turn start every slot (branches, castles, camps) with status 3 is re-reserved and set back to 2, then rechecked (infeasible → 0). So patrols and raids, like production orders and branch transports, repeat every year until changed; attacks and transfers do not. (The troops of types 51..54 come home at the next turn start, section 4; that happens before the re-reservation in the same turn start, so the garrison is complete again.)

Yearly troop pay and training, first step of the owner's turn end (before order execution), for each castle (pay level `P` = person +0xF1, wages `W` = 100, 125, 125, 150) and each camp (`P` = +0xF2, `W` = 75, 100, 100, 125), with the confirmed orders' troops temporarily added back:
```
for k in 0..3:
    cost += W[k] * troops[k] * (P + 100) / 200
    if P < 100:  d = rand((100 - P) / 10 + 1);  training[k] = max(0, training[k] - d)
    if P > 100:  d = rand((P - 25) / 10 + 1);   training[k] = min(100, training[k] + d)
    (P == 100: no change, no RNG call)
```
The cost goes into the Söldner / Räuber lines of the bill. Quirk: the gain above 100 % starts at `rand(8)` for P = 101, so a pay level of 101 already trains 0..7 points per year; below 100 % the loss is only 0..1 points until P drops to 89.

## 3b. Computer-owned castle/camp troop records in detail [C unless marked]

Evidence: castle step 0x69082..0x69A41 (loop over property index i = 0..191, local `[ebp-0xD8]`), computer `settlement_pre` loop 0x6887C..0x688CB, computer record pass 0x69DEE..0x69F53, all in `year_end_world`; `rec_alloc` 0x424A0 = (type, owner, key, counter); `pick_property` 0x42B90; `add_troops` 0x45240; turn-start return loop 0x4BE25.

**Year-end order.** `year_end_world` runs these three blocks in straight-line order (no jump from a later block back into an earlier one): (a) `settlement_pre(p)` for every person index p = 0..191 with an occupied slot and controller computer (0x688C6); (b) the castle step of §3 (0x69082); (c) the record pass over computer persons (0x69DEE). So records created in (b) are fought in (a) of the **next** year end.

**(1) Troops are not taken out of the garrison.** The type-53 (0x6964F) and type-54 (0x6971D) creators write `rec[+5+k] = troops[k] * (20 + rand(40)) / 100` resp. `(40 + rand(40)) / 100` and never write property +0x58..+0x5B. The same holds for the computer revenge creators of types 51/54 (`rules-09` §4 code 1, 0x5B3A6..0x5B560). Only the human order book reserves troops (`order_reserve`, §3a). A computer garrison therefore keeps defending at full strength while its patrol/raid troops exist as a copy in the record.

**(2) Lifetime and deletion; no troop return.** The record is created with counter **2** (`rec_alloc(0x35 | 0x36, owner = property +2, key = i, 2)`). In the same year end the record pass (c) decrements it to 1; type 51..54 have no handler there, so the record is deleted (`0x42550`, mark bit 7) only when the counter reaches 0, i.e. in the record pass of the next year end, after that year end's (a) has used it. Nothing adds +5..+8 back to a garrison: the only return loop is in `turn_start_reports` 0x4BE25, which looks up records with owner = the current human only (`0x426B0(0xFF, human, 0xFF)`, then types 0x33/0x35/0x36). Computer persons have no turn start. Because nothing was subtracted, nothing is lost either; battle losses are subtracted from the record (5.6) and vanish with it. Consequence [L, from 5.6]: when a computer type-51 record conquers a human castle/camp, its surviving record troops move in although the home garrison was never reduced.

Lifetime by creation point [L, derived from the counter rules above]:
- castle step (b) of year end Y: counter 2 → 1 in Y's pass; exists during all human turns of the following year (patrols/raids can be **helpers** in battles in their county, 5.3); used once by `settlement_pre` at year end Y+1; deleted in Y+1's pass. Up to two generations coexist between (b) and (c) of Y+1 (the new one does not fight in Y+1's (a), which ran before it was created).
- revenge records (`rules-09` §4, counter 2) created during a human turn: decremented once per computer record pass, so they are used by (a) at two year ends. Created inside the record pass (type-50 handler 0x5B200): whether the same pass decrements them depends on whether the iteration reaches them (owner index ≥ current person, heap order) [UNVERIFIED].

**(3) Counter.** Created 2; only the year-end computer record pass touches it (`dec byte [rec+3]` at 0x69E3E, decrement before the type switch). `settlement_pre` ignores the counter. The type-16 sale record from step (b) gets counter 100.

**(4) Where computer attacks and raids fight.** `settlement_pre` 0x61410 is the same function for both: at 0x62A1C for the current human at turn end, at 0x688C6 for every computer person at year end. It processes the person's type-51 records (0x6142B), then its type-54 records (0x61514), in heap order; type 53 is not handled (helpers only). Per record: deleted if `rec+1` (owner) differs from the owner byte of property `rec+2`; otherwise not deleted after the fight. Computer specifics:
- type 51: battle only if the target's owner is human (the attacker is computer), code 4 (castle, `+4 < 3`) or 5 (camp), `battle(code, rec, 0)`.
- type 54: candidates are only shipments of **human** owners (the "attacker or owner human" filter); then h = 0, so the raid gate is `rand(100) < 30`; an escorted shipment of a human gives e = 0, gate `rand(100) < 30` again.
- **The castle-step type-51 branch is dead code.** At 0x6982C `pick_property(kind, 0xFF, 0xFF)` (kind 1, then 2) returns a random property of that kind with any location and owner; the code proceeds only if the result is **null** (`jne` to the loop end at 0x6983E) and then reads the null record. A null result needs zero properties of that kind; new games create 4 camps and 3 castles (`rules-00` §11 steps 5, 6) and none of the six callers of `property_free` 0x429A0 looked like a castle/camp path [L, quick inspection only]. If it were reached, the owner byte would be the uninitialised local `[ebp-0x48]` (first written at 0x69E04 in the record pass). So computer type-51 records come only from revenge (`rules-09` §4 code 1).
- Side finding: the type-16 auction lot created at 0x69A3C also takes its owner from the uninitialised `[ebp-0x48]` (key = i, counter 100). The lookup at 0x69219 that suppresses a second lot searches with owner = property owner, so it will normally not find this record. Effects on the auction (seller identity) [UNVERIFIED].

**(5) RNG draws, in order.**

Year end, block (a), per computer person p in index order, inside `settlement_pre(p)`:
1. type-51 records: no draw in `settlement_pre`; a battle (if the target owner is human) draws inside `battle_main` and reseeds the RNG from the clock (5.1), so every later draw of the year end is clock-dependent in the original.
2. per type-54 record with at least one candidate: `rand(1000)` (pick); `rand(100)` (gate < 30); if escorted `rand(100)` (gate < 30); then for Karren only: county 1: `rand(100)` (< 20 → code 1); county 0: `rand(100)` (< 25 → code 0, then `rand(100)` < 20 → 1), else `rand(100)` (< 50 → 1 else 2); county 2: `rand(100)` (< 50 → 1 else 2). Schiff: no draw. Then battle if troop total ≥ qty / 100 (draws and reseed). No candidate: no draw.

Block (b), per property i = 0..191 of kind 1/2 with a computer owner:
1. castle: `rand(100)` (toll accumulator, §3 step 2).
2. no type-16 record (owner = property owner, key = i) and an interested candidate: `rand(100)` (sale, < count·80/6). Sold → nothing else for this property.
3. security/camouflage < 50: `rand(100)` (< 30); hit → `rand(10)`.
4. training slot 0 < 70: `rand(100)` (< 5); hit → `rand(10)` for slots 0, 1, 2, 3.
5. capacity < 80: `rand(100)` (< 50).
6. `rand(100)` (< 40); hit → `rand(40)` (training `40 + r`), `rand(4)` (men), then inside `add_troops` `rand(4)` (start slot) only if the men left after the capacity cap are > 0.
7. castle: `rand(100)` (< 60); hit → type 53: `rand(3)` (county), then `rand(40)` for k = 0..3. Camp and year ≥ 1608: `rand(100)` (< 50); hit → type 54: `rand(3)`, then `rand(40)` for k = 0..3. A castle with a miss and a camp before 1608 (or with a miss) create nothing; a camp draws no 60 % roll.
8. attack step: `pick_property` draws `rand(n)` with n = number of camps (4), then `rand(n)` with n = number of castles (3); no record (dead branch above).
9. owner pays the value increase (no draw).
10. castle, a type-3 0xC0 record exists and no type-16 record was found in step 2's lookup: `rand(100)` (< 5) → type-16 lot.

Block (c): no draws for types 51..54.

## 4. Troop orders and raids (`settlement_pre` 0x61410) [C]
Called for every computer person at year end and for the current human at turn end. Records (byte 1 = owner, byte 2 = home property index):
- type 51: attack on the castle/camp at location `+4 + 14`, troops +5..+8;
- type 53: patrol in county +4 (castles), troops +5..+8;
- type 54: raid in county +4 (camps), troops +5..+8.
A record whose owner no longer owns the home property is deleted.

Lifetime of troop order records [C]: the castle/camp order book creates them at the owner's turn end (economy doc §7 step 2), troops are taken out of the home garrison when the order is confirmed (0x49370).
- Type 51 (attack), 53 (patrol, "Überwacht"), 54 (raid, "Plündert") stay in the heap until the owner's **next turn start** (0x4B900), where the troops (+5..+8) are added back to the home property's garrison slot by slot and the record is deleted. While they exist they act: 51 and 54 in `settlement_pre` (this section), 53 and 54 as possible **helpers** in every battle in their county during that window (5.3: other owners' raids and attacks at their turn ends and at year end). A patrol does nothing else; its only effect is the chance to join fights (and the turn-end message showing its strength).
- Type 52 (transfer, "Schickt"): created with counter 1 and resolved in the same turn end's record countdown (economy doc §7 step 16): its four troop counts are added slot by slot to the garrison of the property at location `+4 + 14` (whoever owns it; a message if the player owns it), then the record is deleted. If no property exists there the troops are lost. There is no type conversion: slot k goes to slot k, so the handbook's conversion example (H p.23) holds only because camp and castle slots are ordered so that matching slots correspond.

Type 51: if the attacker or the target's owner is human, `battle(code, record, 0)` with code 4 for a castle target (`+4 < 3`), 5 for a camp. Between two computer persons nothing happens.

Type 54 (raid):
1. Candidates: shipments (type 9) of other owners whose destination city lies in county `+4 + 10`, transporter (+9) Karren (0) or Schiff (1), and attacker or shipment owner human; at most 50. Weight `qty (+5) * V[g]` with `V[g] = 7 + g` (goods table byte 12).
2. `r = rand(1000) * total / 1000`; the first candidate whose cumulative weight exceeds r is picked (the last one if none).
3. `h = 1` if attacker and victim are both human, else 0. The raid happens only if `rand(100) < 30 + 40h`.
4. If the shipment is escorted (+4 bit 0): `e = 1` if the victim is computer, else 0; the raid happens only if `rand(100) < 30 + 40e`.
5. Battle code: Schiff → 3. Karren: county 1 → 0, then `rand(100) < 20` turns it into 1; county 0 → 0 with `rand(100) < 25`, otherwise 1 or 2 (50 % each); county 2 → 1 or 2 (50 % each). Exact order [C, re-checked at instruction level]: if the raid county (+4) is 1, code 0; else if it is 0 and `rand(100) < 25`, code 0; else `rand(100) < 50` gives 1, otherwise 2 (county 2 draws only this roll, county 0 draws both). Then, whenever the code is 0 (county 1, or county 0 after the 25 % roll), `rand(100) < 20` turns it into 1. (Corrected: an earlier note said county 0 never ends with code 0; it can, with 25 % × 80 % = 20 %.)
6. Battle only if the raiders' total troops >= `qty / 100`.

## 5. Tactical battle (`battle_main` 0x3EE60)

### 5.1 Battle codes and fields [C]
`field = [0, 1, 1, 2, 3, 4][code]` (`0x85803`); background picture `0x857FE[field]`.

| code | situation | field |
|---|---|---|
| 0 | cart raid | 0 |
| 1, 2 | cart raid | 1 |
| 3 | ship raid | 2 |
| 4 | attack on a toll castle | 3 |
| 5 | attack on a robber camp | 4 |

`battle_main` calls `randomize_from_time` at its start: the original reseeds the RNG from the clock for every battle, so battles and everything after them are not reproducible. A reimplementation should keep its seeded RNG instead (deliberate deviation).

### 5.2 Battlefield and terrain [C]
`SZENARIO.DAT` holds five fields of 46 rows by 30 columns (`formats.md` 4). Cell byte: low 5 bits = code, high 3 bits = direction d (0..7). Directions: `dx = [0,1,1,1,0,-1,-1,-1]`, `dy = [-1,-1,0,1,1,1,0,-1]` (d = 0 is up, clockwise). Each code maps to an occupancy value (`0x85809`) and a flag value (`0x8581D`):

| code | occupancy | flags | meaning |
|---|---|---|---|
| 0 | free | 0 | open ground |
| 1 | 0xFE blocked | 0 | obstacle |
| 2 | free | 1 | field edge: target of "Flucht"; never entered by other movement |
| 3 | free | 2 | entrance square: target of "Eingang stürmen/verteidigen", conquest squares, defender anchor |
| 4 | free | 0 | slot for goods markers (unit type 20, defender only) |
| 5 | free | 0 | slot for wagons (unit type 21, defender only) |
| 6 | 0xFB | 0 | wall/palisade: not walkable; a cannon or bomb hit turns it into 0xFE and counts as bombardment damage |
| 7 | free | 0 | continuation square of a deployment line |
| 8..13 | free | 0 | attacker anchors for unit classes 0..5 |
| 14..19 | free | 0 | defender anchors (code + 6) |

Unit class `0x85847[type]`: types 0, 5 → 0; 1, 6 → 1; 2, 7 → 2 (cannon); 3, 8 → 3; 4, 9 → 4 (leader); 10, 11 → 5 (carters, sailors). Anchor code = `0x85831[type] + 6*side` = `8 + class + 6*side`.

### 5.3 Participants [C]
Participant 0 = attacker (owner of the order record), 1 = defender (shipment owner, or castle/camp owner). Up to two helpers (participants 2, 3): from all type-53 and type-54 records (castle/camp attacks) or those in the raid's county (raids), each listed with `rand(100) < 80`, excluding the attacking record, at most 20. One is drawn with `rand(n)`; if n > 1 a second is drawn as `(first + 1 + rand(n-1)) mod n` and kept only if it has another owner. Colours: humans use person +0xF4; each computer participant takes the lowest colour 0..5 not used yet (the same person keeps one colour).

Detection (codes 4, 5 only): with `rand(200) <= security/camouflage` (+0x03) the attack does not take place at all (message, no battle).

### 5.4 Deployment (0x3DDA0) [C]
`deploy(field, type, n, participant, side, training)` places n men of one unit (at most 10 per unit, overflow continues as a new unit; at most 100 units, 200 men):
- Anchor: among free cells whose code equals the anchor code, the one maximising `(y-20)^2 + 4*(x-15)^2` (far from the centre). For the defender (side 1) on fields 2..4 the anchor closest to the first entrance square (row-major) is used instead.
- From the anchor, further men are placed every 1 cell (2 for cannoneers) in direction `(d + 2) mod 8`, while the next cell's full byte equals `(anchor & 0xE0) + 7` (a code-7 cell with the same direction) and is free.
- A cannoneer (types 2, 7) also places his cannon (object type 16) on the neighbouring cell `(x + [1,1,0,-1,-1,-1,0,1][d], y + [0,1,1,1,0,-1,-1,-1][d])`; if that fails, the man is not placed.
- Unit fields: type, participant, side, training, tactic = `0x857D8[type]` (0 for all, 1 for cannoneers, 8 for leaders), next-to-act link sorted by anchor score (higher acts first).
- Man fields: hit points `HP = base[type] + training/4`, `base = 0x857C2 = [40, 30, 20, 60, 70, 40, 30, 20, 40, 70, 30, 30]` (type 16 cannon 60; goods markers and wagons 100); condition = training; shots (cannoneers) `training/20 + 1`; moves per round `0x857AC = [6, 5, 6, 7, 6, 6, 5, 6, 9, 6, 6, 6]`.

Raid (codes 0..3):
- defender, side 1: goods markers type 20, `n = ceil(qty/100)` (this n is the goods total "markers"), training 0;
- carts: codes 0..2: carters type 10 and wagons type 21, `n = shipment +8` (number of carts) each, training 50; ship: sailors type 11, `n = 5 * ships`, training 50;
- escort (+4 bit 0): every mercenary type k (except cannoneers unless ship) `n = ceil(qty / 250)`, training 50; plus one general (type 4), training 87;
- attacker (computer attackers cap each troop count at `max(1, value/350)`, value = `qty * (g + 8)`): types `base + k` with n = order troops k, training = home property training k; one leader (type base + 4) whose training is read from home property +0x60, which is the **kind byte of the next property record** (0..3) [C at instruction level, bug: the attacking leader nearly never hits].

Castle/camp attack (codes 4, 5): defender troops type `(0 | 5) + k`, n = garrison k, training = garrison training k; leader (4 or 9) training 87. Attacker as above. Value = target property value.

### 5.5 Before the fight [C]
1. Ransom (raids and undetected attacks). Defender computer: `s = strength(1)` (5.7); s < 3: offers nothing; s = 3: offers `2 * (80 + rand(40)) * value / 400 + 1`. Defender human: enters an amount.
2. Attacker computer: offer 0 → fight; else if `offer * 200 < value * rand(100)` the robbers **take the money and fight anyway**; else they accept and leave. Attacker human: chooses fight (offer becomes 0, nothing is paid) or accept. In every case the (possibly zero) offer is moved from defender to attacker before the decision takes effect.
3. Helpers (only if fighting). Each of attacker and defender may bribe each helper:
   - computer briber: `b = value / (4 * helpers) + rand(100)`, set to 0 if `strength(briber side) < 2` (already strong), b < 500, or the helper index check fails (quirk: it compares the attacker/defender slot of the same index instead of the helper owner, and skips human ones);
   - human briber: enters amounts per helper.
   Paid amounts move to the helper's owner, and the helper's favour toward the briber rises by `min(100, b*400 / max(100, net_worth(helper)))` (doubled for human bribers).
4. Each helper chooses a side: if its owner is attacker or defender it joins that side; a computer owner otherwise joins the attacker if `favour(→attacker) > favour(→defender)`, `favour(→attacker) > 50` and the attacker's bribes exceed value/7; the defender likewise with threshold 45; else it stays out. A human helper owner chooses. Joining helpers deploy their order's troops (no leader) with their home training.

### 5.6 Rounds and end [C]
State: round counter `idle = 10`. Each round every unit acts in list order (5.7, 5.8). After the round: count men alive per side and entrance squares occupied by attacker men. The battle ends when a side has no men, or 3 entrance squares are held, or after a round, `idle - 1` reaches 0. Any damage dealt sets `idle = 10` again, so a battle ends "undecided" after 10 rounds without damage.

Outcome:
- If the criminal law "Überfälle" is forbidden, every participant on the attacker side gets evidence 0x400 (`add_evidence`).
- With 50 %: the defender retaliates against the attacker (0x5B140 code 4).
- Conquest if (more than 2 entrance squares held or the defender has no men) and not undecided: ship (code 3) → the shipment record is deleted; castle/camp → a human attacker is asked whether to occupy it, a computer always occupies; owner := attacker.
- Destroyed goods markers (`gl`): `qty -= qty * gl / markers`. Destroyed wagons (`wl`): if all carts were destroyed one is spared (`wl - 1`), then shipment +8 -= wl.
- Per participant on a side: prisoners p → type-55 record (participant, key = his home property; for the defender of a castle/camp the castle) +4 += p (humans only, not for a transport defender). Captured goods markers c (not the defender): `amount = c * qty / markers` added to the participant's home property (`branch_add_goods`), mission kinds 19 and 29 add `amount * (g + 8)`, shipment qty -= amount.
- Losses per troop type are subtracted from each participant's order record (defender of a castle/camp: from the garrison).
- Troop premium (`[EXE text]`) [C, re-checked at instruction level]: after the losses have been subtracted from the attacking order record, `R = record(+5) + record(+6) + record(+7) + record(+8)` (the attacker's four troop counts, bytes, so R <= 1020; troops that were never deployed because of the 200-men / 100-unit / 10-per-unit caps are still in the record and count), `premium = value * R / 80` (signed 32-bit, truncating), where `value` is the battle value of 5.4: for raids `qty * (g + 8)`, for castle/camp attacks the target property's value (`net_worth_goods`, section 1). If premium > 0 the **attacker** pays it (`person_pay`, cash may go negative); nobody receives it (the money leaves the game). Otherwise the text `[EXE text]` is shown.
  - Charged for every battle that reaches the end screen: any outcome, undecided included, and for castle attacks as well as raids, computer or human attacker. Not charged when the battle never happens (detection, ransom accepted).
  - Scale check: a raid on 1000 units of Korn (value 8000) with 40 robbers costs 4000; a full 4×255 attack on a castle worth 30000 costs about 382000. The original really is that punitive for large castle assaults; H p.33 warns that robbers demand a premium and that frequent raids ruin their leaders. Keep it for parity; this is not a decoding error.
- Bombardment hits B (castle/camp): casemates/straw `-= min(B/4, current)` (a loss of exactly 1 becomes 0), security/camouflage `-= min(B/3, current)`.
- Raid insurance: if the defender has insurance bit 2 (+0x07 & 4) and goods were captured: pays `captured_total * (V[g] + 4)` only when the shipment was escorted; otherwise the insurer refuses.
- Castle/camp taken: garrison counts are set to 0 and garrison training is copied from the attacker's home property; then the attacker's surviving order troops move in round-robin up to capacity. Purchase interest (type 17) of the new owner and the sale record (type 16) on that property are deleted.

### 5.7 Computer tactic choice (0x389C0) [C]
`strength(s)` (0x38730): S = sum of HP of side s, O = other side. Returns 0 if S > 3O, 1 if S > 2O, 3 if 2S < O, 4 if 3S < O (unreachable, checked after 3), else 2.

A unit is computer-controlled when its participant is computer, when its side has no living leader, or in ESC mode. Each time it acts:
1. n = men, H = sum HP, A = sum shots. A cannon unit with 50 % sets A = 0 if any of its men stands next to an enemy.
2. Pressure P: on fields 0, 1 and for cannon units P = -100; otherwise P = (attacker men − defender men, non-cannon) within the 10×10 box `[ex-5, ex+5) × [ey-5, ey+5)` around the first entrance square.
3. If H <= 10n, or a cannon unit with A = 0: tactic 10 (Flucht).
4. Else if H <= 20n and `rand(100) < 50`: tactic 9 (Rückzug).
5. Else: defender with P >= -2 → 6; attacker with P >= 0 → 5; otherwise with `rand(100) < 50` a weighted choice (else the tactic stays): `r = strength(side)`, weights `(a, b, c) = 0x84E4E[side][r]` with b used only on fields 0..2 and c only on fields 2..4; `x = rand(a + b + c + 1)`:
   - x <= a: cannon → 1, others → 7 with 10 %, else 0;
   - x <= a + b: non-cannon → 3 (attacker) / 4 (defender); cannon unchanged;
   - else: cannon → 2 (attacker only); others → 5 / 6.

| side | r = 0 | 1 | 2 | 3 | 4 |
|---|---|---|---|---|---|
| attacker | 20 30 50 | 50 20 30 | 80 20 0 | 90 10 0 | 100 0 0 |
| defender | 70 20 10 | 60 30 10 | 80 10 10 | 80 0 20 | 50 0 50 |

### 5.8 Tactics, movement and damage
Tactic ids and availability (0x84E38 masks, checked against the unit class, side and field) [C]:

| id | name | classes | side | fields |
|---|---|---|---|---|
| 0 | Gegner angreifen | all but cannon | both | all |
| 1 | Gegner beschießen (shots) | cannon | both | all |
| 2 | Sturmreif schießen | cannon | attacker | 2, 3, 4 |
| 3 | Waren erbeuten | all but cannon | attacker | 0, 1, 2 |
| 4 | Waren zurückerobern | all but cannon | defender | 0, 1, 2 |
| 5 | Eingang stürmen | all but cannon | attacker | 3, 4 |
| 6 | Eingang verteidigen | all but cannon | defender | 3, 4 |
| 7 | Gefangene machen | all but cannon | both | all |
| 8 | Abwarten | all | both | all |
| 9 | Rückzug | all but cannon | both | all |
| 10 | Flucht | all | both | all |

Handbook check: "Gefangene machen" only from castle or camp ✗ (the code allows it everywhere); "Eingang stürmen/verteidigen" also on ships ✗ (fields 3, 4 only); "Sturmreif schießen" only against castle/camp ✗ (also field 2). Sturmreif on a ship field still only damages code-6 cells.

Human control: the attacker's and the defender's hotkeys (+0xF5) open that side's tactics menu while its leader lives; a click cycles the unit to the next allowed tactic. Helpers are always computer-controlled. The unit info shows `count` and the average HP as the condition percentage. ESC (key 27) switches the rest of the battle to computer control for all sides and stops drawing; the game rules are the same, but fewer presentation RNG draws happen (5.11), so the random sequence differs from a displayed battle (correction to an earlier "the simulation is the same"). [C]

Movement (0x37C00) [C for the grid rules, L for which tactic uses which mode]: every move is a breadth-first search over the 8-neighbour grid from goal cells toward the man; the man steps to a neighbour whose distance is one less (neighbour order `0x858C0` = directions 0, 4, 6, 2, 7, 1, 5, 3). Walkable = empty cell without object and not an edge cell. Goals by mode:

| mode | used by tactic | goal cells |
|---|---|---|
| 0 | 0, 7 | cells of living enemy men |
| 1 | 10 | edge cells (flag 1); a man standing on one leaves the battle |
| 2 | 5, 6 | entrance squares (flag 2) |
| 4 → 3 | 9 | distance band: cells at distance 4 from the nearest enemy, moving only through cells closer than that |
| 6, 7 | 1, 2 | the operating square in front of / beside the man's cannon |
| 8 | 3 | goods markers not carrying an own banner |
| 9 | 4 | goods markers carrying an enemy banner |

Melee hit (when an attack animation ends on target t):
- tactic 7: capture if `rand(5 * condition_t + 1) < condition_a`; the target falls as a prisoner and counts for the attacker's participant;
- otherwise: `damage = (rand(100) < condition_a) ? rand(HP_a + 1) : 0`; `HP_t -= damage`, the man dies at 0. Damage > 0 resets the idle counter. Note the damage scales with the attacker's current HP.

Explosion (cannon shot, bomb) on a 3×3 area [L for the firing rules, C for the damage]: the impact cell takes 100; the other eight take `rand(80)` (even index) or `rand(60)` (odd), avoided completely with `rand(100) < condition_t`, halved for men of the shooter's side (friendly fire). Goods markers above 50 HP drop to 50 and count one destroyed marker; wagons likewise count a destroyed wagon; wall cells (0xFB) become rubble (0xFE) and count one bombardment hit. Cannon aiming (tactics 1 and 2) uses an influence map: every living man adds his HP (enemy) or subtracts it (own side) to all cells within ±2; tactic 2 adds 100 within ±2 of every wall cell.

### 5.9 Per-man step logic (0x39453 with jump table 0x39870) [C unless marked]

Man record fields used here (200 records of 33 bytes at `0xD7DCC`): +0x04/+0x06 grid x/y (s16), +0x0C HP, +0x0D condition, +0x0E type, +0x10 participant, +0x11 side, +0x12 action state (0 idle, 1 strike/shoot, 2 parry, 3 walk, 4 fallen, 6 laying a bomb), +0x13 facing 0..7, +0x15 remaining animation frames (s16), +0x19 moves left this round, +0x1E target man (0xFF none; own index = "after a bomb, get away"). The unit's tactic is its unit record +0x0E. A man acts only when his animation has run out (+0x15 == 0).

Path search `path(mode, man)` (0x37C00) returns [L for the exact encoding, consistent with every use]: 0xFF = no goal reachable; 0 = nothing to do (standing on a goal); 1 = goal adjacent; n >= 2 = goal at distance n. It also sets the first-step direction (`0xDC837`) and the occupant of the goal / adjacent cell (`0xDC838`).

**Walk step** (used below): state 3, facing = step direction, the destination cell is reserved (grid byte 0xFE) and entered when the animation ends, moves -= 1. **Go idle**: state 0, moves := 0 (the man is done for this round). **Strike t**: state 1, facing toward t, target := t; if t is not a cannoneer (types 2, 7), t is put into state 2 (parry) facing back at the attacker (`(dir + 4) mod 8`), which also uses up t's time until that animation ends.

When the animation of a man with moves left ends:
1. If his target is another man t: target cleared, man goes idle, and if t has not fallen (state 4) and is a fighter (type < 12) — otherwise nothing is rolled at all (rules-71 §4):
   - unit tactic 7: capture if `rand(5 * condition_t + 1) < condition_man`: t falls (state 4, frame 0, fall frame count, +0x1E := the captor's man index; participant, side, HP, type and unit slot unchanged until the fall ends, `rules-43` §1) and counts as a prisoner of the man's participant (`0xDB804`); otherwise t returns to idle. (Detailed by rules-71 §4.)
   - otherwise: `hit = rand(100) < condition_man` (the roll is always made), `damage = hit * rand(HP_man + 1)` (also always rolled); damage > 0 marks t as hit and resets the battle's idle counter to 10. If damage < HP_t: `HP_t -= damage` and t is set back to idle (a hit interrupts t's parry/action); else HP_t = 0 and t falls.
   - The man's moves are then 0: a strike ends his round.
2. If his target is himself (a bomb layer after laying): `r = path(4, man)` (away from enemies); r = 0xFF or 0: go idle, else walk step; when moves reach 0 the target is cleared.
3. Otherwise act by the unit tactic:

| tactic | behaviour |
|---|---|
| 0 Gegner angreifen, 7 Gefangene machen | `r = path(5 if type == 8 (Bombenleger) else 0, man)` (mode 0 = toward enemy men). Musketeers / Flintenschützen (types 1, 6) with tactic 0 and r in 1..3 **shoot** (below) instead of striking or walking. If r = 0xFF or 0: go idle. If r >= 2 (and not shooting): walk step. If r = 1 (non-musketeers): a Bombenleger with tactic 0 and at least 3 moves left lays a bomb (state 6, target := himself, then flees with step 2 above; the explosion is the 3×3 blast of 5.8 [L for the timing]); everyone else (and the Bombenleger with tactic 7 or fewer than 3 moves) strikes the adjacent enemy. With tactic 7 musketeers do not shoot; they close in like everyone else. |
| 8 Abwarten | `r = path(0, man)`; if r = 1 and the man is not a cannoneer: strike the adjacent enemy; else go idle. The unit never moves, but it does deal damage to enemies that come adjacent. Handbook disagreement: H p.32 says Abwarten "will defend itself but not harm any enemy"; the code strikes back with normal damage. |
| 9 Rückzug | `r = path(4, man)` (distance band, 5.8); r = 0xFF or 0: go idle; else walk step. Never strikes. |
| 10 Flucht | `r = path(1, man)` (field edge); r = 0xFF or 0: go idle; else walk step (the step animation is one frame longer when r = 1). Reaching an edge cell removes the man (5.8). |
| 3 Waren erbeuten / 4 Waren zurückerobern | `r = path(8 / 9, man)`. r = 1: **plant the banner** (below). r >= 2: walk step. r = 0xFF or 0: fall back to `r2 = path(0, man)`: strike if r2 = 1 (and not a cannoneer), otherwise go idle (no walking toward enemies). |
| 5 Eingang stürmen / 6 Eingang verteidigen | `r = path(2, man)` (entrance squares): any r other than 0xFF and 0 → walk step (r = 1 steps onto the square). r = 0xFF or 0 (already on a square, or none reachable): fall back to `path(0)`: strike an adjacent enemy, otherwise go idle. So units on the entrance only fight what stands next to them (H p.33 ✓). |
| 1 Gegner beschießen, 2 Sturmreif schießen | cannon handling (aiming by influence map, 5.8); the per-step details of this case (0x39EA4..0x3A6AB) are not decoded. [open] |

**Musket shot** (types 1, 6, tactic 0, target at path distance 1 to 3): facing := the path direction d. Scan 32 fine directions starting at `a0 = (4d + 4) mod 32` and alternating around it: `a0, a0-1, a0+1, a0-2, a0+2, ...` (k = 0..31: `a = (a0 + 32 + (1 - 2(k mod 2)) * ((k+1) div 2)) mod 32`). For each a, step along the ray table `0x85000` / `0x85100` (4 cell offsets per direction, cumulative) for at most 4 cells; stop when leaving `1 <= x < 29, 1 <= y < 45` or at the first non-empty cell. The first ray whose stopping cell holds a man of the other side who has not fallen and is a fighter (type < 12) gives the target: strike it (state 1, facing `(a + 28) mod 32` divided by 4 with C rounding toward zero). The damage is then the normal strike damage of step 1. No target on any of the 32 rays: go idle. Walls and other men block the line of fire.

**Banner planting** (tactics 3, 4, goods marker adjacent): the marker's banner lives in the cell one to the right of the marker (`x + 1`). If that cell is empty (0xFD), a banner object (type 12) is created there for the man's participant; otherwise the existing banner's participant loses one captured marker (`0xDB800[p]` -1). Then the man's participant gains one (`0xDB800[p]` +1) and the banner takes the man's participant and side. The man goes idle. These counts are the "captured goods markers" of 5.6.

### 5.10 Tables for the step logic [C, dumped from the data object]

Direction steps (`0x84F2C` dx, `0x84F3E` dy, index 0..8; index 8 = centre, used by explosions): dx = 0, 1, 1, 1, 0, -1, -1, -1, 0; dy = -1, -1, 0, 1, 1, 1, 0, -1, 0.

Musket ray table: for fine direction a = 0..31, four cumulative steps `(dx_j, dy_j)`, j = 0..3 (`0x85000` dx, `0x85100` dy, s16):

| a | dx | dy | a | dx | dy |
|---|---|---|---|---|---|
| 0 | -1 -1 -1 -1 | -1 -1 -1 -1 | 16 | 1 1 1 1 | 1 1 1 1 |
| 1 | -1 -1 0 -1 | -1 -1 -1 -1 | 17 | 1 1 0 1 | 1 1 1 1 |
| 2 | -1 0 -1 0 | -1 -1 -1 -1 | 18 | 1 0 1 0 | 1 1 1 1 |
| 3 | 0 0 -1 0 | -1 -1 -1 -1 | 19 | 0 0 1 0 | 1 1 1 1 |
| 4 | 0 0 0 0 | -1 -1 -1 -1 | 20 | 0 0 0 0 | 1 1 1 1 |
| 5 | 0 0 1 0 | -1 -1 -1 -1 | 21 | 0 0 -1 0 | 1 1 1 1 |
| 6 | 1 0 1 0 | -1 -1 -1 -1 | 22 | -1 0 -1 0 | 1 1 1 1 |
| 7 | 1 1 0 1 | -1 -1 -1 -1 | 23 | -1 -1 0 -1 | 1 1 1 1 |
| 8 | 1 1 1 1 | -1 -1 -1 -1 | 24 | -1 -1 -1 -1 | 1 1 1 1 |
| 9 | 1 1 1 1 | -1 -1 0 -1 | 25 | -1 -1 -1 -1 | 1 1 0 1 |
| 10 | 1 1 1 1 | -1 0 -1 0 | 26 | -1 -1 -1 -1 | 1 0 1 0 |
| 11 | 1 1 1 1 | 0 0 -1 0 | 27 | -1 -1 -1 -1 | 0 0 1 0 |
| 12 | 1 1 1 1 | 0 0 0 0 | 28 | -1 -1 -1 -1 | 0 0 0 0 |
| 13 | 1 1 1 1 | 0 0 1 0 | 29 | -1 -1 -1 -1 | 0 0 -1 0 |
| 14 | 1 1 1 1 | 1 0 1 0 | 30 | -1 -1 -1 -1 | -1 0 -1 0 |
| 15 | 1 1 1 1 | 1 1 0 1 | 31 | -1 -1 -1 -1 | -1 -1 0 -1 |

(a = 0 is up-left, a = 4 straight up, then clockwise; a step with (0, 0) stays on the same cell and tests it again.)

Animation lengths in frames (`0x855C8`, u16, row = man/object type, column = action state; `-` = 0xFFFF, unused):

| type | 0 idle | 1 strike | 2 parry | 3 walk | 4 fall | 5 operate cannon | 6 lay bomb | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 Söldner | 1 | 8 | 7 | 6 | 7 | - | - | - | - | - | - |
| 1 Musketier | 1 | 14 | 7 | 6 | 6 | - | - | - | - | - | - |
| 2 Kanonier | 1 | - | - | 6 | 8 | 14 | - | - | - | - | - |
| 3 Offizier | 1 | 8 | 8 | 6 | 6 | - | - | - | - | - | - |
| 4 General | 1 | 8 | 8 | 6 | 6 | - | - | - | - | - | - |
| 5 Räuber | 1 | 8 | 6 | 6 | 7 | - | - | - | - | - | - |
| 6 Flintenschütze | 1 | 14 | 7 | 6 | 6 | - | - | - | - | - | - |
| 7 Kanonier (camp) | 1 | - | - | 6 | 8 | 14 | - | - | - | - | - |
| 8 Bombenleger | 1 | 7 | 8 | 6 | 10 | - | 10 | - | - | - | - |
| 9 Räuberhauptmann | 1 | 8 | 7 | 6 | 7 | - | - | - | - | - | - |
| 10 Fuhrknecht | 1 | 10 | 7 | 6 | 6 | - | - | - | - | - | - |
| 11 Matrose | 1 | 10 | 7 | 6 | 6 | - | - | - | - | - | - |
| 12 banner | 6 | - | - | - | - | - | - | 6 | - | - | - |
| 13 explosion (ground) | - | - | - | - | - | - | - | - | 8 | - | - |
| 14 explosion (wall, variant a) | - | - | - | - | - | - | - | - | - | 8 | - |
| 15 explosion (wall, variant b) | - | - | - | - | - | - | - | - | - | - | 8 |
| 16 cannon | 8 | - | - | - | - | - | - | - | - | - | - |
| 17 cannon ball, 18 / 19 bomb | 1 | - | - | - | - | - | - | - | - | - | - |
| 20 goods marker, 21 wagon | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Projectiles and bombs do not use this table for their flight/fuse time (below). Moves per round (`0x857AC`, byte): 6, 5, 6, 7, 6, 6, 5, 6, 9, 6, 6, 6 for types 0..11.

### 5.11 Round driver and frame loop [C]

`path(mode, man)` returns the breadth-first distance of the man's own cell in the goal map: 0 = the man stands on a goal cell, 1 = a goal is adjacent, n = distance n, 0xFF = no goal reachable. It sets the first-step direction (`0xDC837`, from the neighbour order of 5.8) and the content of the neighbour cell in that direction (`0xDC838`; a man index when the goal is an enemy).

Battle loop (battle_main): `idle := 10`, then rounds:
```
round:
    u = first unit of the act order (list of 5.4)
    while u < 100:
        RNG: sound-ambience counter c (starts 0): if c < 7: c += 1, else if rand(100) < 20: c = 0 (and play a sound if displayed)
              -- this rand(100) is drawn whether or not the battle is displayed
        activate(u)
        u = next(u)                                          (unit record +0x11)
        after every activation: count entrance squares held by attacker men (cell flag 2, man side 0)
            and living fighters (type < 12, not fallen) per side;
            if held >= 3, or either side has no living fighter: the battle ends at once (mid-round)
    idle -= 1
    if idle > 0: next round, else the battle ends ("undecided" unless one of the conditions above held)
```
Any damage > 0 (strike, shot, explosion) sets `idle := 10`.

`activate(u)` (0x389C0): skipped if the unit has no men left. If the unit is computer-controlled (5.7: computer participant, or its side has no living leader, or ESC mode), the tactic is chosen now (5.7). Then for each of the unit's up to 10 men: moves := `MOVES[type]`, animation frames := 0, target := none. Tactics 1 and 2 rebuild the cannon influence map (5.8). Then the **frame loop**:
```
repeat:
    for each man of this unit (slot order 0..9): if his animation frames == 0: step logic of 5.9
    for every object on the field (row by row, all units, objects and projectiles):
        if frames > 0: active = true; frame index += 1 (not for types 18, 19); frames -= 1
        if frames reached 0: end-of-animation handling (walk: enter the reserved cell [L]; bomb laying; explosions; below)
    (display: draw, wait one BIOS clock tick ~1/18 s, read keys: ESC, the two hotkeys)
until no object had frames > 0 in this frame
```
So units act strictly one after another; within a unit its men interleave frame by frame; other units' men only react passively (parry animations, being hit). An activation ends when every man of the unit has used his moves or gone idle (idle sets moves to 0 and plays the 1-frame idle animation) and every animation on the field, including projectiles and bomb fuses started by this unit, has finished. Striking (melee, musket shot, capture attempt) and going idle set the man's moves to 0, so they end his part of this activation; walking costs 1 move per cell; operating a cannon and laying a bomb cost no move.

**Presentation RNG** (parity hazard) [C]: most sound effects call `rand(n)` to pick a sample, but only when the battle is displayed (not in ESC mode): strike, shot, walk, fall and explosion sounds in 5.9 and below. The ambience roll above, the explosion's second-sound roll (`rand(100)`, below) and all game-rule rolls are drawn in both modes. A port that wants to reproduce a *displayed* original battle must draw `rand` at the same places; a port that only needs internal consistency can treat the displayed mode as the ESC mode (fewer draws). Recommendation: follow ESC-mode draws and document the divergence.

### 5.12 Cannon step logic (tactics 1 and 2) [C]

A cannoneer man has his cannon object (+0x1A), a "manned" flag (+0x1B), a "fired" flag (+0x1C) and shots left (+0x1F, start `training/20 + 1`). Each time he may act:
1. Not manned: `r = path(6, man)` (the operating square of his cannon). If r = 0xFF or shots = 0: go idle. Else roll `rand(100)`: if not below his condition: go idle (he hesitates this activation). Else if r > 0: walk step. Else (on the square): state 5 (operate), facing from table `0x84FF0[cannon facing]` (6, 4, 4, 4, 2, 0, 4, 4 for cannon facings 0..7), 14 frames, manned := 1. When that animation ends he acts again (the operate step costs no move):
2. Manned, not fired: `r = path(6, man)`; 0xFF: go idle; r > 0: walk back. r = 0: aim: for k = 5..24 cells along the cannon's facing (x = cx + k·dx, **y = cy + 2k·dy**, rows are half as tall as columns) inside the field (0 <= x < 30, 0 <= y < 46): take the influence value at that cell (5.8); keep the largest (first maximum, strictly greater than the running best, start -10000) and its k (stored in the unit record +0x10). The man goes idle. If the best value > 0: fire: fired := 1, shots -= 1; target = cannon + k steps as above; scatter `x += rand(3) - 1, y += rand(3) - 1` if k < 10, else `x += rand(5) - 2, y += rand(5) - 2`; clamp x to 1..28, y to 1..44. A cannon ball (object type 17) is created at the cannon with that target; flight time `max(24·|cx - tx|, 12·|cy - ty|) / 10` frames, at least 1. (Firing sound `rand(6)` only when displayed.) If the best value <= 0 nothing is fired and the man stays manned.
3. Fired: state 0 facing the cannon, idle, manned := 0, fired := 0. So a cannon fires at most every second activation of its unit.

Tactic 1 and 2 differ only in the influence map (5.8: tactic 2 adds +100 around every wall cell). A cannon ball that leaves the screen area is dropped without effect.

### 5.13 Bombs and explosions [C]

**Bomb**: when a Bombenleger's lay animation (state 6, 10 frames) reaches frame 6, a bomb object (type 19 if his facing < 4, else 18) is placed on the object layer at **his own cell**, owned by his participant and side, fuse 18 frames. His animation finishes (4 more frames); then step 2 of 5.9 makes him walk away with `path(4)` (one cell per 6 frames, needs moves left; he had >= 3). He is normally 2 cells away when the fuse ends, outside the blast; if `path(4)` finds nothing he stays put and is hit by the centre of his own blast (100, halved for own side = 50).

**Explosion** (bomb fuse end, or cannon ball arrival): the object becomes an explosion. A cannon ball first moves to its target cell; if the men layer there holds a wall or obstacle (0xFB..0xFE) the explosion type is `14 + rand(2)` (the only extra RNG draw), otherwise type 13; bombs always type 13. Explosion animation 9 frames. Sounds: `rand(6)` if displayed; then `rand(100)` (always drawn) and, if < 90 and displayed, another `rand(6)`.
Damage, for k = 0..8 over the 8 neighbours (direction order 0..7) then the centre (k = 8), cells outside `1 <= x < 29, 1 <= y < 45` skipped:
- Resolve multi-cell objects (men-layer codes): 0xFD → use the cell at (x-1, y). 0xFC (wagon part): if (x, y-1) is also 0xFC: use (x-2, y) if (x-1, y) is 0xFC, else (x-1, y); otherwise, if (x-1, y) is 0xFC: use (x-2, y+1) if (x-2, y) is 0xFC, else (x-1, y+1); otherwise use (x, y+1).
- goods marker (type 20) or wagon (21) with HP > 50: HP := 50 and one destroyed marker/wagon is counted (once per marker); otherwise no effect.
- other objects (type >= 12) and fallen men: no effect.
- a fighter: `damage = 100` for k = 8, else `rand(80)` (k even) or `rand(60)` (k odd); for k != 8 then `rand(100) < condition` dodges completely (damage 0); men of the same side as the exploding object take half (shift right). Damage > 0 marks a hit and resets idle. HP <= damage: HP 0, the man falls.
- a wall cell (0xFB) becomes rubble (0xFE) and counts one bombardment hit.

This answers the self-damage question: bombers and cannons do hurt their own side, at half strength, only through the blast area.

Entrance reachability [C, computed from SZENARIO.DAT]: on fields 2, 3 and 4 every attacker anchor reaches all three entrance squares by 8-neighbour steps without crossing any code-6 wall or code-1 obstacle, so "Eingang stürmen" never needs a breach. What blocks storming is occupancy: `path()` treats cells holding men or objects as not walkable, so defenders standing on or in front of the entrance (tactic 6) close the route; an attacker whose path is blocked and who has no enemy adjacent goes idle (5.9), which with both sides on 5/6 easily produces 10 rounds without damage and an undecided end. That is consistent with the original.

Remaining open: the exact frame on which a walking man changes cell (the reserve-then-enter rule of 5.9 is certain, the frame is [L]).

## 6. Sieges and relief armies (0x5F080 start, 0x1F2A0 resolution, 0x1F100 plunder, 0x1FA80 relief, 0x200F0 tribute) [C unless marked]

Sieges are not tactical battles; they are resolved by formula. Records: type 25 = siege (owner = besieger, +4 city, +5 u16 army size), type 26 = relief army (owner, +4 city, +5 u16 army), type 27 = tribute (owner, +4 city, +5 u32 yearly amount). Sieges need privilege 28 "Stadt belagern", relief needs 29 "Stadt entsetzen" (`rules-04b`); privileges 38 (Hauptmann) and 67 (Marschall) "Belagerung & Entsetzung" improve the odds as below.

### 6.1 Start (human, privileges menu 0x5F080)
- Refused if 10 or more sieges exist in the world, or the player already has 3 (`[EXE text]`). Only cities not already besieged can be chosen (cycling through the list).
- Army size: proposed `min(population / 6, 50000)` (unsigned division), adjustable by the player up to 50000. Cost now: `army * 10`, paid with `person_pay` (cash may go negative). The screen also shows `army * 5` as the yearly cost of continuing.
- Creates the type-25 record with counter 4.
- Computer besiegers start sieges through revenge case 7 (`rules-09` §4: army = population / 4, cost 20000).

Relief (privilege 29): only cities besieged by someone else; army proposed `min(population / 10, 50000)`, cost `army * 10`; type-26 record, counter 4.

### 6.2 Timeline
Type-25 and type-26 records are handled in the owner's record countdown (human: turn end step 16 of the economy doc; computer: year-end record pass). The counter is decremented before the handler.
- Siege created in a turn: at that turn end the counter becomes 3 and the handler only shows "the siege goes on" (no roll). From the next turn end on (counter 2 or less) it resolves. When the siege continues, the handler sets the counter back to 2, so every later turn end resolves again.
- Relief: resolved at the first countdown after creation, i.e. at the end of the same turn; the relief record is always deleted afterwards.

### 6.3 Siege resolution (each call)
First, for every computer person x whose office place is the besieged city: `favour_decrease(besieger, x, 1 + rand(5))` (in person order; also in the message-only year).

Then, if counter < 3 (`A` = army, `P` = city population, `p38`, `p67` = besieger holds privilege 38 / 67, 0 or 1):
```
r = rand(100 - 10*p38 - 20*p67)
success if r < A * K / P        (unsigned; K = 55 for a computer besieger, 50 for a human)
```
- Success, computer besieger: plunder (6.4), record deleted.
- Success, human besieger: a mission record of kind 22 (if the player has one) counts +1. The player chooses "brandschatzen und plündern" (0), "Tributzahlungen vereinbaren" (1) or "oder beides" (2): choices 0 and 2 plunder (6.4); choices 1 and 2 create a tribute (6.5). Record deleted.
- Otherwise, if `A < P / 5` and `rand(100) < 50`: the siege collapses (record deleted, message). The `rand(100)` is only drawn when `A < P / 5`.
- Otherwise the siege continues: computer: counter := 2. Human: asked whether to continue; yes: pay `A * 5`, counter := 2; no: record deleted.

### 6.4 Plunder (0x1F100)
1. City wealth (+0x12) `-= 20 + rand(30)`, floored at 0.
2. Population `= population * (70 + rand(20)) / 100`, at least 1000.
3. For every computer person x at the city: `favour_decrease(besieger, x, 20 + rand(40))`.
4. Every branch in the city not owned by the besieger: every goods stock halved (integer, `>> 1`), wood storage area (+0x23) halved, every transporter condition (+0x2D..+0x32) halved. Quirk: stone storage (+0x25) is untouched.

### 6.5 Tribute
Type-27 record, counter `3 + rand(4)`, yearly amount `population + 100 + rand(1000)` (population after plunder if both were chosen); the besieger receives `2 * amount` at once as ransom. Each countdown of the record pays `amount` (0x200F0); the record is deleted after the payment that brings the counter to 0. Because the record is created during the countdown pass and appended to the heap, the same pass visits it once more [L], so the number of yearly payments is `3 + rand(4)` including one in the conquering turn.

### 6.6 Relief resolution (0x1FA80)
Let S = the siege record on the same city (if none: message, relief deleted). `A_r`, `A_s` = relief and siege armies, `P` = population; privilege flags as above for the relief owner (`r38`, `r67`) and the besieger (`s38`, `s67`).
- Computer relief owner: two rolls, relief first: `x = rand(100)`, `y = rand(100)`. Relief succeeds if `A_s + y + 10*s38 + 20*s67 < P/8 + A_r + x + 10*r38 + 20*r67` (P/8 unsigned shift). Success deletes the siege.
- Human relief owner: no roll: succeeds if `A_s + 10*s38 + 20*s67 < A_r + P/10 + 10*r38 + 20*r67`; on success the siege is deleted and the relief owner receives cash equal to the city population (reward from the citizens).
- The relief record is deleted in every case.

### 6.7 Effects of an active siege elsewhere
- Production in the city: city factor -60 (economy doc 3.3, "blocked"); shipments from or to the city return (4.4); directors avoid it (`rules-09` §5).
- The city record's siege byte +0x6D is read by the price formula (economy doc 2.2) but is never written anywhere in the code [C: full sweep of the code object], so the siege price modifiers (all prices +25 %, Waffen +15 % more) never take effect. A port aiming at parity should keep +0x6D at 0.

## Open questions
1. Human order screens (0x55BC8, 0x579A1, 0x57E70): how types 51/53/54 are created by humans, their counters, escort requests.
2. Cannon step logic (0x39EA4..0x3A6AB), prisoners' later use (type 55 records), `0x85E21`.
3. Sieges (types 25/26: 0x1F2A0, 0x1FA80) are not covered here.
4. ~~The training argument of the computer castle's year-end `add_troops`.~~ Answered in 3b: `40 + rand(40)`.
