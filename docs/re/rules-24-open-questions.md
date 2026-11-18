# Open questions: insult refusal, transporter lot limit, castle interest pool (rules-24)

Confidence tags as in `rules-00-map.md`: **[C]** read at instruction level, **[L]** decompiler reading, **[G]** guess. Notation as in `rules-22` and `rules-23`: `rand(n)` = 0x36FF0, `box(x, y, w, h, text, args…)` = 0x368E0, `rec_find(type, owner, key)` = 0x426B0 / `rec_next` 0x427B0 (0xFF = wildcard), `rec_alloc(type, owner, key, counter)` = 0x424A0. `print` arguments in consumption order (`rules-12` §1). Everything below is [C] unless marked.

## 1. "Beleidigung aussprechen": the human target's answer (office tree 0x45C30, mode 6, 0x47412..0x47673)

`P` = the insulting human (function argument, the current player), `T` = the clicked target person.

Order of checks:

1. `rec_find(22, P, any)` and the next ones, looking for `+4 == T`. Found: `box(160, 160, 320, 160, 0xBCC10, T)` `[EXE text]`. Answer 0 ("entschuldigen") deletes that record (bit 7); answer 1 keeps it. Done.
2. Otherwise `rec_find(22, P, any)` finds any type-22 record of P: `box(160, 160, 320, 160, 0xBCC70)` `[EXE text]`. Done. This is the "one duel per insulter" rule.
3. Otherwise the insult. RNG order: `r = rand(12)` (taunt), then `rand(2)` (voice), then the texts below. Speech `0x15100(1 + rand(2), gender(T), 2, 1 : 11 + r)`, started by `0x154B0(0)` (`rules-16`).
4. Branch on T's controller nibble (person +0x10 & 0xF0):
   - **T human** (nibble ≠ 0). There is no separate turn or screen for T: the question appears right away in P's office tree (hot-seat). `box(140, 120, 360, 240, [0xB5F50] = 0x9B99E, place, T, taunt, T)` with the text `[EXE text]`. `place` = `0xB5EF8[rand(6)]` (Auf einem Fest, Während einer Beerdigung, Beim Kartenspielen, Während des Gottesdienstes, Bei einer Ratssitzung, Auf dem Markt), `taunt` = `0xB5434[r]` (`rules-16` §2: the twelve `>…<` lines). Menu: answer 0 `Jawohl!`, answer 1 `Nein.`.
     - Answer 1 (refusal): speech stopped (0x15480), leave the case. **No record, no favour change, no further message, no history entry.** The only RNG draws are the three above (`rand(12)`, `rand(2)`, `rand(6)`).
     - Answer 0: falls through to step 5.
   - **T computer** (nibble 0). No question. `box(140, 120, 360, 240, [0xB5EEC] = 0x9B789, place, T, taunt, adverb, T)` with `[EXE text]`; `adverb` = `0xB5F30[rand(8)]` (Wütend, Daraufhin, Wutentbrannt, Lauthals, Sich überschätzend, Mit finsterer Miene, Natürlich, Nach längerem Grübeln). RNG order here: `rand(8)` then `rand(6)` (argument evaluation order).
5. `rec_alloc(22, P, 0xFF, 1)`, `+4 := T`; speech stopped (0x15480). The duel runs at P's turn end (`rules-22` §1).

Corrections: `rules-08` §3 "a human insulted person can refuse (no record)" holds, and refusing costs nothing. The refusal penalties in `rules-09` reaction table, case 6 (`favour_decrease(v, a, 10)`, office-place spread) belong only to the **computer-insulter** path 0x5B200; the human-insulter path has none. `rules-04` "for a computer target 2 RNG calls" undercounts: every insult draws `rand(12)` and `rand(2)`, then `rand(6)` (human target) or `rand(8)`, `rand(6)` (computer target).

## 2. Transporter auction lot limit (transporter screen 0x53A60, button "Versteigern", 0x54363..0x5451D)

`B` = the branch (property index `b`, owner `o` = branch +0x02), `t` = transporter type of the active row (row 0 = screen type, row 1 = screen type + 3), `n` = the amount chosen on the spinner for that row (clamped to the stock at branch +0x27 + t when chosen [L]).

1. `rec_find(3, 0xC0, any)` = 0 (no announced auction): `box(140, 180, 360, 120, 0xBF200)` `[EXE text]`. Done.
2. `n = 0`: nothing happens.
3. Lot lookup: the first `rec_find(12, o, b)` record with `+4 == t`. None: `rec_alloc(12, o, b, 100)`, `+4 := t`, `+5 := 0`, `+6 := 100`. So there is **one lot per (owner, branch, transporter type)**; lots of other branches or types do not count.
4. Limit test: **`lot+5 + n >= 100`** refuses: `box(160, 160, 320, 160, 0xBF234, t + 19)` `[EXE text]` (`%8s` = noun of text id 19 + t, case 8, plural by the count argument [L for the rendered form]). Nothing moves. So a lot holds at most **99** transporters, and a single addition must keep the lot below 100.
5. Otherwise: `lot+6 := (lot+6 · lot+5 + cond(B, t) · n) / (n + lot+5)` (integer division, cond = branch +0x2D + t), `lot+5 += n`, branch stock +0x27 + t `-= n`.
6. In both cases the spinner amount is reset to 0.

Edge [C]: step 3 creates the record before step 4, so a refused first addition (`n >= 100`) leaves an empty lot (count 0, condition 100). Whether `n >= 100` is reachable depends on branch stock reaching 100 of one type [L].

## 3. Castle interest pool (castle info 0x549C0, 0x54A8C..0x54B4F)

At the top of **every pass** of the screen loop (back-edge 0x5506D, i.e. on opening and after every button that does not close the screen), regardless of who is viewing (owner or not):

1. Collect `rec_find(17, any, c)` records (c = castle property index), heap order, stop at 6. The cap is a display/collection cap; it does not stop other code from creating more [L for other creators].
2. If none were found: `rand(100) < 50` → `x = person_pick_random(13, 0xFF, 0xFF, 0)` (0x43560); if `x ≠ 0xFF`: `rec_alloc(17, x, c, 2)` (counter 2, no other fields written) and it joins the list. If the roll fails or nobody qualifies, no record.

`person_pick_random(place, low_nibble, office, controller)` (0x43560 over the iterator 0x433E0/0x434E0), as called here:
- Qualifies: person slot in use (byte +0x00 ≠ 0xFF), **office place byte +0x0F = 13**, **controller nibble (+0x10 & 0xF0) = 0** (computer). Low nibble of +0x10 and office index +0x11 are wildcards (0xFF), so any principality office holder qualifies, church offices included; the castle's owner is not excluded.
- Candidates are listed in slot order 0..191; then **one draw `rand(count)`** picks the index. No candidate: no draw, result 0xFF.

RNG order per pass with no interest record: `rand(100)`; if `< 50` and at least one candidate, `rand(count)`. A pass with at least one record draws nothing here.

Correction to `rules-08` §1.5: (a) the roll is not tied to the owner opening the screen; any viewer and every redraw pass rolls while there are no interest records. (b) A **human** buyer's refusal is honoured: `box(180, 160, 280, 160, 0xBF4C0, buyer, "Ja", "Nein")` `$Z$C%1N0, schlagt Ihr ein?$Z%m %m$N`; any answer ≠ 0 ("Nein") cancels the sale (0x54EAF: non-zero result sets the choice index past the list, and the sale branch is skipped).
