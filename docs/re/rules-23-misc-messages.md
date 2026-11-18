# Miscellaneous messages: arrivals, shipment mishaps, debt seizure, castle sales, relief, Medicus, Kontor confirmations

Confidence tags as in `rules-00-map.md`: **[C]** read at instruction level (call arguments checked), **[L]** decompiler/sweep, **[G]** guess. Text addresses are linear CP437 strings; `%` codes in `rules-12` §1; `print` arguments in consumption order. "Text id" = index into the pointer table `0xB4258`. Windows: turn-end report / year-end report as in `rules-12` §2; `box(x, y, w, h, text, args…)` = message box `0x368E0` (parchment, style 0x8810, `rules-16b`); `speech(c0 c1 c2 c3 : n)` = `0x15100` + `0x154B0` (`rules-15` §0.2). Everything below is [C] unless marked.

Sources: `turn_start_reports` 0x4B900, `shipment_mishaps` 0x174C0 and its call site 0x63926, turn-end tail 0x64108..0x65620, year-end castle step 0x69200..0x694D4, castle info 0x549C0, relief 0x1FA80, privileges menu 0x5F080 (jump table 0x5F4A8), city info 0x4F3C0, residence screens 0x52240..0x53200, transporter screen 0x53A60.

## 1. Arrivals at the player's branch (turn start, 0x4B900) [C]

The whole report is skipped while `year <= 1600` (`cmp [0xEF4B8], 0x640; jbe`): nothing is shown at the first turn start.

Screen: with option bit `0x86AD2 & 1` first a full-screen box (style 0x8910, picture 0xBB) `0xBD9AC` `$8H$Z$C$gNächster Spieler:$Z%1N1, %1N4$N` with (p, p), `wait_click`. Then full-screen window `win_open(0, 0, 640, 480, 0x8100)`, picture 0xD4 at (0, 40), `banner(p, "Erträge" 0xBD9D8)`. Only the owner sees it (it is his turn start).

Record passes, in this order, before any text:

| records | effect | text |
|---|---|---|
| type 9 (shipments of p, heap order, 0x4BB53) | arrival rules of `rules-02` §4.4: sale revenue summed into a local total; deliveries/returns via `branch_add_goods` 0x439F0 and `branch_add_transporters` 0x43930; record deleted | **none per record**. No text for goods arriving, for transporters coming home, for a return because the city is blocked, or for a delivery that falls back to the source branch |
| type 51/53/54 (0x4BE34 loop) | troop returns (`rules-07`) | none |
| production (type 8) | `rules-02` §3.3 | the "Produktion" list |
| type 10 (built transporters, 0x4C814) | counter 0: `branch_add_transporters(branch, type +4, count +5, cond 100)`, deleted | **none** |
| type 11 (repaired, 0x4C8B5) | counter 0: `rand(30)` only when a type-39 record of another owner exists (`rules-02` §6); `branch_add_transporters(…, 100 − that)`, deleted | **none** |
| type 13 (residence, 0x4C96E) | counter 0 or bit 7: `set_residence` 0x520A0, deleted | **none** |

Texts, all in the "Erträge" window after the production list:
1. `0xBD9E0` `$t$|$4H$10>$56<$Z$C^ Produktion _$Z`; no production: `0xBDA04` `[EXE text]`; otherwise one `0xBDA40` `$L       %s:$E$R%d$Z` per good (name, amount), harvest quality, `0xBDA68` "Weiteres" for goods beyond the list (`rules-02` §3.3).
2. `0xBDA84` `$Z$C^ Exporte _$Z`; then **either** `0xBDA98` `[EXE text]` (total 0) **or** `0xBDAC4` `[EXE text]` with (**total sale revenue of all type-9 sell arrivals this turn start**); `cash_add(p, total)` follows the print.
3. If the overflow counter `0x85CC1` ≠ 0: `0xBDAF0` `[EXE text]` with (counter). Quirk: a counter of exactly 1 is incremented to 2 before printing, so the text never says "1 Waren".
4. `win_close`; `wait_click`.

So the only arrival message is the export total; the type-9 loop itself and the three transporter/residence passes print nothing. The Kontor shows the new stock and transporters without comment.

## 2. Shipment mishaps in transit (`shipment_mishaps` 0x174C0) [C]

Called from the human's turn-end record pass (type-9 case at 0x63926 of `turn_end_settlement` 0x61A70, one call per type-9 record of the player), so every text lands in the player's **turn-end report** and only the shipper sees it. Computer persons have no shipments (`rules-02`), so the handler never prints at year end. Record fields: `+4` kind, `+5` u16 qty, `+8` count, `+9` type, `+0xA` destination, `+0xB` condition.

| step (`rules-02` §4.3) | text |
|---|---|
| 1 condition wear | none |
| 2 cargo loss (`qty = qty·(70 + rand(30))/100`) | **none** — the player only sees less arriving |
| 3 one transporter lost (`count > 1`) | `print("$B")` then `T[0xB6E54][type]` with (**destination name** `0xB6194[dest]`, `%s`), `count −= 1`. The six texts, ids 2815..2820: Karren `0xA872F` `[EXE text]`; Schiff `0xA8771` `[EXE text]`; Ballon `0xA87D3` `[EXE text]`; Dampfkarren `0xA8814` `[EXE text]`; Tauchboot `0xA8858` `[EXE text]`; Flugmaschine `0xA88C7` `[EXE text]` |
| 4 off course (`rand(100) < 5`, kind 2/3, `count > 1`; new = `(dest + 1 + rand(9)) mod 10`) | Ballon (type 2): `0xBA380` `[EXE text]` with (**old dest id**, **new dest id**); Dampfkarren (type 3): `0xBA3D0` `[EXE text]` with (old, new). Other types: the draws happen, no text, destination unchanged |

No sound, no picture, no `$V`. The fate texts 3324..3331 of `rules-12` §3.9 (storm, winds, Fuhrleute, escort, …) are **not** used here; they belong to the fate handler 0x226F0 (`rules-06` §4 kinds 61..68) and hit a random shipment there. Robber attacks on shipments are printed by the camp owner's `settlement_pre` (`rules-07` §4).

## 3. Debt seizure (turn-end tail, 0x643CC..0x65620) [C]

Rules in `rules-02` §10. Runs only in a **human's** own turn end; all texts go into his turn-end report, which is still open from the record pass (no new window until the jury court). Nothing is printed for other players; the year end has no counterpart.

1. Header, printed as soon as the trigger holds (no `$V` before it): `0xC10D0` `[EXE text]` with (p).
2. Stages 1..3 (goods, transporters, storage) print **nothing per branch**. After each stage's branch loop, if its counter n > 0, one line followed by `wait_click`: `0xC14B0` `-$E$6>Insgesamt %d Waren zum halben Preis.$|` (n); `0xC14E0` `-$E$6>%d Transportmittel zum halben Preis.$|` (n); `0xC1514` `-$E$6>%d m$18c Lagerraum zum halben Preis.$|` (n; `$18c` = the ² glyph).
3. Stage 4, per branch, each followed by `wait_click`: construction site `0xC1148` `[EXE text]` with (city); existing residence `0xC119C` `[EXE text]` with (**residence word**, city).
4. Stage 5, branch seized: `0xC11C8` `-$E$6>Eure Niederlassung in %o.` (city); if shipments were redirected `0xC11E8` `[EXE text]` (city of the receiving branch); then `0xC1218` `$|`; `wait_click`; `prop_free`.
5. Jury court: scene in `rules-15` §6.1 (picture `0x193 + county`, `win_open(10, 280, 620, 200, 0x8910)`, `0xC121C` with (p, county, 0x14, &bribe), `0xC12F4`, verdict `0xC1320` / `0xC13A8` with (p) on picture 0xB9 with `sound(0x0B)` and speech 0x3B5 for the "letzte Niederlassung" variant; acquittal `0xC1470` with (p) on picture 0xB8 with `sound(0x1E)` and speech 0x3B6, via caption box `0x36C90(10, 220, 620, 100, 0, …)`). Phase := 6 afterwards.
6. No branch left after the loop (`property_find_first(0, any, p)` = 0 at 0x654A9): bankrupt scene `rules-15` §6.2 (picture 0xBA, `sound(0x0B)`, boxes `0xC1544` `[EXE text]` with (p) and `0xC15A4` `[EXE text]`, speech 0x3B7/0x3B8/0x3B9, each with `rand(4)` for the speaker sample 0x4E + r) [L for the sample use].

Note on `rules-02` §10.2: after a stage-4 seizure the code jumps to the loop tail at 0x653FA (next branch), as the stage text implies; the item lines are the only per-branch output.

## 4. Castle sales [C]

Three paths. None of them lets a computer person buy the player's castle on its own: the year-end purchase step (`rules-07` §3 step 3, 0x69200) only visits **computer-owned** properties, and the AI's type-17 interest records on the player's castle (`rules-09` §4 case 1) are created silently and only show up in the castle info screen.

### 4.1 Year end, computer owner (0x69200..0x694D4), printed in the year-end report
- Human candidate (`rand(100) < count·80/6`): `0xC1D4C` `[EXE text]` with (**candidate**, **owner**, **location**, **price**); `menu_choice`: 0 = buys (`cash_pay(candidate, price)`, `cash_add(owner, price)`, owner byte := candidate, history record code 9 via `0x2D750(candidate, 9, location, 0)`, all type-17 records on the property deleted); otherwise nothing more. Everybody watching the year end sees the question; only the candidate's choice counts.
- Computer candidate: `0xC1DD8` `[EXE text]` with (**location**, **price**, **candidate**); no payment by the buyer (`rules-07` quirk), owner paid, ownership moves, interest records deleted.
- No candidate or roll failed: nothing.

### 4.2 Castle info screen, human owner (0x549C0, "Verkaufen")
- Nobody interested: `box(140, 180, 360, 120, 0xBF420 $Z$CDerzeit interessiert sich niemand für %s.$N, castle name)`.
- Otherwise `win_open(140, 140, 360, 200, 0x8810)`, `0xBF454` `[EXE text]` with (castle name, **value** = `net_worth_goods(property)`), one `0xBF498` `%a%4N1$N` per interested person (up to six), `0xBF4A4` `%aNiemandem$N`; `menu`.
- Chosen buyer **human**: `box(180, 160, 280, 160, 0xBF4C0 $Z$C%1N0, schlagt Ihr ein?$Z%m %m$N, buyer, 0x1030, "Ja", 0x1030, "Nein")`. Return 0 ("Ja") sells; **any other answer cancels the sale** (`test ax, ax; jne → choice := none`). Correction to `rules-08` §1.5, which says the refusal is ignored.
- Chosen buyer **computer**: no dialog, no text; the sale happens at once.
- Sale: `cash_pay(buyer, value)`, `cash_add(owner, value)`, owner byte := buyer, history code 9 for a human buyer, all interest records deleted. No message to the seller, no sound.
- Auction route: "Zollburg versteigern" window (`0xBF8E0`, `0xBF900` no auction, `0xBF934` `[EXE text]`, `0xBF9C0` `[EXE text]`); the sale itself is the auction of `rules-08`.

## 5. Relief army, city not besieged (0x1FA80) [C]

Confirmed: text pointer `[0xB6928]` = `0xA3D7D` (id 2484) `[EXE text]` with (**city** `+4`). Printed only in the **human** branch (`test [person+0x10], 0xF0`), into the owner's turn-end report; the record is deleted; no RNG, no sound. Computer owner: record deleted, nothing printed (year-end report untouched). As in `rules-21` §3.

## 6. Privileges menu, "Medicus konsultieren" (code 1, handler 0x5F9C6 via jump table 0x5F4A8[1]) [C]

Menu entry text `0xA1DEF` `%m Medicus konsultieren$N` (id 2333). On selection, for the person x in the menu's person variable (`[ebp-0x34]`: the player, or the blackmailed person switched to in the menu [L for the switch case]):
```
L = life(x) / 2          life = person +0x02, 0..255, unsigned
if L > 11: L = 11
k = 11 − L               0..11
```
`box(120, 160, 400, 160, 0xA1E09 (id 2334) $Z$CIhr begebt Euch zu einem Arzt. Er untersucht Euch und meint:$Z%s$N, T[0xB66D4][k])`. The table is text ids 2335 + k:

| k | life | text |
|---|---|---|
| 0 | ≥ 22 | `0xA1E50` `[EXE text]` |
| 1 | 20..21 | `0xA1E72` `[EXE text]` |
| 2 | 18..19 | `0xA1E9D` `[EXE text]` |
| 3 | 16..17 | `0xA1EB9` `>Es geht Euch gut.<` |
| 4 | 14..15 | `0xA1ECD` `[EXE text]` |
| 5 | 12..13 | `0xA1EF1` `[EXE text]` |
| 6 | 10..11 | `0xA1F18` `[EXE text]` |
| 7 | 8..9 | `0xA1F42` `[EXE text]` |
| 8 | 6..7 | `0xA1F5C` `[EXE text]` |
| 9 | 4..5 | `0xA1F8D` `>Es sieht nicht gut aus.<` |
| 10 | 2..3 | `0xA1FA7` `[EXE text]` |
| 11 | 0..1 | `0xA1FD2` `>Macht Euer Testament!<` |

No cost, no RNG, no sound, no state change. Only text 0 carries its own `$N`; the others end inside the `%s` of 2334. (Neighbouring handlers for reference: code 10 "Einkommen" `box(160, 160, 320, 160, 0xAFE7F $N$CAls %1N4 bezieht Ihr ein Einkommen von jährlich %d Talern.$N, x, amount)`; code 11 `box(160, 160, 320, 160, 0xAFED9 …vom Kirchenzehnt befreit…, x)`.)

## 7. Kontor confirmations [C]

### 7.1 Branch (city info window 0x4F3C0, `rules-11` §1)
| action | message |
|---|---|
| buy, `cash < price` (unsigned) | `box(140, 160, 360, 160, 0xBE500 $N$CFür eine Eröffnung fehlt Euch leider das nötige Bargeld.$N)`; the new record is freed |
| buy, paid | `speech(0 2 2 7 : 0x384 + city)` (one line per city), then `box(140, 160, 360, 160, 0xBE548 $N$CHerzlichen Glückwunsch!$ZIhr habt ein Lager in %s eröffnet.$N, city name string)` |
| sell | `box(140, 160, 360, 160, 0xBE58C $C$ZWollt Ihr wirklich Eure Niederlassung in %s schließen?$Z%m  %m$N, city name, 0x1030, "Ja", 0x1030, "Nein")`; 0 = yes → `cash_add(p, value·2/3)`, `prop_free`; no further text |
| sell refused (only branch) | menu line `0xBE4D4` `[EXE text]` inside the info window; no box |
| other info-window lines | `0xBE45C` `[EXE text]`, `0xBE480` `[EXE text]`, `0xBE4AC` `[EXE text]` |

### 7.2 Residences (0x52240.., `rules-11` §2)
| action | message |
|---|---|
| buy outright (0x52490) | **none** after the choice; the offer window `0xBE99C` `^ Wohnsitz kaufen _ … Angebote`, `0xBE9E0` `%a$L%5W für %d Taler$Z`, `0xBE9F8` `$LGutschrift: %d Taler$N`, `0xBEA18` `%j Kaufen` is the whole dialog |
| build (0x52790) | after paying: `box(160, 140, 320, 160, 0xBEAA8 $N$Z$C%1W wird im Jahr %l errichtet sein.$N, new word r = 0x5000 | k, year + BT[k])` |
| sell (0x52AA0) | `box(80, 120, 480, 240, 0xBEAD8 $Z$C^ Wohnsitz verkaufen _$Z$LWollt Ihr %4W für %d Taler verkaufen?$Z%j Ja$Z%j Nein$N, r, V_res/3, 0x6F, 0x30)`; 0 = yes → `cash_add`, `set_residence(none)`; no further text |
| extend (0x52D00) | all bits set: `box(160, 140, 320, 200, 0xBEC7C $Z$CEs gibt nichts, was Ihr %3W noch hinzufügen könntet.$N, r)`; after a paid change: `box(160, 180, 320, 120, 0xBED48 $N$CEure Wünsche werden im nächsten Jahr erfüllt sein.$N)` |
| renovate (0x53060) | the question itself is the box: `box(80, 100, 480, 280, 0xBED84 $Z$C^ Wohnsitz renovieren _$N$N$L$LDie Renovierung %2W kostet Euch %d Taler und würde bis zum Jahre %l dauern.$Z%j Renovieren$Z%j Nichts dergleichen tun$N, r, cost, year + 1 + [cond < 25], 0x72, 0x30)`; 0 → pay and create the type-13 record, no confirmation |
| auction (0x52B50) | `0xBEB34` `^ Wohnsitz versteigern _`, `0xBEB54` `Derzeit ist keine Versteigerung angesetzt.`, `0xBEB90` `[EXE text]` |

Residence delivery (turn start, §1) is silent; the built residence simply appears.

### 7.3 Transporters (0x53A60, `rules-02` §6)
- Ship/submarine in a non-navigable city: `box(160, 160, 320, 160, 0xBF0B0 $N$C%o liegt an keiner Wasserstraße. Daher nützen Euch Schiffe hier nichts.$N, city)`.
- Build, sell, repair: **no confirmation box**; the window (`0xBF100`/`0xBF114` header, `0xBF124` `Im Lager: %m`, `0xBF13C` `Zustand: %d%%`, `0xBF15C` `[EXE text]`, `0xBF1BC` `Bauauftrag`, `0xBF1D0` `Reparatur / Versteigern`) is redrawn.
- Auction without an announced auction: `box(140, 180, 360, 120, 0xBF200 $N$CDerzeit ist keine Versteigerung angesetzt.$N)`; auction full: `box(160, 160, 320, 160, 0xBF234 $Z$LDie Auktionsleitung nimmt keine weiteren %8s aus Eurer hiesigen Niederlassung mehr an.$N, 19 + type)`.
- Built and repaired transporters arrive silently at the next turn start (§1).

## 8. Corrections to earlier docs
- `rules-08` §1.5: a human buyer's "Nein" in the private castle sale cancels the sale (0x54EAF); it is not ignored.
- `rules-02` §4.3 "one transporter lost, with message": the message texts are ids 2815..2820 (`0xB6E54` table), printed after `$B`, with the destination name as `%s`; the cargo-loss step has no message.
- `rules-04b` §4.3 code 1: the health text index is `11 − min(life/2, 11)`, box at (120, 160, 400, 160), texts ids 2335..2346 as listed in §6.
