# RE rules progress

Ghidra project: `<scratch>` (program `MITTELL`, names applied from `<scratch>`). Helper scripts in `<scratch>` (`dc.py` decompile with names, `gasm.py` listing dump, `fx.py` call/strings summary).

| doc | subsystem | status |
|---|---|---|
| rules-00-map.md | binary map, state layout, IDs, function map | done (record types partly) |
| rules-01-rng-calendar.md | RNG, seeding, calendar, turn and year loop | done |
| rules-02-economy.md | prices, production, storage, transport, costs, taxes, net worth | done; director AI open |
| rules-03-career.md | offices, appointments, dismissals, titles | done; privileges go to laws doc |
| rules-04-backroom-laws.md | bribery, spies, blackmail, denouncing (done); crime detection, laws, trials, privileges | partial |
| rules-05-family.md | marriage, children, heirs, death, inheritance | not started |
| rules-06-events.md | misfortunes, disasters, sub-plots, history | not started |
| rules-07-castles-bandits-combat.md | castles, camps, troops, raids, battles | not started |
| rules-08-auction-feast-duel.md | auctions, feasts, duels, church | not started |
| rules-09-ai.md | computer persons, directors, difficulty | not started |
| rules-10-missions-scoring.md | missions, scoring, winning | not started |

## Facts found but not yet written up (for the next session)
- Year end: `disaster_damage` (0x18BD0) runs with `rand(100) < 12` once per year, before computer persons' per-person processing; then for every computer person `settlement_pre` (0x61410, raids/battles of castle and camp orders) runs. [C]
- Director: at each human's turn start and turn end (0x59640) every branch with a director is run by the director AI 0x58410 with budget `((budget * (rand(151 - pay_directors) + 150)) / 150) * (600 - favour(director→owner)) / 500`. Note `rand(151 - pay)` divides by zero at pay level 151. [C]
- Raids on shipments (0x61410): for each type-54 record (camp "Plündert" order) of the player: candidate shipments are other owners' type-9 records whose destination lies in the target county, transporter type Karren or Schiff, and attacker or victim human; value sum `qty * V[g]`; then rand-based outcome selection (codes 0..3) and `battle_main`. Not yet fully decoded. [L]
- Retaliation/AI reaction entry point 0x5B140(actor, victim, code), codes 2 (lender), 3 (blackmail/denounce victim), 4 (denounce). [L]
- Missions: type-0 record (13 bytes) holds the mission; byte 4 = mission kind (6 = bribery/blackmail counters at +5/+9). Mission code entry 0x17690. [L]
- Phase-6 early exits: debt seizure 0x64108 and death 0x654A9.

## Next steps (priority)
1. Crime detection and trials (0x289E0, 0x2BD05, year end around the `trial_main` call), privilege table 0x5E600.
2. Family and death (0x654A9 inheritance, 0x221F0 births, 0x20D30/0x212B0 courtship, life counter +0x02 decrement site).
3. Events: 0x18BD0 disasters, 0x59940 computer misfortunes, 0x17930 history, 0x226F0 sub-plot dispatcher (option bit 4) with per-sub-plot records 32..49.
4. Castles/camps/combat: 0x54570..0x56E20, 0x61410, 0x3EE60 (battle), 0x37100 (tactics).
5. Auctions 0x26A50/0x27680, feasts 0x20160/0x20530, duel 0x2C750, church 0x5CCA0/0x5D1F0/0x59720.
6. Computer-person AI 0x59F80/0x5B200, director AI 0x58410, difficulty use of `0x85CBE`.
7. Missions and win check.

## Corrections from re-rules-c (not yet applied in the docs above, 2026-11-02)
- rules-04 §3.2: the office-tree price multiplier defaults to 10, not 7. Privilege codes 0x2F/0x49/0x4E give 7/6/5 (sabotage discount, applies to spies too).
- rules-02 option bits: fixed inline (sub-plots = 0x86AD1 bit 1, tips = bit 3).
- names.txt: 0x5F080 is the privileges menu, 0x59940 is the miracle event.

## Session 2026-11-03 (re-rules)
- Settled the 7 implementer discrepancies (option bits, NPC turnover vacates the office, inflation table, spy contract timeline, principality assessors, sabotage target is random, person +0x05/+0x06).
- New: economy §10 debt seizure and jury court; rules-07 §6 sieges/relief/tribute; rules-07 §5.9 per-man battle step logic (all tactics except cannon); rules-09 §4 revenge rows 1, 6, 7, 8 rewritten; rules-07 §4 raid code order corrected; rules-04 spy price multiplier corrected; rules-09 §5 director fee note.
- Found: city siege byte +0x6D is never written (siege price modifiers dead); castle table 0x8647C is not saved.
- Open: cannon step logic (0x39EA4..0x3A6AB), battle round driver (0x3D1DE: moves refill, order), director AI line-by-line re-check (rules-09 §5 is agent C's reading).
- Later on 2026-11-03: rules-07 §5.10 to §5.13 (ray table, animation lengths, round driver and frame loop, cannon, bombs and explosions, presentation RNG), §4 troop order lifetimes (types 51 to 54); rules-09 §5 step 8 director storage purchase and swap re-checked. Remaining [L]: walk cell-change frame.
- rules-11-kontor-actions.md written (branch rights, residences, local sale, money lender/loans, church entry/convert/leave, giving up, directors). rules-07 §5.6 troop premium verified and rewritten; entrance reachability computed.
- Correction: `0x85D3D` / `0x85D41` (standing(p,40) / standing(p,30), set at turn start) are church donation/candle prices (0x5D1F0), not loan offers.
- rules-07 §3a castle/camp order book, order status machine and yearly troop pay/training; economy §3.1 status recurrence note.

## Session 2026-11-08 (report texts)
- New: `rules-12-report-texts.md`: the `print` (0x322C0) argument model (all args on the stack, consumed sequentially; the digit in `%kX` is a case/form selector, not an argument index), every `%` code (`%N` forms 0..6 and office-less variants, `%F` pair forms, `%W` residence phrase, `%u/%e/%E/%p/%S/%s` noun codes, number formatters, items), the noun/case grid of the text table `0xB4258` (4 × 107 singular, rows 4..7 plural, female offices at 1407+), and per event the text address/id, argument list and window: branch count confiscation, office income, castle toll (player and Vogt), loan due, auction announcement/herald/catalogue/bidding/sold texts, sabotage (all five variants), dismissals and elections, mistress/gossip, all 100 fate kinds, divine and world fate events.
- Tooling rebuilt from scratch (the earlier `<scratch>` directories were gone): LE extractor, Ghidra 12.2 headless import of `image.bin` with a Java post-script that dumps all decompiled functions, a full listing with xrefs and string annotations; helper scripts `s.py` (strings/text ids/tables), `fstrings.py` (print calls with argument sources per function range), `site.py`/`ls.py` (listing windows). All under the session scratchpad, outside the repo.
- Open: election screen gate condition, `%*`/`%I` element details, alternate digit glyph set, tornado text argument order (listed in rules-12 §4).

## Session 2026-11-08 (sub-plot dialogs)
- New: `rules-13-subplot-dialogs.md`: every text, dialog and spinner of the 17 sub-plots (types 32..49, starters and handlers, dispatcher 0x63780) and of the newspaper contest (type 4): text addresses, argument lists in print order, answer indices per `%a` line, spinner styles/ranges (`item_range` overrides), the shared dialog mechanics (`menu_choice` 0x36850, spinner click rules of 0x35B60, defaults of 0x34130), the concert scene, name tables with sizes, and the mapping to the sim's `Ask::Plot { kind, step }` / `Event::SubPlot` values.
- Corrections: `0xB6AA8` holds 10 nicknames and `0xB6410` 18 epithets (rules-12 §3.9 k=82/87 has the sizes swapped). Sim discrepancies listed in rules-13 §4 (book quality bit order, topple RNG order, contest judging gate, spinner limits).
- Open: spinner mouse-button mapping, the argument-less `%s` of the expedition ambush text, contest superlatives past the 10-entry table, concert caption click waits (rules-13 §5).

## Session 2026-11-08 (scenes)
- New: `rules-15-scenes.md`: the drawing primitives (caption box 0x36C90, speech caption 0x36E50, caption menu 0x36AC0, message box 0x368E0, film sequence 0x14D10/0x14ED6, strip backup 0x13539) and every scene around events: auction day (hall 0xC1, room 0xD2, auctioneer banks 0xD5..0xD8 with the frame table 0x84C84, catalogue 0xD4/0x22, lot pictures LA..LF/M0..M9/MA..MC at x = 445/437, text area lines 15/16 with the 620×130 backup patch, speech slots), concert day boxes, place-picture function 0x66160 (per place/church/religion, random church for county elections), election layout (candidate lines at y = 150 + 24i, vote coins ICONPC at x = 96 − 18k, vote texts at y = 421 over a saved 640×60 strip), tornado argument order (resolves rules-12 §4), all film call sites with x/y/delay and the window rebuild after each, disaster scene (pictures 0x86/0xC9/0xCA, caption y 216/12), jury court (county castle picture, bribe spinner window, 12 juror markers at 62 + 44i / 420, tower 0xB9 and acquittal 0xB8 screens), trial scene layout (court room bank 0xC6, figure banks E5..F0 and their positions, witness portraits OE..P5, caption geometry), wedding (0xC0 → 0xBE → hochzeit.flc → 0xFF with all texts 0x9D075..0x9D777), matchmaker, mistress boxes, death (kerze.flc, gravestone 0xC7 with the 18 epitaph lines, notary 0xC8, testament window, six funeral steps with mourner lines), bankrupt (0xBA), feast (0xBB), mission-end replay and outro, duel outline. Picture identification table from rendered ICONS3/ICONS3B files.
- Corrections: rules-12 §3.5 "0x1BD/0x1B5 lot pictures" are x coordinates (445/437); the lot pictures are the transporter/residence/castle icons. rules-06 §3.1 tornado has six cosmetic draws, not four. rules-13 §5: the concert caption boxes all wait for a click (flag 0).
- Open: window style bits, disaster first-caption text, trial per-speaker frames and text indices, duel and mission-end details, juror marker meaning (rules-15 §9).

## Session 2026-11-08 (sound)
- New: `rules-14-sound.md`: sample API (load 0x124E1 / play 0x12698 mode 3 / stop mode 4 / unload 0x12730, `SMPL_xy0` names, 32 voices, no volume/pan), speech API (0x15100 name scheme: four alphabet characters + three digits, slot stack, `speech_say` 0x15570, `say_box` 0x36E50), music front end 0x41E50 (modes 0..5, random song groups 0x19..0x1E, song letter mapping, low-memory gate `0x86BC8`), every sample number with trigger and `rand` rule (battle per-state sounds, duel tables 0x84D30, auction hammer, church, turn-end screens), every speech call site, every music call site, and a duration-based MIDI-to-OGG soundtrack mapping.
- Corrections: `rules-12` §3.5 "music 0x4B" is sample 0x4B (hammer); "sound cue n" in rules-12/13 means `music(n, mode)`. Variant digits of `SMPL_xyz` are MIDI instrument samples requested by a private chunk in each `SONGx.MID`, not sound-effect variants (`formats.md` §5.1 should say so).
- Tooling: call-site extractors `calls.py`, `cargs.py`, `pushes.py`, `hints.py` next to the existing scripts in the session scratchpad (`…/4f1e3941-…/scratchpad/tools`).
- Open: interrupt mixer not in the image (song looping, `0x86C58` reset at song end); variable trial speech lines; ~ rows of the soundtrack mapping need listening.

## Session 2026-11-09 (speech map, window styles)
- New: `rules-16-speech-map.md`: every SPC WAV mapped to its call site (51 `speech_load`, 7 `speech_say`, 52 `say_box` sites), with voice/gender digit meaning, `n` formula, caption text address and text; 1328 of 1416 files reachable, 0 referenced files missing, 88 present-but-unreferenced listed (§6). Resolves rules-14 §5 speech items. Corrections: `F023`/`1023` are mourner heckles, not tips; `0229948` is the game-over line; duel answers roll `rand(7)` so file 008 is unused.
- New: `rules-16b-window-styles.md`: window record layout and every style bit (0x8000 own frame, 0x4000 save/restore, 0x2000 scroll ring buffer + rolls, 0x800 erase on close, 0x400 borrow parent frame, 0x100 no tiles; 0x10 unused), compositor 0x2FDF0 semantics, frame tile grid/indices/transforms (blitter transforms 1..4 confirmed), roll strips, item kinds, pointer 0x2EC40, transition 0x2C460 parameters, tick wait. Key correction: caption boxes (0x36C90/0x36AC0/0x36E50) and the 0x8910+compositor windows are frameless text over the picture (gold with dark outline via `$g`); rules-15 §0.1 and formats §7.2 need the amendment.
- Tooling: `rng.py` (listing range dump), `speechcov.py` (speech coverage generator) in the session scratchpad tools dir.
- Open: item flag bits 0x200/0x400, font-table/colour half mapping, pointer shapes ≥ 6, content of the 88 unreferenced WAVs.

## Session 2026-11-09 (testament, trial scene, disaster/birth scenes, music)
- New: `rules-17-testament.md` (death tail 0x654A9..0x6613E, heir list 0x44130, testament window 0x45A70: geometry, item styles, heir cycling, succession writes in order, fifth-part payment, waits are speech-or-delay not clicks, mourner RNG order alignment/line/voice/gender), `rules-18-trial-scene.md` (every step of 0x289E0 with boxes, speech files, figure banks by court kind and gender, the 134-frame segments 0..48/52..99/100..133 shared by all five figures at tick 15/10, witness column margins 78..400 / 240..562 and head positions 10/566, investment and plea windows, bribes, per-seat verdict boxes, sentence window, costs, jury-court bribe screen and markers 0x48 bottle / 0x179 wheel verified on the rendered icons), `rules-19-disaster-birth-scenes.md` (0x18BD0 step order with RNG, 0x221F0 birth and child death, title award/certificate screens, proof that fate events draw no picture, full scene order of a human turn end and of the year end), `rules-20-midi.md` (SMF facts for all 24 songs, private chunk layout: bitmap at payload byte 31 indexed by GM program / drum note, four size dwords; hardware MIDI vs sample-synth path, no loop, no FM; trigger table per scene).
- Corrections: `rules-15` §7.4 (notary waits, mourner rules), `rules-16` §4 (mourner RNG order), `rules-14` §1.2 (bitmap indexed by program, not by note), `rules-06` §2 (marketplace clause unconditional for kinds 0, 1, 3).
- Tooling: `midi.py` (SMF summary + private chunk) in the session scratchpad tools dir; PPM→PNG one-off used to check ICON48/ICONN9.
- Open: item flag 0x400 / menu step `d` for cycling items, kind-0x80 text item width, `0000014..018` content, sample-synth mixer internals, song-finished state, charge-group connector index.
