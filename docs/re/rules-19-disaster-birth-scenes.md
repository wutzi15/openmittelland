# Disaster, birth and the other turn-phase scenes; scene order (rules-19)

Source: `disaster_damage` 0x18BD0, child handler 0x221F0, title award 0x49700, turn-end settlement 0x61A70 and year end 0x66220 (call order), every `0x2E310` picture-draw site in the image (enumerated). Confidence [C] unless marked. Primitives and notation as in `rules-15` §0 and `rules-16` §1; rules of the events in `rules-06` §2 (disaster) and `rules-05` §4 (children).

## 1. City disaster scene (0x18BD0) [C]

Runs at year end (`rand(100) < 12`, `rules-06` §1), after the office cycle's full-screen window has been closed and before the auction. Returns without drawing when the city test fails.

RNG and record order first (all before any drawing): `c = rand(10)`; city checks (population > 1499, bit 7 of +0x6E clear) else return; record type 2 created (owner 0xC0, key c, +5 = 0, +6 = year, counter |= 0x80); `d = rand(4)` repeated until bit `d` of the city's +0x6E is set; bit 7 set.

Scene for kind `d` (0 fire, 1 flood, 2 plague, 3 avalanche), with `y = {216, 12, 12, 216}[d]` (table 0x84B28):
1. `win_open(0, 0, 640, 480, 0x8100)`; picture `{0x86, 0xC9, 0xCA, 0x86}[d]` (table 0x84B24: B/ICON86 stone wall with windows for fire and avalanche, B/ICONC9 flood interior, B/ICONCA the reaper) at (0, 0); `music({0x0D, 0x0C, 0x0B, 0x0D}[d], mode 1)` (table 0x84B30; mode 1 = start unless already playing); for `d = 0` or `3` sample 0x45 is loaded; flip.
2. `say_box(0 2 2 0 : 900 + c, 10, y, 620, 160)` 0xBA898 `$Z$C%o!$N` with (c): the city name, spoken.
3. `x = 2d + rand(2)`; `say_box(0 2 2 0 : 910 + x, 10, y, 620, 160)` with `T[0xB5E10][x]` (the eight disaster sentences, `rules-16` §2).
4. Damage maths (no drawing, RNG in this order): `L = rand(T[d].pop) · pop / 100 + 10`, capped so that at least 1000 remain; `pop -= L`; `w = rand(T[d].wealth)`; `m = 3 + rand(7)`; `wealth_down(c, (w·m) & 0xFF)`; for `d ≠ 2`: marketplace flag := `T[d].byte2` (25/15/–/15, always > 0) and record +5 := 1. (`w` is reused as the adjective index below.)
5. Film, only when `y ≠ 12` (fire, avalanche): icon cache reset; `film(feuer | lawine, 0, 0, 12)` with `sample_play(0x45)` (avalanche rumble) started right after the file is opened, stopped and unloaded after the film; icon cache re-initialised; flip. Nothing is redrawn: the following boxes appear over the last film frame.
6. Caption `(10, y, 620, 280, 0)` (click): `T[0xB5E30][d]` with (L, 0x9B0D7, `T[0xB5E48][w]`, `T[0xB5E58][d ≠ 2]`):
   - fire 0x9AFFD `[EXE text]`, flood 0x9B03B `[EXE text]`, plague 0x9B06A `[EXE text]` (no `%s`, the extra arguments are ignored), avalanche 0x9B0A0 `[EXE text]`;
   - `%s` = 0x9B0D7 `[EXE text]` with (`manche / einige / viele / etliche`[w], ` ` or `[EXE text]`). Since `w = rand(T[d].wealth)` and `T.wealth ≤ 4`, index 3 "etliche" occurs only for fire. The marketplace clause appears for every fire, flood and avalanche, never for plague (`rules-06` §2 says "selects" it; it is unconditional per kind).
7. For every person slot 0..191 that is used, every branch of that person in city `c` (property order): goods `stock[g] = stock[g] · T[d].keep / 100` (15 goods, loss summed at the city price), wood then stone storage `s = s · (T[d].base + rand(10)) / 100` (two draws per branch, loss summed at the storage prices). Then, humans only: caption `(10, y, 620, 280, 0)` 0x9B151 `[EXE text]` with (person, ` ` or ` und einige Waren ` when any good was stocked); history entry 6 (plague) or 7 (flood) for the human. Computer persons take the damage silently.
   After the branches: if the person's loss > 0 and insurance bit 0 (+0x07 & 1): humans get caption `(10, y, 620, 280, 0)` 0xA9DBC `[EXE text]` with (person, loss); the loss is refunded (`add_cash`) for humans and computer persons alike.
8. `win_close`.

Screen notes: the caption boxes for kind 1/2 sit at y = 12 (top), for 0/3 at y = 216 (lower half); all are 620 wide; the two speech boxes are 160 high, the text boxes 280. There is no sample 0x45 for flood/plague. The music started here keeps playing into the auction/year-end screens until replaced.

## 2. Birth and child death (type-31 handler 0x221F0) [C]

Runs in the parent's turn-end record loop (counter already decremented). `R` = the record.

### 2.1 Birth (counter = 45)
1. Free human-name slot search: first of the 32 slots (`0xE377C + 24·i`) whose first byte is 0. None → record deleted, scheduling (`0x220C0`), end (no scene).
2. `R+5 := slot`; `R+3 -= 1` (44).
3. `$V` (0xBA918) into the turn-end report (click if anything was printed); report window geometry saved; window closed; items cleared.
4. `music(0x16, 0)` (An der Wiege); picture **0xCE** (B/ICONCE, cradle with a baby) at (0, 0).
5. RNG: `x = rand(5)`. Speech slots loaded: slot 0 `(0 2 2 6 : 900 + 2·g(parent) + sex)` (sex = `R+4 & 1`, 0 son); slot 1 `(0 0 0 0 : 14 + x)`; `r = rand(4)`; slot 2 `(0 0 0 0 : 14 + (x + 1 + r) mod 5)`.
6. Slot 0 **and** slot 1 are started at once (they overlap; the `0000014..018` files, 1.0–3.1 s, are therefore not narration but a sound over the narrator, presumably a baby's cry [UNVERIFIED content]).
7. Print `T[0xB711C][g(parent)]` with (parent, `T[0xB7124][sex]`): 0xABAE2 `[EXE text]` / 0xABB10 `$C$g%1N1, Ihr habt %s geboren!$N`, `%s` = "einen Sohn" / "eine Tochter". Printed into the full-screen window at the current text cursor (top of the picture) [L: no `$nH` in the text].
8. Print 0xBAA4C `[EXE text]` with (style 0x84, pointer to the name slot): line 16 (y = 396), a text-entry item of kind 0x80 (`rules-16b` §6) writing directly into the 24-byte name slot; `item_range(it, 1, 23)` (1..23 characters); the item is made the keyboard focus (`0x84D50 := it`; item flags: bit 0x8000 cleared, 0x4000 set [L: focus/edit flags]); flip.
9. `menu(it, 0, &d)`: keyboard entry; Enter accepts when at least the minimum length is typed and the name passes the duplicate check `0x2D7D0` [L]; a mouse click on the item also ends it [L].
10. Slot 2 started; loop until `sound_idle()`; all speech popped; item removed.
11. History entry `0x2D750(parent, 4, sex, 0)`.
12. Picture **0xB7**; banner "Ereignisse" (0xBA91C) for the parent; flip; `win_open` with the saved geometry, style 0xA810; flip (the report continues in it).
13. Scheduling `0x220C0(parent)` (`rules-05` §4: next child counter `47 + rand(3m + 3) + rand(3)` and sex `rand(100)`).

Values the dialog can show: parent gender (`+0x00`), child sex (`R+4` bit 0), main-heir bit (`R+4` bit 7, not shown), name slot. The name typed is the only input; it is stored in the human-name table immediately and used by the testament and succession (`rules-17`).

### 2.2 Child death (counter v < 45)
Condition (`rules-05` §4): parent life ≥ 5 and (`v = 0` or (`45 − v < 10` and `rand(100) < 3`)). Then: `$V`; geometry saved; window closed; items cleared; `music(9, 0)` (Trauermarsch); picture **0xC7** (graveyard); print (son: `R+4 & 0x7F = 0`) 0xBAA7C `[EXE text]` or (daughter) 0xBAAF0 `[EXE text]` with (name slot string, `46 − v`) on line 1 (y = 36); `wait_click`; picture 0xB7, banner "Ereignisse", flip, report window reopened (0xA810), flip; name slot freed (first byte := 0); record deleted. No rescheduling.

## 3. Fate events, tornado, title award, certificate [C]

- **Fate events** (0x226F0, "Das Schicksal", `rules-06` §4, texts `rules-12` §3.9): no picture, no film, no speech. They are report lines (and `%a` menus) inside the turn-end report window. The function has no `0x2E310` call.
- **Tornado** (divine event 8): message box `(80, 80, 480, 320)` over the year-end "Ereignisse in Mittelland" window, OK glyph, click (`rules-15` §4.1). The other divine events: `(160, 100, 320, 280)`. No picture.
- **Title award** (0x49700, `rules-03` §2.1), the first thing in every human's turn-end settlement (before the bill): when a title is awarded: `win_open(0, 0, 640, 480, 0x8100)`, picture **0xB7**, flip, `win_open(80, 20, 400, 440, 0x8810)` (parchment); the Urkunde text `T[0xB6ECC][a]` by awarding authority (0 city council 0xA914C `[EXE text]`, 1 Vogt 0xA921C `[EXE text]`, 2 Regent 0xA9298 `Wir, %1N5, verfügen hiermit…`, 3 Emperor 0xA9315 `[EXE text]`), each followed by the seal **0xCB** (ICONCB) drawn at (26, 360); `music(0x1E, 0)` (fanfare group); `wait_click`. Titles that open the county offices (0xBD1A8 `[EXE text]`) and the principality offices (0xBD298 `[EXE text]`) are extra pages (`$P`), each with the seal at (26, 360) and a click [L for which title levels trigger them]. Both windows closed.
- **Trade certificate** (0x494F0, called from the turn start 0x4B900, not turn end): same two windows, text `T[0xB6ED8][L]` (0xBD… certificate II/III), seal 0xCB at (26, 360), `music(0x0E, 1)`, three `speech_say (0 2 2 4 : …)` lines (`rules-16` §2), `wait_click`, windows closed.
- **Game over** (no human left, year loop): picture **0xBA**, speech `0229948`, caption menu `(10, 360, 620, 120)` (`rules-16` §2).

Every other picture-draw site in the image belongs to a Kontor screen (map, branch, castle, camp, church, money lender, office tree, battle, options, title/load menus) or to a scene already specified in `rules-15`: history/mission end 0x17930, concert 0x1AEA0, feast 0x20530, matchmaker 0x20D30, wedding 0x212B0, auction 0x27680, trial 0x289E0, duel 0x2C750, elections/dismissals 0x66160/0x66220, jury court / debtors' tower / bankrupt / death in 0x61A70. There is no further hidden scene.

## 4. Order of scenes and windows at the end of a human turn [C]

Everything below happens inside one full-screen window of style 0x8100 with picture 0xB7, the "Ereignisse" banner for the player and the text window `(30, 110, 520, 290, 0xA810)` (`rules-12` §2). "Scene" = the text window is closed, the scene draws its own pictures, then picture 0xB7, banner and the text window are rebuilt and the report continues (`$B`).

1. Title award screen (§3) — before the report window exists.
2. Report window, page 1: the bill ("Eure Ausgaben", sample 0x84 after "Gesamtkosten"), click, new page.
3. Report lines in this order: too many branches (`music(8, 2)`), office income, castle tolls, assistant tips, fate events 0x226F0 (once, then again with 50 %), blackmail ends / rival reaction, then the **record loop** over the player's records in heap order, each record's handler printing into the window or running its scene: sub-plot handlers (types 32..49; type 36 counter 0 = **concert scene**), feast (type 23, **feast scene**), mistress (29, text), courtship (30; decision year = **wedding scene** with film), child (31; **birth / child-death scene**), siege/relief (25, 26; **belager/entlager films**), yearly fine (19), loan due, troop recruiting and prisoners.
4. Spy evidence: **informant films** (`rules-15` §5.4), then the report rebuilt.
5. Phase-6 tail (0x64108): sub-plot starters (texts/menus in the window), rival grudge, **duels** (type 22, duel scene per record), crime scan, **trials** (type 18, `rules-18`, one scene per record), debt seizure (texts in the window, then possibly the **jury court** scene with its castle picture, tower or acquittal screen, which ends the turn at once).
6. If nothing was printed since the bill: `[EXE text]` (0xC169C) on line 4; `wait_click`; items cleared; window closed.
7. Bankrupt check (no branch): **bankrupt scene** (`rules-17` §1.1), turn over.
8. Mission check 0x17930 (**history replay / mission accomplished scene** with outro films when the mission is complete).
9. Life decrement; death: **candle film, gravestone, notary, testament, funeral** (`rules-17`).
10. Phase 6; the full-screen window is closed by the caller.

Elections and dismissals are not part of a human turn; they are year-end (next list).

## 5. Order at year end (0x66220), as the players see it [C]

1. Full-screen window 0x8100 (no banner, no text window). Favour drift and office dismissals: per dismissal a **place picture** (`rules-15` §3.1) with the text on line 7/8, `music(8, 0)` for humans, click.
2. Computer-person turnover, vacancies, applications; per election a **place picture** with candidate list, vote coins and result (`rules-15` §3.4); appointments without election: place picture + line 8 + click.
3. Window closed (0x68858).
4. **City disaster scene** (§1), own window.
5. Computer persons' castle/camp orders (0x61410: **battles** with their own screens, `rules-07`).
6. **Auction day** (`rules-15` §1), own window.
7. Full-screen window 0x8910, picture **0xB8**, banner "Ereignisse in Mittelland, A.D. year" (player 0xFF), text window `(100, 126, 440, 300, 0xA810)`: herald (fanfare), **sabotage film** (window rebuilt after it), insurance, tolls, castle sales (menu), maintenance/manoeuvres, auction announcement, world fate 0x26640 (tornado and the other divine texts are later), world records (disaster aftermath, contests), contest announcement, auction announcement 0x269D0, computer mistresses, computer-person actions 0x59F80 (texts), divine event 0x59940 (**message boxes**, tornado box), then the **computer-person record pass**: trials (**trial scene** per computer plaintiff, rebuilt as 0xB8 + banner 0xFF), duels (**duel scene**), sieges/relief (**films**), AI follow-ups, mistress events.
8. `$V`; if the cursor never left line 12: `[EXE text]` (0xC2024) and a click. Window closed.
9. Year + 1, city growth, autosave; the next human's turn start (newspaper/chronicle and turn-start screens, `formats.md` §9.3).

## 6. What the sim must supply
- Disaster: `c`, `d`, `x`, `L`, `w`, `m`, per-branch storage draws, per-human loss and insurance flag; the scene needs city name, kind, loss, adjective index, marketplace flag, per-human (branch owner, goods-lost flag, loss, insured).
- Birth: parent, sex, name slot, `x`, `r`; the typed name (1..23 chars) back into the sim; child death: name, shown age `46 − v`, sex.
- The two orderings above, so scene triggers (`Event::…`) are emitted in the same sequence as the original draws them.

## 7. Open
- Content of `0000014..018` (overlapping the narrator at the cradle) [UNVERIFIED].
- Text-entry item details: width of a kind-0x80 item with nibble 4, accepted characters, the duplicate check 0x2D7D0 [L].
- Title-award extra pages: which title levels print 0xBD1A8 / 0xBD298 [L].
- Disaster caption at y = 12 with height 280 for flood/plague reaches y = 292: whether the second box overlaps the first visually is a rendering question, not a code one.

## Corrections (rules-27)
- Title award extra pages: n = 3 (0xBD1A8, county offices) and n = 6 (0xBD298, principality offices), same window, seal at (26, 360), click after each, no music. Charter music `0x1E` follows the charter's seal. Mayor, Vogt and county come from a random own branch. Closes the `[L]` item above.
