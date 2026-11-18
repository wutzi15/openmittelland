# Career: titles, offices, appointments, dismissals

Confidence tags as in `rules-00-map.md`. Field offsets in `rules-00-map.md`. "favour(a→b)" = person a's favour byte toward person b (person a, offset 0x20 + b).

## 1. Favour primitives [C]

- `favour_decrease(a, b, x)` (0x436F0): `x' = x`, doubled if a and b differ in religion (person +0x03); `favour(b→a) = max(0, favour(b→a) - x')`.
- `favour_increase(a, b, x)` (0x43620): only if `favour(b→a) <= 100`: `x' = x`, halved (integer) if religion differs; `favour(b→a) = min(100, favour(b→a) + x')`.
- Values above 100 are not normal favour: they mark "b is blackmailed by a" (section 6). `favour_increase` leaves them alone.
- New persons start with favour 48 + rand(5) toward everyone and 100 toward themselves.
- Year end: every favour value below 10 (any pair, all persons) rises by rand(2) (0 or 1), evaluated person by person, target by target, in index order.

## 2. Titles

Title index (person +0x0D): 0 Herr/Frau, 1 Bürger, 2 Edelmann, 3 Ritter, 4 Landgraf, 5 Freiherr, 6 Baron, 7 Graf, 8 Herzog, 9 Fürst (female forms in `rules-00-map.md`). Maximum 9.

### 2.1 Humans (`title_award` 0x49700, called at the start of every human's settlement) [C]
Not in 1600. Steps:
1. With probability `title * 6` % nothing happens this year (`rand(100) < title*6`).
2. `k` = number of entries in `THR` that are `<= max(0, net_worth)`, `THR` (`0x866E8`) = 0, 6000, 10000, 22000, 52000, 102000, 180000, 280000, 420000, 600000. So `k >= 1`.
3. `res` = highest residence level over all the player's branches (property +0x03 as u16, low 5 bits; 0xFFFF = none counts as nothing).
4. `cap = min(res, k - 1)`.
5. If `cap > title` and `rand(100) < 80`: award `title + 1`. Otherwise, if `title == 0` (and year >= 1601): award title 1 anyway. Otherwise nothing.
6. No award if the title is already 9.

So title n (n >= 2) needs net worth >= THR[n] and a residence of level >= n; one step per year at most. Title 1 (Bürger) comes automatically in 1601 (handbook p.28 ✓).

Awarding authority (text only): title 1 the Bürgermeister of the player's first branch city (fallbacks: any office holder there, any random person); title 2 the Vogt of that city's county; titles 3, 4 the Regent; titles 5..9 a fixed record type 1 owner (the Emperor, [G]).

[UNVERIFIED: residence level numbering (Kate = 1 ... Schloß = 10?) is decoded in the residence doc; the low-5-bit field is confirmed, the mapping to names is not yet.]

### 2.2 Computer persons [C]
- At year end, every computer person with office level location < 13 (city or county office, or no office) gains +1 title if below the cap of his office (`0x86FB8`, by global office index): Ratsherr 1, Baumeister/Richter/Kämmerer 2, Bürgermeister 3, Priester 1, Domherr 3, Geheimrat 4, Hauptmann/Justiz-/Finanzberater 5, Kanzler 6, Vogt 7, Kellermeister/Sakristan 4, Diakon/Abt 5, Bischof 6, Hofrat 8, Marschall/Justiz-/Finanzminister 9, Großkanzler 10, Regent 11, Erzdiakon/Inquisitor 8, Erzbischof 10. Persons without office use index 0 (cap 1). Principality office holders do not gain titles this way.
- On appointment (4.5) a computer person gains +1 title if title < 3 (any office), < 6 (county or principality office) or < 9 (principality office).
- World generation: city office holders 1 + rand(2), county 3 + rand(4), principality 6 + rand(4), others 1 + rand(2).

## 3. Offices and who appoints whom

Appointer table (`0x87318`, 3 bytes per global office index, 0xFF = unused) [C]. Byte = `(index within level << 4) | place`, place 0xF = same location as the office, 0xE = the county of the office's city, otherwise an absolute location id.

| office | appointed by | matches Appendix I |
|---|---|---|
| Ratsherr (3) | Baumeister, Richter, Kämmerer of the city | ✓ |
| Baumeister, Richter, Kämmerer | Bürgermeister of the city | ✓ |
| Bürgermeister | the 3 Ratsherren of the city | ✓ |
| Priester (2) | Domherr of the city | ✓ |
| Domherr | Bischof of the city's county | ✓ |
| Geheimrat (3) | Hauptmann, Justizberater, Finanzberater of the county | ✓ |
| Hauptmann, Justiz-, Finanzberater | Kanzler of the county | ✓ |
| Kanzler | Vogt of the county | ✓ |
| Vogt | nobody (lot) | ✓ "Losentscheid" |
| Kellermeister, Sakristan | Abt of the county | ✓ |
| Diakon | Bischof of the county | ✓ |
| Abt | Kellermeister, Sakristan, Bischof of the county | ✓ |
| Bischof | Erzbischof | ✓ |
| Hofrat (3) | Marschall, Justizminister, Finanzminister | ✓ |
| Marschall, Justiz-, Finanzminister | Großkanzler | ✓ |
| Großkanzler | Regent | ✓ |
| Regent | nobody (lot) | ✓ |
| Erzdiakon, Inquisitor | Erzbischof | ✓ |
| Erzbischof | the 3 Bischöfe | ✓ |

### 3.1 Career levels (person +0x12 secular, +0x13 church) [C]
Each person has a highest reachable global office index per line. New persons: secular 2, church 8 (Ratsherr and Priester reachable). When appointed to office o, each level is raised to at least `UNLOCK_S[o]` / `UNLOCK_C[o]` (`0x868E9` / `0x8690B`):

| office o | UNLOCK_S | UNLOCK_C |
|---|---|---|
| Ratsherr | 5 | 8 |
| Baumeister, Richter, Kämmerer | 6 | 9 |
| Bürgermeister | 12 | 19 |
| Priester | 5 | 9 |
| Domherr | 6 | 19 |
| Geheimrat | 15 | 21 |
| Hauptmann, Justiz-, Finanzberater | 16 | 22 |
| Kanzler | 25 | 32 |
| Vogt | 25 | 32 |
| Kellermeister, Sakristan | 15 | 21 |
| Diakon | 16 | 22 |
| Abt | 16 | 22 |
| Bischof | 25 | 32 |
| Hofrat | 28 | 32 |
| Marschall, Justiz-, Finanzminister | 29 | 33 |
| Großkanzler | 30 | 33 |
| Regent | 30 | 33 |
| Erzdiakon, Inquisitor | 28 | 33 |
| Erzbischof | 30 | 33 |

(World generation sets the levels of each initial office holder to its office's values.)

### 3.2 Eligibility (`office_eligibility(p, place, idx)` 0x47C20) [C]
Returns 0 (not eligible), 1 (eligible, but not a promotion) or 2 (eligible and a promotion). Let `church` = 1 for church offices, `o` = global index.
1. Not eligible if: title 0; title < 3 and place is a county or the principality; title < 6 and place is the principality; religion = none (2) and the office is a church office; idx = 15.
2. Not eligible if the line's career level (+0x12 secular or +0x13 church) < o.
3. Computer persons that hold an office: a city office holder may only move within his own city, except that a Bürgermeister or Domherr may also take offices of his county; a county office holder may only move within his own county, except that a Kanzler, Vogt or Bischof may also take principality offices.
4. Humans: a city office needs a branch in that city; a county office needs a branch in one of its cities; principality offices need no branch.
5. The office must be vacant.
6. Result 2 if p holds no office or the title cap (`0x86FB8`) of p's current office is lower than that of o; else 1.

## 4. Year-end office cycle (in `year_end_world` 0x66220) [C unless marked]

Order inside the year end:

### 4.1 Rival friction
For each human h (ascending slot): `r = human_rival_of(h)`: the computer person with the lowest favour toward h, preferring persons whose office location is h's office location (or, if h has no office, the city of h's first branch, or rand(10) if h has no branch). Then `favour_decrease(h, r, rand(3))`.

### 4.2 Favour floor
Every favour < 10 rises by rand(2) (section 1).

### 4.3 Dismissal for losing wealth
For each person p (index order) that holds an office: compute net worth `NW`; let `old` = cached net worth (+0x1C). If `NW < old`, or the flag +0xF7 is set:
- `r = NW * 100 / old`.
- For each of the up to 3 appointers a of p's office (table above; a missing holder is skipped): `favour_decrease(p, a, favour(a→p) - favour(a→p) * r / 100 + 1)` (note: favour of a toward p is lowered roughly to `r` % of itself, minus 1; the religion doubling applies). Count `n_app` appointers found and `n_low` computer appointers whose favour toward p is now < 30.
- If `n_low > n_app / 2`: p is dismissed (`office_vacate`); if p is a computer person his records of some types are purged (mode 3, 0x42F60).
Then the cache +0x1C = NW (for everyone), and +0xF7 is cleared.

Vogt and Regent have no appointers and are never dismissed this way.

`office_vacate(p)` (0x45980): office index 15, location 0, church flag cleared; every favour of p toward others above 100 is reset to 30 (whoever blackmailed p loses that hold, section 6); a computer p gets new random skills `83 + rand(34)` for all 15 goods.

### 4.4 Wealth drift
- Computer persons: title cap step (2.2), one `rand(5)` call whose result is unused (it still advances the RNG), `skill_gain(p, rand(15), x)` [x not decoded], then `person_init_wealth(p, 20, 7000)`: cash moves by `20 % * (target - NW)` toward `target = OFFICE_WEALTH[o]`, at most 7000 per year (in either direction).
- Humans: `OFFICE_WEALTH[o] = max(0, OFFICE_WEALTH[o] + (NW - OFFICE_WEALTH[o]) / 10)` for the human's office o; humans without office adjust index 0 (Ratsherr). Quirk: so office wealth targets of computer persons follow the humans' wealth.
- `OFFICE_WEALTH` (`0x85E7A`) starts as a copy of `0x86BD0` by global index: Ratsherr 5000, Baumeister/Richter/Kämmerer 20000, Bürgermeister 40000, Priester 8000, Domherr 30000, Geheimrat 50000, Hauptmann/Justiz-/Finanzberater 100000, Kanzler, Vogt 120000, Kellermeister, Sakristan 50000, Diakon, Abt, Bischof 100000, Hofrat 200000, Marschall/Justiz-/Finanzminister 300000, Großkanzler, Regent 500000, Erzdiakon, Inquisitor 200000, Erzbischof 500000. World generation sets every person to its office target (`person_init_wealth(p, 100, 10000000)`).

### 4.5 Computer person turnover
For each location id L = 0..13: with probability `15 + 10*(L < 13) + 10*(L < 10)` % (cities 35 %, counties 25 %, principality 15 %): pick a random computer person whose office location byte equals L (persons without office have location 0 and are in the Fürstenbrück pool). If he owns no castle and no camp, directs no branch, is not the spouse of a human, is not the target of a courtship (type-30 record byte 9) and is not listed in a type-44 record: all his records are purged, and the slot gets a fresh identity: female with probability 1/4 (name from 70) else male (name from 117), no office, title 1 + rand(2), wealth set to the target. His office becomes vacant [C]: the same slot is re-initialised by 0x42CF0, which sets place 0, office index 15, church flag 0. The re-initialisation also resets: cash 750 then wealth init to the no-office target, life 25 + rand(15), his own favour row toward everyone to 48 + rand(5) (other persons' favour toward the slot is NOT reset, so the newcomer inherits how others felt about his predecessor), all 15 skills to 100, career levels to 2 / 8, spouse none. RNG order per replacement: rand(4), name roll, then inside 0x42CF0: rand(10), rand(15), 192 x rand(5), then title rand(2). [This is how computer office holders "go the way of all flesh" (H p.29); deaths of humans are separate.]

### 4.6 Filling vacancies (only from 1601)
Places are processed from 13 (principality) down to 0, and inside a place from the highest office index down to 0. The three Ratsherren and the two city Priester are each processed as one group (a vacancy in any seat of the group fills the lowest free seat). For each vacant office:

Electors: for each appointer slot of the office: the holder a (if any). If anyone blackmails a (some person x with favour(a→x) > 100), every such x votes instead of a; otherwise a votes. [C]

Privileged candidate: for Erzbischof the Erzdiakon, for Bischof the county's Diakon, for Abt the county's Sakristan [L: idx 9 read as Sakristan]. If that person has applied for this office (type-5 record) and is not in prison (types 20, 21), he is the only candidate. (H p.50 ✓ for the Diakon.)

Candidates otherwise: all applicants (type-5 records with this place and office) not in prison.

No applicant at all: every computer person with eligibility 2 and not listed in a type-44 record is a potential candidate; 3 of them are drawn with replacement (rand(count)) and duplicate draws are dropped; humans with eligibility 2 are counted. If any human is eligible, the office stays vacant this year (so humans can apply next turn). Otherwise the candidates are: the drawn computer persons, plus every current human's rival (4.1) that has eligibility 2 and is not listed in a type-44 record (rivals are added directly, without the draw). Duplicates (same person, or same name index +0x08) are dropped, each remaining candidate gets an application record (counter 2) and the election runs now.

Decision:
- One candidate (and no privileged one): he gets the office.
- Otherwise each elector votes once, in elector order:
  - A human elector chooses on screen.
  - A computer elector e scores each candidate c and votes for the highest score (first maximum wins):
    ```
    score = favour(e→c) + title(c)
    if c holds an office of the same kind (secular/church) as the vacant one:
        if c's office location == the vacant office's location:
            colleagues = computer persons at that location with the same kind flag
            if sum of favour(q→c) over colleagues > 0:
                score += sum / (count * 8)
        score += (33 - max(0, o - current_global_index(c))) >> 1
        if c is human: score += rand(20)
    ```
  - After each vote: `favour_increase(e, winner, rand(25))`, and `favour_decrease(e, loser, rand(10))` for every other candidate.
- The candidate with most votes wins; ties are broken uniformly at random (rand(number tied)).

Appointment: the winner's office fields are set; career levels raised (3.1); a human with certificate level (+0x05) 0 appointed to any office, or with level 1 appointed to a county or principality office with index within level >= 2, gets flag +0x06 = 1 (a certificate award follows, [G]); computer winners may gain a title (2.2); all application records of the winner are deleted.

After the cycle, all application records with counter < 2 are deleted.

### 4.7 Human applications (`office_apply` 0x47F90) [C]
The Schreibstube lists up to 6 offices with eligibility 2, scanned from place 13 down to 0 and index high to low (grouped seats once). Toggling "Ja" creates a type-5 record (counter 2) for (place, index); toggling "Nein" deletes it. The rival list shows other applicants; computer applicants that are some human's current rival are marked.

Applications made during a turn are decremented to counter 1 at that human's turn end, used at this year's end, then deleted.

## 5. Office income and costs
Office salary: see `rules-02-economy.md` 7.11. Church offices have no salary.

## 6. Blackmail marker [L]
`favour(a→x) > 100` means "a is blackmailed by x" (x replaces a as elector; x may use a's privileges). Every human turn end, for each person a with favour(a→player) > 100: with probability 33 % the blackmail ends: favour(a→player) = rand(30), and with 50 % a takes revenge (0x5B140 code 3). Vacating a's office ends the blackmail (value set to 30). Creation of the marker is in the back-room code (laws doc).

## Open questions
1. The argument of the computer persons' yearly `skill_gain` call (EBX at the call) is not decoded.
2. Certificate award path (+0x05, +0x06) is only partly decoded.
3. Residence level numbering (needed for titles).

## Corrections (rules-27)
- §2.1: the city for the title text is a random own branch, not the first one (rules-27 §1).
