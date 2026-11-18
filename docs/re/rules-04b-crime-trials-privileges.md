# Laws, crime detection, trials, privileges

Second half of the laws area (first half: `rules-04-backroom-laws.md`). Confidence tags as in `rules-00-map.md`. Person field offsets refer to the person record (`rules-00-map.md` section 4); `g(p)` is the global office index of person p (0x43FF0, 0 when p holds no office), `loc(p)` the office level location (person +0x0F), `human(p)` means the controller nibble (person +0x10 high nibble) is nonzero, `popcount` counts set bits of a 32-bit word.

## 1. Law state [C]

Thirty laws in three books of ten, stored as bytes in the misc-globals block. Value tables: finance and criminal books print value 0 as "erlaubt" and 1 as "verboten" (`0xB6B48`); the church book prints 0 "gottgefällig", 1 "lästerlich" (`0xB6B50`). So for flag laws **1 = forbidden**. Start values are the data-object initial values (no code writes them at world creation, except `0x85A2C`).

| book | # | address | text | start | kind |
|---|---|---|---|---|---|
| Finanz | 0 | `0x85A21` | Kredite | 0 | flag |
| | 1 | `0x85A22` | Größte Umsatzsteuer % | 15 | number 0..33 |
| | 2 | `0x85A23` | Bestechungen | 1 | flag |
| | 3 | `0x85A24` | Kirchenprivilegien | 0 | flag |
| | 4 | `0x85A25` | Mehrere Zollburgen | 0 | flag |
| | 5 | `0x85A26` | Minimaler Arbeiterlohn % | 100 | number 50..150 |
| | 6 | `0x85A27` | Minimaler Söldnerlohn % | 100 | number 50..150 |
| | 7 | `0x85A28` (u32) | Talerhöchstgrenze | 500 000 | number (editor step not decoded) |
| | 8 | `0x85A2C` | Höchstzahl Faktoreien | set by player count (economy doc) | number 2..10 |
| | 9 | `0x85A2D` | Parks und Ländereien | 1 | flag |
| Straf | 0 | `0x85A2E` | Überfälle | 1 | flag |
| | 1 | `0x85A2F` | Spionage | 1 | flag |
| | 2 | `0x85A30` | Sabotage | 1 | flag |
| | 3 | `0x85A31` | Waffenhandel | 1 | flag |
| | 4 | `0x85A32` | Keltern von Wein | 0 | flag |
| | 5 | `0x85A33` | Fischen in den Weihern | 0 | flag |
| | 6 | `0x85A34` | Belagern von Städten | 1 | flag |
| | 7 | `0x85A35` | Eskortierung | 0 | flag |
| | 8 | `0x85A36` | Räuberlager | 1 | flag |
| | 9 | `0x85A37` | Kirchliche Stiftung | 0 | flag |
| Kirche | 0 | `0x85A38` | Religionsfreiheit | 1 | flag |
| | 1 | `0x85A39` | Ablasswesen | 0 | flag |
| | 2 | `0x85A3A` | Mätressentum | 1 | flag |
| | 3 | `0x85A3B` | Fliegerei | 1 | flag |
| | 4 | `0x85A3C` | Schlösser | 1 | flag |
| | 5 | `0x85A3D` | Weltliche Stiftung | 0 | flag |
| | 6 | `0x85A3E` | Feste | 1 | flag |
| | 7 | `0x85A3F` | Edle Tuche | 1 | flag |
| | 8 | `0x85A40` | Silbernes Glanzwerk | 1 | flag |
| | 9 | `0x85A41` | Maximaler Kirchenzehnt % | 15 | number 0..25 |

Other law-like globals in the same block, changed through privileges (section 4):

| address | meaning | start |
|---|---|---|
| `0x85A42 + k` (k = county 0..2) | church tithe percent of county k (also addressed as `0x85A38[loc]` with loc 10..12) | 10, 10, 10 |
| `0x85A45 + k` (`0x85A3B[loc]`) | maximum loan interest percent of county k | 35, 35, 35 |
| `0x85A48 + k` (`0x85A3E[loc]`) | county share of toll income percent | 25, 25, 25 |
| `0x85A4B` | state religion: 0 evangelisch, 1 katholisch, 2 frei wählbar | 1 |
| `0x85A4C` | inquisition: 0 erlaubt, 1 untersagt | 0 |
| `0x85A4D` | harshness of jurisdiction 0..100, five names by value/21: sehr lasch, lasch, normal, hart, sehr hart | 50 |
| city +0x83 | sales tax percent of the city (Bürgermeister sets it) | 15 |

Evidence: law texts `0xB4EF8`, `0xB6930`, `0xB695C` paired with the flags in 0x48690 / 0x4892D / 0x48BA4; initial bytes read from the data object.

### 1.1 Law books screen and editing (0x49010) [C]
`law_book(mode)`: mode 0, 1, 2 = edit that book (finance, criminal, church); mode 3 = read-only browser that can switch between the three books with three extra entries (the last viewed book is remembered in `0x85A20`). In edit mode selecting a flag law toggles it (XOR 1); selecting a number law adds the editor delta. Ranges are enforced by the number editor: sales tax cap 0..33, minimum wages 50..150, max branches 2..10, max tithe 0..25. Who may edit: privilege 56 (Finanzminister) finance, 59 (Justizminister) criminal, 82 (Erzbischof) church (section 4).

A "strictness" value per book is displayed (0x48DE0), integer division by 2 of:
- finance: Kredite + (cap > 9) + Bestechungen + Kirchenprivilegien + Zollburgen + (min wage > 99) + (min merc wage > 99) + (cash cap < 10000) + (max branches < 4) + Parks;
- criminal: the sum of its ten flags;
- church: the sum of flags 0..8 + (max tithe > 9).
It has no effect beyond display [L].

## 2. Crime detection

### 2.1 Turn flags `0x85CBA` (u32) [C]
Cleared at every human turn start (0x4B900). Bits set by actions during the turn:

| bit (value) | set by |
|---|---|
| 0x04 | bribe (office tree, rules-04 3.1) |
| 0x08 | many privilege actions: Inquisitionsverhör trial, change of a county tithe, Grafschaft segnen, bekehren (both), Arbeiter verschwinden lassen, Zwietracht säen, Heiligsprechen, Wunder wirken (0x59940 sets it on entry) |
| 0x20000 | turn-end settlement 0x61A70 (sets byte 2 bit 1; context: transport orders with escort) [L] |
| 0x80000 | Bauwerk stiften, church building (options 0, 1) |
| 0x100000 | Staatsreligion changed; leaving the church (0x5CCA0) |
| 0x200000 | buying an indulgence (0x59720) |
| 0x2000000 | Bauwerk stiften, secular building (options 2..4) |

### 2.2 Offence word (0x447D0) [C]
`offences(p)` builds a 32-bit word, bit i corresponding to law i in table order (finance 0..9 = bits 0..9, criminal = bits 10..19, church = bits 20..29). Evaluation order:

1. `w = human(p) ? 0x85CBA : 0`.
2. bit 0 (Kredite): law forbidden and p owns a loan record (type 56 with byte 1 = p).
3. bit 1 (sales tax): p is Bürgermeister (loc < 10 and index 6) and the city's tax (+0x83) exceeds the cap `0x85A22`.
4. Bestechungen allowed: clear bit 2; then with `rand(100) < 20` set person +0xF3 bit 0x80. (The roll happens every evaluation while bribery is legal.)
5. Kirchenprivilegien allowed: clear bit 3.
6. bit 4: law forbidden and p owns at least two toll castles.
7. bit 5: worker pay level (person +0xF0) below the minimum `0x85A26`.
8. bit 6: mercenary pay level (+0xF1) below `0x85A27` and p owns a toll castle.
9. bit 7: cash (negative counted as 0) above `0x85A28`.
10. For each branch of p: bit 9 if Parks forbidden, the branch has a residence (word +0x03 != 0xFFFF) and residence byte +0x03 has bit 0x80 or 0x20 set (park / extensive grounds); bit 24 if Schlösser forbidden and the residence type (+0x03 & 0x1F) is 9 (Schloß). Then bit 8 if the number of branches exceeds `0x85A2C`.
11. bit 11: Spionage forbidden and p owns an active spy contract (type 6, +9 != 0); also sets person +0xF3 bit 0x08.
12. bit 12: Sabotage forbidden and p owns a saboteur contract (type 7); +0xF3 bit 0x04.
13. bits 13, 14, 15 (Waffen, Wein, Fisch): law forbidden, p human, and the current turn's sales array `0x85A62[g][c]` (u32, 10 cities per good, the same array the sales tax uses) is nonzero for some city, g = 14, 7, 1.
14. bit 16: Belagern forbidden and p owns a siege record (type 25); +0xF3 bit 0x20.
15. Eskortierung allowed: clear bit 17.
16. bit 18: Räuberlager forbidden and p owns a robber camp; +0xF3 bit 0x02.
17. Kirchliche Stiftung, Religionsfreiheit, Ablasswesen allowed: clear bits 19, 20, 21 respectively.
18. bit 22: Mätressentum forbidden and p owns a type-29 record (mistress); +0xF3 bit 0x01.
19. bit 23: Fliegerei forbidden and p owns a shipment (type 9) whose transporter byte +9 satisfies `t % 3 == 2` (Ballon 2 or Flugmaschine 5).
20. Weltliche Stiftung allowed: clear bit 25.
21. bit 26: Feste forbidden and p owns a type-28 record (feast scheduled).
22. bits 27, 28 (Edle Tuche, Silber): like step 13 with goods 11 and 12, law flags `0x85A3F`, `0x85A40`.
23. bit 29: p is Bischof (loc 10..12, index 12) and his county's tithe `0x85A38[loc]` exceeds `0x85A41`.

Notes and quirks:
- Bits 10 (Überfälle) and 30, 31 never come from this function; raids add evidence directly from the battle code (0x40ED3, see combat doc).
- Steps 13 and 22 read the sales of whoever had the last turn, not of p. At year end (2.3) the turn flags are not used either, so goods-trade offences are effectively evaluated against the last human's turn. [C, quirk]
- Person +0xF3 is a sticky offence-history byte (also bit 0x10 set by the debt code at 0x6449F). No reader was found yet. [G]

### 2.3 Adding evidence (0x44E00) [C]
`add_evidence(p, w)`, nothing when w = 0:
1. For every **active** spy contract (type 6, +9 != 0) whose target (+4) is p: evidence word (+0x0B) |= w.
2. If p's mission record (type 0, owner p) has kind 16 or 26 (byte +4): its word at +5 |= w (mission progress, missions doc).
3. If `popcount(w) > 3`: person +0xF3 |= 0x40.

Call sites:
- every human turn end (inside the phase-6 tail of `turn_end_settlement`, address 0x64108 onward): `add_evidence(player, offences(player))` with this turn's flags still set. Order there: sub-plot trigger, 25 % retaliation by the human rival (0x5B140 code 3), the player's pending duels, this evidence step, then the player's own trials (section 3).
- year end, as the last step of `year_end_world` (0x6A868): for every spy contract in the heap (active or not, any owner): `0x85CBA = 0`, then `add_evidence(target, offences(target))`. The flag word is cleared before each evaluation, so at year end only standing offences count. [C at instruction level]
- battle code (raids, 0x40ED3) and two retaliation helpers (0x5B111, 0x5C10F) [L].

Consequence: only persons watched by someone's active spy contract accumulate evidence; spying is the only way evidence exists (H p.18 "Evidence must come from spies" ✓).

## 3. Trials

### 3.1 Trial record, type 18 (6 bytes) [C]
+1 plaintiff, +3 counter, +4 defendant, +5 kind (0 = normal trial, 1 = inquisition trial).

Creation:
- Human, office tree mode 5 (privilege 12 "Prozess initiieren", any office holder): allowed when there is no kind-0 trial by the player on that target yet and the target's office level is not above the player's (principality targets need a principality player, county-or-higher targets a county-or-higher player). Record counter 1, kind 0. An existing one can be withdrawn (deleted).
- Human, office tree mode 7 (privilege 81 "Inquisitionsverhör", Inquisitor while `0x85A4C` = 0): no level restriction, counter 1, kind 1, sets turn flag 0x08 (offence "Kirchenprivilegien" if forbidden).
- Computer persons: AI doc (0x5B200): same level restriction; ensures an active spy contract on the target exists (creating one: years 10, cost 0); if that contract has evidence or with 10 %, creates the trial with counter `rand(4)` and kind = (initiator has privilege 81). [L]

When the trial runs:
- Human plaintiff: the turn-end record pass decrements the counter (1 → 0), and the phase-6 tail then runs every trial the player owns. So a human's trial is held at the end of the same turn in which it was filed. [C]
- Computer plaintiff: the year-end record pass (computer persons only, see 3.6) decrements the counter and runs `trial_main` when it reaches 0. A counter created as 0 wraps to 255 and the trial effectively never happens; it still blocks a new trial against the same target. [C, quirk]

`trial_main(r)` returns immediately when the counter is not 0. The record is deleted at the end of every held trial.

### 3.2 Court composition [C]
Let d = defendant, l = loc(d).
- l < 10 (city office or no office): judge = Richter of city l (office index 4). Assessors = Richter of city `N[l]` and of city `N[N[l]]`, `N = 0x84CEC = [1,2,3,0,5,6,4,8,9,7]`.
- 10 <= l <= 12: judge = Justizberater of county l. Assessors = Richter of the cities `N[3(l-10)+1]` and `N[N[3(l-10)+1]]`.
- l = 13: judge = Justizminister. Assessors = Justizberater of county `c = 10 + rand(3)` and of county `c2 = 10 + ((rand(2) + c) mod 3)`, where c is the county location id 10..12 itself, not 0..2 [C, instruction-level]. With k = c - 10 this is `c2 = 10 + ((k + 1 + rand(2)) mod 3)`, which is never c: rand(2) = 0 gives the next county, 1 the one after. Writing `(rand(2) + k) % 3` with k = 0..2 is wrong (it can return k).

Vacant seats: seat i that is empty takes the next filled seat after it; the seat after seat 2 is the **plaintiff**. So with an empty last seat (or all three empty) the plaintiff sits on the bench. [C, quirk]

### 3.3 Investments (option "Gericht bestechen", `0x86AD2` bit 1) [C]
Per side, plaintiff first, then defendant; the defendant's turn is skipped for kind 1. Arrays per side s: W[s] (witness money), B[s] (bribe money), all 0 if the option is off.
- Computer side, with `n0 = popcount(evidence of the plaintiff's contract on d)`:
  - plaintiff: `W = 1000 + 500*rand(10) + rand(1000) - 100*n0`,
  - defendant: `W = rand(1000) + 1000*n0`,
  - then, by the **side's** own office level: city (loc < 10) `W = W / 3`, county `W = W*2/3`; clamp 0..9999. B stays 0.
- Human side: enters W and B. Both are paid now.
- `K[s] = clamp(W[s] / 1500, 1, 6)` = number of witnesses called (also 1 when the option is off).

If the plaintiff has no spy contract on d, `n0` reads garbage (null record) [C, bug; the original reads memory at address 0x0B]; in practice humans always file with a contract because rules-04 requires evidence.

### 3.4 Evidence and plea [C]
- `E` = evidence word of the plaintiff's contract on d. If there is no contract or `E = 0`: `E = 0x84B4C[rand(30)]` (a single random bit 0..29) and the case is "without contract".
- `n = popcount(E)` (all set bits; E never has bits 30, 31).
- Each nonzero 10-bit group of E (finance, criminal, church) rolls `rand(5)` and `rand(3)` for text. [RNG only]
- Plea `q` 0..8: computer defendant `5 + rand(4)` without contract, else `rand(9)`; human defendant chooses from the list ("Ich gestehe Alles!" = 0 ... "Lüge, alles Lüge!" = 8), default 7.
- Option "Zeugen & Plädoyers" (`0x86AD1` bit 6) only shows witnesses and speeches; it rolls extra `rand` calls for texts and animation frames.

### 3.5 Bribes, scores, votes, sentence [C unless marked]
Second money step, only with "Gericht bestechen": for plaintiff then defendant (both sides, also for kind 1):
- computer plaintiff: `X = (n < 6) ? K[0]*1300 + rand(1000) : 0`;
- computer defendant: `X = without contract ? 0 : K[0]*1300 + rand(1000)` (uses the plaintiff's K, quirk);
- human: `X = B[s]`.
The side pays X (a human therefore pays B twice: once at entry and again here [L, reads as a bug]); each of the three seated judges receives `X / 3`; `W[s] += X`.

Scores (both random terms are always rolled):
```
P = W[0] + (1500 + rand(500)) * n
D = W[1] + (without contract ? 7000 + rand(3000) : 0) + 2000 - 200*q
```

Votes, per seat j = 0..2 (a person sitting twice keeps the first vote):
1. For every person x with `favour(judge_j → x) > 100` (x blackmails the judge) a dialog asks x for the verdict; count answers "guilty" (a) and "innocent" (b).
2. If a != b: vote = (a < b) ? innocent : guilty.
3. Else if the judge is computer: the judge is the defendant → innocent; is the plaintiff → guilty; otherwise innocent iff `favour(j→plaintiff)*100 + P < D + favour(j→defendant)*100`.
4. Else (human judge, no blackmail majority): the human decides.
5. `rand(4)` per seat for text (computer judges).

Guilty iff fewer than 2 innocent votes.

Fine base: `F = net_worth(d) * n * (0x85A4D + 150) / 4000` (unsigned division).

Sentence (guilty only), chosen by seat 0:
- computer: `s = rand(0x85A4D / 21 + 1) + n - 1`; if d has no title (+0x0D = 0) and s = 3 then s = 2; `s = min(s, 6)`.
- human: picks from the list below, starting at 2; "Entzug des Titels" cannot be picked for a defendant without title.

| s | sentence | effect on d |
|---|---|---|
| 0 | Pranger | life (+0x02) = life*9/10 + 1 |
| 1 | Geldstrafe | pay F |
| 2 | Mehrjährige Geldstrafe | type-19 record, counter 3, +4 = F*2/3 per year (3.6) |
| 3 | Entzug des Titels | title (+0x0D) - 1 |
| 4 | Ein Jahr Schuldturm | type-20 record, counter 1 |
| 5 | Entzug des Amtes | office index (+0x11) = 15, location (+0x0F) = 0; the seat is not refilled here |
| 6 | Ein Jahr Kerkerhaft | type-21 record, counter 1 |

Common tail (both verdicts):
1. Payer = d if guilty, else the plaintiff. If guilty and the plaintiff's mission has kind 7, its counter +5 increases by 1.
2. Court costs `C = max(F/5, 100) + rand(100)`. The payer pays C; each seat receives `3*C/10` (90 % paid out, a seat held twice is paid twice).
3. If the payer has insurance bit 1 (person +0x07 & 2): C is refunded to the payer (the 90 % to the judges stays paid).
4. For every spy contract on d (any owner): evidence &= ~E; a contract whose evidence becomes 0 is deleted if it is inactive or belongs to the plaintiff.
5. With `rand(100) < 80`: d retaliates against the plaintiff (0x5B140 code 3, AI doc).
6. Delete the trial record.

Handbook check: judges and assessors as in H p.54 ✓ (city judge / justice adviser / justice minister); H p.54 says serious cases use county justice advisers as assessors, the code uses them only for principality defendants ✗. Penalties match the H p.34 penalty list for Schuldturm (one round) ✓.

### 3.6 Penalty records [C]
- Type 19 (yearly fine): processed in the owner's turn-end record pass (0x2C0A0): counter decremented first; at 0 deleted, otherwise the owner pays +4. With counter 3 that is two payments (counter 2 and 1), i.e. `4/3 F` in total for a human. Computer-owned type-19 records are only aged and deleted by the year-end pass and never charge. [C for the code path; the "never charge" consequence is L]
- Type 20 (Schuldturm), 21 (Kerker): at the start of the owner's turn (0x4AB90 / 0x4B2D4): counter 0 → delete the record and play normally; otherwise counter - 1, show the message (0x2D750 code 0) and skip the whole turn (phase set to 6, no settlement). With counter 1 the next turn is lost. Persons with type 20 or 21 are excluded from auctions and from some year-end office steps. [C]

Year-end record pass (`year_end_world` at 0x69E20) runs only over computer persons (`person_find_first(…, controller 0)`); human records are aged in their own turn end. Per record: counter - 1, then by type: 18 trial, 22 duel, 25 siege, 26 relief, 29 mistress event (0x20B90), 50 AI follow-up (0x5B200); types 18, 22, 25, 26, 29, 50 are not deleted here; every other type is deleted at counter 0. [C]

## 4. Privileges (0x5E600, menu 0x5F080)

### 4.1 Check [C]
`privilege(p, viewer, code, show)` returns 0 (no), 1 (yes) or 2 (yes, adjustable setting). Codes 0..86. "self" below means the entry only counts when viewer = p, which is how every rule check calls it (`privilege(p, p, code)`). Name text id = `0x86778[code]`, description = name id + 1.

| code | name | condition |
|---|---|---|
| 0, 2, 3, 5, 7 | Kirche eintreten, Ihr seid…, Werben, Ablass kaufen, Kirche austreten | never (dead entries) |
| 1 | Medicus konsultieren | self |
| 4 | Mätresse/Liebhaber nehmen | self, p has a spouse (+0x04 != 0xFF) |
| 6 | Testament machen | self, p has at least one child record (type 31, counter < 45) |
| 8, 9, 12, 27 | Fest geben, Bauwerk stiften, Prozess initiieren, Fechtunterricht | p holds any office |
| 10 | Einkommen | self, g > 2, not a church office |
| 11 | Kein Kirchenzehnt | self, church office, g >= 9 |
| 13 | Handelszertifikat (display) | self |
| 14, 15 | Lohnniveau, Einkommensniveau (setting) | g = 5 Kämmerer |
| 16, 17 | Baupreise, Lagerraumpreise (setting) | g = 3 Baumeister |
| 18 | Stimmrecht vor Gericht | g = 4 Richter |
| 19, 20 | Strafmaß vorschlagen, 30 % Prozesskosten | self, g = 4 |
| 21 | 10 % Umsätze der Stadt | self, g = 6 Bürgermeister |
| 22 | Umsatzsteuer (setting 5..33) | g = 6 |
| 23 | Heirat unterbinden | g = 9 Domherr |
| 24 | 10 % Kirchenzehnt der Stadt | self, g = 9 |
| 25 | Versicherung abschließen | loc > 9 |
| 26 | Beleidigung aussprechen | loc > 9, not a church office |
| 28, 29 | Stadt belagern, Stadt entsetzen | g > 12, g not 18 or 19 |
| 30 | 10 % Umsätze der Grafschaft | self, g = 15 Finanzberater |
| 31 | Maximaler Kreditzins (setting 0..60) | g = 15 and loans not forbidden |
| 32 | Anteil an Zolleinnahmen (setting 0..33) | g = 15 |
| 33 | Stimmrecht | g = 14 Justizberater |
| 34, 35 | Strafmaß, 30 % Prozesskosten | self, g = 14 |
| 36, 37, 38 | Billiges Militär (25 %), Effiziente Manöver, Belagerung & Entsetzung | self, g = 13 Hauptmann |
| 39 | 15 % Umsatzsteuer der Grafschaft | self, g = 16 Kanzler |
| 40 | Berater vertauschen | g = 16 |
| 41, 42 | 30 % Zoll, 20 % Umsatzsteuer der Grafschaft | self, g = 17 Vogt |
| 43 | Bevorzugung (Abt) | self, g = 19 Sakristan |
| 44 | Weinproduktion optimal | self, g = 18 Kellermeister |
| 45 | 10 % Kirchenzehnt der Grafschaft | self, g = 21 Abt |
| 46, 47, 49, 50 | Günstige Spione 30 %, Günstige Sabotage 30 %, 10 % Ablässe, Bevorzugung (Bischof) | self, g = 20 Diakon |
| 48 | Arbeiter verschwinden lassen | g = 20 |
| 51 | Kirchenzehnt (setting 0..25) | g = 22 Bischof |
| 52 | 20 % Kirchenzehnt der Grafschaft | self, g = 22 |
| 53, 54, 55 | Grafschaft segnen, bekehren, Wunder wirken | g = 22 |
| 56, 58 | Finanzgesetze ändern, Projekte unterstützen | g = 28 Finanzminister |
| 57 | Befreiungen (no taxes, no interest) | self, g = 28 |
| 59, 62 | Strafgesetze ändern, Stimmrecht | g = 27 Justizminister |
| 60, 63 | 30 % Prozesskosten, Strafmaß | self, g = 27 |
| 61 | Rechtsprechung (setting) | g = 27 |
| 64 | Inquisition erlaubt/untersagt (setting) | g = 26 Marschall |
| 65, 66, 67 | Billiges Militär (50 %), Effiziente Manöver, Belagerung & Entsetzung | self, g = 26 |
| 68 | 20 % Umsatzsteuer des Fürstentums | self, g = 29 Großkanzler |
| 69 | Minister vertauschen | g = 29 |
| 70 | 40 % Umsatzsteuer des Fürstentums | self, g = 30 Regent |
| 71 | Staatsreligion (setting) | g = 30 |
| 72, 73, 75, 76 | Günstige Spione 40 %, Sabotage 40 %, 20 % Ablässe, Bevorzugung (Erzbischof) | self, g = 31 Erzdiakon |
| 74 | Arbeiter verschwinden lassen | g = 31 |
| 77, 78, 79, 80 | Günstige Spione 60 %, Sabotage 60 %, Arbeiter verschwinden lassen, Zwietracht säen | g = 32 Inquisitor |
| 81 | Inquisitionsverhör | g = 32 and `0x85A4C` = 0 |
| 82, 84, 85, 86 | Kirchengesetze ändern, Fürstentum bekehren, Heiligsprechen, Wunder wirken | g = 33 Erzbischof |
| 83 | 20 % Kirchenzehnt des Fürstentums | self, g = 33 |

The menu lists the player's privileges, twelve per page, and can switch to the privileges of every person whose favour toward the player is above 100 (blackmailed persons, career doc 6). Actions then use that person's office and location, while the player pays. Costs that use `standing(x, k)` (rules-04 1) take x = the privilege holder, except code 86 (`standing(player, 10)`).

### 4.2 Effects of privileges used by rule code [C unless marked]
- **Spy/sabotage discount** (office tree prices, rules-04 3.2): multiplier default **10**, privilege 47 → 7, 73 → 6, 78 → 5 (checked in that order). This corrects rules-04, which gives the default as 7. The "Günstige Spione" codes 46, 72, 77 are never checked by rule code: only the sabotage codes discount, and the discount applies to every price in the office tree (spies and saboteurs). So the effective discounts are 30, 40 and 50 % (texts say 30/40/60). [C, instruction level]
- **Office income at year end** (`year_end_world` 0x69FBC..0x6A4C4), humans only, after the year-end record pass. Treasuries: `T[c]` = city sales-tax treasury `0x85E25`, `K[c]` = tithe treasury `0x85E4D`, `A` = indulgence total `0x85E75`. "county" = cities c with `0x868D4[c]` = loc(p), "all" = 10 cities. First matching code wins in each group:
  - turnover group: 21 → `T[loc]/10`; 39 → county `T*15/100`; 41 → county `T*20/100`; 30 → county `T*10/100`; 68 → all `T*20/100`; 70 → all `T*40/100` (division per city);
  - tithe group: 24 → `K[loc]/10`; 45 → county `K/10`; 52 → county `K*20/100`; 83 → all `K*20/100`;
  - indulgence group: 49 → `A/30`; 75 → `A/5`.
  Each positive amount is shown and added to cash. Afterwards: `T[c] = K[c] = city +0x0E`, and `A = sum_c (city +0x0E) / 2`. So every year starts each treasury at that city field (population per the map doc) before taxes accrue. [C; meaning of the base L]
- **Recruiting** (turn end, type-15 record): money M; privilege 36 → `M*4/3`, else 65 → `M*2`; then men = `M / (150 + rand(200))` (castle doc).
- 57: no loan interest at turn end (0x62D6C), no sales tax and no tithe (0x61912). 11: no tithe (0x619B8).
- 44: `branch_effective_skill` returns 150 for wine (good 7) when the branch owner has privilege 44 (also checked in the reminders 0x17C79).
- 37/66 (manoeuvres, 0x56BB3) and 38/67 (siege 0x1F2A0, relief 0x1FA80): castle doc.

### 4.3 Privilege actions (menu 0x5F080) [L unless marked]
- 1 Medicus: shows health text `11 - min(life/2, 11)`.
- 9 Bauwerk stiften: choose a city and one of five buildings; price `rand(100) + [100000, 50000, 25000, 10000, 5000][k]` (`0x872EC`); k = 0, 1 are church buildings (turn flag 0x80000), 2..4 secular (0x2000000). Pay; mission kinds 18/28 add the price to +5; `0x43F30(city, (5-k)*3)` (city effect, not decoded); every person of that city: `favour_increase(player, x, rand((5-k)*2 + 1))`.
- 14..17: city level byte (+0x7F wage, +0x80 income, +0x81 building price, +0x82 storage price) += ±21, clamped 0..100 (five names by value/21).
- 22: city tax +0x83 += delta (editor range 5..33). 31: `0x85A45+k` += delta. 32: `0x85A48+k` += delta. 51: county tithe += delta, sets turn flag 0x08 when changed. 61: `0x85A4D` ±21 clamped 0..100. 64: `0x85A4C ^= 1`. 71: `0x85A4B = (0x85A4B + delta) mod 3`, sets turn flag 0x100000.
- 25 Versicherung: four toggles flipping person +0x07 bits 1, 2, 4, 8 (`0x87300`); text "each insurance costs 1 % of your net worth per year". Value 2 covers court costs (3.5), 4 raids (combat doc 5.6), 8 sabotage (4b); value 1 is not decoded.
- 27 Fechtunterricht: cost `2*f*f + 302 + rand(100)`, f = person +0xF6 (fencing skill, start rand(10)); if accepted and f < 95: f += 1 + rand(5). [C]
- 28 Stadt belagern: refused when there are 10 sieges (type 25) or the player already has 3; choose an unbesieged city and army size N (max 50000); cost `N*10`; type-25 record counter 4, +4 city, +5 u16 N. 29 Entsetzen: choose a city besieged by someone else; cost `N*10`; type-26 record counter 4. (Resolution in the castle doc.)
- 40/69 Berater/Minister vertauschen: the holders of indices 3, 4, 5 at p's location move to `(i+1) mod 3 + 3` (3→4, 4→5, 5→3). [C]
- 48/74/79 Arbeiter verschwinden lassen: once per turn (`0x85E79`); cost `standing(holder, 20)`; choose a person; every production run (type 8) of that person keeps 80 % (48), 70 % (74) or 60 % (79) of its workers (byte +5). Turn flag 0x08.
- 53 Grafschaft segnen: cost `standing(holder, 30)`; type-23 record for p's county (reused if one exists), counter 5. Turn flag 0x08.
- 54/84 bekehren: cost `standing(holder, 30)`; type-24 record for the location, counter 5, +5 chosen religion. Turn flag 0x08.
- 55/86 Wunder wirken: cost `standing(holder, 20)` / `standing(player, 10)`; runs 0x59940 (the miracle event; the map calls it computer misfortunes). Turn flag 0x08 via 0x59940.
- 58 Projekte unterstützen: enter X, choose a person r != player; the player pays `X/5`, r receives X (quirk: four fifths are created); every other person x at r's location: `favour_decrease(player, x, rand(5))` (x likes the player less). [C]
- 80 Zwietracht säen: cost `standing(holder, 30)`, office tree mode 9 (rules-04 3.6). 81 Inquisitionsverhör: cost `standing(holder, 30)`, office tree mode 7 (3.1).
- 85 Heiligsprechen: cost `standing(holder, 35)`; choose r; if r != player every clergy person x: favour toward r rises by `rand(15)`. Turn flag 0x08.
- 4 mistress (0x20A00), 6 testament, 8 feast (feast doc), 12 trial (office tree mode 5), 26 insult (office tree mode 6 → duel), 56/59/82 law books.

## 4b. Sabotage execution (year end, after the auction settlement) [C]
`flag = rand(100) < 90` is rolled once (90 %: no film this year; the film plays at most once). For every saboteur contract (type 7, owner s, target t at +4), with `rand(100) < 25`, if s or t is human:
1. `x` = uniformly random property of t of any kind and location (0x42B90 with kind = 0xFF, location = 0xFF: collect all of t's properties in slot order, pick `list[rand(n)]`; one RNG call; no call and x = none if t has no property). (Corrected: not the first property.)
2. Branch (kind 0): if it has a residence, residence byte +0x04 `*= (40 + rand(40)) / 100`; both storage areas (+0x23, +0x25) `*= (60 + rand(30)) / 100`; each transporter count `*= (60 + rand(30)) / 100`. Damage D = value before − value after, at least 2. Messages depend on whether s is t's human rival.
3. Castle or camp: +0x04 and +0x03 each `*= (70 + rand(26)) / 100`, one random troop slot count `*= (70 + rand(26)) / 100`; D = value loss.
4. Other kinds (no usable property): t pays `(standing(s, 10) + standing(t, 10) + 100) * 2` instead. If t has no property at all the code dereferences a null record [bug].
5. Insurance bit 3 (+0x07 & 8): t is refunded D.
6. With `rand(100) < 70`: t retaliates against s (0x5B140 code `2 + rand(4)`).
7. The contract is deleted (each contract strikes at most once). Contracts that do not strike stay until their 5 turns run out.

## 5. Game options relevant here [C]
Options byte `0x86AD1`: bit 0 Geschichtsschreibung, 1 Abenteuer, 2 Schicksal, 3 Spieltipps, 4 Warnungen, 5 Runde bestätigen, 6 Zeugen & Plädoyers. `0x86AD2`: bit 0 Spieler ankündigen, bit 1 Gericht bestechen, bit 2 Spielende um 1800 (options screen 0x4EFC0, text `0xBE260`). Note for the economy doc: "option bit 4" (sub-plots) is `0x86AD1` bit 1 and "option bit 8" (tips) is bit 3 under this numbering.

## Open questions
1. Editor step for the cash cap `0x85A28`.
2. Jury court for debts (0x6449F region) and its use of `offences`.
3. Who reads person +0xF3.
4. Insurance bit 0 (bit 1 legal costs, bit 2 raids, bit 3 sabotage are decoded) and the yearly premium.
5. Turn flag 0x20000 (escort) set site in 0x61A70: exact condition.

## Indulgence (Ablass) dialog

Source: `0x59720(p)`, called from the Kirchgang menu `0x5D1F0` (entry 4, "Ablass kaufen", at `0x5D448`) and from `npc_act` case 0 (`0x5A297`, rules-09). Helpers: `rec_find`/`rec_next` = `0x426B0`/`0x427B0`, `popcount` = `0x440E0`, `net_worth` = `0x44640`, `clamp0` = `0x441F0`, `cash_pay` = `0x43860` (no affordability check), `rand` = `0x36FF0`, `box` = `0x368E0`. Read at instruction level [C] unless marked.

1. `E` = OR of evidence (+0x0B, u32) over every spy contract (type 6, any owner) whose target (+4) is p, active or not. No RNG.
2. `price = clamp0(net_worth(p)) * popcount(E) / 100 + 7` (32-bit unsigned multiply and divide). No RNG.
3. Human p (person +0x10 & 0xF0): **one** draw `c = rand(10)`, after the price. `%s` = location name `T[0xB6194][c]`, so any of the 10 cities, Fürstenbrück included, uniformly. The city is unrelated to p, p's office or the city p stands in. This is why the live dialog names a different city each visit.
   `box(140, 120, 360, 240, 0xACE40, price, city)`: `[EXE text]`. Style 0x8810 (rules-15 §0.4). Return 0 = buy, anything else = nothing happens (no flag, no draw). Which item index each `%a` line and the OK glyph map to was not read; live, "Den Ablass kaufen" buys [UNVERIFIED for the OK glyph]. FIDELITY measured the window at (143,123) width 353 with height = text + 35 centred on y = 240, which does not match the fixed 240 passed here [UNVERIFIED: box height rule].
   Computer p: no box, no city draw, treated as "buy".
4. On buy, in order:
   - `cash_pay(p, price)`; cash may go negative.
   - The first type-0 record of p (mission): if its kind (+4) is 8, `+5 += price` (rules-10). Only that one record is checked.
   - If p holds an office (person +0x11 != 15): indulgence total `0x85E75 += price` (year-end "aus Ablassverkäufen" text, rules-32 row 17).
   - Every spy contract targeting p: active (+9 != 0) → evidence := 0; inactive → record deleted. No record is created.
   - Human: sample 0x4A loaded and played (mode 3), `box(160, 160, 320, 160, 0xACF03)` `[EXE text]`, then the sample stops. Computer: `print(0xACE21, p)` `$L%1N5, kauft einen Ablass.$Z` into the current window (turn report).
   - Turn flag `0x85CBA |= 0x200000` (byte `0x85CBC |= 0x20`) for human and computer alike.
5. No law or privilege check inside `0x59720` and none at the Kirchgang menu entry: the human can always buy. The Ablasswesen law `0x85A39` acts only afterwards: `offences()` clears bit 21 when the law allows indulgences (§2.2 step 17). The computer path in `npc_act` sets its local 0x200000 only when `0x85A39 != 0`, which suggests nonzero = forbidden [UNVERIFIED: polarity of step 17 not re-read].
6. Opening the screens draws no RNG: `0x5CCA0` has no `rand` call, and `0x5D1F0` reaches `rand` only inside the candle and alms entries. Each visit's only draw is the `rand(10)` above, made when the human opens the indulgence box (also on "Nichts dergleichen tun").
7. Computer trigger (rules-09 case 0): `rand(100) >= 40` and privilege 5 (`0x5E600(p, p, 5, 0)`), then `0x59720(p)`.
