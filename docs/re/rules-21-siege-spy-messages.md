# Siege, relief and spy-evidence messages: who sees what, when

Confidence tags as in `rules-00-map.md`: **[C]** read at instruction level (register/stack arguments checked), **[L]** read from decompiler output or inferred from a byte sweep, **[G]** guess. Text addresses are linear addresses of CP437 strings; `%` codes in `rules-12` §1; `print` arguments are listed in consumption order. Rules (formulas, record changes) are in `rules-07` §6 and `rules-04` §3.2; this document only fixes the message/film timing and corrects `rules-15` §5.2/§5.4.

Record fields used here: type 25 siege / type 26 relief: `+1` owner, `+3` counter, `+4` city, `+5` u16 army. Type 6 spy contract: `+3` counter, `+4` target, `+9` active, `+0xA` notified, `+0xB` u32 evidence. `human(p)` = person `+0x10` high nibble nonzero. `P` = city population (city record `+0x0E`).

## 1. Call sites [C]

| caller | pass | records visited | window open while the handler runs |
|---|---|---|---|
| turn end `0x61A70`, record pass (jump table at `0x63680`, index = type − 6; type 25 → `0x638F6`, 26 → `0x63906`, 27 → `0x63916`, 6 → `0x63936`) | the human player's own turn end | that player's records only | the player's **turn-end report** ("Ereignisse", `win_open(30,110,520,290,0xA810)`, `rules-12` §2) |
| year end `0x66220`, record pass at `0x69E20` | once per year | records of every **computer** person (person iterator `0x433E0(0xFF,0xFF,0xFF,0)` = controller nibble 0) | the shared **year-end report** ("Ereignisse in Mittelland", banner 0xFF, `win_open(100,126,440,300,0xA810)`) |

Both passes decrement `+3` before calling the handler. Consequences for who sees a text: a text printed by the human branch of a handler is seen only by the owner (the besieger / relief owner) in their own report. A text printed by the computer branch lands in the year-end window, which every human watches. **The besieged city's players are never addressed**: neither handler looks at who has branches or offices in the city, so a human whose city is besieged by another human learns nothing from these handlers; if the besieger is a computer, the human reads it in the year-end report like everyone else.

Inside the handlers the human/computer branch is chosen by `human(owner)` (`test [person+0x10], 0xF0`), not by the pass; since the year-end pass only visits computer persons' records, "human branch" = turn end, "computer branch" = year end.

## 2. Siege handler `0x1F2A0` [C]

### 2.1 Every call, before any text
For every computer person x with office location (`+0x0F`) = city (iterator `0x433E0(city, 0xFF, 0xFF, 0)`, person order): `rand(5)` → `favour_decrease(owner, x, 1 + rand(5))`. This runs in the message-only call as well. (`rules-07` §6.3.)

### 2.2 Human besieger (owner's turn-end report)

| counter after decrement | what happens | text and arguments |
|---|---|---|
| ≥ 3 (the turn end of the turn in which the siege was started: 4 → 3) | text only, no roll | `0x9F51A` `[EXE text]` with (**army** `+5`, city, city). No sound, no film. |
| ≤ 2 (every later turn end) | `r = rand(100 − 10·p38 − 20·p67)`; success if `r < army·50/P` (unsigned) | see below |

Success (`r < army·50/P`):
1. Mission record (type 0, owner) with `+4 == 22`: `+5 += 1`.
2. `$V`; `sound(8)`; `win_close`, `items_clear`, icon reset, video mode 1; **`film(belager, 0, 0, 12)`**; `delay_ms(2000)` (the last frame stays 2 s); film closed; video mode restored; banner(**owner**, "Ereignisse"); flip; `win_open(30, 110, 520, 290, 0xA810)`; `$B`; flip. So the film plays **on the success call, not when the army arrives** (correction to `rules-15` §5.2).
3. `0x9F6C6` `[EXE text]` with (city); `menu_choice` → c ∈ {0, 1, 2}.
4. If c ≠ 1: `0x9F7CB` `[EXE text]` with (city); then `plunder(record)` (`rules-07` §6.4; RNG draws in order, checked: `rand(30)`, `rand(20)`, then `rand(40)` per computer person in the city).
5. If c ≠ 0: type-27 record created (owner, `+4` = city); **`rand(1000)`** → amount `+5` = `P + 100 + rand(1000)` (P after plunder when c = 2); **`rand(4)`** → counter `+3` = `3 + rand(4)`; `0x9F91E` `[EXE text]` with (**2·amount**, city, **year + counter**, amount); `cash_add(owner, 2·amount)`.
6. Always: `0x9F9E0` `[EXE text]` with (city); siege record deleted.

Failure (`r ≥ army·50/P`):
- If `P/5 > army` (unsigned): **`rand(100)`**; if `< 50`: `0x9F11A` `[EXE text]` with (city, owner, city); record deleted. The human besieger is named in the third person here (`%3N1` = owner).
- Otherwise: `0x9F5B5` `[EXE text]` with (city, **army·5**); `menu_choice`: 0 ("Ja!") → `0x9F69D` `[EXE text]` (no args), `cash_pay(owner, army·5)`, counter := 2; otherwise → `0x9F673` `[EXE text]` (no args), record deleted.

`0x9F0E7` ("dauert an") is never printed for a human besieger.

### 2.3 Computer besieger (year-end report, everyone sees it)

| counter after decrement | what happens | text and arguments |
|---|---|---|
| ≥ 3 (first year-end visit after creation; revenge case 7 creates with counter 4, `rules-09` §4) | text only | `0x9F08A` `[EXE text]` with (**army**, owner, city). No sound, no film. |
| ≤ 2 | `r = rand(100 − 10·p38 − 20·p67)`; success if `r < army·55/P` | see below |

Success: `$V`; `sound(8)`; window closed, items cleared, icon reset, video mode 1; **`film(belager, 0, 0, 10)`** (no 2 s delay); video mode restored; banner(**0xFF**); flip; `win_open(100, 126, 440, 300, 0xA810)`; `$B`; flip; `0x9F21D` `[EXE text]` with (owner, city, city, owner); `plunder(record)`; record deleted. The film therefore plays **before the conquest text**, not with `0x9F08A` (correction to `rules-15` §5.2).

Failure:
- `P/5 > army` and **`rand(100)`** `< 50`: `0x9F11A` with (city, owner, city); record deleted.
- Otherwise: `0x9F0E7` `[EXE text]` with (city, owner); counter := 2. No question, no payment.

### 2.4 Other texts in `0x9F08A..0x9FD21`
| text | where | args |
|---|---|---|
| `0x9F397` `[EXE text]` | siege start, privileges menu `0x5F080` (`rules-07` §6.1) | none |
| `0x9F3D5` "Welche Stadt wollt Ihr belagern?" | `0x5F080` | menu |
| `0x9F409` `[EXE text]` | `0x5F080` | see §6.1 |
| `0x9F4C5` `[EXE text]` | `0x5F080`, also reused by the relief start (`0xB6914`) | none |
| `0x9FA16` `[EXE text]` | `0x5F080` when no city can be chosen | none |
| `0x9FAAB` `[EXE text]` | tribute handler `0x200F0`, each countdown: print with (city, amount), `cash_add(owner, amount)`, record deleted when the counter is 0. Printed in whichever report is open: the human's turn-end report, or the year-end report for a computer owner (the text says "Ihr"/"Eure" even then) | (city, amount) |
| `0x9FAF4..0x9FBDD` 18 adjectives ("Blutleere", …) and `0x9FBE9` (concert invitation) | not siege texts | — |

RNG draws per siege call, in order: `rand(5)` per computer person in the city; then (counter ≤ 2) `rand(100 − 10·p38 − 20·p67)`; then either the plunder draws and, human only, `rand(1000)`, `rand(4)` (tribute), or `rand(100)` (collapse roll, only when `P/5 > army`). `rand(5)` and the tribute `rand(1000)` / `rand(4)` are game RNG (`0x36FF0`), like all the others.

## 3. Relief handler `0x1FA80` [C]

Every call: `S` = first type-25 record whose city `+4` equals the relief city (`rec_find(25, 0xFF, 0xFF)` then `rec_find_next`); the relief record is deleted at the end in every path.

### 3.1 Human relief owner (owner's turn-end report)
- No siege on the city: `0xA3D7D` `[EXE text]` with (city). Nothing else.
- Siege found: first `0xA3B25` `[EXE text]` with (city, besieger). Then, **no RNG draw**: success if `A_s + 10·s38 + 20·s67 < A_r + P/10 + 10·r38 + 20·r67` (`P/10` unsigned division; privilege lookups in the order r38, r67, s38, s67).
  - Success: `$V`; `sound(0x1E)`; window closed, items cleared, icon reset, video mode 1; **`film(entlager, 0, 0, 12)` twice** (open, play, close, open, play, close); `delay_ms(2000)`; video mode restored; banner(owner, "Ereignisse"); flip; `win_open(30, 110, 520, 290, 0xA810)`; `$B`; flip; `0xA3BA8` `[EXE text]` with (**P**, read before the film); `cash_add(owner, P)`; siege record deleted.
  - Failure: `0xA3C8A` `[EXE text]` with (besieger). **No film, no sound.**

### 3.2 Computer relief owner (year-end report)
- No siege on the city: nothing printed; record deleted.
- Siege found: `0xA3916` `[EXE text]` with (owner, besieger, city). Then **`x = rand(100)`** (added to the relief side), privilege lookups r38, r67, then **`y = rand(100)`** (added to the siege side), lookups s38, s67. Success if `A_s + y + 10·s38 + 20·s67 < P/8 + A_r + x + 10·r38 + 20·r67` (`P/8` = `shr 3`).
  - Success: `$V`; `sound(0x1E)`; window closed etc.; **`film(entlager, 0, 0, 10)` twice**, no delay; video mode restored; banner(0xFF); flip; `win_open(100, 126, 440, 300, 0xA810)`; `$B`; flip; `0xA39D3` `[EXE text]` (no args); siege record deleted. No reward for a computer relief owner.
  - Failure: `0xA396E` `[EXE text]` with (**relief owner**). No film.

The besieger (human or computer) is not notified by this handler in either case; a human besieger only notices that the type-25 record is gone (the next turn end's siege handler is not called).

## 4. Spy evidence "Informant" (turn end, type-6 handler `0x63936`) [C]

### 4.1 The 30 % roll
At the human's turn end, **before the record pass starts** (right after the loop at `0x63569..0x63624` over all persons, which holds the revenge rolls), one game-RNG draw: **`f = rand(100) < 70 ? 1 : 0`** (`0x63629`). The draw happens every human turn end, whether or not a spy contract exists, so it costs one `rand(100)` in the sequence between the revenge loop and the first record handler. `f` is a per-turn-end film gate: the film plays only while `f == 0` (30 %), and `f` is incremented after a film, so **at most one Informant film per turn end** even if several contracts qualify (their texts still print).

### 4.2 Per type-6 record (after the counter decrement)
1. `+3 == 0`: record deleted. Nothing printed.
2. `+3 ≤ 15`: `+9 := 0` (contract goes passive). Nothing printed.
3. `+3 ≥ 16`, `+0xA == 0` and `popcount(+0xB) > 2`: `0xC0CBC` `[EXE text]` with (**target** `+4`, the spied-on person, accusative); `+0xA := 1`. Then, if `f == 0`: `$V`; `sound(0x0D)`; `win_close`; `items_clear`; icon reset; video mode 1; **one** film chosen by CONFIG byte `0x273` (`0x86BC7`, the same switch as the intro, `formats.md` §3.1): 0 → `film(informnt, 0, 24, 10)`, else `film(informnb, 96, 100, 10)` (correction to `rules-15` §5.4, which has both films in a row); video mode restored; banner(**0xFF**); flip; `win_open(30, 110, 520, 290, 0xA810)`; `$B`; flip; `f += 1`.

So the text is shown **once per contract**, at the first turn end in which the evidence count exceeds 2 while the counter is still ≥ 16 (turn ends 1..4 after hiring or after a resume, `rules-04` §3.2). `+0xA` is set only here and zeroed only by record creation (`rec_new` zero-fills; a byte sweep of the code object finds no other write of `+0xA` on a record pointer) [L for the sweep], so a resumed contract does not re-notify. If the evidence count passes 2 only after the counter dropped below 16, the text never appears. The 30 % roll is independent of the evidence: it is drawn before anything is known about the records.

## 5. RNG order, one human turn end (the parts touched here)
1. blackmail-end loop (`0x63569`) over persons q in index order with favour(q → player) > 100 and alive: `rand(100)`; if `< 33`: text `0xC0C84` `[EXE text]` with (q), favour := `rand(30)`, then `rand(100)`; if `< 50`: revenge code 3 (`0x5B140`);
2. `rand(100)` film gate `f`;
3. record pass in heap order: per type-25 record the draws of §2; per type-26 record none (human); per type-6 record none.

Year end (`0x69E20`, computer persons in person order, records in heap order): per type-25 record the draws of §2 (K = 55); per type-26 record `rand(100)` relief side then `rand(100)` siege side (only when a siege exists on the city).

## 6. Corrections to earlier docs
- `rules-15` §5.2: the belager film is **not** tied to the arrival texts `0x9F51A` / `0x9F08A`; it plays on the successful conquest roll, before `0x9F6C6` (human) / `0x9F21D` (computer). The entlager film plays only on **success** (both owners); failure has no film and no sound. This resolves the [L] there.
- `rules-15` §5.4 and §0.5: only one of informnt/informnb plays, chosen by CONFIG `0x273`; the 30 % is a single per-turn-end gate drawn before the record pass.
- `rules-07` §6.5: the tribute draws are `rand(1000)` (amount) **then** `rand(4)` (counter).
