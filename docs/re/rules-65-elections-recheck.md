# Office elections re-check (rules-65)

Re-read of the vacancy cycle in `year_end_world` `0x66220` (`0x66CE0..0x6884D`), prompted by a playtester report ("I get every office I apply for; my rival votes for me even when he is the only elector"). Conventions as in `rules-44`: `rand(n)` = `0x36FF0`, `rec_find(type, owner, key)` = `0x426B0` + `rec_next` `0x427B0`, `rec_new(type, owner, key, counter)` = `0x424A0`, `rec_delete` = `0x42550`, `office_eligibility(p, place, idx)` = `0x47C20`, `holder(place, kind, idx, controller)` = `0x433E0`. Person fields: `+0x0D` title, `+0x0F` office place, `+0x10` low nibble church flag / high nibble controller (0 = computer), `+0x11` office index, `favour(a→b)` = byte `+0x20 + b` of a. R = the year-start rival snapshot (one rival per human, `rules-44` §4). Tags: **[C]** instruction level, **[L]** decompiler only.

Short version: the playtester is right. When a vacancy has no applicant, the game draws three computer candidates and adds the humans' rivals **before** it tests whether a human is eligible, and it gives every one of them an application record (type 5, counter 2). The human-eligible test then only skips this year's election. Those records survive into the next year, so a human who applies the following year faces the drawn computers and his rival(s) in a real vote. The "office stays open until a human applies alone" behaviour of our build exists in the original only when no computer person at all was eligible.

## 1. When is the computer draw skipped? [C]

Answer: never skipped on its own. Order inside one vacant seat, after the elector list and the applicant scan (`0x672AD`):

1. `n` = number of applicants found (type-5 records matching place and group index, not in prison). If `n > 0` go to step 7: nothing is drawn, no rival is added.
2. `n == 0` (`0x672B1`): scan persons 0..191 (`0x672D2..0x673AD`). Computer person with `office_eligibility(p, L, seat) == 2` and not listed in a type-44 record → pool. Human with eligibility 2 → flag **H** = 1 (`0x67389`). Nothing else is done with humans.
3. For each human h in slot order (`0x673B2..0x6744B`): if R[h] has eligibility 2 and is not in a type-44 record, R[h] is appended to the candidate list. No draw, no de-duplication yet.
4. If the pool is non-empty (`0x67450`): three times `rand(pool size)`, the drawn person is appended (`0x67485`). The three draws happen even when H = 1.
5. De-duplication over the whole list (`0x674C0..0x6761B`): a candidate equal to an earlier one, or with the same name index (`+0x08`), is removed (last entry moved into the hole). Every survivor gets `rec_new(5, candidate, 0xFF, counter 2)` with `+4 = L`, `+5 = seat` (`0x675B9..0x675D6`). This happens **regardless of H**.
6. Only now (`0x67620`): if H = 1 the seat is skipped (jump to `0x687E1`, index decremented, no election, no appointment). The records of step 5 stay on the heap.
7. H = 0 (or `n > 0` from the start): election as in `rules-03` §4.6 / `rules-44` §3.

So the condition for "office stays vacant this year" is exactly: nobody applied AND at least one human has eligibility 2 (regardless of whether that human could or did apply, regardless of the pool). The drawn computers are not "in addition to applicants": the draw only runs when there are none. But they become next year's applicants (§3).

Corrects `rules-03` §4.6: "If any human is eligible, the office stays vacant this year" is incomplete. The rivals are added and the three draws are made before that test, and every drawn or rival candidate keeps a counter-2 application record. Also the order is rivals first, then the three draws (rules-03 lists the draws first); this order decides who survives the name de-duplication, the candidate list order on screen, and which candidate wins a scoring tie ("first maximum").

Corrects `rules-40` §1 last bullet: the skip is right, but it is not RNG-free in the sense of "nothing happened": the three `rand(pool)` draws and the record allocations did happen for the first seat of the group.

## 2. Applicants present: are computers added? [C]

No. With `n > 0` the branch of §1 steps 2–5 is not entered (`0x672B1` jumps to `0x67620`), and no other code adds candidates. A lone human applicant with no privileged candidate is appointed without a vote, text 0xC1998 (`0x68431`, `rules-44` §3 step 1).

But "applicants present" normally includes the computer persons registered the year before (§3). Timeline for a seat that becomes vacant at the year end of year Y (dismissal §4.3 or turnover §4.5 both run before the vacancy cycle):

- Year end Y: no applicant (nobody could apply for a seat that was not vacant during the year). Rivals + three draws get records. If some human is eligible: no election. Otherwise the election runs among them now.
- Year Y+1: the Schreibstube lists the seat for eligible humans, and its "bewirbt sich bereits" window shows the registered computer applicants, rivals marked (`rules-64` §2.3). This is why that window exists.
- Year end Y+1: applicants = human(s) who applied + the registered computers (their eligibility is not re-checked; only prison, `0x671EB`/`0x6720A`). `n ≥ 2` → vote. If no human applied: the computers elect among themselves, the seat is filled. A seat is therefore open to humans for exactly one year, and a human never gets it uncontested unless the pool and the rival set were empty at year end Y, or the seat became vacant in a way that leaves no year end between vacancy and application (1600: the whole cycle is skipped, `0x66CC7` tests year > 1600, so a seat vacated at the 1600 year end gets no draw and a 1601 applicant is alone).

## 3. Type-5 records: creators, counters, deletion [C]

All 86 call sites of `rec_new` `0x424A0` were checked for the type argument. Exactly two create type 5:

| site | who | when | owner | counter | +4 / +5 |
|---|---|---|---|---|---|
| `0x4857F` in `office_apply` `0x47F90` | human, Schreibstube "Ja" | during the human's turn | the human | 2 | place / listed (highest vacant) group index |
| `0x675B9` in `year_end_world` | the no-applicant branch of §1 | year end, vacancy cycle | each surviving drawn computer or rival | 2 | place / the vacant seat |

Nothing in the computer AI turn, the record handlers or the sub-plots creates one. Computer persons never apply "during the year"; their applications are only the year-end registrations above.

Decrements and deletions:

- Human's turn end (`0x61A70`, loop `0x63661`: `rec_find(any, human, 0xFF)`, `0x63752` decrement): counter 2 → 1. Type 5 has no handler; the default case deletes at 0 only, so the record survives.
- Vacancy cycle, after the whole place loop (`0x68824..0x6884D`): every type-5 record with counter ≤ 1 is deleted. This removes the humans' records (now at 1) and last year's computer registrations (at 1, see next bullet).
- Winner: all his type-5 records are deleted at appointment (`0x687B5..0x687D3`, `rec_find(5, winner, 0xFF)`).
- Year-end record pass over computer persons (`0x69DFF` enumerates `holder(0xFF, 0xFF, 0xFF, controller 0)`, `0x69E20` `rec_find(any, p, 0xFF)`, `0x69E3E` decrement; types below 0x12 are deleted only at 0). It runs **after** the vacancy cycle, so a registration made this year end goes 2 → 1 here, is an applicant at the next year end, and is removed by the counter ≤ 1 purge after that election (or by the winner deletion).
- Schreibstube "Nein" deletes the human's own record (`rules-64` §2.4). Record purges `0x42F60` (dismissal mode 3, turnover and retirement mode 1) can remove a computer registration with the person's other records [L: which types mode 3 covers is in `rules-05` §7.3, not re-read here].

So a computer registration lives exactly one year, a human application lives from the click to the next year end.

## 4. Computer elector's score, re-verified [C]

Per elector entry e (the appointer's holder, or each person blackmailing him, `rules-40` §6), computer electors only (`0x67B1B..0x67E10`), per candidate c:

```
score = favour(e→c) + title(c)                               ; 0x67B48 favour, 0x67B6C title (bytes +0x20+c of e, +0x0D of c)
if church_flag(c's current office) == church_flag(vacant office)   ; low nibble of +0x10
   and c holds an office (+0x11 < 15):                       ; 0x67C31
    if place(c's office) == L:                               ; 0x67C3D
        chain = holder(L, kind of vacant office, any idx, controller 0), then next  ; 0x67CAF / 0x67CCD
        sum = Σ favour(q→c) over the chain, count = chain length
        if sum > 0: score += sum / (count * 8)               ; 0x67D48
    g = global index of c's current office                   ; 0x67D5A (0x43FF0)
    score += (33 - max(0, o - g)) >> 1                       ; 0x67D6E..0x67D7E (0x441F0 = max(0, x))
    if c is in R (c is some human's year-start rival):       ; 0x67D84..0x67DDC, scans the rival array
        score += rand(20)                                    ; 0x67DE3
vote for the first candidate with the highest score (strict >, so ties go to the earlier candidate)   ; 0x67DF9
```

Then `rand(25)` → `favour_increase(e, winner)` and, per loser, `rand(10)` → `favour_decrease(e, loser)` (`0x68002..0x68090`), as documented.

Corrects `rules-03` §4.6: the `rand(20)` bonus goes to candidates that are a human's rival (members of R), **not** to human candidates. The loop at `0x67D84` walks the rival array (the one filled by `human_rival_of` at `0x662CD`), not the human array; the text arguments of 0xC1818 use the same array. A human candidate gets no random term.

No term looks at: whether c is human, whether e is c's rival, whether e is the rival of a human candidate, bribes (the Bestechung path `0x4F370` writes favour and is therefore already inside `favour(e→c)`; nothing else), blackmail (it only decides who votes, §4.6), citizenship, residence, wealth. A rival elector votes against the human only through his low favour; nothing forces it.

Quirk [C]: the colleague chain filters on place and church nibble only, not on "holds an office". Persons without office have place 0 and nibble 0, so for a secular seat in place 0 the officeless computer persons count as colleagues (favours summed and counted). For every other place the chain is the office holders there with the same kind.

Note on "same kind": an officeless candidate has nibble 0, equal to a secular vacancy's flag, but the `+0x11 < 15` test excludes him, so the net effect is "holds an office of the same kind", as `rules-03` says.

## 5. Other ways a human applicant loses or is refused [C]

- Loses the vote to the registered computer candidates or rivals of §2/§3, or to another human. There is no minimum vote count, no favour threshold, no quorum; the winner is the candidate with most votes, ties broken by `rand(number tied)` (`0x68159`), drawn also when one candidate leads (`rules-44`).
- Privileged candidate (Erzdiakon / Diakon / Sakristan with his own application, `0x67036..0x6717B`): he is the only candidate, other applicants are shut out (their controller nibble still flags the election as displayed).
- Prison: an applicant with a type-20 or type-21 record is skipped (`0x671EB`, `0x6720A`).
- No eligibility, title, citizenship or branch check at election time. Eligibility is only tested when the Schreibstube lists the seat (and for the pool and the H flag of §1). A human who sold his branch after applying still competes.
- If a seat has no elector at all (all appointer seats empty), the vote loop runs zero times, every candidate has 0 votes, and `rand(n)` picks the winner among all of them.
- Humans can also be refused by the H flag of §1 in a roundabout way: a vacancy they could not apply for yet is held for them one year, but the registered computers of that year are then their opponents.

## 6. Deviations in `crates/mad-sim/src/career.rs`

- `fill_one`: returns at "a human is eligible" before the rivals are added and before `new_application` is called for the drawn candidates. The original creates the counter-2 records for rivals and draws first (§1 steps 3–5), then skips. This is the bug behind the playtester's report: in our build the seat stays empty of computer applicants, so the human is alone next year and is appointed via 0xC1998 without a vote.
- `fill_one`: the three draws are pushed before the rivals; the original appends rivals first (per human slot), then the three draws, then de-duplicates the combined list. Order matters for the name de-duplication, the screen order and the first-maximum vote.
- `vote_score`: `if c is human → rand(20)` must be `if c is in the rival snapshot → rand(20)` (§4).
- `vote_score` colleagues: the sim requires `has_office()`; the original chain does not (place-0 secular quirk, §4). Minor.
- The rest (electors, privileged, prison, counter ≤ 1 purge, winner's records deleted, year-end computer record pass decrementing to 1) matches.

## Summary

| question | answer | § |
|---|---|---|
| draw skipped when? | never on its own; the no-applicant branch always adds rivals, draws 3, de-duplicates and registers all of them (type 5, counter 2); only the election is skipped when a human has eligibility 2 | 1 |
| computers added to existing applicants? | no, but last year's registrations are applicants this year, so a human normally faces 1–3 drawn computers plus rivals in a vote | 2 |
| type-5 creators | `0x4857F` (human, Schreibstube) and `0x675B9` (year-end registration); both counter 2; human: 2→1 at his turn end; computer: 2→1 in the year-end computer record pass after the cycle; all ≤ 1 deleted after the cycle; winner's deleted at appointment | 3 |
| score terms | favour + title; office bonus gated on same kind and holding an office; colleague term on same place; `rand(20)` for candidates in the rival snapshot, not for humans; no rival/bribe/blackmail/human term | 4 |
| other refusals | vote loss, privileged candidate, prison; no eligibility re-check, no minimum votes | 5 |

## Corrections to other docs

- `rules-03` §4.6: (a) rivals and the three draws are added and registered (type 5, counter 2) before the human-eligible test; the test only skips the election; (b) rivals come before the draws; (c) the `rand(20)` term is for candidates in the rival snapshot, not for human candidates; (d) applicant records of computer persons therefore exist during the following year and the Schreibstube shows them.
- `rules-03` §4.7 / `rules-64` §2.4: "computers never apply" is wrong as a rule for the heap: they do not apply in their turn, but the year-end registration gives them application records that live through the next year.
- `rules-40` §1: the skipped group still consumed the three `rand(pool)` draws and allocated the records for its first seat.
- `rules-44` §3: "Candidates drawn by the computer rule never set A" holds in the year they are drawn; in the next year they are ordinary applicants (still computer, so A is still unaffected by them).
