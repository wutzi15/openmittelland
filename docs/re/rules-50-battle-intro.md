# Battle intro captions, attack call-off, harvest lines (rules-50)

Source: `battle_main` `0x3EE60` (listing `0x3EE60..0x3FB40`), its caller `settlement_pre` `0x61410` (type-51 loop `0x6142B..0x61505`), turn start `0x4B900` (production page `0x4C331..0x4C77C`). Helpers: `caption(x, y, w, h, delay, fmt, …)` = `0x36C90` (`rules-15` §0.1, `rules-42` §2), `print` = `0x322C0`, `rand(n)` = `0x36FF0`, `randomize_from_time` = `0x37030`, `deploy` = `0x3DDA0`, `music` = `0x41E50`, `activate` = `0x389C0`, `menu` = `0x35B60`, `clock()` = `0x6AC50` (1/100 s). `P` = target property (96-byte record, `rules-00` §5), `A` = attacker = record owner `rec+1` (`0xDB7F4`), `D` = `P+2` owner (`0xDB7F5`). Tags: **[C]** instruction level, **[L]** decompiler or partial trace, **[UNVERIFIED]** inferred. Everything is [C] unless marked.

## 1. Intro captions

### 1.1 Texts

Four pointers per battle code at `0xB5FBC + 16·code` (line 1..4). Castle and camp rows:

| code | line | addr | id | text | args |
|---|---|---|---|---|---|
| 4 castle | 1 | `0x9C684` | 1897 | `$C%v%vBurg %o...$N` | (junk, junk, `P+1` location) |
| | 2 | `0x9C697` | 1898 | `[EXE text]` | (D) |
| | 3 | `0x9C701` | 1899 | `[EXE text]` | none |
| | 4 | `0x9C75E` | 1900 | `[EXE text]` | (A, troop word, `P+1`) |
| 5 camp | 1 | `0x9C7F5` | 1901 | `$C%v%vRäuberlager %o...$N` | as code 4 |
| | 2 | `0x9C80F` | 1902 | `$C%2N3 Räuber gehen gemächlich ihren Geschäften nach; aus der Schmiede ist …$N` | (D) |
| | 3 | `0x9C8DA` | 1903 | `[EXE text]` | none |
| | 4 | `0x9C978` | 1904 | `[EXE text]` | (A, troop word, `P+1`) |

- "Komando" is the original spelling.
- Line 1 pushes three args (`0x3F947..0x3F975`): byte `0xDC86D + 0x60·[EBX+2]`, byte `[EBX+0xA]`, `P+1`. For codes 4/5 the caller passes EBX = 0 (`0x614C8`/`0x614D9`), so the first two are reads through a null record pointer. `%v%v` swallows them. Only `%o` = `P+1` (location id) is shown: "Burg Hohenfels...".
- Troop word (line 4) = `0xB601C[k]`, k = 1 if the attacker's home property (`rec+2`) kind byte ≠ 2, else 0. Entries: 0 `Söldner` (`0x87448`), 1 `Räuber` (`0x87440`). A castle owner's attack says "Söldner". A camp owner's attack (the capture case) says "Räuber".
- Codes 0..3 (raids) use the same table with their own four lines (`0x9C07D..0x9C5AF`). Line 1 there is `… zwischen %o und %o …` with the two shipment-record args.
- There is no choice by time of day, owner, or `rand`. The lines are fixed per code.

### 1.2 Geometry and wait

- Lines 1..3: `caption(40, 180, 560, 160, 0, …)`. Line 4 and the call-off text (§2): `caption(40, 160, 560, 160, 0, …)`.
- `delay = 0`, so each caption waits for a click (`wait_click` `0x36720`, wheel cursor). It shows gold text over the field picture, centred by `$C`, with the text cursor at y = 2 in the box. The box is closed (erased) before the next caption.
- The field is already drawn when the captions show. It holds the **defender** deployment only, because the attacker is deployed after line 4.

### 1.3 Order inside `battle_main` for codes 4/5

1. `music_stop_if_lowmem` `0x41FF0`, battle field tables, **`randomize_from_time`** (`0x3EEA7`; `rules-07` §5.1).
2. `P = property at location rec+4 + 14` (`0x42A50`). Call-off roll: **`rand(200)`** (`0x3F1D0`). See §2.
3. Helper candidates (`rules-07` §5.3): for every type-53 record, then every type-54 record, while fewer than 20 are listed, draw `rand(100)`. The roll is drawn before the filter. Then `rand(n)` if n > 0, and `rand(n−1)` if n > 1.
4. Colours (no draws).
5. Defender deployment: `deploy` × 4 garrison slots, then the leader (training 87). No draws (`0x3D7C0..0x3E293` calls no `rand`).
6. `value := net_worth_goods(P)` (`0x44340`).
7. `music(0x1B, 4)`: **`rand(3)`** (`rules-39` §5).
8. `activate(…, BL = 1)` (draw-only). With BL ≠ 0 it jumps past the tactic choice (`0x38A5F`→`0x39070`→`0x393C8`) and the man step logic (`0x39431`→`0x3AED4`) to the frame tail. No game-rule draws [L: the frame tail has no running animations at this point].
9. The field is presented. Captions 1, 2, 3 follow, with no draws.
10. If the roll in step 2 called off the attack: the call-off caption, then return (§2). Otherwise: line 4, attacker deployment (`rules-07` §5.4, including the computer-attacker troop cap), ransom (`rules-07` §5.5), and so on.

Answer: the intro is three fixed captions from `0xB5FBC[code]` (castle: "Burg %o...", `[EXE text]`, `[EXE text]`), each at (40, 180, 560, 160), each waiting for a click. A fourth line at (40, 160, 560, 160) is either the attack line `0xB5FC8[code]` or the call-off text. There is no condition or random choice. The only arguments are `P+1` (place), D (genitive, `%2N3`), and for line 4 A plus Söldner/Räuber. Before the captions the RNG order is: clock reseed, `rand(200)` call-off, helper `rand(100)`s and picks, `rand(3)` music. The captions draw nothing. The attacker deployment and every other battle draw come after the captions.

## 2. Attack call-off ("Detection")

- Rule (`0x3F1C0..0x3F1FA`): `called_off = rand(200) <= P[+3]`. `P+3` is castle security (Sicherheit) or camp camouflage (Tarnung), start value 60. This gives `(P[+3] + 1) / 200`, so 30.5 % at 60. The roll is made only for codes 4 and 5, right after the target lookup.
- No strength comparison. Troop counts, garrison, value and helpers are not read. Attacker and defender being human or computer does not matter.
- The test happens after captions 1..3 (`0x3FA2F`: `code ≥ 4 && called_off`). The texts have no text id:
  - castle (code 4): `0xBB8AC` `[EXE text]` with (A). `%2N0` is A's genitive.
  - camp (code 5): `0xBB950` `[EXE text]` with (A).
- Box (40, 160, 560, 160), click wait. Then the battle buffer is freed (`0x1331E`), the window is closed (`0x30190`), and `battle_main` returns (`0x41E34`, void).
- Effects: none. There is no attacker deployment, ransom, helper bribe or side choice, loss, evidence (`Überfälle`), retaliation, premium, damage, or conquest. `settlement_pre` does nothing after the call (`0x614E8`). The type-51 record stays, and its troops (+5..+8) go back to the home garrison at A's next turn start (`rules-07` §4). In the capture: 250 men of each kind, unharmed and unpaid.
- RNG consumed by a called-off castle attack: `rand(200)`, the helper `rand(100)`s and picks, and `rand(3)`. All are drawn after the clock reseed.
- Other texts of the same kind come from a **different** decision: ransom (`rules-07` §5.5, `0x3FFB2..0x400DF`), on the fight path only, after line 4. Table `0xB7248`:
  - human attacker question: 3069 `0xAD431` `…soll der Angriff stattfinden?…`
  - computer attacker, offer 0: 3072 `0xAD549` `[EXE text]`
  - offer taken, then fight: 3071 `0xAD4F6` `[EXE text]`
  - ransom accepted: 3070 `0xAD497` `%1N0 entschließt sich, von einem Angriff abzusehen; die %s ziehen sich zurück.` with (A, `0xB601C[k]`)
  - Undecided end (`rules-07` §5.6): `0xBBC64` `[EXE text]`.

Answer: the call-off is `rand(200) <= security/camouflage` of the target. It is the "Detection" line of `rules-07` §5.3 and involves no strength comparison. The roll comes before the helper draws and the deployment. The message comes after intro line 3, in place of line 4. Nothing else changes: the record is kept, and its troops return at the attacker's next turn start. Our sim must add this roll at this position and skip the rest of the battle (no premium).

## 3. Harvest-quality lines on the Erträge page

- Table `0xB628C[g]` (ids 2061..2075), one per good:

| g | text | g | text | g | text |
|---|---|---|---|---|---|
| 0 | `Die Kornernte war %s.` | 5 | `Die Rinderzucht verlief %s.` | 10 | `Die Salzgewinnung war %s.` |
| 1 | `Der Fischfang verlief %s.` | 6 | `Die Erzausbeute war %s.` | 11 | `Das Tuchspinnen verlief %s.` |
| 2 | `Die Bierproduktion war %s.` | 7 | `Das Weinkeltern verlief %s.` | 12 | `Der Silberabbau war %s.` |
| 3 | `Die Schafschur verlief %s.` | 8 | `Die Fellgerbung verlief %s.` | 13 | `Die Gewürzimporte waren %s.` |
| 4 | `Der Holzschlag war %s.` | 9 | `Die Glasproduktion war %s.` | 14 | `Die Schmiede arbeiteten %s.` |

  Each text is `$L` … `$N`. Quality words `0xB62C8[q]`: 0 `ausgezeichnet`, 1 `gut` (`0x9CB78`), 2 `normal`, 3 `mäßig`, 4 `schlecht`.
- `q = (59 − (W[g] − 70)) / 12` (signed division, `0x4C4D1..0x4C4F4`). `W[g] = 70 + rand(60)` is the weather byte rolled per good at this turn start (`0x4BF47`, `rules-02` §3). This gives: W 118..129 → ausgezeichnet, 106..117 → gut, 94..105 → normal, 82..93 → mäßig, 70..81 → schlecht.
- Condition: printed in the production row loop (`0x4C43D`), once for each good g (ascending) with **`total[g] > 0`**, while rows < L. Here `total[g]` is the sum of this turn's output of g over the 10 cities. L is 14 (11 if the overflow counter `0x85CC1` ≠ 0), minus 1 when more goods were produced than fit, to leave room for "Weiteres". If no good was produced, only `[EXE text]` appears, with no harvest line. Goods that go under "Weiteres" get no harvest line.
- The harvest line is **transient**. Per row:
  1. Back up a 280 × 48 strip of the window at the text cursor.
  2. Print the harvest line at the row position.
  3. Copy to the screen.
  4. Call `menu(…, 150, …)`. This waits until `clock()` has advanced 150 (1.5 s). A key press or a right click ends the wait early. A left click on empty space does not [L].
  5. Restore the text state and the strip, which erases the line.
  6. Draw the good icon `0x14A + g` at (x, y − 9) and print the normal row `$L       %s:$E$R%d$Z` (name, total).
  7. Draw `rand(3)` for skill +1..3 (`rules-02` §3).
  
  The harvest line itself draws no RNG.
- So the line sits where the good's row will appear, and the row overwrites it. The capture caught Fisch's row during its 1.5 s wait: Fisch had `total > 0` and W in 106..117. It does not mean failed or zero production [UNVERIFIED: depends on capture timing; the code allows no other reading].

Answer: `0xB628C[g]` is one line per good with `%s` = `0xB62C8[(129 − W[g]) / 12]`. It is shown only for produced goods that get a row, at that row's position, for up to 1.5 s (key or right click skips). Then the goods row replaces it. No RNG and no other arguments.
