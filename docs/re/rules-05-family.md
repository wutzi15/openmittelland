# Family: courtship, wedding, children, mistress, life, death, inheritance

Confidence tags and field offsets as in `rules-00-map.md`. `favour(a→b)` = byte at person a, offset `0x20 + b`. `rand(n)` = `random_n(n)` (`rules-01`). `standing(p, k)` = the helper at 0x17430:

```
standing(p, k) = (clamp(net_worth(p), 100, 999999) * k / 1000) * (rand(60) + 70) / 100 + 1
```
(integer division at each step, net worth freshly computed; one RNG call). It is used as a price everywhere in this doc. [C]

`favour_increase(a, b, x)` raises `favour(b→a)`, `favour_decrease(a, b, x)` lowers it (religion rules in `rules-03` §1).

## 1. Records used

| type | size | layout | owner byte 1 |
|---|---|---|---|
| 29 | 4 | mistress/lover; byte 3 = years left | keeper |
| 30 | 11 | courtship: +3 years left, +4 i32 gift total, +8 required-gift factor k, +9 courted person, +10 wedding city | human suitor |
| 31 | 6 | child (pending or born): +3 counter, +4 flags (bit 0 sex 0 male/1 female, bit 7 main heir), +5 human-name slot of the child | parent (human) |

Record counters: at the end of a human's turn (`turn_end_settlement`, 0x61A70) every record owned by that human has byte 3 decremented **before** its type handler runs; records of computer persons are decremented the same way in the year-end loop over computer persons (0x66220). A record whose type has no handler is deleted when the decremented counter is 0. Records allocated while that loop runs are appended to the heap and are visited (and decremented) in the same pass. [C]

## 2. Matchmaker (Kupplerin) — 0x20D30, reached from the church screen "Hochzeitsglocken" (0x5CCA0)

Entry: the church screen only calls the matchmaker if the human's spouse byte (+0x04) is 0xFF; a married human gets the text `[EXE text]` [C]

1. If the human already has a type-30 record: show "Ihr werbt bereits um …" with a yes/no; choosing the first answer deletes the courtship (gives it up). Nothing else. [L]
2. Proposal limit: global counter `0x85E07` is reset to 0 at every human's turn start (0x4B900) and incremented on every proposal shown. At 2 or more the matchmaker says nobody comes to mind. So at most **2 proposals per turn** (declined ones count). [C]
3. Candidate list, all persons j in slot order with:
   - slot used, gender differs from the human's;
   - j holds an office (+0x11 ≠ 15);
   - same tier: both `+0x0F < 10` (city level) or both `+0x0F ≥ 10` (county/principality). The human's own +0x0F decides the tier (a human without office has a city location here);
   - j's byte +0x10 = 0 (computer person, not clergy);
   - j unmarried (+0x04 = 0xFF). [C]
   Empty list: `[EXE text]`, no fee.
4. Pick `c = list[rand(len)]`. Wedding city W from c's office location L: L = 13 → 0 (Fürstenbrück); L in 10..12 → `(L-10)*3 + 1` (1, 4, 7, the first city of that county); else L. [C]
5. Display only: the average favour toward c of all persons whose location byte equals W, `clamp((avg - 25) / 10, 0, 4)`, selects an adjective ("geachtete" …); 2 if nobody. No rule effect. [L]
6. Fee = `standing(human, 40)`. Offer "Macht %d Taler". Accept: pay the fee (cash may go negative), create type 30 with counter `n = 4 + rand(3)`, gift total 0, `k = n - 2`, target c, wedding city W. Decline: nothing (the proposal still counted). [C]

## 3. Courtship years — 0x212B0 (type-30 handler, human's turn end)

Years with counter ≠ 0 after the decrement (so `n - 1` years: 3, 4 or 5):
- Three prices `P[i] = standing(c, 25)` for i = 0, 1, 2 (target's wealth, three independent draws), then three gift names (cosmetic, `rand(6)`, `rand(7)`, `rand(7)` into three name tables: Kerzenleuchter…Kupferstich, Leibeigene…Streckbank, Diamantring…Maßkrug).
- Menu: three gifts or `[EXE text]`. Gift i: one cosmetic `rand(15)` reaction text, pay `P[i]`, gift total += `P[i]`. Refusing costs nothing. One gift per year at most. [C]

Decision year (counter = 0), in this order [C]:
1. **Rival check.** Walk all type-30 records in heap order; stop at the first record r with `gifts(r) + 20*favour(c→owner(r)) > gifts(this) + 20*favour(c→suitor)`. If such r exists and its owner is not this suitor: `[EXE text]`; delete this record; stop.
   **Bug:** r's own target is not compared, so any other human's courtship of anybody can beat you; r's gifts are weighed with *your* target's favour toward r's owner. The winner is not married by this; it only knocks you out.
2. **Gift threshold.** If `gifts < k * net_worth(c) * 25 / 1000` (net worth not clamped; integer): `[EXE text]`; delete; stop. Each gift averages about 2.5 % of c's net worth and `k = (gift years) - 1`, so roughly one year may be skipped.
3. **Canon (Domherr) veto.** Canon D = holder of office index 9 at city W (person search location W, index 9). If there is none, skip to step 4. Otherwise:
   a. For each person i in slot order with `favour(D→i) > 100` (i holds blackmail on D, see `rules-03`/`rules-04`): show i the dialog `[EXE text]`. The dialog is shown whether or not i is human. First answer ("untersagen lassen") ends the loop with the flag *blackmail used*; second answer continues the loop.
   b. If D is human and the blackmail flag is not set: D is asked `[EXE text]` ("Ja!" forbids).
   c. Otherwise (D computer, or blackmail used): veto if and only if `favour(D→suitor) ≤ 50` and `c ≠ D` and the blackmail flag is not set.
   **Quirk [C]:** choosing "die Heirat untersagen lassen" in (a) guarantees the wedding goes ahead. The text and the effect disagree; reproduce the effect.
   Veto: `[EXE text]`; delete; stop.
4. **Vows.** Menu "Ja!" / "Äh, eigentlich nicht." / "Zögern." Zögern repeats the question. "Eigentlich nicht": for every person i with `favour(i→c) > 50`: `favour_decrease(suitor, i, rand(5))`; delete; stop.
5. **Wedding** ("Ja!"):
   - history entry code 1 (wedding) for the suitor;
   - titles (+0x0D): `t = title(c)`; if t is 4 or 7, c's title and t both drop by 1. If `title(suitor) < t` the suitor's title becomes t (`[EXE text]`); otherwise c's title becomes the suitor's title;
   - favour merge: for every slot i (0..191, used or not) `favour(i→suitor)` and `favour(i→c)` are both set to the larger of the two;
   - spouse bytes +0x04 set mutually;
   - wedding feast menu, prices for options o = 0..3 computed in this order: `F[o] = (3 - o) * standing(suitor, 20) + rand(100) + 2`. Options: 0 `[EXE text]`, 1 `[EXE text]`, 2 "Bedeutende Persönlichkeiten <W>", 3 "Lediglich die Familie", 4 "Niemanden". Options 0..3 pay `F[o]`; then for every slot i that qualifies: `favour_increase(suitor, i, rand(5))` then `favour_increase(c, i, rand(5))`. Qualifies: option 0 everyone; option 2 location byte (+0x0F) = W; option 3 nobody; option 1 location byte = `(W - 1) / 3` (C division, so 0 for W = 0..3, 1 for 4..6, 2 for 7..9). **Bug [C]:** option 1 should compare with the county id (10..12) and instead selects persons located in Fürstenbrück, Finsterwalde or Moortal;
   - pregnancy scheduling (§4) for the suitor;
   - delete every type-30 record whose target is c or whose target is the suitor (this one included).

## 4. Children — 0x220C0 (scheduling), 0x221F0 (type-31 handler)

**Scheduling** (called only at the wedding and after each birth, both for the human parent) [C]:
- `m` = number of type-31 records of the parent (pending and born, alive).
- Only if `m < 3` and (`year < 1670`, or `year < 1770` and option "Spielende um 1800" is on, §8).
- Counter `47 + rand(3m + 3) + rand(3)`; sex: with `rand(100) < 66` the parent's own sex, else the other; bit 7 (main heir) set if `m = 0`.

The spouse plays no role (alive or not). No births without a wedding.

**Yearly handler** (after the turn-end decrement, counter value v):
- `v = 45`: **birth.** Find the first free human-name slot (32 slots of 24 bytes, free = first byte 0). None free: delete the record, then still call scheduling. Otherwise store the slot at +5, decrement the counter once more (v becomes 44), ask the player for the child's name, write history entry code 4 with the sex, then call scheduling. So the first child arrives `2 + rand(3) + rand(3)` decrements after the wedding (the first decrement happens in the wedding turn itself); later children `2 + rand(3m+3) + rand(3)` after the previous birth.
- Otherwise, if the parent's life (+0x02) ≥ 5 and `v < 45` and (`v = 0` or (`45 - v < 10` and `rand(100) < 3`)): **child death** (`[EXE text]`, shown age `46 - v`). Free the name slot, delete the record. No rescheduling.
  - Years since birth y = `44 - v`. The 3 % roll applies for y = 1..8 (shown ages 3..10, two more than y: display quirk).
  - `v = 0` (y = 44) kills the child unconditionally. A parent with life < 5 skips the whole check, so the counter can wrap to 255 and the child lives on. [C]
- Born children stay type-31 records of the parent; they never become persons until inheritance (§7).

## 5. Mistress / lover — 0x20A00 (acquire), 0x20B90 (type-29 handler)

Acquire (0x20A00, from the privileges menu entry 4 for humans (0x5F080); at year end for 10 computer persons drawn with `person_pick_random(any, computer)`, repeats possible) [C]:
- Already has a type 29: human gets `[EXE text]`; nothing else.
- Otherwise create type 29 with counter `2 + rand(5)`. Human: message `[EXE text]` Computer person: if some human owns a property at the person's location (table `0x85E08`, rebuilt at year start by 0x43E60) and `rand(100) < 6`, a news line "… geht fremd" (`rand(5)` variant) is shown. No cost at acquisition.

Yearly handler, counter v after decrement [C]:
- `v = 0`: human gets `[EXE text]`; delete.
- else: compute `standing(keeper, 10)` and show it to a human as `[EXE text]`. **Bug [C]: the amount is never deducted.** Then for every type-29 record r (all keepers, self included) `favour_increase(keeper, owner(r), 10 + rand(10))`, i.e. every mistress keeper likes this keeper more by 10..19 (halved across religions).
- Being a keeper is an offence when the mistress law is in force (crime scan 0x447D0, bit 0x400000; see `rules-04b`).

## 6. Life ("Lebenskraft", person +0x02)

- Start: `25 + rand(15)` (25..39) for every new person including humans (0x42CF0). [C]
- Decrement: only for **humans**, once per own turn at the very end of turn-end settlement (after the debt check, after the mission/history check 0x17930): `life -= 1` if `year < 1620`, else `life -= rand(3)` (0, 1 or 2). Then if life as a **signed** byte is ≤ 0 the human dies (§7). Computer persons have no life countdown; they leave the game through the replacement rule (§9). [C]
- Other changes (specified in their own docs): Schicksal events (0x226F0, adds, cap 100), duel losses (0x2C750), computer misfortunes (0x59940, ±5, cap 100, floor 0).
- Doctor (privilege menu entry 1): shows one of 12 texts with index `11 - min(11, life / 2)`: life ≥ 22 "Ihr strotzt vor Gesundheit!" … life 0..1 "Macht Euer Testament!". Display only. [C]
- End-of-turn tip (option Spieltipps): when `life < 15`, unmarried and no type-31 record. [C]

## 7. Death and inheritance — tail of turn-end settlement at 0x654A9

Order at the end of a human's turn [C]:
1. No branch (property kind 0) left: bankruptcy screens ("Ihr seid bankrott"), purge (§7.3 mode 1), clear person; turn over. Life is not touched.
2. If the human has a type-0 (mission) record: mission/history check 0x17930 (`rules-10`).
3. Life decrement and death test (§6). Alive: done.
4. Death: candle film, epitaph "Hier ruht …". The epitaph line picks a random set bit of the person's deed flags (+0xF3) with up to 16 tries of `rand(8)` (bits: 0x01 mistress, 0x02 robber camp, 0x04 saboteur, 0x08 spy, 0x10 debtors' tower, 0x20 siege, 0x40 and 0x80 set by the crime scan [G]). Cosmetic.
5. Heirs: list of the human's type-31 records with counter < 45 (born children) in heap order; main heir index h = the list index of the first record with bit 7, or `len` if none ("Das Erzbistum"). [C]
6. No children: `[EXE text]`; purge mode 1; clear person. That player is out.
7. Otherwise a funeral sequence (cosmetic speeches, `rand(len)+1` mourners, random texts) and the testament is read. If h = len ("alles dem Erzbistum vermacht"): out as in 6.
8. Succession (h < len), the same person slot continues [C]:
   - old human-name slot freed; name slot := child's +5; gender := child's sex bit;
   - skills: each of the 15 production skills `(skill + 100) / 2`;
   - spouse := 0xFF (the widow's own spouse byte still points to this slot: quirk, it keeps that computer person "married" and out of the replacement rule and matchmaker);
   - life := the child's record counter (= 44 - years since birth, so a younger heir lives longer);
   - insurance flags := 0;
   - if there were 2 or more children: pay `net_worth / 5` (computed before the purge; the money goes nowhere, it is the compulsory share in the testament text `[EXE text]`);
   - purge mode 2 (below): all children records go, siblings and the heir's own;
   - kept: cash, title, offices, properties, favour of everyone toward this slot, religion, colour, hotkey.
   Message `[EXE text]`.
9. Phase := 6.

### 7.1 Testament (0x45A70)
Lists the main heir (or "Das Erzbistum") and, with 2 or more children, the fifth-part clause. Editable when called from the privileges menu (entry 6, `rules-04b`); each click moves the main heir `h := (len + h + 1 + d) mod (len + 1)` with d the click direction, clearing bit 7 on the old and setting it on the new record; index len = Erzbistum. At death it is shown read-only. [L]

### 7.2 Person clear (0x432D0)
All properties of the person pass to a random computer person at location 13 (principality office holders), or any computer person if none; free human-name slot; zero the record; byte 0 := 0xFF. The code first tests property kind against 0x5B, which never matches (dead branch meant for deleting branches). [C]

### 7.3 Record purge (0x42F60, mode m)
Records owned by the person: mode 1 all; mode 2 types 5, 6, 7, 18, 22, 23, 24, 28, 29, 30, 31, 50, 57. Records owned by others that reference the person (both modes): type 4 if the person is at +5 or in the 30 bytes from +0x0B; types 6, 7, 18, 22, 36, 43, 46, 50 if +4 = person; type 30 if +9 = person; type 45 if +5 = person. Mode 3 (used for dismissals) skips types 4 and 30. [C]

## 8. Game length and the "Spielende um 1800" option

Option word `0x86AD1` (default 0xFFFF, all on), bit per menu line (0x4EFC0): 0x001 Geschichtsschreibung, 0x002 Abenteuer, 0x004 Das Schicksal, 0x008 Spieltipps, 0x010 Warnungen, 0x020 Runde bestätigen, 0x040 Zeugen & Plädoyers, 0x200 Gericht bestechen, 0x100 Spieler ankündigen, 0x400 Spielende um 1800 (0x080 is a temporary internal flag). [C]

Bit 0x400 has exactly one reader: pregnancy scheduling (§4). There is no year-1800 end check anywhere in the program; the loop stops at 2000 (`rules-01`). The option ends a game around 1800 only indirectly: no births after 1769, so the last heirs die out. With the option off births stop after 1669. [C]
**Handbook disagreement:** the option name suggests a hard end; the code has none.

## 9. Computer-person turnover (year end, 0x66220, after dismissals) [C]

Runs every year, 1600 included. For each location L = 0..13 in order: probability `p = 15 + 10*(L < 13) + 10*(L < 10)` (cities 35 %, counties 25 %, principality 15 %); roll `rand(100) < p`, then pick `x = person_pick_random(L, computer)`. x is replaced only if: x owns no toll castle and no robber camp, is director of no branch, no human has x as spouse, x is not the target of any courtship, and x is not referenced at +4 by any type-44 record. Replacement: purge mode 1 of x, then a new identity in the same slot: `rand(4) = 0` female with name `rand(70)`, else male with name `rand(117)` (0x42CF0 re-initialises the record, life, favours 48..52, cash), title `1 + rand(2)`, wealth init (0x42EA0, args 100, 10 000 000). The office does NOT stay with the slot: the re-initialisation (0x42CF0, called with the same slot number) sets the place byte +0x0F to 0, the office index +0x11 to 15 (none) and the church flag to 0, so the office becomes vacant and is filled by the normal vacancy cycle later in the same year end (`rules-03` §4.6) [C, instruction-level: the slot is passed in ECX, controller 0 in EBX]. This is the only way computer persons "die". [C]

## Evidence
0x17430 standing; 0x5CCA0 church; 0x20D30 matchmaker; 0x212B0 courtship/wedding; 0x220C0 / 0x221F0 children; 0x20A00 / 0x20B90 mistress; 0x42CF0 person init; 0x654A9..0x6613E death and inheritance (inside turn-end settlement); 0x44130 heir list; 0x45A70 testament; 0x432D0 person clear; 0x42F60 purge; 0x4EFC0 options; 0x66220 turnover block; 0x5F080 privileges menu (doctor, mistress, testament entries).

## Handbook comparison
- H p.56 "assume age 20 in 1600": the game stores remaining years only, 25..39 at start, about one per year (0..2 after 1620).
- H p.56 partner proposal "depends on wealth, office, standing": only the office tier and the price (wealth) matter; candidates are random within the tier.
- H p.57 / R p.9 canon veto and bribe: the veto is a favour test (> 50 passes); there is no separate bribe in this code, raising the canon's favour (bribery, `rules-04`) is the lever. Blackmail of the canon works (inverted dialog, §3).
- H p.57 "standing with friends rises": favour merge plus wedding feast.
- R p.9 testament via privilege; "Erzbistum" when no heir is named: confirmed; the estate then passes to a random principality office holder, not to a church entity.
- H p.57 "disinherited relatives may oppose the heir for many years": not found in this code. [NEEDS SOURCE in code]
- Quack/Medicus visits (H p.56): a Schicksal event raises life (see `rules-06`).

## Open
- Answer index convention of the dialog helpers (0x36AC0 returns 0 for the first option) inferred from consistent use, not from reading the helper. [L]
