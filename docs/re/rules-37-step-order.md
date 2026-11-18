# Step order: turn start, turn-end settlement, year-end world part

Source: `turn_start_reports` 0x4B900..0x4CC76, `turn_end_settlement` 0x61A70..0x66160 (record dispatcher 0x63661..0x64103, jump table 0x63680), `year_end_world` 0x66220 (part 0x68790..0x69BB8), `prices_update` 0x49FE0, director driver 0x59640, order refresh 0x45820, `branch_add_goods` 0x439F0, music front end 0x41E50. Confidence: **[C]** call order and call arguments read in the disassembly; **[L]** decompiler reading only. "RNG" marks a step that draws from `random_n` (0x36FF0), directly or through a callee (callee reach checked with a call-graph walk over the whole image). `p` = the current human.

This document fixes **order** only. Formulas stay in the cited docs.

## 1. Turn start `turn_start_reports(p)` 0x4B900 [C]

Runs after the prison check of `play_turn` (`rules-01` §3.1).

| # | address | step | RNG | doc |
|---|---|---|---|---|
| 1 | 0x4B90x | clear `sales[g][c]` (0x85A62, 15×10 u32), offence word 0x85CBA = 0, `sold[c][g]` (city +0x31, 10×15 u32); 0x85CC0 = 1; give-away counter 0x85CC1 = 0 | – | rules-02 §2.2, §4.5; rules-04b §2.2 |
| 2 | 0x4BA01 | "Nächster Spieler" box if config bit 0x86AD2 & 1 | – | formats §9.3 |
| – | 0x4BA3x | **if year > 1600** (`cmp [0xEF4B8], 0x640`), steps 3..17; otherwise skip to 18 | | rules-23 §1 |
| 3 | 0x4BA6F | open "Erträge" window | – | formats §9.3 |
| 4 | 0x4BACF | blocked cities: every type-25 record (any owner) marks city +4 | – | rules-02 §4.4 |
| 5 | 0x4BB13 | sale bonus `b = 2` if `p` has a type-42 record with counter < 7 | – | rules-02 §4.4 |
| 6 | 0x4BB53 | **type-9 arrivals** of `p`, heap order, every record regardless of counter, each deleted | RNG: one `rand(100)` per `branch_add_goods` call (delivery kinds 4/5 and blocked returns; sale kinds 2/3 call only `branch_add_transporters`, no RNG) [L for the per-kind branches] | rules-02 §4.4, §5.1 |
| 7 | 0x4BE34 | **troop return**: every record of `p` of type 51, 53 or 54 adds +5..+8 to the garrison of property key, deleted | – | rules-07 §4 |
| 8 | 0x4BEAB | 0x446F0: garrison trim. Every property of kind 1 or 2 (castle/camp): while troop sum (+0x58..+0x5B) > capacity (byte +4), remove one from slots in turn 0,1,2,3,0,… | – | not elsewhere [L] |
| 9 | 0x4BEE3 | blessing scan: every type-23 record (any owner) sets `bonus10 = 10` for the cities of county +4 | – | rules-02 §3.3 |
| 10 | 0x4BF4C | weather: `w[g] = 70 + rand(60)`, g = 0..14 | RNG ×15 | rules-02 §3.3 |
| 11 | 0x4BFE3 | **production**: every type-8 record of `p`, heap order: output formula; `rand(10)` (0x4C23B); `branch_add_goods`; delete | RNG ×2 per record (`rand(10)`, then `rand(100)` in 0x439F0) | rules-02 §3.3 |
| 12 | 0x4C3FE | "Produktion" page; per listed good `rand(3)` + skill gain 0x43BE0; then for every remaining index up to 14 `rand(3)` + 0x43BE0 | RNG, one `rand(3)` per index from the first listed good to 14 | rules-02 §3.3 quirk |
| 13 | 0x4C782 | "Exporte" page; `cash_add(p, total)`; overflow line | – | rules-23 §1 |
| 14 | 0x4C7F6 | wait for click (**the report closes before steps 15..17**) | – | |
| 15 | 0x4C814 | **type 10** (built transporters) of `p` with counter **exactly 0**: `branch_add_transporters(cond 100)`, delete | – | rules-02 §6 |
| 16 | 0x4C883 / 0x4C8B5 | find the first type-39 record whose owner ≠ `p`; then **type 11** (repaired) of `p` with counter exactly 0: `rand(30)` **always**, condition `100 − (cartel ? r : 0)`, delete | RNG ×1 per due type-11 record | rules-02 §6 (see §5 correction) |
| 17 | 0x4C96E | **type 13** (residence) of `p` with counter 0 or ≥ 0x80: `set_residence` 0x520A0, delete; then items cleared, window closed | – | rules-11 |
| 18 | 0x4C9D6 | **certificate award** 0x494F0 (runs every turn start, 1600 included; `music(0x0E)` does not draw) | – | rules-00 §4, rules-19 §3 |
| 19 | 0x4C9E5 | per city c = 0..9: `S = wood + stone m²` of `p`'s branch in c (`0x42A50(0, c, p)`, +0x23 + +0x25), **S = 15 if no branch**; per slot s = 0..2: residence offer `rand(100)`, `rand(8)`, `rand(10)` → 0x85CC5; storage offer `((rand(10) + 15 + 20s) * S) / 100 + 1`, cap 4999 → 0x85D01; `rand(100) < 40` → +10000 | RNG ×5 per slot, 150 total, interleaved per slot exactly in this order | rules-11 (buy outright), rules-02 §5.2, rules-34 |
| 20 | 0x4CB47 | clear 0x85D46[0..191] (listener memory) | – | rules-04 §1 |
| 21 | 0x4CB77 | church prices: 0x85D3D = `standing(p, 40)`, 0x85D41 = `standing(p, 30)` | RNG ×2 | rules-PROGRESS (correction line) |
| 22 | 0x4CB95 | lender mood 0x85D45 = `50 + rand(20)` | RNG ×1 | rules-11 |
| 23 | 0x4CBA6 | proposal counter 0x85E07 = 0; once-per-turn flag 0x85E79 = 0 | – | rules-05 §2, rules-04b |
| 24 | 0x4CBB4 | **order refresh**, properties 0..191 with owner `p`: `0x45820`; then every slot with status 3: reserve 0x49370, status = 2, `0x45820` again | – | rules-36 §6, rules-02 §3.1 |
| 25 | 0x4CC61 | **directors** `0x59640(p, 0)`: every property of `p` with a director (+0x53 ≠ 0xFF), property order: `rand(151 − pay)`, director AI 0x58410 | RNG (1 + inside 0x58410) per director | rules-09 §5 |
| 26 | 0x4CC66 | phase = 4 | | |

There is no title award, price update or residence/church-price step anywhere else in 0x4B900.

## 2. Turn-end settlement `turn_end_settlement(p)` 0x61A70 [C]

| # | address | step | RNG | doc |
|---|---|---|---|---|
| 1 | 0x61A8B | **directors** `0x59640(p, 1)` | RNG (as §1 #25) | rules-09 §5 |
| 2 | 0x61A90 | `prices_update` 0x49FE0 | RNG ×2 per city (§3.1) | rules-02 §2.2 |
| 3 | 0x61AB2 | title award 0x49700 (`rand(100)` draws; `music(0x1E)` on award) | RNG | rules-03 §2.1 |
| 4 | 0x61B06 | castles (kind 2), then camps (kind 1) of `p`: unreserve confirmed castle orders; per slot troop pay into the bill; training `rand(…)` only when the pay level ≠ 100; re-reserve | RNG only when pay ≠ 100 [L] | rules-07 §3a |
| 5 | 0x61F3x | **orders**, properties 0..191 with owner `p`, slots 0..3 with status 2: kind 1 → type-8 record, `rand(20)`, status 3; kinds 2..5 → type-9 record, `rand(10)`-based cost, escort (odd kind) another `rand(10)`, toll, status 3; kinds 6, 7 → type-9 record from the best stock good, `rand(10)`, status **1**; castle/camp kinds 0x11/0x21 → type 51, 0x12/0x22 → type 52 (status **1**), 0x13 → type 53, 0x23 → type 54 (status 3); these four create no RNG | RNG per executed order of kinds 1..7 | rules-02 §3.2, §4.2; rules-07 §3a |
| 6 | 0x62A1C | `settlement_pre` 0x61410(p) (robbers, battles) | RNG | rules-07 §4..5 |
| 7 | 0x62A34 | report window opens | – | rules-12 |
| 8 | 0x62A9C | taxes and tithe 0x618C0 | – | rules-02 §4.5 |
| 9 | 0x62ABx..0x62ED3 | bill lines in print order: Arbeiter, Produktionsmittel, Transporte, Zölle, Direktoren (fee 0x458F0 per director branch), Verkaufsteuern, Kirchenzehnt, castle pay, camp pay, Informanten (type 6), Saboteure (type 7), Kreditzinsen (type 56, unless ability 57), Versicherungen, Gesamtkosten; pay 0x43860; click | – | rules-02 §7 |
| 10 | 0x62EEE | too many branches: `music(8)`, extra charge | – | rules-02 §7.10 |
| 11 | 0x62FEE | office income 0x5E500 | – | rules-02 §7.11 |
| 12 | 0x6303C | castle toll income | – | rules-02 §7.12 |
| 13 | 0x631F0 | tips, only with option bit 0x08: `rand(5)` | RNG ×1 if option | rules-02 §7.13 |
| 14 | 0x63532 | fate, only with option bit 0x04: 0x226F0, `rand(100) < 50` → 0x226F0 again | RNG if option | rules-06 §4 |
| 15 | 0x6355F | blackmail end, persons 0..191 alive with favour(→p) > 100: `rand(100) < 33` → text, favour = `rand(30)`, `rand(100) < 50` → `retaliate(x, p, 3)` | RNG | rules-02 §7.15 |
| 16 | 0x63629 | `film_skip = rand(100) < 70` (spy informant film flag; the film plays only when it is 0) | RNG ×1, always | §2.1 |
| 17 | 0x63652 | **record countdown** (§2.1) | per handler | |
| 18 | 0x64108 | phase-6 tail: sub-plot starter, grudge, duels (type 22), crime scan, trials (type 18), debt seizure, bankrupt, mission, death | RNG | rules-19 §4, rules-06 §5.1, rules-02 §10, rules-17 |

### 2.1 Record countdown (0x63661..0x64103) [C]

Every record of `p` (`find(0xFF, p, 0xFF)`, heap order): counter −1 (byte, wraps), then dispatch on `type − 6` through 0x63680 (51 entries; type < 6 or > 56 → default).

| types | handler | deleted by the dispatcher? |
|---|---|---|
| 0..5, 7, 12, 16, 17, 20, 21, 23, 35, 50, ≥ 57 | default | yes, when the counter is 0 |
| 8, 10, 11, 13, 14, 18, 22, 51, 54 | none (jump to the loop end) | **never** here |
| 6 spy | inline 0x63936: counter 0 → delete; counter ≤ 15 → +9 = 0; else evidence check 0x440E0 > 2 → text, +10 = 1; first such hit with `film_skip == 0` shows the informant films | own |
| 9 shipment | `shipment_mishaps` 0x174C0 (RNG) | no |
| 15 recruiting | inline 0x63B5F: `rand(200)`, add troops 0x45240 (RNG), texts | always |
| 19 fine | 0x2C0A0 | inside handler |
| 24 conversion | 0x1F050 (RNG) | inside handler |
| 25 siege / 26 relief | 0x1F2A0 / 0x1FA80 (RNG) | inside handler |
| 27 tribute | 0x200F0 | at counter 0 |
| 28 feast | 0x20530 (RNG via music) | inside handler |
| 29 mistress / 30 courtship / 31 child | 0x20B90 / 0x212B0 / 0x221F0 (RNG) | inside handler |
| 32..34, 36..49 sub-plots | 0x19DC0 … 0x1ED90 (`rules-PROGRESS-b`) | inside handler |
| 52 troop transfer | inline 0x63F2A: troops +5..+8 into the garrison of property `location +4 + 14`, text if `p` owns it | always |
| 53 patrol | inline 0x63E96: text only | **no** (returns at the next turn start, §1 #7) |
| 55 prisoners | inline 0x63D0D: `rand(30)`, 0x45240 (RNG), texts | always |
| 56 loan | inline 0x64023: counter 0 → text, `pay(p, +5)`, `rand(100) < 10` → `retaliate(+4, p, 2)`, delete | at counter 0 |

Types 18 and 22 are resolved and deleted in the phase-6 tail (§2 #18). Types 8 and 9 are resolved at the next turn start. Types 10, 11, 13 wait for counter 0 at a turn start (§1 #15..17). Types 51 and 54 wait for the next turn start.

## 3. Year end `year_end_world` 0x66220, world part [C]

| # | address | step | RNG | doc |
|---|---|---|---|---|
| 1 | 0x66258..0x68790 | favour drift, dismissals, turnover, vacancies, elections | RNG | rules-03 §4, rules-05 §9 |
| 2 | 0x68815 | delete every type-5 record with counter ≤ 1; office window closed (0x68858) | – | |
| 3 | 0x6885D | `rand(100) < 12` → **disaster** 0x18BD0 | RNG | rules-06 §2 |
| 4 | 0x68872 | **`settlement_pre` 0x61410** for every live computer person (slot order, controller nibble 0) | RNG | rules-07 §4..5 |
| 5 | 0x688CD | world type-3 record exists → **auction settlement** 0x27680 | RNG | rules-08 |
| 6 | 0x688FB | "Ereignisse in Mittelland" window | – | rules-12 |
| 7 | 0x68957 | world type-3 record with counter 1 → `music(0x1E)` (RNG, `rules-20` §4), herald text 0xC19D0 | RNG if shown | rules-32 |
| 8 | 0x689A0 | `film_skip = rand(100) < 90` | RNG ×1, always | rules-04b §4b |
| 9 | 0x689D8 | **sabotage**: every type-7 record (any owner), heap order: `rand(100) < 25` → strike (only when saboteur or victim is human), insurance, `rand(100) < 70` → one more draw and `retaliate`, film when `film_skip == 0` (first strike only); the record is deleted after a strike, kept otherwise [L] | RNG | rules-04b §4b |
| 10 | 0x6900B | average shipment 0x85E1D update from all type-9 records | – | rules-07 §3.1 |
| 11 | 0x69093..0x69A41 | **computer castles and camps** (tolls, sale, upkeep, troop orders types 53/54 created with counter 2) | RNG | rules-07 §3 |
| 12 | 0x69A46 | option bit 0x04 and `rand(100) < 15` → **world fate** 0x26640 | RNG if option | rules-32, rules-06 |
| 13 | 0x69A73 | world records (owner 0xC0), heap order: counter −1; type 2 → aftermath 0x19360, type 4 → contest judging 0x19890 | RNG (0x19360) | rules-06 §1, rules-32 |
| 14 | 0x69B0C | option bit 0x02: `year > 1605 + rand(5)` and `rand(100) < 4` → contest start 0x19480 | RNG if option | rules-06 §1 |
| 15 | 0x69B4D | 0x85E06 > 0 → decrement; else `rand(100) < 20` → auction announcement 0x269D0 | RNG if 0x85E06 = 0 | rules-08 |
| 16 | 0x69B7D.. | mistress ×10, `npc_act` rolls, divine event, computer record pass 0x69E20, office income, emperor, chronicle 0x18AE0, evidence pass | RNG | rules-09 §2, rules-32, rules-04b §3.6 |

`prices_update` 0x49FE0 is **not** called in `year_end_world`. Its callers are `world_init` and `turn_end_settlement` (0x61A90) only.

### 3.1 A/B rolls inside `prices_update` [C]

Per city c = 0..9, in this order: the 15 prices of c (g = 0..14); then `A = (Y − 1600)/7 + lvl82/11 + 23 + extra88 + rand(12)` (0x4A2C3); then `B = … + 30 + … + rand(12)` (0x4A320); then the six transporter prices of c. Then the next city. The rolls interleave with the price loop per city.

### 3.2 Computer troop-order lifetime [C]

Types 53 (0x6966D) and 54 (0x6973B) are created with counter **2** (`rec_alloc` ECX = 2). Troops are computed from the garrison (`troops[k] * (20 + rand(40)) / 100` for 53) and **not** subtracted from it. Year N: created in the castle step, after that year's `settlement_pre` (§3 #4); the computer record pass (§3 #16) makes the counter 1 (not deleted). Year N+1: `settlement_pre` acts on them, then the record pass makes the counter 0 and deletes them (types 53/54 are not in its keep list, `rules-04b` §3.6).

## 4. Answers to the sim's SPEC-GAP comments

| location | answer |
|---|---|
| `turn.rs:219` certificate award "first" | Wrong. It is §1 #18: after the whole report block (arrivals, troop return, production, transporters, residences) and before the offers. It runs in 1600 too. |
| `turn.rs:286` transporter delivery point | §1 #15/#16: same turn start, after the report closes and before residence deliveries (#17). Condition: counter **exactly 0** (no bit-7 test, unlike type 13). Type 11 always draws `rand(30)`. Skipped entirely in 1600. |
| `turn.rs:304` status 3 → 2 / 1 | No kind distinction at turn start. Every status-3 slot is re-reserved and set to 2 (§1 #24), then 0x45820 drops it to 0 when infeasible. The 1 comes from turn end: kinds 6, 7 and castle kinds 0x11/0x21/0x12/0x22 are set to 1 when executed, kinds 1..5 and 0x13/0x23 to 3 (§2 #5). |
| `turn.rs:344` order of church prices, residence offers, lender mood, residence deliveries, directors | Residence deliveries (#17) → certificate (#18) → residence + storage offers interleaved per slot (#19) → listener clear (#20) → church prices (#21) → lender mood (#22) → counters reset (#23) → order refresh (#24) → directors (#25). |
| `turn.rs:740` directors in the settlement | Very first step, before `prices_update` and the title award (§2 #1). Only the fee is charged later on the bill (§2 #9). `rules-02` §7 step 1 should read "directors; prices_update; title award; …". |
| `turn.rs:988` 8..11, 13 exempt | 8, 10, 11, 13 are exempt. **9 is not a no-op**: it runs `shipment_mishaps` 0x174C0 at its heap position, not deleted. Also exempt: 14, 18, 22, 51, 54 (§2.1). |
| `turn.rs:1022` other handlers | Full table in §2.1. Types that the default deletes at 0: 0..5, 7, 12, 16, 17, 20, 21, 23, 35, 50, ≥ 57. 52, 55, 15 delete always; 53 is never deleted at turn end. |
| `turn.rs:1080` disaster/auction/sabotage/castles/fate order | §3 #3..#12: disaster → computer `settlement_pre` → auction settlement → window → herald → sabotage → computer castles → world fate → world records → contest → auction announcement. |
| `turn.rs:1092` settlement_pre vs. castle step | Not "right after". It runs **before** the auction settlement and before the window, long before the castle step (§3 #4 vs #11). |
| `turn.rs:1124` computer troop orders | Confirmed not taken from the garrison; created with counter **2**, not 1 (§3.2). |
| `turn.rs:1229` group amounts summed | Not covered by this pass. |
| `economy.rs:147` A/B rolls | Per city, after that city's 15 prices and before its transporter prices; A before B (§3.1). "After all prices" is wrong if it means after all cities. |
| `economy.rs:357` castles/camps in give-away | 0x439F0 reads `price = byte [0x85F26 + 0x8C·loc + g]` with no location check, so for locations 14..20 it reads bytes beyond the city table, which are other globals. Whether any reach < 100 is [UNVERIFIED]; "nothing qualifies" is not what the code does. |
| `economy.rs:388` storage S | Confirmed: `S = +0x23 + +0x25` (wood + stone m²) of `p`'s branch in that city, and `S = 15` when `p` has no branch there (§1 #19). |
| `economy.rs:488` type-9 layout | Confirmed by the creation code (kinds 2..5 copy slot bytes 0..6 to +4..+10): +4 kind, +5 u16 qty, +7 good, +8 count, +9 transporter type, +10 destination, +11 condition = branch +0x2D + type. Kinds 6/7 write +4 = kind − 2, +7 = best good, +8 = count, +9 = type, +10 = source city, +11 = condition [L]. |

## 5. Corrections to other docs

- `rules-23` §1 table: type 11 draws `rand(30)` for **every** due record; the cartel record only decides whether the result is subtracted. The type-10/11/13 passes run **after** the report's click, not "before any text".
- `rules-02` §7: directors (0x59640 flag 1) precede `prices_update`.
- `rules-19` §4 item 3: the feast handler is type **28** (0x20530); type 23 (blessing) has no turn-end handler.
- `rules-04b` open question 5: the escort flag 0x20000 is set in §2 #5 for every executed transport order of kinds 2..5 with an odd kind (bit 0 set, i.e. with escort), together with the second `rand(10)` [L].
- `rules-01` §3.2 lists disasters after deaths in one bullet; the exact order is §3 above.

## Open

- §2 #4 and #5 RNG argument details are in `rules-07` §3a and `rules-02` §3.2/§4.2; not re-derived here.
- `turn.rs:1229` (privilege income grouping) is outside this pass.
