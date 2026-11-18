# 04 Office (Im Kontor) and all its menus

Handbook chapter 5, H p.9-28; readme 5.7, 5.8, R p.8-9.

## Turn structure (as seen from the office)
- Round- and year-based. Human players play their round one after another. H p.9
- Per player turn: (except in the first year) a report on last year's trade results (fish caught, harvest) -> office -> the player makes decisions -> on round end the consequences are shown. H p.9
- After all human players have finished, the "well over a hundred" computer-controlled characters act and players are informed about events in Mittelland. H p.9
- Detailed end-of-round sequence: see 05-events.md.

## Office hotspots
| Hotspot | Opens | Source |
|---|---|---|
| Globe (Globus) | trade map | H p.9 |
| Desk (Schreibtisch) | counting room (Schreibstube) | H p.4, p.17 |
| Chest (Truhe) | payment (Bezahlung) | H p.4, p.19 |
| Board above the chest (Statistik-Tafel) | score window | H p.4, p.28 |
| Cross, top right | church | H p.5, p.20 |
| Crossed swords | castle/camp overview map | H p.21 |
| Door, left | back room (Hinterzimmer) | H p.26 |
| Right-click anywhere | end round | H p.5, p.28 |

---

## 5.1 Trade

### 5.1.1 Trade map (Handelskarte)
- Map of Mittelland. Small flags in each owner's colour wave near cities whose branch rights are taken (own and other human players'). H p.9-10
- Each city has two click areas: H p.10
  - **City name** -> city information window.
  - **City graphic** -> own branch if the player has one there; otherwise a window asking whether to buy the branch rights (if not yet granted to anyone).
- City information shows: whether a disaster struck the city; which goods are produced especially well there; which goods are in high demand (high price); whether the city is under siege; the price of the branch rights. H p.10
- Branch rights in each city are granted to only one human player. H p.10
- At game start the price of a second branch is beyond the player's means. H p.10

### 5.1.2 Branch screen (Niederlassung)
Layout: top third = stock of goods; centre = order book; bottom = control icons. H p.11

#### 5.1.2.1 Stock (Warenbestände)
- Each city can produce six different goods; which ones depends on the city's geography. H p.11
- Above each good: a two-digit number = gold obtained by selling one unit there. H p.11
- Around the price: a ring (golden per handbook; readme: blue ring with golden end caps) showing the player's personal production skill for that good; the more closed the ring (closing upwards), the more experienced. H p.11, R p.8
- Below each good: number of units in stock. Start: none. H p.11
- At game start four of the six goods are darkened = not producible. Certificates granted during play unlock trade in more valuable goods. H p.11
- A production order given in year 1 delivers goods into stock the following year. H p.11
- Selling locally (three ways): H p.11
  1. Click the stock count below a good with "-" cursor: sell one unit per click.
  2. Click the good's graphic: sell the whole lot of that good.
  3. Click the empty-warehouse icon left of the goods icons: sell all goods in stock.

#### 5.1.2.2 Order book (Auftragsbuch)
- Two orders per branch per year. H p.12
- Four order types: production, sell, move ("Schafft"), fetch loot ("Holt Beute"). H p.12-14
- The book shows a forecast of expected costs and of warehouse utilisation in percent. H p.12
- Costs are deducted only at the end of the player's round. H p.12
- Status symbol at the end of each order line: H p.12
  1. Crossed circle = possible and confirmed; will be executed.
  2. Plain circle = possible, not confirmed; will not be executed.
  3. Broken circle = impossible; cannot be clicked/activated. Example: transport needs 4 carts but branch has 3. Lowering transporter count or goods quantity can make it possible.
- The chosen status persists across years as long as the order remains feasible, so an order can be "paused" for a year. H p.12
- Order type is cycled by clicking the first word of the order (up arrow = next). Production -> sell (only if goods are in stock) -> move ... H p.13
- Abbreviations used in the book: see 08-appendices.md (Appendix III). H p.12

#### 5.1.2.3 Production order (Produktionsauftrag)
- "Rents" workers and means of production to make one good. Optimal worker : means ratio depends on the good (Appendix II). H p.12-13
- Template with editable parts in bold: `[EXE text]` ("Brew beer with 6 workers at 3 kettles.") H p.13
  - Good field cycles through goods producible in this city. In Falkenhorst, clicking "Bier" gives `[EXE text]` H p.13
  - Fish ratio is 3:1, so with 3 boats the player should add 3 workers (to 9). H p.13
- Verb and means noun change with the good (Braut/Kessel, Fangt/Boote; Korn: `[EXE text]`, H p.12).
- Readme rule on deviating from the ratio: a factor may be raised as long as it would not require a proportional increase of the other. Corn (5:1): 9 workers on 1 field is fine, 10 is not; 10 workers definitely require 2 fields to avoid a productivity loss. R p.7
- Better worker pay raises output with the same means; below-average pay lowers motivation. H p.19


#### 5.1.2.4 Sell order (Verkaufsauftrag)
- Ships a number of goods with a number of transports to another city and sells them there. The transports return and are available next year. H p.13
- Template: `[EXE text]` H p.13
- Number of required transports is determined automatically by the quantity. H p.14
- Example in H p.12: `[EXE text]` (target must differ from the branch city).

#### Transport capacities (units of one good per transporter)
| Transport | Capacity | Source |
|---|---|---|
| Cart (Karren) | 100 | H p.14 |
| Ship (Schiff) | 1,000 | H p.14 |
| Balloon (Ballon) | 50 | H p.14 |
| Steam cart (Dampfkarren) | 80 | H p.14 |
| Submarine (Tauchboot) | 700 | H p.14 |
| Flying machine (Flugmaschine) | 30 | H p.14 |

#### 5.1.2.5 Move order (Schafft-Auftrag)
- Only to a target city where the player has another branch. Transports and goods both go to the target; transports do not return and goods are not sold automatically. H p.14
- Template: `Schafft 1643 Korn auf 2 Schiffen nach Greifenheim.` H p.14

#### 5.1.2.6 Fetch-loot order (Holt-Beute-Auftrag)
- If the player owns a toll castle or robber camp holding goods (after a plunder), the loot must first be fetched with transporters into one of the player's branches before it can be sold. H p.14
- Template: `Holt Beute auf 6 Ballons aus Meucheltann.` (6 balloons fetch loot from robber camp Meucheltann). H p.14

#### 5.1.2.7 Ship reachability
- Carts and balloons can go from any place to any place. H p.14
- Not reachable by ship: Moortal, Finsterwalde, Falkenhorst, Waldstein, and all four robber camps. H p.14
- Ships reach only places on rivers and coasts. H p.15

#### 5.1.2.8 Escort (Eskortierung)
- Every transport order type can be escorted. Two crossed swords appear after the first words of the order (shown as "X", e.g. "Holt Beute X..."; abbreviation "Verk. (X)" = sold escorted). H p.14, p.37
- Escort = some Landsknechte defend the goods in a raid. H p.15
- Transport cost rises "considerably" again. H p.15

#### 5.1.2.9 Residence (Wohnsitz) - control icon 1 of 6
- Start: almost empty parchment, no residence. H p.15
- Actions: buy, build, sell (if owned), auction (Versteigerung). Renovation may be needed after a storm or worse. Extensions: parks, outbuildings, extensive estates. H p.15
- The more splendid and better-kept the residence, the more dignitaries favour the player. H p.15
- Readme: a residence gives a direct bonus to the favour of all office holders in that city; residences are a prerequisite for certain noble titles. R p.9

#### 5.1.2.10 Storage (Lagerraum) - control icon 2
- Icon shows storage size in m²; 15 at game start. H p.15
- Capacity: 15 to 20 units of goods per m², depending on the goods type. H p.15
- Overflow: if storage cannot hold old stock plus production results, the excess is given away to the needy. H p.15
- Two kinds: wooden (Holz) and stone (Stein). Wooden is cheaper, but in a fire produces more smoke and so loses more storage space and possibly stored goods. H p.15
- Each year a certain number of purchase offers is made; the player picks among them. Selling storage is possible at any time without limit. H p.15

#### 5.1.2.11 Transports - the three bottom-right control icons
- Three icons: carts, ships, balloons. H p.15
- Cart: cheap, reaches every destination, easily raided. H p.15
- Ship: expensive at start, carries large quantities, only river/coast places. H p.15
- Balloon: quite expensive, small capacity, sometimes drifts off course; cannot be raided by anyone. H p.15-16
- Per transport menu: build orders, sell, repair; auction if an auction is scheduled. H p.16

#### 5.1.2.12 Director (Direktor), part 1 - seal ring icon
- Seal ring control icon opens the window to hire a director to run the branch. H p.16
- Pay: depending on his skill, a yearly percentage of the yearly allotment the player sets in the counting room for that branch. The remainder runs the branch. H p.16, R p.8
- Advantages listed: saves time; maximises production experience (player's and director's skills combine: player skilled in corn + director master weaver -> both corn and cloth produced well); checks repairs (carts, ships); speculates (e.g. cheap stone storage); avoids saturated markets. H p.16
- Readme adds: reacts to demand changes and sells where profit is highest; supplies building material to cities after a fire; adapts to sieges; never forgets escorts; optimises quantities; re-activates transport orders that became impossible. R p.8
- Disadvantages: costs money; unexpected reactions to demand; directors enrich themselves from the player's assets if they think they deserve it. H p.17
- Readme: a director may quit at round end, e.g. when the player's favour with him is too low; each director is an independent game character with an office who can become a target or opponent. R p.9
- Readme guidance (non-binding): play a new branch yourself first; consider a director above 20-30 workers; 10,000, later 20,000 or 25,000 thalers maximum per city should suffice. R p.9

Control-icon count: "the first of the six icons" (H p.15). Icons named: residence (1), storage (2), seal ring/director, and three transport icons bottom right. The seal ring's position among the icons is not stated.

---

## 5.2 Counting room (Schreibstube)

| Hotspot | Opens | Source |
|---|---|---|
| Letter in front of the bookkeeper | applications for offices | H p.17 |
| Further right on the desk ("Privilegien") | privileges menu | H p.18 |
| Book stack right of the bookkeeper | law books | H p.18 |
| Left book stack | moneylender | H p.18 |
| Left half of the bookshelf wall | loan book | H p.19 |
| Bookshelf right of the bookkeeper's face | director budgets | H p.19 |

### 5.2.1 Applications (Bewerbungen)
- No application possible unless the player is naturalised (eingebürgert) in a city. H p.17
- Dignitaries die or are dismissed; lower holders move up (example chain: mayor -> privy councillor, judge -> mayor, alderman -> judge). A vacancy is announced at round end (`[EXE text]`). H p.17
- The player applies by switching "Nein" to "Ja" at the end of the line. The crystal at the start of the line shows the competitors. H p.17
- Usually several candidates. Certain offices appoint other offices (Appendix I). Check favour with the appointers via favour bars; bribe if unsure. H p.17-18

### 5.2.2 Privileges (Privilegien)
- Office-specific actions available only to the holder. H p.18
- Examples stated:
  - Treasurer (Kämmerer): lower the wage level in the city, reducing wages the player pays. H p.18
  - Canon (Domherr): forbid weddings in the city. H p.18, p.57, R p.9
  - Master builder (Baumeister): set the construction price level. H p.27
  - Deacon (Diakon): preferred consideration in the bishop election. H p.50
  - Finance minister: change money laws; justice minister: change criminal laws; archbishop: enact/strike church laws. H p.54
  - Bring someone to trial (`[EXE text]`). H p.26
  - Relief army for a besieged city ("Entsatzheer mithilfe Eures entsprechenden Privilegs"). R p.8
  - Make a will (`[EXE text]`). R p.9
- Blackmail grants temporary use of the victim's privileges (see 5.6.3).

### 5.2.3 Law books
- About 30 written laws exist that can be broken. H p.18. Structure: three books of ten laws each. H p.54 (see 13-law-books.md)
- Offences can be used successfully in court only if the accusations are based on spy investigations. H p.18

### 5.2.4 Moneylender (Geldleiher)
- Loan offers with high interest; over 30 percent per year is "not unusual". H p.18
- Narrative example: 27 percent loan. H p.58

### 5.2.5 Loan book (Kreditbuch)
- Lists loans taken with amount, interest and term. Loans can be repaid here. H p.19
- At loan expiry, if the player cannot pay, the lender's bailiffs bring the player before the jury court (Geschworenengericht); escape is possible only by paying the jury court a suitable tribute (bribe). H p.19

### 5.2.6 Director budgets (Direktoren zweiter Teil)
- Set the yearly gold amount per director. H p.19
- The director computes his salary from the budget. Sales taxes and church tithes are still due on top. H p.19
- Rule of thumb: give directors in total no more than two thirds of the player's gold; up to three quarters if they have proved steady over some years. Keep the rest for tax and tithe. H p.19

---

## 5.3 Payment (Bezahlung) - chest
- Set wages of subordinates. Four coin columns; one is "Arbeiter". H p.4, p.19
- Underpaid directors may exceed their budget. H p.19
- Below-average worker pay lowers motivation; better pay makes workers produce more with the same means. H p.19
- Mercenary and robber motivation depends directly on their pay; better pay = better fighting. H p.19
- Paying workers starvation wages is an offence (H p.53).

## 5.4 Church (Kirche) - cross
### 5.4.1 Outside
| Click | Effect | Source |
|---|---|---|
| Meadow left of church | leave the church (pay a price) | H p.20 |
| Meadow right of church | convert to the other faith | H p.20 |
| Bell tower | marriage (matchmaker, "Hochzeitsglocken") | H p.20, R p.9 |
| Portal | enter church | H p.20 |

### 5.4.2 Inside
| Action | Effect | Source |
|---|---|---|
| Confessional: buy indulgence (Ablass) | cost grows with offences; atones all offences so the player can hardly be convicted | H p.20 |
| Donation for the poor | raises standing with secular offices of the principality | H p.21 |
| Donate candles | may raise standing with ecclesiastical offices | H p.21 |
| Pray (click one of the two crosses) | prays (effect not stated) | H p.21 |

## 5.5 Toll castles and robber camps
### 5.5.1 Overview map (crossed swords)
- Shows the 3 toll castles and 4 robber camps. H p.21
- One toll castle per county: Mooringen - Waldenfels; Auenthal - Seefels; Bergenhöh - Hohenfels. H p.21
- Click name: current owner and some data. Click graphic: register purchase interest with the owner; if the player is wealthy enough an offer may follow "some time later". H p.21
- Toll castle: toll income and ability to act against robber camps. Robber camp: raid peaceful merchants or other camps; some robber captains are said to have attacked a toll castle. Information/purchase for camps works like castles. H p.21

### 5.5.2 Inside (click graphic of an owned castle/camp)
Layout like a branch: top = four troop types; middle = order book; bottom = control icons. H p.21

#### 5.5.2.1 Troop types
| Slot | Toll castle | Robber camp | Weapon / note | Source |
|---|---|---|---|---|
| 1 | Söldner (mercenary) | Räuber (robber) | rapier (Degen) | H p.21 |
| 2 | Musketier | Flintenschütze (rifleman) | musket | H p.21 |
| 3 | Kanonier | Kanonier | cannon: balls and canister | H p.21 |
| 4 | Offizier (much stronger, rapier) | Bombenleger (bomb layer) | bomb: sneaks up, lays bomb, flees; may blow himself up | H p.21-22 |

(Slot order is the listing order in H p.21; conversion pairing from the Schickt example, H p.23. See open questions.)

- Click a troop's torso: info window with current training level of that group. H p.22
- Four numbers below torsos = men present. Clicking with "-" dismisses men; hiring is more complicated (recruiting). H p.22
- Pay (Sold): "roughly 100 gold pieces per man" per year; officers earn more than ordinary mercenaries. Due at the end of every year whether or not orders were given. H p.22

#### 5.5.2.2 Order book
- Works like the branch order book; order types depend on castle vs camp; same status semantics; type cycled by clicking the first word. H p.22
- Four editable two-digit troop counts per order. H p.23

#### Order types
| Order | Available at | Template | Effect | Source |
|---|---|---|---|---|
| Überfallt (attack) | castle and camp | `[EXE text]` | attack any castle or camp not owned by the player; goal: reduce security (castle) / camouflage (camp) via cannoneers, or conquer. Target field cycles to next castle/camp not owned by player | H p.23 |
| Schickt (send) | castle and camp | `Schickt Hohenfels 04+02+07+03 Räuber.` | send troops to any castle or camp (own or another's); castle<->camp transfers convert types | H p.23 |
| Plündert (plunder) | camp only | `[EXE text]` | ambush transports whose destination city lies in the target county; not guaranteed to find them | H p.24 |
| Überwacht (patrol) | castle only | `[EXE text]` | patrolling mercenaries may join fights when robbers raid a transport in that county; rescued merchants often tip; mercenaries are said to have allied with robbers and shared the loot | H p.24 |

- Example Überfallt counts: 8 mercenaries, 3 musketeers, 6 cannoneers, 12 officers. H p.23
- Schickt example: send 4 robbers, 2 bomb layers, 7 cannoneers, 3 riflemen to Hohenfels; arriving: 4 mercenaries, 3 musketeers, 7 cannoneers, 2 officers. H p.23
- Plunder example Bergenhöh: potential victims are transports to Greifenheim, Bergstedt, Falkenhorst. H p.24
- With large parts of the troops away, a castle/camp is an easy target. H p.23

#### 5.5.2.7 Control icons
Four (camp) or five (castle) icons: expand base, recruit, manoeuvre; castles also set toll rate. H p.24

| # | Icon | Function | Source |
|---|---|---|---|
| 1 | status overview | stored loot awaiting "Holt Beute"; security (castle) / camouflage (camp) level; build casemates (castle) / straw camps (camp) to raise capacity | H p.24 |
| 2 | recruit | tavern (Kneipe) / harbour dive (Hafenkaschemme); give recruiters an amount of money; result reported at year end | H p.25 |
| - | manoeuvre (cannon in castle, bomb in camp) | costs much gold, raises training | H p.25 |
| - | toll rate (castle only) | toll as share of goods value | H p.25 |
| - | seal ring | hide/show the order book | H p.25 |

- Very secure castles may deter attackers so no attack happens; well-camouflaged camps may not be found. H p.24

#### Toll obligation by transport destination
| Destination | Pays toll to |
|---|---|
| Fürstenbrück | no toll |
| Finsterwalde | Waldenfels |
| Moortal | Waldenfels |
| Lichterheide | Waldenfels |
| Greifenheim | Hohenfels |
| Bergstedt | Hohenfels |
| Falkenhorst | Hohenfels |
| Waldstein | Seefels |
| Lauenburg | Seefels |
| Kronsbach | Seefels |
H p.25

- Toll is measured on the goods value of the transport. Very high toll rates may make traders sell in a city of another county instead. H p.25

## 5.6 Back room (Hinterzimmer) - door
| Hotspot | Action | Source |
|---|---|---|
| Cloak (Umhang) | spy | H p.26 |
| Fireplace (Kamin) | denounce | H p.27 |
| Desk | blackmail letter | H p.27 |
| Pistol | hire saboteurs | H p.27 |

### 5.6.1 Spying
- Select target person, price is shown, confirm or cancel. H p.26
- After hiring, an informant symbol ("I") appears next to the favour bar for that office holder. H p.26
- Wait some years. If the target committed crimes meanwhile, the hollow "I" turns solid black = evidence held. H p.26
- Evidence uses: trial (via privilege), denouncing, blackmail. H p.26
- Spies appear in the end-of-round cost report. H p.28. Narrative: 1,870 gold per year for spying on the archbishop. H p.50

### 5.6.2 Denounce (anschwärzen)
- Requires evidence against the victim. Choose the victim first, then the person to denounce them to. H p.27
- Success depends on (1) enough evidence and (2) high standing with the listener. Goal example: get the judge dismissed by the mayor so an alderman can move up. H p.27

### 5.6.3 Blackmail (erpressen)
- Requires "very much" evidence. If successful, the player can use the victim's privileges for a certain time. H p.27
- Example 1: blackmail the master builder to set the construction price level low. H p.27
- Example 2: blackmail a county chancellor (Kanzler) so he appoints the player as justice adviser. H p.27
- After some years the victim stops yielding; then privileges are lost and the victim holds a grudge. H p.27

### 5.6.4 Sabotage
- Hire saboteurs (pistol) who try to destroy a rival's branch. Not cheap. H p.27

## 5.7 Score (Spielstand)
- Shows progress of all human players and some selected computer players. Clicking a blue crystal gives more information about that person. H p.28

## Source map
| Claim | Source |
|---|---|
| Turn sequence, >100 NPCs | H p.9 |
| Office hotspot list | H p.4-5, p.9, p.17, p.19-21, p.26, p.28 |
| Trade map flags, two click areas, city info contents | H p.9-10 |
| Branch rights one human per city; second branch unaffordable at start | H p.10 |
| Branch layout | H p.11 |
| Six goods per city, two-digit price, skill ring, stock count | H p.11 |
| Ring colour blue/gold | R p.8 |
| 4 of 6 darkened, certificates | H p.11 |
| Three local sale methods | H p.11 |
| Two orders per branch per year; four types | H p.12 |
| Cost and storage forecast; costs at end of round | H p.12 |
| Three status symbols; status persists | H p.12 |
| Production template, Falkenhorst Bier/Fisch, fish 3:1 add 3 | H p.13 |
| Corn 10 workers 2 fields example | H p.12 |
| Ratio deviation rule (9 ok, 10 not) | R p.7 |
| Sell order template; transports return | H p.13 |
| Transport capacities | H p.14 |
| Auto transporter count | H p.14 |
| Schafft rules | H p.14 |
| Holt Beute rules | H p.14 |
| Non-navigable places | H p.14 |
| Escort marker and cost | H p.14-15 |
| Residence actions | H p.15 |
| Residence favour bonus, title prerequisite | R p.9 |
| Storage 15 m², 15-20 units/m², overflow donation, wood vs stone | H p.15 |
| Storage offers per year; sell anytime | H p.15 |
| Cart/ship/balloon properties; transport menu | H p.15-16 |
| Director hire, pay as % of budget, pros/cons | H p.16-17 |
| Director readme pros, quitting, guidance numbers | R p.8-9 |
| Counting room hotspots | H p.17-19 |
| Naturalisation requirement; promotion chain; Nein/Ja; crystal | H p.17 |
| Privilege examples | H p.18, p.26-27, p.50, p.54, R p.8-9 |
| ~30 laws; spy-based evidence only | H p.18 |
| Interest >30 % | H p.18 |
| Loan book, jury court at expiry | H p.19 |
| Director budget, taxes on top, 2/3 and 3/4 rules | H p.19 |
| Payment effects | H p.19 |
| Church outside/inside actions | H p.20-21 |
| Matchmaker via bell tower | R p.9 |
| Castle per county; purchase interest | H p.21 |
| Troop types and weapons | H p.21-22 |
| Training info, dismiss, ~100 gold/man/year | H p.22 |
| Castle/camp order types and examples | H p.23-24 |
| Control icons 4/5; overview; recruit; manoeuvre; toll; hide book | H p.24-25 |
| Toll table | H p.25 |
| Back room actions | H p.26-27 |
| Score window | H p.28 |


## Open questions
1. Production formula: output as a function of workers, means, ratio, location yield, player/director skill, wage level, worker motivation. Readme example implies required means = floor(workers / ratio)? Confirm, and what is the penalty for imbalance?
2. Which six goods does each of the ten cities produce? Handbook only names two "best" cities per good (10-trade-goods-relations.md) and that Falkenhorst produces beer and fish.
3. Price formula (popularity vs wealth, yield, market saturation from repeated sales, siege/fire effects).
4. Cost formulas: worker wage, means-of-production rent, transport cost per transporter and distance, escort surcharge ("considerably").
5. Can a production and a sell order exist in the same year when stock is empty at turn start? Order type cycle order when there is no stock (H p.13 says sell only if goods in stock).
6. Units per m² per good (15-20 range): exact table.
7. Branch rights price formula; storage offer count per year and pricing; storage wood vs stone price ratio.
8. Transport purchase/build/repair prices; build lead time; wear model ("Muscheln", "in Schuss"); balloon drift probability and consequence.
9. How are steam carts, submarines and flying machines obtained, given only three transport icons (cart, ship, balloon)? Are they adventure rewards? Do submarines follow ship reachability?
10. Is the mixed "Verkauft ... auf N Karren" transporter type selectable per order (cart/ship/balloon/others)?
11. Fetch-loot: can loot be fetched by ship from a toll castle on water (Seefels on the lake)? Toll castles are not in the non-navigable list.
12. Director pay percentage range and how it depends on skill; how a director's skill combines with the player's (max?); embezzlement rule.
13. Payment screen: names of the four columns (workers, directors, mercenaries, robbers?), range, and the formula linking pay to motivation and to fighting strength.
14. Church: price to leave the church and to convert; effect of being churchless; indulgence price formula; donation/candle effect sizes; effect of praying (H p.58 hints prayer/candles influence fate).
15. Toll castle/camp purchase: when is the offer made, price formula, what happens to the seller.
16. Security/camouflage scale; casemate/straw camp cost and capacity per unit; max troops.
17. Recruiting: men gained per gold; which types.
18. Manoeuvre cost and training gain; training scale.
19. Exact pay per troop type (officers higher than ~100).
20. Toll rate range; how NPC traders react to high tolls.
21. Order slot count in castle/camp order books (two as in branches?).
22. Troop slot order vs conversion: Überfallt example lists "08+03+06+12" as mercenaries, musketeers, cannoneers, officers, but the Schickt example lists "04+02+07+03 Räuber" as robbers, bomb layers, cannoneers, riflemen, and arrivals "4 mercenaries, 3 musketeers, 7 cannoneers, 2 officers". The implied mapping is robber->mercenary, rifleman->musketeer, cannoneer->cannoneer, bomb layer->officer, but the slot order of the robber string differs from the listing order. Which is correct?
23. Spy price formula; how many years until evidence; amount of evidence ("enough", "very much") as a number.
24. Denounce success formula and effect (favour loss of victim with listener? dismissal chance?).
25. Blackmail success formula, duration in years, and grudge size.
26. Sabotage price, success chance, damage (storage, goods, branch destroyed?).
27. Which computer players appear in the score window and how "progress" is scored.
28. Naturalisation: H p.28 says citizenship comes in 1601 after one year in a city; is naturalisation the same as the title "Bürger"?
