# Speech map: every SPC WAV file and its trigger (rules-16)

Source: MITTELL.EXE disassembly, every call of `speech_load` 0x15100, `speech_say` 0x15570 and `say_box` 0x36E50 (51 + 7 + 52 sites), cross-checked against the 1416 file names in `SPC22KHZ` with a generator script (same formulas as below): 1328 files are reachable from code, 0 referenced files are missing, 88 files exist but no code path names them (§6). Confidence tags as in `rules-14`: [C] traced at instruction level, [L] decompiler reading, [UNVERIFIED] inferred. Nobody listened to the files; "says" is the caption printed with the line, so names and numbers inside `%` codes are not in the recording.

Supersedes `rules-14` §3 and resolves §5 "Speech". Corrections to `rules-14`: the `F023`/`1023` files are the funeral mourners' heckles, not tips; `0229948` is the game-over line, not a year-start line; the `0x15570` sites of the trial carry the defendant's gender in c2 or c1 as listed below.

## 1. Name scheme [C]
`speech_load(c0, c1, c2, c3, n)` builds `<c0><c1><c2><c3><nnn>.WAV` with each `c` an index into `0123456789abcdefghijklmnopqrstuvwxyz` (printed upper-case in the directory) and `nnn` = n as three decimal digits. So `(11, 0, 0, 2, 74)` → `B002074.WAV`, `(0, 2, 2, 9, 905)` → `0229905.WAV`.

Dimensions used by the game:
- **c0 = voice.** 0 narrator. 1 = "human" voice, 2 = "computer" voice: wherever a party speaks, c0 = 1 if the person is human-controlled (person byte +0x10 high nibble ≠ 0) and 2 if computer-controlled; the office-tree remark rolls `1 + rand(2)` instead. 3..10 = eight witness voices (`3 + rand(8)`). 11 (`B`) = the defence counsel. 12..14 (`C`..`E`) = computer judges (`12 + rand(4)`, and a roll of 15 becomes voice 2). 15..17 (`F`..`H`) and 1 = the four mourner voices (`15 + rand(4)`, 18 becomes 1).
- **c1, c2 = gender slots** (person byte +0x00, 0 male / 1 female) of the persons named in the line; 2 where unused. Which person goes where is listed per group.
- **c3 = text family:** 0 disasters, 1 duel/insults, 2 trial, 3 funeral, 4 certificate, 5 wedding, 6 births, 7 branch opening, 8 auction, 9 history and ruin screens.
- **n:** 900 + line for the narrator families (c0 = 0); 0..92 for the voiced families.

Wait rules (`rules-14` §1.3): `say_box` waits until the sample ends, or for a click if nothing plays; `speech_say` waits for the end or a mouse button; plain `speech_load`+`speech_play` sites wait with `sound_idle` or a click as noted. With speech switched off (option bit 0x80) every site falls back to a click wait. Trigger geometry is the caption box (x, y, w, h) of `rules-15` §0.2; caption boxes are frameless text over the picture (`rules-16b` §3).

## 2. Table: file pattern → trigger → n → caption

Column "caption" gives the linear address of the printed text (strings decode as in `formats.md` §8.3); `T[a][i]` = pointer table at `a`, entry `i`.

| files | trigger (function) | c0 c1 c2 c3 | n | caption / says | conf |
|---|---|---|---|---|---|
| `0000000..012` | start-up intro film (`formats.md` §8.1): preloaded into slots 0..12, played per segment table `0x8700C` | 0 0 0 0 | 0..12 | no caption; film sync | C |
| `0000013` | mission accomplished 0x17930: loaded and played (no wait) just before `outro.flc`, and again before `outrob.flc` | 0 0 0 0 | 13 | no caption (88 s narration over the outro) | C |
| `0000014..018` | birth 0x221F0: after the parent line (below) two of these five play: slot 1 = `14 + x`, `x = rand(5)`, played right after the parent line; slot 2 = `14 + (x + 1 + rand(4)) % 5`, played after the name has been entered | 0 0 0 0 | 14..18 | no caption; caption on screen is `0xBAA4C` "Welchen Namen soll das Kind tragen?" with a name input. Content of the five lines [UNVERIFIED] (1.0–3.1 s each) | C for slots, L for the second play |
| `0220900..909` | city disaster 0x18BD0, box `(10, y, 620, 160)`, y = 216/12/12/216 for kind 0..3 | 0 2 2 0 | 900 + city (0..9) | `0xBA898` `$Z$C%o!$N` — the city name shouted | C |
| `0220910..917` | same scene, second box | 0 2 2 0 | 910 + 2·kind + rand(2) | `T[0xB5E10][2·kind + rand(2)]`: fire `[EXE text]` / "Ein grausiger Brand verschlingt…"; flood "Ein furchtbarer Sturm tobt…" / `[EXE text]`; plague `[EXE text]` / `[EXE text]`; avalanche `[EXE text]` / `[EXE text]` | C |
| `0221921..931` | duel 0x2C750, first box `(40, 40, 560, 120)` | 0 2 2 1 | 921 + t, t = index into `T[0xB67AC]` (11 entries, rolled by the duel setup [L: `rand(11)`]) | `0xA2B04` `[EXE text]` with %s = im Morgengrauen / um die Mittagsstunde / in der Morgendämmerung / zu früher Stund' / des Nachmittags / beim alten Glockenturm / auf dem Friedhof / im Pfarrgarten / auf einer lieblichen Aue / vor den Toren Fürstenbrücks / an einem geheimen Ort | C |
| `0221932` | duel, box `(240, 380, 320, 100)` | 0 2 2 1 | 932 | `0xA3FEA` `[EXE text]` | C |
| `0221933..940` | duel, after the fencing animation, box `(40, 40, 560, 120)` | 0 2 2 1 | 933 + k, k = index into `T[0xB6CA0]` (8) [L: `rand(8)`] | `[EXE text]` / `[EXE text]` / `[EXE text]` / `[EXE text]` / `[EXE text]` / "Dieser Schlagabtausch findet wohl im Nebel statt!" / "Vom nahegelegenen Moor zieht unverhofft Nebel auf…" / `[EXE text]` | C |
| `0221941..945` | duel, after the exchanges, box `(40, 40, 560, 120)` | 0 2 2 1 | 941 + k, k into `T[0xB7090]` (5) [L: `rand(5)`] | `[EXE text]` / `[EXE text]` / "Später…" / "Einige Stunden später…" / `[EXE text]` | C |
| `0201946`, `0211946`, `0221946` | duel result, box `(40, 40, 560, 120)` | 0 2 w 1, w = 0 male human / 1 female human / 2 computer (the winner) | 946 | `0xA8277` "%1N1 grinst triumphierend." | C |
| `02x1947` | duel result | 0 2 l 1, l = loser code as above | 947 | `0xA8296` `[EXE text]` | C |
| `02x1948` | duel result when the loser is an office holder and `rand(…) ≤ 32` [L] | 0 2 l 1 | 948 | `0xA82D7` `[EXE text]` | C |
| `2021001..007`, `1121001..007` | duel: each duellist answers the question, boxes `(100, 180, 240, 120)` and `(250, 150, 240, 120)` | c0 = 2 for a man, 1 for a woman; c1 = gender; 2 1 | 1 + rand(7) | `T[0xB6988]`: ">Ja!<" / `[EXE text]` / ">Natürlich! Beiseite, Sekundant!<" / ">Jawohl!<" / ">Geht's jetzt endlich los?<" / ">Aus dem Weg, Sekundant!<" / ">Ja, bei meiner Ehre!<" (entry 7 ">Ja, ich liebe Duelle!<" = file 008 is never rolled) | C |
| `2021009/010`, `1121009/010` | duel: first duellist's voice, box `(240, 380, 320, 100)` | as above | 9, 10 | `[EXE text]`, ">En garde!<" | C |
| `1021011..073`, `1121…`, `2021…`, `2121…` | duel taunts: one per exchange, box 240×120 at a random position [L]; c0 = 1 human / 2 computer duellist, c1 = gender | v g 2 1 | 11 + k, k advances `(k + 1 + rand(…)) % 63` from a random start | `T[0xB5434][k]` (63 lines): `[EXE text]`, `[EXE text]`, … ">Aua!<", ">Wo seid Ihr?<", … `[EXE text]` | C for n, L for geometry |
| `1021011..022`, `1121…`, `2021…`, `2121…` | office tree 0x45C30 (bar clicked on a person who holds no office and has no pending record) and computer-person reaction 0x5B200 case 6: the insult | `1 + rand(2)`, gender of the insulted person, 2 1 | 11 + rand(12) | the same first twelve taunts, then a message box `(140, 120, 360, 240)` `0xB5EEC`/`0xB5F50` "%s beleidigt Ihr %4N0…" (office tree) or `0xB5EF0` "%s beleidigt %1N5, %4N0, %s" (reaction) with the taunt text; waits on `sound_idle` | C |
| `1022000..029`, `1122…`, `2022…`, `2122…` | trial 0x289E0, reading of the charge: one box `(20, 20, 600, 120)` per law bit set | c0 = 1 human / 2 computer plaintiff, c1 = plaintiff gender, 2 2 | 10·group + bit (group 0..2, bit 0..9) | `T[0xB5B5C][10·group + bit]`: `[EXE text]` … `[EXE text]` (30 lines, `rules-04b` law order) | C |
| `0202903..908`, `0212…` | trial, charge-group header before each group's lines | 0 2 d 2, d = defendant gender | 907 − 2·group + (more than one bit in the group) | `T[0xB5C58][rand(3)]` `[EXE text]` with "folgendes Gesetz" / "folgende Gesetze" | C |
| `2022030..038`, `2122…` | trial, computer defendant's plea, box `(20, 20, 600, 120)` | 2 d 2 2 | 30 + plea (0..8) | `0x99E7A` "%1N1 meint: %s" with `T[0xB5C9C]`: ">Ich gestehe Alles!<" ">Ich bin schuldig.<" ">Kann schon sein.<" `[EXE text]` ">Öh, na ja…<" `[EXE text]` ">Ich bin unschuldig…<" ">Nichts von alledem habe ich getan!<" ">Lüge, alles Lüge!<" | C |
| `1022030..038`, `1122…` | trial, human defendant confirms a plea in the 0x8910 window `(80, 20, 480, 200)`; `speech_say` | 1 d 2 2 | 30 + plea | the same nine strings as the cycling item | C |
| `0222910/911` | trial, before the plea; `speech_say` | 0 2 2 2 | 910 + d | `0x99E4F` `[EXE text]` (two recordings by defendant gender) | C |
| `3002039..055` … `A012…` | trial witnesses, one per witness, printed into a 2/3-width column with a talking head | `3 + rand(8)`, 0, d, 2 | 39 + 8·side + k; side 0 (plaintiff's witnesses) k = rand(8), side 1 (defence) k = rand(9) | `T[0xB58E4][d + 2k + 16·side]` ">Ich habe alles genau gesehen…<" … (16 strings per side, gender-paired); waits `sound_idle` | C |
| `1002056..073`, `1012…`, `1102…`, `1112…`, `2002…` … `2112…` | trial, prosecution plea, four lines each with `sound(0x10)` and figure animation | c0 = 1/2 plaintiff controller, c1 = plaintiff gender, c2 = defendant gender, 2 | 56 + rand(7); 63 + rand(6); 69 + rand(4); 73 | `T[0xB5730][7d + k]` `[EXE text]`; `T[0xB5768][6d + k]` `[EXE text]`; `T[0xB5798][k]` `[EXE text]`; `T[0xB57B8][d]` `[EXE text]` | C |
| `B002074..080`, `B112…` | trial, defence plea, three lines | 11 d d 2 | 74 + rand(3); 77 + rand(2); 79 + rand(2) | `T[0xB6694]` `[EXE text]` / ">Dieser Mann kann es nicht sein…<"; `T[0xB66AC][d + 2k]` `[EXE text]` / `[EXE text]` / `[EXE text]`; `T[0xB66BC][d + 2k]` `[EXE text]` / ">Dieses geachtete Mitglied…<" / `[EXE text]` / ">Meine Mandantin…<" | C |
| `0222900` | trial, first box after investments | 0 2 2 2 | 900 | `0x99C64` `[EXE text]` | C |
| `0222901/902` | trial | 0 2 2 2 | 901 human plaintiff / 902 computer plaintiff | `0x99C82` `[EXE text]` / `0x99CB9` `[EXE text]` | C |
| `0222914` | trial, human side's investment window 0x8910 `(100, 20, 440, 200)`; load+play, no wait | 0 2 2 2 | 914 | `0xAFD31` `[EXE text]` | C |
| `0222915` | trial | 0 2 2 2 | 915 | `0x968BA` `[EXE text]` | C |
| `0222916` | trial, box `(20, 20, 600, 200)` | 0 2 2 2 | 916 | `0x955FB` `[EXE text]` | C |
| `0222917` | trial | 0 2 2 2 | 917 | `0xA15AC` `[EXE text]` | C |
| `0222918..921` | trial bribes, computer side (`rules-04b` §3.5) | 0 2 2 2 | 918 + 2·(side = defence) + (does not bribe) | `0xACD18` `[EXE text]` / `0xACD3E` `[EXE text]` (one recording per side) | C |
| `0222922/923` | trial bribes, human side | 0 2 2 2 | 922 bribed / 923 not | `0xACD81` `[EXE text]` / `0xACDBD` `[EXE text]` | C |
| `0222924` | trial | 0 2 2 2 | 924 | `0xA5EE2` `[EXE text]` | C |
| `1022080/081`, `2022…`, `C022…`, `D022…`, `E022…` and the `x122` variants | trial verdict, one per judge, box at the judge's seat: `(56, 151)`, `(0, 184)`, `(108, 127)`, 190×30 | c0 = 1 human judge; computer judge `12 + rand(4)` with 15 → 2 (re-rolled per judge); c1 = that judge's gender; 2 2 | 80 guilty / 81 not guilty | `0xBB3A4` `$0>$0<%s` with ">Schuldig!<" / ">Nicht schuldig!<" | C |
| `1022080/081`, `1122…`, `2022…`, `2122…` | jury court of the creditors (turn end 0x64AF9, `rules-15` §6.1): all four `(v, c1)` combinations of 80 and of 81 are preloaded as slots 0..3 and 4..7; per vote slot `i` (guilty) or `i + 4`, with `i = rand(4)` at the start and `i = (i + 1 + rand(3)) % 4` after each vote (never the same voice twice in a row) | v ∈ {1, 2}, c1 ∈ {0, 1} (four juror voices, not genders), 2 2 | 80 / 81 | no caption; the juror markers of `rules-15` §6.1 | C |
| `1g d2084/085` (`1002`, `1012`, `1102`, `1112`) | trial, after the three verdicts; load+play, figures animate, then wait | 1, presiding judge's gender, defendant gender, 2 | 84 convicted (≥ 2 guilty) / 85 acquitted | `T[0xB6BD0]` "Damit ist %1N1 überführt." / "Damit ist %1N1 freigesprochen." | C |
| `1022086..092`, `2022…`, `C022…`, `D022…`, `E022…`, `x122…` | trial sentence, box `(20, 20, 600, 120)` | c0 = presiding judge's voice (re-rolled: human 1, computer `12 + rand(4)`, 15 → 2); c1 = presiding judge's gender; 2 2 | 86 + sentence (0..6) | `T[0xB6D94]`: `[EXE text]` / `[EXE text]` / `[EXE text]` / `[EXE text]` / "…zu einem Jahr Schuldturm…" / `[EXE text]` / "…zu einem Jahr Kerkerhaft…" | C |
| `0202925/926`, `0212…` | trial costs, box `(20, 20, 600, 160)` | 0 2 p 2, p = payer's gender | 925 plaintiff pays / 926 defendant pays | `0xAFDCE` `[EXE text]` | C |
| `0222927` | trial, insured party | 0 2 2 2 | 927 | `0xAFE07` `[EXE text]` | C |
| `0203900`, `0213900` | funeral (turn end 0x657C6, `rules-15` §7.4) step 0, printed at y = 10 | 0 2 s 3, s = 0 when exactly one child and it is a son, else 1 | 900 | `T[0xB5E7C][0]` ">Liebe Hinterbliebene!<" (the s = 0 recording presumably addresses a single male heir [UNVERIFIED]) | C |
| `0223901` | funeral step 1 | 0 2 2 3 | 901 | `[EXE text]` | C |
| `0223902/903` | funeral step 2 | 0 2 2 3 | 902 + gender of the deceased | `[EXE text]` | C |
| `0223904/905` | funeral step 3 | 0 2 2 3 | 904 + gender | `[EXE text]` | C |
| `0223906` | funeral step 4 | 0 2 2 3 | 906 | ">Hier das Testament:<" | C |
| `0223907` | funeral step 5, heir chosen | 0 2 2 3 | 907 | `0x9B4AF` `[EXE text]` | C |
| `0223908` | funeral: no children (cursor y = 1) | 0 2 2 3 | 908 | `0x9B6BC` `[EXE text]` | C |
| `0223909` | funeral: everything to the archbishopric | 0 2 2 3 | 909 | `0x9B70C` `[EXE text]` | C |
| `F023000..018`, `F123…`, `G023…`, `G123…`, `H023…`, `H123…`, `1023…`, `1123…` | funeral mourners' heckles at y = 421 (`rules-15` §7.4): after steps 1..4 `1 + rand(len)` lines from the first 14, at the testament step from the last 5 | `15 + rand(4)` (18 → 1), `rand(2)`, 2, 3 | 0..13 = `T[0xB5E94]` index; 14..18 = 14 + `T[0xB5ECC]` index | ">Endlich!<" ">Hurra!<" `[EXE text]` ">Darauf habe ich schon lange gewartet.<" ">Ha-haaa!<" ">Ich bin der wahre Erbe!<" ">Mir alles!!<" `[EXE text]` ">Ooohh…<" ">Darauf einen Humpen!<" ">Geizkragen!<" ">Endlich bin ich reich!<" ">Lasst die Vorrede!<" ">Zur Sache, Notar!<"; 14..18: ">Verrat!<" ">MIR ward alles versprochen.<" ">Das ist fingiert!<" `[EXE text]` ">Am Sterbebett wurde mir alles versprochen!<" | C |
| `0224900/901`, `0224912/913` | trade certificate award 0x494F0 (`speech_say`, each only if the previous was not clicked away) | 0 2 2 4 | 900 + 12·L, 901 + 12·L with L = 0 for certificate II, 1 for III | the two sentences of `0xA93EF` `[EXE text]` / `0xA94B9` `[EXE text]` (split point [UNVERIFIED]) | C |
| `0224902..911`, `0224914..917` | certificate, third line | 0 2 2 4 | 902 + place + 2·(place ≥ 10), place = office location 0..13 | `[EXE text]` (cities) / `[EXE text]` (counties, Mittelland) with `T[0xB6194][place]`: Fürstenbrück … Kronsbach, Mooringen, Bergenhöh, Auenthal, Mittelland | C |
| `0225900/901` | wedding 0x212B0: started after `sound(5)` on the church-door picture, before the first caption; waits on the caption click | 0 2 2 5 | 900 + gender of the courted person | `0x9D075` `[EXE text]` | C |
| `0225902/903` | wedding feast choice | 0 2 2 5 | 902 options 0..3 / 903 option 4 | `0x9D752` `[EXE text]` / `0x9D777` `[EXE text]` | C |
| `0226900..903` | birth 0x221F0, cradle picture `0xCE`, slot 0 | 0 2 2 6 | 900 + 2·gender(parent) + sex(child; 0 son) | `T[0xB711C][gender]` `[EXE text]` / "%1N1, Ihr habt %s geboren!" with "einen Sohn" / "eine Tochter" | C |
| `0227900..909` | city info 0x4F3C0, menu item 6 (open a branch), message box `(140, 160, 360, 160)` | 0 2 2 7 | 900 + city | `0xBE548` `[EXE text]` | C |
| `0227900` | options 0x4E810 sound check, under `0xBE114` "Soundcheck läuft…" | 0 2 2 7 | 900 | the Fürstenbrück branch line | C |
| `0228900..915` | auction 0x26A50 idle tick, `speech_say` | 0 2 2 8 | 900 + rand(16) | `T[0xB6520][k]` `[EXE text]` … `[EXE text]` (`rules-12` §3.5) | C |
| `0228916..918` | auction hammer count, played at the start of the auctioneer animation | 0 2 2 8 | 916 + count (0..2) | `T[0xB6EB4]` ">%d Taler zum Ersten…<" / "…zum Zweiten…" / "…zum Dritten…" | C |
| `0228919` | auction end 0x27680, `speech_say` | 0 2 2 8 | 919 | `0xBB1D0` "Dies war das letzte Angebot…" | C |
| `0229900` | mission accomplished 0x17930, box `(80, 200, 480, 160)` | 0 2 2 9 | 900 | `0xBA7E0` `[EXE text]` | C |
| `0229904..916` | history replay 0x17930, boxes `(80, 200, 480, 160)` | 0 2 2 9 | 904 "Große Ereignisse werfen ihre Schatten voraus"; 905 `[EXE text]`; 906 Schuldturm; 907 Hochzeit; 908 Fest; 909 Konzert; 910 befördert (m/f texts share one file); 911 Sohn; 912 Tochter; 913 Pest; 914 Jahrhundertflut; 915 Räuber überfielen Reisende; 916 Räuberlager | texts of `rules-15` §7.8 | C |
| `0229918..947` | mission text after 0229900, box `(40, 200, 560, 200)` | 0 2 2 9 | 900 + `W[0x84AE8][mission]`, W = 38 43 39 44 40 45 41 46 42 47 28 33 29 34 30 35 31 36 32 37 18 23 19 24 20 25 21 26 22 27 for missions 0..29 | `T[0xB5BD4][mission]` (`[EXE text]` …); the same word table orders the 30 missions by difficulty row | C |
| `0229948` | year loop: no human left (`[0xEF4C6] == 0xFF`): full-screen 0x8810 window, picture `0xBA` (dungeon corridor), load+play, flip, caption menu `(10, 360, 620, 120)`, click, then credits 0x4DA50 | 0 2 2 9 | 948 | `0xBD5B0` `[EXE text]` | C |
| `0229949` | jury court lost with fewer than 2 branches (`rules-15` §6.1 step 7) | 0 2 2 9 | 949 | `0xC13A8` `[EXE text]` | C |
| `0229950` | jury court acquittal, played over the night picture before the reveal transition | 0 2 2 9 | 950 | `0xC1470` `[EXE text]` | C |
| `0229951..954` | bankrupt screen (`rules-15` §6.2), one per caption | 0 2 2 9 | 951..954 | `0xC1544` `[EXE text]`, `0xC15A4` `[EXE text]`, `0xC1620` "Eure Bemühungen…", `0xC167C` "%1N1, Ihr seid bankrott…" | C |

## 3. Resolved items of `rules-14` §5
- **Group 0x29FD2 (variable trial lines):** the four prosecution lines 56..73 with `(1/2, g_plaintiff, g_defendant, 2)` as in the table; the three defence lines 74..80 with `(11, g_def, g_def, 2)`; the verdict summary 84/85 with `(1, g_judge0, g_def, 2)`.
- **Duel gender boxes:** c0 = 2 for men, 1 for women with c1 = gender (answers and the two conditions), c0 by controller with c1 = gender (taunts), and c2 ∈ {0, 1, 2} = human-by-gender / computer for the three result lines.
- **`0000000..012`** intro, **013** outro, **014..018** birth extras (two per birth).
- **`022A900/901`, `022B902..909`:** no code path builds a name with c3 = 10 or 11; unused content (1.5–5.4 s each).
- **History reminders:** 904..916 as listed; 900 and 918..947 are the mission end.
- **Death/notary:** `(0 2 s 3 : 900)`, `(0 2 2 3 : 901)`, `(0 2 2 3 : 902 + g)`, `(0 2 2 3 : 904 + g)`, `(0 2 2 3 : 906)`, then 907 / 908 / 909 by outcome; mourners `(15 + rand(4) → 15/16/17/1, rand(2), 2, 3 : 0..13 | 14..18)`.
- **Disaster `0220xxx`:** 900 + city, 910 + 2·kind + rand(2); 918..920 unused.
- **"Tips" `1023`/`F023`:** mourners, see above. The game tips (option bit 0x08) are text only.
- **Office tree / reaction remarks and births:** as in the table.

## 4. RNG draws per site (parity)
All `rand` calls above use the game RNG 0x36FF0 and are drawn even with speech off: witness voice `rand(8)` then line `rand(8)`/`rand(9)`; judge voice `rand(4)` per computer judge, again for the presiding judge before the sentence; prosecution `rand(7)`, `rand(6)`, `rand(4)`; defence `rand(3)`, `rand(2)`, `rand(2)`; mourner voice `rand(4)`, gender `rand(2)`, line `rand(14)`/`rand(5)` per line; births `rand(5)`, `rand(4)`; duel answers `rand(7)` per duellist; remarks `rand(12)`, `rand(2)`; jury `rand(4)` then `rand(3)` per vote; auction `rand(16)` per idle tick. Order inside each scene follows the rows above; see `rules-07`/`rules-08` for the surrounding draws.

## 5. Durations
`(size − 44) / 22050` s. Narrator lines 1–6 s; the outro `0000013` 88 s; everything else under 6 s. Not listened to.

## 6. Present but unreferenced (88 files) [C: no call site; content UNVERIFIED]
`0220918..920`, `0222909`, `0222912`, `0222913`, `0229901..903`, `0229917`, `022A900/901`, `022B902..909`, `1121008`, `2021008` (duel answer 8), `x022082/083` and `x122082/083` for x ∈ {1, 2, C, D, E} (a judge line pair between the verdict and the sentences), `2gg2084/085` and `Cgg2084/085`, `Dgg2…`, `Egg2…` (verdict summary in the computer/judge voices; the code always uses voice 1), `C022084`, `C002085`, `C012084/085`, `B012074..080`, `B102074..080` (defence counsel with mismatched gender digits). They look like a recorded voice matrix wider than the code uses.

## Corrections (rules-27)
- Rows `0224900/901`: three speech lines run back to back, each only if the previous was not clicked away (`speech_say` return value); line 3 has no `12·L` term. See rules-27 §2.
