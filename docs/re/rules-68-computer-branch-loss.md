# rules-68: how a computer person loses a branch

Question: in a passive run (one human, no actions, difficulty 3) property slot 1 (branch, city 3, owner computer person 111) became free (kind 3, owner 0xFF) between the 1611 and 1612 year starts while person 111 kept name, title, office and cash. The reimplementation never frees a computer branch.

Tags as in `rules-67`: [C] instruction level, [L] decompiler only, [G] guess. `rand(n)` = 0x36FF0. Property record = `rules-00` §5 (96 bytes at 0xDC86C + 96·i). Helpers: `property_free` 0x429A0, `property_find` 0x42A50(kind, location, owner), `property_create` 0x42870(kind, location, owner), `pick_random_property` 0x42B90(kind, location, owner), `pick_random_person` 0x43560(place, church, office, controller), `rec_alloc` 0x424A0, `rec_delete` 0x42550, `add_cash` 0x43790, `pay_cash` 0x43860, `branch_add_transporters` 0x43930.

## 1. Every way a property slot becomes free [C]

`property_free` 0x429A0 is the only code that writes kind 3 or owner 0xFF into a property record. Scan of the whole code segment 0x10000..0x7F000: the only immediate store of 3 to a byte through a register is 0x42A1B, the only immediate store of 0xFF to `[reg+2]` is 0x42A21, both inside 0x429A0; there is no store to 0xDC86C/0xDC86E with a displacement form. Owner-byte stores through registers elsewhere are transfers to a picked person (inheritance, `person_clear`, castle sale) and never carry 0xFF unless the picker failed (the doc of each path says so). So the complete list is the caller list of 0x429A0. Ghidra finds **five** call sites, not six as `rules-07` §3 note said:

| call site | function | spec | can hit a computer branch? |
|---|---|---|---|
| 0x2897D | `auction_settle` 0x27680, end of the auction day | this doc §2 | **yes** |
| 0x4331E | `person_clear` 0x432D0 | `rules-11` §4 step 2, `rules-40` | no: the kind test is `== 0x5B`, never true; properties are handed to a random computer person instead |
| 0x4F8F8, 0x4FA3D | city info 0x4F3C0 | `rules-11` §1 (sale of the human's own branch), `rules-48` (temporary price-quote slot) | no: the branch is `(0, city, human)`; the temporary slot is created and freed inside the same call |
| 0x64C74 | debt stage 5 inside `turn_end_settlement` 0x61A70 | `rules-02` §10, `rules-40` | no: the debtor is the human whose turn is being settled |

`person_clear` is called from 0x4DBC0 (player creation), 0x5D610 (Kontor, retirement) and three places in 0x61A70 (human death and debt paths); none frees anything.

What `property_free` does [C]: index i = (record − 0xDC86C)/96; every heap record found by `rec_find(type 0xFF, owner 0xFF, key i)` gets bit 7 of its type byte set (deleted); the three iterator globals 0xEF490/0xEF494/0xEF498 are saved and restored around the loop; then kind := 3, owner := 0xFF. Nothing else in the 96 bytes is touched: stock, storage, transporters, order book, director byte and residence word stay in the slot until `property_create` zero-fills it for the next owner.

## 2. Auction day dummy sellers (the only computer-branch path) [C]

`auction_settle` is called from `year_end_world` at 0x688F6 whenever a type-3 record with owner 0xC0 exists (`rec_find(3, 0xC0, 0xFF)` at 0x688DC). It runs the auction only when that record's counter (byte 3) is 0. Position in the year end: after the office cycle, after `rand(100) < 12 → disaster_damage 0x18BD0` (0x68862), after `settlement_pre` 0x61410 for every computer person (0x6887C..0x688CB), before the "Ereignisse in Mittelland" window (0x688FB). See `rules-07` §3 "Year-end order" and `rules-12` §2.

### 2.1 The dummy loop (0x276A5..0x278A4), six iterations i = 0..5

Per iteration, in this order:

1. `s = pick_random_person(0xFF, 0xFF, 0xFF, 0)` = a uniformly random **computer** person (controller nibble 0, occupied slots only). One `rand(n_computer)` draw if n ≥ 1; if there is no computer person the result is 0xFF, entry i := none, no further draws, next i.
2. `h = pick_random_person(0xFF, 0xFF, 0xFF, 1)` = a uniformly random human. One `rand(n_human)` draw (n ≥ 1 in any running game).
   - If h = 0xFF (no human): `city = rand(10)`.
   - Else `b = pick_random_property(0, 0xFF, h)` = a random branch of h, one `rand(n_branches)` draw if h has at least one branch; `city = b.location`. (The `rules-08` §1.2 phrase "first branch" is wrong: 0x42B90 draws.) If h has no branch, 0x42B90 returns null and the location byte is read from address 1 [C for the missing test, the value is whatever the loader left there].
3. `P = property_find(0, city, s)` = s's existing branch in that city, slot order. **If s already has a branch there, that real branch is the dummy's property.** Only if none exists: `P = property_create(0, city, s)`: first slot of kind 3, zero-filled, kind 0, owner s, residence word 0xFFFF, 1 Karren, all conditions 100, storage 15 wood / 0 stone, no director. Null if all 192 slots are used; then entry i := none and the lot rolls below are skipped (no draws).
4. `P.residence_word (+3) := (rand(100) << 8) | (rand(8) << 5) | rand(10)`, three draws in that order. On a found branch this overwrites the real residence.
5. `rand(100) < 30`: `rec_alloc(12, s, P_index, 1)`; +4 transporter type := `1 + rand(5)`; +5 count := `2 + rand((year − 1600)/5 + 5)` (signed division of the year word 0xEF4B8); +6 condition := `rand(100)`. Four draws when the roll hits, one otherwise. The transporters are **not** taken from P; the lot is created from nothing.
6. `rand(100) < 20`: `rec_alloc(14, s, P_index, 1)`. One draw always.

Draw count per iteration: 2 (+1 if h has a branch; or 1 for `rand(10)` if no human) + 3 + 1 (+3) + 1. The same person may be picked for several i; the second pick then *finds* the branch the first pick created, and both entries point at the same slot.

The six entries are the local array `[ebp−0x80 + 4i]`.

### 2.2 During the auction

Everything in `rules-08` §1.3/1.4 and `rules-15` §1 applies. Relevant to the found-branch case:

- Catalogue and lots use P's city for transporter prices (city field +0x73+2t of P.location) and P's residence word for the residence lot.
- A transporter lot of a dummy: sold → the winner's branch receives the transporters (`branch_add_transporters`), winner pays, **seller s (= P.owner byte, read at 0x2832D) receives the price** via `add_cash` (clamped at 9 999 999, `rules-60`). Unsold, or nobody attends → the lot's transporters are **added to P** by `branch_add_transporters(P, type, count, condition)` (0x28368). So a real branch can gain transporters minutes before it is freed.
- A residence lot of a dummy: sold → winner pays, s receives the price, P's residence word := 0xFFFF (0x520A0 at 0x28583). Unsold → the random residence word from 2.1 step 4 stays on P until the free.
- Lot records are deleted one by one (`rec_delete`), then the type-3 auction record.

No report or history text names the dummy branch or its removal. The only texts are the auction-day screen texts of `rules-12` §3.5/§3.6 (catalogue 0xBAE64/0xBAF20, "Zur Versteigerung gelangt jetzt" 0xB72C8/0xB72CC pointers, `[EXE text]` 0xB71D8, 0xB71CC/0xB71D0). The year-end report gets nothing; `history` 0x17930 is not called from 0x27680.

### 2.3 The free (0x28947..0x28984)

After the closing text and the window close: for i = 0..5, if entry i is non-null, `property_free(entry i)`. No test whether the slot was created or found. Then the pause counter 0x85E06 := 4.

Effects on a found real branch of computer person s:

| item | effect |
|---|---|
| kind / owner | 3 / 0xFF |
| stock, storage, order book, transporters (incl. ones returned from an unsold lot), transporter conditions, director byte and budget | left in the dead slot, invisible; zero-filled by the next `property_create` that reuses the slot |
| residence | already overwritten by the random word (or 0xFFFF if the lot sold); lost with the slot |
| heap records with key byte 2 = slot index | deleted: shipments in transit from that branch (type 9), construction / renovation records (13), any remaining lot (12, 14), director and other contracts keyed on the property. Records keyed on s by owner byte only are unaffected |
| cash of s | unchanged by the free itself; s keeps whatever the lots earned (`rules-08` §1.3) |
| person record of s | untouched: name, title, office, favour and cash stay. This is the signature the lead observed: branch gone, person intact |
| other owners | none |

Consequences for the AI: `npc_act` (`rules-09` §2) looks up `B = property_find(0, office place, v)`; after the free B is none for that city and the rows that need B are skipped until the AI founds a new branch there.

### 2.4 Frequency and the observed case [C for the mechanics, G for the magnitude]

With one human holding one branch, every dummy slot targets that human's city. So each auction day frees the branch of every computer person who (a) is drawn in one of the six `rand(n_computer)` picks and (b) has a branch in the human's city. New-game setup places several computer branches in the human's start city, so a few are at risk per auction. Auctions recur every `3..5 + 4 + geometric(20 %)` years (`rules-08` §1.1), so one to four per 30-year run. A loss of about one branch per 30 years across 100 games is the right order of magnitude; the lead's 48 → 47 average is consistent, not a verification.

In the reported case the auction day of 1611 (type-3 counter reached 0) drew person 111 and city 3 (the human's branch city), found slot 1, and freed it at the end of the day. Person 111 would also have been credited the price of any sold dummy lot that year.

### 2.5 Zero branches

Yes. A computer person whose only branch is in the chosen city, drawn as a dummy seller, ends the auction day with no branch. Nothing re-creates it. The person is otherwise normal (office, title, cash); `rules-09` rows that need a branch at the office place do nothing for them. On the created-branch path the slot count is unchanged (create then free in the same call).

## 3. What the reimplementation has to do for parity

Reproduce 2.1 exactly, including the find-before-create step and the unconditional free of all six entries, draw order as listed; credit sale proceeds to the slot's owner byte; return unsold transporters to the slot before freeing it. The slot's residual bytes do not matter for parity unless a save made between the free and the next create is compared byte for byte (the autosave is). Leave them as they were, do not zero on free.

## 4. Open

- Whether `pick_random_person` counts persons in the debtors' tower or dead-this-year slots (0x433E0 tests only the controller nibble against the filter and slot byte 0 != 0xFF) [L].
- The exact city when the picked human has no branch (null-pointer read) [L].
