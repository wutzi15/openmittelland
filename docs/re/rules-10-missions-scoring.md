# Missions, winning and game end

Confidence tags and offsets as in `rules-00-map.md`.

## 1. Difficulty choice (new game, 0x4AB90) [C]

Menu `[EXE text]` lines in order: Leicht, Mittel, Schwer, Keine Aufträge. The chosen index is stored in `0x85CBE`: **0 Leicht, 1 Mittel, 2 Schwer, 3 none**.
**Correction to `rules-01` §3:** it lists 0..3 as keine, leicht, mittel, schwer; the menu text order and the mission code (§2) show 3 = none.

## 2. Secret code and mission record (0x17690, called by setup_human_player) [C]

Only if `0x85CBE < 3`. The player types a 4-digit code (digits only; any non-digit makes the entry invalid). The code is looked up in a table of 90 u16 values at `0x84A34` (3 valid codes per mission, missions 0..29 in order). Index i found → mission `m = i / 3`. Results:
- not found (i = 90) or more than 4 characters: "Falscher Geheimcode", ask again;
- found but `m / 10 ≠ difficulty`: `[EXE text]` (colour table `0xB737C`: grün, blau, third [G] rot), ask again;
- found with matching difficulty: accepted.
The loop only ends with a valid code of the chosen difficulty; there is no skip. Then a type-0 record (13 bytes) is created for the human: counter 100, +4 m, +5 u32 = 0 (progress counter), +9 u32 = 0 (secondary counter).

The code table is game content printed on the mission cards; a reimplementation should read it from the user's EXE (like the strings, `strings.md`), not embed it.

Mission m belongs to difficulty `m / 10`: 0..9 Leicht, 10..19 Mittel, 20..29 Schwer.

## 3. Mission check (0x17930, at the end of each own turn) [C]

Called from the turn-end tail before the life decrement (`rules-05` §7), only while the human has a type-0 record. It first sets the record counter to 100 (it never expires). "Streak" means: if the condition holds, `+5 += 1` and the mission is met when `+5` exceeds the given value; if it fails, `+5 := 0`. "Counter" means `+5` is accumulated elsewhere (list in §4) and only compared here.

| m | condition (met when) | kind |
|---|---|---|
| 0 | net worth > 49 999 | state |
| 1 | own Schiff + Tauchboot over all branches ≥ 12, streak > 4 (5 turns) | streak |
| 2 | certificate level (+0x05) = 2 | state |
| 3 | title > 3 (Landgraf or higher) | state |
| 4 | holds city office index 6 (Bürgermeister), streak > 4 | streak |
| 5 | city office index 9 (Domherr), streak > 6 | streak |
| 6 | counter > 2: successful blackmails | counter |
| 7 | counter > 2: won lawsuits (see §4) | counter |
| 8 | counter > 9 999: Taler spent on indulgences | counter |
| 9 | owns a branch whose residence word has low byte 0xE6 (level 6 with all three extensions) and condition byte ≥ 84, streak > 2 | streak |
| 10 | net worth > 200 000 | state |
| 11 | at least 3 goods with own skill ≥ 140 (Wein also counts when the human holds ability code 0x2C) | state |
| 12 | counter > 499 (guests at feasts of the top level 5) | counter |
| 13 | title > 6 (Graf or higher) | state |
| 14 | county office index 6 (Kanzler), streak > 4 | streak |
| 15 | county office index 12 (Bischof), streak > 4 | streak |
| 16 | popcount(counter) > 15: at least 16 different offence bits committed (crime scan bitmask OR-ed in) | counter |
| 17 | in some county (10, 11, 12, checked in order) more than 10 persons located there share the human's religion (+0x03) | state |
| 18 | counter > 159 999: Taler donated for buildings (privilege "Bauwerk stiften") | counter |
| 19 | counter > 39 999: battle booty | counter |
| 20 | net worth > 750 000 | state |
| 21 | all three law books have severity index (0x48DE0) ≥ 2, streak > 3 | streak |
| 22 | counter > 6: successful sieges | counter |
| 23 | owns ≥ 3 toll castles, streak > 2 | streak |
| 24 | principality office index 6 (Großkanzler), streak > 6 | streak |
| 25 | principality office index 10 (Erzbischof), streak > 6 | streak |
| 26 | popcount(counter) > 23 (at least 24 offence bits) | counter |
| 27 | more than 39 persons located at 10..13 share the human's religion | state |
| 28 | counter ≥ 600 000: building donations | counter |
| 29 | counter > 149 999: battle booty | counter |

Office tests use the human's location byte (+0x0F: city < 10, county 10..12, principality 13) and index (+0x11); "city office index 6" requires +0x0F < 10, county tests require 10 ≤ +0x0F ≤ 12, principality tests +0x0F ≥ 13.

## 4. Counter updates (only for the matching mission kind) [C unless noted]

| m | where | increment |
|---|---|---|
| 6 | office tree, blackmail succeeded (0x45C30) | +5 += 1; and +9 += bribe amount on a bribe (not used by any check) |
| 7 | trial (0x289E0) and verdict (0x2BD05), branch with outcome < 2 [G: the human won as accuser] | +5 += 1 |
| 8 | indulgence purchase (0x59720) | +5 += price paid |
| 12 | feast (0x20530), feast level 5 only | +5 += number of guests |
| 16, 26 | crime scan (0x44E00) | +5 \|= offence bitmask of this scan |
| 18, 28 | privilege "Bauwerk stiften" (0x5F080 / 0x613AD) | +5 += donation |
| 19, 29 | battle (0x3EE60), per participant | +5 += that person's booty value |
| 22 | siege resolution (0x1F2A0), city taken | +5 += 1 |

## 5. Mission success [C]

When the condition is met:
1. `[EXE text]`: for each history code 0..9, the first type-57 record of the human with that code is shown once (codes in `rules-06` §7; code 1 only if still married).
2. Congratulation text, the mission's own success text, outro film.
3. The mission record is deleted (no further missions; the check stops).
4. "Wollt Ihr weiterspielen?" "Nein" calls 0x4D880 (quit: save config and game state, exit to DOS). Otherwise play continues without a mission; other humans keep their missions.

No score, ranking or winner screen exists. Missions are per human and independent; several humans can complete theirs.

## 6. Other ways a game ends [C]

- A human leaves the game: bankruptcy (no branch left), death without children, or death with the estate left to the Erzbistum (`rules-05` §7). When no human is left: `[EXE text]`, back to the title menu.
- Year 2000: the main loop ends and the program exits to DOS without message (`rules-01`).
- Option "Spielende um 1800": no hard end; it only allows births until 1769 instead of 1669 (`rules-05` §8). **Handbook/FEATURES disagreement:** the readme-cited "game end 1800" does not exist as a rule.
- New players can join until 1670 (`[EXE text]`), at most 6 humans.

## Evidence
0x4AB90 (difficulty menu), 0x17690 (code entry), 0x84A34 (code table), 0x17930 (check, jump table 0x17944), 0x2D750 (history records), 0x4D880 (quit), the counter sites listed in §4.

## Open
- Meaning of the "outcome < 2" branch for mission 7. [G]
- Mission success texts and the card wording are external (cards); the check predicates above are complete for all 30 kinds.
