# 10 Trade, goods and production relations (Der Handel / Über die Relationen)

Handbook H p.41-46 (in-world essay by "Humbert vom Horkenhaine"; signed "Horkenheine" on H p.46). These chapters are written in-world; numbers in them are prose hints and are marked as such.

## Pricing rules (Der Handel)
Two price drivers: H p.41-42
- **Yield (Ergiebigkeit)**: determined by the city's geography. High yield -> low local price.
- **Popularity (Beliebtheit)**: depends directly on wealth. As wealth rises, simple goods lose popularity and luxury goods gain it.

Three trader rules: H p.42
1. A good's price in a city rises with its popularity. Richer city -> expensive/luxury goods more popular; poorer city -> simple/cheap goods more popular.
2. A good's price in a city falls with its yield there.
3. If rules 1-2 make the local price too low, sell in another city with high popularity and low yield; buy carts or ships.

Prose example: a barrel of fish fetches 5 gold in a coastal city and 10 in a mountain city. H p.41

Related rules elsewhere:
- Repeated selling into the same market saturates it (wool sold in Lauenburg five times in a row). H p.16
- After a fire, building material sells at very high prices. R p.8
- No trade is possible with a besieged city. R p.8
- Weapons are in demand in rich cities and in besieged cities. H p.46

## Goods: class, best locations, ratio hints
Class = as described in prose (cheap food / cheaper / luxury). "Best cities" = places the essay names as famous producers. Ratio hint = prose recommendation; the canonical ratio is Appendix II (08-appendices.md).

| Good | Cert. (App. II) | Class (prose) | Named producer cities | Prose staffing hint | App. II ratio | Source |
|---|---|---|---|---|---|---|
| Korn (grain) | I | cheap food | Moortal, Kronsbach ("granary") | 2 fields: at most 12 men; fewer than 8 -> low yield | 5:1 | H p.43 |
| Fisch (fish) | I | cheap food | Lauenburg, Lichterheide | more than 5 per boat is bad; 1 per boat is dangerous | 3:1 | H p.43 |
| Bier (beer) | I | cheap food | Falkenhorst, Greifenheim | 5 kettles, 10-20 workers | 2:1 | H p.43 |
| Wolle (wool) | I | cheaper goods; some years high prices | Fürstenbrück (quality), Bergstedt (robust) | 1 worker shears 2-4 sheep | 1:2 | H p.44 |
| Holz (wood) | I | common good | Finsterwalde, Greifenheim | at least 4, more than 6 harms the forest and lowers yield | 7:1 | H p.44 |
| Rindfleisch (beef) | II | high prices | Kronsbach ("Kronbach"), Lichterheide | not fewer than 1 worker per cattle | 1:1 | H p.44 |
| Erz (ore) | II | - | Falkenhorst, Bergstedt | 3-7 per mine; high prices especially by the sea | 5:1 | H p.44 |
| Wein (wine) | II | for wealthy buyers | Moortal, Lauenburg | 1 worker per wine barrel | 1:1 | H p.44 |
| Fell (fur) | II | luxury | Finsterwalde, Greifenheim | about 10 vats need at most 4-6 tanners | 1:2 | H p.45 |
| Glas (glass) | II | profitable, fashionable | Fürstenbrück, Greifenheim | 2-4 workers per furnace; good wages | 3:1 | H p.45 |
| Salz (salt) | III | - | Bergstedt, Waldstein | usually 6 miners per mine; sells well where no salt deposits | 6:1 | H p.45 |
| Tuch (cloth) | III | worn by high-ranking people (vanity) | Moortal, Finsterwalde | at most 2 per loom | 2:1 | H p.45 |
| Silber (silver) | III | special luxury, sells in rich cities even in war | Falkenhorst, Kronsbach | 5-10 per silver mine | 8:1 | H p.45 |
| Pfeffer (pepper) | III | among the most lucrative | Lauenburg, Lichterheide (pepper ships) | about 10 buyers per expedition | 9:1 | H p.46 |
| Waffen (weapons) | III | considerable profit where legal | none named | 4 per forge; forging is at times a privilege of few | 4:1 | H p.46 |

Notes on internal consistency (not resolved here, see open questions): several prose hints differ from Appendix II: fur "about 10 vats, 4-6 tanners" vs 1:2; wood "4-6 men per forest patch" vs 7:1; pepper "about 10 buyers" vs 9:1. Most other prose ranges contain the Appendix II ratio.

## Source map
| Claim | Source |
|---|---|
| Yield and popularity definitions | H p.41-42 |
| Three rules | H p.42 |
| Fish 5 vs 10 gold | H p.41 |
| Market saturation example | H p.16 |
| Fire -> building material | R p.8 |
| Siege blocks trade | R p.8 |
| Per-good prose rows | H p.43-46 |
| Certificate and ratio column | H p.36 |

## Open questions
1. Price formula: base price per good, yield factor per city/good, popularity as function of city wealth, saturation decay, siege and disaster modifiers.
2. City wealth: how it is modelled and how it changes.
3. Is the per-city good list (6 goods) fixed? Falkenhorst produces fish (H p.13) though it is not reachable by ship (H p.14);
 geography alone does not predict the list.
4. Which goods count as "building material" (wood? ore? glass?) for post-fire demand.
5. Weapons legality: which law governs it; is forging restricted by privilege?
6. Are the prose staffing ranges meaningful in the engine (minimum workers per means, maximum before decline), or is only the ratio used?
7. Is the 5 vs 10 gold fish example representative of actual price ranges (prices are shown as two-digit numbers, H p.11)?
