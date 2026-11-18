# Trial scene (rules-18)

Source: `trial_main` 0x289E0..0x2C09A and its tables. Rules (court composition, investments, scores, votes, sentence, costs) are in `rules-04b` §3; speech files in `rules-16` §2; this document gives what the player sees and does, step by step, with coordinates, ids and waits. Confidence: [C] unless marked. Notation: `d` = defendant, `q` = plaintiff, `g(x)` = gender byte of x (0 male, 1 female), `hum(x)` = x is human-controlled; `say_box(c0 c1 c2 c3 : n, x, y, w, h, text, args)` as in `rules-15` §0.2 (waits for the speech to end, or for a click when nothing plays); `caption(x, y, w, h, flag, …)` as in `rules-15` §0.1 (flag 0 = click, otherwise ms). All caption boxes are frameless text over the picture (`rules-16b` §3).

## 1. Entry and the hall [C]

Called from the human plaintiff's turn end (phase-6 tail, after duels and the crime scan, one call per type-18 record of the player) or from the year-end computer-person record pass (`rules-04b` §3.1). Returns at once if the record counter is not 0.

1. Seats: judge, assessor 1, assessor 2 by `rules-04b` §3.2 (the two `rand` draws for a principality defendant happen here); empty seats are filled from the next seat, the seat after seat 2 is the plaintiff.
2. Court kind `k` = record +5 (0 trial, 1 inquisition).
3. Figure banks chosen now (drawn in §2):
   - judge: `k = 0` → **0xE9** (male) / **0xEA** (female); `k = 1` → **0xEB** / **0xEC**; gender = g(seat 0);
   - assessors: `k = 0` → **0xED** and **0xEE**; `k = 1` → **0xEF** and **0xF0** (fixed, not by gender);
   - defendant: **0xE5** / **0xE6** by g(d); plaintiff: **0xE7** / **0xE8** by g(q).
4. `$V` into the current report window (click if anything was printed); the report window's geometry (12 dwords at 0xD5000) is saved; window closed; items cleared.
5. Picture **0xC1** (hall stairs) at (0, 0); `music(0x10, 0)`; flip.
6. Captions, each `(20, 20, 600, h, 0)` (click):
   - 0xA8332 `[EXE text]` with (d, q, `T[0xB6194][loc(d)]` location name, `T[0xB6DEC][k]` = "angeklagt" / "dem Inquisitionsverhör unterzogen"), h = 120;
   - 0xA83A3 `[EXE text]` with (seat 0, seat 1, seat 2), h = 160;
   - 0xA8499 `[EXE text]` with (seat 0), h = 120;
   - only `k = 1`: 0xA83D4 `[EXE text]` with (d), h = 120.

## 2. Court room [C]

1. Icon cache reset and re-initialised; the six banks (0xC6 and the five figure banks) are loaded.
2. Bank **0xC6** frame 0 (B/ICONC6, 640×480 court room) at (0, 0).
3. Five figures, `bank_frame(x, y, bank, 0)`:

| figure | x, y | bank |
|---|---|---|
| judge (seat 0) | 126, 162 | 0xE9/0xEA or 0xEB/0xEC |
| assessor 1 (seat 1) | 60, 162 | 0xED or 0xEF |
| assessor 2 (seat 2) | 189, 162 | 0xEE or 0xF0 |
| defendant | 387, 223 | 0xE5/0xE6 |
| plaintiff | 235, 306 | 0xE7/0xE8 |

Each bank has 134 frames (`formats.md` §1.7). Frame use: 0 rest; 0..15, 16..31, 32..48 defence speech segments; 52..67, 68..83, 84..99 prosecution segments; 100..133 verdict; 49..51 unused. When a segment plays, **all five figures** step through the same frame numbers (§7, §9).
4. flip. A 620×160 off-screen frame `TOP` is allocated for backing up the caption area (10, 10, 620, 160).

The caption boxes of all later steps sit at `(20, 20, 600, 120)` unless stated; they cover the top of the room above the bench.

## 3. Investments (option "Gericht bestechen") [C]

Order: plaintiff side (s = 0), then defendant side (s = 1); the defendant's step is skipped for `k = 1`. `W[s]` witness money, `B[s]` court money, both 0 when the option is off.
- Computer side: `W` by `rules-04b` §3.3 (`rand(10)`, `rand(1000)` for the plaintiff; `rand(1000)` for the defendant), clamp 0..9999; caption `(20, 20, 600, 120, 0)` 0xAFCFF `[EXE text]` with (side person). Click.
- Human side: `win_open(100, 20, 440, 200, 0x8910)`, compositor over its own rectangle (frameless, `rules-16b` §3); speech `(0 2 2 2 : 914)` loaded and started (not waited for); print 0xAFD31 `[EXE text]` with (side person, style 0x14 → `&W[s]`, style 0x14 → `&B[s]`): two 4-digit spinners (0..9999, preset 0 [L]) and one bullet. Loop: `menu` returns item `c`; `c = 0` → `W += d`, `c = 1` → `B += d` (d = ±10^digit by the half of the line clicked, `rules-13` §1.3); any other item (the bullet) ends. Speech popped; items removed; window closed. Both amounts are paid at once (`pay(side, W)`, `pay(side, B)`).
- `K[s] = clamp(W[s] / 1500, 1, 6)` witnesses per side.

## 4. Reading of the charge [C]

1. `say_box(0 2 2 2 : 900, 20, 20, 600, 120)` 0x99C64 `[EXE text]`.
2. `say_box(0 2 2 2 : 901 | 902)` with text `T[0xB5C50][hum(q) ? 0 : 1]`: 0x99C82 `[EXE text]` (901) / 0x99CB9 `[EXE text]` (902) with (q). Geometry (20, 20, 600, 120) [L: passed through locals].
3. Evidence word `E` (`rules-04b` §3.4; `rand(30)` when there is no contract). For each 10-bit group `G = 0, 1, 2` (finance, criminal, church) with at least one set bit:
   - `c = rand(5)` connector index, `t = rand(3)` text index [L: whether the first group uses connector 0 " " and later groups `1 + c`];
   - `say_box(0 2 g(d) 2 : 907 − 2·G + more)` where `more` = 1 if the group has more than one bit, with `T[0xB5C58][t]` = 0x99CF0 `[EXE text]` / 0x99D3C `[EXE text]` / 0x99D84 `[EXE text]` with (d, `T[0xB5C64][·]` = " " / " desweiteren " / " darüberhinaus " / " außerdem " / " zudem " / " weiter ", `T[0xB5C7C][more]` = "folgendes %sgesetz" / "die folgenden %sgesetze" with `T[0xB5C84][G]` = Finanz / Straf / Kirchen);
   - for each set bit `b` of the group (0..9): `say_box(hum(q) ? 1 : 2, g(q), 2, 2 : 10·G + b, 20, 20, 600, 120)` with `T[0xB5B5C][10·G + b]` (the 30 law lines, `rules-16` §2). The plaintiff's voice reads the charges.
4. `speech_say(0 2 2 2 : 910 + g(d))` (blocking, no box) for 0x99E4F `[EXE text]` — the text is not printed, only spoken [L: no print call between the two sites].
5. Plea `p` (0..8):
   - computer defendant: `p = 5 + rand(4)` without contract, else `rand(9)`; `say_box(2, g(d), 2, 2 : 30 + p, 20, 20, 600, 120)` 0x99E7A `$C%1N1 meint: %s$N` with (d, `T[0xB5C9C][p]`).
   - human defendant: `win_open(80, 20, 480, 200, 0x8910)` + compositor; print 0x99E8D `[EXE text]` with (d, style 0xA0, `T[0xB5C9C][7]`): one bullet (item `it`) and a cycling string item (`it + 1`) whose label is set to `T[0xB5C9C][p]`, preset `p = 7` (">Nichts von alledem habe ich getan!<"); loop: `menu` → `c`; `c = 1` → `p = (p + 9 + d) mod 9`, label updated, loop; `c = 0` (bullet) ends. Then `speech_say(1, g(d), 2, 2 : 30 + p)` (the chosen plea spoken, blocking); items removed; window closed.
   Pleas `T[0xB5C9C]`: 0 >Ich gestehe Alles!< 1 >Ich bin schuldig.< 2 >Kann schon sein.< 3 >Also, das Meiste stimmt nicht.< 4 >Öh, na ja...< 5 >Ich weiß nicht, was Ihr meint.< 6 >Ich bin unschuldig...< 7 >Nichts von alledem habe ich getan!< 8 >Lüge, alles Lüge!<.

## 5. Option "Zeugen & Plädoyers" off (`0x86AD1` bit 0x40 clear) [C]
`TOP` receives a copy of (10, 10, 620, 160) (always, before the test). With the bit clear: caption `(20, 20, 600, 120, 0)` 0xBB28C `[EXE text]`, click, and §6, §7 are skipped (no witness/plea RNG draws, no speech). Continue at §8.

## 6. Witnesses [C]

1. `say_box(0 2 2 2 : 915, 20, 20, 600, 120)` 0x968BA `[EXE text]`.
2. `side := 0`; `left[0] = K[0]`, `left[1] = K[1]` (for `k = 1` the defence has `left[1] = 1` as well, the inquisition note is cosmetic [L]). While `left[0] > 0` or `left[1] > 0`: if `left[side] = 0` skip to the other side; else one witness of `side`, then `side ^= 1`. So the sides alternate, starting with the plaintiff's witnesses, until both are exhausted.
3. One witness:
   - column: side 0 → print 0xBB2FC `$|$H$g$13>$40<$C` (home, gold, left margin 13 units = 78 px, right margin 40 units = 240 px: text spans x = 78..400, centred); side 1 → 0xBB310 `$|$H$g$40>$13<$C` (x = 240..562). The 64×64 talking head sits on the outer side of the column.
   - `line = rand(8)` (side 0) or `rand(9)` (side 1); `voice = 3 + rand(8)`; speech `(voice, 0, g(d), 2 : 39 + 8·side + line)` loaded and started; print `T[0xB58E4][g(d) + 2·line + 16·side]` (the witness sentence, `rules-16` §2); flip.
   - `head = 0x18E + rand(8)` (B/ICONOE..P5, 23 frames of 64×64); bank loaded; `x = 10 + 556·side` (10 or 566), y = 10; for `f = 0..22`: `bank_frame(x, 10, head, f)`, the 64×64 cell at (x, 10) is presented, `tick(12)` (120 ms; 2.76 s per head).
   - the area (10, 10, 620, 160) is restored from `TOP` (text and head vanish); `left[side] -= 1`; wait: `sound_idle()` already true → `delay(2500)`, else loop until idle; speech popped.
   RNG order per witness: line, voice, head.
4. flip; print 0xBB324 `$|`.

## 7. Pleas (Plädoyers) [C]

### 7.1 Prosecution
1. `say_box(0 2 2 2 : 916, 20, 20, 600, 200)` 0x955FB `[EXE text]`.
2. Four lines, `v = hum(q) ? 1 : 2`:

| line | roll | speech `(v, g(q), g(d), 2 : n)` | text | frames |
|---|---|---|---|---|
| 1 | `r = rand(7)` | 56 + r | `T[0xB5730][7·g(d) + r]` | 52..67 |
| 2 | `r = rand(6)` | 63 + r | `T[0xB5768][6·g(d) + r]` | 68..83 |
| 3 | `r = rand(4)` | 69 + r | `T[0xB5798][r]` | 84..99 |
| 4 | none | 73 | `T[0xB57B8][g(d)]` | none |

Per line: `music(0x10, 1)` (re-assert), roll, speech loaded and started, print 0xBB328 `$H$N$g$C` then the text (home, one line down, gold, centred: the line appears at the top of the court picture, y ≈ 36, full width), then for lines 1..3 the frame loop: for `f` in the range, all five figures drawn with frame `f` at their positions (§2), flip, `tick(15)` (150 ms; 16 frames = 2.4 s); then wait: idle → `delay(2500)`, else until idle; (10, 10, 620, 160) restored from `TOP`; speech popped. Line 4 has no frame loop: wait, restore, pop. Texts in `rules-16` §2.

### 7.2 Defence
1. `say_box(0 2 2 2 : 917, 20, 20, 600, 120)` 0xA15AC `[EXE text]`.
2. Three lines, voice 11 (`B`), `speech (11, g(d), g(d), 2 : n)`:

| line | roll | n | text | frames |
|---|---|---|---|---|
| 1 | `rand(3)` | 74 + r | `T[0xB6694][r]` | 0..15 |
| 2 | `rand(2)` | 77 + r | `T[0xB66AC][g(d) + 2r]` | 16..31 |
| 3 | `rand(2)` | 79 + r | `T[0xB66BC][g(d) + 2r]` | 32..48 |

Same per-line procedure as 7.1 (music re-assert, `$H$N$g$C` + text, frame loop with `tick(15)`, wait, restore, pop). The last line ends with a flip.

## 8. Bribes (option "Gericht bestechen") [C]

For `s = 0` (plaintiff) then `s = 1` (defendant), also for `k = 1`:
- computer side: `X` per `rules-04b` §3.5 (`rand(1000)` when applicable); `say_box(0 2 2 2 : 918 + 2·s + (X = 0), 20, 20, 600, 120)` with `T[0xB71EC][X = 0]` = 0xACD18 `[EXE text]` / 0xACD3E `[EXE text]` (side person).
- human side: `X = B[s]`; `say_box(0 2 2 2 : 922 + (X = 0))` with `T[0xB71F4][X = 0]` = 0xACD81 `[EXE text]` / 0xACDBD `[EXE text]` (side person).
- `pay(side, X)`; each of the three seats receives `X / 3`; `W[s] += X`.

Then the scores `P`, `D` (`rand(500)`, `rand(3000)` always drawn), `music(0x10, 1)`, `say_box(0 2 2 2 : 924, 20, 20, 600, 120)` 0xA5EE2 `[EXE text]`.

## 9. Verdict [C]

`nv := 0` (not-guilty count). For seat `j = 0, 1, 2`:
1. If the seat's person already sat in an earlier seat `i < j`: `verdict[j] = verdict[i]`, no dialog, no speech, next seat.
2. Blackmail: for every person `x` (slot order 0..191, used slots) with `favour(seat_j → x) > 100`: caption menu `(20, 20, 600, 200)` 0xBB334 `[EXE text]` with (x, seat_j); answer 0 → `a += 1`, answer 1 → `b += 1`. The menu is shown whoever x is (human or computer) and waits for a click.
3. `a ≠ b`: `verdict = (b > a)` (1 = not guilty); caption `(20, 20, 600, 120, flag = 2000)` 0xA5F1D `$C%1N3 stimmt für...$N` with (seat_j): shown 2 s, no click.
4. `a = b`, computer judge: `verdict = 1` if seat_j = d, `0` if seat_j = q, else `verdict = (favour(j→d)·100 + D > favour(j→q)·100 + P)`; the same 2 s caption.
5. `a = b`, human judge: caption menu `(20, 20, 600, 120)` 0xA5F5A `[EXE text]` with (seat_j); answer 1 → not guilty.
6. Voice: human judge → 1; computer → `12 + rand(4)`, 15 → 2. `say_box(voice, g(seat_j), 2, 2 : 80 + verdict, X_j, Y_j, 190, 30)` 0xBB3A4 `$0>$0<%s` with `T[0xB6BC4][verdict]` = 0xA5F34 `$C>Schuldig!<$N` / 0xA5F44 `$C>Nicht schuldig!<$N`; seat boxes `(X_j, Y_j)` = **(56, 151)**, **(0, 184)**, **(108, 127)** (table 0x84CF8 / 0x84D08: a 190×30 strip at each bench figure's head). `music(0x10, 1)`. `nv += verdict`.

Then: `NW = max(0, net_worth(d))` (for the fine); speech `(1, g(seat 0), g(d), 2 : 84 + (nv ≥ 2))` loaded and started; print `T[0xB6BD0][nv ≥ 2]` = 0xA5F9E `$H$g$CDamit ist %1N1 überführt.$N` / 0xA5FC0 `$H$g$CDamit ist %1N1 freigesprochen.$N` with (d) at the top of the picture; frame loop 100..133 for all five figures with `tick(10)` (100 ms; 3.4 s); (10, 10, 620, 160) restored from `TOP`; flip; wait: idle → `delay(2500)`, else until idle; speech popped; `TOP` freed.

Convicted iff `nv < 2`.

## 10. Sentence (convicted only) [C]

1. Choice `s`:
   - computer seat 0: `s = rand(0x85A4D / 21 + 1) + n − 1` etc. (`rules-04b` §3.5).
   - human seat 0: `win_open(100, 20, 440, 200, 0x8910)` + compositor; print 0xA600D `[EXE text]` with (seat 0, style 0xA0, `T[0xB6DB0][2]`): bullet + cycling string item preset to 2 "Mehrjährige Geldstrafe"; click on the string item → `s = (s + 7 + d) mod 7`, and if d has no title (`+0x0D = 0`) and `s = 3` the step is applied again (so "Entzug des Titels" is skipped); label updated; the bullet ends. Items removed, window closed. List `T[0xB6DB0]`: Pranger / Geldstrafe / Mehrjährige Geldstrafe / Entzug des Titels / Ein Jahr Schuldturm / Entzug des Amtes / Ein Jahr Kerkerhaft.
2. Voice of seat 0 re-rolled: human → 1, computer → `12 + rand(4)` (15 → 2).
3. Sentence applied (`rules-04b` §3.5 table) and announced with `say_box(voice, g(seat 0), 2, 2 : 86 + s, 20, 20, 600, 120)`, text `T[0xB6D94][s]`:

| s | file n | text | args |
|---|---|---|---|
| 0 | 86 | `[EXE text]` | (d, 7) |
| 1 | 87 | `[EXE text]` | (d, F) |
| 2 | 88 | `[EXE text]` | (d, 3, F·2/3) [L for the order of the last two] |
| 3 | 89 | `[EXE text]` | (d, old title name, new title name) via `0xB4258` |
| 4 | 90 | `…zu einem Jahr Schuldturm…` | (d) |
| 5 | 91 | `[EXE text]` | (d, d) |
| 6 | 92 | `…zu einem Jahr Kerkerhaft…` | (d) |

The record creations (types 19/20/21) happen between choice and announcement.

## 11. Costs and end [C]

1. Payer = d if convicted, else q. `C = max(F / 5, 100) + rand(100)`; `say_box(0, 2, g(payer), 2 : 925 + (payer = d), 20, 20, 600, 160)` 0xAFDCE `[EXE text]` with (C, payer); the payer pays `C`, each seat receives `3C/10`.
2. Insurance bit 1 of the payer: `say_box(0 2 2 2 : 927, 20, 20, 600, 120)` 0xAFE07 `[EXE text]` with (payer); `C` refunded.
3. Spy evidence cleared, retaliation roll `rand(100) < 80` (`rules-09`), trial record deleted.
4. Screen rebuild: human plaintiff (turn-end case) → picture **0xB7** and banner "Ereignisse" (0xBB3B0) for q; otherwise (year-end case) picture **0xB8** and banner for player 0xFF; flip; `win_open` with the saved geometry and style 0xA810; flip. The caller continues printing into that window. Music 0x10 keeps playing until the next `music` call (no stop).

## 12. Jury court bribe screen and juror markers (debt court, for cross-reference) [C]

Not part of `trial_main`; it is the creditors' court of `rules-02` §10.3 / `rules-15` §6.1 and shares the speech matrix. Verified here:
- bribe input: window `(10, 280, 620, 200, 0x8910)` with the compositor (frameless), text 0xC121C ending in `%a%m Taler`: bullet + 4-digit spinner (`item_range(it, 0, 9999)` [L]); spinner clicks add `d`, the bullet ends; window closed; `pay(p, X)`.
- marker table `0x87310` = [0x48, 0xA7, 0x179] indexed by juror state: state 2 = not yet voted → **0x179** (ICONN9, 31×32: a wooden spoked wheel, the breaking wheel); state 0 = guilty → **0x48** (ICON48, 48×48: a wine bottle with grapes, a trade-goods icon); state 1 = not guilty → nothing (the 32×32 cell of the backed-up strip is copied back, so the marker disappears). Markers at `(62 + 44·i, 420)`, i = 0..11, all twelve redrawn after every vote. Both icon contents were checked on the rendered sheets; the bottle is an odd choice but it is what the code draws.

## 13. What the sim must supply
Seats, `k`, `E` and `n`, `W/B/K` per side, plea `p`, bribes `X`, `P/D`, per-seat verdicts and `nv`, sentence `s`, `F`, `C`, payer, insurance, and the RNG sequence in this order: seat draws (principality only) → investments (computer sides) → evidence (`rand(30)` without contract) → per group `rand(5)`, `rand(3)` → plea (computer d) → per witness `rand(8|9)`, `rand(8)`, `rand(8)` (only with the option on) → prosecution `rand(7)`, `rand(6)`, `rand(4)` → defence `rand(3)`, `rand(2)`, `rand(2)` (option on) → bribes (computer sides) → `rand(500)`, `rand(3000)` → per computer judge `rand(4)` (voice; skipped for repeated seats) → sentence `rand(…)` (computer seat 0) → seat-0 voice `rand(4)` (computer) → `rand(100)` costs → `rand(100)` retaliation. The witness and plea rolls are presentation-only but consume game RNG.

## 14. Open
- Connector-word index for the first charge group and the geometry of the 901/902 box (passed through locals) [L].
- Whether 0x99E4F is printed anywhere (only `speech_say` was found) [L].
- Order of the two numeric arguments of sentence 2 [L].
- Inquisition: `left[1]` for the defence witnesses (1 by `K` clamp) [L].
