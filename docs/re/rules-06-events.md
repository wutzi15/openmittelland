# Events: disasters, divine events, fate (Schicksal), sub-plots (Abenteuer), contests

Confidence tags and offsets as in `rules-00-map.md`. `rand(n)` = `random_n(n)`. `standing(p, k)` as defined in `rules-05-family.md` (one RNG call, about k/1000 of p's net worth). `favour_increase(a, b, x)` raises `favour(b→a)`, `favour_decrease(a, b, x)` lowers it (`rules-03` §1). `skill_up(p, g, x)` = skill + x capped at 150 (0x43BE0); `skill_down(p, g, x)` = skill - x floored at 50 (0x43C50). `wealth_up(c, x)` = city wealth (+0x12) + x capped at 99 (0x43F30); `wealth_down(c, x)` = minus x floored at 0 (0x43F90). `pick_property(kind, owner)` = uniformly random property of that kind and owner (0x42B90, one RNG call, none if the owner has none). "Year gate" = `rand(6) + 1605 <= year` (the roll is always made).

`Fy(k) = ((70 + rand(60)) * (year - 1570) * k) / 3000 + 1` (unsigned, integer) is the year-scaled amount used by many fate events.

Option bits (word 0x86AD1) are listed in `rules-05` §8. Messages are appended to the turn report; they do not block.

## 1. Where events run

| when | what | condition |
|---|---|---|
| human turn end, after tips, before the record loop (0x61A70) | fate event 0x226F0 (§4) | option Schicksal; once, then again if `rand(100) < 50` |
| human turn end, phase-6 tail (0x64108) | sub-plot starter (§5) | option Abenteuer and `year > 1605 + rand(6)` |
| human turn end record loop | sub-plot handlers for record types 32..49 (§5) | record owned by the human |
| year end (0x66220), after offices | city disaster 0x18BD0 (§2) | `rand(100) < 12` |
| year end, after auction announcement | world fate 0x26640 (§3.2) | option Schicksal and `rand(100) < 15` |
| year end, world record loop (owner 0xC0) | disaster aftermath type 2 (0x19360), contest type 4 (0x19890) | every year |
| year end, after that loop | contest announcement 0x19480 (§6) | option Abenteuer, `rand(5) + 1605 < year`, `rand(100) < 4` |
| year end, after computer-person actions | divine event 0x59940 (§3.1) | `rand(50) < prayers`, see §3.1 |

## 2. City disasters (0x18BD0) [C]

Rolled once per year with probability 12 %:
1. `c = rand(10)`. Nothing happens unless population (+0x0E) > 1499 and the city's "recent disaster" bit (byte +0x6E bit 7) is clear. No retry.
2. Type d: repeat `d = rand(4)` until bit d is set in byte +0x6E. Start masks (world init): 5 = {fire, plague}, 7 = {fire, flood, plague}, 13 = {fire, plague, avalanche}. Set bit 7.
3. Record type 2 (owner 0xC0, key c, counter 0x83, +4 d, +5 = 1 unless plague, +6 u16 year).
4. Cosmetic text `2d + rand(2)`.
5. Population loss `L = rand(T[d].pop) * pop / 100 + 10`; if `L + 1010 > pop` then `L = pop - 1000`. `pop -= L`.
6. `wealth_down(c, rand(T[d].wealth) * (3 + rand(7)))` (one byte product).
7. Every person (slot order, computer persons too) with a branch (kind 0) in c, each branch in property order: every good `stock = stock * T[d].keep / 100`; wood storage (+0x23) `= s * (T[d].wood + rand(10)) / 100`, stone (+0x25) `= s * (T[d].stone + rand(10)) / 100`. Loss = Σ goods `(old - new) * price[c][g]` + Σ storage `(old - new) * storage_price[c][i]` (city words +0x6F, +0x71). Humans see a message per branch; history entry code 6 (plague) or 7 (flood).
8. If loss > 0 and the person has insurance bit 0 (Katastrophen): `loss` is paid back in full.

| d | name | T.pop (% max) | T.wealth | T.keep goods % | T.wood % | T.stone % | film |
|---|---|---|---|---|---|---|---|
| 0 | fire (Brand) | 25 | 4 | 70 | 60 + rand(10) | 80 + rand(10) | feuer.flc |
| 1 | storm flood (Sturm/Springflut) | 20 | 3 | 75 | 75 + rand(10) | 85 + rand(10) | none |
| 2 | plague (Pest) | 40 | 2 | 85 | 90 + rand(10) | 90 + rand(10) | none |
| 3 | avalanche (Lawine) | 20 | 3 | 90 | 75 + rand(10) | 80 + rand(10) | lawine.flc |

(table 0x84B34, 6 bytes per type; byte 2 = 25, 15, 5, 15 only selects the `[EXE text]` text.)

**Aftermath (type 2 at year end, 0x19360):** the world record loop decrements the counter first. Counter with bit 7 set (the creation year): clear bit 7 (counter becomes 2). Otherwise with `rand(10) < 2`: plague only `pop = pop * 95 / 100`; then if `pop < 1000` set 1000, else counter + 1 (one more year) with a per-type message. Counter 0: clear the city's bit 7, delete. So a disaster lasts 2 year ends plus 20 % chance per year of extension.

While bit 7 is set the city's wood price is raised (`rules-02` price update). No production or trade block.

## 3. World-level random events

### 3.1 Divine events (0x59940, named npc_misfortunes) [C]
Triggers: (a) year end, `rand(50) < P` where P (byte 0x85CBF) counts "Beten" in church by any human since the last trigger (0x5D1F0 menu entries 8 and 12 add 1), and only if a computer Erzbischof exists; P := 0 on trigger; (b) privilege "Wunder wirken" (privileges menu code 0x56): pay `standing(p, 10)`, then trigger; (c) computer-person action 0x59F80.

`k = rand(11)`; if k = 3 a computer person at city `rand(10)` is drawn first; if none, k := 2.

| k | event | effect |
|---|---|---|
| 0 | weeping Madonna | `c = rand(10)`; `wealth_up(c, 5 + rand(10))` |
| 1 | divine health | `c = rand(10)`, `rand(12)` text; every person whose location byte (+0x0F) = c: life +5 (≥ 95 → 100) |
| 2 | lightning burns a church | `c = rand(10)`; pop < 1100 → 1000, else `pop * 10 / 11` |
| 3 | insanity | the drawn person loses his office (office_vacate) |
| 4 | drought | `c = rand(10)`; every type-8 production run of Korn or Wein in a branch in c: workers := 0 |
| 5 | ideal weather | as 4 but workers doubled, cap 99 |
| 6 | inspiration | `c = rand(10)`, `rand(12)` text; every person located in c: `skill_up(p, rand(15), 5)` |
| 7 | summer snow | message only |
| 8 | tornado | A = rand(10), B = (A + 1 + rand(9)) mod 10, n = 50 + rand(50), cosmetic rand(3), rand(4), rand(5), rand(3); `wealth_down(A, 5 + rand(10))`, `wealth_up(B, 5 + rand(10))`; pop(A) -= n (floor 1000); pop(B) += n/2 + 1 |
| 9 | epidemic | as 1 but life -5 (below 5 → 0); illness name `rand(12)` |
| 10 | silver strike in Bergenhöh | every city: price of Silber `= price / 2 + 1` |

"Located in c" includes humans whose location byte is c (city office holders of c).

### 3.2 World fate (0x26640), year end, 15 % with Schicksal [C]
`k = rand(5)` (debug override when `0xEF4A0` is 100..199):
- 0 import duties: `g = rand(15)`; in all 10 cities `price[g] = min(99, price * 3 / 2)`.
- 1 duties lifted: `price[g] = max(1, price / 2)`.
- 2 boom: `c = rand(10)`, `wealth_up(c, 5 + rand(5))`.
- 3 decline: `wealth_down(c, 5 + rand(5))`.
- 4 siege rumour: `c = rand(10)`, besieger name `rand(12)` (cosmetic). With `rand(2) = 0` nothing more. Otherwise every type-9 record (any owner) whose destination (+10) = c, or whose source branch (+2 as property index) lies in c: destination := byte +2; and kinds 2/3 (sell) become 4/5 (deliver). **Quirk [C]:** byte +2 is a property index, so the new "destination" is a property number, not a city; the arrival code then indexes cities with it. Reproduce only after a runtime check.

## 4. Fate events for a human (0x226F0, "Das Schicksal") [C]

Selection: `k = rand(100)` (debug override `0xEF4A0 >= 200`: k = value - 200). If k equals the previous k (global byte 0xC25AC, start 0, shared by all humans) then k := 100 (nothing). The previous-k byte stores the final k (so 100 after a repeat). Effects use the human h, "branch" = `pick_property(0, h)` unless stated. "nothing" means no event this time.

| k | event | condition | effect |
|---|---|---|---|
| 0 | tax back-payment | year gate, has branch | pay `standing(h, 25)` |
| 1 | tax refund | has branch | receive `standing(h, 25)` |
| 2 | mine speculation | year gate | pay `standing(h, 30)` |
| 3 | salt-mine speculation | | receive `standing(h, 30)` |
| 4 | saved someone from an attack | computer person x at h's location (`person_pick_random`) | `favour_increase(h, x, 2 + rand(5))` |
| 5 | robbers threaten a woman | has branch in city c | `v = rand(2)`; every computer non-clergy person located in c: v = 0 `favour_increase(h, p, 1 + rand(6))`, v = 1 `favour_decrease(h, p, 1 + rand(6))` (two texts: rescue / knocked out and questioned) |
| 6 | goods confiscated | has branch | every good `-(5 + rand(35)) %` (truncating) |
| 7 | church tax back-payment | year gate, religion ≠ 2 (2 = left the church) | pay `standing(h, 30)` |
| 8 | tithe refund | religion ≠ 2 | receive `standing(h, 30)` |
| 9 | inheritance | | receive `Fy(200)` |
| 10 | charity | year gate | pay `standing(h, 20)` |
| 11 | fined for bawdy songs | year gate | `standing(h, 25)`, tavern `rand(30)`; pay |
| 12 | card game loss | year gate | `standing(h, 30)`, tavern `rand(30)`; pay |
| 13 | dice win | | `standing(h, 20)`, tavern `rand(30)`; receive |
| 14 | — | | nothing |
| 15 | director drinks | branch with director | every good: `skill_down(director, g, 1 + rand(5))` |
| 16 | director inspired by wine | branch with director | every good: `skill_up(director, g, 3 + rand(6))` |
| 17 | bad book | branch with director | book `rand(49)`; as 15 |
| 18 | good book | branch with director | book `rand(49)`; as 16 |
| 19 | severe illness | year > 1620 | illness `rand(12)`; life -= 1 + rand(4) (floor 0) |
| 20 | quack | year gate | pay `standing(h, 35)`; life += 1 + rand(3) (cap 100) |
| 21 | orphanage donation | year gate | `standing(h, 30)`, city `1 + rand(9)` names the county `(city-1)/3 + 10`; pay; every computer clergy person located at that county: `favour_increase(h, p, 1 + rand(8))` |
| 22 | useless river island | year gate | pay `standing(h, 15)` |
| 23 | goes bald | | message only |
| 24 | fashion at a feast | computer person x at h's location | text `rand(10)`; `favour_increase(h, x, 1 + rand(10))` |
| 25 | cheese toast | `rand(100) < 33` | text only (`rand(16)`, `rand(2)`) |
| 26 | dowry for a relative | year gate | pay `standing(h, 15)` |
| 27 | pays poor children's schooling | year gate | pay `Fy(150)` |
| 28 | unknown benefactor | | receive `Fy(150)` |
| 29 | garden rest | the drawn branch has a residence (+3 word ≠ 0xFFFF) | life += 1 + rand(4) (cap 100) |
| 30 | warehouse fire | year > 1610, has branch | `x = rand(40)`: wood storage `* (100 - x) / 100`, stone `* (100 - x/2) / 100` |
| 31 | leaking roof | has branch | perishable goods (Korn, Fisch, Wolle, Holz, Rind, Fell, Salz, Tuch, Pfeffer; table 0x84BDD) each `-rand(30) %` |
| 32 | grand feast in a county | county `10 + rand(3)`; some person located there ≠ h | every computer person located there: `favour_increase(h, p, rand(10))` |
| 33 | charity ball | year gate, has branch | pay `standing(h, 20)` |
| 34 | wins a cart | has branch | +1 Karren at condition 100 |
| 35 | directors' course | first branch with a director | pay `standing(h, 20)`, `v = rand(2)`, tavern `rand(30)`; v = 0: every director of h's branches gains `skill_up(d, g, 5 + rand(5))` for every good; v = 1 money wasted |
| 36 | smuggling | | receive `Fy(250)` |
| 37 | guild exhibition | year gate | `Fy(350)`, city `rand(10)` (text); pay; every computer Bürgermeister: `favour_increase(h, p, 1 + rand(10))` |
| 38 | paintings sold | | receive `Fy(200)` |
| 39 | theft | has branch | every good `-(10 + rand(35)) %` |
| 40 | director quits | branch with director | director := none |
| 41 | — | | nothing |
| 42 | old silver coins | has branch | receive `Fy(200)` |
| 43 | bookkeeper | year gate, has branch | pay `X = Fy(200)`; wood and stone storage each `+ X*5 / (storage_price[c][i] * 4)`, cap 4999 |
| 44 | cart breaks | branch with ≥ 1 Karren | Karren -1 |
| 45 | good book on production | | `g = rand(15)`, book `rand(49)`; `skill_up(h, g, 1 + rand(10))` |
| 46 | confusing book | | `skill_down(h, g, 1 + rand(10))` |
| 47 | alms | year gate; random person x ≠ h (any) | `X = Fy(200)`; h pays X, x receives X |
| 48 | public park | | `c = rand(10)`; `wealth_up(c, 3 + rand(5))`; every computer non-clergy person located in c: `favour_increase(h, p, 1 + rand(8))` |
| 49 | storm damages residence | first branch with residence | residence condition (+4) -= 1 + rand(30), floor 0 |
| 50 | candle fire in residence | first branch with residence | text amount `1 + rand(1000)` and colour `rand(3)`; condition as 49. **Bug [C]: the helpers' payment in the text is never deducted** |
| 51 | saves someone from a tax | computer person at h's location | `favour_increase(h, x, 3 + rand(10))` |
| 52 | strike | year ≥ 1608, h's worker pay level (+0xF0) ≤ 99, has a type-8 run | random run picks the branch; every run of h in that branch: workers `* (70 + rand(20)) / 100` |
| 53 | land tax | year gate, first branch with residence | pay `V * (3 + rand(6)) / 100`, V = residence value (below) |
| 54 | building subsidy | residence construction record (type 13) exists | text with `V * (3 + rand(6)) / 100` of the residence under construction. **Bug [C]: never paid** |
| 55 | field fire | first type-8 run of Korn | workers halved |
| 56 | overloaded transport | first type-9 shipment | qty halved |
| 57 | uncovers a fraud | random computer person x | `favour_increase(h, x, 1 + rand(10))` |
| 58 | mows the lawn | | message only |
| 59 | mercenaries strike | owns a toll castle | message only (no effect found) |
| 60 | camp fears patrols, no raids | owns a robber camp | message only (no effect found) |
| 61 | storm damages transport | first shipment | transporter condition (+11) `* (30 + rand(40)) / 100` |
| 62 | winds blow a balloon off course | first shipment with Ballon | destination := (dest + 1 + rand(9)) mod 10; deliver kinds 4/5 become sell 2/3 |
| 63 | carters desert | first shipment whose transporter is not Schiff/Tauchboot (`t mod 3 ≠ 1`) with count ≥ 2 | count -1 |
| 64 | carter defects | as 63, plus a random branch of anyone not owned by h | count -1; that branch gains 1 transporter of type t at the shipment's condition |
| 65 | wins at gambling | first branch | gains `1 + rand(2)` transporters of type `2*rand(2)` (Karren or Ballon) at 100 |
| 66 | escort runs off | first shipment of kind 3 or 5 (escorted) | kind -1 (unescorted) |
| 67 | carters pilfer | first shipment | qty `* (70 + rand(20)) / 100` |
| 68 | drunken foreman | first shipment | destination as in 62 |
| 69 | order book stolen | year > 1610, has a type-8 run | that branch: all 4 order slots zeroed; N = 5 * (owner certificate +0x05) + 5; every run in the branch switches good g → `z = (g + 1 + rand(N - 1)) mod N`, workers `clamp(w * WB[g] / WB[z], 1, 60)`, means `clamp(m * MC[g] / MC[z], 1, 60)` (WB wage base, MC means cost, `rules-02`) |
| 70 | builder enlarges storage | has branch | wood and stone storage `* 106 / 100`, cap 4999 |
| 71 | wood store turned into stone | branch with wood storage > 0 | stone += wood; stone < 5000: wood := 0, else wood := stone - 4999 and stone := 4999 |
| 72 | worker bonus | year gate; W = total workers of h's type-8 runs > 0 | pay `(2 + rand(5)) * W` |
| 73 | workers chip in | W > 0 | receive `(2 + rand(5)) * W` |
| 74 | director's own bonus | branch with director | pay `Fy(1000)` |
| 75 | helps another human | year gate; random human x ≠ h | h pays `standing(h, 40)` to x |
| 76 | inherits a Dampfkarren | year > 1604 | random branch gets 1 Dampfkarren at condition `10 + rand(80)` |
| 77 | aunt's inheritance | | receive `Fy(250)` (text `rand(10) + 2`) |
| 78 | outbuilding fire / vermin | first branch with residence extension 0x20 or 0x40 | if 0x40 is clear: "Nebengebäude" burn, clear 0x20; else vermin, clear 0x40 |
| 79 | vagrants join camp | owns a robber camp | recruits (below), kind 1 |
| 80 | robbers desert | camp | each of 4 troop types `-rand(3)` (floor 0) |
| 81 | robber discipline | camp | each training `-rand(5)` (floor 0) |
| 82 | robber captain trains | camp | names `rand(18)`, `rand(10)`; each training `+rand(5)` (cap 100) |
| 83 | robbers want extra pay | camp | pay `troops * (10 + rand(10))` |
| 84 | robbers eat stock | camp | every good `-(10 + rand(30)) %` |
| 85 | robbers defect | camp | transfer out (below), kind 1 |
| 86 | robbers defect to you | camp | transfer in (below), kind 1 |
| 87 | robber treasure | camp | names `rand(18)`, `rand(10)`; receive `Fy(1500) / 2` |
| 88 | stray soldiers join | owns a toll castle | recruits, kind 2 |
| 89..93 | as 80..84 for the castle (desertion, discipline, manoeuvre, pay demand, consumption) | castle | same numbers |
| 94 | mercenaries poached | castle | transfer out, kind 2 |
| 95 | mercenaries defect to you | castle | transfer in, kind 2 |
| 96 | fortifications improved | castle | names `rand(18)`, `rand(10)`; byte +3 (security) `+rand(10)`, cap 100 |
| 97 | fortifications improved (same text) | castle | byte +4 (casemates) `+rand(10)`, cap 150 |
| 98 | Ark of the Covenant | castle | text `rand(9)`; life += 1 + rand(5) (cap 100) |
| 99 | cracks in the walls | castle, else camp | byte +3 `-= rand(10)`. **Bug [C]: the floor test is on an unsigned value, so it wraps to 246..255** |

Helpers:
- **Recruits** (79, 88): property P = `pick_property(kind, h)`; `z = 1 + rand(6)`; T = sum of the 4 troop counts (+0x58..+0x5B); if `T + z > P[+4]` (capacity: camp straw stores, castle casemates) then `z = P[+4] - T`. If z > 0: `y = rand(4)`; z times: `training[y] = (count[y] * training[y] + 30) / (count[y] + 1)`, `count[y] += 1`, `y = (y + 1) mod 4`.
- **Transfer out** (85, 94): up to 20 tries of `x = random human`; accept x ≠ h owning a property Q of the same kind (`pick_property`). If found: `z = min(1 + rand(6), T(P))`; if `T(Q) + z > Q[+4]` then `z = Q[+4] - T(Q)`. If z > 0: `y = rand(4)`; z times: advance y while `count_P[y] = 0`, then `count_P[y] -= 1`, `count_Q[y] += 1` (y does not advance after a transfer). Training is not changed.
- **Transfer in** (86, 95): the same with P and Q swapped (capacity of h's property).
- **Residence value V** (0x44230), residence word r (+3, u16; 0xFFFF none), city c: `V0 = ((r >> 8) + 150) * ((build_level_c + 550) * RB[r & 0x1F] / 600) / 250`, build_level = city byte +0x81, `RB = 500, 1200, 2500, 4000, 6000, 8000, 11000, 15000, 19000, 25000` (0x86698); then `+V0/10` if bit 0x40, `+V/5` if bit 0x20, `+V/3` if bit 0x80 (each on the running value); `V = (wealth_c + 150) * V / 200`. r >> 8 is the condition byte (+4).

Notes:
- Payments use person_pay and may drive cash negative; receipts use person_add_cash (clamped, `rules-02`).
- The table has 17 gated entries; when the gate fails nothing happens. Effective event rate is below 100 % per call.
- **Handbook (H p.58):** praying or candles winning God's favour: praying feeds the divine-event trigger (§3.1), which is as likely to harm as to help; candles have no effect on fate here.

Evidence: 0x226F0 (jump table 0x22780, 100 entries), 0x42B90, 0x43BE0, 0x43C50, 0x43F30, 0x43F90, 0x44230, 0x18BD0, 0x19360, 0x26640, 0x59940, 0x5D1F0, 0x61A70, 0x66220.


## 5. Sub-plots (Abenteuer)

### 5.1 Starter (phase-6 tail of turn-end settlement, 0x64108) [C]
After the record loop of the human's turn end: if option Abenteuer and `year > 1605 + rand(6)`: `k = rand(100)` (debug override `0xEF4A0 >= 500`: k = value - 500). k = 0..16 runs starter k below; 17..99 nothing. So at most one new sub-plot per turn, 17 % when the gate passes. Most starters refuse if the human already has a record of that type. A record created here gets its first counter decrement at the human's **next** turn end (`rules-05` §1), where the handler for its type runs with the decremented counter c.

Dialog answers: `ask` returns the 0-based index of the chosen line. Where the code reads a number from a spinner ("%m"), the allowed range was not decoded [L]; the value is used as entered.

All starters and handlers write report text; only the rules are listed. Names, taverns, book titles, adjectives are cosmetic random draws whose counts are given so the RNG sequence can be kept.

| k | starter | type | size | handler |
|---|---|---|---|---|
| 0 | 0x19C30 inventor | 32 | 6 | 0x19DC0 |
| 1 | 0x1A000 book publishing | 33 | 9 | 0x1A240 |
| 2 | 0x1A680 trade expedition | 34 | 13 | 0x1A770 |
| 3 | 0x1ACC0 concert | 36 | 11 | 0x1AEA0 |
| 4 | 0x1B400 wager | 37 | 10 | 0x1B660 |
| 5 | 0x1B8D0 robbers' raid on a count's money | 38 | 11 | 0x1BC00 |
| 6 | 0x1C040 cartwright cartel | 39 | 9 | 0x1C270 |
| 7 | 0x1C350 scholar's lecture | 40 | 11 | 0x1C4C0 |
| 8 | 0x1C820 excavation | 41 | 4 | 0x1C990 |
| 9 | 0x1CB90 church seminar (sales bonus) | 42 | 6 | 0x1CCE0 |
| 10 | 0x1CF60 card game 17+4 | 43 | 6 | 0x1D000 |
| 11 | 0x1D4E0 contract to topple an official | 44 | 11 | 0x1D6A0 |
| 12 | 0x1D840 wedding speech | 45 | 9 | 0x1D9A0 |
| 13 | 0x1DD70 peasant revolt | 46 | 12 | 0x1DE90 |
| 14 | 0x1E580 sales contract | 47 | 16 | 0x1E7F0 |
| 15 | 0x1E910 cart lease | 48 | 9 | 0x1EA80 |
| 16 | 0x1EC50 regatta | 49 | 5 | 0x1ED90 |

Type 35 exists in the size table (4 bytes) but has no starter or handler.

### 5.2 Scripts

**32 Inventor.** Start (no type 32): tavern `rand(30)`; up to four yes/no dialogs (listen, approach, keep listening, buy the directions); any refusal ends it. Accepting the last: pay 1000 (fixed), create type 32 counter 4. Handler [L]:
- c = 3: visit; refuse → delete. Else price `P = Fy(800)` stored at +4 (u16); second refusal → delete; else pay P.
- c = 2 or 1: pay P again (three payments in total).
- c = 0: `u = rand(3)`, `n = 1 + rand(B[u])` with `B = 8, 2, 6` (0x84BD8); branch = `pick_property(0, h)`; if u = 1 (Tauchboot) and the branch city is not navigable (0x85A00: 1 0 0 1 1 1 0 0 1 1 for cities 0..9) then u := 0. Add n transporters of type `3 + u` (Dampfkarren, Tauchboot, Flugmaschine) at condition 100. Delete.

**33 Book.** Start: cosmetic `rand(117)`, `rand(32)`, `rand(2)`, `rand(4)`; refuse → end. Else book `b = rand(49)` (+4), price `Fy(500)`, content grade `a = rand(11)` (+5, lower is better), cosmetic `rand(2)`; refuse → end; else pay `Fy(500)`, create type 33 counter `5 + rand(3)`, +7 (u16) = 0xFFFF. Handler [L]:
- first run (+7 = 0xFFFF): three print offers, each 4 random bits (`rand(2)` ×4, offer 0 first) forming quality `q` 0..15 (leather, illustration, paper, binding). Chosen q → +6; copies n (spinner, starts at 10) → +7; pay `(q + 5) * n`.
- c = 0: leftover copies unsold; delete.
- otherwise demand `D = (q + 5) * (11 - a) + rand(10)`. If `n < D`: receive `(q + 8) * n * 2`, delete (sold out). Else receive `(q + 5) * D * 2`, `n -= D`.

**34 Expedition.** Start: human's certificate level (+0x05) > 1; no type 34; `person_pick_random(10 + rand(3), clergy 0, any, computer)` result must not be 0 (**quirk:** the code tests ≠ 0, so "none" (0xFF) passes and person 0 fails). Create type 34 counter 5, destination `rand(7)` (+4), cosmetic `rand(10)`. Handler [C]:
- c = 4: others' wagons `W = 10 + rand(90)` (+9), others' mercenaries `M = W - 9 + rand(150)` (+11). Own free Karren = sum of type-0 transporters over all branches; none → delete. Spinner: carts `n` (+5), mercenaries `m` (+7); n = 0 → delete. Else `W += n`, `M += m`, pay `100n + 200m`, remove n Karren from branches in property order.
- c = 3, 2, 1: "no news".
- c = 0: branch = `pick_property(0, h)`. Ambush if `W * rand(100) >= 100 * M`: everything lost, delete. Else if `M * rand(100) < 70 * W`: modest return: n Karren back at condition `30 + rand(50)`, goods 9, 10, 11 (Glas, Salz, Tuch) each `3n + rand(3n)`. Else rich return: pay `Fy(1000)` for a celebration feast, n Karren at `30 + rand(50)`, goods 12, 13, 14 (Silber, Pfeffer, Waffen) each `4n + rand(3n)`. Delete. The other merchants are fictitious.

**36 Concert.** Start: no type 36; host x = `person_pick_random(10 + rand(4), any)` (counties or principality, computer); `R = office_index(x) + rand(3) + 2 + title(x)` (+5); expected quality `Q = 5 - rand(6) * R / 30` (+6, signed byte); cosmetic `rand(10)`; refuse → end; else type 36 counter 2, +4 = x. Handler [L]:
- c = 1: three ensembles i = 0..2: size `n_i = (60 + rand(50)) * R / 100 + 2`; badness `s_0 = Q + rand(3)`, `s_1, s_2 = rand(11)`; fee `F_i = (30 - s_i) * n_i`. Cosmetic `rand(10)` ×3. Choice i: +7 = n_i, +8 = s_i, +9 (u16) = F_i, pay F_i.
- c = 0: if `R < n` and `s < Q`: success, every person located at 10..13: `favour_increase(h, p, 2 + rand(10))`, payout `(100 + rand(100)) * F / 100`; else if `n < R` and `Q < s`: failure, `favour_decrease(h, p, 2 + rand(10))` for the same persons, payout 0; else payout `(80 + rand(40)) * F / 100`. Payout is truncated to 16 bits before it is added. History code 3 (n, s). Delete.

**37 Wager.** Start (no uniqueness check; a second wager can coexist): tavern `t = rand(30)`, cosmetic `rand(8)`, `rand(7)`, `rand(5)`; type 37 counter 1; kind `w = rand(3)` (+4): w = 0 price bet: city `rand(10)` (+5), good `rand(15)` (+6), current price (+7); w = 1 arm wrestling (+6 = t); w = 2 newspaper contest: city `rand(10)`, discipline `rand(8)`. Stake `standing(h, 50)` (+8, u16: truncated to 16 bits). Cosmetic `rand(4)`. "Nein" deletes. Handler (next turn) [C]: win if w = 0: stored price ≤ current price; w = 1: human's life > 14; w = 2: `rand(50) < 100` (**always true**, plus cosmetic `rand(9)`). Win: receive stake; lose: pay stake. Delete.

**38 Robbers' raid.** Start: no type 38; x = random computer person who is male; h owns a robber camp (`pick_property(1, h)`). Type 38 counter 2, key = camp property index. Cosmetic `rand(3)`. x asks whether h runs the camp.
- deny: if the robber-camp law (0x85A36) is off: `favour_decrease(h, x, rand(10))`, delete; if on: +5 = 0 (x will denounce).
- admit with the law off: invitation (below). Admit with the law on: x wants hush money, spinner amount b; answer "threat" → +5 = 0, end; `b < standing(h, 40)` → `favour_decrease(h, x, rand(10))`, delete; else pay b to x, then invitation.
- invitation: count name `rand(117)` (+4), cosmetic `rand(10)`, `rand(2)`, +5 = 1.
Handler [L]:
- c = 1, +5 = 0: arrest raid; fine `standing(h, 50)`; delete.
- c = 1, +5 = 1: camp troops T; T = 0 → delete. Spinner robbers r; r = 0 → delete. Move robbers: for i = 0..r-1, troop type `i mod 4`: if that type is non-zero, move one (+7 + type); +6 = r.
- c = 0: for i = 0..r-1: `rand(100) < 65` → one robber returns to camp type `i mod 4` (cap 150), else lost (L counts the lost). Cosmetic `rand(3)`. Goods 11, 12, 13 (Tuch, Silber, Pfeffer) to the camp, each `L * (4 + rand(18)) + rand(25)`: **quirk [L]** loot grows with the robbers lost, not with those returning. Cosmetic `rand(18)`, `rand(10)`. Delete.

**39 Cartwright cartel.** Start: no type 39; city `rand(10)`, cosmetic `rand(3)` ×2; first answer declines. Yearly fee `P = standing(h, 20)` (+7, u16); spinner years y; cancel or y = 0 → end. Type 39 counter y, +4 city, +5 (u16) year; pay commission `P / 2`. Handler: every turn end pay P; delete at c = 0. **Effect** (turn start, 0x4B900): when any human receives bought transporters (type 11), they arrive at condition `100 - rand(30)` if some type-39 record of another owner exists, else 100. The cartel holder's own purchases are unaffected. [C]

**40 Lecture.** Start: no type 40; type 40 counter 2; cosmetic +4 `rand(9)`, +5 `rand(2)`, +6 `rand(10)`, +8 `rand(10)`, `rand(117)`, `rand(7)`; "Warum?" deletes. Handler [L]:
- c = 1: cosmetic `rand(10)`; dialog repeats while the answer is 0 or 1 (each repeat `rand(10)`); answer 2 continues. Good `g = rand(15)` (+7). Refuse → delete. Asking price `A = Fy(1500)`; offer o (+9, u16): answer 0 takes A, otherwise a spinner. Accepted if `A * rand(100) < 100 * o`, else delete.
- c = 0: pay o, `skill_up(h, g, 5 + rand(10))`, delete.

**41 Excavation.** Start: no type 41; branch `pick_property(0, h)` with a residence; cost `Fy(1000)`; accept: pay, type 41 counter `2 + rand(5)`, key = branch index. Handler [L]: if c = 0 or the residence is gone: stop, delete. Else next cost `C = Fy(1000)`; `rand(100) < 60`: find (`rand(15)`), choose "display at home" (no effect) or "town hall" (`wealth_up(branch city, rand(8))`); then "continue for C?": no → delete, yes → pay C.

**42 Church seminar.** Start: no type 42; type 42 counter 8, title `rand(4)` (+4), city `rand(10)` (+5); fee `standing(h, 10)`; cosmetic `rand(4)`; refuse → tavern `rand(30)`, delete; accept → pay fee. Handler [L]:
- c = 7: seminar (cosmetic `rand(5)`, `rand(20)`, `rand(3)`, then `1 + rand(3)` times `rand(10)`, `rand(11)`).
- c = 6: if some computer clergy person exists and `rand(100) < 40`: the envelope only invites again; every person located in city +5: `favour_increase(h, p, rand(4))`; delete. Otherwise the envelope holds selling tricks and the record stays.
- c = 0: forgotten, delete.
- **Effect:** while a type-42 record of the seller has counter < 7, sale revenue per unit is `price + 2` (`rules-02` §4.4, also shown in the order book). So 6 years of bonus (c = 6..1) when the envelope is good.

**43 Card game 17+4.** Start: no type 43; opponent x = random computer person; type 43 counter 1, +4 x, +5 tavern `rand(30)`; plays at once (handler body). Handler [C], also run each later year while the record lives:
- c = 0 and `rand(100) < 50`: x does not show up; delete.
- stake s from a spinner; 0 or "ablehnen" → delete.
- Deck of 52: card i has value `i mod 13 + 2` for `i mod 13 < 9`, else 11 (**quirk:** jack, queen, king and ace all count 11). Shuffle: 200 times draw `a, b, c = rand(52)` each and rotate a ← b ← c ← old a.
- x gets cards 0, 1; h gets 2, 3; a pair totalling 22 counts 21. x draws while `(21 - x_total) * 13 > 95` (total ≤ 13). h draws on request until ≥ 21.
- Both over 21 or equal: draw. Else h loses if x ≤ 21 and (h > 21 or h < x): x receives s, h pays s. Otherwise h wins: h receives s, x pays s.
- Rematch offer: accept → counter := 1 (play again next turn), else delete.

**44 Topple an official.** Start: no type 44; target `t = person_pick_random(10 + rand(3), any clergy, index rand(13), computer)` (rand(13) drawn first); reward `Fy(5000)`; cosmetic `rand(3)`; accept → type 44 counter `3 + rand(6)`, +4 t, +5 t's location, +6 t's office index, +7 (u32) reward. Handler [C]: c = 0: if +6 = 15 receive the reward, else failure text only (no penalty); delete. Otherwise if t still holds the same office: "not yet"; else success text, +6 := 15, counter := 1 (paid next turn). Computer-person turnover never replaces a type-44 target (`rules-05` §9).

**45 Wedding speech.** Start: no type 45; `x = person_pick_random(10 + rand(3), clergy 0, any, computer)` (**no check for none**); accept → type 45 counter 2, +4 county, +5 x. Handler [L]:
- c = 1: three monks i: cosmetic `rand(3)`, grade `g_i = rand(5)`, fee `F_i = rand(10) + (4 - g_i) * 50 + 100`. Choice: +6 = g, +7 (u16) = F, pay F.
- c = 0: `band = g * 3 / 5`: band 0 (g 0, 1): payout `(110 + rand(190)) * F / 100` and every person located at the county: `favour_increase(h, p, 2 + rand(10))`; band 1 (g 2, 3): `(100 + rand(20)) * F / 100`; band 2 (g 4): `(20 + rand(100)) * F / 100` and `favour_decrease(h, p, 2 + rand(10))` for the same persons. Receive payout; delete.

**46 Peasant revolt.** Start: no type 46 **of anyone** (one revolt in the world at a time); h owns a toll castle; a computer principality person x exists (`person_pick_random(13, clergy 0, any, computer)`); county `k = rand(3)`; accept → type 46 counter 4, +4 x, +5 k. Handler [L]:
- c = 3: total mercenaries S over h's castles; cosmetic `rand(3)`; spinner n (0 if S = 0). n = 0: `favour_decrease(h, x, 10)`, delete. Else +6 = n; from each castle and troop type take `count * n / S` (floor) into record +8 + type; the remainders (sum of `count * n mod S`) are taken one unit at a time from random non-empty types (`rand(4)`, then cycling) while the remainder ≥ S.
- c = 2: `rand(100) < 66`: victory text (`rand(7)`); else defeat (`rand(3)`), each pooled type `* (60 + rand(30)) / 100`. Then "not over": with `rand(100) < 60` counter := 3, so c = 2 repeats next turn.
- c = 1 or 0: survivors return to h's castles in property order, each castle filled up to 150 troops in total (counting troops already ordered in its order book); R = returned. Reward shown: `100n + 50R`. **Bug [C]: the reward is never paid.** If `n > 19`: city `3k + 1 + rand(3)`; if no branch of anyone exists in that city, h gets a new branch there (0x42870). Delete. Troops that do not fit are lost.

**47 Sales contract.** Start: no type 47; type 47 counter `2 + rand(3)`; city `C = rand(10)` (+4); good g (+5) = first good whose yield in the city of `pick_property(0, h)` is ≥ 140 (15 if none: **out-of-range quirk**); quantity `Q = 200 + rand(800)` (+6, u16); yearly pay `Y = Q * price[C][g] / 2` (+8, u32); penalty `(c * Y / 100) * 100 + 100` (+12, u32) with c the initial counter; cosmetic `rand(117)`, `rand(4)`; refuse → delete. Handler (every turn end, c after decrement): if units of g sold into C this turn (city +0x31 + 4g) < Q: pay the penalty, delete. Else receive Y; delete at c = 0. [C]

**48 Cart lease.** Start: no type 48; branch B = `pick_property(0, h)`; `n = 2 + rand(10)`; pay `P = Karren price in B's city (city word +0x73) * n / 2` (+5, u32); cosmetic `rand(3)`; answer 0 declines; else type 48 counter 5, key B, +4 n. Handler [C]:
- c = 4: if B has fewer than n Karren: contract broken, delete. Else remove n Karren from B, receive P.
- c = 3..0: `rand(100) < 10`: the merchant smuggled with them, carts lost, delete. Else a "you receive %d Taler" text (**no payment**); at c = 0 n Karren return to B at condition 100, delete.

**49 Regatta.** Start: no type 49; tavern `rand(30)`; answer 0: join (second dialog: 1 → end) → v = 0; otherwise side with the fishermen (second dialog: 0 → end) → v = 1. Type 49 counter 1, +4 v. Handler [C]:
- v = 0: cosmetic `rand(3)` ×2; receive `Fy(200)`; delete.
- v = 1: x = computer non-clergy person located in Fürstenbrück (`person_pick_random(0, 0, any, computer)`); none → delete. `f = min(4, favour(x→h) / 21)`; bribe spinner b. If `2000 f + b < 10000`: refused. Else pay b to x, `skill_up(h, Fisch, 5 + rand(10))`, and for office indices 0..4 in Fürstenbrück (first holder each): `favour_increase(h, p, rand(10))`. Delete.

## 6. Newspaper contests (record type 4, owner 0xC0)

**Start (0x19480), year end** with option Abenteuer, `rand(5) + 1605 < year`, `rand(100) < 4`, and no type-4 record [C]:
- judge J = random computer person; if none, nothing. Type 4 (75 bytes) counter 2, +4 city `rand(10)`, +5 J; five disciplines d = 0..4 with good `3d + rand(3)` (+6..+10); fee `Fy(1000)` (+0x47, u32); cosmetic `rand(6)`.
- Each human in slot order (at most 6 entrants stored) toggles disciplines (bits 1, 2, 4, 8, 16); entrant slot +0x0B + i, mask +0x29 + i. A human with mask 0 does not take part (slot reused). Fee paid: `fee * (number of disciplines selected)`.
- No human entered: cancelled, record deleted. Otherwise the remaining slots up to 30 get random computer persons with mask `rand(32)`.

**Resolution (0x19890)**, the next year end (the world record loop calls it whatever the counter): for each discipline d with good g, every entrant e with bit d set scores `favour(J→e) * max over e's branches of (yield[branch city][g] * effective_skill(branch, g))` (`rules-02`). A discipline is judged only if at least one human scored in it. Top three by score (strictly greater wins, ties go to the lower slot; score 0 gets no place); prize for place 1, 2, 3 = `4, 3, 2 * fee`, paid from 3rd to 1st. Record deleted. [L]

## 7. Chronicle and history records

- Type 1 record (owner 0xC0, the Emperor, see `rules-03` title awards) is passed to 0x18AE0 at the end of every year end: with option Geschichtsschreibung and `year < 1700`, the year's chronicle text (text id `year - 627`, skipped when it starts with "?") is shown. Its counter is set to 1 and its byte +4 (emperor number) increases in 1612, 1619, 1637 and 1657 (deaths of Rudolf II, Matthias, Ferdinand II, Ferdinand III). [C]
- Type 57 history entries (0x2D750, 9 bytes): +4 code, +5 u16 year, +7, +8 arguments, counter 25 (expire after 25 own turns). Codes: 0 debtors' tower, 1 wedding, 2 feast (level), 3 concert (size, grade), 4 child born (sex), 5 promotion, 6 plague (city), 7 flood (city), 8 robber camp opened (`3EE60`), 9 castle/camp related (`0x549C0`, year end). They feed the `[EXE text]` screen at mission success (`rules-10`). [C]

## Handbook comparison
- 05-events open question 4 (number of adventures): 17 sub-plots plus the newspaper contest; the 100-entry fate table and the 11 divine and 5 world events come on top.
- 15-misfortune: insurance against catastrophes pays the full branch loss of city disasters; Prozesskosten, Überfälle and Sabotageakte are the other three insurances (bits 1..3), cost 1 % of net worth each per year (`rules-02`). The flood of H p.58 (fields, boats) matches disaster type 1 (storm flood, goods and storage loss) and fate events 55 and 61 rather than a separate rule. Storage renting (H p.58) has no counterpart in these events.
- Praying (H p.58): see §3.1.

## Open
- Spinner limits in sub-plots 33, 34, 38, 39, 40, 43, 46, 49. [L]
- Fate event 59/60 texts promise a mercenary strike / no raids; no state change found in 0x226F0; check the castle/camp code for a flag (`rules-07`).
