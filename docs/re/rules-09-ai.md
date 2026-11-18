# Computer persons, directors, difficulty

Confidence tags as in `rules-00-map.md`. `favour_increase(a, b, x)` raises b's favour toward a, `favour_decrease(a, b, x)` lowers it (career doc). "Offence bit" refers to the offence word of `rules-04b-crime-trials-privileges.md` 2.2; each AI routine collects offence bits for the laws its action breaks and passes them to `add_evidence(actor, bits)` at its end.

## 1. Difficulty [C]
The new-game difficulty (`0x85CBE`, 0..3 keine/leicht/mittel/schwer) is read only by the mission code (0x17690, missions doc) and stored by `game_main`. It does not change any computer behaviour, price or battle rule.

## 2. When computer persons act (`year_end_world`) [C]
In this order, after the auction announcement step:
1. Ten times: `0x20A00(person_pick_random(any))` (mistress events, family doc).
2. For each human h holding an office: with `rand(100) < 40`, a random computer person at h's office location acts (`npc_act`, 3).
3. For each county 10..12: with `rand(100) < 50`, a random computer person of that county acts.
4. With `rand(100) < 90`, a random computer person of the principality acts.
5. For each active spy contract owned by a human on a computer target: with `rand(100) < 50` the target acts.
6. Prayer miracle (church doc).
7. The year-end record pass over computer persons (crime doc 3.6) runs revenge records (4), trials, duels, sieges.
Also at year end: computer castle/camp management and raids (combat doc 3, 4), sabotage strikes (crime doc 4b).

## 3. `npc_act(p)` (0x59F80) [C unless marked]
Only for computer p. Bits w = 0.

Step A, law change (`rand(100) < 40`):
- privilege 56 and `rand(100) < 80`: k = `rand(10)`; else privilege 59 and `rand(100) < 90`: k = `10 + rand(10)`; else privilege 82: k = `20 + rand(10)`; else no change.
- k = 1: sales-tax cap = `rand(33)`; 5: minimum worker wage = `50 + rand(100)`; 6: minimum mercenary wage = `50 + rand(100)`; 7: cash cap = `200000 + 1000*rand(700)`; 8: nothing; 29: maximum tithe = `rand(25)`; any other k: toggle the flag law k (k 0..7 → `0x85A21 + k`, k >= 9 → `0x85A24 + k`).
- If a change was made the action is done (go to the end) and offence bit 0x08 is set when "Kirchenprivilegien" is forbidden.

Step B, up to 10 tries until one action succeeds; each try `c = rand(24)`:

| c | condition | action |
|---|---|---|
| 0 | `rand(100) >= 40` and privilege 5 | buy an indulgence; never happens because privilege 5 is never granted (bit 0x200000 if forbidden) |
| 1 | `rand(100) >= 30`, privilege 8 | feast: b = `rand(5)`, city = `rand(10)`; pay `10000 - b*rand(2000)`; every person of that city: `favour_increase(p, x, 4 - b)`; message if a human lives there; bit 0x4000000 if "Feste" forbidden |
| 2 | `rand(100) >= 20`, privilege 9 | building donation: b = `rand(5)`, city `rand(10)`; pay `10000 - rand(2000)*b`; city wealth `+= 10 - 2b` (max 99) |
| 3 | first privilege held of 16, 31, 40, 53, 58 | 16: building price level of p's city = `(old + 80 + 40*rand(2)) mod 100`; 31: county maximum interest = `(old + 55 + rand(10)) mod 60`; 40: rotate the advisers (crime doc 4.3); 53: bless the county (type 23, counter 5); 58: a random person receives `standing(x, 20)` |
| 4 | privilege 32 | county toll share = `(old + 28 + rand(10)) mod 33` |
| 4 | else not 54 and privilege 84 | conversion record (type 24) for p's location with religion `p.religion mod 2` |
| 4 | else privilege 61 | harshness `0x85A4D = rand(100)` |
| 5 | privilege 55 or 86 | miracle event (0x59940) |
| 5 | else 64 | toggle inquisition |
| 5 | else 69 | rotate the ministers |
| 5 | else 71 | state religion `= (old + 1 + rand(2)) mod 3`; bit 0x100000 if "Religionsfreiheit" forbidden |
| 6 | privilege 85 | canonise a random person r: every clergy x: `favour_increase(r, x, rand(10))`; bit 0x08 if forbidden |
| 7..11 | | trade shipment of Waffen, Wein, Fisch, Tuch, Silber; bits 0x2000, 0x4000, 0x8000, 0x8000000, 0x10000000 when that trade is forbidden |
| 12 | | nothing except bit 0x04 if bribery is forbidden |
| 13, 14 | | trade shipment of good `[0,2,3,4,5,6,8,9,10,13][rand(10)]` |
| 15 | `rand(100) >= 30`, privilege 27, fencing < 80 | fencing `+= 5 + rand(5) + rand(10)` |
| 16 | `rand(100) >= 40`, privilege 25 | toggle insurance bit 2 or 8 (`rand(2)`); message if switched on and a human shares the location |
| 17..23 | | no action, but sets offence bits [L, mapping by code order]: 17 loans (0x01), 18 bribery (0x04), 19 Kirchenprivilegien (0x08), 20 parks (0x200), 21 starvation wages (0x20, unconditional), 22 flying (0x800000), 23 castles (0x1000000), each only while the law forbids it except 21 |

Cases 17..23 do not count as the action, so the loop continues and may collect several bits.

Trade shipment (cases 7..14, good g): `qty = avg_shipment * (40 + rand(120)) / ((50*g)/4 + 50)` (avg_shipment = `0x85E1D`), at least 100. A type-9 record is created for p with the **first branch in the property table** as source (any owner) [C, quirk], counter `2 + rand(2)`, Karren, carts `ceil(qty/100)`, destination `(source_city + 1 + rand(9)) mod 10`, condition 90. If source and destination are both ports: Schiff, `qty *= 3`, ships `ceil(qty/1000)`. If the destination is not Fürstenbrück, the castle of the destination's county accumulates toll: `acc += (100 - county_share) * (rate * qty * V[g] / 100) / 100`. These shipments are the main raid targets (combat doc 4) and feed the castle toll income.

End: `add_evidence(p, w)` (0x5B111).

## 4. Revenge (0x5B140 and 0x5B200) [C unless marked]
`retaliate(actor, victim, years)` creates a type-50 record (actor, +4 victim, counter = years) only if the actor is computer, the victim human and no such record exists yet. The third argument, called "code" in older notes, is the counter: 2..5 depending on the caller (crime doc and combat doc list the callers).

Year end, per type-50 record (after the counter decrement): counter 0 → delete; otherwise one revenge act per year, up to 10 tries of `c = rand(10)` (a = actor, v = victim, loc = a's office location):

| c | action |
|---|---|
| 0 | hire a saboteur against v (type 7, counter 5, no cost) |
| 1 | military, **does not end the year's revenge** (the only case that never sets the done flag, so after it the loop rolls again, up to the 10-try limit; it can even fire twice) [C, instruction-level]. `K` = a random camp of a (`pick_property(camp, a)`). If a has a camp: if v has a castle (random pick) → type-51 attack record (owner a, key K, counter 2, +4 = castle location - 14), troops slot i = `camp_troops[i] * (50 + rand(30)) / 100`; else → type-54 raid record (owner a, key K, counter 2, +4 = `rand(3)` county), troops `camp_troops[i] * (20 + rand(40)) / 100`. Else, if a has a castle (random pick, key K): if v has a camp → type-51 attack on it, troops `castle_troops[i] * (50 + rand(40)) / 100`; else, if the castle's toll rate (castle table +0x14) is below 14: rate += `1 + rand(14 - rate)` (message). Else (a owns neither) and a's office place >= 10: `x` = random property at location `14 + rand(7)` (the castle or camp there); if a has no type-17 record for x, create one (key = x's property index, counter 4) = purchase interest. RNG order inside each branch: the picks, then per troop slot one roll. |
| 2 | `favour_decrease(v, a, rand(30))` |
| 3, 4 | trial against v (crime doc 3.1, computer rule) |
| 5 | office harassment in a's city (needs v's branch there for the price levels): Kämmerer: wage level := 100 (if below 100); else Einkommensniveau := 0 if v lives there and has office income; else Baumeister: storage price level := 100; else Bürgermeister: sales tax `+= rand(33 - tax)`; else Arbeiter verschwinden lassen (keep 80/70/60 % of v's production workers); else Bischof: county tithe `+= rand(10)` max 25; else Zwietracht: `favour_decrease(v, f, 30 + rand(20))` with f = the computer person with the highest favour toward v (0x45080); else as c = 6. Offence bits 0x02 / 0x08 where the laws are broken |
| 6 | duel challenge [C]: needs privilege 26 for a, `fencing(v) < fencing(a) + 20` (fencing = person +0xF6), and no type-22 record of a against v. Text rolls: `rand(12)`, `rand(2)`, `rand(8)`, `rand(6)` (insult wording and voice). v (human) chooses: accept → type-22 duel record (owner a, counter 1, +4 v), fought at v's turn-end tail (economy doc 10); refuse → `favour_decrease(v, a, 10)` (a likes v 10 less) and, only if v holds an office, every person (any controller, v included) whose office place equals v's: `favour_decrease(v, x, 1 + rand(6))`. Both outcomes end the year's revenge. |
| 7 | siege [C]: needs privilege 28 for a. `x` = a random branch of v (`pick_property(branch, v)`, one roll); if v has none, or x's city is already besieged by anyone, the try fails (loop continues). Else type-25 siege (owner a, counter 4, +4 = x's city, army u16 = population / 4, truncated to 16 bits), a pays 20000 (`person_pay`), done. Siege rules: `rules-07` §6. |
| 8 | relief [C]: needs privilege 29 for a. `s` = the **first** siege record owned by v (heap order, not random); if none, or a already has a relief record on that city, the try fails. Else type-26 relief (owner a, counter 4, +4 = s's city, army = population / 6), no cost, done. |
| 9 | a random computer r at loc (not a or v): `favour_decrease(v, r, rand(30))` |

End: `add_evidence(a, bits)` (0x5C10F).

## 4b. Harassment (revenge c = 5) in detail [C, instruction level, 0x5B83E..0x5BCB8]
Setup, once per record before the try loop: a = record +1, v = record +4, `loc` = a's office place (person +0x0F), `B` = `0x42A50(0, loc, v)` = v's branch (property kind 0) at loc, or none. `privilege(a, a, code, 0)` is 0x5E600; `g` = a's global office index (crime doc 4.1). City fields are the city record at `0x85F04 + 0x8C*loc`: +0x7F wage level, +0x80 Einkommensniveau, +0x82 storage price level, +0x83 sales tax.

The branches are tested in this order. The first privilege held selects the branch, and a branch whose inner condition fails ends the **try** without the done flag (the loop rolls a new c). It never falls through to the next office.

| # | privilege (office) | condition | effect, in order | text (args) | bits |
|---|---|---|---|---|---|
| 1 | 14 (g = 5 Kämmerer) | B exists and wage level < 100 | wage level := 100; print if `0x85E08[loc] != 0xFF` (a human owns a property at loc; always true here because of B) | 2723 (a, loc) | none |
| 2 | 15 (g = 5) | v's +0x0F = loc, `office_income(v)` (0x5E500) > 0, Einkommensniveau > 0 | Einkommensniveau := 0; print if a human has +0x0F = loc (`0x433E0(loc, 0xFF, 0xFF, 1)` != 0xFF) | 3107 (a, loc) | none |
| 3 | 17 (g = 3 Baumeister) | B exists and storage price level < 100 | storage price level := 100; print if `0x85E08[loc] != 0xFF` | 3110 (a, loc) | none |
| 4 | 22 (g = 6 Bürgermeister) | B exists and tax < 33 | tax += `rand(33 - tax)` (may add 0); print always | 3113 (a, loc, new tax) | 0x02 if new tax > cap `0x85A22` |
| 5 | 48 → keep 80, else 74 → 70, else 79 → 60 (g = 20 Diakon, 31 Erzdiakon, 32 Inquisitor) | none (no branch, no location test) | print always, **then** for every type-8 record (production run) owned by v, any branch and city: workers (+5) := `workers * keep / 100` (truncated) | 2974 (a, 100 - keep, v) | 0x08 if `0x85A24` != 0 |
| 6 | 51 (g = 22 Bischof) | none | `t = 0x85A38[loc] + rand(10)`, capped at 25, stored back; print always | 3128 (a, name string `0xB6194[loc]`, t) | 0x08 if `0x85A24` != 0 |
| 7 | 80 (g = 32 Inquisitor) | none | f = `0x45080(v)`; `favour_decrease(v, f, 30 + rand(20))`; print always | 2803 (a, v, f) | 0x08 if `0x85A24` != 0 |
| 8 | none of the above | | control jumps into the c = 6 code (duel challenge, §4) and runs it unchanged, including its own privilege-26 test, done flag and RNG draws | | |

Notes:
- Every successful branch sets the done flag, even when the change is 0 (rows 4 and 6) or v has no production runs (row 5).
- Bischof: loc is a county 10..12, so `0x85A38[loc]` is the county tithe `0x85A42 + (loc - 10)` and `0xB6194[loc]` the county name. This settles the [L] on 3128 in `rules-32` §6.2. The cap is the literal 25; the tithe law `0x85A41` is not checked here.
- Row 2 is unreachable: privileges 14 and 15 have the same condition (g = 5, checked in 0x5E600 at 0x5EA13 and 0x5EA6F), so every holder of 15 takes row 1 first. Row 7 is unreachable too: 79 and 80 share the same condition (g = 32, same jump target 0x5F037), so an Inquisitor always takes row 5, which never fails. [C, quirk]
- `0x45080(v)`: scans persons 0..191 in index order and keeps the one with the highest byte `x +0x20 + v` among persons that exist (+0 != 0xFF), are computer (+0x10 high nibble 0) and whose byte is not 101. A candidate must exceed the running maximum, which starts at 0, so ties keep the lower index. If no one has a value above 0 the result is an uninitialised local [C, quirk]. The actor a can be f. Unreachable in practice (previous note).
- RNG draws inside c = 5, after the `c = rand(10)` roll: row 4 one `rand(33 - tax)`, row 6 one `rand(10)`, row 7 one `rand(20)` after the 0x45080 scan. The other rows draw nothing. 0x5E600, 0x42A50 and 0x45080 contain no `rand` call. Row 8 draws whatever c = 6 draws.
- "Else as c = 6" in §4 means this jump; 0x5B200 has no return value. Offices that reach row 8: every office except g = 3, 5, 6, 20, 22, 31, 32.
- Bits are OR-ed into the routine's bit word; at the end it calls `add_evidence(a, bits)` (0x44E00) only when the word is non-zero (the call site is at 0x5C10F).

## 5. Directors (0x59640, 0x58410) [C unless marked]
At every human turn start (flag 0) and turn end (flag 1), for every branch of that human with a director:
```
budget = ((director_budget * (150 + rand(151 - pay_directors))) / 150) * (600 - favour(director→owner)) / 500     (unsigned)
```
`pay_directors` is person +0xEF; with pay 151 or more the `rand` argument wraps (151: division by zero) [C, bug]. A lower pay gives a larger random bonus and a director who likes his employer less gets more money; both read reversed but are literal.

`director_ai(branch, budget, flag)`:
1. Besieged cities (type-25 targets) are excluded as source and destination.
2. Confirmed transport slots are released and their feasibility rechecked.
3. `budget = budget * (100 - fee%) / 100`, `fee% = sum of the director's skills over the goods he may trade / (9 * that count)` (0x458F0; goods `0 .. 5*certificate + 4`). [re-checked, C] The range uses the **director's own** certificate level (person +0x05), which is always 0 for computer persons (`rules-00` §4), so in practice `fee% = (skill[0] + ... + skill[4]) / 45` over the director's Korn..Holz skills, i.e. about 10 to 13 % for typical skills of 83..150. The goods the director may produce (step 7) use 0x43DD0 = the higher of owner's and director's certificate levels, i.e. effectively the owner's.
4. For every good g in stock: local value `(100 - tax_c) * stock * price_c[g] / 100`. For every other city d and transporter t (Schiff and Tauchboot only between ports): `n = ceil(stock / CAP[t])`; if n exceeds the usable transporters `count*condition/100`, the missing number (beyond count) is noted and n = usable; n is capped by `budget*2/10 / TC[t]` and by 60; shipped `q = min(stock, n*CAP[t])`; value = rest sold locally + q at d's price and tax − `n*TC[t]`. Best option per good kept.
5. Repeatedly take the good with the highest positive value. If it is a shipment and no transport order was made yet: order type 3 (escorted, needs t < 2 and value > 4000 + rand(6000)) or 2; slot filled (qty, good, count, t, destination), status 2 if feasible and reserved; `budget -= (type) * TC[t] * n / 2`. With flag 0 the missing transporters are bought if affordable (type-10 build record, counter = build time, pay count × city transporter price). Then the whole remaining stock of that good is sold locally at the city price (added to the sales array, cash to the owner).
6. Flag 1: every transporter type with condition < 80: repair cost `(100 - cond) * count * price / 100`; if below the budget pay it, create a type-11 record (counter 1) with the count, and set the branch count to 0 until it returns.
7. Production (at most two orders including the transport order): `L = 155 - (year - 1600)` before 1620, else 135. For goods the director may trade with city yield > 100: `score = (E + 200) * (L * base[g] * (skill + 250) / 3500) / 300` (skill term fixed at 350 when skill >= 140). Pick the highest `score * 200 / base[g]`; per-worker cost `cw = ((wage_level + 450) * (WF * wage_base[g] / 100)) / 500 + MC[g]*10/R[g]`; workers `w = min(budget / cw_total, 30 + rand(10))`; output `w * score / 100`, limited so that output/17 fits the free storage (`w = free * 1700 / score`); means `m = w*10 / R[g]`; slot type 1, status 2 if feasible; `budget -= w*cw + m*MC[g]`.
8. Storage [C, re-checked line by line]. Let `A`, `B` = the city's wood and stone prices per m² (city +0x6F, +0x71), `Wd`, `St` = the branch's wood and stone areas (+0x23, +0x25), `budget` = the director's remaining budget, `OW`, `OS` = the sizes of this turn's offers for this city summed by type (`0x85D01`, three u16 per city; a value above 10000 is a stone offer of value - 10000, else a wood offer). The offers are only summed into these local counters; the offer table itself is **not** reduced, so the same offers are still shown to the player and can be used again next call [quirk].
   - Purchase, **only at turn end (flag 1)**: `need = used * 140 / 100 - (Wd + St)` with `used` = the storage the director's new production orders of step 7 will fill: the sum over those orders of `expected_output / 17` (integer). Goods already in stock are not counted here. First type `t = 1` (stone) if `B < A + 7`, else `t = 0` (wood); then the other type. For each of the two types, while `need > 0`: `q = min(budget / price_t, offers_t, need)`; if `q > 0`: pay `q * price_t`, `budget -= q * price_t`, area_t += q, offers_t -= q, need -= q.
   - Swap, only at turn end (flag 1) and if `budget >= 0` after the purchase: `D = B - A`. If `D < 7`: target type stone (t = 1), source wood; cap `c = OS` (remaining stone offers) if `D < 1`, else `c = min(budget / D, OS)`. If `D >= 7`: target wood (t = 0), source stone, `c = OW` (no budget cap, because this direction earns money). Then `q = min(c, source area)`; source area -= q and the owner is credited `q * price_source`; target area += q and the owner pays `q * price_target`; offers_t -= q. The budget is not reduced by the swap. So a director converts as much as the offers allow, every turn end, toward stone whenever stone costs less than 7 more than wood per m².

Tables CAP, TC, R, MC, WF, base, wage_base: economy doc.

## Open questions
1. Exact mapping of the dummy cases 17..23 of `npc_act` (read from code order, not from the jump table).
2. Computer persons' family and career behaviour (family and career docs).
