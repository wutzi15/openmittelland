# Open questions, round 3 (rules-40)

Answers to the SPEC-GAP comments left in `crates/mad-sim` after `rules-39` §10. Conventions as in `rules-39` (header): `rand(n)` = `0x36FF0`, `rec_new` = `0x424A0`, `rec_find` = `0x426B0`, `persons` = `0x433E0`, `person_pick_random` = `0x43560`, `pick_property` = `0x42B90`, `favour_increase`/`favour_decrease` = `0x43620`/`0x436F0`. Tags: **[C]** instruction level, **[L]** decompiler only, **[UNVERIFIED]** inferred. Everything is [C] unless marked. Each section ends with a verdict on the sim's guess. Not covered (answered in `rules-39` §10 or excluded): `rng.rs:30`, `heap.rs:46`, `save.rs:62`, `trials.rs:97`, `ai.rs:10`, `career.rs:353`, `career.rs:417`, `world.rs:3`, `world.rs:164`, `social.rs:174`.

## 1. `career.rs:97` Seat groups (Ratsherren / Geheimräte / Hofräte / Priester)

Vacancy filling sits in `year_end_world` `0x66220` (`0x66D9C..0x67FF0` region, decompiler lines for the place loop). Per place (13 down to 0) the index runs from the level's top index (city 9, county 12, principality 10) down to 0. For each index whose seat is empty (`0x433E0` returns 0xFF) the office is filled on its own [C]:

- Grouping is by index only, at every level: when the vacant index is 0, 1 or 2 the applicant scan covers indices 2, 1, 0 (the loop forces the start to 2 and keeps going while the index is 0 or 1); when the place is a city and the index is 7 or 8 the scan covers 8 and 7. So the three Geheimräte (county 0..2) and the three Hofräte (principality 0..2) are grouped exactly like the Ratsherren [C].
- The seat that gets filled is the vacant seat that triggered the pass (the saved index, restored before the election), not "the lowest free seat". After the election the loop continues with the next lower index, so a group with two empty seats runs **two** elections in the same year end, highest seat first. Each election repeats the whole procedure (electors, applicant scan over the whole group, and when nobody applied the three-draw `rand(count)` pool and the rival list), so the RNG is consumed once per vacant seat. The winner's own applications are deleted (`0x42550` over his type-5 records); other applicants' records stay and compete again for the next seat.
- When the no-applicant path stops because some human is eligible, the loop skips the rest of the group (the index is left at the group's first index − 1), so no further pool draws happen for that group this year.

Verdict: guess correct for the membership (indices 0..2 at every level, city 7..8). Guess wrong on seat placement: fill each vacant seat separately, highest index first, one full election (with its own RNG draws) per vacant seat; the winner sits in that seat.

## 2. `career.rs:259` Computer office holder moving between offices

`office_eligibility` `0x47C20`, rule 3, for a computer person (`+0x10 & 0xF0 = 0`) holding an office (`+0x11 ≠ 15`), with `l` = his place `+0x0F`, `idx` = his index `+0x11` [C]:
- `l < 10` (city): allowed only if the place equals `l`, or his index is 6 (Bürgermeister) or 9 (Domherr) and the place equals the county of `l` (`0x868D4[l]`).
- `10 ≤ l < 13` (county): allowed only if the place equals `l`, or his index is 6, 7 or 12 (Kanzler, Vogt, Bischof) and the place is 13. City offices (place < 10) are refused, including cities of his own county.
- `l = 13`: no restriction.

Verdict: guess wrong: a county office holder may only take offices of the county itself (place = `l`), plus the principality for Kanzler/Vogt/Bischof; drop the "cities of the county" alternative.

## 3. `career.rs:333` human_rival_of preference

`0x44F10(h)` [C]:
1. Location `l`: if h holds an office, h's `+0x0F`. Otherwise `0x42B90(kind 0, any place, owner h)` = a **uniformly random** branch of h (collects h's branches in slot order, one `rand(n)`, also when n = 1); `l` = that branch's place. Only if h has no branch: `l = rand(10)`.
2. One scan over persons 0..191 that exist and are computer. Two running minima, both starting at **102**: persons whose `+0x0F = l`, and all others. Compared value is `favour(x→h)` (person x's byte `+0x20 + h`), strict `<`, so the first (lowest index) minimum wins and a person with favour ≥ 102 is never chosen.
3. Result: the minimum at `l` if any, else the minimum elsewhere. If neither exists the return value is an uninitialised local [C, quirk; unreachable in practice].

RNG: one `rand(n_branches)` for an office-less human with branches, one `rand(10)` for one with neither; none for office holders. Every call draws again, also the per-human calls from text checks (`%F`).

Verdict: guess correct for the preference (lowest favour at `l`, else lowest elsewhere, first minimum). Guess wrong on the location of an office-less human: use a random branch (`rand(count)` over h's branches in slot order), not the first branch. Also exclude persons whose favour toward h is ≥ 102.

## 4. `career.rs:399` Yearly skill gain of computer persons

At `0x668D6..0x668FE` in `0x66220` [C]: `x = 2 + rand(5)`, then `g = rand(15)`, then `skill_up(p, g, x)` (`0x43BE0`: skill byte `+0xE0 + g` += x, capped at 150). Then `person_init_wealth(p, 20, 7000)` (`0x42EA0`).

RNG order: `rand(5)`, `rand(15)` (as in the sim).

Verdict: guess wrong: the rand(5) is not unused; apply skill g += 2 + rand(5), capped at 150. (Closes rules-03 open question 1.)

## 5. `career.rs:457` Index stored in a grouped seat's application

- Human applications (`office_apply` `0x47F90`) [C]: the Schreibstube list scans place 13..0, index high to low; the first seat of a group that has eligibility 2 (which needs it to be vacant) is listed and the rest of the group is skipped. So the listed index is the **highest** vacant seat of the group the human is eligible for. "Ja" creates a type-5 record with `+4` = place, `+5` = that listed index. The "already applied" lookup and the rival list in the Schreibstube match `+5` exactly against the listed index.
- Computer applications created in the no-applicant path (`0x66220`, `rec_new(5, …)` then `+4` = place, `+5` = saved index) store the vacant seat being filled.
- The election (career.rs:97 above) accepts any index of the group, so the stored index has no effect on who may win.

Verdict: guess wrong (minor, no gameplay effect): store the seat that was listed / filled, i.e. the highest vacant eligible seat of the group for a human, the vacant seat itself for an auto-created record; the Schreibstube should list that seat (not the lowest free one) and match existing records by exact index.

## 6. `career.rs:597` Blackmailer replacing two electors

Elector build in `0x66220` [C]: for each of the three appointer slots, holder a (if any); for every existing person x (index order) with `favour(a→x) > 100`, x is appended to the elector list (tagged with a). If no such x, a is appended. Nothing deduplicates the list. The vote loop runs once per list entry, each with its own scoring (and `rand(20)` for human candidates), its own `rand` + `favour_increase(e, winner, …)` and `rand` + `favour_decrease` per loser. A human blackmailer gets one voting screen per entry ("anstelle von %s").

Verdict: guess correct: a blackmailer holding two appointers votes twice (with all RNG draws twice).

## 7. `trials.rs:175` Computer blackmailer of a judge

`trial_main` `0x289E0` (vote loop around the dialog call with text `0xBB334` `[EXE text]`) [C]: for every existing person x with `favour(judge→x) > 100` the dialog `0x36AC0` is shown, with no human/computer test on x. Answer 0 counts "guilty", answer 1 "innocent", any other result (right click outside the items) counts for neither. So a computer blackmailer's choice is made by whoever is at the screen. No RNG in this step.

Verdict: guess wrong: there is no automatic answer; route every blackmailer, human or computer, through the same `BlackmailVerdict` question (the player at the screen decides). In practice blackmail markers come from human back-room actions, so computer blackmailers are rare.

## 8. `family.rs:9` Purge mode 3

`0x42F60(p, mode)` [C]. Only live records (type byte < 0x80) are looked at.
- Records owned by p (`+1 = p`): mode 1 deletes all; mode 2 deletes types 5, 6, 7, 18, 22, 23, 24, 28, 29, 30, 31, 50, 57; **mode 3 deletes none**. Records owned by p never enter the reference check below, in any mode.
- Records not owned by p, in every mode: delete type 6, 7, 18, 22, 36, 43, 46, 50 with `+4 = p`; type 45 with `+5 = p`; type 4 (contest) with `+5 = p` or p in `+0x0B..+0x28` and type 30 with `+9 = p` **only when mode ≠ 3**.
- The only mode-3 caller is the wealth dismissal in `0x66220` (`0x6684A`, computer persons only).

Verdict: guess wrong: mode 3 deletes none of p's own records; it only removes other persons' records that point at p (types 6, 7, 18, 22, 36, 43, 45, 46, 50), skipping types 4 and 30. Also, in all modes a record owned by p must not be deleted through the reference rule (the sim's `own || refs` can delete e.g. p's own type 6 with `+4 = p` in mode 2/3).

## 9. `family.rs:42` Heir draw in person_clear

`0x432D0(p)` [C]: iterates `0x42A50(kind any, place any, owner p)` / `0x42B10` = every property slot whose owner byte is p, in slot order, any kind. Per property:
- if its kind byte is 0x5B (`'['`): `0x429A0` frees it (kind 3, owner 0xFF, deletes the records keyed on it). Kinds are 0..3, so this branch is dead [L];
- else `x = person_pick_random(13, computer)` (`0x43560`, one `rand(count)` if any computer has place 13, none if empty); if that is 0xFF, `x = person_pick_random(any place, computer)` (one more `rand(count)`); if x ≠ 0xFF the owner byte becomes x.
Then the human name slot is cleared and the record zeroed (as specified).

RNG: one draw per property (two calls only when the place-13 pool is empty, but the empty call draws nothing).

Verdict: guess wrong: draw a new heir per property, in slot order, each with its own `rand(count)`. Also do not filter out kind 3 slots: any slot with owner byte p is reassigned (free slots normally have owner 0xFF, so this rarely matters).

## 10. `ai.rs:538` Purchase interest "random property at location"

`0x5B200` revenge c = 1, a owns neither camp nor castle, a's place ≥ 10 (`0x5B5F6..0x5B675`) [C]: `L = 14 + rand(7)`; `x = 0x42B90(kind any, place L, owner any)`: all property slots whose place byte is L, any kind and any owner, in slot order, `rand(n)` once (no draw if none; x = none → `rec_find(17, a, none-index)` is done on a garbage index, see below). If `rec_find(17, a, x)` finds nothing: `rec_new(17, a, x, 4)`.

RNG order: `rand(7)`, then `rand(n)` if n > 0.

Quirk [C]: when no slot has place L, `0x42B90` returns 0 and the code computes the property index from it (`(0 − 0xDC86C) / 0x60`, byte-truncated), so a type-17 record for a junk index can be created [L for reachability; places 14..20 always hold their castle/camp slot unless it was freed, and a freed slot keeps its place byte].

Verdict: guess correct for the draw (one uniform `rand(count)`, none if empty). Adjust the candidate set: every slot with place byte L, any kind and owner (not only castles/camps), slot order.

## 11. `ai.rs:548` Harassment branches

Already answered by `rules-09` §4b (`0x5B83E..0x5BCB8`, instruction level): row 2 (Einkommensniveau) and row 7 (Zwietracht) are unreachable, row 5 (Arbeiter verschwinden lassen) and row 6 (Bischof) are reachable. The current `harass` in ai.rs implements all eight rows in the documented order; the doc comment at ai.rs:548 is stale. Small difference: row 6 has no location test in the original (a Bischof's place is always a county), the sim's `(10..13).contains(&loc)` guard is harmless.

Verdict: guess correct (implementation matches rules-09 §4b; remove the stale SPEC-GAP comment).

## 12. `social.rs:263` Which address holds which church price

Turn start `0x4CB6D..0x4CB90` [C]: `0x85D3D = standing(p, 40)` first, then `0x85D41 = standing(p, 30)`. Kirchgang `0x5D1F0` [C]: "Kerzen stiften" shows text `0xC0130` `[EXE text]` with `0x85D41`; `[EXE text]` shows `0xC01A8` `[EXE text]` with `0x85D3D`.

Verdict: guess wrong: candles = `0x85D41` (`standing(p, 30)`), alms = `0x85D3D` (`standing(p, 40)`). RNG order at turn start: alms price first, then candles price.

## 13. `events.rs:63` Residence level index ≥ 10 (`0x44230`) [C]

`0x44230` indexes `RB` (`0x86698`, ten u32) with `r & 0x1F` and no bound check. Index 10..31 would read the next table, the build times `BT` at `0x866C0` (u32 1, 1, 1, 1, 2, 2, 2, 3, 4, 4, …) and beyond. No code path in the original produces a level ≥ 10: Kaufen offers roll `rand(10)` (`0x4B900`), Bauen offers k = 0..9 (`0x52790`), Erweitern (`0x52D00`) and Renovieren only touch bits 5..7 and the condition byte. Only a corrupt save could carry such a word.

Verdict: guess correct (unreachable in play). If byte-exact behaviour on corrupt saves matters, read RB[10..19] as the BT values instead of clamping.

## 14. `events.rs:297` Siege rumour destination quirk (`0x26640` case 4, arrival `0x4B900` at `0x4BB53`) [C]

World fate case 4: `c = rand(10)`, besieger `rand(12)`, print, then `rand(2)`. Non-zero: for every type-9 record in heap order whose destination (+10) = c, or whose source property (+2) lies in city c: +10 := +2 (property index), and kinds 2/3 become 4/5. Kinds 0/1/4/5 also get the new +10.

What the arrival code at the owner's next turn start does with that value [C]:
1. Blocked test: a 21-byte local flag array indexed by location, filled from siege records. It tests `flag[+10]` and `flag[city of source]`. For +10 ≥ 21 the read lands in other stack locals, so the result is arbitrary.
2. Blocked: transporters then goods go back to the source branch (`0x43930`, then `0x439F0`).
3. Not blocked, kind 4/5: `0x42A50(kind 0, location +10, owner)` searches for a branch of the owner at location +10. Branches only sit in cities 0..9, so the search succeeds only when +10 < 10 and the owner has a branch in that city. Found: goods then transporters go to that branch. Not found: goods (`0x439F0`) then transporters (`0x43930`) go back to the source branch.

So every +10 ≥ 10 ends in "return to the source branch", whichever way the garbage blocked test goes. For +10 < 10 the property number is used as a city literally. Both outcomes draw exactly one `rand(100)` (inside `0x439F0`). The only difference between the two return paths is the call order of goods and transporters, and that changes no state the sim observes.

The sim stores the source city when +2 ≥ 10. `branch_at(h, source_city)` then finds the source branch itself, which gives the same goods, transporters and RNG count. The blocked check uses the source city twice, which the original also checks.

Verdict: guess correct (outcome-equivalent). Caveat [UNVERIFIED]: I did not check whether any code between the year-end fate and the owner's next turn start reads +10 of a type-9 record (castle interception, mishaps). If something does, it sees the raw property number in the original.

## 15. `events.rs:321` "branch with director" (fate 15–18, 40, 74; `0x226F0`) [C]

Cases 15 (`0x22F96`), 16, 17, 18, 40 and 74 all call `pick_property(kind 0, any location, h)` (`0x42B90`), which collects all of h's branches in property order and draws one `rand(count)` (no draw if none). They then require the drawn branch's director byte +0x53 ≠ 0xFF; otherwise nothing happens. No re-draw, and no fallback to another branch. RNG for 15/16: the pick, then per good g = 0..14 one `rand(5)` / `rand(6)` for the skill change. For 17/18 the book `rand(49)` comes after the director check and before the 15 per-good draws.

Verdict: guess correct.

## 16. `events.rs:940` Fate 69 "that branch" (`0x24F4A`) [C]

Gate: year ≥ 1610 (`CMP year, 0x64A` / `JC`, i.e. **year ≥ 1610**, not > 1610). Then `rec_find(8, owner h, key 0xFF)` takes the **first type-8 record of h in heap order**. Its +2 is the branch. That branch's 4 order slots (+0x33..+0x52) are zeroed, and N = 5·certificate(owner of the branch, person +0x05) + 5. Then `rec_find(8, h, branch)` / `rec_next` walks every run of h in that branch, in heap order, with one `rand(N − 1)` per run.

Verdict: guess correct ("branch of the first run in heap order"). Separate finding: the year gate is `year ≥ 1610`. The sim has `self.year > 1610`, which is wrong for 1610. rules-06 §4 row 69 also says "> 1610" and should be corrected.

## 17. `subplots.rs:87` Type 33, price paid (`0x1A000`) [C]

The price is computed once (`Fy(500)` inline at `0x1A0D8..0x1A117`, one `rand(60)`), stored at local −0x24, printed, and that same local is passed to `person_pay` (`0x43860`) at `0x1A1E7`. RNG order of dialog 2: `rand(49)` book, `rand(60)` price, `rand(11)` grade, `rand(2)` place. After the yes answer: pay, then `rand(3)` for the counter (5 + rand(3)), then `rec_new`.

Verdict: guess correct.

## 18. `subplots.rs:122` Type 36, "office_index(x)" (`0x1ACC0`) [C]

`R = byte x.+0x11 + rand(3) + 2 + byte x.+0x0D` (`0x1AD1F..0x1AD5B`). Byte +0x11 is the raw office index within its level (15 = no office, as in rules-39 §2), and +0x0D is the title. It is not the global office index. The host x comes from `person_pick_random(10 + rand(4), 0xFF, 0xFF, computer)` (`0x43560`). RNG: `rand(4)`, the pick's `rand(count)`, `rand(3)`, `rand(6)` (Q = 5 − rand(6)·R/30), `rand(10)` place.

Verdict: guess wrong. Use `persons[x].idx()` (raw +0x11, 15 if no office) instead of `global_office(x)`. With the global index, county and principality hosts get R inflated by 10 or 23, which changes Q and the stored R.

## 19. `subplots.rs:616` Type 38, +6 holds the requested count (`0x1BC00`) [C]

Handler c = 1, +5 = 1: the spinner value r is stored in +6. The move loop runs i = 0..r−1: if `camp.troops[i mod 4] ≠ 0`, decrement it and increment record byte `+7 + (i mod 4)`. Empty types are skipped, so fewer than r may leave. Return, c = 0: the loop runs i = 0..(+6)−1. Each step draws `rand(100)`: `< 65` adds 1 to `camp.troops[i mod 4]` (clamp to 150 after each step), else counts as a loss. The bytes +7..+10 are never read; they are write-only.

So with a short camp, more men can come back than left. That is an original quirk, and it is reproduced by using +6.

Verdict: guess correct.

## 20. `subplots.rs:949` Type 46 revolt, distributing the remainder (`0x1DE90`, c = 3) [C]

s = sum of troops over h's castles (`0x42A50(kind 2, any, h)` / `0x42B10`, property order). After the spinner n (stored as a **u16** at +6, `0x1E069`), +8..+11 are cleared. Then remainder `rem := 0` **once** before the castle loop (`0x1E08B`), and **per castle P in property order**:
1. for t = 0..3: take = P.troops[t]·n / s, `rem += P.troops[t]·n mod s`; move take from P to +8+t;
2. while `rem > 0` (not ≥ s): draw t = `rand(4)`, then scan cyclically up to 4 types of **this castle P** for a non-empty one. If found: move one man from P[t] to +8+t and `rem −= s` (rem can go negative and carries into the next castle). If all four types of P are empty, leave the loop for this castle; the `rand(4)` has still been drawn.

Differences from the sim:
- The extras are taken from the current castle, interleaved with the proportional shares castle by castle. The sim does all shares first and then takes from the first non-empty castle.
- The loop condition is rem > 0, which means a rounding up per castle with a negative carry. The sim uses rem ≥ s.
- A castle that is empty while rem > 0 costs one `rand(4)`.
- +6 is u16.

Verdict: guess wrong. Implement the per-castle loop above, in property order: share, then extras while rem > 0 from that castle; rem is a signed accumulator shared across castles; +6 is written as a u16.

## 21. `subplots.rs:1045` Contest entrant slots (`0x19480`) [C]

After the human entry loop, if any human entered: for slot = (entrants)..29, unconditionally `+0x0B+slot := person_pick_random(…)` (0xFF when there is no candidate) and `+0x29+slot := rand(32)`. The loop does not break on 0xFF, so `rand(32)` is drawn for every remaining slot. Unused slots hold 0xFF, but only because the failed pick stores 0xFF.

Verdict: guess correct for the stored value. The RNG differs only in the unreachable case: on a failed pick the original keeps drawing `rand(32)` for each remaining slot. To match exactly, store the pick result (0xFF) and draw `rand(32)` for all slots up to 29 without breaking.

## 22. `director.rs:7` Which order slots the director fills

`director_ai` `0x58410` [C]. The director keeps one counter `orders` (starts 0). The transport order is written to slot `orders` (only when `orders < 1`, so always **slot 0**), production orders to slot `orders` as well, and the production loop runs while `orders < 2`. So the director writes slots 0 and 1 only, transport (if any) in slot 0, production in the next free one. Slots 2 and 3 are never written.

Related details read on the way, all [C]:
- The release step at the top (step 2) loops over slots **0 and 1 only** and touches every slot whose status byte (+7) is 2, of any type including production: unreserve (`0x49430`), then status := `order_feasible` (`0x45360`, 0 or 1). The sim's loop over 0..4 restricted to types 2..7 differs: it misses confirmed production slots and touches slots 2/3.
- New transport slot status: feasible → 2 and reserve (`0x49370`); **not feasible → 1** (not 0).
- New production slot: feasible → status 2 and reserve; not feasible → the status byte is **left as it was** (bytes 0..6 are rewritten, byte 7 is not).
- RNG: `rand(6000)` (`0x589D2`) is drawn **before** the `t < 2` test, i.e. for every transport order, also for transporter types ≥ 2. The test is `rand(6000) + 4000 < value && t < 2` → type 3, else 2. The sim's `t < 2 && v > 4000 + r(6000)` skips the draw for t ≥ 2: RNG order diverges.
- Production budget charge is `w · wage_part + m · MC[g]` where `wage_part = ((wage_level + 450) · (WF · wage_base[g] / 100)) / 500` **without** the `MC·10/R` term; that term is only in the divisor used for `w = budget / (wage_part + MC[g]·10/R[g])`. The sim subtracts `w · (wage_part + MC·10/R) + m · MC`, charging means twice.
- Production pick: goods are scanned from 14 down to 0 with a strict `>`; on equal keys the **highest** good index wins. The sim scans upward with `>` (lowest index wins).

Verdict: guess correct (slots 0 and 1, transport in slot 0). Separate fixes needed for the release loop, the two status quirks, the `rand(6000)` order, the double-charged means and the tie-break.

## 23. `director.rs:69` Besieged source city

`0x58410` [C]. At entry the director marks every location that is the `+4` of a type-**25** record (`rec_find(25, 0xFF, 0xFF)` chain, `0x5845B`) as blocked. The only use of the flags is the shipment search of step 4: a destination d is tried only if `d ≠ own city`, `blocked[own city] = 0` and `blocked[d] = 0`. Nothing else checks them. So in a besieged city the director runs everything else normally: slot release, fee, local valuation and **local sale of every good with stock**, repairs (flag 1), production orders with their `rand(10)` draws, storage purchase and swap. Only shipments are impossible (every good's best option stays "sell locally", type 6 = none).

Verdict: guess wrong: do not return early. Treat a blocked source like "no destination qualifies": all steps run, only the shipment options are skipped.

## 24. `director.rs:185` "Free storage" for the production cap

`0x58410` production loop (`0x59000..0x59170`) [C]. `free = (Wd + St) − U`, where `Wd`, `St` = branch wood and stone area (+0x23, +0x25) and `U` = the sum of `output / 17` over the production orders this call has already written (starts 0; the same `U` feeds the storage purchase, `need = U·140/100 − (Wd+St)`). Stock on hand is **not** counted, and there is no clamp at 0. Test: if `output / 17 > free` (output = `w · score / 100`), then `w = free · 1700 / score` and the output is recomputed. Means `m = w · 10 / R[g]`.

The loop continues even when no order results: an order is written only if `w > 0` **and** `m > 0`; otherwise the good's score is zeroed and the next best good is tried (while some score is > 0 and `orders < 2`). Each attempt draws `rand(10)` (`0x590A0`, the `30 + rand(10)` worker cap) before the storage test.

Verdict: guess wrong: free = Wd + St − (sum of output/17 of this call's earlier production orders); ignore stock; no `max(0)`. Also replace `break` on `w ≤ 0` with "skip this good, try the next", require `m > 0` too, and keep the `rand(10)` per attempt.

## 25. `missions.rs:46` Mission 9 "condition byte"

Mission check `0x17930`, case 9 (`0x17B83..0x17BC9`) [C]. Iterates the human's **branches** (`0x42A50(kind 0, 0xFF, owner)` / `0x42B10`) and stops at the first whose u16 at property +0x03 has low byte `0xE6` and high byte (`>> 8`, i.e. byte +0x04) `≥ 0x54` (84). No separate "has residence" test; `0xFFFF` fails the `0xE6` test anyway.

Verdict: guess correct (byte +0x04 = high byte of the residence word).

## 26. `economy.rs:357` Give-away for castles and camps

`branch_add_goods` `0x439F0` [C]. Callers include the battle booty code `0x3EE60` (`0x414A2`), `0x1A770`, `0x1BC00` and the arrivals `0x4B900`. After the overflow cap check, it repeatedly picks the good with stock > 0 and the lowest byte `[0x85F26 + 0x8C·loc + g]` that is `< 100` (start value 100, strict `<`, ascending g, so first minimum), and gives away from it. There is no location check. For loc ≥ 10 the read goes past the 10-record city table (`0x85F04..0x8647B`) into the castle table and the following constant tables. Bytes in the EXE image for the castle/camp locations (g = 0..14):

| loc | address | bytes |
|---|---|---|
| 14 | `0x866CE` | 0 0 2 0 0 0 2 0 0 0 2 0 0 0 3 |
| 15 | `0x8675A` | 0 0 3 0 0 0 1 0 0 0 2 0 0 0 3 |
| 16 | `0x867E6` | 0 0 91 8 0 0 169 9 0 0 124 12 0 0 46 |
| 17 | `0x86872` | 0 0 170 12 0 0 162 11 0 0 172 12 0 0 174 |
| 18 | `0x868FE` | 16 25 28 28 28 29 29 29 30 30 28 28 30 8 8 |
| 19 | `0x8698A` | 114 32 95 0 0 0 0 0 0 0 0 0 0 0 0 |
| 20 | `0x86A16` | 32 76 101 101 114 32 95 0 0 0 0 0 0 0 0 |

Almost every good qualifies, most at "price" 0, so an over-full castle or camp gives goods away in g order (first good with value 0 and stock). Whether these addresses are ever written at run time was not checked [L: they lie in constant tables and a string area].

Verdict: guess wrong: castles/camps do give goods away. For parity, use the table above as the "price" row for loc 14..20.

## 27. `turn.rs:1347` Office income groups

`year_end_world` `0x69FBC..0x6A51B` [C]. Per human, three independent blocks in order turnover, tithe, indulgence. Each block computes its amount (unsigned divisions); if `> 0` (signed) it prints its own line and credits it with `add_cash` `0x43790` (which clamps cash at 9 999 999):
1. turnover: `0xC1ED4` `[EXE text]` (h, h, amount);
2. tithe: `0xC1F30` `… %d Taler aus Rom.$N`;
3. indulgences: `0xC1F7C` `… %d Taler aus Ablassverkäufen.$N`.
No RNG. Because every amount is positive and the clamp is monotone, cash after three separate credits equals cash after one summed credit.

Verdict: guess correct for cash; wrong for output: emit up to three separate messages (each with its own text, in the order above) instead of one summed event.

## 28. `debt.rs:143` Seizure with no other branch

Debt stage 5 inside `0x61A70`, `0x64AFE..0x64C3F` [C]. `y` = first branch of the debtor (`0x42A50`/`0x42B10`) that is not the seized branch x. If there is none, the type-9 redirect loop is skipped entirely: shipments keep their source byte, the redirect counter stays 0, so only `0xC11C8` "Eure Niederlassung in %o." is printed (no `[EXE text]`). Then the slot is freed (`0x429A0`), so those shipments point at a freed property slot.

Verdict: guess correct.

## 29. `save.rs:21` Castle table after loading

Load `0x42200` reads the 12 save blocks (`formats.md` §3.2); none covers `0x8647C..0x86531`. A scan of the whole code for the immediates `0x8647C`, `0x8647E`, `0x86480`, `0x8648A`, `0x8648C`, `0x8648E`, `0x86490`, `0x86491` (base and record-0 fields) finds only readers/writers in the castle screens (`0x54570`, `0x549C0`, `0x55090`, `0x56080`, Zollsatz `0x56CF0`, `0x56E20`), the AI/subplot code (`0x59F80`, `0x5B200`), the settlement `0x61A70` and the year-end world `0x66220`; nothing in load or `world_init` `0x4A490` [C for the scan]. So the table keeps whatever is in memory: the EXE's initial values after a fresh program start, but the **previous game's values** (toll rates, accumulators) when a game was loaded, or a new game started, after playing in the same program run.

Verdict: guess correct for "load right after program start" (still not observed live). The same-run case keeps the old values; undecidable for the port without modelling program lifetime, so the start-value fallback is the reasonable choice.

## 30. `battle.rs:323` Defender anchor "closest to the first entrance square"

Deployment `0x3DDA0` [C]. The first entrance square is found by scanning rows 0..45 (outer) and columns 0..29 (inner) for the first cell whose runtime flag byte (`0xD979C`) has bit 2; with none found the reference point is (30, 46) (the loop end values). The anchor scan then visits every cell in the same row-major order and, for each cell whose terrain low 5 bits equal `0x85831[type] + 6·side` and whose men-layer byte (`0xDBD6C`) is 0xFF:
- keeps the far-from-centre maximum of `(y−20)² + 4·(x−15)²` (strictly greater, so the first maximum wins);
- if side = 1 and field ≥ 2: also keeps the minimum of `(y−ey)² + (x−ex)²` (strictly smaller, so the first minimum wins).
If no cell qualifies the call returns 0. Otherwise, in the side-1/field-≥2 case, the position becomes the entrance-nearest cell.

Two details the port gets wrong:
1. The entrance rule has **no unit-class condition**. It also applies to goods markers (type 20, code `0xFE + 6 = 4`) and wagons (type 21) because they are deployed with side 1. On field 2 (ship raid, the only field that has both goods slots and entrance squares) the goods markers go to the code-4 cells nearest the first entrance, not to the far-from-centre ones. The Rust `code >= 8` restriction does not exist.
2. The unit's stored score (unit +0x12, `0xD7546`), which orders the act list, is **always the far-from-centre maximum over all candidate cells**. It is never the score of the entrance-nearest cell that was actually used.

Verdict: guess correct for "squared distance, first minimum". Guess wrong in two places: drop the `code >= 8` condition (it applies to every type deployed on side 1 with field ≥ 2), and set the unit score to the maximum far-from-centre score of all candidates, not to `score(chosen cell)`.

## 31. `battle.rs:403` Cannon cell not free

`0x3D7C0` (place one man), cannoneer branch [C]. The man's own cell must have men-layer byte 0xFF, else the call fails. For types 2 and 7 the routine then places the cannon (type 16) by a recursive call at `(x + 0x84F50[d], y + 0x84F60[d])`, where d is the anchor direction. That table is `[1,1,0,−1,−1,−1,0,1]` / `[0,1,1,1,0,−1,−1,−1]`, the same as `DX/DY[(d+2) mod 8]`. The cannon placement fails only if that cell's men-layer byte is not 0xFF. It checks neither bounds nor the object layer, and the cannon occupies the men layer. On failure the cannoneer's record is released and 0xFF is returned. The deploy loop (`0x3E07F..0x3E086`) stops at the first 0xFF return. Then the remaining men are deployed by the recursive `deploy` call (see 427).

Quirk [C]: if the **first** cannoneer of a call fails, the unit record has already been allocated and linked into the act list with 0 men, and the recursion runs with the same n. The anchor cell is still free, so the same anchor is chosen and fails again. This repeats until all 100 unit slots are used up, after which every later `deploy` of that battle returns 0. In the shipped `SZENARIO.DAT` the cannon cell is never pre-occupied at deployment time, so this is not reachable with the original maps [L].

Verdict: guess correct (the line stops; the remainder goes on as in 427). The port's `placed == 0 → pop and return` differs only in the unreachable quirk above.

## 32. `battle.rs:415` Goods marker's right cell

`0x3D7C0`, type 0x14 [C]: the marker's index goes into the men layer at (x, y) and **0xFD is written to (x+1, y) unconditionally**, with no free test. (A wagon, type 0x15, writes 0xFC to (x+1, y), (x+2, y) and to x..x+2 of row y−1, also unconditionally.) The anchor test only checks (x, y). In all five shipped fields every code-4 cell has a code-0 right neighbour, and markers are deployed first (`0x3F5A2`), so the cell is always free in practice.

Verdict: guess correct in effect. The original has no "when free" test; writing 0xFD unconditionally matches it exactly.

## 33. `battle.rs:427` Men left over when the line ends

`0x3DDA0` [C]: the placement loop ends when a placement fails, when the count reaches n, when the next cell's terrain byte differs from `(anchor & 0xE0) + 7`, or when the next cell's men layer is not 0xFF. If fewer than n men were placed, `deploy` calls itself (`0x3E275`) with n − placed and the same type, participant, side and training. That allocates a new unit at a new anchor. The return value is the total placed. The deploy loop itself has no 10-per-unit cap. The 10 man slots of a unit are enforced only in `0x3D7C0`, which calls the fatal error `0x2FA00` on an 11th fighter. The longest code-7 line in the shipped fields gives 6 men per line, so the cap is never reached.

Verdict: guess correct.

## 34. `battle.rs:441` Goods markers / wagons in the act list

`0x3DDA0` at `0x3E169..0x3E178` [C]: after placing type 20 or 21, the unit record is freed again (unit type byte := 0xFF) and is **not** linked into the act list. `0x3D7C0` also adds only fighters (type < 12) to a unit's 10 man slots. So markers and wagons never act.

The act list itself, also confirmed here [C], differs from the port. It is a singly linked list (head `0xDC83C`, next = unit +0x11, end = 100) sorted **ascending** by the unit score:
- if the head's score is ≥ the new score, the new unit becomes the head;
- otherwise the code walks while the next score is < the new score and inserts there.
So the **lowest** score acts first. Among equal scores the most recently deployed unit comes first. The round loop in `battle_main` (`0x3EE60`, head `0xDC83C`, then `0xD7545`) walks it in that order. This contradicts `rules-07` §5.4/§5.11 ("higher acts first").

Verdict: guess correct for "markers/wagons not in the list". Guess wrong for `act_order`: sort ascending by score, and among ties put later-deployed units first. Use the score defined in 323.

## 35. `battle.rs:526` Path mode 5 (Bombenleger)

`path` `0x37C00` [C]. Modes 0, 4 and 5 build the same base map:
- a cell holding a record of the other side with type < 12 becomes a goal (0);
- a cell holding the walker's own side is blocked;
- an empty cell with an empty object layer and without the edge flag becomes walkable;
- everything else is blocked.

Mode 5 then adds one step. For every goal cell and each of its 8 neighbours that is walkable, look at that neighbour's own 8 neighbours. If any of them holds a record (men-layer byte < 0xFB) of the **walker's side but a different unit** (man +0x0F ≠ the walker's unit), the approach cell is blocked. The records counted are men, cannons, markers and banners. In effect a Bombenleger only approaches enemies from cells that are not next to friendly troops of other units. BFS and return values are then as for mode 0. Mode 4 instead runs the 4-step band BFS.

Verdict: guess wrong. Mode 5 means mode-0 goals, plus blocking every walkable cell adjacent to a goal that is itself adjacent to a same-side record of another unit.

## 36. `battle.rs:584` Step cell became occupied

Walk step in `0x389C0` (e.g. near `0x39453`) [C]. The original does not test the destination. It writes 0xFE into the men-layer cell in the path direction (`0xDC837`) and starts the walk animation. The test is unnecessary because `path` is computed for that man immediately before the step, and the first step always goes to a cell the BFS treated as walkable. Such a cell has men layer 0xFF, object layer 0xFF and no edge flag, except for the mode-1 goal (edge) and the mode-2/6/7 goals, which are also required to be empty. Cells reserved by other walkers (0xFE) are blocked in every mode.

Verdict: guess correct. The branch is dead in the original. Keep it as a safety net that never fires if path and free agree.

## 37. `battle.rs:878` Influence map "every living man"

`0x389C0` at activation start, for tactics 1 and 2 [C]. The map is cleared. Then every men-layer cell (row-major) holding a record with type < 12 adds −HP (record's side = unit's side) or +HP (other side) to all in-field cells within ±2. For tactic 2 every 0xFB wall cell adds +100. There is no state test, but nothing can slip through it. At activation start no animation is running. Killed men are removed from the layer when their fall animation ends (`0x3DC40`, HP = 0). Captured men become type 12 at that point, with side flipped and owned by the captor (case 4 of the end-of-animation switch). Men who fled are removed. So the set is exactly the living fighters, each at the cell they stand on.

Verdict: guess correct.

## 38. `battle.rs:921` Explosion length

End-of-animation, state 0, object types 0x11/0x12/0x13 [C]. The explosion's frame counter is set to `0x855C8[type][state] + 1`. The table value is 8 (type 13 state 8; types 14/15 states 9/10), so the counter is **9**.

Related findings in the same code, which change damage and RNG:
- The own-side halving and the dodge roll sit inside the "k ≠ 8" branch (`0x3C9F0` → `0x3CA3B`). So the **centre cell always takes 100, even for the bomber's own side**. Correction to `rules-07` §5.13, which says "100, halved for own side = 50".
- RNG order of one explosion:
  1. cannon ball on a wall/obstacle cell: `rand(2)`;
  2. `rand(6)` only if displayed;
  3. `rand(100)` always;
  4. `rand(6)` only if that was < 90 and displayed;
  5. for k = 0..8: for a fighter that is not fallen, k ≠ 8: `rand(80)` (k even) or `rand(60)` (k odd), then `rand(100)` dodge;
  6. **then a crater-decal loop that runs in both modes** (`0x3CAE3..0x3CD00`, drawn onto the background page `0xDC834`, no `0xDC83F` test).
- The crater-decal loop works on the centre pixel `px = 24·cx − 28`, `py = 12·cy − 34`:
  - dx runs over `max(−36, −px) ≤ dx < min(36, 640 − px)` (outer loop);
  - dy runs over `max(−18, −py) ≤ dy < min(18, 479 − py)` (inner loop);
  - per pixel: `rand(5)`; if that is < 2, then `rand(300)`; if that is > `dx²/2 + dy²`, then `rand(2)` (sprite choice `0xFB − q/100 + rand(2)`).
  That is about 2,600 `rand(5)` calls per explosion. The port draws none of them.

Verdict: guess correct (9). Separately: centre damage must not be halved, and the crater loop's RNG draws must be added after the damage loop.

## 39. `battle.rs:993` Frame loop order ("row by row")

`0x389C0` frame scan (from `sVar11 = −0x1C` at about `0x3AD..`) [C]. Each frame first clears every record's "done" flag (+0x1D). Then, for rows y = 0..45, with baseline pixel `B = 12·y − 28`, it makes **four passes over x = 0..29**:
- pass 0: men-layer records in that cell whose pixel y < B;
- pass 1: men-layer records with pixel y = B;
- pass 2: object-layer (`0xDC2D0`) records with pixel y = B;
- pass 3: men-layer records with pixel y > B.

Each record is handled at most once per frame. A record's men-layer cell is its origin cell until its walk ends. A walking man's pixel y follows `0x84ECC[dir][frame]`:
- 0 at frame 0;
- then −2 per frame for directions 0, 1, 7 (pass 0);
- +2 per frame for directions 3, 4, 5 (pass 3);
- 0 for directions 2 and 6 (pass 1).
Men, cannons, banners/prisoners, markers and wagons are in the men layer. Bombs, cannon balls and explosions are in the object layer.

Per record, in this order:
1. **Draw section.** If the frame index is below the animation length: for a man in state 3 (walk) whose frame index is 2 or 5, `rand(100)` is drawn **in both modes**, and the sound sample `rand` follows only if that is < 60 and displayed. Other draw-section sounds are display-only.
2. **Advance.** If frames ≠ 0: index += 1 (not for types 18, 19), frames −= 1. Walk pixel update. Bomb drop at lay frame 6. End-of-animation handling at 0.

So at one cell, men-layer records come before object-layer records, as guessed. But the real order is not (y, x): all pass-0 and pass-1 men of a row (every x) come before all objects of that row, and up-moving men come before non-moving ones. The walk-sound roll (two `rand(100)` per walked cell, at frame indices 2 and 5) is drawn in ESC mode too. The port does not draw it at all. Correction to `rules-07` §5.11 ("presentation RNG only when displayed").

Verdict: guess wrong. Order by (row of the layer cell, pass 0/1/2/3 as above, x), and draw the walk-sound `rand(100)` at walk frame indices 2 and 5 before advancing.

## 40. `fight.rs:168` Computer helper bribe, "helper index quirk"

`battle_main` `0x3EE60`, bribe loop `0x404D1..0x4054C` [C]. For a computer briber `s` (person `0xDB7F4[side]`) and helper index i, in this order:
1. `b = value / (4·n) + rand(100)`;
2. `strength(side)` (`0x38730`);
3. b := 0 if strength ≤ 1, or b < 500, or **`0xDB7F4[i]` = s**, or **`0xDB7F4[i]` is human** (+0xE3A8C & 0xF0).

`0xDB7F4[i]` is participant i: the attacker for i = 0, the defender for i = 1. It is not helper i (`0xDB7F6[i]`). So:
- the attacker never bribes helper 0, and the defender never bribes helper 1;
- no computer bribes helper 0 while the attacker is human, or helper 1 while the defender is human;
- whether the helper's owner is human is not checked at all.

Then, whatever b is, the text is printed. If the helper's owner is the briber, it is `0xBBA98` `[EXE text]`. The bribe is added to the side total, and pay, receive, net worth and `favour_increase` are called, with b possibly 0, even when the briber owns the helper.

Verdict: guess wrong. Replace "helper owner is human" with "participant slot i (attacker for helper 0, defender for helper 1) is the briber or is human". The RNG order stays `rand(100)` then the strength test, once per (briber, helper) pair.

## 41. `castles.rs:370` Average shipment with nothing in transit

`year_end_world` `0x66220` at `0x68FE8..0x69073` [C]. The routine sums +5 (u16 quantity) over all type-9 records and counts them. Only if the count is > 0 does it set `0x85E1D := (sum / count + 0x85E1D) >> 1`, with signed division and an unsigned shift. With no shipment the global is left unchanged.

Verdict: guess correct.

## Summary

| item | verdict | section |
|---|---|---|
| `career.rs:97` | guess correct for the membership (indices 0..2 at every level, city 7..8). Guess wrong on seat placement: fill each vacant seat separately, highest index first, one full election (with its own RNG draws) per vacant seat; the winner sits in that seat. | §1 |
| `career.rs:259` | guess wrong: a county office holder may only take offices of the county itself (place = `l`), plus the principality for Kanzler/Vogt/Bischof; drop the "cities of the county" alternative. | §2 |
| `career.rs:333` | guess correct for the preference (lowest favour at `l`, else lowest elsewhere, first minimum). Guess wrong on the location of an office-less human: use a random branch (`rand(count)` over h's branches in slot order), not the first branch. Also exclude persons whose favour toward h is ≥ 102. | §3 |
| `career.rs:399` | guess wrong: the rand(5) is not unused; apply skill g += 2 + rand(5), capped at 150. (Closes rules-03 open question 1.) | §4 |
| `career.rs:457` | guess wrong (minor, no gameplay effect): store the seat that was listed / filled, i.e. the highest vacant eligible seat of the group for a human, the vacant seat itself for an auto-created record; the Schreibstube should list that seat (not the lowest free one) and match existing records by exact index. | §5 |
| `career.rs:597` | guess correct: a blackmailer holding two appointers votes twice (with all RNG draws twice). | §6 |
| `trials.rs:175` | guess wrong: there is no automatic answer; route every blackmailer, human or computer, through the same `BlackmailVerdict` question (the player at the screen decides). In practice blackmail markers come from human back-room actions, so computer blackmailers are rare. | §7 |
| `family.rs:9` | guess wrong: mode 3 deletes none of p's own records; it only removes other persons' records that point at p (types 6, 7, 18, 22, 36, 43, 45, 46, 50), skipping types 4 and 30. Also, in all modes a record owned by p must not be deleted through the reference rule (the sim's `own || refs` can delete e.g. p's own type 6 with `+4 = p` in mode 2/3). | §8 |
| `family.rs:42` | guess wrong: draw a new heir per property, in slot order, each with its own `rand(count)`. Also do not filter out kind 3 slots: any slot with owner byte p is reassigned (free slots normally have owner 0xFF, so this rarely matters). | §9 |
| `ai.rs:538` | guess correct for the draw (one uniform `rand(count)`, none if empty). Adjust the candidate set: every slot with place byte L, any kind and owner (not only castles/camps), slot order. | §10 |
| `ai.rs:548` | guess correct (implementation matches rules-09 §4b; remove the stale SPEC-GAP comment). | §11 |
| `social.rs:263` | guess wrong: candles = `0x85D41` (`standing(p, 30)`), alms = `0x85D3D` (`standing(p, 40)`). RNG order at turn start: alms price first, then candles price. | §12 |
| `events.rs:63` | guess correct (unreachable in play). If byte-exact behaviour on corrupt saves matters, read RB[10..19] as the BT values instead of clamping. | §13 |
| `events.rs:297` | guess correct (outcome-equivalent). Caveat [UNVERIFIED]: I did not check whether any code between the year-end fate and the owner's next turn start reads +10 of a type-9 record (castle interception, mishaps). If something does, it sees the raw property number in the original. | §14 |
| `events.rs:321` | guess correct. | §15 |
| `events.rs:940` | guess correct ("branch of the first run in heap order"). Separate finding: the year gate is `year ≥ 1610`. The sim has `self.year > 1610`, which is wrong for 1610. rules-06 §4 row 69 also says "> 1610" and should be corrected. | §16 |
| `subplots.rs:87` | guess correct. | §17 |
| `subplots.rs:122` | guess wrong. Use `persons[x].idx()` (raw +0x11, 15 if no office) instead of `global_office(x)`. With the global index, county and principality hosts get R inflated by 10 or 23, which changes Q and the stored R. | §18 |
| `subplots.rs:616` | guess correct. | §19 |
| `subplots.rs:949` | guess wrong. Implement the per-castle loop above, in property order: share, then extras while rem > 0 from that castle; rem is a signed accumulator shared across castles; +6 is written as a u16. | §20 |
| `subplots.rs:1045` | guess correct for the stored value. The RNG differs only in the unreachable case: on a failed pick the original keeps drawing `rand(32)` for each remaining slot. To match exactly, store the pick result (0xFF) and draw `rand(32)` for all slots up to 29 without breaking. | §21 |
| `director.rs:7` | guess correct (slots 0 and 1, transport in slot 0). Separate fixes needed for the release loop, the two status quirks, the `rand(6000)` order, the double-charged means and the tie-break. | §22 |
| `director.rs:69` | guess wrong: do not return early. Treat a blocked source like "no destination qualifies": all steps run, only the shipment options are skipped. | §23 |
| `director.rs:185` | guess wrong: free = Wd + St − (sum of output/17 of this call's earlier production orders); ignore stock; no `max(0)`. Also replace `break` on `w ≤ 0` with "skip this good, try the next", require `m > 0` too, and keep the `rand(10)` per attempt. | §24 |
| `missions.rs:46` | guess correct (byte +0x04 = high byte of the residence word). | §25 |
| `economy.rs:357` | guess wrong: castles/camps do give goods away. For parity, use the table above as the "price" row for loc 14..20. | §26 |
| `turn.rs:1347` | guess correct for cash; wrong for output: emit up to three separate messages (each with its own text, in the order above) instead of one summed event. | §27 |
| `debt.rs:143` | guess correct. | §28 |
| `save.rs:21` | guess correct for "load right after program start" (still not observed live). The same-run case keeps the old values; undecidable for the port without modelling program lifetime, so the start-value fallback is the reasonable choice. | §29 |
| `battle.rs:323` | guess correct for "squared distance, first minimum". Guess wrong in two places: drop the `code >= 8` condition (it applies to every type deployed on side 1 with field ≥ 2), and set the unit score to the maximum far-from-centre score of all candidates, not to `score(chosen cell)`. | §30 |
| `battle.rs:403` | guess correct (the line stops; the remainder goes on as in 427). The port's `placed == 0 → pop and return` differs only in the unreachable quirk above. | §31 |
| `battle.rs:415` | guess correct in effect. The original has no "when free" test; writing 0xFD unconditionally matches it exactly. | §32 |
| `battle.rs:427` | guess correct. | §33 |
| `battle.rs:441` | guess correct for "markers/wagons not in the list". Guess wrong for `act_order`: sort ascending by score, and among ties put later-deployed units first. Use the score defined in 323. | §34 |
| `battle.rs:526` | guess wrong. Mode 5 means mode-0 goals, plus blocking every walkable cell adjacent to a goal that is itself adjacent to a same-side record of another unit. | §35 |
| `battle.rs:584` | guess correct. The branch is dead in the original. Keep it as a safety net that never fires if path and free agree. | §36 |
| `battle.rs:878` | guess correct. | §37 |
| `battle.rs:921` | guess correct (9). Separately: centre damage must not be halved, and the crater loop's RNG draws must be added after the damage loop. | §38 |
| `battle.rs:993` | guess wrong. Order by (row of the layer cell, pass 0/1/2/3 as above, x), and draw the walk-sound `rand(100)` at walk frame indices 2 and 5 before advancing. | §39 |
| `fight.rs:168` | guess wrong. Replace "helper owner is human" with "participant slot i (attacker for helper 0, defender for helper 1) is the briber or is human". The RNG order stays `rand(100)` then the strength test, once per (briber, helper) pair. | §40 |
| `castles.rs:370` | guess correct. | §41 |

## Corrections to other docs

- `rules-03` open question 1: the yearly computer skill gain is `+ (2 + rand(5))` on good `rand(15)`, cap 150 (§4).
- `rules-06` §4 row 69: the year gate is `year ≥ 1610`, not `> 1610` (§16).
- `rules-07` §5.4 / §5.11: units act in **ascending** score order; among equal scores the later-deployed unit acts first (§34).
- `rules-07` §5.11: the walk-sound `rand(100)` (frame indices 2 and 5) and the explosion crater-loop draws happen in ESC mode too (§38, §39).
- `rules-07` §5.13: the explosion's centre cell takes 100 on both sides; it is not halved for the own side (§38).
- `rules-09` §5: the director does not stop at a besieged source city; only the shipment options are skipped (§23).
- `rules-11` (church prices): candles = `0x85D41` (`standing(p, 30)`), alms = `0x85D3D` (`standing(p, 40)`) (§12).
