# Auctions, feasts, duels, church

Confidence tags as in `rules-00-map.md`. `standing(p, k)` is 0x17430 (`rules-04-backroom-laws.md` 1). `human(p)` = controller nibble set.

## 1. Auctions

### 1.1 Schedule [C]
- An auction is a type-3 record with byte 1 = 0xC0 (no owner), counter c.
- Announcement (0x269D0): only if none exists; c = `2 + rand(3)`; message.
- At year end: `auction_settle` runs if the record exists (before the 0xC0 record pass); it holds the auction when c = 0. The 0xC0 pass later in the same year end decrements c. So an auction takes place `c + 1` year ends after its announcement (3..5 years).
- After an auction the pause counter `0x85E06 = 4`. Each later year end: if the pause counter is 0, `rand(100) < 20` announces a new auction, else the counter is decremented.

### 1.2 Lots [C]
Lots are records naming a seller (byte 1) and a property (byte 2):
- type 12, transporters: +4 type, +5 count, +6 condition. Base `B = condition * price_city[type] * count / 100` (city field +0x73 + 2*type of the property's city), then `B*9/10` for Karren, `B*12/10` otherwise.
- type 14, a branch's residence: `B = ((residence & 0x0F)*4 + 80) * residence_value / 100` (0x44230).
- type 16, a castle: `B = property_value * 3 / 2`.

Players create lots from the castle screen (type 16, only while an auction is announced) and from the branch screens (types 12, 14; UI not decoded). In addition `auction_settle` creates six dummy sellers: each picks a computer person and a city (the city of a random human's first branch, or `rand(10)` if no human), finds or creates a branch of that person there, sets its residence word to `(rand(100) << 8) | (rand(8) << 5) | rand(10)`, then with `rand(100) < 30` adds a type-12 lot (type `1 + rand(5)`, count `2 + rand((year - 1600)/5 + 5)`, condition `rand(100)`) and with `rand(100) < 20` a type-14 lot. These helper branches are deleted after the auction.

### 1.3 Order of the auction day [C]
1. Catalogue: every lot is listed with its starting price (2.4).
2. If no seller is human, every human is asked whether to attend; if nobody attends, all lots are dropped unsold (dummy transporters vanish).
3. Transporter lots, then residence lots, then castle lots, each with `auction(price, kind)`:
   - transporters: kind 11 (bidder needs a branch in a port city, `0x85A00 = [1,0,0,1,1,1,0,0,1,1]`) for Schiff and Tauchboot, otherwise no bidding restriction (if the winner then has no branch, the transporters are not delivered but the price is still paid [L]). Unsold: returned to the seller's branch. Sold: added to the winner's first branch (first port branch for kind 11); the winner pays, the seller receives the price [L for the seller payment amount].
   - residences: kind = the lot's city (bidder needs a branch there without a type-13 or type-14 record on it). Sold: winner pays, seller is paid, the seller's branch loses its residence. A computer winner gets it in his branch there. A human winner who already has a residence there chooses: take the new one and be paid the value of the old one, or keep the old one and be paid the value of the new one; in both cases the new residence is installed afterwards [L, reads as a bug].
   - castles: anybody may bid. Sold: pay, seller paid, owner := winner.
4. Each lot record is deleted; the auction record is deleted; dummy branches removed; `0x85E06 = 4`.

### 1.4 Bidding (`auction_main` 0x26A50) [C unless marked]
Step table (`0x84BEC` thresholds, `0x84C38` increments): `i` = number of leading thresholds <= price; increment = `inc[i]`.

| price below | 100 | 200 | 500 | 1000 | 2000 | 5000 | 10000 | 20000 | 50000 | 100000 | 200000 | 500000 | 1 000 000 | 2 000 000 | 5 000 000 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| increment | 5 | 10 | 25 | 50 | 100 | 250 | 500 | 1000 | 2500 | 5000 | 10000 | 25000 | 50000 | 100000 | 250000 |

Start price `S = B/3`, rounded down to a multiple of its increment; 0 becomes 5. With kind 10 the function only returns S.

Bidders: up to six humans (by hotkey), and six computer bidders chosen at random: start below 1000 → `person_pick_random(rand(10))` (a city person), below 40000 → location `10 + rand(3)`, else `10 + rand(4)` (county or principality). Computer bidder j has limit `L_j = rand(300) * S / 100`.

Loop, one tick per animation step:
1. A human pressing his hotkey bids if allowed: branch rules of the lot kind (1.3), cash >= current price, not in prison (type 20/21). A refused human is excluded for the rest of the lot. Every human can press only once per lot (flag never cleared) [L].
2. Otherwise one computer bidder `j = rand(6)` bids if he is not the current high bidder and current price <= `L_j`.
3. Bids stop once the price reaches 4·S.
4. A bid raises the price by the current increment, makes the bidder high bidder, resets the hammer count.
5. A tick without a bid: from the third tick on, with `rand(100) < 70` the hammer count rises ("zum Ersten / Zweiten / Dritten"); with no high bidder the lot is withdrawn at once. ESC withdraws/ends immediately.
6. At hammer count 3 the high bidder wins at the current price.

### 1.5 Selling a castle privately (castle info 0x549C0) [L]
Interest records (type 17, at most six per castle). When the owner opens the screen and nobody is interested, with `rand(100) < 50` a random computer person of the principality level becomes interested. The owner can sell to one interested person for exactly the castle value (net worth of the property); the buyer pays (a human buyer's refusal in the confirmation dialog is ignored), ownership moves, all interest records on it are deleted. The owner can also put the castle into the next auction (needs an announced auction). Non-owners can register or withdraw interest. Computer owners sell at year end (combat doc 3).

## 2. Feasts

### 2.1 Planning (privilege 8, menu 0x20160) [C]
One planned feast per person (type 28); planning again offers to cancel it. Choices: size A (+5) 0..3 "kleines, normales, großes, riesiges" (default 1), musicians B (+4) 0..4 "sehr schlechten … sehr guten" (default 2), a city where the player has a branch, and a year from the current year to 10 years ahead (default next year). The shown cost estimate is `cost500 = (70 + rand(60)) * 500 * (year - 1570) / 3000 + 1` (0x173D0). Record: type 28, owner, key = that branch, counter = years ahead + 1. A planned feast is an offence if "Feste" is forbidden (crime doc).

### 2.2 Holding it (0x20530, from the owner's turn-end pass when the counter is 0) [C]
1. The host pays `(B + 1) * 1000 + cost500` (re-rolled).
2. Guests: `G = sum over i in 0 .. (office_income(p) + standing(p, 1)) / 130 of (rand(A + 2) + 2)`.
3. Food `3G` is taken from the branch stock, first Rindfleisch, then Fisch, then Korn; drink `2G` first Wein, then Bier. `food_short` / `drink_short` = 1 if not covered.
4. Quality `q = 3 - food_short - 2*drink_short + B/2` (0..5): furchtbares, ziemlich mäßiges, durchschnittliches, nettes, wirklich gelungenes, grandioses. Chronicle entry (0x2D750 code 2).
5. Mission kind 12 adds G when q = 5.
6. `G/4` times: `favour_increase(p, rand(192), rand(q + 1))` (random persons, including empty slots) [L for the amount argument order].
7. The record is deleted.

Handbook H p.60: feasts with office holders as guests ✓ in spirit; the guests are not modelled as persons, only as a count.

## 3. Duels (0x2C750) [C]
Created by "Beleidigung aussprechen" (privilege 26, office tree mode 6): type-22 record (insulter, +4 insulted), at most one per insulter. A human insulted person can refuse (no record); a computer always accepts. Held at the insulter's turn end (human) or year end (computer).

Score of each side x with opponent y: `office_index_of(x) + L + fencing(x) + rand(20)`, `fencing` = person +0xF6, `L` = life(x) (+0x02) for a human x, `life(y) + rand(10)` for a computer x. The insulter wins ties. Loser:
- holds an office and `rand(100) <= 32`: loses it (location 0, index 15; the seat is not refilled here);
- if human: life `-= 1 + rand(5)`.
Both favours between the two are set to 50 unless they are above 100 (blackmail kept). There is no weapon choice; the animation is picked by the two genders. H p.61 "insulted party chooses the weapon" ✗ (not a mechanic). The fencing skill rises only through privilege 27 (crime doc 4.3).

## 4. Church (0x5CCA0 Kirche, 0x5D1F0 Kirchgang) [C unless marked]
Kirche screen:
- Hochzeitsglocken: unmarried → matchmaker (family doc); married → message suggesting a mistress.
- Konvertieren: cost `net_worth / 30`; religion (+0x03) toggles between 0 and 1; turn flag 0x100000.
- Austreten: cost `net_worth / 10`; religion = 2 (none); a church office is lost (location 0, index 15, church nibble cleared); office applications (type 5) for church offices are deleted (+5 > 7, or +5 = 7 with +4 < 10); a running courtship (type 30) ends with `favour_decrease(p, partner, 20)` (the partner likes p less) and is deleted.

Kirchgang screen (prices computed at turn start: candles `standing(p, 30)`, alms `standing(p, 40)`):
- Ablass kaufen (0x59720): price `net_worth * E / 100 + 7` with E = number of evidence bits in the OR of all spy contracts on p. Pay; mission kind 8 adds the price; if p holds an office the price is added to the indulgence total `0x85E75`; every spy contract on p: inactive ones deleted, active ones get evidence 0; turn flag 0x200000. Computer persons buy without asking.
- Kerzen stiften: pay candles; every clergy person: favour toward p `+rand(3)`.
- Beten: prayer counter `0x85CBF + 1`. At year end, `rand(50) < 0x85CBF` resets it and, if a computer Erzbischof exists, triggers the miracle event (0x59940).
- Spenden für das arme Volk: pay alms; every computer person who is not clergy: favour toward p `+rand(6)`.

## Open questions
1. Branch-screen UI that creates transporter and residence lots; seller payment amount for transporter lots.
2. Mistress records (type 29, 0x20A00 / 0x20B90): family doc.
