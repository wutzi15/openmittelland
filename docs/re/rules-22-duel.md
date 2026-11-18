# Duel scene (rules-22)

Source: `duel_main` 0x2C750..0x2D747, the two wipe effects 0x2C140 and 0x2C460, `office_index` 0x44050, the turn-end call site 0x6431F and the year-end call site 0x69ED5. Rules summary in `rules-08` §3; speech rows in `rules-16` §2; sound tables in `rules-14` §2.4. Confidence tags as in `rules-00-map.md`: **[C]** read at instruction level, **[L]** decompiler reading, **[G]** guess. Everything below is [C] unless marked.

Notation: record `R` of type 22: `R+1` = insulter `A`, `R+4` = insulted `B`. `g(x)` gender byte (person +0x00, 0 male / 1 female), `hum(x)` = controller nibble set (person +0x10 & 0xF0), `life(x)` = person +0x02, `fenc(x)` = person +0xF6, `fav(x→y)` = person(x) +0x20 + y. `say_box(c0 c1 c2 c3 : n, x, y, w, h, text, args…)` as in `rules-15` §0.2 (speech started, box stays until the sample ends, click when nothing plays). `rand(n)` = game RNG 0x36FF0. `bank_frame(x, y, bank, k)` = 0x2E420 (draws frame `k` of a bank into the current window's surface, no frame-count check). `region_copy(screen, x, y, surface, x, y, w, h)` = 0x13539 copies a rectangle from the window surface `0xD500E` to the screen `0xEF4BA`.

## 1. Who sees it, and when [C]

| caller | who | window open when called |
|---|---|---|
| turn end, phase-6 tail of `turn_end_settlement` (0x6431F): `rec_find(22, player, 0xFF)` / `rec_find_next` loop; before each call `print("$V")` | every type-22 record whose **insulter is the current human player**, all in one turn end, no counter check | that player's turn-end report (picture 0xB7, "Ereignisse", `win_open(30, 110, 520, 290, 0xA810)`) |
| year end (0x69ED5), computer-person record pass (`person_iter(0xFF, 0xFF, 0xFF, 0)` = computer persons in person order, records in heap order, counter decremented before dispatch, not checked by the duel) | every type-22 record of a **computer insulter** | the shared year-end report (picture 0xB8, banner 0xFF, `win_open(100, 126, 440, 300, 0xA810)`) |

Position in the turn-end tail (`rules-04b` §3.1): sub-plot starter, the 25 % rival retaliation roll, **duels**, crime scan, trials, debt seizure. Position in the year end (`rules-19` §5): after the divine event 0x59940, inside the computer record pass together with trials (type 18), sieges (25), relief (26), mistress events (29), in heap order.

There is no human test in the routine. A human who is insulted by a computer person sees the duel at year end like every other player; a human insulter sees it at his own turn end; a human insulted by another human sees it only if he is watching that player's turn end. The routine does not check whether either person is alive or still has a record counter.

## 2. Setup [C]

1. `print("$V")` (click if the report has text); the current window record (12 dwords at `0xD5000`) is saved; `win_close`; `items_clear`.
2. Fight bank `F = T16[0x84D18][g(B) | 2·g(A)]` = **0xD9** (A male, B male), **0xDA** (A male, B female), **0xDB** (A female, B male), **0xDC** (both female); B/ICOND9..DC, 105 frames each over a 640×480 base frame (meadow under a stormy sky, mountains; seconds and a doctor with a red bag stand around; the insulter is the left, nearer duellist in tan, the insulted the right one in yellow, identified from the result frames). `icon_load(F)`; `bank_frame(0, 0, F, 0)`.
3. Samples: `sample_load(T8[0x84D30][i])` for i = 0..17: 0x83 0x39 0x3A 0x5F 0x31 0x3B 0x80 0x32 0x3C 0x81 0x3D 0x3E 0x82 0x3E 0x3F 0x83 0x40 0x41 (the swing/hit triples of `rules-14` §2.4; the 19th byte 0x53 is **not** preloaded).
4. `music(0x17, 0)` (song 0x17, no RNG draw); `flip`.
5. **`t = rand(11)`** time/place index.
6. Speech global `0x8457C := 0x8457C·11/10`, clamped to 255 (CONFIG byte 0x278; passed to the driver by `speech_play`; restored at the end from the CONFIG image `0x86BCC`) [C for the arithmetic, L for what the driver does with it].

## 3. Opening captions [C]

| # | box | speech | text, args |
|---|---|---|---|
| 1 | (40, 40, 560, 120) | (0 2 2 1 : 921 + t) | 0xA2B04 `[EXE text]` with (**X**, `T[0xB67AC][t]`, **Y**) where X = A and Y = B if `hum(A)`, else X = B and Y = A. The line is always addressed to the insulted person when the insulter is a computer, so at year end it reads as if the insulted had chosen the duel |
| 2 | (240, 380, 320, 100) | (0 2 2 1 : 932) | 0xA3FEA `[EXE text]` with (A, B) |
| 3 | (100, 180, 240, 120) | (1 + [g(A) = 0], [g(A) = 1], 2, 1 : a + 1) | **`a = rand(8)`**; `T[0xB6988][a]` (answers, `rules-16` 2021001..) |
| 4 | (250, 150, 240, 120) | (1 + [g(B) = 0], [g(B) = 1], 2, 1 : b + 1) | **`b = (a + 1 + rand(7)) mod 8`**; `T[0xB6988][b]` |
| 5 | (240, 380, 320, 100) | (1 + [g(A) = 0], [g(A) = 1], 2, 1 : 9) | 0xA40D5 `[EXE text]` |
| 6 | (240, 380, 320, 100) | same voice : 10 | 0xA40FE `$C>En garde!<$N` |

The place strings (t = 0..10): im Morgengrauen, um die Mittagsstunde, in der Morgendämmerung, zu früher Stund', des Nachmittags, beim alten Glockenturm, auf dem Friedhof, im Pfarrgarten, auf einer lieblichen Aue, vor den Toren Fürstenbrücks, an einem geheimen Ort. The answers (0..7): >Ja!<, >Was soll die Frage?<, >Natürlich! Beiseite, Sekundant!<, >Jawohl!<, >Geht's jetzt endlich los?<, >Aus dem Weg, Sekundant!<, >Ja, bei meiner Ehre!<, >Ja, ich liebe Duelle!<. The voice digits of boxes 3..6 are c0 = 2 for a man, 1 for a woman, c1 = 1 for a woman (so files `2021nnn` for men, `1121nnn` for women); the duellist's controller is not used here.

Correction to `rules-16`: the first answer is `rand(8)` (not `rand(7)`), so answer 7 / file `x021008` / `1121008` is reachable; the second answer is `(a + 1 + rand(7)) mod 8` and is always different from the first.

## 4. Fight animation [C]

Pointer-hide counter `0x84D6C` += 1. For k = 0..104: `bank_frame(0, 0, F, k)`; `region_copy(screen, 100, 220, surface, 100, 220, 400, 200)`; `delay_ticks(5)` (50 ms). Counter −= 1. About 5.3 s, no RNG, no sound, no input check.

## 5. Fog [C]

1. **`f = rand(8)`**; `say_box(0 2 2 1 : 933 + f, 40, 40, 560, 120, T[0xB6CA0][f])` (the eight "Nebel" lines, `rules-16` 0221933..940).
2. `picture(0xD1)` (B/ICOND1, mountains in clouds) drawn into the window surface, then the **wipe 0x2C140**: for each row y = 1..479 two draws, **`rand(9)`** then **`rand(25)`** (958 draws; row offsets `a[y] = a[y−1] + rand(9) − 4` from `a[0] = 0`, widths `w[y] = w[y−1] + rand(25) − 12` from `w[0] = −250`); then 200 steps, `delay_ticks(1)` on every odd step, each step growing every row's revealed span from both screen edges inwards (`region_copy` of `[0, a+w/2)` and `[a + 640 − w/2, 640)` per row) until the two spans meet. ~1 s. `flip`.

## 6. Exchanges [C]

**`n = 4 + rand(5)`** exchanges (4..8); **`k = rand(63)`** taunt start index. The loop runs exactly n times; nothing ends it early (a key only cuts the current speech sample). Each exchange has two halves, first A then B; per half, in this order:

1. **`y = rand(240) + 120`** (A) / **`y = rand(240) + 100`** (B), then **`x = rand(240) + 20`** (A) / **`x = rand(240) + 150`** (B).
2. `say_box(hum(p) ? 1 : 2, g(p), 2, 1 : 11 + k, x, y, 240, 120, T[0xB5434][k])` (the 63 taunts, `rules-16` x x 2 1 : 011..073; the box has no picture change, it sits over the fog picture).
3. **`k = (k + 1 + rand(62)) mod 63`** (never the same taunt twice in a row).
4. **`s = 4 + rand(5)`** swings; each: **`rand(300)`** → `delay_ms(100 + that)`; **`rand(7)`** → `sample_cmd(T8[0x84D30][3·r], 3)` (column 0 of the triple table: 0x83, 0x5F, 0x80, 0x81, 0x82, 0x83, 0x53; the last one was never loaded, so r = 6 is silent).
5. **`h = 1 + rand(2)`** hits; each: `delay_ms(100)`; **`c = rand(2)`**; **`r = rand(6)`**; `sample_cmd(T8[0x84D30][1 + 3·r + c], 3)` (hit columns: 0x39/0x3A, 0x31/0x3B, 0x32/0x3C, 0x3D/0x3E, 0x3E/0x3F, 0x40/0x41).

Correction to `rules-14` §2.4: there are `4 + rand(5)` swings and `1 + rand(2)` hits per half, not one of each. All the draws happen with sound off (`sample_cmd` is a no-op then) and with speech off (`say_box` falls back to a click).

Draws per half: 2 (position) + 1 (advance) + 1 + 2s + 1 + 2h = 5 + 2s + 2h, s ∈ 4..8, h ∈ 1..2.

## 7. After the fog [C]

1. **`e = rand(5)`**; `say_box(0 2 2 1 : 941 + e, 40, 40, 560, 120, T[0xB7090][e])` (`[EXE text]` …).
2. Scores. `L(x, y)` = `life(x)` if `hum(x)`, else `life(y) + rand(10)`:
   - `S_A = L(A, B) + office_index(A) + fenc(A) + rand(20)`, draws in that order: **`rand(10)`** only if A is a computer person, then **`rand(20)`**;
   - `S_B = L(B, A) + office_index(B) + fenc(B) + rand(20)`: **`rand(10)`** only if B is a computer person, then **`rand(20)`**.
   `office_index` (0x44050) is the global office index 0..33 of `rules-00`, **0 for no office and also 0 for the first Ratsherr seat**.
   B wins iff `S_A < S_B`; ties go to A. Winner `W`, loser `V`.
3. Result bank `Q = T16[0x84D20][4·[B wins] | g(B) | 2·g(A)]`: A wins → **0xDD** (mm), **0xDE** (mf), **0xE0** (fm), **0xE1** (ff); B wins → **0xDF** (mm), **0xE3** (mf), **0xE2** (fm), **0xE4** (ff). B/ICONDD..E4, 19 frames each over the same meadow base; the loser lies on the grass, the winner stands over him. `icon_load(Q)`; `bank_frame(0, 0, Q, 0)` into the surface; **wipe 0x2C460(0)**: again per row y = 1..479 **`rand(9)`** then **`rand(25)`** (958 draws; centre `c[y] = c[y−1] + rand(9) − 4` from 320, width as above), 200 steps with `delay_ticks(1)` on odd steps, each row revealed from its centre outwards. `flip`.
4. Hide counter += 1; for k = 0..18: `bank_frame(0, 0, Q, k)`; `region_copy(screen, 80, 200, surface, 80, 200, 420, 220)`; `delay_ticks(5)`. Counter −= 1. ~1 s.

## 8. Result captions and effects [C]

Voice slot for a person p: `v(p) = hum(p) ? g(p) : 2`.

1. `say_box(0 2 v(W) 1 : 946, 40, 40, 560, 120)` 0xA8277 `$C%1N1 grinst triumphierend.$N` with (W).
2. Office roll: if `office_index(V) ≠ 0`: **`o = rand(100)`**; office lost iff `o ≤ 32`. A loser whose only office is global index 0 (first Ratsherr seat) never rolls and never loses it.
   - lost: `say_box(0 2 v(V) 1 : 948, …)` 0xA82D7 `[EXE text]` with (V, `T[0xB6DD8][g(V)]` = "er"/"sie", V); then person(V) +0x0F := 0, +0x11 := 15 (seat not refilled here);
   - otherwise: `say_box(0 2 v(V) 1 : 947, …)` 0xA8296 `[EXE text]` with (V, `T[0xB6DE0][g(V)]` = "Seine"/"Ihre").
3. If `hum(V)`: **`life(V) −= 1 + rand(5)`** (signed byte, no floor; a computer loser's life is untouched).
4. `fav(A→B) := 50` if it is < 101; `fav(B→A) := 50` if it is < 101 (`+0x20 + index`; blackmail values above 100 are kept).
5. Screen rebuild by the saved window width: width 440 (the year-end window) → `picture(0xB8)`, `banner(0xFF, 0, 0)`; otherwise → `picture(0xB7)`, `banner(A, "Ereignisse" 0xBB3B0, 0)`. Samples `T8[0x84D30][17..0]` unloaded in reverse order; `flip`; `win_open(saved x, y, w, h, 0xA810)`; `flip`.
6. `rec_delete(R)`; `0x8457C := CONFIG[0x278]`.

Nothing is printed into the report window afterwards; the report continues with the next record. There is no chronicle or history entry and no mission counter.

## 9. RNG order, whole routine [C]

All draws are game RNG and are drawn whether or not sound, speech or the animation are visible (none of `say_box`, `sample_cmd`, `music(0x17)`, `bank_frame`, `region_copy`, `delay_*`, `banner`, `rec_delete` draws; checked over their code ranges).

1. `rand(11)` place/time t.
2. `rand(8)` answer a; `rand(7)` → b.
3. `rand(8)` fog line f.
4. wipe 0x2C140: 479 × (`rand(9)`, `rand(25)`).
5. `rand(5)` → n = 4 + …; `rand(63)` → k.
6. n times, for p = A then p = B: `rand(240)` y, `rand(240)` x, `rand(62)` advance, `rand(5)` → s, s × (`rand(300)`, `rand(7)`), `rand(2)` → h, h × (`rand(2)`, `rand(6)`).
7. `rand(5)` after-fog line e.
8. [`rand(10)` if A computer], `rand(20)`, [`rand(10)` if B computer], `rand(20)`.
9. wipe 0x2C460: 479 × (`rand(9)`, `rand(25)`).
10. [`rand(100)` office roll if `office_index(V) ≠ 0`].
11. [`rand(5)` life loss if V human].

Fixed count: 1 + 2 + 1 + 958 + 2 + 1 + 958 = 1923 draws plus the exchange draws (n × 2 × (5 + 2s + 2h)) and the three optional ones. `rules-08` §3 lists only the draws of step 8, 10 and 11; the 1916 wipe draws and the exchange draws sit **before** the score rolls, so any replay of the scores must run the whole scene's draws.

## 10. Corrections to earlier docs
- `rules-15` §7.7: the fencing animation is the 105-frame bank **0xD9..0xDC** chosen by the two genders, drawn before the opening captions (base frame as backdrop) and animated after "En garde!"; the result bank **0xDD..0xE4** is chosen by the winner and the genders; **0xD1** is only the fog picture between them. `0x2C140` / `0x2C460` are row-wise wipe transitions, each costing 958 RNG draws.
- `formats.md` §1.7: `BD9-BDC` are the four fight banks (105 frames: base + 104 overlays, many empty); `BDD-BE4` are the eight result banks (19 frames). `BD5-BD8` are the auctioneer banks.
- `rules-16`: duel answers `rand(8)` then `(a + 1 + rand(7)) mod 8`; file 008 is reachable. Taunt box geometry resolved: 240×120 at `(20 + rand(240), 120 + rand(240))` for the insulter and `(150 + rand(240), 100 + rand(240))` for the insulted, y drawn before x. Result-line voice digit c2 = gender for a human, 2 for a computer (as stated).
- `rules-14` §2.4: `4 + rand(5)` swings and `1 + rand(2)` hits per half-exchange; the swing table's seventh entry 0x53 is never loaded and therefore silent.
- `rules-08` §3: the office-loss condition is `office_index(V) ≠ 0`, which excludes global office 0; the first caption names the human side first even when the computer insulted.

## Open questions
1. Meaning of the speech global `0x8457C` inside the driver (0x11C1B) [L].
2. Whether `delay_ticks` / `delay_ms` are scaled in the DOSBox build (`rules-15` §0 [L]).
