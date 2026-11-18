# MITTELL.EXE rules map

Scope: where the game rules live in `MITTELL.EXE` and how the game state is laid out. Formulas are in the subsystem docs (`rules-01-*` onwards). Addresses are linear (code object at `0x10000`, data object at `0x80000`, see `00-loader.md`). "Evidence" names refer to the RE Ghidra project `<scratch>` (program `MITTELL`); the function names given here are the ones applied there.

Confidence tags: **[C]** confirmed by reading the code and, where noted, the instruction-level arithmetic; **[L]** likely, read from decompiler output without instruction-level check; **[G]** guess from strings or usage, needs confirmation.

## 1. Analysis setup (RE side only)

- Watcom C/C++32 (runtime banner 1988-1995), DOS/4GW. Register calling convention: arguments in EAX, EDX, EBX, ECX, rest on stack, result in EAX.
- The Ghidra program is a raw load of both objects with fixups applied. 48 functions start with a Watcom stack probe (`push N; call 0x6B392`); these probe calls were replaced by NOPs inside the Ghidra program only, so the decompiler does not lose the stack frame.
- 55 switch jump tables live inside the code object. Their case labels are not functions.

## 2. Global game state

The save game (`confi2\game<N>.dat`) is the raw concatenation of these blocks, in this order (save 0x42360, load 0x42200) [C]:

| # | address | size | content |
|---|---|---|---|
| 1 | `0xE377C` | 0x300 | 32 human name strings, 24 bytes each |
| 2 | `0xE3A7C` | 0xBA00 | persons: 192 records x 248 bytes |
| 3 | `0xDC86C` | 0x4800 | properties: 192 records x 96 bytes |
| 4 | `0xE106C` | 10000 | record heap (events, orders in transit, contracts) |
| 5 | `0xEF4B8` | 2 | current year (u16) |
| 6 | `0xEF4C6` | 1 | person index whose turn it is (`0xC0` = none) |
| 7 | `0xEF4C5` | 1 | turn phase of that person (0..6) |
| 8 | `0x85A18` | 4 | record heap: free bytes |
| 9 | `0x85A1C` | 4 | record heap: used bytes |
| 10 | `0x85F04` | 0x578 | cities: 10 records x 140 bytes |
| 11 | `0x86532` | 0xE1 | goods table: 15 records x 15 bytes (constant data, saved anyway) |
| 12 | `0x85A20` | 0x4E2 | miscellaneous globals (laws, castles, statistics, offers) |

The RNG state is not part of the save (see `rules-01-rng-calendar.md`). Slot 5 is the autosave written at the start of each year ("Das letzte Spiel fortsetzen").

## 3. ID spaces

### Locations (one byte, used everywhere a "place" is stored) [C]
| id | name | kind |
|---|---|---|
| 0 | Fürstenbrück | city (capital, no toll) |
| 1 | Finsterwalde | city |
| 2 | Moortal | city |
| 3 | Lichterheide | city |
| 4 | Greifenheim | city |
| 5 | Bergstedt | city |
| 6 | Falkenhorst | city |
| 7 | Waldstein | city |
| 8 | Lauenburg | city |
| 9 | Kronsbach | city |
| 10 | Mooringen | county |
| 11 | Bergenhöh | county |
| 12 | Auenthal | county |
| 13 | Mittelland | principality |
| 14 | Hohenfels | toll castle |
| 15 | Seefels | toll castle |
| 16 | Waldenfels | toll castle |
| 17 | Meucheltann | robber camp |
| 18 | Henkerswald | robber camp |
| 19 | Dunkelforst | robber camp |
| 20 | Säbeltal | robber camp |

The name order is from the pointer table at `0xB6194` [C]. City-to-county table `0x868D4` = `[10,10,10,10,11,11,11,12,12,12]` [C]: cities 1-3 lie in Mooringen, 4-6 in Bergenhöh, 7-9 in Auenthal. Fürstenbrück is filed under county 10 but is exempt from toll in the code. This matches the handbook toll table (H p.25) for the city grouping.

**Disagreement [needs runtime check]:** the toll code indexes the castle table with `county - 10`, and the castle income code indexes the same table with `castle_location - 14`. Taken literally that pairs Mooringen with Hohenfels, Bergenhöh with Seefels and Auenthal with Waldenfels. The handbook pairs Mooringen with Waldenfels, Bergenhöh with Hohenfels and Auenthal with Seefels. Either the name table order for castles differs from the location id order, or the handbook is wrong. The castle records at `0x8647C` (26 bytes each) start with words `0x07DD`, `0x07DE`, `0x07DF`, which may be text ids that would resolve this.

### Offices (global office index 0..33) [C]
Name table `0xB4378`. A person stores (level location, index within level); the global index is index for cities, index+10 for counties, index+23 for the principality (function 0x43FF0; within-level index 15 means "no office" and maps to 0).

| global | office | level | church |
|---|---|---|---|
| 0, 1, 2 | Ratsherr | city | no |
| 3 | Baumeister | city | no |
| 4 | Richter | city | no |
| 5 | Kämmerer | city | no |
| 6 | Bürgermeister | city | no |
| 7, 8 | Priester | city | yes |
| 9 | Domherr | city | yes |
| 10, 11, 12 | Geheimrat | county | no |
| 13 | Hauptmann | county | no |
| 14 | Justizberater | county | no |
| 15 | Finanzberater | county | no |
| 16 | Kanzler | county | no |
| 17 | Vogt | county | no |
| 18 | Kellermeister | county | yes |
| 19 | Sakristan | county | yes |
| 20 | Diakon | county | yes |
| 21 | Abt | county | yes |
| 22 | Bischof | county | yes |
| 23, 24, 25 | Hofrat | principality | no |
| 26 | Marschall | principality | no |
| 27 | Justizminister | principality | no |
| 28 | Finanzminister | principality | no |
| 29 | Großkanzler | principality | no |
| 30 | Regent | principality | no |
| 31 | Erzdiakon | principality | yes |
| 32 | Inquisitor | principality | yes |
| 33 | Erzbischof | principality | yes |

There are 100 city offices (10 per city), 39 county offices (13 per county) and 11 principality offices: 150 in total.

### Goods (0..14) [C]
Korn, Fisch, Bier, Wolle, Holz, Rind(fleisch), Erz, Wein, Fell, Glas, Salz, Tuch, Silber, Pfeffer, Waffen (name table `0xB6618`). Production means per good (`0xB4304`): Feld, Boot, Kessel, Schaf, Wald, Weide, Grube, Faß, Bottich, Ofen, Mine, Webstuhl, Mine, Land, Schmiede.

### Transporters (0..5) [C]
Karren, Schiff, Ballon, Dampfkarren, Tauchboot, Flugmaschine (`0xB42A4`).

### Troop types [C]
Castle: Söldner, Musketier, Kanonier, Offizier. Camp: Räuber, Flintenschütze, Kanonier, Bombenleger (order in the 4 troop slots per castle/camp is not yet tied to names, see `rules-08-castles-bandits.md` when written).

### Titles [C]
Male `0xB6A58`: Herr, Bürger, Edelmann, Ritter, Landgraf, Freiherr, Baron, Graf, Herzog, Fürst. Female `0xB6A80`: Frau, Bürgerin, Edelfrau, Hofdame, Landgräfin, Freifrau, Gräfin, Baroness, and two more (not yet dumped). Title index 0 = Herr/Frau (no title).

### Residences [C]
Kate, Hütte, Haus, Bürgerhaus, Landhaus, Villa, Landsitz, Burg, Wasserburg, Schloß, plus extensions Park, Nebengebäude, weitläufige Ländereien (`0xB4340`).

## 4. Person record (248 bytes at `0xE3A7C + 248*i`, i = 0..191)

Persons 0..159 are created by the world generator, humans and children take free slots. Byte 0 = `0xFF` marks a free slot. Known fields:

| offset | size | meaning | conf. |
|---|---|---|---|
| 0x00 | 1 | gender (0 male, 1 female); 0xFF = slot free | C |
| 0x02 | 1 | remaining life in years ("Lebenskraft"); new persons get 25 + rand(15); the end-of-turn tip warns when it is below 15 and the person has no spouse and no child on the way | L |
| 0x03 | 1 | religion (0 evangelisch, 1 katholisch) | L |
| 0x04 | 1 | spouse person index, 0xFF none | G |
| 0x05 | 1 | trade certificate level 0..2 (Handelszertifikat I..III = may produce goods 0-4 / 0-9 / 0-14, see economy doc 3.1). Starts 0 for everyone. Only ever raised by the turn-start check 0x494F0 (if +0x06 is set: clear it, +0x05 += 1, show the certificate). Computer persons therefore always stay at 0. | C |
| 0x06 | 1 | certificate pending flag: set at the year end when a human is appointed to an office and either +0x05 == 0, or +0x05 == 1 and the office is a county/principality office with index within level >= 2 (the code adds 1 to the threshold when the whole byte +0x10 is 0, which never holds for a human because the controller nibble is 1, so the threshold is always 2); consumed at the human's next turn start | C |
| 0x07 | 1 | insurance flags, low 4 bits each one insurance | L |
| 0x08 | 1 | name index (NPC: first-name table index; human: index into the 32 human name strings) | L |
| 0x0D | 1 | title index | L |
| 0x0F | 1 | office level location (city 0..9, county 10..12, principality 13) | C |
| 0x10 | 1 | low nibble: church office flag; high nibble: controller (0 = computer, 1 = human) | C |
| 0x11 | 1 | office index within level, 15 = none | C |
| 0x12, 0x13 | 1 each | per-office constants from tables `0x868E9` / `0x8690B` (meaning open) | G |
| 0x18 | 4 | cash (signed, clamped to -9 999 999 .. 9 999 999) | C |
| 0x1C | 4 | cached net worth | C |
| 0x20 + j | 1 x 192 | this person's favour toward person j (self = 100, start 48..52) | L |
| 0xE0 + g | 1 x 15 | production skill for good g (start 100 for humans) | C |
| 0xEF | 1 | pay level for directors, percent (start 100) | L |
| 0xF0 | 1 | pay level for workers, percent | C |
| 0xF1 | 1 | pay level for mercenaries, percent | C |
| 0xF2 | 1 | pay level for robbers, percent | C |
| 0xF4 | 1 | colour (0..5) | C |
| 0xF5 | 1 | hotkey | C |
| 0xF6 | 1 | rand(10) at creation, meaning open | G |

## 5. Property record (96 bytes at `0xDC86C + 96*i`, i = 0..191)

| offset | size | meaning | conf. |
|---|---|---|---|
| 0x00 | 1 | kind: 0 branch (Niederlassung), 1 robber camp, 2 toll castle, 3 free slot | C |
| 0x01 | 1 | location id | C |
| 0x02 | 1 | owner person | C |
| 0x03, 0x04 | 1 each | branch: residence data (0xFF = none); castle: security 60 and casemates 40; camp: camouflage 60 and straw stores 20 | L |
| 0x05 + 2g | 2 x 15 | goods in stock, max 9999 each | C |
| 0x23 | 2 | storage m², wood (branch start 15; castle/camp 4999) | C |
| 0x25 | 2 | storage m², stone | C |
| 0x27 + t | 1 x 6 | transporters owned per type, max 99 (new branch: 1 Karren) | C |
| 0x2D + t | 1 x 6 | transporter condition per type, percent (start 100) | C |
| 0x33 + 8k | 8 x 4 | order book slots k = 0..3 (layout in `rules-02-economy.md`) | C |
| 0x53 | 1 | director person, 0xFF none | C |
| 0x54 | 4 | director budget per year | C |
| 0x58 + k | 1 x 4 | troop counts (castle/camp) | C |
| 0x5C + k | 1 x 4 | troop training, 0..100 | C |

## 6. City record (140 bytes at `0x85F04 + 140*c`)

| offset | size | meaning | conf. |
|---|---|---|---|
| 0x00 | 2 | article/text id (values 1999..2008) | G |
| 0x0E | 4 | population (start: Fürstenbrück 10000; 5000 or 2500 elsewhere) | C |
| 0x12 | 1 | wealth level 0..99 (start 70, 60 or 40) | L |
| 0x13 + g | 1 x 15 | yield (Ergiebigkeit) of good g: one of 25, 50, 75, 120, 140 | C |
| 0x22 + g | 1 x 15 | current price of good g, 1..99 | C |
| 0x31 + 4g | 4 x 15 | units of good g sold into the city in the current turn | C |
| 0x6D | 1 | siege flag (nonzero = besieged) | L |
| 0x6E | 1 | bit 7: recent fire flag (raises wood demand); low bits: start values 5, 7 or 13, meaning open | G |
| 0x6F | 2 | yearly level A (formula in economy doc) | L |
| 0x71 | 2 | yearly level B | L |
| 0x73 + 2t | 2 x 6 | purchase price of transporter type t | L |
| 0x7F | 1 | wage level (start 50) | C |
| 0x80, 0x81, 0x82 | 1 each | further levels, start 50 (0x82 feeds levels A/B; likely building price level) | G |
| 0x83 | 1 | start 15, meaning open | G |
| 0x84 | 4 | start 12000 / 8000 / 5000; likely price of branch rights | G |
| 0x88 | 4 | start 10 / 5 / 0; added to levels A and B | L |

## 7. Goods table (15 bytes per good at `0x86532 + 15*g`) [C]

| bytes | meaning |
|---|---|
| 0..9 | demand weight by city wealth decile (index = wealth/10). Goods 0-4 fall 120,116,...,84; goods 5-14 rise 84,88,...,120 |
| 10..11 | u16 wage base per worker: 50 + 5g (Korn 50 ... Waffen 120) |
| 12 | value weight: 7 + g (Korn 7 ... Waffen 21) |
| 13..14 | u16 base output per worker: 155, 221, 218, 239, 186, 211, 200, 192, 280, 177, 182, 209, 179, 164, 256 |

## 8. Record heap

Variable-length records in `0xE106C`, 10000 bytes. Header (every type): byte 0 = type (0..57; bit 7 set = deleted), byte 1 = owner/subject person (0xC0 = world, no owner), byte 2 = key (often a property index or a second person, 0xFF = none), byte 3 = counter, usually "turns/years remaining". Allocation `rec_alloc(type, owner, key, counter)` appends a record of the table size, zero-filled after the header (0x424A0); deletion sets bit 7 (0x42550); compaction runs after every human turn and at year end (0x42580). Search is linear by (type, byte1, byte2) with 0xFF as wildcard (0x426B0 / 0x427B0), in heap (= creation) order. [C]

Counter handling: in a human's turn end every record owned by that human has byte 3 decremented **before** its type handler runs; computer persons' records the same way in the year-end pass; records owned by 0xC0 in the year-end world pass. A record whose type has no handler is deleted when the decremented counter reaches 0. [C]

### 8.1 Size table (`0x85980`, one byte per type) [C]

| type | size | type | size | type | size | type | size | type | size | type | size |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 13 | 10 | 6 | 20 | 4 | 30 | 11 | 40 | 11 | 50 | 5 |
| 1 | 5 | 11 | 6 | 21 | 4 | 31 | 6 | 41 | 4 | 51 | 9 |
| 2 | 8 | 12 | 7 | 22 | 5 | 32 | 6 | 42 | 6 | 52 | 9 |
| 3 | 4 | 13 | 6 | 23 | 5 | 33 | 9 | 43 | 6 | 53 | 9 |
| 4 | 75 | 14 | 4 | 24 | 6 | 34 | 13 | 44 | 11 | 54 | 9 |
| 5 | 6 | 15 | 6 | 25 | 7 | 35 | 4 | 45 | 9 | 55 | 5 |
| 6 | 15 | 16 | 4 | 26 | 7 | 36 | 11 | 46 | 12 | 56 | 10 |
| 7 | 9 | 17 | 4 | 27 | 9 | 37 | 10 | 47 | 16 | 57 | 9 |
| 8 | 7 | 18 | 6 | 28 | 6 | 38 | 11 | 48 | 9 | | |
| 9 | 12 | 19 | 8 | 29 | 4 | 39 | 9 | 49 | 5 | | |

### 8.2 Field layouts

Bytes 0..3 are the header above; the table gives byte 1, byte 2, byte 3 and the body (offset: meaning). u16/u32 are little-endian, unaligned. "prop" = property index (0..191). Where the body is "-", the record has no body. Details in the doc named in the last column.

| type | meaning | byte 1 | byte 2 | byte 3 | body | conf. | doc |
|---|---|---|---|---|---|---|---|
| 0 | mission | human | - | 100 (reset each turn) | +4 mission 0..29, +5 u32 progress, +9 u32 secondary sum | C | 10 |
| 1 | emperor / chronicle | 0xC0 | - | 1 | +4 emperor number (+1 in 1612, 1619, 1637, 1657) | C | 06 §7 |
| 2 | city disaster aftermath | 0xC0 | city | bit 7 = created this year; years left | +4 disaster type 0..3, +5 flag (1 unless plague), +6 u16 year | C | 06 §2 |
| 3 | auction announced | 0xC0 | - | years until held | - | C | 08 |
| 4 | newspaper contest | 0xC0 | - | 2 | +4 city, +5 judge, +6..+10 good per discipline, +0x0B..+0x28 30 entrant persons, +0x29..+0x46 30 discipline masks, +0x47 u32 fee | C | 06 §6 |
| 5 | office application | applicant | - | 2 | +4 place (location 0..13), +5 office index within level | C | 03 |
| 6 | spy contract | client | - | years (20) | +4 target, +5 u32 yearly cost, +9 active, +10 notified, +0x0B u32 evidence bits | C | 04, 04b |
| 7 | saboteur contract | client | - | years (5) | +4 target, +5 u32 cost per turn | L | 04 |
| 8 | production run | owner | prop (branch) | 1 | +4 good, +5 workers, +6 means | C | 02 §3 |
| 9 | shipment | owner | prop (source branch) | 1 | +4 kind (2..5 sell/deliver, odd = escorted), +5 u16 quantity, +7 good, +8 transporter count, +9 transporter type, +10 destination location, +11 condition | C | 02 §4 |
| 10 | transporters being built | owner | prop | build time | +4 transporter type, +5 count | L | 02 |
| 11 | transporters in repair / bought | owner | prop | 1 | +4 transporter type, +5 count | L | 02 |
| 12 | auction lot: transporters | seller | prop | - | +4 transporter type, +5 count, +6 condition | C | 08 |
| 13 | residence construction | owner | prop | build time (bit 7 = done) | +4 u16 new residence word | L | 02 |
| 14 | auction lot: residence | seller | prop (branch) | - | - | C | 08 |
| 15 | recruiting money | owner | prop (castle/camp) | 1 | +4 u16 money | C | 07 |
| 16 | auction lot: castle | seller | prop (castle) | - | - | C | 07, 08 |
| 17 | purchase interest in a castle | interested person | prop (castle) | - | - | C | 07 |
| 18 | trial | plaintiff | - | 1 (human) / rand(4) (computer) | +4 defendant, +5 kind (0 trial, 1 inquisition) | C | 04b §3 |
| 19 | yearly fine | convict | - | 3 | +4 u32 amount per year | C | 04b |
| 20 | debtors' tower (Schuldturm) | prisoner | - | turns to sit out | - | C | 04b |
| 21 | dungeon (Kerker) | prisoner | - | turns to sit out | - | C | 04b |
| 22 | insult / duel pending | insulter | - | - | +4 insulted person | C | 08 |
| 23 | county blessing (+10 yield) | holder | - | 5 | +4 county | C | 04b, 02 |
| 24 | conversion of a location | holder | - | 5 | +4 location, +5 religion | C | 04b |
| 25 | siege (city blocked, -60 yield) | besieger | - | 4 | +4 city, +5 u16 army size | C | 04b, 07 |
| 26 | relief army | sender | - | 4 | +4 city, +5 u16 army size | L | 07, 09 |
| 27 | yearly tribute from a taken city | receiver | - | 3 + random (set at siege success) | +4 city, +5 u32 amount | L | 07 |
| 28 | feast planned | host | prop (branch) | years ahead + 1 | +4 musicians 0..4, +5 size 0..3 | C | 08 |
| 29 | mistress / lover | keeper | - | 2 + rand(5) | - | C | 05 §5 |
| 30 | courtship | suitor | - | gift years left | +4 u32 gift total, +8 required factor k, +9 courted person, +10 wedding city | C | 05 §2 |
| 31 | child (pending or born) | parent | - | counter (45 = birth, see 05) | +4 bit 0 sex, bit 7 main heir; +5 human-name slot | C | 05 §4 |
| 32..49 | sub-plots | human | see 06 | script step | per type, all listed in `rules-06` §5.2 | C/L | 06 §5 |
| 35 | unused (no creator or handler) | | | | | C | |
| 50 | AI revenge | computer actor | - | years | +4 human victim | C | 09 |
| 51 | castle/camp attack order | owner | prop (home castle/camp) | 1 | +4 target location - 14, +5..+8 troops per troop type | C | 07 |
| 52 | troop transfer | owner | prop (home) | 1 | +4 destination location - 14, +5..+8 troops (added to the destination at the next turn end) | L | 07 |
| 53 | patrol order (castle) | owner | prop | 1 | +4 county index 0..2, +5..+8 troops | C | 07 |
| 54 | raid order (camp) | owner | prop | 1 | +4 county index 0..2, +5..+8 troops | C | 07 |
| 55 | prisoners held | person | prop (home) | 1 | +4 prisoner count | L | 07 |
| 56 | loan | borrower | - | years left | +4 lender, +5 u32 amount, +9 interest rate % | C | 02 |
| 57 | history entry | person | - | 25 | +4 code 0..9, +5 u16 year, +7 argument 1, +8 argument 2 | C | 06 §7 |

Order slots map to records at turn end: castle/camp order types 0x11 / 0x21 → 51, 0x12 / 0x22 → 52, 0x13 → 53, 0x23 → 54; the five slot bytes 2..6 are copied to record +4..+8. [C]

Sub-plot record sizes (types 32..49) match the layouts in `rules-06` §5.2.

## 9. Function map by subsystem

Main flow
- `0x4AB90` game_main: menu, new game / load, main year loop (tail continues at `0x4B2D4`). [C]
- `0x4A490` world_init: year 1600, NPCs, offices, castles, camps, starting branches, prices. [C]
- `0x4DBC0` setup_human_player (also "Neuer Spieler" mid-game). [C]
- `0x4B900` turn_start_reports: resolves last year's production and shipments, storage offers, credit offers. [C]
- `0x5D610` kontor (phases 1..4 of a human turn). [L]
- `0x61A70` turn_end_settlement: prices, titles, order execution and costs, tips, sub-plots, per-turn record processing. [C]
- `0x66220` year_end_world: computer persons, deaths, offices, auctions, trials, sieges, misfortunes, chronicle. [L]

Economy
- `0x49EF0` prices_init, `0x49FE0` prices_update. [C]
- `0x439F0` branch_add_goods (storage limit, giving away). [C]
- `0x43930` branch_add_transporters. [C]
- `0x43CC0` branch_effective_skill. [C]
- `0x4FA70` branch screen and order book; `0x50DD3` order cost forecast. [L]
- `0x531A0` storage purchase; `0x53680` / `0x53990` director hire/fire; `0x53A60` transporter menus. [G]
- `0x44640` / `0x441F0` net worth (cash + stock value - loans). [C]
- `0x43790` add cash, `0x43860` pay cash. [C]
- `0x5E1D0` / `0x5E500` money lender and office income. [G]
- `0x64108` debt seizure, `0x6449F` / `0x6540A` seizure lists. [G]

Career and politics
- `0x47F90` office applications, `0x47C20` appointment vote, `0x49700` title award (Urkunde). [G]
- `0x45C30` office tree / favour bars. [G]
- `0x5E600` privilege/ability check (record lookup by person pair and code). [L]
- `0x48690`..`0x49010` law book texts (Finanz-, Straf-, Kirchengesetze). [C]
- `0x289E0` trial, `0x2BD05` verdicts, `0x2C0A0` yearly fines. [L]
- `0x59720` indulgence, `0x5CCA0` / `0x5D1F0` church. [L]

Family
- `0x20D30` matchmaker, `0x212B0` courtship gifts, `0x221F0` births, `0x45A70` testament, `0x654A9` death and inheritance, `0x20A00` / `0x20B90` mistress. [L]

Events
- `0x17930` history and reminders, `0x18AE0` chronicle, `0x18BD0` disaster damage, `0x26640` siege effects on prices, `0x59940` miracle/divine event (not misfortunes; see rules-06 §3.1), `0x59F80` / `0x5B200` computer-person actions, `0x226F0` sub-plot dispatcher, `0x19480`..`0x1ED90` individual sub-plots. [L]

Castles, bandits, combat
- `0x54570`..`0x56E20` castle/camp screens and orders, `0x56760` recruiting, `0x56950` manoeuvre, `0x56CF0` toll rate. [L]
- `0x3EE60` battle_main (tactical battle), `0x3E2A0` loads `SZENARIO.DAT` (battle map), `0x37100` battle tactics menu. [L]
- `0x5F080` privileges menu (not siege planning), `0x1F2A0` siege resolution, `0x1FA80` relief army. [L]

Auctions, duels, feasts
- `0x26A50` auction, `0x27680` auction settlement, `0x269D0` announcement. [L]
- `0x2C750` duel. [L]
- `0x20160` feast planning, `0x20530` feast. [L]

Random numbers
- `0x6B491` rand, `0x6B4B5` srand, `0x36FF0` random_n, `0x37030` seed from clock. [C]

## 10. Castle and camp table (`0x8647C`, 7 records x 26 bytes) [C for bytes, L for meanings]

Order: records 0..2 = toll castles (location 14, 15, 16), 3..6 = robber camps (location 17..20); code indexes it with `location - 14` (castles) and `county - 10` for tolls (see the pairing caveat in section 3).

Initial bytes (hex):
```
0 dd 07 8c 01 ac 00 ee 01 9b 00 1b 02 8f 00 e8 03 00 00 ba 04 05 00 00 00 00 01
1 de 07 5b 01 04 01 37 01 0a 01 5d 01 f2 00 e8 03 00 00 14 05 05 00 00 00 00 01
2 df 07 2d 01 94 00 17 01 93 00 1d 01 8b 00 e8 03 00 00 d8 04 05 00 00 00 00 01
3 e0 07 df 00 6d 00 fb 00 7d 00 ea 00 62 00 e8 03 00 00 00 00 00 01 00 00 00 00
4 e1 07 90 00 ea 00 67 00 fa 00 87 00 e3 00 e8 03 00 00 00 00 00 01 00 00 00 00
5 e2 07 78 01 20 01 93 01 32 01 c1 01 33 01 e8 03 00 00 00 00 00 01 00 00 00 00
6 e3 07 bf 01 05 01 04 02 f6 00 f9 01 e3 00 e8 03 00 00 00 00 00 01 00 00 00 00
```
| offset | size | meaning | conf. |
|---|---|---|---|
| 0x00 | u16 | 2013..2019, text/name id (same scheme as city +0x00) | G |
| 0x02 | 6 x u16 | three (x, y) pairs, plausibly map coordinates of name, picture and flag | G |
| 0x0E | u16 | 1000 | G |
| 0x10 | u16 | 0 | G |
| 0x12 | u16 | castles 1210, 1300, 1240; camps 0 | G |
| 0x14 | u8 | toll rate in percent (castles 5, camps 0); set by the "Zollsatz" screen (0x56CF0) | C |
| 0x15 | u32 | toll income accumulator, paid out and reset at the owner's settlement (economy doc 4.2, 7.12) | C |
| 0x19 | u8 | castles 1, camps 0 (camps have 1 at +0x15 instead, i.e. the accumulator's low byte) | G |

**Not saved** [C]: the save blocks are cities `0x85F04 .. 0x8647B` (0x578 bytes, ends exactly at `0x8647C`) and the goods table from `0x86532`. The 0xB6 bytes `0x8647C .. 0x86531` are not in any block. So after loading a game the castle table has the values left in memory: after a fresh program start that is the initial data above (toll rates back to 5 %, accumulators 0). This looks like an original bug (a player's toll rate and unpaid toll income are lost on reload). A port that wants parity on load should reset this table to the initial values when loading; a port that wants correctness should save it. [the consequence is derived, not observed]

## 11. World generation (`world_init` 0x4A490), in RNG call order [C, L where marked]

Runs once at program start right after `srand(clock)`, before the title menu. Humans are added afterwards by `setup_human_player`.

1. Year = 1600, phase 0, current person 0xC0. Create the fixed record(s) of 0x18A80 (no RNG). Free all 192 property slots, clear all 192 persons (no RNG).
2. `prices_init` (no RNG), then `prices_update` (2 RNG calls per city for storage levels A and B, 20 calls).
3. Persons 0..159, for i in order:
   - `rand(4) == 0` → female with name `rand(70)`, else male with name `rand(117)`.
   - `pick_name` fills slot i (all slots are free, so slot = i): `rand(10)` → +0xF6, `rand(15)` → life +0x02 = 25 + r, then 192 calls `rand(5)` → favour toward every person = 48 + r (own slot then set to 100). Cash 750, skills 100, pay levels 100, no spouse, title 0, no office (index 15), religion from `0x85A4B` (overwritten next).
   - religion = `rand(2)`.
   - offices by i:
     - i < 100: city `i / 10`, index `i % 10`, church flag = index > 6 (Priester, Priester, Domherr); title `1 + rand(2)`; +0x0E = city.
     - 100 <= i < 139: county `10 + (i-100) / 13`, index `(i-100) % 13`, church = index > 7; title `3 + rand(4)`; +0x0E = county + 64 * `rand(2)`.
     - 139 <= i < 150: principality 13, index `i - 139`, church = i > 146; title `6 + rand(4)`; +0x0E = 13 + 64 * `rand(4)`.
     - 150 <= i < 160: no office; title `1 + rand(2)`.
     So every one of the 150 offices starts filled, in index order, and persons 150..159 are office-less. (+0x0E is a place byte with 2 extra bits on top; meaning of the top bits [G].)
   - career levels: +0x12 = `0x868E9[o]`, +0x13 = `0x8690B[o]` for the person's global office index o (0 if no office), see career doc 3.1.
   - skills for g = 0..14: `skill[g] = (o + 2g + 53 + rand(34)) mod 256` [L: byte arithmetic read from decompiler].
4. 16 rounds, each creating three computer branches (48 total, duplicates in the same city are possible):
   - city `rand(10)`; owner = random computer person whose place byte (+0x0F) is that city (`rand(count)`); new branch; Karren `1 + rand(5)` at condition 100.
   - city `rand(10)`; owner = random computer person of that city's county; same transporter roll.
   - city `rand(10)`; owner = random computer person at place 13 (principality); same transporter roll.
   Persons without office have place 0 and are in the Fürstenbrück pool. If no candidate exists the branch is created with owner 0xFF [L].
5. Robber camps k = 0..3 (location 17 + k): owner = random computer person at place `10 + rand(4)` (any county or the principality); for troop slot 0..3: count `2 + rand(4)`, then training `30 + rand(40)`. Camouflage 60, straw 20 (property_create).
6. Toll castles k = 0..2 (location 14 + k): owner as for camps; troops `5 + rand(5)`, training `30 + rand(40)`. Security 60, casemates 40.
7. Office wealth targets `0x85E7A[0..33]` = `0x86BD0[0..33]`.
8. For every person slot 0..191 (empty slots too): `person_init_wealth(p, 100, 10 000 000)` (cash := target for its office), then cache net worth in +0x1C. No RNG.
9. Globals: `0x85E06` = 5, `F1` (`0x85E1D`) = 100, city treasuries `0x85E25[c]` = population / 5 and `0x85E4D[c]` = population / 10, `0x85E75` = 5000. No RNG.

Option flags [C]: dword at `0x86AD1` (CONFIG.DAT offset 0x17D; saved with the config, not with the game). One bit per line of the options screen (0x4EFC0, labels in this order on screen): 0x001 Geschichtsschreibung (chronicle, 0x18AE0), 0x002 Abenteuer (sub-plots), 0x004 Das Schicksal (fate events 0x226F0 at a human's turn end, world fate at year end), 0x008 Spieltipps, 0x010 Warnungen, 0x020 Runde bestätigen, 0x040 Zeugen & Plädoyers (trial), 0x200 Gericht bestechen (trial investments), 0x100 Spieler ankündigen ("Nächster Spieler" screen, 0x4B900), 0x400 Spielende um 1800. 0x080 is a temporary internal flag set by the main menu during the intro. (Correction: an earlier version of this paragraph called 0x004 "sub-plots"; 0x004 is Schicksal, and the function it gates in the turn-end settlement, 0x226F0, is the fate-event roller.) "Spielende um 1800" has exactly one effect in the code: new children can be conceived only while year < 1670, or year < 1770 if the option is set (0x220C0). There is no year-1800 check anywhere; the hard stop stays at 2000 (`rules-01`).
