# 06 Combat (Hilfe ein Überfall - Der Kampf)

Handbook chapter 8, H p.30-33; readme 5.3, R p.7.

## 8.1 Lead-in (transport raid, from the victim's view)
1. Switch to tactical combat, a separate game within the game. H p.30
2. Intro sequence: the transport travels toward its destination. With an escort, considerably more defenders are shown than just the carters. H p.30
3. Robbers approach. The defender is first asked whether to pay the attacker a small sum to desist. Refusing, or paying too little, leads to combat. H p.30
4. Mercenaries patrolling the county (assigned by their castle lord) are usually nearby. Both defender and attacker may try to secure their support. They decide and, if they join, split onto the defender's or attacker's side. H p.30-31
5. Battle begins. H p.31

## 8.2 Units
- All troops are divided into units of 1 to 5 men (e.g. 2 carters, 4 riflemen). H p.31
- All men of a unit act simultaneously according to the unit's tactic, independent of their position. H p.31

## 8.3 General / robber captain
- Each side gets a special figure: general (mercenaries) or robber captain (robbers). Attacker's appears in the bottom-left corner; defender's in the top-right corner. H p.31
- While he lives, the player can press their hotkey to open the tactics menu and give each unit orders. If he dies, the computer takes over that side. H p.31
- His default tactic at combat start is "Abwarten" (wait). He is very strong in combat. H p.31

## 8.4 Changing tactics
1. Press hotkey: a flag in the player's colour appears in one of the top corners. H p.31
2. After the current action finishes, a banner is shown. H p.31
3. Hovering over one of the player's men draws a golden frame around the feet of all soldiers of that unit. Banner shows left: unit state, e.g. "3 Offiziere (67%)"; right: current tactic, e.g. "Gegner angreifen". H p.31
4. The percentage is the unit's condition: higher = deals more damage and can take more. H p.31
5. Left-click cycles to the next tactic available to that unit. Move the mouse to select another unit. H p.31

## 8.5 Tactics
| Tactic | Behaviour | Availability stated | Source |
|---|---|---|---|
| Gegner angreifen (attack enemy) | move toward nearest enemies and attack | - | H p.32 |
| Abwarten (wait) | stand still; defend but deal no damage. Useful to keep a reserve for recapturing goods | - | H p.32 |
| Gefangene machen (take prisoners) | encircle enemies to capture; captured enemy falls, flag of the player's colour appears on body; after combat prisoners are "persuaded" to switch sides | only when acting from a toll castle or robber camp | H p.32 |
| Waren erobern (capture goods) | move to nearest uncaptured good and plant banner; good belongs to player | attacker raiding a transport | H p.32 |
| Waren zurückerobern (recapture goods) | fight back captured goods; player's flag shows on them | defender | H p.32 |
| Gegner beschießen (shoot enemy) | cannoneers fire at spots with many enemies; can hit own men | cannoneers | H p.32 |
| Sturmreif schießen (bombard) | damage the castle/camp: security/camouflage installations damaged, casemates/straw camps destroyed | cannoneers attacking a castle or camp | H p.32 |
| Eingang stürmen (storm entrance) | move to entrance; attack only enemies blocking the target fields | attacking castle, camp or ship | H p.32 |
| Eingang verteidigen (defend entrance) | group in entrance area, attack all enemies approaching | defending ship, castle or camp | H p.32 |
| Rückzug (retreat) | maximise distance to enemies without leaving the field | - | H p.32 |
| Flucht (flee) | leave the battle area by the shortest path | - | H p.32-33 |

Conquest condition: a toll castle or robber camp counts as conquered when three specific fields in the entrance area are occupied by three of the attacker's men. Handbook advises establishing clear superiority first. H p.32

## 8.6 End of battle
- Outcome: won, lost, or undecided. H p.33
- Tabular summary of gains and losses: prisoners taken by whom, goods captured, crippled sailors, bomb layers lost to their own bombs. H p.33
- Robbers always demand a bonus (Prämie) for raids the player instigates; too many raids can ruin the camp owner. H p.33

## Auto-resolve (readme)
Pressing ESC during combat computes the outcome in the background; the result is shown after a few seconds. The player then has no influence on tactics. R p.7

## Ship raids
"Eingang stürmen" / "Eingang verteidigen" apply to ships, so ships can be boarded. H p.32. Sailors (Matrosen) appear in the summary. H p.33

## Source map
| Claim | Source |
|---|---|
| Tactical mode, intro, escort shows more men | H p.30 |
| Ransom offer before fight | H p.30 |
| Patrolling mercenaries choose a side | H p.30-31 |
| Units 1-5 men, simultaneous action | H p.31 |
| General positions, hotkey while alive, AI on death, default Abwarten | H p.31 |
| Tactic UI, "3 Offiziere (67%)" | H p.31 |
| Tactic list and effects | H p.32-33 |
| Conquest by 3 men on 3 entrance fields | H p.32 |

| End summary contents | H p.33 |
| Robber bonus | H p.33 |
| ESC auto-resolve | R p.7 |

## Open questions
1. Battlefield size, tile grid, entrance field coordinates per scenario (transport, ship, castle, camp).
2. Unit stats per troop type (damage, HP, speed, range); how training, pay/motivation and the general's strength factor in.
3. How men are grouped into units (by type? max 5 per unit).
4. How the condition percentage is computed.
5. Ransom: minimum accepted amount, who decides (NPC formula)?
6. Patrolling mercenaries' decision formula; bribing them; mercenary-robber alliance chance (H p.24).
7. Exact tactic availability per unit type and side (e.g. can non-cannoneers "Gegner beschießen"?).
8. Prisoner conversion rate.
9. Goods capture: what share of a transport each captured "good" marker represents; what attacker gets when undecided.
10. Bomb layer self-kill probability; cannon friendly-fire model.
11. Robber bonus formula.
12. Auto-resolve: is the result computed by the same simulation or a separate formula?
13. Can a human attacker also control tactics when both sides are human (two hotkeys)?
14. Escort composition and size per transport.
