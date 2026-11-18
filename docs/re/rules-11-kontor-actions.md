# Kontor actions: branch rights, residences, local sales, money lender, church entry, giving up, directors

Confidence tags and offsets as in `rules-00-map.md`. `rand(n)` = `random_n(n)`. `V_res(r, c)` = residence value (`rules-06-events.md` §"Residence value V", function 0x44230). `value(p)` = property value (`net_worth_goods`, economy doc §8). All amounts are integer; "/" truncates.

## 1. Branch rights and selling a branch (city info window 0x4F3C0, from the trade map 0x4CC90) [C]

**Rights holder table** (`0x85E08`, one byte per location 0..20, 0xFF = free): recomputed from scratch by 0x43E60 at the start of every year end and every time the trade map is drawn: all entries 0xFF, then for every **human** person (index order) every property he owns (any kind) writes his index into the entry of its location. Computer-owned branches never block a city. The last human in index order wins if two own something at the same place.

**Buying** (click on a city where the player has no branch):
1. A new branch record is created in a free property slot (`property_create(0, city, player)`: 15 m² wood storage, 1 Karren at condition 100, no residence, 0 stock; see `rules-00` §5). If no slot is free, or the rights entry of the city is not 0xFF (some human owns something there), the offer is refused and the record is freed again.
2. Price shown and charged: `P = value(new branch)` = city +0x84 (rights base, start 12000 / 8000 / 5000, +rand(200) per year) + `A * 15` (wood level A, economy §5.2) + `TP[c][Karren]` (one Karren at condition 100) + `population[c]`.
3. On "buy": refused if `cash < P` **as an unsigned comparison** [C]: a negative cash value is a huge unsigned number, so a player in debt passes the check [quirk]. Otherwise pay P (`person_pay`), keep the record (the uncle's order book is not copied: the new branch has no orders).
4. There is no limit on the number of branches here; `MaxBr` (`0x85A2C`) only adds the yearly extra charge of economy §7 step 10. One human per city is enforced only through the rights table.

**Selling** (own branch, "verkaufen"): only if the player owns at least one other branch (any city; otherwise a refusal text). Receive `value(branch) * 2 / 3` (`person_add_cash`); the property slot is freed (0x429A0: kind := 3, owner := 0xFF, and every heap record whose key byte 2 is this property index is deleted: orders in transit, constructions, director contracts etc.).

**Start city of a new human** (setup 0x4DBC0) [C]: `i = rand(4)`; loop: if `i == 8`: `i += rand(2)`; `city = T[i]`, `i += 1`, until the city's rights entry is free or i reaches 12 (then "Es gibt keinen Startort mehr"). `T` (`0x870C2`) = Lichterheide, Bergstedt, Lauenburg, Kronsbach, Lichterheide, Bergstedt, Lauenburg, Kronsbach, Moortal, Falkenhorst, Moortal, Falkenhorst. So the first four players get one of the four port cities (random start offset), later ones Moortal or Falkenhorst (H p.6 ✓). Joining is possible only before 1671 (year < 1671, [L: the refusal text for later years was not read]) and while fewer than 6 humans exist.

## 2. Residences (branch screen icon 1: 0x52240 menu, 0x52490 buy, 0x52790 build, 0x52AA0 sell, 0x52B50 auction, 0x52D00 extend, 0x53060 renovate, 0x520A0 change) [C]

**Residence word** `r` (branch +0x03, u16): 0xFFFF = none. Low byte: bits 0..4 = building level index 0..9 into `RB` (Kate .. Schloß, name table `0xB4340`), bit 5 (0x20) = Park, bit 6 (0x40) = Nebengebäude, bit 7 (0x80) = weitläufige Ländereien (`0x8711C` = 0x20, 0x40, 0x80). High byte (+0x04) = condition 0..100. Value `V_res(r, c)` as in rules-06.

**Score** (0x52020) `S(r) = (r & 0x0F) + bit5 + bit6 + bit7` (0 for none). Note the level is masked with 0x0F here but with 0x1F for the value; levels are 0..9 so both agree.

**Change of residence** `set_residence(branch, r_new)` (0x520A0), used by buy, sell and deliveries: `d = (S(r_new) - S(r_old)) * 3`; then for the branch city, then its county (`d := d / 3`), then the principality (`d := d / 3` again): every person (any controller) whose office place equals that location: `d < 0`: `favour_decrease(owner, x, -d)`; else `favour_increase(owner, x, d)`. Finally branch +0x03 := r_new. So a better residence raises favour among the city's office holders by 3 per score point, county holders by 1, principality holders by 0 (integer division: 3/3/3 = 0 for one point).

Menu preconditions (0x51EB0, flags per action): buy and build are refused while a type-14 (renovation/extension) or type-13 (construction) record exists for this branch; sell, auction, extend and renovate additionally need an existing residence (all three checks).

**Buy outright** (0x52490): three offers per city per turn (`0x85CC5 + 6c`, three u16 residence words, rolled at every turn start in 0x4B900: `r = (rand(100) << 8) | (rand(8) << 5) | rand(10)`, i.e. random condition 0..99, random extension bits, random level 0..9). The player picks one: he is credited `V_res(old) * 2 / 3` for his current residence (if any) and pays `V_res(offer) * 11 / 10`; the new residence takes effect **immediately** via `set_residence`. The offer table is not cleared [quirk: the same offer can be bought again in the same turn].

**Build** (0x52790): choose level k = 0..9; new word `r = 0x5000 | k` (condition 80, no extensions). Credited `V_res(old) * 2 / 3` (the old residence is sold even though it stays until completion); pay `V_res(r)`. A type-13 record is created (owner, key = branch property index, counter = build time `BT[k]` years, `0x866C0` = 1, 1, 1, 1, 2, 2, 2, 3, 4, 4; +4 = r). The price list shown uses `(build_level_c + 150) * V_res / 200` (city +0x81), which is display only.
**Delivery** (turn start 0x4B900): every type-13 record of the player with counter 0 **or bit 7 of the counter set** (a construction started in this turn has been decremented once at turn end; the bit-7 test catches counter underflow) is completed with `set_residence(branch, +4)` and deleted.

**Sell** (0x52AA0): receive `V_res(r) / 3`; `set_residence(branch, none)`.

**Extensions** (0x52D00): refused when all three bits are set. The player toggles any of Park / Nebengebäude / Ländereien (each shown with its cost `V_res(r | bit) - V_res(r)`); on confirm with a changed word: pay `V_res(r_new) - V_res(r_old)` (removing an extension gives money back through a negative payment), type-13 record with counter 1 and +4 = r_new (delivered at the next turn start).

**Renovate** (0x53060): target word = current low byte with condition 100 (`r_low | 0x6400`); cost `max(100, V_res(target) - V_res(r))`; pay; type-13 record, counter `1 + (condition < 25)` (2 years when the condition is below 25), +4 = target.

**Auction**: see `rules-08` (type-14 lot).

Condition decay of residences comes only from events and sabotage (rules-06, rules-04b); there is no yearly wear.

## 3. Selling stock in the branch's own city (order screen 0x50DD3, three controls) [C]

`b = 2` if the owner has a type-42 record with counter < 7 (trade-contract bonus), else 0 (computed once when the branch screen opens). Unit price `u = price[c][g] + b` (current local price).
- Click on the stock number: sells **one** unit: cash += u, stock -= 1.
- Click on the good: sells the **whole stock** n of that good: cash += n * u.
- "Empty store" icon: the whole stock of every good, in good order 0..14.
Every local sale adds the units to `sold[c][g]` (city +0x31, the saturation counter of economy §2.2) and the revenue to `sales[g][c]` (`0x85A62`), so sales tax and tithe are charged on it at this turn's settlement exactly as for shipments (economy §4.5). The price does not change during the turn (prices are only recomputed at settlement), so selling one by one or all at once yields the same total.

## 4. Money lender and loans (Schreibstube 0x5C450: money lender 0x5E1D0, loan book) [C]

**Lender mood** `M` (`0x85D45`): set at every turn start to `50 + rand(20)`.

**Taking a loan** (each click on the money lender):
1. If `M > 100`: "nobody lends" (no RNG).
2. `n` = number of the player's loans (type-56 records). `x` = a random branch of the player (`pick_property(0, player)`, one RNG call; a player without branch would crash the original [bug]). Lender pool: if the player's office place is a city (< 10): computer persons at x's city; if a county: computer persons at the county of x's city; else principality. `L = person_pick_random(pool)` (one RNG call, or none if empty).
3. If `n >= 4`: refusal ("at most four loans"), stop (the two RNG calls above have been made).
4. If L is none: refusal, stop.
5. `amount = max(100, standing(player, 30 + rand(50)))` (two RNG calls: the rand(50) and the one inside `standing`, rules-04 §1).
6. `years = 3 + rand(8)` (3..10).
7. `maxrate = 0x85A3B[county(x.city)]` = law byte "largest interest" of that county, start 35 % for all three counties (`0x85A45..0x85A47`). `rate = maxrate * M / 100 + years / 2`; at least 1, at most maxrate.
8. Offer shown ("%d Taler zu %d%% bis ..."). Accept: type-56 record (owner = player, counter = years, +4 = L, +5 u32 = amount, +9 = rate); cash += amount.
9. Whether accepted or not: `M = (M - 5 + rand(25))` as a byte; if the result is above 200 (underflow), `M = 1`. So each inquiry moves the mood by -5..+19; once it exceeds 100 the lender closes for the rest of this turn.

Handbook: H p.18 "over 30 percent is not unusual" ✓ (cap 35 %; the rate approaches the cap as the mood rises).

**Yearly interest**: economy §7 step 8 (`amount * rate / 100` per loan, not charged with privilege 57); the principal does not change.

**Loan book** (Kreditbuch): lists the loans (loans with at most 2 years left are highlighted). "Kredit tilgen" on a loan: pay `amount` (`person_pay`, no cash check, cash may go negative) and delete the record. No interest refund, no penalty.

**Expiry** (turn-end record countdown, type 56 handler): when the counter reaches 0: message, `favour_decrease(player, lender, 8)` (religion doubling applies), the player pays `amount` (`person_pay`, unconditionally, cash may go negative), with `rand(100) < 10` the lender retaliates (`retaliate(lender, player, 2)`, rules-09 §4), record deleted. There is no separate "creditors' court" for an expired loan: if the payment drives cash below the 5 % overdraft limit, the debt seizure and jury court of economy §10 run at the end of the same turn (H p.19 `[EXE text]` ✓ through that path). Net worth already subtracts outstanding loans (economy §8).

Law effect: if the finance law "Kredite" forbids loans, holding a type-56 record is an offence (rules-04b offence bit 0).

## 5. Church: entry, conversion, leaving (Kontor 0x5D610, church square 0x5CCA0) [C]

- **Kircheneintritt** (shown in the Kontor instead of "Kirche" while religion = 2, konfessionslos): free of charge; the player chooses evangelisch (0) or katholisch (1); religion := choice. No other effect (no favour change, no flag).
- **Convert** (meadow right of the church): cost `max(0, net_worth) / 30`; pay; religion := 1 - religion; turn flag bit 0x10 of `0x85CBC` (input to the "Religionsfreiheit" offence, rules-04b).
- **Leave the church** (meadow left): cost `max(0, net_worth) / 10`; pay; religion := 2. If the player holds a church office, he loses it (place 0, index 15, church flag cleared; no `office_vacate` side effects on blackmail markers). All his applications (type 5) for church offices are deleted (index >= 8, or index 7 at city level, i.e. Priester/Domherr, county and principality church offices). If he is courting (type-30 record): message, `favour_decrease(player, partner, 20)`, the courtship is deleted. He can no longer pay the tithe (economy §4.5) and cannot hold church offices (career §3.2).

## 6. Giving up ("Geld zum Fenster hinauswerfen", Kontor) [C]

After a confirmation:
1. Purge mode 1 (0x42F60): every heap record owned by the player (byte 1 = player) is deleted.
2. `person_clear(player)` (0x432D0): for every property the player owns: the code intends to free castles/camps but compares the kind byte with 0x5B, which never matches [bug], so **every** property (branches, castles, camps) is handed over: owner := a random computer person whose office place is the principality (13), or, if none, a random computer person anywhere; if no computer person exists at all the owner stays (dangling). Then the player's human name slot is cleared (if the name index < 32), the whole 248-byte person record is zeroed and byte 0 := 0xFF (slot free).
3. Phase := 6; the turn ends. The player no longer exists; with no human left the game returns to the title menu (rules-01 §3).

This is not the bankruptcy path: debt seizure (economy §10) never deletes the person.

## 7. Directors (branch icon "Siegelring": hire 0x53680, fire 0x53990; budget screen in the Schreibstube) [C]

**Hire**: candidate place = the branch's city if the owner's office place byte is below 10 (a city office, or no office at all, since office-less persons have place 0); otherwise the owner's office place (his county, or the principality). Three draws `person_pick_random(place, church flag 0, any office, computer)`; a duplicate of an earlier draw is dropped. Each shown candidate displays his fee percentage `director_fee_pct` (rules-09 §5: sum of his first 5 skills / 45). Choosing one: branch +0x53 := candidate, +0x54 (yearly budget) := 1000. **No hiring fee.** Nothing prevents the same computer person from directing several branches, or a person who directs a branch of another human.
**Fire**: confirmation; branch +0x53 := 0xFF. No cost, no favour change.
**Budget** (Schreibstube bookshelf): the player sets +0x54 per branch with the number control; no limits beyond the control's range [L: range not checked]. The director's yearly fee and how he spends the budget: economy §7 step 4 and rules-09 §5.
**Quitting**: only through the fate event 40 (rules-06 §4, director := none) and the death of the director person (computer-person turnover, career §4.5, which skips persons that direct a branch, so a director is never replaced by turnover). There is no favour- or pay-based quitting [C for the absence in the director code paths read].

## Open
1. Budget control range on the director budget screen.
2. Whether the rights table ever matters for castles and camps (it is filled for every property kind, but only the city entries are read by the purchase code).
