# Report texts: placeholder codes, event messages and where they appear

Confidence tags as in `rules-00-map.md`: **[C]** read at instruction level (call-site argument order checked), **[L]** read from decompiler output only, **[G]** guess. Text addresses are linear addresses of NUL-terminated CP437 strings in the data object; "text id n" means entry n of the text pointer table at `0xB4258` (3373 entries, one 32-bit pointer each). The layout escapes (`$N`, `$Z`, `$L`, `$nH`, …) are in `formats.md` §8.3; this document covers the `%` codes and the report events.

Behaviour (amounts, conditions, who pays) is in the other rules docs; this document only adds which text is printed, with which arguments, and in which window.

## 1. `print` argument model [C]

`print(fmt, …)` (0x322C0) takes **all** arguments on the stack, left to right. Every `%` code consumes its arguments **sequentially** from that list; there is no positional selection. The digit in `%<k>X` is therefore **not** an argument index. It is a parameter of the code: a grammatical case for the person/noun codes, a form selector for `%F`, or ignored. A code without a digit has k = 0.

All persons are passed as the person index (0..191). Locations are location ids (0..20, `rules-00` §3). Nouns are text ids (see 1.3). Numbers are 32-bit signed.

### 1.1 Codes without arguments
| code | effect |
|---|---|
| `%%` | literal `%` |
| `%M` | invisible placeholder item (style 0x1630, empty text); reserves an item index |
| `%a` | expands to `$L$2>%m$E$B$6>` with a fixed bullet item (style 0x1090, glyph 0x73): a clickable "-" line; the text after it is the item's label |

### 1.2 Numbers, strings, icons, items
| code | args | effect |
|---|---|---|
| `%d`, `%b`, `%w` | number | decimal digits, `.` every three digits (1.234.567) |
| `%i` | number | as `%d`, with a leading `-` when negative |
| `%B`, `%C`, `%D` | number | as `%d` but with the alternate digit glyph set (font codes 0x10..0x19) [L: small digits] |
| `%K` | number | as `%B`; negative values go through the same formatter with the sign handled first [L] |
| `%l` | number | decimal **without** thousands separators (used for years) |
| `%c` | char | one glyph |
| `%s` (no digit) | char pointer | copy of the string (recursively parsed: a `%s` argument may itself contain `$`/`%` codes) |
| `%g` | char pointer | string followed by the genitive marker: `'` if the string ends in `s`, `z` or `x`, else `s` |
| `%G` | person | the person's name string followed by the same genitive marker |
| `%v` | any | consumes one argument, prints nothing |
| `%o` | location | location name from the table at `0xB6194` |
| `%m` | style, data | clickable menu item; returns the item index to the caller |
| `%n` | style, data, a, b | menu item with two extra 32-bit payload words (style \| 0x10000) |
| `%j` | icon id | clickable inline icon item (style 0x1090) |
| `%J` | icon id | inline icon, advance = icon width from the ICONIN width table (`0xC2DA0`) |
| `%I` | width, kind, pointer, height | inline sprite element with an explicit width [L] |
| `%*` | width, colour, value | inline bar element of `width` px (used for the score bars in Spielstand) [L] |

Line-break and `$` escapes inside `%s` strings are honoured because the parser recurses into the string.

### 1.3 Noun table and cases
Text ids 0..427 are a 4 × 107 grid: **row r = (k − 1), column = noun id** for the singular of case k (1 nominative, 2 genitive, 3 dative, 4 accusative). Rows 4..7 (ids 428..855) hold the plural in the same case order, but only columns 6..68 are filled (the rest of those rows is ordinary event text). So:
- singular noun id n in case k = text id `(k−1)·107 + n`;
- plural = text id `(k+3)·107 + n`.

Columns [C]:
| id | words |
|---|---|
| 0..2 | der, die, das (definite article, m/f/n) |
| 3..5 | ein, eine, ein |
| 6..8 | Euer, Eure, Euer |
| 9..11 | sein, seine, sein |
| 12..14 | ihr, ihre, ihr |
| 15..18 | adjective endings: 15 `e`; 16 `er`, 17 `e`, 18 `es` (m/f/n after Euer/ein) |
| 19..24 | Karren, Schiff, Ballon, Dampfkarren, Tauchboot, Flugmaschine (= 19 + transporter type) |
| 31..42 | Arbeiter, Räuber, Söldner, Kanonier, Musketier, Flintenschütz', Bombenleger, Offizier, Fuhrknecht, Matrose, Mann, Gefangene |
| 43..57 | production means Feld … Schmiede (= 43 + good) |
| 58..68 | Kate, Hütte, Haus, Bürgerhaus, Landhaus, Villa, Landsitz, Burg, Wasserburg, Schloß, Park |
| 71 | Warenlager (in the accusative row) |
| 72..105 | office names, male, by global office index (72 + o); plural row 4 = "Ratsherren" etc. |
| 106 | Besitzer |

Female office names are a flat run at text ids **1407..1440** (1407 + o): Ratsfrau, Baumeisterin, Richterin, Kämmerin, Bürgermeisterin, Priesterin, Domherrin, Geheimrätin, Hauptfrau, Justizberaterin, Finanzberaterin, Kanzlerin, Vogtin, Kellermeisterin, Sakristanin, Diakonin, Äbtissin, Bischöfin, Hofrätin, Marschallin, Justizministerin, Finanzministerin, Großkanzlerin, Regentin, Erzdiakonin, Inquisitorin, Erzbischöfin.

Gender class of a noun id: byte table `0xB8258[id]` (0 masculine, 1 feminine, 2 neuter). Values set for 20 Schiff (2), 23 Tauchboot (2), 24 Flugmaschine (1), 43 Feld (2), 44 Boot (2), 46 Schaf (2), 48 Weide (1), 49 Grube (1), 50 Faß (2), 53 Mine (1), 55 Mine (1), 56 Land (2), 57 Schmiede (1); everything else 0. The article/ending for a noun is looked up through the word table `0x84E02` (15 words): `[6 7 8 | 3 4 5 | 9 10 11 | 12 13 14 | 16 17 18]`, i.e. group g = 0 Euer/Eure/Euer, 3 ein/eine/ein, 6 sein/seine/sein, 9 ihr/ihre/ihr, 12 adjective endings; word = `0x84E02[g + gender]`.

### 1.4 Noun codes (count + noun id) [C]
All take `(count, id)` where `id` is a column of 1.3. "Singular" means count = 1, otherwise the plural row is used.

| code | prints |
|---|---|
| `%ks` | noun only, case k, singular/plural by count (`%4s`) |
| `%kS` | count, space, noun |
| `%ku` | singular: `ein/eine/ein` (group 3) in case k, space, noun; plural: count, space, noun. `%4u` with (5, 33) = "5 Söldner", with (1, 33) = "einen Söldner" |
| `%ke` | `Euer/Eure/Euer` (group 0) in case k (singular or plural row), space, noun. No number. `%4e` (3, 19) = "Eure Karren" |
| `%kE` | singular: `Euer…` + noun; plural: `Euer…` (plural row) + count + noun. `%4E` (5, 19) = "Eure 5 Karren" |
| `%kp` | takes `(person, count, id)`: `sein…` (group 6) if the person is male, `ihr…` (group 9) if female, then as `%E` |
| `%kP` | as `%p` but **without** a person argument: it reuses the person consumed by the previous `%N`/`%F`/`%q`/`%f` code (shares the same temporary). Treat as "possessive of the last named person" |

### 1.5 Persons: `%kN<n>` [C]
Arguments: one person. `k` = case, `n` = form (second digit, mandatory). Person fields used: gender (+0x00), title (+0x0D), name pointer (+0x09, a `char*` to the name string), place byte (+0x0E: low 6 bits = location, top 2 bits = prefix 0 "aus", 1 "zu", 2 "von", 3 "von und zu"), office location (+0x0F), office index within level (+0x11, 15 = none), controller/church byte (+0x10).

The 8 pieces: T = title (male table `0xB6A58`, female `0xB6A80`), X = title suffix, O = office noun (male id 72 + global office, female 1407 + global office, always nominative), N = name, P = place prefix, Q = place name, O again, R = office location name.

| n | with office | without office (index 15) |
|---|---|---|
| 0 | N | N |
| 1 | T X N | T X N |
| 2 | T X N P Q | T X N P Q |
| 3 | O N | N |
| 4 | O in R | "ohne Amt" |
| 5 | T X N, O in R | T X N, ohne Amt |
| 6 | T X N P Q, O in R | T X N P Q, ohne Amt |

Case k: k = 1 nominative; k = 2 genitive: N gets the genitive marker (`%g` rule), e.g. "Graf Hans' Gegenspieler", "Hansens" is never produced; k = 3 or 4 (dative/accusative) are identical and only change X. X = "n" when k ≠ 1 **and** the title is 0 ("Herrn Hans"); otherwise empty (titled persons are not declined: "Graf Hans" in every case). The office noun is never declined.

Quirk [C]: for k = 2 the form index is n + 11; for an office-less person with n ≥ 3 the index is n + 4 + 11 = 18..21, past the end of the 18-entry format table. `%2N3`..`%2N6` of an office-less person therefore prints garbage; the EXE texts only use `%2N0` and `%2N3`, and `%2N3` only for castle/camp owners (who always exist as persons, but may be office-less). Reproduce as "name" for safety.

### 1.6 Person pairs: `%kF` [C]
Arguments: `(a, b)`, b = 0xFF for "none". Expands to a fixed sub-format with the arguments `(a, b, a)`:

| k | b = none | b given |
|---|---|---|
| 0 | `%4N5` (a) | `%4N3, %2N0 %f` → "den Ratsherr Hans, Peters Gegenspieler" (a acc. office+name, b genitive name, a's gender for `%f`) |
| 1 | `%1N1` | `%1N0, %2N0 %f,` |
| 2 | `%4N0` | `%4N0, %2N0 %f` |
| 3 | `%1N4` | `%2N0 %f` |

`%f` (one person): "Gegenspieler", plus "in" if that person is female. `%q` (one person): "sein" / "ihr" (the text supplies the ending: `%qen` = seinen/ihren, `%qes` = seines/ihres). The "b" of every `%F` in the EXE is the human whose current rival (`human_rival_of`, 0x44F10) is a, or none.

### 1.7 Residences: `%kW` [C]
Argument: the residence word (property +0x03/+0x04 as u16: low 5 bits level 0..9, bits 5/6/7 extensions Park/Nebengebäude/Ländereien, high byte condition 0..100). Output: article, condition adjective, adjective ending, noun, extension clause.
- article: group 0 (Euer/Eure/Euer) when k ≤ 4, group 3 (ein/eine/ein) when k ≥ 5, in case row (k−1) mod 4; noun gender from `0xB8292[level]` = Kate f, Hütte f, Haus n, Bürgerhaus n, Landhaus n, Villa f, Landsitz m, Burg f, Wasserburg f, Schloß n;
- adjective from `0xB6F8C[condition / 21]`: 0..20 verwahrlost, 21..41 verfallen, 42..62 gut erhalten, 63..83 schön, 84..104 prächtig (values above 104 index past the table);
- ending from group 12 in the same case row (`er/e/es` nominative, `en/en/en` genitive and dative, `en/e/es` accusative);
- noun from the residence table `0xB4340` (its own 4 case rows: Haus/Hauses/Haus/Haus, Schloß/Schlosses, Landsitz/Landsitzes, Park/Parkes);
- extension clause by bits 5..7: none; "mit Park"; "mit Nebengebäuden"; `[EXE text]`; "mit weitläufigen Ländereien"; `[EXE text]`; `[EXE text]`; `[EXE text]` (fixed dative forms regardless of k).

Examples: `%1W` = `[EXE text]`, `%3W` = "Eurem schönen Haus", `%5W` = "ein verfallenes Bürgerhaus", `%8W` = "einen gut erhaltenen Landsitz" (k = 8 → accusative row, indefinite).

## 2. Windows used by the reports [C]

| screen | how it is built |
|---|---|
| **Turn-end report** (turn_end_settlement 0x61A70) | full-screen window style 0x8100; icon 0xB7 at (0,0); banner (`0x49D60`) for the current player with title "Ereignisse"; text window `win_open(30, 110, 520, 290, 0xA810)`. First page: "Eure Ausgaben" bill, wait for click, new page (`0x30F10`, same as `$P`). Then the texts of §3 flow in order; `$V` waits for a click when something was printed, `$P` starts a new page. When a film plays in between, the whole screen is rebuilt (banner + the same 0xA810 window) and printing continues with `$B` (justified) |
| **Year-end report** (year_end_world 0x66220) | first a full-screen 0x8100 window for the office cycle (dismissals and elections, each on a place picture drawn by `0x66160`; every message waits for a click). After the auction settlement: full-screen window 0x8910, icon 0xB8 at (0,0), banner for player 0xFF ("Ereignisse in Mittelland, A.D. year"), text window `win_open(100, 126, 440, 300, 0xA810)`. The herald, sabotage, toll, castle, mistress gossip, auction announcement, siege, divine and chronicle texts are printed into it in that order. At the end `$V`; if the cursor is still on line 12 (nothing printed) `[EXE text]` (0xC2024) is shown and the game waits for a click |
| **Message box** (`0x368E0(x, y, w, h, fmt, …)`) | modal box; used for the divine events (160,100,320,280; tornado 80,80,480,320) and the human's mistress dialogs (160,140,320,200) |
| **Auction** (0x27680 / 0x26A50) | see §3.6 |

The human's own turn-end messages (§3.1..3.4, 3.8 handler texts) appear in the turn-end report; everything the world does at year end appears in the year-end report, except where a message box is named.

## 3. Report events

Argument lists are given in print order (= the order the `%` codes consume them).

### 3.1 Too many branches (0xC0874) [C]
Turn end, right after the bill page: if the player's branch count B exceeds `MaxBr` (`0x85A2C`, `rules-02` §7.10): sound cue 8, then
`[EXE text]` with (emperor name `0xB707C[type-1 record byte 4]` = Rudolf II., Matthias, Ferdinand II., Ferdinand III., Leopold I.; percent = `(B − MaxBr) · 7`; amount = `percent · net_worth / 100`). The amount is then paid; wait for click; new page.

### 3.2 Office income (0xC08EC) [C]
If `office_income` (0x5E500) > 0: `[EXE text]` with (player, amount), then the amount is added.

### 3.3 Castle toll income (0xC0934, 0xC09AC, 0xC09C4) [C]
For every toll castle the player owns (property kind 2, in slot order), k = location − 14: the accumulator `acc` (castle table +0x15) is first increased by `rate · 7 · 0x85E1D · 0x85E21 / 400000` (see `rules-02` §7.12), `share = acc · 0x85A48[k] / 100` (the county's share, default 25 %). If `acc` ≠ 0:
1. `[EXE text]` with (castle location, **acc**, **share**, 10 + k). First `%d` = gross accumulator, second = county share.
2. If the county's Vogt exists (office index 7 at location 10 + k, any controller) and is not the player: `vogt = acc · 30 / 100`; ` %1N3 erhält %d Taler.` (0xC09AC) with (Vogt, vogt), and the Vogt receives it.
3. `$N$N` (0xC09C4); the player receives `acc − share − vogt`; `acc := 0`.
Then the tips (0xC09CC..) and fate events follow in the same window.

Year end, computer-owned castles (0xC1D14): after the computer owner's accumulation, if a **human** Vogt of that county exists and `acc · 30 / 100 > 0`: `%1N3 erhält %d Taler Zolleinnahmen aus Burg %o.` with (Vogt, amount, castle location) in the year-end report; the Vogt receives the amount.

### 3.4 Loan due (0xC1060) [C]
Turn-end record pass, type 56 whose counter reached 0: `[EXE text]` with (lender, lender, amount +5). Then `favour_decrease(borrower, lender, 8)`, the borrower pays the amount (cash may go negative), with `rand(100) < 10` the lender starts revenge code 2 (0x5B140), and the record is deleted. [The record's own +3 counter is the trigger; no grace.]

### 3.5 Auctions
**Announcement** (0x269D0, year end, 20 % when the pause counter `0x85E06` is 0 and no type-3 record exists; `rules-08` §1.1) [C]: the record is created with counter `c = 2 + rand(3)` and the year-end report gets `[EXE text]` (0xBAB68) with (`year + c + 1`). This is the year in which `auction_settle` will run (counter decremented at the following year ends, auction held when it is 0).

**Herald** (0xC19D0) [C]: at the very start of the year-end "Ereignisse" window (before sabotage), if a type-3 record exists **and its counter is 1** at that moment: sound cue 0x1E and `[EXE text]` (no arguments). Since the world record pass decrements the counter later in the same year end, this appears exactly one year before the auction, i.e. in year `announcement year + c`. In the announcement year itself only 0xBAB68 is shown.

**Auction day** (`auction_settle` 0x27680, year end when the counter is 0, before the Ereignisse window) [C]:
- Screen: full-screen window 0x8910, icon 0xC1 (hall) at (0,0), sound 0x12, music 0x4B; caption box `0x36C90(10, 20, 620, 80, 0, fmt, …)` with `[EXE text]` (0xBAE00) and (year, **rand(10)**): the city named in the header is random and has no effect [C].
- Catalogue: for every lot, in record order, types 12, then 14, then 16: icon 0xD4 (book) at (0,40), icon 0x22 at (0,0), `$H$Z$C^ Katalog _` (0xBAE24), left page `$t$|$4H$10>$56<$Z$C^ Position %d _$N$N%J$N` with (running number from 1, icon id), right page:
  - transporters (0xBAE64): `[EXE text]` with (count +5, 19 + type +4, seller, condition +6, start price); left-page icon = `0x84CD4[type]` [L: transporter picture table];
  - residence (0xBAF20): `[EXE text]` with (residence word, branch location, seller, start price); icon `0x160 + (residence & 0x0F)`;
  - castle (0xBAFD0): `[EXE text]` with (seller, condition adjective `0xB6F8C[security / 21]` (no digit: plain string, the text appends "e"), castle location, troop total, 33 (Söldner), casemates +4, start price); icon `0x15C + (location − 14)`.
  Each catalogue page waits for a click.
- Attendance (only when no seller is human): window `win_open(120, 240 − h/2, 400, h, 0x8810)` with h growing with the number of humans [C for x, w; L for h], `^ Euer Bargeld _` then one line per human `$L%1N0$E$R%d Taler` (0xBB0C0; negative cash uses 0xBB0D8 with the absolute value highlighted), then `[EXE text]` with items "Ja"/"Nein" (0xBB100). Nobody attends → all lots dropped (0xBB228 below).
- Per lot (line 16 of the auction screen, icon 0xD2 auctioneer at (0,0); 0x1BD/0x1B5 lot pictures at (44,…)/(56,…) [L]):
  - intro: transporters text id 3100 `[EXE text]` (count, 19 + type, seller); residence 3101 `[EXE text]` (word, location, seller); castle 3102 `[EXE text]` (castle name string); wait for click.
  - bidding (`auction_main` 0x26A50): first tick `$15H` + text id 3370 `[EXE text]` (start price). Ticks without a bid print `$N` + one of text ids 2226..2231 chosen by `rand(6)` (`>%d Taler - Kommt schon, …<`, `[EXE text]`, `… Das Gebot steht. Bietet jemand mehr?<`, `… Das ist geschenkt!<`, `[EXE text]`, `[EXE text]`) with (current price), and play speech sample 900 + index. A bid prints `$16H` + `%1N3 bietet %d Taler!` (0xBADAC) with (bidder, new price). Hammer counts print `$16H` + text ids 2839..2843 (`>%d Taler zum Ersten...<`, same, `>%d Taler zum Zweiten...<`, `>%v...zum Zweiten...<`, `>%d Taler zum Dritten!<`) with (price). Refused human bids print at `$15H`: no branch in the city (0xBABB0, residence lots), asleep/already bid (0xBAC1C), not enough cash (0xBAC78), no port branch (0xBACD8), in the debtors' tower (0xBAD48), each with (human).
  - unsold: `$16H` + `[EXE text]` (0xBADC4), wait for click.
  - sold (line 15): transporters text id 3037 `[EXE text]` with (winner, count, 19 + type, location of the receiving branch), or 3040 `[EXE text]` (winner) when the winner has no branch to receive them; residence 3038 `[EXE text]` with (winner, word, city name string); castle 3039 `[EXE text]` (castle name string, winner). A human winner who already owns a residence in that city gets the choice 0xBB14C `[EXE text]` (winner, city, old word, old value, new word, new value) [L for the value arguments].
- End: `[EXE text]` (0xBB1D0) if at least one lot was offered, else `[EXE text]` (0xBB228); wait for click.

**Castle for auction, year end** (0xC1E60) [C]: when a computer owner puts his castle up (`rules-07` §3 last step): `[EXE text]` with (castle location, year + counter of the type-3 record). Private sale offer to a human (0xC1D4C): `[EXE text]` with (candidate, owner, castle location, price), answered by a menu; a computer buyer gives `[EXE text]` (0xC1DD8) with (location, price, buyer). Both in the year-end report.

### 3.6 Sabotage (year end, `rules-04b` §4b) [C]
In the year-end report, for every striking saboteur contract (owner s, target t, property x of t):
- Castle (kind 2): `[EXE text]` (0xC1A8C) with (s, location of x, t, D). "%2N0 Saboteure" = "Hans' Saboteure" (genitive name of the saboteur **owner**); there is no count of saboteurs anywhere in the text or the code (a contract is one record; the number of saboteurs is not a game quantity).
- Camp (kind 1): 0xC1B28 `[EXE text]` with the same four arguments.
- Branch (kind 0): if `human_rival_of(t) == s` **and** s is a computer person **and** t is human: 0xC1BB4 (t male: "… auf dessen Niederlassung …") or 0xC1C54 (t female: "… auf deren Niederlassung …"): `[EXE text]` with (s, t, s, location of x, D) → `[EXE text]`. Otherwise text id 3139 `[EXE text]` with (s, t, location of x, D) → `[EXE text]`.
- t owns no usable property: text id 3141 `[EXE text]` with (s, t, amount), and t pays the amount.
- Insurance bit 3: text id 3140 `[EXE text]` with (t), then the refund.
- Film: once per year end with 10 % (the flag `rand(100) < 90` is rolled before the loop): after the message `$V`, sound 0x0D, the window is torn down, `flcfilme\sabotage.flc` plays, the banner and the 0xA810 window are rebuilt and `$B` printed.
D is the property's value loss (minimum 2 for branches), `rules-04b` §4b.

### 3.7 Office dismissal and elections (year end, `rules-03` §4.3, §4.6) [C]
All on the first full-screen year-end window, each text preceded by the place picture (`0x66160(place, mode)`), followed by sound cue 8 if the person concerned is human, then a click.

Dismissal of p (shown only if p is human, or p holds a principality office (office location 13), or p is some human's current rival): the dismissers are the **computer** appointers whose favour toward p fell below 30, in appointer-slot order (`0x87318` table, `rules-03` §3); `h` = the human whose rival is p (0xFF if none, which switches `%0F` to its "no b" form).
| dismissers | text |
|---|---|
| 1 | `[EXE text]` (0xC170C) with (a1, p, h) |
| 2 | `[EXE text]` (0xC1734) with (a1, a2, p, h); if the office has 3 appointers and the third one (the one who did not dismiss) is not h: `[EXE text]` (0xC1768) with (that appointer) |
| 3 | `[EXE text]` (0xC1790) with (a1, a2, a3, p, h) |

Election of a vacant office (global index o at place L), only when a human is involved (a human candidate or a human elector; `local_40 || local_1c` in the code) [C for texts, L for the exact "human involved" condition]:
1. Place picture, strip `0x13539` at (0, 420, 640, 60), `[EXE text]` (0xC17C8) with (office name `0xB4378[o]`, L).
2. One line per candidate c: `$L$20>%m$E$25>$(%1N0, %1N4$)$|` (0xC17F8) with (0x90, 0x73, c, c), or, if c is some human's rival, `$L$20>%m$E$25>$(%1N0, %2N0 %f$)$|` (0xC1818) with (0x90, 0x73, c, that human, c).
3. Privileged candidate x (Erzdiakon/Diakon/Sakristan rule): `[EXE text]` (0xC183C) with (x, x, x). Click.
4. Per elector e: human: `$C%1N3, wählt einen Kandidaten!$N$H` (0xC18A0) with (e), or `[EXE text]` (0xC18C4) with (voting human, blackmailed appointer); then a menu over the candidate lines. Computer: `$C%1N3 stimmt für...$N$H` (0xC190C) with (e), then `$C%4N0!$N$H` (0xC1928) with (chosen candidate).
5. Result (w = winner, h = human whose rival w is or 0xFF, office noun = text id 72 + o for a male, 1407 + o for a female winner): majority `$CDamit wird %1F %s in %o.$N$H` (0xC1934) with (w, h, office noun, L); tie `[EXE text]` (0xC1954) with (w, h). A human winner of a Kanzler (16), Bischof (22) or principality office (> 28) also gets a history entry (0x2D750). Sound cue if w is human; click.
6. Without an election: single candidate `[EXE text]` (0xC1998) with (w, h, office noun, L), shown only if w is human or w is some human's rival; one candidate drawn by the computer rule: `$8H$C$g%1F wird %s in %o.$N$H` (0xC1978), same arguments, shown only if w is some human's rival.

The year-end report later lists the offices that will be elected next year: `[EXE text]` (0xC1FD4) then per office `%s in %o` (0xC2004, name table `0xB4A28`, L) or `Der %s von %o` (0xC2010, `0xB4378`) [L: which table for which office kind], then a click.

### 3.8 Mistress / lover (`rules-05` §5) [C]
- Acquire, human (privileges menu): message box (160,140,320,200): already has one → text id 2404 `[EXE text]` with (" " or "n " by gender, "Mätresse" / "Geliebten"); success → text id 2407 `[EXE text]`
- Acquire, computer person p (year end, 10 draws): if a human owns a property at p's location (`0x85E08`) and `rand(100) < 6`: gossip line text id 2401 `$L%1N5, %s.` with (p, text id 2408 + rand(5)) = "geht fremd", `[EXE text]`, `[EXE text]`, `[EXE text]`, "feiert wüste Orgien"; in the year-end report.
- Yearly handler (type 29), human keeper, in the turn-end report: counter 0 → text id 2851 `$LEu%s %s verlässt Euch!` with ("re"/"er", "Mätresse"/"Geliebter" by the keeper's gender: "Eure Mätresse" / "Euer Geliebter"); otherwise text id 2856 `[EXE text]` with (the same two, `standing(keeper, 10)`), amount not deducted (bug noted in `rules-05`).
- Related fixed texts: `[EXE text]` (0xC0034) / "… einen Geliebten." (0xC0074) in the church screen.

### 3.9 Fate events for a human (0x226F0, `rules-06` §4) [C for ids and argument order]
All printed into the turn-end report (after `$B`, 0xBA37C). Most kinds use text id **3270 + k − (number of silent kinds below k)**; the exact ids are listed. Arguments in print order; "branch" = the drawn branch, "city" = its location (`%s` with a location name string from `0xB6194`, or `%o` with the id, as the text says). Amount = the value described in `rules-06` §4.

| k | text id | arguments |
|---|---|---|
| 0 | 3270 `[EXE text]` | city name, amount |
| 1 | 3271 `[EXE text]` | city name, amount |
| 2 | 3272 `[EXE text]` | amount |
| 3 | 3273 `[EXE text]` | amount |
| 4 | 3274 `[EXE text]` | x, x, 1, 71 (→ "sein/ihr Warenlager"), x |
| 5 | 538 `[EXE text]` then 539 + v (rescue / knocked out) | city name; none |
| 6 | 3275 `[EXE text]` | city name |
| 7 | 3276 `[EXE text]` | amount |
| 8 | 3277 `[EXE text]` | amount |
| 9 | 3278 `[EXE text]` | amount |
| 10 | 3279 `[EXE text]` | amount |
| 11 | 3280 `[EXE text]` | tavern name (`0xB5AE4[rand(30)]`), amount |
| 12 | 3281 `[EXE text]` | tavern, amount |
| 13 | 3282 `[EXE text]` | tavern, amount |
| 14 | none | |
| 15 | 3284 `[EXE text]` | city name |
| 16 | 3285 `[EXE text]` | city id |
| 17 | 3286 `[EXE text]` | book title (`0xB5530[rand(49)]`), city name |
| 18 | 3287 `[EXE text]` | book, city name |
| 19 | 3288 `Eine schwere Krankheit streckt Euch danieder. Ihr habt %s; …` | illness (`0xB683C[rand(12)]`: Schnupfen & Husten … Zahnausfall) |
| 20 | 3289 `[EXE text]` | amount |
| 21 | 3290 `[EXE text]` | amount, city name, county name |
| 22 | 3291 `[EXE text]` | amount |
| 23 | 3292 `[EXE text]` | none |
| 24 | 2908 `[EXE text]` | location name of h, x, 2909/2910 ("den eitlen Pfauen"/"die eitle Matrone" by x's gender), 2911/2912 ("seiner"/"ihrer") |
| 25 | 645 `[EXE text]` | `rand(16) + 2`, text id `646 + rand(2)` ("nicht mehr" / "wieder ganz viel") |
| 26 | 3293 `[EXE text]` | amount |
| 27 | 3294 `[EXE text]` | amount |
| 28 | 3295 `[EXE text]` | amount |
| 29 | 3296 `[EXE text]` | residence word |
| 30 | 3297 `[EXE text]` | city id |
| 31 | 3298 `[EXE text]` | city name |
| 32 | 3299 `[EXE text]` | host x, county name |
| 33 | 3300 `[EXE text]` | city name, amount |
| 34 | 3301 `[EXE text]` | none |
| 35 | 752 `[EXE text]` | amount, 753 + v (`… im Gasthaus >%s< …`, which itself takes the tavern name) |
| 36 | 3302 `[EXE text]` | amount |
| 37 | 3303 `[EXE text]` | city name (rand(10)), amount |
| 38 | 3304 `[EXE text]` | amount |
| 39 | 3305 `[EXE text]` | city name |
| 40 | 3306 `[EXE text]` | city name |
| 41 | none | |
| 42 | 3308 `[EXE text]` | city name, amount |
| 43 | 3309 `[EXE text]` | amount, city name |
| 44 | 3310 `[EXE text]` | city name |
| 45 | 3311 `[EXE text]` | book, good name (`0xB6618[g]`) |
| 46 | 3312 `[EXE text]` | book, good name |
| 47 | 3313 `[EXE text]` | x, amount |
| 48 | 3314 `[EXE text]` | city name |
| 49 | 3315 `[EXE text]` | city name |
| 50 | 3013 `[EXE text]` (continues with the helpers' payment `%d`) | city name, text id 3014 + rand(3) ("weiß-blaue", "grüne", "rote"), amount 1 + rand(1000) |
| 51 | 3073 `[EXE text]` | x, text id 3074 + gender(x) ("Sein" / "Ihr") |
| 52 | 3316 `[EXE text]` | city name |
| 53 | 3317 `[EXE text]` | residence word, city name, amount |
| 54 | 3318 `[EXE text]` | new residence word (type-13 record +4), city name, amount |
| 55 | 3319 `[EXE text]` | city name |
| 56 | 3320 `[EXE text]` | good name, destination name |
| 57 | 3076 `[EXE text]` | x, text id 3077 + gender(x) ("Er" / "Sie") |
| 58 | 3321 `[EXE text]` | none |
| 59 | 3322 `[EXE text]` | castle name |
| 60 | 3323 `[EXE text]` | camp name |
| 61 | 3324 `[EXE text]` | transporter count (+8), 19 + type (+9), destination name |
| 62 | 3325 `[EXE text]` | count, 19 + type, old destination name, new destination name |
| 63 | 3326 `[EXE text]` | 1, 19 + type |
| 64 | 3327 `[EXE text]` | 1, 19 + type, owner of the receiving branch |
| 65 | 3328 `[EXE text]` | count, 19 + type |
| 66 | 3329 `[EXE text]` | good name, destination name |
| 67 | 3330 `[EXE text]` | destination name, good name |
| 68 | 3331 `[EXE text]` | old destination name, new destination name |
| 69 | 3332 `[EXE text]` | city name |
| 70 | 3333 `[EXE text]` | city name |
| 71 | 3334 `[EXE text]` | city name |
| 72 | 3335 `[EXE text]` | per-worker amount (2 + rand(5)) |
| 73 | 3336 `[EXE text]` (ends with `%d` total) | per-worker amount, total |
| 74 | 3337 `[EXE text]` | city name, amount |
| 75 | 3338 `[EXE text]` | x, amount |
| 76 | 3339 `[EXE text]` | city id |
| 77 | 3340 `[EXE text]` | rand(10) + 2, amount |
| 78 | 3158 `[EXE text]` or 3159 `[EXE text]` | residence word with the extension bits masked (`& 0x1F` on the low byte), city id |
| 79 | 3341 `[EXE text]` | camp name |
| 80 | 3342 `[EXE text]` | camp name |
| 81 | 3343 `[EXE text]` | camp name |
| 82 | 3344 `[EXE text]` | nickname (`0xB6AA8[rand(18)]`), epithet (`0xB6410[rand(10)]`), camp name |
| 83 | 3345 `[EXE text]` | camp name, amount |
| 84 | 3346 `[EXE text]` | camp name |
| 85 | 3347 `[EXE text]` | camp name, receiving owner |
| 86 | 3348 `[EXE text]` | giving owner |
| 87 | 3349 `[EXE text]` | camp name, nickname, epithet, amount |
| 88 | 3350 `[EXE text]` | castle name |
| 89 | 3351 `[EXE text]` | castle name |
| 90 | 3352 `[EXE text]` | castle name |
| 91 | 3353 `[EXE text]` | castle name |
| 92 | 3354 `[EXE text]` | castle name, amount |
| 93 | 3355 `[EXE text]` | castle name |
| 94 | 3356 `[EXE text]` | castle name, receiving owner |
| 95 | 3357 `[EXE text]` | giving owner |
| 96, 97 | 3358 `[EXE text]` | castle name (96 passes two extra cosmetic strings that the text ignores) |
| 98 | 2640 `[EXE text]` | castle name, relic 2641 + rand(9) (die Bundeslade, ein heiliger Kelch, ein Nagel vom Kreuz Christi, …) |
| 99 | 3160 `[EXE text]` (castle) or 3161 `[EXE text]` (camp) | location id |

Kinds 14 and 41 print nothing. Where `rules-06` says "message only", the text above is the whole effect.

### 3.10 Divine events (0x59940, `rules-06` §3.1) [C]
Each is a message box at (160, 100, 320, 280) with text id **2350 + k** (`$Z$L…`): 0 weeping Madonna (city name, city name); 1 divine health (city name, illness name — the text's `%v` swallows it); 2 lightning (city name, city name); 3 insanity, text 2353 `[EXE text]` (x, x); 4 drought (city name); 5 ideal weather (city name); 6 inspiration (city name, illness name swallowed by `%v`); 7 summer snow (none); 8 tornado, text 2358 at (80, 80, 480, 320) with (city A name, then nine `%w` numbers and names in text order: churches, museums, hospitals, inmates, A again … B name, …) — the exact mapping of the nine random numbers follows the `%w`/`%s` order of the text and is [L]; 9 epidemic (city name, illness name); 10 silver strike (none).

### 3.11 World fate (0x26640, `rules-06` §3.2) [C]
Printed into the year-end report after `$B`: 0 → text id 3360 `[EXE text]` (good name); 1 → 3361 (good name); 2 → 3362 boom (city name); 3 → 3363 decline (city name); 4 → 3079 `[EXE text]` (besieger name `0xB686C[rand(12)]`, city name) followed by 3080 `Sämtliche Warentransporte müssen unverrichteter Dinge wieder umkehren.` when shipments are redirected, else 3081 `[EXE text]` (besieger name).

### 3.12 Other year-end report lines met while tracing [C]
`$L%1N5, saniert Burg %o.` (0xC1E00: computer castle owner repairs, owner, location); `[EXE text]` (0xC1E20); `[EXE text]` (0xC1ED4), `… aus Rom.` (0xC1F30), `… aus Ablassverkäufen.` (0xC1F7C), each with (human, human, amount); `[EXE text]` (0xC1E90) before a divine event triggered by prayers.

## 4. Open / unverified
- The exact `human involved` condition that gates the election screens (step 3.7) and the two office-name tables of the "next year's elections" list are [L].
- `%*` and `%I` element semantics beyond "inline bar / inline sprite with explicit width" [L].
- `%B/%C/%D/%K` digit glyph set (0x10..0x19) is read from the formatter; which glyphs the font has there is [UNVERIFIED].
- The tornado text's nine numbers (3.10 k = 8) are listed in `rules-06` §3.1 as cosmetic draws; their order relative to the `%w` codes is [L].
