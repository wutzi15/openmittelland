# rules-70: new-game seeding, setup draws, disaster and donation gates

Tags: [C] instruction level, [L] decompiler only, [G] guess. `rand(n)` = `random_n` 0x36FF0 (`rules-01` §1). Addresses in `game_main` 0x4AB90 were read with a raw disassembly of 0x4AB90..0x4B3DB because Ghidra has no function there (which is why `rules-69` §7.1 missed the start-up seed; corrected there in §7.5).

## 1. Program start to the first turn, in order [C]

| step | code | RNG |
|---|---|---|
| 1 | C runtime (entry 0x6B098), `game_main` 0x4AB90: video/mouse init (0x2F970, 0x131C4, 0x2F470, 0x2F400, 0x12DFE), option bit 0x80 set, music (0x41E50) | music draws only for ids 0x19..0x1E (`rules-14`); whatever is drawn here is irrelevant, the seed below overwrites S |
| 2 | intro film: FLC open 0x14D10, frame loop 0x14ED6 / speech 0x154B0 / 0x154F0 / 0x15530, key poll 0x12C71 until a key or the film ends, 0x15480 | no `rand` in 0x14D10, 0x14ED6, 0x15045, 0x12C71 (none of them is a caller of 0x36FF0) |
| 3 | options restored, video re-init, 0x2F4B0 | none |
| 4 | **`randomize_from_time` 0x37030 at 0x4ADEC**: INT 21h AH=2Ch, `srand(DX)` with DX = seconds·256 + hundredths (0..15203, 6000 distinct values) | **S := clock** |
| 5 | `world_init` 0x4A490 at 0x4ADF1 (`rules-00` §11 for the draw order) | world draws |
| 6 | title screen: `music(4, 1)` (no draw for id 4), picture, "^ Mittelland AD _", slot scan, menu box (`formats.md` §11.1): new game / load / continue; waits in 0x36850 | none: 0x36850, 0x368E0, 0x322C0, 0x12DE4, 0x2FA80, 0x2E310 are not callers of 0x36FF0 |
| 7 | new game: player count box, difficulty box (0x368E0), `0x85CBE := difficulty` | none |
| 8 | per player i = 0..n−1: `setup_human_player(i)` 0x4DBC0 | §2 |
| 9 | load / continue instead: `0x42200(slot)`; S is **not** in the save (`formats.md` §3.2) | none |
| 10 | both paths: first human := `0x433E0(0xFF,0xFF,0xFF,1)`; if year < 2000: **autosave slot 5 (0x42360 at 0x4B14A)**, record compaction 0x421A0, then the turn loop | none |

So: the seed is the DOS clock at the moment the intro ends (key press or last frame). Everything after it is a deterministic function of (seed, player count, the "Nein" restarts in setup, later inputs). No keyboard or mouse poll, timer, or idle loop between step 4 and the first turn draws from or writes to the generator. The only clock entry points into S in the whole image are 0x4ADEC (start-up) and 0x3EEA7 (`battle_main`, every tactical battle); `srand` 0x6B4B5 has no other caller and nothing else writes 0xB9284 (byte scan of the image for `E8/E9` targets and for the address constant).

Consequence for the harness: a game restarted from an autosave gets a fresh clock seed, so a replayed year is a fresh draw, not a replay. And step 10 writes GAME5.DAT immediately after "Das letzte Spiel fortsetzen", before any turn: a harness that copies the autosave on every mtime change collects a second copy of the same year after every restart (see `rules-69` §7.5).

## 2. Setup draws and inputs (`setup_human_player` 0x4DBC0, `person_init` 0x42CF0) [C]

Order per player:
1. Year > 1670 (0x686): refusal text 0xBDD54, no draws, return.
2. **Start city** (0x4DC76..0x4DCC1): `idx = rand(4)`; then loop: if `idx == 8`: `idx += rand(2)`; `city = T[idx]`, `idx += 1`; if `idx >= 12` → "Es gibt keinen Startort mehr" (0xBDDC4), return; if `rights[city]` (0x85E08) != 0xFF (city already taken by a human) → loop. Table `T` = 0x870C2 = `3 5 8 9 3 5 8 9 2 6 2 6` (Lichterheide, Bergstedt, Lauenburg, Kronsbach twice, then Moortal, Falkenhorst twice). So player 1 gets one of the four small 2 500-inhabitant cities uniformly; later players the next free one in table order from their own random offset; the `rand(2)` fires only when the scan reaches entry 8, i.e. when all four small cities are taken (5th and 6th player). Draw count: 1, plus 1 from the 5th player on. The player does **not** choose the city.
3. Dialogs: name, gender, religion, colour, hotkey (`formats.md` §8.2), "zufrieden?" — no draws. Message boxes 0x368E0 — no draws.
4. `person_init(slot, controller 1)` 0x42CF0: `rand(10)` → +0xF6; `rand(15)` → life +0x02 = 25 + r; then **for every person slot j = 0..191: `rand(5)` → favour byte +0x20+j = 48 + r** (192 draws). Total 194 draws, independent of all inputs.
5. Religion +0x03, hotkey +0xF5, colour +0xF4 written from the dialog (no draws); `property_create(0, city, p)` (no draws); uncle's order slot; cash top-up only when year > 1600; +0x1C net worth; `0x85A2C := (n + 9) / n`; mission 0x17690 (no `rand` in it or its callees 0x17xxx on this path; the mission helpers 0x173D0/0x17430/0x174C0 that do draw are called later from the turn code).
6. "Nein" at "Seid Ihr zufrieden?": `person_clear`, branch freed, setup repeats from step 2 — the draws of the rejected attempt stay consumed.

Inputs that reach the RNG or the world: **none of name, gender, religion, colour, hotkey**. Difficulty is read only by the mission code (`rules-09` §1). The player count changes the draw count (194 per player, 195 from the 5th on) and the cities. Religion affects later game logic (church), not the generator.

Determinism statement: two new games with the same seed (same DX at intro end), the same player count and the same "Nein" count have identical worlds, persons, prices and start positions, whatever is typed.

## 3. Year-end disaster gate (cross-check) [C]

- `year_end_world` 0x66220 at 0x68862: `rand(100)`; `CMP AX, 0x0C; JNC skip` → `disaster_damage` 0x18BD0 runs when the draw is **< 12**. One draw per year end, always made.
- 0x18BD0: `c = rand(10)` (0x18BE3, always drawn); conditions `population(+0x0E) > 1499` (`CMP 0x5DB, pop` unsigned, i.e. pop ≥ 1500) **and** bit 7 of city byte +0x6E clear; if either fails, nothing else happens (no retry, no further draw). Then `d = rand(4)` repeated until `(1 << d) & byte(+0x6E)` is non-zero (0x18C59, loop); the test uses the whole byte with the shift count masked to 5 bits, but with d ∈ 0..3 only bits 0..3 matter, so "mask & 0x0F" is equivalent. Start masks 5, 7, 13 (`rules-02` §1.3). The rest (record, population loss `rand(T.pop)`, `wealth_down(c, rand(T[d].wealth) · (3 + rand(7)))` at 0x18E4C/0x18E5E, stock and storage losses) is as in `rules-06` §2.

Per city and year, the disaster probability is therefore `0.12 · 0.1 = 1.2 %` while the city is ≥ 1500 inhabitants and not in a recent-disaster state; a 2 500-city that has lost population below 1500 is immune.

## 4. Computer "Bauwerk stiften" gate (cross-check) [C for the gate, L for the draw list]

`npc_act(p)` 0x59F80, Step B (`rules-09` §3): `tries = 0; while (tries < 10 && !done) { c = rand(24); switch (c) … }`. Case 2 (0x59F80 decompile, `case 2`): `rand(100) >= 20` (`0x13 < r`) **and** `privilege(p, p, 9, 0)` = "p holds any office" (`rules-04b` §4.1) → `b = rand(5)`, `city = rand(10)`, text 0xB69B4 if a human owns rights in that city, `rand(2000)`, `pay_cash(p, 10000 − b·that)`, **`wealth_up(city, 10 − 2b)`** (0x43F30, capped at 99), `done`. Case 12 does not set `done`; cases 17..23 do not either.

Probability that one `npc_act(p)` call ends in a donation, for an office holder without the law privileges 56/59/82 (Step A then never acts): per try `q₂ = 0.8/24 = 1/30`; the loop ends at the first successful try, so `P(donation) = q₂ / Q · (1 − (1 − Q)^10)` with `Q` = total success probability per try. For a typical city office holder (loc ≤ 9, g not in {3, 5, 13..22}): successes are case 1 (0.7), case 2 (0.8), cases 7..11, 13, 14 (always, 7 cases), case 15 (0.7 while fencing < 80), case 16 (0 since loc ≤ 9): `Q = 9.2/24`, `P ≈ 0.8/9.2 ≈ 8.7 %` per call (the `(1 − Q)^10` term is 0.5 %). Holders of special offices add their case-3..6 successes to `Q` and get less; a person without any office gets 0.

How often `npc_act` is called per person and year (`rules-09` §2): once per human office holder h with `rand(100) < 40` for a random computer person at h's office place; once per county with 50 % for a random person of the county; once for the principality with 90 %; once per human spy contract with 50 % for the target. A given computer person's yearly call probability is the sum over these of (gate · 1/number of candidates in that pool). Example, a city with one human office holder and k computer persons: `0.4/k` calls per year, so about `0.035/k` donations per year per person, or 3.5 % per year for the city as a whole, each worth `10 − 2b` (b uniform 0..4, mean +2) spread over a `rand(10)` city, not necessarily this one.

## 5. Corrections to other documents

- `rules-69` §7.1 said there is no clock seeding at program start. Wrong: 0x4ADEC. §7.5 added there.
- `rules-01` §1 item 1 is right ("once at program start"); make it precise: after the intro, immediately before `world_init`, so world generation is seeded by the time the intro ends.
- `rules-30` §3 / `formats.md` §8.2: the start city is assigned by the table rule in §2 step 2, not chosen.
