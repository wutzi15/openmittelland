# 07 Special events (Besondere Ereignisse): auction, trial, duel

Handbook chapter 9, H p.33-34; readme 5.4, 5.5, R p.7.

## 9.1 Auction (Versteigerung)
- Held at irregular intervals in some city. Announced some years in advance. H p.33
- Players may release objects for auction in the respective menus: residence (H p.15, p.33), transports (H p.16; balloons H p.33). An auction usually yields more than a plain sale for a nice object. H p.33
- Start: auction catalogue with an exposé of every lot (including own): condition, for transports also count, minimum bid, previous owner. H p.33
- After the catalogue a player may choose not to attend; the auction then runs without them. Not possible if the player has their own lot in the auction. H p.33
- Per lot: auctioneer presents it and opens at the minimum bid. Pressing the player's hotkey places a bid. The auctioneer comments on the bidders until "...und zum Dritten..."; the lot is sold and the next is announced. H p.33
- Lots can go very cheap or far above value. H p.33-34
- Readme: if the player did not opt out at the start, ESC skips the current lot and jumps to the next. R p.7

## 9.2 Trial (Vor Gericht)
Sequence: H p.34
1. Prosecution reads the list of charges.
2. The accused may respond.
3. Both sides may hire witnesses (costs gold, raises own credibility).
4. Witnesses testify.
5. Closing speeches (Plädoyers).
6. Judges may be bribed.
7. Verdict.

Outcome factors: whether the accusations are true, the witnesses, and the standing with the judges. H p.34
Option "Zeugen & Plädoyers" off shortens trials. H p.8, R p.7
Evidence must come from spies to be used successfully. H p.18
Indulgence atones offences so the player can hardly be convicted. H p.20

Possible penalties: H p.34
| Penalty | Effect stated |
|---|---|
| Hours in the pillory (Pranger) | harms health and so life expectancy |
| Fine | not very painful |
| Multi-year fine | can hurt a lot |
| One year debt tower (Schuldturm) | skip one round |
| Loss of a title | prestige loss |
| Removal from office | may endanger the mission |

Judges: normally city judges; if the accused holds a county office, the justice adviser (Justizberater) judges; a principality office, the justice minister. Assessors are usually judges from neighbouring cities, or in serious cases the county justice advisers. H p.54
Narrative (H p.51): an acquitted defendant's accuser paid the trial costs (1,700 gold in the example).

## 9.3 Duel (Duelle)
- Main reason to insult someone: the player's standing with that person is extremely low (e.g. after being denounced) but the player depends on their favour. H p.34
- After the duel, the player's standing with the opponent returns to a fully average level. H p.34
- The player's health suffers depending on fencing skill; in the extreme the player can no longer hold office. H p.34
- Insults received from others: accept (standing damaged) or demand satisfaction (duel at dawn at the old bell tower). H p.30
- Narrative: duel ends with standing "normal" again and health reduced. H p.61

## Source map
| Claim | Source |
|---|---|
| Auction irregular, announced years ahead | H p.33 |
| Release residence/balloons | H p.33 |
| Catalogue contents | H p.33 |
| Opt out unless own lot | H p.33 |
| Hotkey bid, "zum Dritten" | H p.33 |
| ESC skips lot | R p.7 |
| Trial sequence | H p.34 |
| Outcome factors | H p.34 |
| Penalty list | H p.34 |
| Judges/assessors | H p.54 |
| Trial cost to loser (narrative) | H p.51 |
| Duel reason and effects | H p.34 |

| Insult choice | H p.30 |

## Open questions
1. Auction scheduling: frequency, lead time, city selection, NPC lots.
2. Minimum bid formula; bid increment per hotkey press; NPC bidding model; how long until "zum Dritten".
3. What objects can be auctioned (residence, transports; storage? castles?).
4. Payment/transfer rules when the winner cannot pay.
5. Trial: verdict formula (truth, witness spend, judges' favour, bribes); witness cost; who can sue (privilege holders only?); penalty selection per offence; trial costs.
6. Pillory health loss; fine amounts; "multi-year fine" duration.
7. Debt tower: what happens to the player's branches during the skipped round (narrative H p.55: managers in his other branches kept trading)?
8. Duel resolution: is it interactive (fencing mini-game) or computed? Health loss formula; can the player die?
9. Can the player insult someone proactively, and from which screen?
