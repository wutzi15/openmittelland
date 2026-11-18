# 03 Banner (Das Banner) and options

Handbook chapter 4, H p.6-9; readme 2.2, 5.5, 5.6, R p.4, p.7-8.

## Banner contents
Shown at the top of most screens. Displays: current year, cash (gold), the player's title, the player's office. H p.6

Clicking the gold display opens bribery; clicking anywhere else on the banner opens the options screen (market square). H p.6-7. Readme: "click the middle of the banner". R p.8

## 4.1 Bribery and office selection
Available whenever the banner is visible. H p.6

1. Click gold display -> map of the principality. The selected location is named above the map; a frame appears around the chosen target.
2. Selectable targets: any of the 10 cities, one of the 3 counties, the whole principality, a toll castle, or a robber camp. The principality is selected by clicking the Great Lake in the map centre; it shows the office tree of the highest princely offices. H p.6
3. Office tree for a city (secular level): mayor (Bürgermeister) at top; master builder (Baumeister), judge (Richter), treasurer (Kämmerer) below; three aldermen (Ratsherren) at the bottom. H p.6
4. Under each office a black favour bar shows the player's standing with the holder; wider is better. H p.6
5. At the bottom of the page the player can switch to the ecclesiastical offices; in a city these are two priests (Priester) and the canon (Domherr). H p.6-7
6. Faith marker right of the bar: narrow cross = Protestant holder, wider cross = Catholic holder. H p.7
7. Click an office -> window to enter an amount -> gold is paid. A sufficiently high amount widens the favour bar. H p.7
8. Same faith as the bribed holder gives a larger benefit than different faith. H p.7

Generic office-selection procedure used in many places (H p.7):
1. choose city, county, principality, robber camp or toll castle;
2. choose secular or ecclesiastical level;
3. choose office.

Bribery is an offence (law against making others compliant with money, H p.53; Heinrich example H p.48).

## 4.2 Options (market square)
| Item | Sign / control | Behaviour | Source |
|---|---|---|---|
| Save (Speichern) | sign "S" | up to five save slots, each with a comment | H p.7 |
| Load (Laden) | sign "L" | continue a saved game | H p.8 |
| Graphics (Grafik) | - | colour depth 256 or 32,768 colours (default 32,768); window size for animations; text scroll speed. Video mode "Lichtspielhaus" supported; "Blackbox" plays only the audio over a black screen | H p.8 |
| Music (Musik) | - | three volume sliders: digital sound, music, speech; speech on/off; "Soundcheck". Music slider has no function in this release | H p.8 |
| Options (Optionen) | toggles | history (Geschichtsschreibung), adventures (sub-plots), fate (random events), game tips, warnings, "Runde bestätigen", "Spieler ankündigen", "Zeugen & Plädoyers" | H p.8 |
| New player (Neuer Spieler) | - | add a player mid-game | H p.8 |
| Quit (Spiel beenden) | town gate | leave the game | H p.9 |

Option details (H p.8):
- Tips/warnings on: warns about trade mistakes and about approaching end of life with no children.
- "Runde bestätigen": after right-click in office, ask whether the round should really end.
- "Spieler ankündigen": before each round, show the name of the next player.
- "Zeugen & Plädoyers" off: shortens trials (skips witness statements and pleas, R p.7).
- All options are on after installation. H p.8, R p.7

Readme warnings: change only voice and digital-sound volumes and the soundcheck in the music menu (R p.4). Keep "Filmgröße" at "Lichtspielhaus"; other settings may show no film, black film, or crash, because only full-screen films were re-edited (R p.4). Save path `[Spielverzeichnis]\game\install\MITTELL\CONFIG\` (R p.4).

## New player (Neuer Spieler)
- Allowed while not all six start locations are taken and at least one city is not occupied by another player. H p.8
- The newcomer starts with the average wealth of the existing human players; office and title must be earned. Example: two players reach 1617, a third joins with the average of the two. H p.8

## Quit and continue
- Quit via the town gate on the market square. H p.9
- After restart, "Letztes Spiel fortsetzen" continues the last finished session. H p.9
- No autosave. H p.9

## Source map
| Claim | Source |
|---|---|
| Banner shows year, cash, title, office | H p.6 |
| Gold display opens bribery map | H p.6 |
| Targets: 10 cities, 3 counties, principality, toll castle, robber camp | H p.6 |
| Lake click selects principality | H p.6 |
| City secular tree layout | H p.6 |
| Favour bar | H p.6 |
| City ecclesiastical: 2 priests + canon | H p.7 |
| Cross width = faith | H p.7 |
| Same-faith bribe bonus | H p.7 |
| 3-step selection procedure | H p.7 |
| Other banner click opens market square | H p.7, R p.8 |
| Save 5 slots with comment | H p.7 |
| Load | H p.8 |
| Graphics options, default 32,768 | H p.8 |
| Music sliders, music slider non-functional | H p.8 |
| Option list and effects, all on by default | H p.8, R p.7 |
| New player rules and average wealth | H p.8 |
| Quit via town gate; continue last game; no autosave | H p.9 |
| Film size warning; save path | R p.4 |

## Open questions
1. Bribery effect: formula from amount to favour change; size of the same-faith multiplier; diminishing returns; is there a minimum amount?
2. Is bribery recorded as an offence immediately (for spies to find)?
3. Favour bar: numeric range and the "average" value (duels reset to average, H p.34
).
4. Exact layout of county and principality office trees in the bribery screen (which offices are shown).
5. What do toll castle / robber camp office trees contain?
6. "Average wealth" for a new player: cash only or total assets? Does the newcomer also get the uncle's branch?
7. Save format (`CONFIG/` files); what does "Letztes Spiel fortsetzen" load?
8. Does "Geschichtsschreibung" off change the simulation or only the report?
