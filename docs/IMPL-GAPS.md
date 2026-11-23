# Open items and guesses

What is still open, what we do differently from the original, and every reading we chose where the specs are silent. Built features are described in git history and `docs/FIDELITY.md`; the sim's guesses also carry a `// SPEC-GAP:` comment at their code site (`grep -rn SPEC-GAP crates/mad-sim`). Cleaned up 2026-11-19.

## 1. Open questions

### For the RE role
Round 8 (rules-66, 2026-11-19) answered the earlier list; built from it: the year-end office cycle as one loop per person, the computers' skill gain and wealth drift through `pay`/`cash_add` with both clamps, the predecessor's bytes kept on turnover (+0x01, +0x05..+0x07, +0x0E, +0x14..+0x17, net-worth cache +0x1C, +0xF4), the computer shipment's random source branch, case 12 no longer an action, the escort evidence bit 0x20000, no report line for refusals and for place texts without a place. The parity diff of the 1619 save dropped from 188 to 27 differing fields. Still open:
- **Year-end parity**: redone 2026-11-22 on the passive long run's 1619 year start (`longrun/saves/1619-auto18.DAT`, the original's 1620 autosave `1620-auto19.DAT`; FIDELITY "Year-end parity, long-run save 1619"): 0 of 69 191 bytes differ after two fixes (vote favour draw order, indulgence pool halved per city). The playtest-save items still open from the earlier run: the `npc_act` draw gap for trade shipments (rules-09 §3; this year end made none) and the tower wait loop (not reproducible). The two live-check items of 2026-11-22 (jury markers, dismissal picture): answered by rules-71 §1 and §2.
- **BattlePending line**: ours only, for a game without `SZENARIO.DAT` (no original counterpart).
- **Battle hit sound**: answered by rules-71 §3.
- **Prisoners**: answered by rules-71 §4.
- **Tactics hotkeys of two humans**: answered by rules-71 §5.

### Needs a live check in the original
- **Pre-fight bribe spinner** and **donation window**: answered from the code by rules-71 §6 and §7 (no live check needed).
- **Late-game economy** (passive long run of the original to 1847, 2026-11-20, `economy_run`, FIDELITY.md "Passive long run"): the original drifts like ours. Median computer net worth, population, prices (including the drop to about 22 around 1840) and the passive human's net worth agree with ours every 25 years. Open: the mean city wealth index rises 0.103 per city and year in the original, 0.040 in ours (about 2 standard errors, one original game). Decomposed 2026-11-21 (FIDELITY "Passive long run" 4): same step sizes, no drift source missing; the original had fewer disasters (21 in 245 years, ours' rate predicts 27) and slightly more donations, each about 1.3 standard errors; rules-70 confirmed both gates as built. Treated as noise. Extended to 1917 (2026-11-22): prices match after 1850; the original's cities wrap at about 4.09 million (32-bit growth product, kept as a deliberate deviation below), so its population falls behind ours after 1870; open: the original's net worth runs 30-75 % above ours from 1870 (one original game).
- **New-game seed**: answered by rules-70 (2026-11-21). The original seeds once per program start from the DOS clock (seconds · 256 + hundredths) right before world generation; setup draws (start city from table 0x870C2, `person_init`) ignore every typed input. Ours already built the world and the start city that way; since 2026-11-21 the seed uses the same clock formula (6000 values) and a load from the menu starts at a clock seed instead of state 1 (a load inside a running game keeps the running state).
- Answered gift menu, report-window menus longer than the window, election gems, Bewerbung, siege questions, "Wohlan" and player creation: checked live 2026-11-22 (FIDELITY "Live checks of the open items"); all built as seen. Left from that run: the blank lines below (§2 Behaves differently).

## 2. Known differences from the original

### Not built
- **MIDI**: the OGG soundtrack plays instead (decision 2026-11-18); no tower ambience (its `rand()` calls are not drawn either, see the tower wait below).
- **Scenes on pictures** (closed 2026-11-22 from the specs, no capture): dismissals print their EXE text with its own codes (`$8H$C$g…$Z…$Z…`, `$7H` for two or three dismissers) into the full-screen window on the place picture (rules-15 §3.2/§3.3), the no-election messages 0xC1978/0xC1998 alone on line 8 (§3.4 step 6; ours had shown a candidate list box as well), the jury's "Die Geschworenen stimmen ab..." on line 15 (§6.1 step 3) and the Schuldturm text on line 1 (step 7). Verdicts (rules-18 §9), the acquittal (§6.1 step 6) and the bankruptcy (§6.2) are caption boxes in the original too (frameless text, rules-16b §3), as built.

### Behaves differently
- **Turn-end window** (rules-63 §2, rules-13; built 2026-11-19): the turn end reads as one window: each question opens with the window's text so far above it (scrolled until its last line ends at the window bottom, live A/B 2026-11-22), answered menus stay in it, and the next window starts with all of it, opening where the last one ended. Since 2026-11-22 the lines before a question wait for a click only when they and the menu overflow the window (a fresh window at its first overflow, a window continued under an answered menu once the text passes shown + h − 24, rules-42 §1 step 3; `real_short_window_before_question`); a scene in the middle of the turn end starts a fresh window in the original too (rules-19 §4: the text window is closed for the scene and rebuilt after it). Still different, not visible: our window is rebuilt at each question (the original keeps one); and when the text before a question overflows, ours shows the lines alone, waits, then the window scrolled to the menu's end, where the original shows the first menu lines in the full window before the wait.
- **RNG**: the original re-seeds from the clock at every battle start and keeps drawing while the debtors' tower picture waits for a click (1 417 draws in the parity run); ours does neither, since the question replay needs a deterministic generator. GAMEn.DAT has no field for the RNG state; a load from the menu starts at a clock seed, as the original does after a program start (rules-70 §1); a load inside a running game keeps the running state, as the original's does.
- **Battle presentation draws**: only the ESC-mode draws (rules-07 §5.11 recommendation), so a displayed battle diverges from the original's; the hit sound plays at the damage frame (original: when the animation restarts); one queued tactics hotkey (the original queues one per side); a prisoner keeps his participant and strikes on a falling captive are ignored (not stated in the spec); the explosion object-layer slot is taken only when free (cannons are objects here, men-layer records in the original).
- **Turn-end lines** (live A/B 2026-11-22; built 2026-11-22): a `describe` line ends as its EXE text does (`plain` keeps the `$N`/`$Z` codes at both ends of the text as markers, the report window prints them back), so consecutive events are separated by the texts' own `$N$N` (e.g. 0x9F9E0) and a text that ends with one `$N` (0x9F7CB) by none. A text with no break at its end (0xAC911 "...ab...") is followed by a blank line as well (live A/B 2026-11-22, two contract penalties in a row; ours matches). The banner during a turn end (built 2026-11-22): the title awarded at the start of the settlement (rules-19 §4 step 1, before the "Ereignisse" banner is drawn, formats §9.1 `%1N1`) shows from the first window on, also in the windows shown before a question while our state is still the old one (`early_title`; the `held` persons of the election and dismissal scenes are untouched).
- **Windows and dialogs**: since 2026-11-22 the donation and army dialogs print their closing line into the dialog and wait for a click (rules-38 §2.3 step 4: "Ihr stiftet ein..." was not shown at all before; §2.6/2.7: "Euer Heer wird..."), the three are scrolling windows (style 0xA810), a right click on the building menu is "Nichts", and "Mehr Privilegien" sits at window y 364 (§1.3); still different: the donation window drops the city question's text once the city is chosen (the original keeps the printed text, its items freed [UNVERIFIED whether the city name stays]), and a window whose closing line overflows scrolls to its end without the original's overflow click; the applicants window closes on a right click only, as in the original (live A/B 2026-11-22); the trial investment is one window with two amounts; since 2026-11-22 the human judge's vote (caption menu (20,20,600,120), rules-18 §9 step 5), the sentence choice ((100,20,440,200), bullet ends and cycling item, §10 step 1) and "Wollt Ihr weiterspielen?" ((80,180,480,120) on the lute, plain Ja/Nein items, rules-39 §8) are gold caption menus on their pictures; the indulgence box is drawn at the measured size (FIDELITY "Beten and Ablass"; rules-04b passes a fixed 240 height [UNVERIFIED box height rule]); the mission-success captions follow rules-15 §7.8 apart from the mission's own text after the congratulation and the samples 0x1D/0x1F; the pre-fight spinner sets the amount (rules-52 §1 [UNVERIFIED] "adds"), a right click answers the decline item, and a battle where only a helper owner is human shows its pages without the field.
- **Timing**: report windows scroll one step per 17 ms (the original once per repaint with no timer, rules-42 §1 and rules-56 §8: machine-dependent, so no target to build); funeral waits per rules-17 §4 since 2026-11-22 (notary line: speech or 2 s, then 4 s at step 0, else 2 s; each heckle 2 s then until its speech ends; the last words speech or 3.5 s, then 2 s, or 4 s and no heckles when the only child inherits; caption codes `$<n>T`, `$<k>I`, `$<m>U` in `hold_of`); still different: the testament window waits for a click (the original shows it under the last words with no click, rules-17 §4.2), and the "alles dem Erzbistum" case (0x9B70C, speech 909, 3.5 s) is a report line, not a notary caption; a skipped outro film cannot stop its speech (no per-sound handle in our mixer); the credits reveal uses a private generator instead of 958 game-RNG draws per line.
- **Deliberate deviations**: dismissal with a cached net worth of 0 gives r = 0 (the original faults on a divide); loading a save sets the castle table to its start values (the original keeps the running program's, rules-55 §14); denunciation box 0xBCEFC gets both names (the original reads an unset stack slot, rules-56 §5); world fate 4 turns shipments back only when the property index is a valid city (the original reads past the table); 64-bit population arithmetic (the original computes it in 32 bits and wraps, rules-66 §5; ours does not); the music 0x15 of the Privilegien menu is not played (the original plays it only on the 4-MB build).
- **Options**: music and digital volume scale all output linearly as master levels; no sound-card test (the original forces speech off without one).

## 3. Guesses in mad-app, all [UNVERIFIED]

| what | current choice | where |
|---|---|---|
| `$Z` | §8.3: half-height break. Taken literally, text overlaps, so a line with text moves down a full line and an empty line half; `$Y` adds a full line. Pixel A/B (law book, save/load, Löhne at 0 px) supports this | `text.rs` |
| text colour | dark half for body text; titles `^…_` gold; `$h` runs and hovered/selected choices highlighted (solid red on dark text, outline plus palette 0x01 on gold; `$h` red checked live on the year-end denunciation line) | `text.rs` |
| justified text (`$B`, default) | only wrapped lines are stretched, the last line is not | `text.rs` |
| word wrap | greedy at spaces, only for lines with no choices | `text.rs` |
| ignored codes | flags `$d $n`, waits `$W $V`, `$a` (fixed string not expanded); `$ns` is built (formats §8.6) | `text.rs` |
| frame tile transforms | 2 = vertical flip, 3 = horizontal, 4 = both | `screens/mod.rs` `draw_window` |
| hidden start-menu items | original prints `%M$E`; drawn as nothing | `screens/menu.rs` |
| outro | `OUTRO.FLC` on right click/Escape in the start menu, then quit (not in §7) | `screens/menu.rs` |
| film skip | Escape, Enter, any click (spec names only Escape) | `screens/film.rs` |
| intro play modes | the hundreds digit of the cue (modes 3 / 1 / 7) is ignored; all cues play once at full volume | `screens/film.rs` |
| HOCHZEIT film | header speed (`speed_ms`); every other film plays at its caller's delay (rules-56 §10) | `screens/game.rs` `film` |
| menu music | §7.1 says `SONG4.MID`; MIDI is unsupported, so OGG track `04 - …` plays (MIDI song n = OGG track n is a guess) | `audio.rs` `play_song` |
| pointer | icons `00-04`, hotspot top-left; only the arrow and the wait wheel (0x16D..0x17C) are used so far | `input.rs` |
| Kontor and room hotspots | §10.8 rects read as window-relative; labels drawn centred on their position; item text from the item origin without window inset; where the Bezahlung and Handel hotspots overlap (143..255 × 350..385) Bezahlung wins | `screens/game.rs`, `game/rooms.rs` |
| fonts | `$t` = dark font, `$g` = gold font (fits every §9 use) | `text.rs` |
| Neuer Spieler refusals | which of the EXE refusals (six players, after 1670, no free start city) wins when several apply | `screens/game.rs` |

## 4. Readings in mad-sim where the spec is silent or ambiguous

| where | gap | chosen |
|---|---|---|
| save.rs | `heap_free` field | 10000 - used |
| save.rs | castle table when loading GAMEn.DAT | **deviation** (rules-55 §14): the original keeps the running program's values (start values only after a fresh program start); we load the start values (toll 5 %, accumulator 0). Our snapshot saves the table, also a deviation |
| career.rs | dismissal when the cached NW is 0 | **deviation** (rules-55 §6): r = 0; the original faults on an unguarded divide |
| family.rs | funeral RNG calls | done per rules-17 §4.3; the app shows the drawn heckles |
| turn.rs | order of the turn-start steps (church prices, residence offers, lender mood, deliveries, directors) | in that order, after the storage offers |
| social.rs | which of 0x85D3D / 0x85D41 is candles / alms | candles first |
| turn.rs | order of the year-end steps | pieced together from rules-04b/06/07/08/09 |
| trials.rs | table 0x84B4C (one random offence bit) | bit `rand(30)` (rules-39 §10: entry i = 1 << i) |
| trials.rs | verdict of a computer blackmailer | guilty unless it is the defendant |
| events.rs | "branch with director" | the drawn branch, if it has a director |
| events.rs | "that branch" in fate 69 | the first run's branch |
| events.rs | residence level >= 10 | RB index clamped to 9 |
| events.rs | world fate 4, second half (destination := property index) | only the roll is kept; the doc asks for a runtime check first |
| castles.rs | F1 when nothing is in transit | kept |
| castles.rs | uninitialised add_troops training | 30 |
| social.rs | duel `office_index_of` | global office index |
| missions.rs | "condition byte" of mission 9 | residence byte +0x04 |
| fight.rs | when the human tactic menu opens | rules-44 §2 (after the activation in which the hotkey was pressed); `Ask::Tactics` answer packing is ours (nibble = tactic + 1, 0 = keep, `u32::MAX` = ESC) |
| battle.rs | tactic of a human group without a menu answer (`Auto`) | stays as deployed (0, cannons 1, leaders 8) |
| subplots.rs | dialog encoding | `Ask::Plot { kind, step }`: 0 = first line / accept |
| subplots.rs | book price | the shown Fy(500) is paid (no second roll) |
| subplots.rs | concert `office_index(x)` | global office index |
| subplots.rs | revolt remainder "random non-empty types" | across h's castles in order |
| subplots.rs | contest: unused entrant slots | 0xFF |
| subplots.rs | lecture dialog repeat | capped at 50 (`ponytail:`) |
| battle.rs | defender anchor "closest to first entrance" | squared distance, first minimum |
| battle.rs | line ends early / cannon cell blocked | new unit at a new anchor / line stops |
| battle.rs | more than 10 men on one line | a new unit at the next anchor (rules-07 §5.4): a safeguard only; the original has a 10-slot unit and exits fatally on an 11th man, but no shipped field has a line longer than 6 (rules-53 §5), so both behave the same. Closed. |
| battle.rs | goods marker right half (0xFD) | set when free |
| battle.rs | path mode 5 (Bombenleger) | as mode 0 |
| battle.rs | explosion length 9 (§5.13) vs 8 (§5.10) | 9 |
| battle.rs | influence map men | living fighters only |
| battle.rs | frame-loop tie order in a cell | men before objects |
| battle.rs | walk into a cell that became occupied | go idle |
| battle.rs | presentation RNG | ESC-mode draws only (rules-07 §5.11 recommendation); diverges from displayed original battles |
| fight.rs | computer helper-bribe index quirk | read as "skip human helpers" |
| director.rs | order slots the director writes | 0 and 1, transport first |
| director.rs | besieged source city | the director does nothing |
| director.rs | "free storage" in step 7 | m² minus stock/17 |
| director.rs / turn.rs | director position in turn start / settlement | after the offers / before the orders run |
| castles.rs | year-end settlement_pre position | right after the computer castles |
| debt.rs | seizing a branch when no other branch is left | shipments keep their source |
| ai.rs | revenge case 1 "random property at location" | uniform rand(count) |
