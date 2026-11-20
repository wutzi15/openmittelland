# rules-69: debt stage 5 and the last branch, re-read at instruction level

Question: in a passive one-human run the original never seized the human's only branch over ~40 free years (jury and tower every free year), while `rules-02` §10.2 predicts a 50 % seizure roll per free year once `debt > m/2`. Re-read of 0x643A0..0x654C2 inside `turn_end_settlement` 0x61A70, plus `net_worth` 0x44640, `property_value` 0x44340, `residence_value` 0x44230, `clamp0` 0x441F0, `property_find` 0x42A50.

Tags: [C] instruction level, [L] decompiler only, [G] guess. Locals named by their frame offset: `cash` = person +0x18 (0xE3A94 + 0xF8·p), `debt` = [ebp−0xC8], `debt0` = [ebp−0xC4], `NW0` = [ebp−0xC0], `B` = [ebp−0xB4], `n` = [ebp−0xD0], stage `s` = [ebp−0x9C], current branch `x` = [ebp−0x124], `F` = [ebp−0xA8], `m` = [ebp−0x120], seized branch `xs` = [ebp−0x88].

## 1. Result in one line

The code is exactly what `rules-02` §10 says. There is no extra guard for a last branch, no cap or reset of `debt`, no `B >= 2` requirement for a seizure. The only thing that protects a last branch is the inequality `debt <= m/2`. If the original never seized, then in the original `m/2 >= debt` held in every free year, which means the inputs (`m` from 0x44340, `NW` from 0x44640) differ from the numbers the port computes. §5 gives the exact arithmetic to check against the original saves.

## 2. Trigger (0x643CC..0x64463) [C]

1. `cash >= 0` (signed dword compare at 0x643D7, `JGE`) → skip the whole debt block, jump to the death check 0x654A9.
2. `NW = net_worth(p)` (0x643F0). `t = (−NW) / 20` (`IDIV` after `SAR EDX,31`: truncating signed division). `t > cash` (signed `JG` at 0x6440B) → continue; else skip to 0x654A9. So the condition is `cash < 0 && cash < −NW/20`.
3. `NW` is computed **again** (0x64422, same function, same result) and `debt = (−NW)/20 − cash` (0x64437).
4. `debt0 = clamp0(debt)` (0x64449, 0x441F0 = `x < 0 ? 0 : x`), `NW0 = clamp0(net_worth(p))` (third call, 0x64459).
5. Text 0xC10D0 "%1N0, wegen Eurer Verschuldung …".
6. `B = 0` (0x6447D); `s = 0`.

`net_worth` 0x44640 [C]: `cash` + Σ `property_value(q)` over every property q with owner p of any kind (`property_find(0xFF, 0xFF, p)`, 0x42A50 masks kind/location out when 0xFF, then `property_find_next` 0x42B10) − Σ dword +5 of every type-0x38 record with owner p (loans). Iterator globals 0xEF490..0xEF498 are saved and restored around the loop.

`property_value` 0x44340 for kind 0 [C], all unsigned-widened byte/word reads, 32-bit sum:
```
city = 0x85F04 + 0x8C * location
v  = city.rights(+0x84, dword)
   + city.levelA(+0x6F, word) * wood(+0x23, word)
   + city.levelB(+0x71, word) * stone(+0x25, word)
   + residence_value(residence word +3, location)        (0 when the word is 0xFFFF)
   + Σ g=0..14  city.price[g](+0x22+g, byte) * stock[g](+5+2g, word)
   + Σ t=0..5   cond[t](+0x2D+t) * count[t](+0x27+t) * city.TP[t](+0x73+2t, word) / 100
   + Σ k=0..3 for order slot +0x33+8k with byte 7 == 2:
         if 1 < byte0 < 6:  qty(word +1) * city.price[byte3]
         if 1 < byte0 < 8:  cond[byte5] * byte4 * city.TP[byte5] / 100
   + city.population(+0x0E, dword)
```
Castles and camps have their own formulas (not relevant here).

## 3. Stage loop (0x64491..0x6449F, body 0x644AD..0x6540A) [C]

For `s = 0..5`: `n = 0`; `x = property_find(0, 0xFF, p)`; `F = index(x)` (computed once per stage from this first branch, never updated); if `x` is null → stage summary 0x6540A. Then per branch: dispatch through the table 0x644FC = {0x64535, 0x64546, 0x6466A, 0x64726, 0x64871, 0x649F9}, after the stage body `x = property_find_next` (0x653FA) and back to the null test at 0x644EC. After the loop the stage summary (0x6540A: stage 1 "Insgesamt %d Waren", stage 2 "%d Transportmittel", stage 3 "%d m² Lagerraum", only when `n > 0`), then `s += 1`. After `s = 5` → 0x6449A → death check 0x654A9.

Stages 0..4 are as in `rules-02` §10.2; re-read for this document:
- Stage 0: `B += 1` per branch (0x64535). `B` is never written anywhere else except `B −= 1` after a seizure (0x64C7F); the court reads it (0x65204).
- Stage 1: `q = debt·2/price + 1` (signed), `q = min(q, stock)` (0x645C4 `JGE` on `stock >= q`), proceeds `price·q/2`, `stock −= q`, `n += q`, `debt −= proceeds`. Zero stock gives zero proceeds, nothing underflows.
- Stage 2: `t` from 5 down to 0 while `debt > 0`; `count[t] == 0` → next `t` (0x64697 `JBE`), so no underflow. One transporter per iteration, `TP/2`.
- Stage 3: wood then stone while `debt > 0`: `q = debt/level + 1`, capped at `area·2/5` (signed, 0x647B6), proceeds `level·q/2`, `area −= q`. Note the cap: an area of 2 or less is never reduced (`2·2/5 = 0`), so a branch that only passes through stage 3 keeps **2 m²** of each kind, never 0. A save that shows 0 m² got there by another path (or the reading is of a different field).
- Stage 4: type-13 record for `F` → half its value, record deleted, residence word := 0xFFFF; else residence word != 0xFFFF → delete a type-14 record for `F` if any, half `residence_value`, word := 0xFFFF. Nothing else. `debt` is only ever decreased by proceeds; it is never clamped, reset, or recomputed after the trigger.

## 4. Stage 5, exact (0x649F9..0x64C85) [C]

Per branch `x`, in this order:

1. `debt > 0`? (signed `JG` at 0x64A00). If **not**: `rand(50)` (0x64A07); result 0 → skip this branch (next branch, no court for it); otherwise continue.
2. Save the iterator globals 0xEF484..0xEF48F. `xs = x`, `m = clamp0(property_value(x))` (0x64A41/0x64A46). Then for every later branch `y` of p (0x42B10 from the outer iterator, so slot order, kind 0, owner p): `v = clamp0(property_value(y))`; if `v < m` (signed `JGE` skips when `v >= m`): `xs = y`, `m = v`. Restore the iterator globals. So `m` is the minimum over this branch and all branches after it, and `xs` the first branch holding that minimum. No RNG.
3. `B > 1` (signed `JG` on `CMP B,1` at 0x64AB8) → go to the roll (step 4).
   Else (`B <= 1`): `h = m/2` (signed, the `2m; sar/shl/sbb; sar 2` idiom gives `m/2` for `m >= 0`); `h >= debt` (signed `JGE` at 0x64ADC) → **jury court** (0x64AEE → 0x64AF9 → 0x64C8A), no RNG. Else fall into the roll.
4. `r = rand(100)` (0x64AE3). `r < 50` (unsigned `CMP AX,0x32; JC`) **and** `debt > 0` (0x64AF0, signed) → **seize** (step 5). Otherwise (`r >= 50`, or `r < 50` with `debt <= 0`) → jury court.
5. Seize `xs`: `y` = first branch of p in slot order that is not `xs` (0x64B18/0x64B3C), may be null. If `y` exists: every type-9 record of p with key byte 2 == `F` gets key := index(y) and, for kinds other than 4 and 5, its transporter count (+8) is summed into a counter. Texts 0xC11C8 "Eure Niederlassung in %o." (location of `xs`), then 0xC11E8 `[EXE text]` only if the counter > 0, then 0xC1218 "$|". `add_cash(p, m)` (0x64C58, the full `m`), wait for click, `debt −= m`, `property_free(xs)` (0x64C74), `B −= 1`, next branch.

Entry points into the court block 0x64C8A: only 0x64AF9 (from 0x64AEE and from the roll). No stage 0..4 code and nothing outside the stage loop jumps there. The court itself is as in `rules-02` §10.3; re-read: `pg = (debt0 − bribe)·100 / max(1, NW0) + 50`, clamped at 0 from below and unbounded above; with `debt0` of the order of `NW0` every juror votes guilty with probability 1. Convicted with `B < 2` the text is 0xC13A8 `[EXE text]` (with `B >= 2`: 0xC1320). That sentence is the designer's intent; the code implements it only through step 3.

RNG draws in stage 5 per branch, in order: [`rand(50)` only if `debt <= 0`], [`rand(100)` only if `B > 1` or `m/2 < debt`]. Then the court's draws (`rules-02` §10.3: `rand(4)`, then 12 × (`rand(12)`, `rand(100)`, `rand(3)`)), or the seizure (no draws).

After a seizure of the last branch the loop runs out, 0x6449A jumps to 0x654A9, which does `property_find(0, 0xFF, p)` (0x654B5); null → the ruin screen (picture 0xBA, music 0x0B, `rand(4)`, …). So "seize the last branch" is ruin in the same turn [C for the test, L for the screen].

## 5. What must hold for 40 juries in a row

With one branch and no loans: `NW = cash + m`, `debt = (−NW)/20 − cash = (m − |cash|)/20·(−1) − cash`. Writing `c = −cash > 0` and ignoring truncation: `debt = c − (m − c)/20 = (21c − m)/20`. Jury without a roll needs `m/2 >= debt`, i.e. `10m >= 21c − m`, i.e.

```
m >= 21·|cash| / 11   ≈ 1.91 · |cash|
```

(exact: `floor(m/2) >= floor((−(cash+m))/20) − cash` with C truncation). With the lead's numbers: 1630 needs `m >= 10 300`, reported 12 718 → jury. 1650 needs 23 630, reported 17 294 → a roll. 1690 needs 50 800, reported 31 753 → a roll. Twenty-plus rolls without a seizure has probability below 1e-6, so the reported `m` (or `cash`) cannot be the original's. Candidates, in order of likelihood:

1. `property_value` of the original branch is larger than the port's formula gives. Every term is listed in §2; the two that grow without any player action are `city.population` (×(1000 + rand(50))/1000 per year, `rules-01` §3.3) and `city.rights` (+rand(200) per year). Over 90 years Lichterheide's 2 500 inhabitants become about 22 000 and rights 5 000 become about 14 000. Storage at 2 m² each adds 2·levelA + 2·levelB. Confirmed order slots (byte 7 == 2) add goods and transporters that are not in stock; a passive human has none, but a slot left over from setup would count. Check the save bytes: population dword at `0x85F04 + 0x8C·3 + 0x0E`, rights at `+0x84`, and the branch's 96 bytes.
2. `cash` in the original is less negative than read, or the sign/width of the read is off (dword at person +0x18).
3. A second property of p (any kind, e.g. a slot whose owner byte happens to be p) raises `NW`, lowers `debt`. `net_worth` iterates **all kinds**; `B` and `m` count only kind 0. Scan all 192 slots for owner byte == p.
4. Loans (type 0x38) lower `NW` and raise `debt`; they cannot explain fewer seizures.

A quick discriminator from the saves: in any free year compute `m` and `debt` as in §2 and §4 from the year-start autosave of the original; if `m/2 >= debt` in every free year, the mechanism is confirmed and the port's inputs are wrong. If not, the premise "never seized" must be rechecked (a seized last branch shows as the ruin screen and the end of that human).

## 6. Side findings

- `rules-02` §10.2 stage 3: add the floor: storage area never drops below 2 m² through seizure alone.
- `rules-02` §10.1: `net_worth` is evaluated three times in the trigger (0x643F0, 0x64422, 0x64459); no state changes between them, so the value is the same, but a port that logs calls will see three.
- The seizure's `property_free` is the debt caller listed in `rules-68` §1; it is reached only through step 4 with `debt > 0`.

## 7. Follow-up: the roll is fair; the run's evidence is survivorship [C unless marked]

The lead verified the inputs (one slot of any kind owned by p, `net_worth` equal to the original's cached +0x1C, `debt > m/2` from 1640 on, court screen in every free year). So the question became the generator and the harness.

### 7.1 Generator

- The roll at 0x64AE3 is `random_n(100)` = 0x36FF0, the same wrapper as every other draw (100 call sites, `rules-01` §1). 0x36FF0: `rand()` (0x6B491) then `IDIV` by the 16-bit argument, remainder returned; `rand()` is the Watcom LCG `S = S·0x41C64E6D + 0x3039`, result `(S >> 16) & 0x7FFF`. No masking or shifting at the call site: `CMP AX, 0x32; JC` on the remainder 0..99. A fair 50 %.
- Seed storage 0xB9284, initial value **1** in the image. The only writer is `srand` 0x6B4B5, whose only caller is `randomize_from_time` 0x37030, whose only caller is `battle_main` 0x3EE60 (0x3EEA7). There is **no clock seeding at program start** (xrefs; the other INT 21h AH=2Ch sites 0x107E8, 0x107F0, 0x12DBB, 0x14F5E, 0x6B683, 0x6C0AB are the timers of `rules-01` §5 and do not touch 0xB9284). This corrects `rules-01` §1 item 1 [C for the reference scan, L for "no indirect path"].
- The RNG state is not part of the save file (`formats.md` §3.2). Hence: **program start + "Das letzte Spiel fortsetzen" replays the loaded year with S = 1 every time.** Two loads of the same autosave with the same clicks produce the same world until a tactical battle reseeds.
- No reseed exists between the start of a human's turn end and stage 5. Draws before the roll in a passive turn end: sub-plot starter `rand` at 0x64116 only with option bit 2 of 0x86AD1 (Abenteuer); rival grudge `rand(100)` at 0x64148; `rand(100)` at 0x642FD (`< 25` → 0x5B140); then duels/trials of p (none); stages 1–4 draw nothing (no stock, transporters, storage above 2 m², residence); stage 5 step 1 draws nothing while `debt > 0`. So the roll is the third or fourth draw of the turn end, and with a fixed start state its value is fixed.

### 7.2 Same turn

The court block 0x64C8A is entered only through 0x64AF9, i.e. from the `m/2 >= debt` test or the roll in the same stage-5 pass. The court texts 0xC121C and 0xC12F4 are referenced only from 0x64D41/0x64D46 and 0x64DFE/0x64E03 inside this block; there is no second court screen and no unconditional court for a person who sat in the tower. The type-20 tower record only skips the next turn.

### 7.3 What the long run really did [O: `longrun/orig/orig.log`, `crash-*.png`; C: `tools/orig/longrun.sh`]

`longrun.sh` restarts DOSBox from the last autosave ("Das letzte Spiel fortsetzen") whenever the DOSBox process is gone, and logs it as `crash N after auto-K`. The log has 38 such restarts; from `auto-33` (1633) on they come after auto-33, 34, 35, 40, 43, 44, 45, 46, 47, 54, 57, 58, 67, 70, 73, 74, 81, 82, 85, 88, 89, 94, 95, 96, 101, 110, 111, 112, 119, 120, 121, 126, 131, 132, 135, 136, 139, 142: about every second free year. `crash-20.png` is the **credits screen** (Projektleitung …), i.e. the game ended normally, which is what follows the ruin screen of 0x654A9 when the last human has no branch.

So the original **did** seize the last branch in roughly half of the free years. Each time the game ended, the harness reloaded the year-start autosave, the LCG restarted at S = 1, the replayed turn end drew a fixed sequence, and that fixed roll happened to be ≥ 50 → jury → tower → the run survived into the next autosave. The year-start autosaves therefore only ever show years in which either the first attempt or the deterministic replay ended in a jury; the seizures are in `crash-*.png`, not in the saves. There is no bias in the game. The port's bankruptcies by 1630..1650 are the original's behaviour.

### 7.4 Consequences

- `rules-02` §10.2 stays as written. `rules-01` §1: remove "once at program start" from the seeding list; the game starts every session with S = 1 (world generation of a new game is seeded only by the clock-dependent number of draws made before it, e.g. title-screen frames, if any; not traced).
- Long-run harness: a DOSBox exit after the turn-end confirm box is a game over, not a crash. Count it as "human ruined in year K" and stop or start a new seed; restarting from the autosave replays the year with a fixed RNG and silently filters the outcome.
- Any A/B statistic taken from the original's year-start autosaves after a restart is conditioned on survival. The computer-branch counts of `rules-68` are unaffected (they do not end the game), but anything that can end the game is.
