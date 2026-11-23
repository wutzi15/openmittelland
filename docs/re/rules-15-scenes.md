# Scenes: full-screen and modal presentation around events

Confidence tags as in `rules-00-map.md`: **[C]** read at instruction level (register/stack arguments checked), **[L]** read from decompiler output only, **[G]** guess. Text addresses are linear addresses of CP437 strings (escapes in `formats.md` §8.3, `%` codes in `rules-12` §1). Icon ids are hex ICONIN indices; the file name follows `formats.md` §1 (`hi` = id/16 as base-36 digit, `lo` = id%16 hex; ids whose index flag has bit 0x80 live in the `B` directory). "640×480 picture" means a full-screen picture drawn at (0,0). Coordinates are screen pixels, (x, y, w, h).

Behaviour (who pays, RNG order, record changes) is in the other rules docs; this document only adds what is on the screen, in which order, and what the player must do to continue. `rules-12` §2 covers the two report windows; everything below that is not a report window is a *scene*.

## 0. Drawing primitives used by the scenes [C unless marked]

| primitive | code | behaviour |
|---|---|---|
| `picture(id)` | `0x2E310(x=0, y=0, id)` | draws the icon at (0,0). Scenes always use (0,0) for the backdrop |
| `bank_frame(x, y, bank, frame)` | `0x2E420` | draws frame `frame` of a bank icon at (x, y); loads the bank if needed |
| `win_open(x, y, w, h, style)` | `0x2FA80` | opens a framed window (`formats.md` §7.2). Styles seen in scenes: 0x8100 and 0x8910 (full-screen backdrops), 0x8800 (caption box), 0x8810 (message box), 0x8500 (matchmaker), 0xA810 (report text window). Which bits select frame/scroll/transparency is [UNVERIFIED]; the implementer may draw all of them as parchment frames except where the text says a box is drawn over a picture |
| `flip` | `0x30400` | presents the work frame |
| `win_close` | `0x30190` | closes the topmost window and restores what was under it |
| `items_clear` | `0x49EB0` | removes all clickable items |
| `region copy` | `0x13539(dst, dx, dy, src, sx, sy, w, h)` | copies a rectangle between the screen and an off-screen frame; used to back up a strip of a picture before text is printed on it and to restore it before the next text |
| `wait_click` | `0x36720` | waits for a mouse click or key |
| `delay(ms)` | `0x6C083` | millisecond pause [L: scaled by `0xEF4E4`] |
| `tick(n)` | `0x12DB5(n)` | n timer ticks (DOS clock, 1/100 s granularity) [L] |
| `speech(slot)` | `0x15100(slot)` + `0x154B0` | preloads and plays the speech sample `SPCxxKHZ/<4 digits><slot as 3 digits>.WAV` (`formats.md` §8.1); `0x15570(…, slot)` plays it and waits until it ends or a key is hit |
| `sound(n)` | `0x41E50(n)` | sound cue n |
| `music(n)` | `0x124E1(n)` | starts song/sample n; `0x12730(n)` stops it |

### 0.1 Caption box `0x36C90(x, y, w, h, flag, fmt, …)` [C]
A window of style 0x8800 at (x, y, w, h) is opened, `$g` is printed, the text cursor is set to y = 2 inside the box (not the usual 12), the text is printed. `flag = 0`: wait for a click; `flag ≠ 0`: wait `flag` ms [L]. Then the box's items are removed and the window is closed. There is no OK button. Most scenes use boxes 600 or 640 px wide across the bottom of a picture; the box has the parchment frame of `formats.md` §7.2 [L: no special-casing seen for style 0x8800].

### 0.2 Caption box with speech `0x36E50(0, 2, 2, v, slot, x, y, w, h, fmt, …)` [C for arguments, L for wait rule]
As 0.1 but the speech sample `slot` (voice variant digits 0, 2, 2, v) is started before the text is printed; the box stays until the sample has finished (if the sample is not playing it waits for a click instead); the speech is stopped and the box closed.

### 0.3 Caption menu `0x36AC0(x, y, w, h, fmt, …)` [C]
As 0.1 but the text contains `%a` lines; the box waits for a click on one of its items and returns `clicked − first item` (answer index, `rules-13` §1.2).

### 0.4 Message box `0x368E0(x, y, w, h, fmt, …)` [C]
Window of style 0x8810 at (x, y, w, h); text; an OK item (style 0x1690, glyph 0xCD) at (right − 32, bottom − 32) of the window [L for the corner]; waits for a click on any item of the box (the OK glyph or a `%a` line), returns `clicked − first item`; closes. When a box is already open (`0x84D46 ≠ 0`) no window and no OK item are created and only the text is printed.

### 0.5 Film `film(name, x, y, d)` [C]
Always the same sequence: the current window is closed (`win_close`, items cleared), `0x2F970` resets the icon cache and `0x2F4B0` later empties it (every picture is reloaded after a film), the FLC `flcfilme\<name>.flc` is opened (`0x14D10`) and played by `0x14ED6(x, y, d)` with `d`/100 s per frame (`formats.md` §8.1: d = 10 → 10 fps, d = 12 → 8.3 fps), then closed (`0x15045`). Esc aborts. After the film the caller rebuilds its screen (see each scene). Films seen:

| file | x, y, d | where (section) |
|---|---|---|
| sabotage.flc | 0, 0, 10 | year end, §5.1 |
| belager.flc | 0, 0, 12 (human besieger) / 0, 0, 10 (computer) | §5.2 |
| entlager.flc | 0, 0, 12 (twice, human) / 0, 0, 10 (twice, computer) | §5.2 |
| feuer.flc, lawine.flc | 0, 0, 12 | city disaster, §5.3 |
| informnt.flc then informnb.flc | 0, 24, 10 then 96, 100, 10 | spy evidence, §5.4 |
| kerze.flc | 0, 0, 10 | death, §7.4 |
| hochzeit.flc | 0, 0, 12 | wedding, §7.1 |
| outro.flc / outrob.flc | 0, 24, 10 / 96, 100, 10 | mission accomplished, §7.8 |
| intro.flc / introb.flc | start-up (`formats.md` §8.1) | not a game scene |

The x, y of the two-part films (informant, outro) are the EAX/EDX arguments of the frame loop [L: that these are screen coordinates].

## 1. Auction day (`auction_settle` 0x27680, `auction_main` 0x26A50) [C unless marked]

Trigger: year end, type-3 record with counter 0 (`rules-08` §1.1), before the "Ereignisse in Mittelland" window.

### 1.1 Screen
1. `win_open(0, 0, 640, 480, 0x8910)`.
2. A 217×164 off-screen frame is allocated (`0xC25B0`). Frame 0 of bank **0xD5** (B/ICOND5, the auctioneer at his lectern, 640×480 base frame) is drawn at (0,0) and the rectangle (196, 28, 217, 164) of it is copied into that frame: this is the *auctioneer patch* (the lectern and the man behind it).
3. Picture **0xC1** (ICONC1: brick stairs leading to an arched door, the hall entrance) at (0,0); `sound(0x12)`; `music(0x4B)`; flip.
4. Caption box `(10, 20, 620, 80, 0)` with `[EXE text]` (0xBAE00) and (year, `rand(10)`) — the city is random (`rules-12` §3.5).

### 1.2 Catalogue
For every lot (types 12, then 14, then 16, record order; running number n from 1): the start price is computed (`auction_main(price, 10)`), then
- picture **0xD4** (ICOND4, open book) at (0, 40); icon **0x22** (banner strip) at (0, 0);
- `$H$Z$C^ Katalog _$N` (0xBAE24) on the banner;
- left page: `$t$|$4H$10>$56<$Z$C^ Position %d _$N$N%J$N` with (n, picture id); the picture id is `0x84CD4[type]` = **0x15A, 0x15C, 0x15E, 0x15B, 0x15D, 0x15F** (B/ICONLA..LF, 144×144: Karren, Schiff, Ballon, Dampfkarren, Tauchboot, Flugmaschine) for transporters, **0x160 + (residence & 0x0F)** (B/ICONM0..M9, 160×120 residence by level) for residences, **0x15C + location** = 0x16A..0x16C (B/ICONMA..MC, 160×120, the three toll castles at locations 14, 15, 16) for castles;
- right page: the Exposé texts of `rules-12` §3.5 (0xBAE64 / 0xBAF20 / 0xBAFD0), `$|$4H$56>$10<` column;
- wait for a click.
The catalogue is skipped entirely when there are no lots.

### 1.3 Attendance question
Only when no seller is human: `win_open(120, 240 − h/2, 400, h, 0x8810)` with h = 72 + 24·(number of humans) [L for the formula], `$Z$C^ Euer Bargeld _$Z`, one line per human (0xBB0C0 / 0xBB0D8), then `[EXE text]` with two items of style 0x1030 labelled "Ja" (0xBB0FC) and "Nein" (0xBB0F4); wait for a click on one of them; close. "Nein" by everybody drops all lots (1.6).

### 1.4 Per-lot screen
1. Picture **0xD2** (B/ICOND2: the auction room, red carpet, lectern and blackboard, no auctioneer) at (0,0); the auctioneer patch is copied back to (196, 28) so the auctioneer stands at his lectern.
2. On the first lot a second off-screen frame (`0xC25B2`, 620×130) receives a copy of the screen rectangle (10, 350, 620, 130): the *text area patch*. Before every new line of auction text this patch is copied back, which erases the previous text.
3. Lot picture: transporters `bank_frame`-less `0x2E310(445, 44, 0x84CD4[type])` (144×144 at x = 445); residence `0x2E310(437, 56, 0x160 + level)` (160×120); castle `0x2E310(437, 56, 0x15C + location)`. The rules-12 note "0x1BD/0x1B5" is wrong: 0x1BD = 445 and 0x1B5 = 437 are **x coordinates**, the pictures are the ones listed in 1.2.
4. Intro text at line 16 (`$g$16H`, y = 12 + 16·24 = 396): text id 3100 / 3101 / 3102 (`rules-12` §3.5); wait for a click.
5. `auction_main(price, kind)`: every tick restores the text area patch, prints `$15H` (y = 372) and then the texts of `rules-12` §3.5. Speech: the idle lines (table `0xB6520[rand(16)]`, 16 entries) are spoken with speech slot `900 + index` and the game waits for the sample to end (`0x15570`), the hammer counts "zum Ersten/Zweiten/Dritten" with slot `0x394 + count` (916..918), the closing line with slot 0x397 (919). Bids and refusals are not spoken. Refused human bids are message boxes `(140, 160, 360, 160)` (0xBABB0, 0xBAC1C, 0xBAC78, 0xBACD8, 0xBAD48).
6. Auctioneer animation: four banks **0xD5..0xD8** (B/ICOND5..D8, 63..105 frames each; frame 0 is the base, later frames are small deltas of face and hands). The bank used is `0xD5 + (high bidder's person index mod 4)`; frames are drawn at (0,0) with `bank_frame`, the (196, 28, 217, 164) rectangle is presented after each frame, `tick(5)` between frames. Frame ranges come from the table `0x84C84` (5 words per bank: 0x1B 0x27 0x3F 0x0E 0x38 / 0x1F 0x2B 0x43 0x13 0x3C / 0x23 0x2F 0x47 0x16 0x40 / 0x23 0x2F 0x47 0x18 0x40):
   - hammer count k (1..3): play on from the current frame up to `T[k−1]`;
   - after a bid, with `rand(100) < 20` while the figure is at rest: frames 1 .. `T[3]` (a gesture), figure back at rest;
   - when a bidder of a different bank takes over while the figure is not at rest: frames `T[4]` .. `T[2]` of the *old* bank (return to rest), then the new bank is current.
   With a hammer count the frame loop ends with `sample_play` of the current speech and a 2-second `delay` at count 2 [L for the sample call].
7. Sold (line 15): texts 3037..3040 (`rules-12`); the "which residence to give up" question is a message box `(120, 120, 400, 240)` (0xBB14C). Unsold: 0xBADC4 at line 16; wait for a click after each.

### 1.5 End
Text area patch restored; if at least one lot was offered: `[EXE text]` (0xBB1D0), flip, speech 919 (`0x15570`, waits for the sample), then wait for a click; else `[EXE text]` (0xBB228), wait for a click. Music 0x4B stopped, window closed (the year-end code then opens its own 0x8910 window, `rules-12` §2).

## 2. Concert day (type-36 handler, 0x1AEA0, counter 0) [C]
Trigger: `rules-13` §2.4. Flow: `$V` into the turn-end report; report window closed, items cleared; picture **0xBB** (B/ICONBB: a lute on red cloth) at (0,0); flip.
1. Caption box `(0, 380, 640, 100, 0)` with 0xAA8DE (…`[EXE text]`) and (host). Click.
2. `sound(0x1D)`; caption box `(0, 420, 640, 60, 0)` with 0xAAD2F "$CDie Musik erklingt...$N". Click.
3. Outcome: caption box `(0, 160, 640, 240, 0)` with 0xAA968 / 0xAAA9C / 0xAAC0C and (host, payout). Click.
All three boxes have `flag = 0`, so each waits for a click (this settles the [L] of `rules-13` §5). Then picture **0xB7** (ICONB7: wooden desk, the turn-end backdrop), banner "Ereignisse" (0xBA91C) for the player, the report window reopened with its saved geometry, style 0xA810.

## 3. Office cycle: place pictures, dismissals, elections (year end, 0x66220) [C]

### 3.1 Place picture `0x66160(place, church, religion)`
| place | church = 0 | church ≠ 0 |
|---|---|---|
| 0..9 (city) | **0xC1** (hall stairs, same picture as the auction hall) | **0xC0** (ICONC0: stone portal with a closed double door) |
| 10..12 (county) | **0x193 + place** = 0x19D, 0x19E, 0x19F (ICONPD, PE, PF: three castles in a landscape) | religion 0 → **0xC2** (ICONC2: small white village church), religion 1 → **0xC3** (ICONC3: large white church with two towers) |
| 13 (principality) | **0xBF** (B/ICONBF: church interior, a niche with a cross) | **0xBE** (B/ICONBE: church interior, altar with candles) |

Religion 0/1 is the person's byte +0x03 (0 = Evangelisch, 1 = Katholisch in the order of the new-game menu, `formats.md` §8.2 [L for which is which]). Callers:
- dismissal of p: `(office location of p (+0x0F), the whole byte +0x10 of p, religion of p (+0x03))` — the controller nibble is included, so a human is always shown the church variant (rules-71 §2);
- election header: `(place L, office-is-church flag, rand(2))` — for a county church office the picture is **random** between 0xC2 and 0xC3;
- the two "no election" messages (`rules-12` §3.7 step 6): `(L, office-is-church flag, religion of the winner)`.

### 3.2 Screen
The year end starts with `win_open(0, 0, 640, 480, 0x8100)` and allocates a 640×60 frame (`strip`). Every message of §3.3/§3.4 is printed directly into this full-screen window on top of the place picture (no inner text window): `$8H$C$g…` puts the dismissal text on line 8 (y = 204), `$7H` on line 7 for the two- and three-dismisser texts, `$1H` the election header on line 1 (y = 36).

### 3.3 Dismissal
Place picture, text (`rules-12` §3.7), `sound(8)` if p is human, `wait_click`. Nothing else is drawn.

### 3.4 Election
1. Place picture; the screen strip (0, 420, 640, 60) is copied into `strip`; header 0xC17C8 on line 1.
2. The text cursor is set to y = 150; one line per candidate (0xC17F8 / 0xC1818): bullet item (style 0x90, glyph 0x73) at x = 120 (`$20>`), name at x = 150 (`$25>`), 24 px per line, so candidate i sits at y = 150 + 24·i.
3. Privileged candidate: 0xC183C, the other candidates' bullets are changed to glyph 0xCD (`0x34B80(item, 0xCD)`), click.
4. Each vote:
   - human elector: 0xC18A0 / 0xC18C4 printed at y = 421 (cursor forced to 0x1A5) after the strip has been restored, the candidate items re-armed; the game waits for a click on a candidate line (`menu`). The chosen line's bullet is then restored to 0x73 for the next elector [L].
   - computer elector: strip restored, `$C%1N3 stimmt für...$N$H` (0xC190C) at y = 421, flip, wait up to 180 ticks or a click (`menu` with timeout 0xB4) [L for the timeout semantics], strip restored, `$C%4N0!$N$H` (0xC1928) at y = 421.
   - vote marker: if the candidate has fewer than 6 votes so far, an item of style 0x130 with data **0x19C** (ICONPC: 14-frame bank, 16×12, a spinning coin) is created at (96 − 18·k, 153 + 24·c) with size 16×12, k = the candidate's votes before this one, c = candidate index; `$H` is printed (0xC1908) to flush it. Coins therefore stack right-to-left in front of the candidate's bullet, at most six. Then flip, wait up to 180 ticks or a click, strip restored, vote count incremented.
5. Result 0xC1934 / 0xC1954 at y = 421 (after a strip restore); `sound(0x1E)` if the winner is human; flip; `wait_click`.
6. Without an election: place picture, (`sound(0x1E)` for a human winner), 0xC1978 / 0xC1998 on line 8, flip, `wait_click`.
After the office cycle the 0x8100 window is closed (`win_close` at 0x68858) and the auction/Ereignisse screens follow.

## 4. Divine events (0x59940) [C]
Message boxes (§0.4) with the texts 2350 + k (`rules-12` §3.10); no picture, drawn over whatever is on screen (the year-end "Ereignisse" window, or the Kirchgang screen when triggered by the privilege). Geometry: `(160, 100, 320, 280)` for k ≠ 8, `(80, 80, 480, 320)` for the tornado. The box has the OK glyph item; a click closes it.

### 4.1 Tornado, text 2358 (0xA2475)
RNG order: `A = rand(10)`, `B = (A + 1 + rand(9)) mod 10`, `n = 50 + rand(50)`, then six cosmetic draws `h2 = rand(3) + 2`, `m2 = rand(4) + 2`, `c2 = rand(5) + 2`, `h1 = rand(3) + 4`, `m1 = rand(3) + 5`, `c1 = rand(3) + 6` (in this order; `rules-06` §3.1 lists only four of them). The text's codes consume, in text order:

| code | value |
|---|---|
| `%s` "… wird das Opfer" | name of A |
| `%w` Kirchen | c1 = rand(3) + 6 |
| `%w` Museen | m1 = rand(3) + 5 |
| `%w` Hospitäler | h1 = rand(3) + 4 |
| `%w` Insassen | n |
| `%s` "Reichtum von" | name of A |
| `%w` Kirchen (verschlagen) | c2 = rand(5) + 2 |
| `%w` Museen | m2 = rand(4) + 2 |
| `%w` Hospitäler | h2 = rand(3) + 2 |
| `%w` Einwohner | n / 2 + 1 |
| `%s` "gen …" | name of B |

After the box: `wealth_down(A, 5 + rand(10))`, `wealth_up(B, 5 + rand(10))`, population changes as in `rules-06`. This resolves the [L] in `rules-12` §4.

## 5. Films and the scenes around them

### 5.1 Sabotage (year end, 0x68ECB) [C]
After the sabotage message `$V`; `sound(0x0D)`; the Ereignisse text window is closed; `film(sabotage, 0, 0, 10)`; then banner for player 0xFF, flip, `win_open(100, 126, 440, 300, 0xA810)`, `$B`. Condition `rules-12` §3.6 (10 % per year end, once).

### 5.2 Siege and relief (0x1F2A0, 0x1FA80; rules `rules-07` §6) [C]
Superseded by `rules-21-siege-spy-messages.md` §2/§3: the belager film plays on the successful conquest roll (not at the arrival texts), and the entlager film plays only on relief success. The screen rebuild sequences below are correct.
- Human besieger's army arrives (text 0x9F51A `[EXE text]`): `$V`, `sound(8)`, `film(belager, 0, 0, 12)`, `delay(2000)`, banner "Ereignisse" for the player, `win_open(30, 110, 520, 290, 0xA810)`, `$B`.
- Computer besieger (text 0x9F08A `[EXE text]` in the year-end window): `$V`, `sound(8)`, `film(belager, 0, 0, 10)`, banner 0xFF, `win_open(100, 126, 440, 300, 0xA810)`, `$B`.
- Relief army (0x1FA80), human: after the relief texts (0xA3B25 ff.), `$V`, `sound(0x1E)`, `film(entlager, 0, 0, 12)` **twice in a row**, `delay(2000)`, rebuild as the human case above. Computer relief: `$V`, `sound(0x1E)`, `film(entlager, 0, 0, 10)` twice, rebuild as the year-end case.
Whether the film plays for both outcomes (success and failure) of a relief attempt is [L]: both call sites sit after the outcome texts.

### 5.3 City disaster (0x18BD0; rules `rules-06` §2) [C]
Trigger: year end, 12 % (`rules-06` §2). Scene for type d (0 fire, 1 flood, 2 plague, 3 avalanche):
1. `win_open(0, 0, 640, 480, 0x8100)`; picture `0x84B24[d]` = **0x86, 0xC9, 0xCA, 0x86** (B/ICON86: a stone wall with windows; B/ICONC9: dark interior with rising water [content L]; B/ICONCA: the reaper) at (0,0); `sound(0x84B30[d])` = 0x0D, 0x0C, 0x0B, 0x0D; for fire and avalanche `music(0x45)`; flip.
2. Caption box with speech (§0.2) `(10, y, 620, 160)` with y = `0x84B28[d]` = **216, 12, 12, 216**: text `$Z$C%o!$N` (0xBA898) with (city c), speech slot 900 + c (the city name spoken).
3. Second speech caption `(10, y, 620, 160)`: text `0xB5E10[2d + rand(2)]` (eight texts, `[EXE text]` … `[EXE text]`), speech slot 910 + 2d + rand(2).
4. Film only for fire and avalanche (the code tests y ≠ 12): `film(feuer / lawine, 0, 0, 12)` with sample 0x45 played in mode 3 during the film, stopped afterwards. No picture is redrawn after the film; the following boxes are drawn over the last film frame.
5. Caption box `(10, y, 620, 280, 0)`: `0xB5E30[d]` (e.g. `[EXE text]`) with (loss L, `0xB5E40` `[EXE text]` which takes (`0xB5E48[rand(4)]` manche/einige/viele/etliche, `0xB5E58[…]` " " or `[EXE text]`)). Click.
6. Per human branch hit: caption box `(10, y, 620, 280, 0)` `0xB5E60` `[EXE text]` with (owner, " " or " und einige Waren "); insurance refund: caption `(10, y, 620, 280, 0)` `0xB6FC4` with (owner, amount). Click each.
7. Window closed.

### 5.4 Spy evidence, "Informant" (turn end, 0x639A9) [C for the sequence, L for the condition]
Superseded by `rules-21-siege-spy-messages.md` §4: only one of informnt/informnb plays (CONFIG byte 0x273), and the 30 % is one `rand(100)` drawn per turn end before the record pass.
Condition `rules-04` §(spies) [L]: notification when `popcount(evidence) > 2`, and the film with 30 %. Text `[EXE text]` (0xC0CBC) in the turn-end report, `$V`, `sound(0x0D)`, report window closed, `film(informnt, 0, 24, 10)` immediately followed by `film(informnb, 96, 100, 10)`, then banner 0xFF [C: the code passes 0xFF here, i.e. the "Ereignisse in Mittelland" banner even though this is the human's turn-end report — quirk], `win_open(30, 110, 520, 290, 0xA810)`, `$B`.

## 6. Debt court, debtors' tower, trials

### 6.1 Jury court of the creditors (turn end, 0x64AF9; rules `rules-02` §10.3) [C]
1. Report window closed, items cleared; picture **0x193 + 0x868D4[office location of p]** = the county castle picture ICONPD/PE/PF of the county the player's office belongs to (table `0x868D4`: cities 0..3 → 10, 4..6 → 11, 7..9 → 12, 10/11/12 → themselves, 13.. → 10); flip.
2. `win_open(10, 280, 620, 200, 0x8910)`; `0xC121C` `[EXE text]` with (p, county, spinner style 0x14, &bribe): a bullet line with a 4-digit spinner. Clicking the spinner adds the step; clicking the bullet ends the input. Window closed; the bribe is paid.
3. `[EXE text]` (0xC12F4) printed into the full-screen turn-end window (y = 372) over the castle picture; flip. The strip (0, 420, 640, 32) is backed up. Speech files for "guilty"/"not guilty" are preloaded: slots 0x50 and 0x51 in four voice variants each (digits 1/2 × 0/1) [L].
4. Twelve juror markers at (62 + 44·i, 420), i = 0..11, redrawn after every vote: not yet voted → icon **0xB3** (ICONB3, 32×32, the framed Justice figure); guilty → icon **0x30** (ICON30, 32×32, the framed barred cell) (corrected by rules-71 §1: word table `0x87310` = [0x30, 0xA7, 0xB3]); not guilty → the backed-up strip is copied back for that 32×32 cell (nothing). Per vote: `rand(12)` start juror, next unvoted juror, `delay` [L: the two delay calls are 1000 and 200 ms], the vote speech sample plays; flip.
5. After the 12th vote `delay`, wait for a key/click, speech stopped.
6. Fewer than 7 guilty: picture **0xB8** (ICONB8: night landscape with a moon, also the year-end backdrop); speech 0x3B6; `sound(0x1E)`; screen transition `0x2C460(0)` [L: a per-scanline effect]; flip; caption box `(10, 220, 620, 100, 0)` `[EXE text]` (0xC1470) with (p). Click.
7. 7 or more: picture **0xB9** (B/ICONB9: a round tower at night, the Schuldturm); `sound(0x0B)`; ambient samples 0x54..0x57 and 0x85, 0x86 started; transition `0x2C460(0)`; text `[EXE text]` (0xC1320; with fewer than 2 branches 0xC13A8 and speech 0x3B5) on line 1 of the full-screen window; flip; wait loop: every 3 ticks `rand(100) < 2` → `sample_play(0x54 + rand(4))`, `rand(100) == 0` → `sample_play(0x85 + rand(2))` (the parity hazard of `rules-02` §10.3) until a key or click; samples stopped; type-20 record, flag 0x10.
8. Window closed; phase 6; the death check is skipped this turn.

### 6.2 Bankrupt (turn end, no branch left, 0x654C2) [C]
Report window closed; picture **0xBA** (B/ICONBA: a torch-lit dungeon corridor); samples 0x4E..0x51 started; `sound(0x0B)`; flip; then four speech captions, each preceded by `sample_play(0x4E + rand(4))`: `(40, 376, 560, 104)` 0xC1544 `[EXE text]` (p) with speech 0x3B7; `(40, 380, 560, 80)` 0xC15A4 `[EXE text]` speech 0x3B8; `(40, 380, 560, 100)` 0xC1620 "Eure Bemühungen…" speech 0x3B9; `(40, 380, 560, 100)` 0xC167C `$C%1N1, Ihr seid bankrott...$N` (p) speech 0x3BA. These use the plain caption box (`flag = 0`, click) with the speech started separately before each box. Samples stopped; purge; person cleared.

### 6.3 Trial scene (`trial_main` 0x289E0; rules `rules-04b` §3) [C for geometry and ids, L for the per-speaker frame logic]
Runs in the plaintiff's turn end (human) or at year end (computer plaintiff). Layout:
1. `$V`; the report window is closed and its geometry saved; picture **0xC1** (hall stairs) at (0,0); `sound(0x10)`; flip.
2. Caption boxes `(20, 20, 600, 120, 0)` unless noted: `0xA8332` `[EXE text]` with (defendant, plaintiff, location name, "angeklagt" / "dem Inquisitionsverhör unterzogen"); `(20, 20, 600, 160)` `0xA83A3` `[EXE text]` (judge, assessor 1, assessor 2); `0xA847B` `$CHeute Prozess gegen %4N1.$N` (defendant) [L: which of 0xB6E00's entries]; kind 1 adds `0xA83D4` (inquisition: no defence witnesses). Click each.
3. Court room: bank **0xC6** (B/ICONC6, one 640×480 frame: the court room) at (0,0); then five animated figures (`bank_frame(x, y, bank, 0)`): judge at (126, 162) bank **0xE9** (male) / **0xEA** (female), or **0xEB/0xEC** in the other court kind [L: which kind]; assessors at (60, 162) bank **0xED** (or 0xEF) and at (189, 162) bank **0xEE** (or 0xF0); defendant at (387, 223) bank **0xE5/0xE6** (male/female); plaintiff at (235, 306) bank **0xE7/0xE8**. These are B/ICONE5..F0, the "character animations, banks of 134" of `formats.md` §1.7. Flip. A 620×160 frame backs up the top area (10, 10, 620, 160) for the caption boxes.
4. Investments (`rules-04b` §3.3): computer side → caption `(20, 20, 600, 120)` `0xAFCFF` `[EXE text]`; human side → `win_open(100, 20, 440, 200, 0x8910)`, speech 0x392, `0xAFD31` `[EXE text]` with two 4-digit spinners (style 0x14, max 9999); click the bullet to confirm; window closed.
5. Reading of the charge: speech captions `(20, 20, 600, 120)`: `0x99C64` `[EXE text]` (slot 0x384), `0x99C82`/`0x99CB9` (plaintiff rises, slot 0x385 + …), the charge `0x99CF0`/`0x99D3C`/`0x99D84` with the law-group words (`0xB5C58..`), `0x99E4F` `[EXE text]`. Plea: computer `0x99E7A` `$C%1N1 meint: %s$N` (slot 0x38B − gender + …); human: `win_open(80, 20, 480, 200, 0x8910)`, `0x99E8D` `[EXE text]` with a string item (style 0xA0) cycling through the nine pleas of `0xB5C9C` (">Ich gestehe Alles!<" … ">Lüge, alles Lüge!<"); the bullet confirms.
6. With option "Zeugen & Plädoyers" off: caption `(20, 20, 600, 120, 0)` `0xBB28C` `[EXE text]` and nothing of step 7.
7. Witnesses (K per side, `rules-04b` §3.3): speech caption `0x968BA` `[EXE text]` (slot 0x393); per witness: `$|$H$g$13>$40<$C` (plaintiff side) or `$|$H$g$40>$13<$C` (defence side) sets a 2/3-width column, the witness line (table `0xB58E4`, 16 per side × gender) is printed and spoken (slot 0x27 + side·8 + line); a portrait bank **0x18E + rand(8)** (B/ICONOE..P5, 23 frames 64×64, a talking head) is drawn at (10 + 556·side, 10) frame by frame (23 frames, `tick(12)` each), then the top area is restored from its backup.
8. Pleas: `0x955FB` `[EXE text]` (slot 0x394) then the plaintiff's speech lines (`0xB5730`-table, four lines, `$H$N$g$C`), each with `sound(0x10)`, the five figures redrawn (speaking frames) [L], `tick(15)`, 2.5 s delay; `0xA15AC` `[EXE text]` (slot 0x395) and the defence lines (`0xB6694`, `0xB66AC`, `0xB66BC`) likewise.
9. Bribes (`rules-04b` §3.5): speech captions `0xACD18`.. (`[EXE text]` / `[EXE text]` / "…Eure Zuwendung erhalten" / …) slots 0x39A, 0x39B.
10. Verdict: `0xA5EE2` `[EXE text]` (slot 0x39C); per judge: human blackmailer → caption menu `0xBB334` `[EXE text]`; human judge → caption menu `0xA5F5A`; computer judge → caption `0xA5F1D` `$C%1N3 stimmt für...$N` with a 2000 ms timer (`flag = 2000`), then speech caption `(0x84CF8[j], 0x84D08[j], 190, 30)` = **(56, 151) judge, (0, 184) assessor 1, (108, 127) assessor 2** [the table gives x, y per seat; w = 190, h = 30] with `$0>$0<%s` and ">Schuldig!<" / ">Nicht schuldig!<" (slots 0x50/0x51 + voice). Then `0xA5F9E` `$H$g$CDamit ist %1N1 überführt.$N` if convicted.
11. Sentence (convicted): human judge chooses in `win_open(100, 20, 440, 200, 0x8910)` with `0xA84C7`-style string item over `0xB6DB0` ("Pranger", "Geldstrafe", "Mehrjährige Geldstrafe", "Entzug des Titels", …); the sentence texts `0xA802D..0xA81CC` are speech captions `(20, 20, 600, 120)` with slots 0x56..0x5C. Costs: `0xAFDCE` / `0xAFE07` (slots 0x39D.., 0x39F).
12. End: picture **0xB8** + banner 0xFF (year-end case) or picture **0xB7** + banner "Ereignisse" (0xBB3B0) for the plaintiff (turn-end case); flip; the report window is reopened with its saved geometry, style 0xA810.

## 7. Other scenes in the turn-end and year-end flow

### 7.1 Church wedding (courtship decision year, 0x212B0; rules `rules-05` §3) [C]
After the courtship texts in the report (`0x9CF95` rejection / `0x9D009` rival / none): `$V`; report window closed; picture **0xC0** (stone portal, the church door) at (0,0); flip; `sound(5)`; speech slot 0 (voice 0, 2, 2, 5) started. Captions are all `(20, 180, 600, 280, 0)` unless noted; each waits for a click:
1. `0x9D075` `[EXE text]` (target c).
2. `0x9D0F6` `$C$NHochzeit zwischen %3N1 und %3N1.$N` (suitor, c).
3. Picture **0xBE** (altar) replaces the door; flip.
4. `0x9D11E` `[EXE text]` (priest).
5. Domherr step (`rules-05` §3 step 3): blackmail question `0xBA9AC` `[EXE text]` as caption menu `(80, 180, 480, 280)` with (i, D); human Domherr `0x9D215` `[EXE text]` as caption menu `(80, 180, 480, 280)` with (D, D). Outcomes: `0x9D288` `$C$N%1N3 hat keine Einwände.$N` (D) or `0x9D2A7` `[EXE text]`; veto: `0x9D2C8` `[EXE text]` (D) then `0x9D32A` `[EXE text]`; record deleted; back to the report (step 10).
6. No veto: `0x9D1E8` `[EXE text]`.
7. `0x9D3A6` `[EXE text]`; vows as caption menu `(80, 180, 480, 280)` `0x9D427` `[EXE text]` with (suitor, c, "Manne"/"Weibe" by c's gender). Answer 2 → `0x9D591` `[EXE text]` (c) and the question again; answer 1 → `0x9D53E` `[EXE text]` (c, "seinen"/"ihren"), record deleted, step 10.
8. Answer 0: `0x9D4B4` `[EXE text]` (c); `film(hochzeit, 0, 0, 12)`; picture **0xFF** (B/ICONFF: a banquet table with guests) loaded; history entry; title text `0x9D5BE` `[EXE text]` (title name) if the title rises; picture 0xFF drawn, `sound(0x1D)`, flip.
9. Feast invitation as caption menu `(20, 120, 600, 360)` `0x9D605` `[EXE text]` with (c, F0, F1, name of W, F2, F3); options 0..3 → speech 0x386 + caption `(20, 340, 600, 80, 0)` `0x9D752` `[EXE text]`; option 4 → speech 0x387 + `(20, 340, 600, 80, 0)` `0x9D777` `[EXE text]`.
10. Picture **0xB7**, banner "Ereignisse" for the suitor, report window reopened (saved geometry, 0xA810).

### 7.2 Matchmaker (Kirche → Hochzeitsglocken, 0x20D30; rules `rules-05` §2) [C for geometry, L for flow]
Already courting: message box `(160, 100, 320, 280)` `0x9D96C` (abort / continue). Otherwise `win_open(0, 0, 640, 480, 0x8500)`, picture **0xBD** (B/ICONBD: a person in a ruff at a desk, the Kupplerin), `sound(0x11)`; nobody suitable → caption menu `(40, 400, 560, 80)` `0xBA95C` `[EXE text]` (one bullet) / caption `(0, 390, 640, 90, 0)` `0x9DB01` `[EXE text]`; otherwise caption `(0, 370, 640, 110, 0)` `0x9D9F9` `[EXE text]` with ("e "/"r "/" ", adjective `0xB6154[…]` unbeliebte … sehr angesehene, " ", c); price question as caption menu `(120, 390, 400, 90)` `0x9DB78` `[EXE text]`; decline → caption `(0, 420, 640, 60, 0)` `0x9DAB2` `[EXE text]`; accept → a portrait **0x186 + rand(5)** (male, B/ICONO6..OA) or **0x18B + rand(3)** (female, B/ICONOB..OD), 64×64, at (20, 390); caption `(100, 390, 540, 90, 0)` `0x9DAD4` `[EXE text]` (c); caption `(100, 360, 540, 120, 0)` `0x9DBD0` `[EXE text]` ("ihm"/"ihr", "seine"/"ihre", W). Window closed.

### 7.3 Mistress (0x20A00, 0x20B90) [C]
Message boxes `(160, 140, 320, 200)` with the texts of `rules-12` §3.8 (2404, 2407); the yearly texts go into the turn-end report. No pictures.

### 7.4 Death: candle, gravestone, funeral, testament (turn end, 0x657C6; rules `rules-05` §7) [C]
1. `sound(9)`; `film(kerze, 0, 0, 10)`.
2. Picture **0xC7** (B/ICONC7: graveyard with a tombstone and a bare tree); `$g$7H$35>$35<$Z` (line 7, margins 210 px each side); `$CHier ruht$Z$N%1N0$Z` (0x9EB83) with (p); epitaph line `0xB6330[2·bit + gender]`: bit = a set deed bit found by up to 16 `rand(8)` tries, else 8 → "Hoffentlich in Frieden"; the 18 lines: ohne seine Mätresse / ohne ihren Geliebten; Seine/Ihre Räuber taten alles für ihn/sie; Viele Saboteure sind nun arbeitslos (×2); Seine/Ihre Spione trauern; Der Schuldturm wird lange leerstehen (×2); Belagern war seine/ihre Passion; Das Lesen der Gesetze war seine/ihre Stärke nicht; Bestechung war sein/ihr Leben; Hoffentlich in Frieden (×2). Each ends in `$|`. `wait_click`.
3. Picture **0xC8** (ICONC8: a scholar at a desk full of books, the notary); `0x2EC40(0)` [L: palette/transition]; flip.
4. No children: cursor y = 1, speech 0x38C, `0x9B6BC` `[EXE text]` (p); flip; `wait_click`; purge; clear.
5. Otherwise two 640×60 frames back up the strips (0, 0, 640, 60) (notary strip) and (0, 420, 640, 60) (mourners' strip). Six steps i = 0..5, each printed at y = 10 (cursor forced) into the notary strip after it has been restored:
   - i = 0..4: speech slot 900 (i = 0), 0x385 (1), 0x386 + gender (2), 0x388 + gender (3), 0x38A (4); text `0xB5E7C[i]`: `$C>Liebe Hinterbliebene!<$N`, `[EXE text]`, `[EXE text]`, `[EXE text]`, `>Hier das Testament:<$N`, all with (p, net worth) on the stack; flip; wait until the sample ends (or a click when none plays).
   - i = 5: flip, **testament** `0x45A70`: `win_open(40, 100, 560, 280, 0x8810)` listing the heirs (`rules-05` §7.1, read-only here); `0x2EC40(0)`; if an heir was chosen (h < len): succession, speech 0x38B, `0x9B4AF` `[EXE text]` (new name); else speech 0x38D, `0x9B70C` `[EXE text]`, purge, clear.
   - after every step except i = 0 (and except i = 5 with a single child who is the heir): `delay`, then `1 + rand(len)` mourner lines (one when there are no children) printed at y = 421 into the mourners' strip with alignment `rand(3)` → `$L` / `$R` / `$C` (the centred variant adds 30 px to y): i < 5 → `0xB5E94[rand(14)]` (">Endlich!<", ">Hurra!<", `[EXE text]`, ">Darauf habe ich schon lange gewartet.<", ">Ha-haaa!<", ">Ich bin der wahre Erbe!<", ">Mir alles!!<", `[EXE text]`, ">Ooohh...<", ">Darauf einen Humpen!<", ">Geizkragen!<", ">Endlich bin ich reich!<", ">Lasst die Vorrede!<", ">Zur Sache, Notar!<") with speech slot = line index and a random voice `rand(4) + 15` (18 → 1); i = 5 → `0xB5ECC[rand(5)]` (">Verrat!<", ">MIR ward alles versprochen.<", ">Das ist fingiert!<", `[EXE text]`, ">Am Sterbebett wurde mir alles versprochen!<") with slot 14 + index. Each line: flip, `delay`, wait for the sample, `$H$C$|`, mourners' strip restored.
   - notary strip restored after each step.
6. Phase 6, window closed.

### 7.5 Feast (type-23 handler 0x20530; rules `rules-08` §2.2) [C for geometry]
Report window closed; picture **0xBB** (lute); `sound(1)`; captions `(20, 340, 600, 140, 0)`: `0x9ED91` `[EXE text]` (location, cost); `0x9EDD6` "Die Musikanten spielen zum Tanze..."; `0x9EDFE` `[EXE text]` (guests, kaum/mittelmäßig/gut/prächtig/königlich); then one of `0x9EE3F` (jeden Wunsch), `0x9EEAB` (no food), `0x9EF30` (no drink), `0x9EF98` (neither); `0x9F015` `$C... ein %s Fest!$N$N` (`0xB63AC[…]`, e.g. "furchtbares"). Then picture 0xB7, banner "Ereignisse", `win_open(30, 110, 520, 290, 0xA810)`.

### 7.6 Siege and relief dialogs
The siege texts (`0x9F08A` ff., `0xA3916` ff.) are printed into the report windows; only the films (§5.2) are scenes.

### 7.7 Duel (0x2C750; rules `rules-08` §3) [L]
`sound(0x17)`; speech captions `(40, 40, 560, 120)` with slots 0x399 + …, `(240, 380, 320, 100)` slot 0x3A4; picture **0xD1** (B/ICOND1: mountains in clouds, the duelling ground) with the fencing animation `0x2C140` (banks B/ICONDD..E4 of `formats.md` §1.7); result captions `(40, 40, 560, 120)` slots 0x3A5 + …. Not traced further.

### 7.8 Mission accomplished (history replay, 0x17930; rules `rules-10`) [C for pictures and texts, L for the trigger]
`sound(0x1D)`; picture **0xBB**; speech captions `(80, 200, 480, 160)` (voice 0, 2, 2, 9): `0xBA56C` `$C%1N1!$ZGroße Ereignisse werfen ihre Schatten voraus.$N` (slot 0x388), `0xBA5A8` `[EXE text]` (0x389); then one caption per history record (type 0x39 records of the player), each on its own picture: tower **0xB9** `[EXE text]` (0x38A); portal **0xC0** `[EXE text]` (0x38B); banquet **0xFF** `[EXE text]` (0x38C); lute **0xBB** `[EXE text]` (0x38D); place picture `0x66160(place, church, religion)` `[EXE text]` (0x38E); cradle **0xCE** (B/ICONCE) `[EXE text]` (0x38F + sex); reaper **0xCA** `[EXE text]` (0x391); flood **0xC9** `$C%l - Die Jahrhundertflut in %o...$N` (0x392); `[EXE text]` (0x393); forest **0xA3** `[EXE text]` (0x394). Then picture **0xBB**, `0xBA7E0` `[EXE text]` (0x384) and the mission's own text; `sound(0x1F)`; `film(outro, 0, 24, 10)` with speech 13 and `sound(0x1D)` during it, then `film(outrob, 96, 100, 10)` the same way; picture 0xBB; a caption menu `(80, …)` asks whether to continue [L]; `0x4D880` (game over / continue).

## 8. Picture identification (ICONS3 / ICONS3B rendered with `icon_dump`)

| id | file | content |
|---|---|---|
| 0x22 | ICON22 | parchment banner strip 640×72 |
| 0x30 | ICON30 | 32×32 framed barred cell with a figure — the "guilty" juror marker (corrected by rules-71 §1; 0x48 was a misread) |
| 0x86 | B/ICON86 | stone wall with windows (fire / avalanche backdrop) |
| 0xA3 | B/ICONA3 | forest at dusk (robber camp) |
| 0xB7 | ICONB7 | wooden desk with quill (turn-end report) |
| 0xB8 | ICONB8 | night landscape with moon (year-end report, acquittal) |
| 0xB9 | B/ICONB9 | round tower at night (Schuldturm) |
| 0xBA | B/ICONBA | torch-lit dungeon corridor (bankrupt) |
| 0xBB | B/ICONBB | lute on red cloth (concert, feast, mission end) |
| 0xBD | B/ICONBD | person in a ruff at a desk (Kupplerin) |
| 0xBE | B/ICONBE | church interior, altar with candles |
| 0xBF | B/ICONBF | church interior, niche with cross |
| 0xC0 | ICONC0 | stone portal, closed double door (church door; city church office) |
| 0xC1 | ICONC1 | brick stairs to an arched door (auction hall, trial, city secular office) |
| 0xC2 | ICONC2 | small white village church (county church office, religion 0) |
| 0xC3 | ICONC3 | large white twin-tower church (county church office, religion 1) |
| 0xC6 | B/ICONC6 | bank, one frame: court room |
| 0xC7 | B/ICONC7 | graveyard with tombstone |
| 0xC8 | ICONC8 | scholar at a book-laden desk (notary) |
| 0xC9 | B/ICONC9 | dark interior (flood) [content L] |
| 0xCA | B/ICONCA | the reaper (plague) |
| 0xCE | B/ICONCE | cradle with a baby |
| 0xD1 | B/ICOND1 | mountains in clouds (duel) |
| 0xD2 | B/ICOND2 | auction room with lectern and blackboard, no auctioneer |
| 0xD4 | ICOND4 | open book (catalogue) |
| 0xD5..0xD8 | B/ICOND5..D8 | auctioneer animation banks (frame 0 = auctioneer at lectern, full screen) |
| 0xE5..0xF0 | B/ICONE5..F0 | court figures (defendant m/f, plaintiff m/f, judge m/f, assessors) |
| 0xFF | B/ICONFF | banquet table with guests (wedding feast) |
| 0x15A..0x15F | B/ICONLA..LF | transporter pictures 144×144 (LA = covered wagon) |
| 0x160..0x169 | B/ICONM0..M9 | residences by level 160×120 (M0 = hut) |
| 0x16A..0x16C | B/ICONMA..MC | the three toll castles 160×120 |
| 0xB3 | ICONB3 | 32×32 framed Justice figure (juror not yet voted; corrected by rules-71 §1, was 0x179) |
| 0x186..0x18D | B/ICONO6..OD | 64×64 portraits (matchmaker candidates) |
| 0x18E..0x195 | B/ICONOE..P5 | 64×64 talking-head banks, 23 frames (trial witnesses) |
| 0x19C | ICONPC | 14-frame spinning coin 16×12 (election votes) |
| 0x19D..0x19F | ICONPD..PF | three castles in landscapes (county places, jury court) |

## 9. Open / unverified
- Window style bits (0x8100/0x8910/0x8800/0x8810/0x8500/0xA810): frame type per bit [UNVERIFIED]; the caption-box text offset y = 2 is read from the code, its visual effect is not.
- Disaster scene: whether the 0x2FDF0 call in the caption box darkens the window area [L].
- Auction: the sample call at the end of a hammer animation and the 2 s pause at hammer count 2 [L]; the attendance window height formula [L].
- Election: the `menu` timeout value 0xB4 (180) is ticks or hundredths [L]; re-arming of the bullet glyphs between electors [L].
- Jury court: contents of ICON48 / ICONN9 as juror markers are odd (a goods icon, a wheel glyph) and should be checked against a running game [L on meaning, C on ids]; `0x2C460` transition effect [L].
- Trial: the per-speaker frame selection of the five figure banks, the court-kind condition selecting 0xE9/0xED/0xEE vs 0xEB/0xEF/0xF0, and the exact text indices of steps 5, 8 and 11 [L].
- Duel and mission-end scenes are outlined only ([L]).
- Funeral: the `0x2EC40` call (palette or transition) [L]; speech voice digits of the mourner lines [L].
- Film x, y arguments for the two-part films (0, 24) and (96, 100) [L: assumed screen position of the frame loop].
