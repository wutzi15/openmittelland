# Back room, bribery, spies, blackmail, denouncing (laws and trials: partial)

Status: the actions started from the office tree (`office_tree_view` 0x45C30) are decoded. Crime detection (how evidence bits get set), the law flags, trials (0x289E0), verdicts (0x2BD05), privileges (0x5E600) and sabotage execution are NOT yet decoded. Confidence tags as in `rules-00-map.md`.

## 1. Person standing (0x17430) [C]
```
standing(p, k) = (clamp(net_worth(p), 100, 999999) * k / 1000) * (70 + rand(60)) / 100 + 1
```
Each call rolls the RNG once. Used for prices of spies/saboteurs (k = 10) and for loan offers (k = 40 and 30, economy doc).

## 2. Prices in the office tree [C, instruction-level check]
When the office tree opens, a price is computed for every existing person q, in index order:
```
mult = 10; if player has privilege 47: mult = 7; if 73: mult = 6; if 78: mult = 5     (checked in that order, the last match wins)
price[q] = ((standing(player, 10) + 100 + standing(q, 10)) * mult) / 10       (unsigned)
```
`standing(player, 10)` is re-rolled for each q (2 RNG calls per existing person). Privileges 47, 73, 78 are the spy discounts; their texts promise 30 %, 40 % and 60 %, but the multipliers 7, 6, 5 of 10 give 30 %, 40 % and 50 % (text/code disagreement, code wins); which offices grant them is in `rules-04b`. (Corrected: an earlier version gave 7 as the default.)

## 3. Actions

### 3.1 Bribe (Bestechen) [C]
Allowed only with cash > 0. The player enters an amount X (> 0):
1. Player pays X; the target receives X (`person_add_cash`).
2. `gain = X * 400 / max(100, net_worth(target))`, capped at 100.
3. If briber and target differ in gender (person byte 0x00), `gain = gain * 11 / 10`.
4. `favour_increase(player, target, gain)`: halved if they differ in religion (H p.7 ✓ "same faith = bigger advantage"); no effect if the target's favour is already above 100; result capped at 100.
5. Sets the "bribed this turn" flag (bit 2 of `0x85CBA`) [L: input to crime detection].
6. If the player has a type-0 record (mission) with byte 4 == 6, X is added to its counter at +9 (mission progress "bribe a total amount", missions doc).

### 3.2 Hire spies (Spione) [C]
- No contract on the target yet: create type-6 record: owner player, byte 3 (years) = 20, +4 target, +5 (u32) yearly cost = `price[target] / 3`, +9 active = 1, +10 notified = 0, +0x0B (u32) evidence bits = 0.
- Existing contract: pause (+9 = 0) or resume (+9 = 1, years reset to 20); a paused contract without evidence can be cancelled.
- Yearly cost: the settlement charges +5 for every active contract (economy doc 7.7).
- Timeline [C]: hiring sets the counter to 20 during the turn. At every turn end of the owner, the costs (economy doc 7, step 7, charged only while +9 = 1) come first and the record countdown (step 16) comes later: counter -1, then the type-6 handler. Hence: turn ends 1..5 after hiring are charged (counter goes 19, 18, 17, 16, 15); at turn end 5 the counter reaches 15, which is below 16, so the handler sets +9 = 0 and no further cost is charged. The record (and its evidence) lives on until the counter reaches 0 at turn end 20, then it is deleted. Resuming sets +9 = 1 and the counter back to 20, which buys another 5 charged turn ends. Notification check in the handler, only while the counter is still >= 16 (i.e. at turn ends 1..4): if the player was not yet notified and `popcount(evidence) > 2`: message, +10 = 1, and with 30 % chance (`rand(100) < 70` skips) the "Informant" film plays. [L]
- Evidence count everywhere = number of set bits of the 32-bit evidence word.

### 3.3 Hire saboteurs (Sabotage) [C]
Type-7 record: years 5, +4 target, +5 cost = `price[target]` (charged at every turn end while the record exists). Execution is in the year end (not decoded).

### 3.4 Blackmail (Erpressung) [C, with a quirk]
Requires: target's favour toward the player <= 100 (not already blackmailed), a spy contract on the target with evidence count >= 3.
Then the outcome is decided by bit 1 of the low byte of the target's cash (person +0x18): set = success, clear = failure message. [C at instruction level: `TEST byte [person+0x18], 2`. This is effectively a 50 % roll keyed to the target's cash parity; it is almost certainly a bug for a flag test, but it is what the original does.]
Success: target's favour toward the player is set to 101 (the blackmail marker, career doc section 6), the spy contract is consumed (deleted), and a type-0 mission record with byte 4 == 6 gets +1 at +5.

### 3.5 Denounce (Anschwärzen) [C]
Choose a victim v, then a listener l. Per turn the game remembers per listener (`0x85D46[l]`, cleared at turn start):
- first attempt on l this turn: needs a spy contract on v with a nonzero evidence word. Success if `rand(100) < favour(l→player)` and the contract exists:
  - v's flag +0xF7 = 1 (forces the dismissal check of career doc 4.3 at this year end, even without a wealth loss);
  - `favour_decrease(v, l, rand(20))` (l likes v less), `favour_decrease(player, l, rand(10))` (l likes the player a little less);
  - state 1.
  Failure: `favour_decrease(player, l, rand(30))`, `favour_decrease(player, v, 10 + rand(10))`, and with 30 % v retaliates (0x5B140 code 4). The listener state is set only on success.
- second attempt on the same l in the same turn: message, state 2.
- third: `favour_decrease(player, l, 1 + rand(7))`, state 3.
- later: `favour_decrease(player, l, 10 + rand(20))`.

### 3.6 Set two persons against each other [C; which UI entry leads here is G]
Mode 9 then 10 of the office tree: choose a and b (b != a):
`favour_decrease(a, b, 20 + rand(20))`, `favour_decrease(b, a, 20 + rand(20))`; with 50 % both a and b retaliate against each other (0x5B140 code 4 both ways); with 20 % each a and b retaliate against the player (code 3), two separate rolls.

### 3.7 Other office-tree actions [L]
- Mode 5: type-18 record (target, flag 0): allowed if the target's office level is not above the player's (principality targets need a principality player, county targets a county-or-higher player). [G: start a trial / "Prozess machen"]
- Mode 7: type-18 record with flag 1, sets bit 3 of `0x85CBA`. [G]
- Mode 6: type-22 record (target), at most one per player; for a computer target 2 RNG calls (text choice). [G: insult / duel challenge]

## 4. Law book texts (0x48690 .. 0x49010) [C, texts only]
Three books (Finanz-, Straf-, Kirchengesetze). Recognised law subjects from the strings: loans, largest turnover tax (percent), bribery, wages (Hungerlöhne), raids, espionage, sabotage, sieges, religious freedom, indulgences, mistresses, feasts, church tax. The law state lives in the `0x85A20` block (`0x85A20 .. 0x85A41`, mostly 0/1 flags plus tax percentages); per-flag meaning not yet tied.

## Open questions
1. Crime detection: where evidence bits are OR-ed into spy contracts (year end) and which bit is which law.
2. Trials, verdicts, punishments and their probabilities.
3. Ability/privilege codes (0x5E600): which office grants which code (73, 78, 11, 44, 57, 36, 65 seen so far).
4. Sabotage execution and insurance payout.

## 5. Refusal and outcome messages of the office-tree actions (follow-up) [C]

All texts come from `0x45C30`'s action switch (`0x46A40..0x47BC0`). Every box is `msgbox 0x368E0(x, y, w, h, fmt, …)`: a framed window of style 0x8810 with an OK item. It waits for a click on the OK glyph or a `%a` line, and returns the line index (`rules-15` §0.4). No sound plays except `menu`'s click sample. `P` = the player, `T` = the clicked person, `L`/`V` = listener/victim. `fav_dec(a, b, d)` = `0x436F0`.

1. **Bestechung** (`0x46A79`).
   - Cash `P[+0x18] ≤ 0`: `0xBC5CC` `[EXE text]` (T), box (160, 160, 320, 160). Nothing else happens.
   - Otherwise `win_open(160, 120, 320, 240, 0x8810)` with `0xBC618` `[EXE text]`, args (T, style 0x27, &X preset 0, min 0, max = cash). The loop runs while item 1 (the spinner) is clicked.
   - **Amount 0**, or any answer other than 0 (a right click): the window closes **without a message** (`0x46B54..0x46B64` → `0x46C6A`). Nothing is paid, there is no favour change, no RNG, and the flag `0x85CBA` is not set.
2. **Erpressung** (`0x46C7F`).
   - Already blackmailed (`fav(T→P) > 100`): `0xBC678` `$N$CIhr erpresst %4N1 schon.$N` (T), box (160, 180, 320, 120).
   - Fewer than 3 evidence bits (no type-6 contract of P on T counts as 0): `0xBC698` `[EXE text]` (T), box (160, 160, 320, 160). No other effect and no RNG.
   - Enough evidence but bit 1 of `T[+0x18]` is clear: `0xBC760` `[EXE text]` (T), box (160, 140, 320, 200). No other effect; the contract stays.
   - Success: `0xBC6CC` (T, T, T), same box. Then mission +5 += 1, fav := 101, and the contract is deleted (bit 7).
3. **Anschwärzen** (listener step, `0x477F3`; state `s = 0x85D46[L]`):
   - L = V: L is reset and the choice is repeated, with no message.
   - s = 1 (L already believed P this turn): `0xBCE3C` `[EXE text]` (L, P), box (160, 160, 320, 160). s := 2. No favour change, no RNG. **This is the "already heard you" text.**
   - s = 2: `0xBCEA8` `[EXE text]` (L, P), box (160, 160, 320, 160). Then `fav_dec(P, L, 1 + rand(7))`, s := 3.
   - s = 3 (every later try): `0xBCEFC` `[EXE text]` (L), box (160, 140, 320, 200). Then `fav_dec(P, L, 10 + rand(20))`.
   - s = 0: look for P's type-6 contract on V with a non-zero evidence word. Then **`rand(100)` is always drawn**, and success requires `rand < fav(L→P)` **and** the contract.
     - Success: `0xBCF60` `[EXE text]` (L), box (160, 180, 320, 120). Then `V[+0xF7] := 1`, `fav_dec(V, L, rand(20))`, `fav_dec(P, L, rand(10))`, s := 1.
     - Failure: `0xBCF98` `[EXE text]` (L, V), box (160, 160, 320, 160). Then `fav_dec(P, L, rand(30))`, `fav_dec(P, V, 10 + rand(10))`, and if `rand(100) < 30`, `retaliate 0x5B140(V, P, 4)`.
   - **No evidence has no text of its own.** It is the failure branch above, with its penalties and draws (rand(100), rand(30), rand(10), rand(100)).
4. **Klage einreichen** (mode 5, `0x4732D`).
   - An open trial of P against T already exists: `0xBCB04` question (drop / keep), box (160, 140, 320, 200). Answer 0 deletes it.
   - Office check on the office byte `+0x0F` (`0xE3A8B`): refused if (`T ≥ 13` and `P < 13`) or (`T ≥ 10` and `P < 10`). Text `0xBCB7C` `[EXE text]` (T), box (160, 160, 320, 160). No other effect, no RNG.
   - Otherwise a type-18 record is created (+4 T, +5 0) with `0xBCBE0` `[EXE text]` (T), box (160, 180, 320, 120).
   - The inquisition variant (mode 7) has no office check: `0xBCCD4` existing / `0xBCD6C` new.

Reachability: every refusal above is tested inside the action handler, after the menu item was chosen. So each case can be reached whenever the office-tree menu offers the action for that person [UNVERIFIED: whether the menu builder hides Bestechen at cash ≤ 0 or Klage for higher offices was not traced].
