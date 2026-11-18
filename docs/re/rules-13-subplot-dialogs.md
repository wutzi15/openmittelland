# Sub-plot and contest dialogs: texts, arguments, answers

Confidence tags as in `rules-00-map.md`: **[C]** argument order checked at instruction level, **[L]** decompiler only, **[G]** guess. Text addresses are linear addresses of CP437 strings (see `rules-12` for the `%` codes, `formats.md` §8.3 for the `$` escapes). Rules (amounts, record layout, RNG draws) are in `rules-06-events.md` §5 and §6; this document adds what is printed, with which arguments, which answers exist and what each answer index means. Argument lists are in print order.

`Fy(k)` as in `rules-06`. "human" = the human whose turn it is (sub-plots) or the human being asked (contest). `rand(n)` draws are listed where they pick a text, so the RNG sequence can be kept; counts match `rules-06` unless a correction is noted.

## 1. Mechanics shared by all sub-plot dialogs [C]

### 1.1 Window
Every sub-plot text, dialog and spinner is printed into the **turn-end report** text window (`rules-12` §2: full-screen style 0x8100, icon 0xB7, banner "Ereignisse" for the player, text window `win_open(30, 110, 520, 290, 0xA810)`). Starters run in the phase-6 tail of the human's turn end, handlers in the turn-end record loop (dispatcher at 0x63780: record type − 6 indexes a jump table; types 32..49 call the handlers of `rules-06` §5.1). The only exception is the concert day (type 36, counter 0), which tears the screen down for a scene and rebuilds it (§2.5). There are no message boxes.

The year-end contest texts (§3) are printed into the **year-end "Ereignisse in Mittelland"** window (`rules-12` §2: `win_open(100, 126, 440, 300, 0xA810)` after icon 0xB8 and the player-0xFF banner); the world record loop (resolution) and the announcement run after that window is open.

### 1.2 Answer menus (`%a` lines, `menu_choice` 0x36850)
A dialog is one `print` whose text ends in `$|` and contains `%a` bullet lines. After the print the game waits for a click on any item created by **that print** and returns `clicked item − first item of the print`, counting every `%a`, `%m`, `%M` item of the print in text order from 0. Clicks outside the items (and right clicks) are ignored, so a bullet menu cannot be cancelled. The returned index is the "answer" below; the lines are listed in index order. Most texts print a follow-up line (`$Z$L...$N$N`) for each answer.

### 1.3 Number spinners (`%m` with a numeric style)
`%m` takes `(style, pointer)`. Styles used here: `0x12` 2-digit, `0x13` 3-digit, `0x14` 4-digit u16 field; `0x25` 5-digit u32 field; `0x415` = a 5-digit display-only field (the pointed value is re-rendered on every redraw, no clicks); `0x430` = string item (prints the string at the pointer). Default range of a numeric field is 0 .. 10^digits − 1; `item_range(item, min, max)` (0x34B00) overrides it where noted.

Clicking a digit of a numeric field adds 10^(position from the right) when the click is in the **upper half** of the line and subtracts it in the lower half; the result is clamped to [min, max] (a step past max lands on max, a step below min on min). Thousands separators are skipped when the digit position is computed. Only left clicks are acted on [L: button mapping]. The loop ends when a bullet item of the same print is clicked; the value in the field at that moment is the answer. The sim's `Ask::Plot` spinner steps return this final value; the implementer may use any input method that yields a number in the stated range.

### 1.4 Name tables used below
| table | entries | content |
|---|---|---|
| `0xB5AE4` | 30 | tavern names ("Zum feisten Abt", "Der Maßkrug", …) |
| `0xB4FB8` | 117 | first names ("Achim", "Adalberth", "Adi", …) |
| `0xB5530` | 49 | book titles |
| `0xB6618` | 15 | good names (by good id) |
| `0xB6194` | 21 | location names by id; `0xB6198` = the same table from id 1, `0xB61BC` from id 10 (Mooringen, Bergenhöh, Auenthal, Mittelland) |
| `0xB6AA8` | **10** | robber nicknames ("Blutbarth", "Dumpfhold", …) |
| `0xB6410` | **18** | robber epithets ("Blutleere", "Triefnasige", …) — `rules-12` §3.9 k=82/87 has the two sizes swapped; the draws there are `0xB6AA8[rand(10)]`, `0xB6410[rand(18)]` [C] |
| `0xB4378` / `0xB5854` | 34 | male / female office names by global office index |

Person arguments are person indices printed with `%N`; a "name string" argument is the `char*` at person +0x09 printed with `%s`.

### 1.5 Mapping to the sim
`Ask::Plot { kind, step }`: kind = record type, step as numbered in `crates/mad-sim/src/subplots.rs` (0.. for the starter, 10.. for the handler). `Event::SubPlot { person, kind, step }`: 0xFF = record created by the starter (the starter's own texts are the "started" message), 0xFE = expedition lost (34), otherwise the handler ran with counter c after the decrement. Where the EXE dialog has more answers or a different shape than the sim's step, it is noted under **sim**.

## 2. Sub-plots

### 2.1 Type 32, inventor (starter 0x19C30, handler 0x19DC0) [C]
Starter (needs no type-32 record of the human):
1. `0xA44DB` `[EXE text]` with (tavern `0xB5AE4[rand(30)]`). Answer 2 → `0xA468F` `[EXE text]`, end. Answer 0 → dialog 2. Answer 1 → dialog 3.
2. `0xA4767` `[EXE text]` (no args). Answer 1 → `0xA46FC` `[EXE text]`, end. Answer 0 → dialog 3.
3. `0xA49C7` `[EXE text]` with (`"r Herr"`/`" Dame"` by the human's gender, `0xB6AE0[gender]`; Korn price in Bergstedt + 3 (city 5 price byte, good 0)). Answer 1 → `0xA468F`, end.
4. `0xA5005` `[EXE text]` (no args). Answer 1 → `0xA5332` `[EXE text]`, end. Answer 0 → pay 1000, `0xA5386` `[EXE text]`, record (counter 4).

**sim:** steps 0..3 are modelled as four yes/no; the EXE's first dialog has three lines and its answer 1 skips dialog 2. Treat step 0 answer 1 as "invite" (jump to step 2) and answer 2 as refuse.

Handler, c = 3: `0xA9EE0` `[EXE text]` (no args). Answer 1 → `0xAA202` "Soll er sich doch anderswo sein Geld erschnorren…", delete. Answer 0 → P = Fy(800) (stored +4): `0xAA26D` `[EXE text]` with (P). Answer 1 → `0xAA580` `[EXE text]`, delete. Answer 0 → pay P, `0xAA5E4` `[EXE text]` (sim steps 10, 11).
c = 2, 1: `0xB3A1D` `[EXE text]` with (P); pay.
c = 0: `0xABD7E` `[EXE text]` with (n, noun id 22 + u (Dampfkarren/Tauchboot/Flugmaschine, after the navigability fallback), description `0xB7180[u]`: `[EXE text]` / `[EXE text]` / `[EXE text]`); delete.

### 2.2 Type 33, book publishing (0x1A000, 0x1A240) [C]
Starter (draw order: name `rand(117)`, profession `rand(32)`, dress `rand(2)`, season `rand(4)`):
1. `0x9CA0E` `[EXE text]` with (season `0xB6028[rand(4)]` = wunderbaren Sommer / stürmischen Herbst / eiskalten Winter / regnerischen Frühlings, dress `0xB6038[rand(2)]` = gut / ärmlich, first name `0xB4FB8[name]`, profession `0xB596C[rand(32)]` = Advokat, Bäcker, Baumeister, …). Answer 1 → `0x9CB84` `[EXE text]`, end.
2. Then book b = `rand(49)`, price = Fy(500), grade a = `rand(11)`, place = `rand(2)`: `0x9CBE7` `[EXE text]` with (the human's name string, `0xB6048[place]` = im Keller / auf dem Dachboden, `0xB5530[b]`, `0xB6050[a]` = ganz hervorragend, spannend, höchst interessant, wissenschaftlich, unterhaltsam, reichlich abgehoben, recht spekulativ, ziemlich trocken, abgegriffen, gotteslästerlich, obszön (a = 0..10, lower is better), `0xB4FB8[name]` (same name as in 1), price). Answer 1 → `0x9CE87` `[EXE text]`, end. Answer 0 → `0x9CEBF` `[EXE text]` with (profession); pay the **shown** price (one Fy roll) [L]; record.

Handler, first run (+7 = 0xFFFF): three offers i = 0..2, each with 4 draws `rand(2)` b0, b1, b2, b3 (b0 first); quality **q_i = 8·b0 + 4·b1 + 2·b2 + b3** (first draw is the most significant bit; the sim builds it the other way round). `0xA308B` `[EXE text]` with (book, grade adjective `0xB6050[a]`, then per offer the sub-format `0xA3124` `[EXE text]` with (`0xB68BC[b0]` starren/beweglichen, `0xB68A4[b1]` grobem Papier/edlem Pergament, `0xB68AC[b2]` unbebildert/bebildert, `0xB68B4[b3]` Rinds/Hirsch, q_i + 5)). Answer 0..2 picks q (sim step 10). Then `0xA31DA` `[EXE text]` with (style 0x13, pointer to +7, preset **10**): 3-digit spinner 0..999, confirmed by the bullet (sim step 11). Pay (q + 5)·n, `0xA3218` `[EXE text]` with ((q + 5)·n, book).
c = 0: `0xAF54A` `[EXE text]` with (n, book); delete.
Otherwise D as in `rules-06`: n < D → `0xAF639` `[EXE text]` with (book, (q + 8)·n·2), receive, delete; else `0xB3A56` `[EXE text]` with (book, D, (q + 5)·D·2), receive.

### 2.3 Type 34, trade expedition (0x1A680, 0x1A770) [C]
Starter: no dialog. Notice `0xA619E` `[EXE text]` with (city name `0xB6194[rand(10)]`, the drawn computer person x, destination `0xB6C44[dest]` = Bajuristan, Kalifat El Baghdiar, Ustschenien, Ostindien, Arabien, Persien, Kurprinzentum Feistenbeck). Draw order: `rand(3)` county for the pick, pick, dest `rand(7)`, city `rand(10)`.

Handler, c = 4: W, M drawn as in `rules-06`; `0xAA65D` `[EXE text]` with (dest, W, M). No free Karren → `0xAA6AC` `[EXE text]`, delete. Else `0xAA710` `[EXE text]` with (K = free Karren, 0x12 → +5 carts (range 0..K), 0x12 → +7 mercenaries (0..99), 0x415 → cost display = 100·carts + 200·mercs); items 0 bullet, 1 carts, 2 mercs, 3 cost; the bullet ends the loop (sim steps 10 carts, 11 mercs). n = 0 → `0xAA7E7` `[EXE text]`, delete. Else `0xAA880` `[EXE text]`, pay.
c = 3, 2, 1: `0xB3AB6` `[EXE text]` with (dest).
c = 0, no branch or ambush (event step 0xFE): `0xAD761` `[EXE text]` — **the code passes no argument** for the `%s` [C quirk]; reproduce with the branch city name or an empty string [G]. Modest return: `0xAD8F6` `[EXE text]` with (dest, branch city name). Rich return: `0xADA51` `[EXE text]` with (dest, branch city, Fy(1000)).

### 2.4 Type 36, concert (0x1ACC0, 0x1AEA0) [C]
Starter (host x, R, Q as in `rules-06`; then `rand(10)` for the place name): `0x9FBE9` `[EXE text]` with (x, `"r "`/`" "` by the human's gender (`0xB6464[g]`; → "Bester Hans" / "Beste Anna"), human, `0xB6498[g]` = Musikus / Konzertmeisterin, year + 2, `0xB6194[rand(10)]`, R, `0xB646C[Q]`, x). Quality adjectives `0xB646C[0..10]`: gänzlich hervorragender, begnadeter, außergewöhnlicher, recht guter, durchschnittlicher, gewöhnlicher, mieser, erbarmungsloser, durstiger, tölpelhafter, dilletantischer. Answer 1 → `0x9FFCD` `[EXE text]`, end. Answer 0 → `0x9FF6F` `[EXE text]`, record.

Handler, c = 1 (ensembles n_i, s_i, F_i as in `rules-06`; then three tavern draws, the **third** group first: t3 = `rand(10)`, t2 = `rand(10)`, t1 = `rand(10)`): `0xAF6CF` `[EXE text]` with (x, `0xB5AE4[t1]`, `0xB5AE4[10 + t2]`, `0xB5AE4[20 + t3]`, then (n_0, `0xB646C[s_0]`, F_0), (n_1, …), (n_2, …)). Answer 0..2 (sim step 10). Then `0xAF85A` `[EXE text]`, pay.

c = 0, **scene**: `$V` (click) into the report, then the report window is closed and the screen rebuilt: picture 0xBB at (0, 0); caption box `0x36C90(0, 380, 640, 100, 0, …)` with `0xAA8DE` `[EXE text]` (x); sound cue 0x1D; caption `0x36C90(0, 420, 640, 60, 0, …)` `0xAAD2F` "$CDie Musik erklingt...$N" (no args); outcome box `0x36C90(0, 160, 640, 240, 0, text, x, payout)`: failure `0xAA968` `[EXE text]` (x; the payout argument is unused), middling `0xAAA9C` `[EXE text]` (x, payout), success `0xAAC0C` `[EXE text]` (x, payout). Each caption box waits for a click [L]. Then icon 0xB7, banner "Ereignisse" for the human, and the report window is reopened with its saved geometry and style 0xA810; delete.

### 2.5 Type 37, wager (0x1B400, 0x1B660) [C]
Starter (draw order: tavern `rand(30)`, shoes `rand(8)`, hair `rand(7)`, drink `rand(5)`): `0x947A8` `[EXE text]` with (tavern, `0xB55F8[drink]` = Wein, Starkbier, Blaupichler, mysteriöser Brühe, Lauenburger; `0xB560C[hair]` = wirren, zerfransten, hochgebundenen, wenigen, blonden, fettigen, nassen; `0xB5628[shoes]` = karierten Pantoffeln, antiken Sandalen, niederländischen Holzklocks, nackten Füßen, eleganten Schuhen, schicken Lederstiefeln, einem dümmlichen Grinsen, eleganten Beinkleidern). Then the record, kind w, stake, and greeting `rand(4)`: `0x9499C` `[EXE text]` with (`0xB564C[rand(4)]` = ">Hey!<, tönt sie" / ">Pssst<, flüstert sie" / `[EXE text]` / ">Hört mal<, raunt sie"; stake; claim `0xB565C[w]` as a `%s` sub-format which consumes three further arguments (good name `0xB6618[+6]`, location name `0xB6194[+5]`, discipline `0xB5668[+6]`): w = 0 `[EXE text]`, w = 1 `[EXE text]`, w = 2 `[EXE text]`). For w = 1 byte +6 is the tavern index and the swallowed table reads are out of range but harmless. Disciplines `0xB5668[0..7]`: Maßkrugstemmen, Bombenlegen, Schwertkampf, Sackhüpfen, Luftanhalten, Den-rechten-Glauben-zeigen, An-den-Pranger-stellen, Mit-verbundenen-Augen-zechen. Answer 0 → `0x94BF3` "$Z$L>Fein. Dann bis zum nächsten Jahr!...<"; answer 1 → `0x94BA7` `[EXE text]`, delete.

Handler (win as in `rules-06`): w = 0 `0x881AF` `[EXE text]` with (good, city, `0xB4BA0[win]` = leider gefallen / nicht gefallen); w = 1 `0x881F2` `[EXE text]` (tavern `0xB5AE4[+6]`) then `0xB4BAC[win]`: lost `0x8826B` `[EXE text]`, won `0x882D0` `[EXE text]`; w = 2 `0x88334` `[EXE text]` with (discipline, location name `0xB6194[(city + (1 − win)·(1 + rand(9))) mod 10]`, i.e. the stored city when won, another city when lost; the `rand(9)` is drawn in both cases) then `0xB4BB8[win]`: lost `0x8835B` `[EXE text]`, won `0x883EE` `[EXE text]`. Then `0xB4BC0[win]` with (stake): lost `0x88459` `[EXE text]`, won `0x884AB` `[EXE text]`; delete.

### 2.6 Type 38, robbers' raid on a count's silver (0x1B8D0, 0x1BC00) [C]
Starter (x = random male computer person, camp as in `rules-06`; look `rand(3)`): `0xA0DDB` `[EXE text]` with (`0xB6658[rand(3)]` = zwielichtiger / unsympathischer / hochgewachsener, x, `"r "`/`" "` by the human's gender, human, camp name, x). Answer 0 = admit, 1 = deny (sim step 0).
- Deny, law off: `0xA106D` `[EXE text]`, favour, delete. Deny, law on: `0xA0FAC` `[EXE text]` (x), +5 = 0.
- Admit, law on: `0xA1106` `[EXE text]` with (style 0x25, pointer to a local preset 0): items 0 bribe, 1 spinner (0..99999), 2 throw out; ends on 0 or 2. Answer 2 → `0xA123C` `[EXE text]`, +5 = 0, record stays. Answer 0 with b < standing(h, 40) → `0xA12B2` `[EXE text]` (b), favour, delete. Else `0xA1304` `[EXE text]`, pay b to x. **sim:** step 1 = the spinner value b, step 2 = 1 for "throw out" (answer 2), 0 for "bribe".
- Invitation (admit with law off, or paid): count name `rand(117)` (+4), then `rand(10)` city, `rand(2)` confession: `0xA1341` `[EXE text]` with (`0xB4FB8[+4]`, `0xB6688[rand(2)]` = evangelischen / katholischen, `0xB6194[rand(10)]`); +5 = 1.

Handler, c = 1, +5 = 0 (denounced): `0x885C6` `[EXE text]` (human); answer 0 → `0x886EA` `[EXE text]` (fine), answer 1 → `0x888CD` `[EXE text]` (fine); fine = standing(h, 50) either way; pay, delete. **sim gap:** this dialog has no effect and is not asked; add a cosmetic step 11 if the UI needs it.
c = 1, +5 = 1: no robbers → `0x88988` `$ZIn Eurem Räuberlager >%s< befinden sich zur Zeit keine Räuber; Pech gehabt...` (camp name), delete. Else `0x889DF` `[EXE text]` with (count name, style 0x12 → r, range 0 .. min(T, 99)) (sim step 10). r = 0 → `0x88A5E` `[EXE text]`, delete. Else `0x88AA7` `[EXE text]`.
c = 0 (returns, L lost, then place `rand(3)`): `0xA891C` `[EXE text]` with (camp name, human, count name, `0xB6E70[rand(3)]` = Auf einem Hohlweg / Auf einem Bergpass / In einer Schlucht, L, noun id 32 (Räuber: "einem Räuber" / "5 Räubern")); then for goods 11, 12, 13: `0xA8A6F` `%d %s$N` with (quantity, good name); then epithet `rand(18)`, nickname `rand(10)`: `0xA8A77` `[EXE text]` with (`0xB6AA8[rand(10)]`, `0xB6410[rand(18)]`); delete.

### 2.7 Type 39, cartwright cartel (0x1C040, 0x1C270) [C]
Starter (city `rand(10)`, figure `rand(3)`, time `rand(3)`): `0x88BB5` `[EXE text]` with (city name, `0xB4D4C[time]` = an einem Sonntagnachmittag / zu vorgerückter Stunde / während einer Hinrichtung, `0xB4D58[figure]` = einem seltsam aussehenden Mann / einer kauzigen Gestalt / einem fromm dreinblickenden Priester, human). Answer 0 → `0x88DB3` `[EXE text]`, end (sim step 0: answer 0 declines). Answer 1 → P = standing(h, 20): `0x88E20` `[EXE text]` with (P, style 0x12 → years y, range **0..10**, preset 0, style 0x430 → suffix string: "" while y = 1, "e" otherwise, so "1 Jahr" / "3 Jahre"); items 0 rent, 1 spinner, 2 suffix, 3 shout; ends on 0 or 3 (sim step 1 = y; answer 3 ≡ y = 0). y = 0 or shout → `0x88FD3` "Ängstlich blickt sich der Mann um und als er sieht, dass Euer Ruf die Aufmerksamkeit einiger Leute erregt hat, nimmt er die Beine in die Hand...", end. Else record, `0x8906F` `[EXE text]` with (P / 2), pay.

Handler, every turn: `0xAAD4A` `[EXE text]` with (year of the contract (+5), city name (+4), `0xB701C[c = 0]` = `" "` / `" ein letztes Mal "`, P, `0xB7024[c = 0]` = `".."` / `[EXE text]`); pay; delete at c = 0.

### 2.8 Type 40, scholar's lecture (0x1C350, 0x1C4C0) [C]
Starter (record first; +4 `rand(9)` profession, +5 `rand(2)` title, +6 `rand(10)` surname, +8 `rand(10)` city, then first name `rand(117)`, reputation `rand(7)`): `0x97204` `[EXE text]` with (`0xB59F0[rand(7)]` = bekannte, harmlose, gefährliche, zweifelhafte, umstrittene, berühmte, hochgeehrte; `0xB5A0C[+4]` = Mathematicus, Theoretiker, Alchemist, Häretiker, Hypothetiker, Analytiker, Rethoriker, Bajurethiker, Animaloge; `0xB5A30[+5]` = Doktor / Professor; `0xB4FB8[rand(117)]`; `0xB5A38[+6]` = Sorglos, Kühngeist, Weißnix, Tralloper, Klotz, Kopflos, Wirrgeist, Mabuse, Schneppberger, Geistheimer; `0xB6194[+8]`). Answer 1 → `0x973E3` `[EXE text]`, delete. Answer 0 → `0x97450` `[EXE text]`.

Handler, c = 1: hours `rand(10)`: `0xA63D8` `[EXE text]` with (city, title, surname, profession, hours + 2, title). Loop: `0xA6631` `[EXE text]`; answers 0 and 1 → `rand(10)`, `0xA66AD` `[EXE text]` (hours + 2) and the dialog repeats; answer 2 continues (sim step 10). Good g = `rand(15)`: `0xA6786` `[EXE text]` (good name) (sim step 11). Answer 1 → `0xA6855` `[EXE text]` (profession), delete. Answer 0 → A = Fy(1500), +9 := 0: `0xA6903` `[EXE text]` with (good name, A, style 0x14 → +9, range 0..9999, preset 0); items 0 accept, 1 counter-offer, 2 spinner; ends on 0 or 1; answer 0 sets the offer to A, answer 1 keeps the spinner value (sim step 12 = this answer, step 13 = the spinner value; in the EXE both come from one dialog). Accepted (`A·rand(100) < 100·offer`) → `0xA6AA9` `[EXE text]`; refused → `0xA6A23` `[EXE text]` (title), delete.
c = 0: `0xB3B09` `[EXE text]` (good name, offer); pay, skill, delete.

### 2.9 Type 41, excavation (0x1C820, 0x1C990) [C]
Starter: `0xAC74F` `[EXE text]` with (residence word of the branch, city name, Fy(1000)) followed by `0xAC7C6` `%aJa!%aNein!$|`. Answer 1 → `0xAC7D5` `[EXE text]`; answer 0 → pay, `0xAC7F5` `[EXE text]`, record.

Handler: c = 0 or residence gone → `0xB3BC3` `[EXE text]`, delete. Else C = Fy(1000); `rand(100) < 60`: find `rand(15)`: `0x9E83F` `[EXE text]` with (`0xB62F0[rand(15)]` = eine Büste Karls des Kopflosen, einen reichverzierten Keuschheitsgürtel, ein römisches Feldzeichen, einen schwarzen Steinquader mit den Seitenverhältnissen 9:4:1, einen Doppelchunkomer, eine eiserne Blumenvase aus der Bronzezeit, eine vorchristliche Streckbank, ein Schachspiel aus Ton, eine edelsteinbesetzte Krone, eine Truhe aus Birnbaumholz, eine Marmorsäule, eine Flasche mit der Aufschrift 'BVRGVNDER MCCCLXXIV', spärliche Überreste einer römischen Siedlung, einige Keilschrifttafeln, einen goldenen Ring; residence word, city name) (sim step 10); answer 0 → `0x9E8DF` `[EXE text]`; answer 1 → `0x9E90E` `[EXE text]` (city) and wealth_up; then `0x9E931` `[EXE text]` (C). No find → `0x9E7BD` `[EXE text]` (C). Then `0xAC7C6` Ja!/Nein! (sim step 11): 1 → delete, 0 → pay C. Then `$N`.

### 2.10 Type 42, church seminar (0x1CB90, 0x1CCE0) [C]
Starter (+4 title `rand(4)`, +5 city `rand(10)`, fee, season `rand(4)`): `0x892D7` `[EXE text]` with (`0xB4E98[title]` = Preise aushandeln - aber richtig / Subtile Beeinflussung durchschnittlich Begabter / Versteckte Drohungen in Theorie & Praxis / Göttlicher Beistand für Kaufleute, `0xB715C[rand(4)]` = Frühling, Sommer, Herbst, Winter, city name, fee). Answer 1 → tavern `rand(30)`, `0x894C4` `[EXE text]` (tavern), delete. Answer 0 → `0x8957E` `[EXE text]`, pay.

Handler, c = 7 (greeting `rand(5)`, attendees `rand(20)`, seat `rand(3)`): `0x97480` `[EXE text]` with (`0xB5A6C[seat]` = auf eine wurmstichige Holzbank der Kirche / in die erste Reihe im Dom / auf den kalten Steinfußboden im Kapitelsaal der Kirche, city name, attendees + 10, title, `0xB5A78[greeting]` = Brüder und Schwestern, Kinder Gottes, Sünder, die Ihr allesamt seid, Der schnöde Mammon ist nicht alles, Halleluja). Then `1 + rand(3)` times: a sermon quote `0xB5AB8[rand(10)]` (ten quotes, `0x97A15` … `0x98316`, each ending `$Z`), then a reaction `0xB5A8C[rand(11)]` (`0x97656` `[EXE text]` … `0x979D6` `[EXE text]`). Then `0x9844B` `[EXE text]`.
c = 6: `0xADD87` `[EXE text]` (title). Invitation only (clergy person J exists and `rand(100) < 40`; next title `rand(4)`): `0xADE02` `[EXE text]` with (`0xB4E98[(title + rand(4)) mod 4]`, J), favour, delete. Else `0xADF4B` `[EXE text]`.
c = 0: `0xB3C1E` `[EXE text]`, delete.

### 2.11 Type 43, card game 17+4 (0x1CF60, 0x1D000) [C]
The starter creates the record (counter 1) and runs the game at once; later years the handler runs with c = 0.
- c = 0 and `rand(100) < 50`: `0xAF981` `[EXE text]` (x), delete.
- Stake dialog (sim step 10; style 0x14, range 0..9999, preset 0). First game (c = 1): `0xA278A` `[EXE text]` with (tavern `0xB5AE4[+5]`, x, spinner); items 0 play, 1 spinner, 2 decline. Rematch (c = 0): `0xAF8D4` `[EXE text]` (same args; item 2 is the invisible `%M`, so the rematch can only be declined with stake 0). Stake 0 or decline → `0xA2841` `[EXE text]` (x), delete.
- Deal: `0xA289E` "$Z$LJeder erhält zwei Karten.$N"; if x drew k > 0 cards: `0xA28BF` `%1N1 nimmt %d weitere.$N` (x, k); `0xA28D9` `%1N1 ist zufrieden.$N` (x). While the human's total < 21: `0xA28F1` `[EXE text]` (total); 0 = draw (sim step 11). `0xA2938` `[EXE text]` with (human, human total, x, x total). Both over 21: `0xA2977` "$LIhr habt beide zu viele Punkte; Unentschieden!$N"; equal: `0xA29AA` `[EXE text]`; lost: `0xA29D7` `[EXE text]`; won: `0xA29F3` `[EXE text]` (x). After a loss or draw: `0xA2A49` `[EXE text]` (x). Rematch answer (sim step 12): 1 → `0xA2841` (x), delete; 0 → `0xA2ACB` `$Z$L>Gut<, nickt %1N0. >Bis zum nächsten Mal!<` (x), counter := 1.

### 2.12 Type 44, contract to topple an official (0x1D4E0, 0x1D6A0) [C]
Starter. **Draw order:** `rand(13)`, `rand(3)`, pick t, **`rand(6)` (deadline/counter), `rand(60)` (Fy(5000)), `rand(3)` (figure)** — the sim draws Fy, then rand(3), then rand(6). `0xA6B65` `[EXE text]` with (`0xB6C84[rand(3)]` = seltsamer Geselle / ganz in schwarz gekleideter Mann / Mann mit einem arrogant wirkenden Gesicht, human, t, `0xB6C90[gender of t]` = sein / ihr, year + 3 + rand(6), reward). Answer 1 → `0xA6D83` "Mit den Worten >Nun gut, ich werde jemand anderen finden; wir haben uns jedenfalls nie gesehen!<, verlässt der Mann Euer Kontor.", end. Answer 0 → `0xA6E13` `[EXE text]` (t), record with counter 3 + the same rand(6).

Handler, c > 0: still in office → `0xAF9F7` `[EXE text]` (t). Toppled → `0xAFA46` `[EXE text]` with (t, office name by global index of (+5, +6): `0xB4378[o]` for a man, `0xB5854[o]` for a woman, location name +5); +6 := 15, counter := 1.
c = 0: +6 = 15 → `0xAFB50` `[EXE text]` (reward, t), receive; else `0xAFAA1` `[EXE text]` (t). Delete.

### 2.13 Type 45, wedding speech (0x1D840, 0x1D9A0) [C]
Starter (county `rand(3)`, x): `0xAADEC` `[EXE text]` with (x, county name, `"r "`/`" "` by the human's gender, the human's name string, year + 2, `"Euer"`/`"Eure"` (`0xB4270`/`0xB4274`) by x's gender, x). Answer 1 → `0xAAFC0` `[EXE text]` (x), end. Answer 0 → `0xAAFE7` `[EXE text]` (x), record.

Handler, c = 1 (per monk i: rank `rand(3)`, grade `rand(5)`, fee as in `rules-06`): `0xA544B` `[EXE text]` with (x, then per monk (`0xB6AFC[grade]` = hochgebildeter, intelligent aussehender, durchschnittlich gebildet erscheinender, etwas tumb wirkender, geistig völlig verwirrter; `0xB6B10[rank]` = Abt, Sakristan, Klosterbruder; fee)). Answer 0..2 (sim step 10). `0xA564F` `[EXE text]` (fee), pay.
c = 0: `0xA8AB7` `[EXE text]` with (x, `0xB6E94[gender of x]` = der Bräutigam / die Braut, band text `0xB6E88[band]` as a `%s` sub-format consuming (x, payout)): band 0 `0xA8C17` `[EXE text]`, band 1 `0xA8D25` `[EXE text]`, band 2 `0xA8D99` `[EXE text]`. Receive, delete.

### 2.14 Type 46, peasant revolt (0x1DD70, 0x1DE90) [C]
Starter: `0xAE05D` `[EXE text]` with (x, `0xB61BC[k]`). Answer 1 → `0xAE143` `[EXE text]` (x), end. Answer 0 → `0xAE16E` `[EXE text]`, record.

Handler, c = 3 (S = mercenaries over the castles; city `rand(3)`): `0xAB060` `[EXE text]` with (location name `3k + 1 + rand(3)`, a city of the county). S = 0 → `0xAB0CE` `[EXE text]`. Else `0xAB114` `[EXE text]` with (S, style 0x13 → n, range 0..S) (sim step 10). n = 0 → `0xAB16C` `[EXE text]` (x), favour −10, delete. Else `0xAB199` `[EXE text]` (county name).
c = 2: victory (`rand(100) < 66`; place `rand(7)`) → `0xAE1D9` `[EXE text]` with (`0xB4B3C[rand(7)]` = Bei den Deppenbergen, Am Fuße des Schädelhügels, Unweit eines Baches, In saftigen Auen, In der Nähe eines alten Steinbruchs, Nahe einem Wasserfall, Nahe einer Höhle; x); defeat (city `rand(3)`) → `0xAE22F` `[EXE text]` with (x, location name `3k + 1 + rand(3)`), losses. Then `0xAE2AD` `[EXE text]` and the 60 % repeat.
c = 1 or 0: `0xAE2E1` `[EXE text]` with (n, noun id 33 (→ "Euren n Söldnern"), R, x, 100·n + 50·R (never paid), `0xB72C0[new branch]` = `".%v"` / `[EXE text]`, location name of the new branch (swallowed by `%v` when none)); delete.

### 2.15 Type 47, sales contract (0x1E580, 0x1E7F0) [C]
Starter (draw order: counter `rand(3)`, city `rand(10)`, Q `rand(800)`, name `rand(117)`, adjective `rand(4)`): `0xA7495` `[EXE text]` with (`0xB6D44[rand(4)]` = grauhaariger / verschlagen aussehender / gutgekleideter / geschäftig wirkender, `0xB4FB8[rand(117)]`, city name C, years c, city name C, Q, good name, Y, penalty). Answer 1 → `0xA7785` `[EXE text]`, delete. Answer 0 → `0xA77F6` `$Z$L>Abgemacht!< Der Mann schüttelt Eure Hand. >Und denkt daran: %w Jahre lang sind %d Einheiten %s in %s zu verkaufen; vergesst das nicht!<...` (c, Q, good name, city name).

Handler: too little sold → `0xAC911` `[EXE text]` (good name, city, penalty), pay, delete. Else `0xAC85B` `[EXE text]` with (Y, `0xB71B0[c = 0]` = `"..$N$N"` / `[EXE text]`), receive; delete at c = 0.

### 2.16 Type 48, cart lease (0x1E910, 0x1EA80) [C]
Starter (n `rand(10)`, place `rand(3)`): `0xA8E7E` `[EXE text]` with (`0xB6EA0[rand(3)]` = Auf dem Marktplatz / In der Kirche / Vor dem Rathaus, city name of B, human, year + 5, n, P). Answer 0 → `0xA901E` `[EXE text]`, end (sim step 0: answer 0 declines). Answer 1 → `0xA9073` `[EXE text]`, record.

Handler, c = 4: fewer than n Karren in B → `0xAFC7E` `[EXE text]` with (Karren in B, noun id 19, city name), delete. Else `0xAFBDB` `[EXE text]` with (city name, n, 19, P), receive.
c = 3..0: `rand(100) < 10` → `0xACABD` `[EXE text]` (city name, n, 19), delete. Else `0xAC9ED` `[EXE text]` with (P, n, 19, city name, `0xB71C0[c = 0]` = `".."` / `[EXE text]`); nothing is paid; at c = 0 the Karren return and the record is deleted.

### 2.17 Type 49, regatta (0x1EC50, 0x1ED90) [C]
Starter: `0xA788B` `[EXE text]` (tavern `rand(30)`) (sim step 0).
- Answer 0: `0xA79BE` `[EXE text]` (sim step 1). 1 → `0xA7AA1` `[EXE text]`, end. 0 → `0xA7AE4` `[EXE text]`, v = 0.
- Answer 1: `0xA7B39` `[EXE text]` (sim step 2). 0 → `0xA7C17` `[EXE text]`, end. 1 → `0xA7C58` `[EXE text]`, v = 1.

Handler, v = 0 (Fy(200) first, then `rand(3)` → second place, `rand(3)` → first place): `0xA56C6` `[EXE text]` with (1 + rand(3), 4 + rand(3), Fy(200)); receive, delete.
v = 1: nobody in Fürstenbrück → `0xA5B20` `[EXE text]`, delete. Else f = min(4, favour(x→h)/21): `0xA575D` `[EXE text]` with (x, x, `0xB6B28[f]` = absolut nicht gewogen / nicht sonderlich gewogen / zu nichts verpflichtet / durchaus gewogen / äußerst gewogen, style 0x14 → b, range 0..9999, value 0 after the print [L: the preset is written after printing, so the field may first show stale memory]) (sim step 10). `2000·f + b < 10000` → `0xA5902` `[EXE text]` (x); else `0xA59AA` `[EXE text]` (x). **Quirk [C]:** with f = 0 the 4-digit field cannot reach 10000, so the bribe needs favour ≥ 21. Delete.

## 3. Newspaper contest (type 4, start 0x19480, resolution 0x19890) [C]

### 3.1 Announcement and entry (year end, `rules-06` §6)
`0x9A0C8` `[EXE text]` with (newspaper `0xB5D34[rand(6)]` = `[EXE text]`, "Die >Nordland-Nachrichten< geben", `[EXE text]`, "Die >Finsterwalder Post< gibt", "Der >Lauenburger Bote< gibt", "Die >Greifenheim-Gazette< gibt"; fee). The `rand(6)` is drawn after the fee.

Per human h in slot order (at most 6 stored entrants): `0x9A2B4` `[EXE text]` (h); then for d = 0..4 one line `0x9A2EA` `%a>%s %s<$|` with (superlative `0xB5D58[good_d]`, good name `0xB6618[good_d]`), then `0x9A2F6` `%aFertig$|`. Superlatives by good id: Das goldenste, Der größte, Das hochprozentigste, Die weißeste, Das bestgemaserte, Das zarteste, Das eisenhaltigste, Der schwerste, Das glänzendste, Das durchsichtigste (ids 0..9; the five goods are `3d + rand(3)`, so ids up to 14 index past the table [C quirk, text garbage for Silber, Pfeffer, Waffen etc.; reproduce with an empty superlative [G]]). The six items form a **multi-select**: clicking item d (0..4) toggles bit `1 << d` of the mask and redraws the bullet glyph (0x73 off, 0x74 on); item 5 ("Fertig") ends. Then `0x9A3FA` `[EXE text]` (mask 0) or `0x9A437` `$Z$L%1N1 nimmt am Wettbewerb teil.$Z` (h) and the fee × number of disciplines is paid. `Ask::ContestEntry { person }` returns the mask.
No human entered: `0xBA8D4` `[EXE text]`, delete. Else `0x9A45D` `[EXE text]` with (city name +4, judge J).

### 3.2 Result (next year end, world record loop)
`$B` then `0xA7D49` `[EXE text]` (city name, J). For each discipline d (good g) in which **at least one human entered** (bit set; a score of 0 does not matter — the sim's "scored" test is stricter): if somebody placed, `0xA7E39` `[EXE text]` (superlative, good name) and then, **third place first**, `0xB6D80[k]` with (person, prize = [4, 3, 2][k] × fee): k = 2 `0xA7F40` `[EXE text]`, k = 1 `0xA7EEC` `[EXE text]`, k = 0 `0xA7E86` `[EXE text]`; each is paid as printed. If every score is 0: `0xA7FC4` `[EXE text]` (superlative, good name). Finally `0xA7F8E` `[EXE text]`, delete.

## 4. Sim discrepancies found while tracing (not fixed here)
- 33: print-offer quality bits reversed (first draw is the MSB, §2.2).
- 44: RNG order rand(6), rand(60), rand(3) (§2.12).
- 32: the first dialog has three answers; answer 1 skips the second dialog (§2.1).
- 38: the arrest dialog (two answers, no effect) is not asked (§2.6).
- 40: accept / counter-offer and the counter-offer amount are one dialog (§2.8).
- Contest: a discipline is judged when a human **entered** it, not when a human scored; zero scores print `0xA7FC4` (§3.2).
- 39: years spinner is limited to 0..10 (§2.7); 38: robbers to min(T, 99); 46: mercenaries to S; 34: carts to the free Karren; all other spinners 0..10^digits − 1 (§1.3).

## 5. Open / unverified
- Mouse-button mapping of the spinner (left = act, right = ignored) and whether the spinner redraws the display-only cost field on every click: [L].
- The `%s` of `0xAD761` (expedition ambush) has no argument; what the original prints there depends on the stack and is [UNVERIFIED].
- Contest superlatives for good ids ≥ 10 read past the 10-entry table: the printed text is [UNVERIFIED].
- Concert scene: whether each caption box waits for a click, and the exact picture ids 0xBB / 0xB7: [L] (read from the call sequence, not from the box code).
- Which person-record nibble (+0x10 & 0xF0) marks "human entrant" in the contest judging gate: [L].
