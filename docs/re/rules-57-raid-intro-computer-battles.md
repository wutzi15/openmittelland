# Raid intro args, computer-only battles, blackmailed canon (rules-57)

Source: `battle_main` `0x3EE60` (intro block `0x3F8D7..0x3FB77`, pre-fight caption `0x3FD30..0x3FDE6`), its only caller `settlement_pre` `0x61410` (type-51 loop `0x6142B..0x61505`, type-54 loop `0x6150A..0x618B0`), church wedding `0x212B0` (Domherr step `0x21683..0x21926`). Helpers as in `rules-50`: `caption` `0x36C90`, caption menu `0x36AC0`, `rand(n)` `0x36FF0`, `music` `0x41E50`, `activate` `0x389C0`, `deploy` `0x3DDA0`. `A` = attacker `0xDB7F4`, `D` = defender `0xDB7F5`, `S` = shipment record (heap type 9, `rules-00` §5: +1 owner, +2 source branch property index, +5 u16 quantity, +7 good, +10 destination location), `H` = attacker home property (`0xDC86C + 0x60·rec[+2]`), `P` = "target property" local `[EBP−0x60]`. Human test = person flags `0xE3A8C + 0xF8·p` & `0xF0` ≠ 0 (as in `rules-07` §4). Tags: **[C]** instruction level, **[L]** decompiler or partial trace, **[UNVERIFIED]** inferred. Everything is [C] unless marked.

## 1. Raid intro captions (codes 0..3)

### 1.1 What A, D and P are for a raid

Set once at `0x3F125..0x3F18F`, before any draw:
- `A := rec[+1]` (raid record owner), `H := property rec[+2]`, for every code.
- Code < 4: **`D := S[+1]`** (the shipment owner) and **`P := property S[+2]`** (the shipment's source branch). The call-off block (`0x3F191..0x3F1FA`, `rand(200)`) is only on the code ≥ 4 branch, so a raid draws no call-off roll and the flag `[EBP−0x38]` is never written for raids.
- Code ≥ 4: `P := property at location rec[+4]+14`, `D := P[+2]`, call-off roll as in `rules-50` §2.

### 1.2 Texts and arguments

Table `0xB5FBC + 16·code`, lines 1..4 (ids 1881..1896):

| code | 1 | 2 | 3 | 4 |
|---|---|---|---|---|
| 0 | `0x9C07D` `[EXE text]` | `0x9C0B5` `$C%2N3 Karren rollen gemächlich dahin...$N` | `0x9C0E1` `[EXE text]` | `0x9C112` `[EXE text]` |
| 1 | `0x9C16A` `[EXE text]` | `0x9C1A6` `[EXE text]` | `0x9C1DC` `[EXE text]` | `0x9C242` `$CEin Knacken im Unterholz kündigt das Schreckliche an; ein Fuhrmann ruft: >Ein Hinterhalt!< Doch es ist zu spät: %2N3 %s springen mit gezückten Waffen auf den Weg...$N` |
| 2 | `0x9C30A` `[EXE text]` | `0x9C349` `[EXE text]` | `0x9C399` `[EXE text]` | `0x9C3E9` `[EXE text]` |
| 3 | `0x9C471` `[EXE text]` | `0x9C4CB` `[EXE text]` | `0x9C532` `[EXE text]` | `0x9C5AF` `[EXE text]` |

"Passstraße", "Laue Brise" and "Flußbiegung" are the original spellings.

Arguments (the same push sequence serves all six codes; `caption` is cdecl, first argument pushed last):
- Line 1 (`0x3F947..0x3F9A9`): `(byte 0xDC86D + 0x60·S[+2], S[+10], P[+1])`. The first is the location byte of the property at index `S[+2]`, i.e. the **source branch city**; the second is the **destination location** `S[+10]`. `%o … %o` consumes these two: "zwischen <source> und <destination>". The third argument, `P[+1]`, is for a raid the same source location again (P is that very property) and is not consumed by the raid texts. Confirmed as asked.
- Line 2 (`0x3F9B1..0x3F9EC`): `(D)` with `D = S[+1]`, so `%2N3` is the **shipment owner** in the genitive: "<owner>s Karren rollen …", "<owner>s Fuhrleute …", "<owner>s Matrosen …".
- Line 3: no arguments.
- Line 4 (`0x3FAC0..0x3FB40`): `(A, 0xB601C[k], P[+1])` with `k = 0` if `H[+0]` (home property kind) = 2, else `k = 1` (`0x3FAD4..0x3FAE8`). Same rule and same table as the castle line 4 (`rules-50` §1.1): `Söldner` for a castle-based attacker, `Räuber` for a camp-based one. `%2N3 %s` consumes `(A, word)`; `P[+1]` is unused by the raid texts (castle/camp line 4 uses it for `%o`).

### 1.3 Geometry, waits, conditions

- Lines 1..3: `caption(40, 180, 560, 160, 0, …)`; line 4: `caption(40, 160, 560, 160, 0, …)`. `delay = 0`, so each waits for a click. Identical to codes 4/5: the three calls at `0x3F9A9`, `0x3F9EC`, `0x3FA27` and the line-4 call at `0x3FB40` are straight-line code with no test of the battle code between `0x3F947` and `0x3FA2F`.
- The only code-dependent branch in the intro is `0x3FA2F`: `code < 4` jumps straight to line 4 (`0x3FAC0`); `code ≥ 4` first tests the call-off flag. So raids: no call-off roll, no call-off caption, always four lines.
- Field state during the captions: defender deployment already placed (for raids: goods markers, carts/sailors, escort, `rules-07` §5.4); attacker deployment follows line 4 (`0x3FB77`).

### 1.4 Order around the raid captions

For code < 4, from the value computation on:
1. `value := S[+5] · (S[+7] + 8)` (`0x3F763..0x3F783`), then jump to `0x3F8E2` (the castle path's `net_worth_goods` at `0x3F8DA` is skipped).
2. `music(0x1B, 4)` (`0x3F8EC`): **`rand(3)`**.
3. `activate(field, 0, BL = 1)` (draw only) and the present of the field (`0x3F8F1..0x3F941`).
4. Intro lines 1, 2, 3 (click each).
5. Line 4 (click).
6. Attacker deployment loop (`0x3FB77..0x3FCD5`, with the computer-attacker cap `max(1, value/350)` for code < 4 when A is computer, `0x3FBAA..0x3FBC6`), then `activate` + present again (`0x3FCDA..0x3FD2A`).
7. **Raid pre-fight caption** (`0x3FD30`, code < 4 only): `0xBBA04` `[EXE text]` when `S[+7] ≠ 14`, else `0xBBA48` (`… haben …`); args `(D, 0xB6618[S[+7]], value)`; box `(40, 180, 560, 160, 0)`, click.
8. `win_open(120, 120, 400, 240, 0x8810)` (`0x3FDE6`), ransom dialog (`rules-52` §1).

No `rand` is drawn between step 2 and the ransom dialog other than inside `deploy` (none, `rules-50` §1.3 step 5) [C for the intro block; the attacker loop calls only `deploy`].

Answer: for a raid `D = S[+1]` (shipment owner) and `P = property S[+2]` (source branch), both set at `0x3F166..0x3F18C`. Line 1 shows `zwischen <source city> und <destination S[+10]>`; the args are `(byte 0xDC86D + 0x60·S[+2], S[+10], P[+1])` as asked. Line 2's `%2N3` is the shipment owner. Line 4 is `(A, 0xB601C[home kind ≠ 2])`, the castle rule. Same boxes, same click waits, no call-off roll (the `code < 4` test at `0x3FA2F` skips it). The captions come after `music(0x1B)`'s `rand(3)` and before line 4, the attacker deployment and the `0xBBA04`/`0xBBA48` value caption, which comes right before the ransom window.

## 2. Battles with no human side

`battle_main` has exactly three callers, all in `settlement_pre` `0x61410` (`0x614D2` code 4, `0x614E3` code 5, `0x6188E` raid). It contains no "both computer" test of its own: the human tests inside it (`0x3F426`, `0x3FBAA`, `0x3FE21`, `0x3FFA5`, `0x401D4`, `0x40543`, `0x4072A`, `0x40FDF`, `0x410D7`, `0x41333`) only pick between human and computer dialogs for one side. The field, captions, tactics, pre-fight window and report are always presented once `battle_main` runs.

The filter is in the caller:
- **Type 51 (castle/camp attack)**, `0x6148B..0x614BD`: `battle_main` is called only if `human(rec[+1])` **or** `human(owner of target property P[+2])`. If both are computer the code jumps to `0x614E8`: no call, no record deletion, no draw. Nothing else happens with that record; its troops return at the owner's next turn start (`rules-07` §4).
- **Type 54 (raid)**, candidate loop `0x6158C..0x615EE`: a shipment is listed only if its owner ≠ p, its destination county matches, `human(p)` **or** `human(S[+1])` (`0x615BE..0x615EC`), and the transporter is Karren or Schiff. With both sides computer the shipment is not a candidate. With zero candidates (`0x61666`) the record draws nothing: no `rand(1000)`, no gate rolls, no `battle_main`.

Consequences:
- No clock reseed (`randomize_from_time` is inside `battle_main`, `0x3EEA7`), no `rand(200)`, no helper draws, no `music` `rand(3)`, no activate frame logic. The year-end RNG stream for a computer-vs-computer order is untouched by that order.
- This matches `rules-07` §4 ("Between two computer persons nothing happens") and §3 (5) and adds the instruction addresses.
- A human-involved battle at year end (computer attacker, human target or human shipment owner) is presented in full, including the human's dialogs, even though it is not the human's turn.

Answer: there is no presentation-skipping mode. `settlement_pre` never calls `battle_main` unless A or D is human (`0x6149C..0x614BD` for attacks, `0x615C9..0x615EC` for raid candidates). A computer-vs-computer attack record is left alone, and a computer shipment is invisible to a computer raider. No RNG draw happens for such a pair.

## 3. Blackmailed canon in the church scene

Domherr step of `0x212B0` (`rules-15` §7.1 step 5, `rules-05` §3 step 3). `D` = canon (`0x433E0` search, `[EBP−0x20]`), `f` = flag `[EBP−0x18]`, `i` = loop slot `[EBP−0x40]`.

1. Blackmail loop `0x216B4..0x2174D`: for `i = 0..191` with an occupied slot and `favour(D→i) > 100`: caption menu `0xBA9AC` (`[EXE text]`, box (80, 180, 480, 280), args `(i, D)`). There is no human test on `i`; the menu is shown for computer blackmailers too. Answer 0 → `f := 1` and the loop ends with `i < 192`; any other answer → `f := 0`, loop continues.
2. `0x21752`: the human-Domherr question `0x9D215` runs only if `human(D)` **and** `i ≥ 192` (no blackmail answer 0). Its result: 0 ("Ja!") → caption `0x9D2A7` `[EXE text]`, `f := 0`; otherwise caption `0x9D288` `$C$N%1N3 hat keine Einwände.$N` (D), `f := result + 1` (≥ 2).
3. Otherwise (`0x21809`): `f := 0` (veto) only if `favour(D→suitor) ≤ 50` and `suitor ≠ D` and `i ≥ 192`; else `f := 1`. Veto prints `0x9D2C8` `[EXE text]` (D).
4. `0x21893`: `f = 0` → `0x9D32A` (`[EXE text]`), record deleted, back to the report. `f = 1` → `0x9D1E8` `[EXE text]`. `f ≥ 2` (human D said no) → nothing more; straight to the priest's question `0x9D3A6`.

So when a blackmailer chose "die Heirat untersagen lassen": step 2 is skipped even for a human canon, step 3 yields `f = 1` with no caption, and the canon says nothing. The only line is `0x9D1E8` `[EXE text]`, the same line as for a silent computer canon. There is no text for "the canon was forced" and no inverted outcome; the quirk of `rules-05` §3 (the answer that reads as a ban guarantees the wedding) stands.

Answer: no. A blackmailed canon prints nothing of his own; the scene shows `0x9D1E8` `[EXE text]` (table slot `0xB6098`) and continues with the vows. `0x9D288` "hat keine Einwände" appears only when a human canon was asked and declined, and that path is closed by the blackmail flag (`i < 192` test at `0x21762`).
