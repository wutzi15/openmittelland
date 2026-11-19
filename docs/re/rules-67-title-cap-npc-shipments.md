# Title cap after a dismissal, computer shipments (rules-67)

Answers to the two open items of `docs/IMPL-GAPS.md` §1 "For the RE role" after round 8 (`rules-66`). Conventions as in `rules-66`: `rand(n)` = `0x36FF0`, `has(p, viewer, code, print)` = `0x5E600` (EAX, EDX, EBX, ECX), `person_pick_random` = `0x43560` over the enumerator `0x433E0`/`0x434E0`, `global_office(p)` = `0x44050`, `office_vacate` = `0x45980`, `rec_new` = `0x424A0`. Person record `0xE3A7C + 248·i`; `+0x0D` title, `+0x0F` place byte, `+0x10` controller nibble (high) and church nibble (low), `+0x11` office index. Tags: **[C]** instruction level, **[L]** decompiler only, **[G]** guess.

## 1. Title cap after a dismissal (`year_end_world` 0x66220, per-person loop) [C]

Answer: the cap step re-reads everything after the dismissal. At `0x66889..0x668D0` the code reads the place byte `+0x0F` again (`CMP byte [p+0x0F], 0xD` at `0x66893`), calls `global_office(p)` again (`0x668AE`) and indexes `0x86FB8` with that result; the `o` loaded before the dismissal check (`[EBP-0x48]`, written at `0x6640A`) is not used here. It is used only by the human branch at `0x6691C` (`OFFICE_WEALTH[o]` update), which is what `rules-66` §1 step 6 and `career.rs` already do.

`office_vacate` (`0x45980`) writes `+0x11 = 15`, `+0x0F = 0`, clears the church nibble of `+0x10`, floors favour cells above 100 to 30, and redraws the 15 skills for a computer person. So a person dismissed in step 4 reaches step 6 with place 0 and office 15; `global_office` returns 0 for office 15 (`0x44050`: `+0x11 == 15 → 0`), and the cap is `0x86FB8[0] = 1`. The step then raises the title only when it is 0 (`CMP AL(cap), [p+0x0D]; JBE skip` at `0x668BE`), i.e. a dismissed title-0 person becomes title 1; everyone else keeps his title.

Consequences:
- A principality officer (place 13) dismissed in the same pass cannot reach title 10 or 11 through this step: the cap is read after the vacate, and the entries 10/11 (`0x86FB8[29]`, `[30]`, `[33]` = 0x0A, 0x0B, 0x0A) are only ever indexed by a person whose place byte is 13, which the `< 13` test rejects. Titles above 9 are unreachable from this step. The table: `01 01 01 02 02 02 03 01 01 03 | 04 04 04 05 05 05 06 07 04 04 05 05 06 | 08 08 08 09 09 09 0A 0B 08 08 0A` (34 entries, city, county, principality).
- The quirk that is reachable is the other way round: a dismissed person is treated as office-less (cap 1) in the same pass. It fires only for title 0, and title 0 does not occur (re-initialisation writes `1 + rand(2)`, the 1619 save holds titles 1..9 only, office-less persons 1..6). In practice the dismissed person's title is unchanged.
- No other clamp is involved; the step is the only writer of `+0x0D` in this loop.

So `career.rs` `office_year` (cap from the office after the check, `loc() < 13` on the post-vacate place byte) is the original's behaviour; the SPEC-GAP comment there can go. Corrects `rules-66` §1 step 6: `o` is re-derived (`global_office(p)` is called twice per office holder, at `0x66400` and `0x668AE`), the place byte is re-read; the step is "if `+0x0F < 13` and `title < 0x86FB8[global_office(p)]` then `title += 1`" with the post-dismissal record.

## 2. Computer shipments: case 16 is not an action [C]

Answer: `npc_act` case 16 (insurance toggle) does not end the try loop. The done counter (`[EBP-0x2C]`, `local_30`) is incremented in cases 0, 1, 2, 3 (every branch that acts), 4, 5, 6, 15 and after every shipment (`0x5B0E6`), and not in 12, 16, 17..23. Case 16 (`0x5AD82..0x5AE64`): `rand(100)`; if ≥ 40 and `has(25)`: `rand(2)`, toggle the insurance bit (`0x84B4C[2·r + 1]` = 2 or 8), message when a human shares the place; then `break` without touching the counter, so the next try follows. `ai.rs` returns `true` from case 16.

That is the whole gap. Draw-for-draw parse of the original's `npc_turns` block in the 1619 playtest stream (state `0x13bc5ed1` at load, draw numbers as in `docs/FIDELITY.md`; pools from the 1620 autosave, in slot order):

| draw | value | use |
|---|---|---|
| 3661 | 90 | county 10 `rand(100)`: idle |
| 3662 | 25 | county 11: act |
| 3663 | 12026 % 13 = 1 | pick → person 43 (pool 35, 43, 48, 51, 62, 66, 103, 113, 114, 115, 119, 123, 124) |
| 3664 | 87 | step A off |
| 3665 | 6 | case 6, `has(85)` false |
| 3666, 3667 | 16, 65 | case 16, `has(25)` true (office 8 at place 11) |
| 3668 | 0 | `rand(2)`: insurance bit 2 (43's `+0x07` is 0 before, 2 after, in the autosave), **not done** |
| 3669, 3670 | 14, 3 | case 14, good table `0x8723C[3]` = 4 |
| 3671..3674 | 23, 32320 % 49 = 29, 0, 3 | qty 100·63/100 = 63 → 100; source branch 29 (city 7); counter 2; destination (7+1+3) mod 10 = 1 |
| 3675 | 16 | county 12: act |
| 3676 | 25077 % 13 = 0 | pick → person 73 |
| 3677 | 28 | step A: `has(56)`, `has(59)`, `has(82)` false (office 2 at place 12) |
| 3678..3680 | 5, 17, 5 | nothing |
| 3681, 3682 | 13, 4 | case 13, good 5 |
| 3683..3686 | 8, 23065 % 49 = 35, 0, 4 | qty 42 → 100; branch 35 (city 6); counter 2; destination 1 |
| 3687 | 71 | principality: act |
| 3688 | 5781 % 11 = 6 | pick → person 142 |
| 3689 | 80 | step A off |
| 3690, 3691 | 20, 6 | nothing (`has(85)` false) |
| 3692 | 8 | case 8, good 7 |
| 3693..3696 | 61, 5366 % 49 = 25, 1, 4 | qty 73 → 100; branch 25 (city 1); counter 3; destination 6 |
| 3697 | 89 | the one type-6 record: idle |

37 draws, three records `t9 o43 k29 [02 64 00 04 01 00 01 5a]`, `t9 o73 k35 [02 64 00 05 01 00 01 5a]`, `t9 o142 k25 [03 64 00 07 01 00 06 5a]`, exactly the autosave's. With case 16 counted as done (ours), 43 stops at 3668; the county-12 roll then reads 3669 = 78 (idle), the principality picks 107 at 3671 and ships good 5 from branch 35 at 3683..3686; our parity run makes exactly that record (`t9 o107 k35 … 05 … 01`) and nothing else. That is the one-versus-three shipments of FIDELITY item 5.

Everything else re-checked against `ai.rs` matches draw for draw:
- `npc_turns` (`0x69BB8..0x69D83`): humans in slot order, office required, `rand(100) < 40`, pick at the human's place byte; counties 10, 11, 12 with `rand(100) < 50`; principality `rand(100) < 90`; type-6 records with byte 9 ≠ 0, human owner, computer target, `rand(100) < 50`. Same as ours.
- Pick (`0x43560`): arguments EAX place, EDX church nibble, EBX office, ECX controller nibble, 0xFF = any; the enumerator masks the dword at `+0x0F` and skips unused slots. `npc_turns` passes (place, 0xFF, 0xFF, 0) → every used computer person whose place byte matches, office or not, in slot order; one `rand(count)` whenever the list is not empty, also for a list of one (`rand` always steps the generator). Same as `pick_computer`.
- Step A: `has(56)` before `rand(100) < 80`, `has(59)` before `rand(100) < 90`, `has(82)`; `k` draws `rand(33)`, `rand(100)`, `rand(100)`, `rand(700)`, `rand(25)`; k = 8 nothing; a change ends the action. Same.
- `has()` for the codes `npc_act` uses (5, 8, 9, 16, 25, 27, 31, 32, 40, 53, 54, 55, 56, 58, 59, 61, 64, 69, 71, 82, 84, 85, 86): jump table `0x5E674`, keyed on the global office index `0x43FF0` (15 → 0), the place byte and the church nibble; no draw; `laws.rs` `privilege` agrees for all of them (code 25 is `place ≥ 10`, which with place 0 for office-less persons equals our `off && loc > 9`).
- Shipment: `rand(120)`, `property_pick_random(0, 0xFF, 0xFF)` = `rand(number of kind-0 properties)` over the property table in order (the 1619 save has 49), `rec_new` appends at the heap end (creation order kept), `rand(2)`, `rand(9)`; quantity `0x85E1D · (40 + r) / ((50·g) / 4 + 50)`, unsigned divide; ship conversion; toll accumulation; bit 0x20000 on counter 3 with `0x85A35` set. Same.
- Case 15: `rand(100)`, then `has(27)` and fencing < 80, then `rand(5)`, `rand(10)`, counted as done. Case 12: not done. 17..23: not done. Same.

Remaining differences to `ai.rs`, by weight:
1. **Case 16 is not an action** (above). The only one that changes the 1619 year end.
2. Case 3 (`has(58)`) and case 6 (`has(85)`): the original picks with (0xFF, 0xFF, 0xFF, controller 1), i.e. the persons whose controller nibble is exactly 1 (`0x5A77F`, `0x5AB71`); ours picks among all humans. Same with one human, differs in a multi-human game (the original never picks player 2's slot).
3. Case 6 clergy loop: the original enumerates (0xFF, church nibble 1, 0xFF, controller 0) (`0x5ABB7`): computer clergy only, one `rand(10)` each; ours includes a human cleric. Differs by one draw per human cleric.
4. Shipment floor: the original compares the low 16 bits of the quantity with 100 (`CMP word [EBP-0x34], 0x64` at `0x5AF16`) and stores the carts byte as `(qty16 + 99) / 100` truncated to 8 bits; ours floors the 32-bit value and clamps the carts at 255. Differs only for quantities ≥ 65 536 (average shipment `0x85E1D` above ~20 000).
5. Shipment without any branch: the original still creates the record (source index from a null pointer); ours returns after the `rand(120)`. Unreachable while a branch exists.
6. `npc_turns` type-6 loop: ours also requires the target slot to be used; the original tests the controller nibble only. No difference on a consistent heap.

Corrects `rules-09` §3 and `rules-66` §2: case 16 belongs with 12 and 17..23 (the try is used up, the loop continues); the pick arguments of cases 3/6 and the enumeration of case 6 are as in items 2 and 3. Corrects `rules-66` §2's closing paragraph: the 11-draw gap was not the source draw plus case 12 alone; the case-16 stop is the part that remained.
