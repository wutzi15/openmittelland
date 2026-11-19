# Open questions, round 8 (rules-66)

Answers to `docs/IMPL-GAPS.md` §1 "For the RE role" and the RE questions of `docs/FIDELITY.md` "Year-end parity, playtest save 1619". Conventions as in `rules-44`/`rules-65`: `rand(n)` = `0x36FF0`, `rec_new` = `0x424A0`, `rec_find`/`rec_next` = `0x426B0`/`0x427B0`, `favour_decrease` = `0x436F0`, `cash_add` = `0x43790`, `pay` = `0x43860`, `net_worth(p)` = `0x44640` + `0x441F0`, `global_office(p)` = `0x44050`, `has(p, privilege)` = `0x5E600` (no draw), `person_pick_random(loc, controller)` = `0x43560`, `property_pick_random(kind, loc, owner)` = `0x42B90`. Person record `0xE3A7C + 248·i`. Tags: **[C]** instruction level, **[L]** decompiler only, **[G]** guess.

## 1. Year-end office loop (`year_end_world` 0x66220) [C]

Answer: yes, one loop over the person slots 0..191 (`0x66327..0x6697C`), and favour floor, dismissal check and wealth drift happen per person inside it. The rival friction of `rules-03` §4.1 is a separate loop before it (`0x6627F..0x66312`, per human: rival, `rand(3)`, `favour_decrease`).

Per slot p, in this order:

1. Slot unused (byte 0 = 0xFF, `0x66350`): jump to step 7. Nothing else.
2. Favour floor (`0x66367..0x663BC`): for j = 0..191, if `favour(p→j) < 10`: `favour(p→j) += rand(2)`. One draw per low cell, in j order.
3. `NW = net_worth(p)` (`0x663C6`).
4. If p holds an office (`+0x11 ≠ 15`, `0x663E0`): `o = global_office(p)`; dismissal check exactly as `rules-03` §4.3 (`NW < cache` or `+0xF7` set; `r`; appointers from `0x87318`; `favour_decrease` per appointer; `n_low > n_app/2` → scene when p is human, or p's office location > 12, or p is a human's rival (`0x66720`), then `office_vacate` `0x45980` (its 15 × `rand(34)` skill draws for a computer p), then purge `0x42F60` mode 3 for a computer p). Without office `o = 0` and no check.
5. `cache +0x1C = NW` (`0x66866`), humans and computers alike.
6. Controller nibble 0 (computer, `0x66872`): title cap step: if office location < 13 and `title < 0x86FB8[o]`: `title += 1` (no draw); then `x = 2 + rand(5)` (`0x668D6..0x668E5`, EBX), `g = rand(15)` (`0x668E8`), `skill_gain(p, g, x)` (`0x43BE0`: `skill[g] = min(skill[g] + x, 150)`), `person_init_wealth(p, 20, 7000)` (`0x66915`, EDX = 20, EBX = 7000). Human (`0x6691C`): `OFFICE_WEALTH[o]` update of `rules-03` §4.4, no draw.
7. `+0xF7 = 0` (`0x6697C`, written for unused slots too).

Draws per person: one `rand(2)` per favour cell below 10, then for a computer `rand(5)`, `rand(15)` (plus the 15 `rand(34)` of a dismissal); a human draws only the floor. This is the interleaving the parity run saw (person 12's floor between person 11's and 12's skill draws).

Corrects `rules-03` §4.4: the `rand(5)` is not unused. Its result + 2 is the amount of the yearly skill gain: `skill_gain(p, rand(15), 2 + rand(5))`, so a computer person's skill in one random good rises by 2..6 a year (cap 150). This closes `rules-03` open question 1.

## 2. Computer actions, draw list (`npc_turns` part of 0x66220, `npc_act` 0x59F80) [C]

`npc_turns` (`0x69BA0..0x69D00`), after the ten mistress events:
- Humans in slot order with an office: `rand(100) < 40`, then `person_pick_random(office location, computer)` = one `rand(count)` (`0x43560` lists the matching persons via `0x433E0`/`0x434E0` and draws only when the list is not empty), then `npc_act`.
- Counties 10, 11, 12: `rand(100) < 50`, pick (`rand(count)`), act.
- Principality: `rand(100) < 90`, pick, act.
- Type-6 records in heap order with byte 9 ≠ 0, human owner, computer target: `rand(100) < 50`, act (no pick).

`npc_act(p)`:
- Step A: `rand(100) < 40`; then `has(56)` and `rand(100) < 80` → text, `k = rand(10)`; else `has(59)` and `rand(100) < 90` → text, `k = 10 + rand(10)`; else `has(82)` → text, `k = 20 + rand(10)`; else no further draw, on to step B. `k` = 1: `rand(33)`; 5, 6: `rand(100)`; 7: `rand(700)`; 29: `rand(25)`; other k none. A change ends the action.
- Step B (`0x5A1A6`): up to 10 tries while nothing succeeded, each `c = rand(24)`, then:

| c | draws, in order |
|---|---|
| 0 | `rand(100)`; success needs ≥ 40 and `has(5)` (then `0x59720`) |
| 1 | `rand(100)`; if ≥ 30 and `has(8)`: `rand(5)`, `rand(10)`, [text], `rand(2000)` (always drawn) |
| 2 | `rand(100)`; if ≥ 20 and `has(9)`: `rand(5)`, `rand(10)`, `rand(2000)` |
| 3 | `has(16)`: `rand(2)`; else `has(31)`: `rand(10)`; else `has(40)`, `has(53)`: none; else `has(58)`: `person_pick_random` (`rand(count)`), then `standing(x, 20)` `0x17430` |
| 4 | `has(32)`: `rand(10)`; else `!has(54) && has(84)`: none; else `has(61)`: `rand(100)` |
| 5 | `has(55) || has(86)`: the miracle `0x59940` (its own draws); else `has(64)`, `has(69)`: none; else `has(71)`: `rand(2)` |
| 6 | `has(85)`: `rand(count)` pick, then one `rand(10)` per clergy person enumerated |
| 7..11 | shipment (below) |
| 12 | **no draw and no success**: offence bit 0x04 only, the loop goes on to the next try |
| 13, 14 | `rand(10)` for the good, then shipment |
| 15 | `rand(100)`; if ≥ 30, `has(27)`, fencing (`+0xF6`) < 80: `rand(5)`, `rand(10)` |
| 16 | `rand(100)`; if ≥ 40 and `has(25)`: `rand(2)` |
| 17..23 | none |

Shipment (`0x5AEE6..0x5B0F0`), good g, in order: `rand(120)` (quantity); **`property_pick_random(0, 0xFF, 0xFF)`** = one `rand(number of branches)` over every branch of every owner (`0x42B90` called at `0x5AF30` with EAX = 0, EDX = 0xFF, EBX = 0xFF); `rec_new(9, p, source property, …)`; `rand(2)` (counter `2 + rand(2)`); `rand(9)` (destination `(source city + 1 + rand(9)) mod 10`); then the ship conversion and the toll accumulation, no draws. Also, not in `rules-09`: offence bit 0x20000 when the counter came out 3 and flag law `0x85A35` (Eskortierung) is set. So a shipment is 4 draws after the case roll (5 for c = 13, 14).

Corrects `rules-09` §3: (a) the source is a **random branch** (uniform over all branches, one draw), not "the first branch in the property table"; (b) case 12 does **not** count as the action, it behaves like 17..23 (the try is used up, the loop continues).

Compared with `crates/mad-sim/src/ai.rs`: `npc_shipment` takes the first branch without a draw (one missing draw per shipment), case 12 returns `done = true` (the original keeps drawing), and the 0x20000 evidence bit is missing. Everything else matches the table draw for draw. Three shipments without their source draw and one early stop at a case-12 try account for the 11-draw gap of the parity run (the exact split depends on which tries the county-11 actor rolled).

## 3. Person re-initialisation (`0x42CF0`, called from the turnover at `0x66C45`/`0x66C78`) [C]

Writes, in code order: `+0x00` gender (AL), `+0x08` name index (DL), `+0x0F = 0`, `+0x10 = controller << 4` (BL = 0 here), `+0x11 = 15`, `+0xF5 = 0` (hotkey), `+0xF6 = rand(10)`, `+0x12 = 2`, `+0x13 = 8`, `+0xE0..+0xEE = 100`, `+0x02 = 25 + rand(15)`, `+0x20..+0xDF = 48 + rand(5)` each (192 draws), `+0x20 + self = 100`, `+0x18 = 750`, `+0xEF..+0xF2 = 100`, `+0x04 = 0xFF`, `+0x03 = state religion 0x85A4B`, `+0xF7 = 0`, `+0xF3 = 0`, then `0x42C40` rewrites the name pointer `+0x09..+0x0B`. The caller then writes `+0x00` again (same value), `+0x0D = 1 + rand(2)` and calls `person_init_wealth(p, 100, 10 000 000)`.

Left as they were: `+0x01`, `+0x05`, `+0x06`, `+0x07` (insurance), `+0x0E` (place byte), `+0x14..+0x17`, `+0x1C` (net-worth cache), `+0xF4` (colour). So the parity observation (cache and `+0x0E` kept) is the whole list of game-relevant survivors; `+0xF3` and `+0xF7` are cleared.

`+0xF6` is the fencing skill: `npc_act` case 15 reads it against 80 and raises it, and this function seeds it with `rand(10)`. Corrects `rules-00` §4 (`+0xF6` "meaning open").

Side finding [L]: the turnover condition tests castles and camps (`0x42A50` for kinds 1 and 2) and directorships, not branches; a replaced person's branches stay his, so the newcomer inherits them, and `person_init_wealth` then moves 100 % of the gap between their value and the no-office target in one step (cap 10 000 000).

## 4. Wealth drift writer (`person_init_wealth` 0x42EA0) [C]

`d = pct · (NW − OFFICE_WEALTH[global_office(p)]) / 100` (`0x42EDA..0x42EF4`, signed 32-bit). `d > 0` (richer than the target): `pay(p, min(d, cap))` (`0x42F0D`/`0x42F1C`). `d ≤ 0`: `cash_add(p, min(−d, cap))` (`0x42F35`/`0x42F46`). So the downward move goes through `pay`, which clamps at −9 999 999 (`rules-60` §1), and the upward move through `cash_add`, which clamps at +9 999 999. The band of `rules-00` §4 holds for the drift; a computer person at the floor cannot drift below it.

## 5. Population arithmetic width [C]

All three are 32-bit; the products wrap at 2^32.
- Disaster aftermath (`0x193C7..0x193D7`): `imul eax, [city+0x0E], 0x5F` then `xor edx, edx; div ebx` (ebx = 100): 32-bit product, **unsigned** divide.
- City growth (`0x4A416..0x4A426`): `imul eax, [city+0x0E]` with eax = `1000 + rand(50)`, then `xor edx, edx; div ebx` (1000): 32-bit, unsigned.
- Divine event (`0x59A63..0x59A81`): `imul edx, [city+0x0E], 0xA`, then `sar edx, 31; idiv ebx` (11): 32-bit, **signed**.

No `cdq`/`edx:eax` wide multiply anywhere in these. A population above ~45 million overflows the first two (unsigned), above ~214 million the third.

## 6. Battle goods markers and wagons [C]

Types 20 and 21 are not drawn by the `0x13AE0` path of `rules-58` §2. The object renderer handles them first (`0x3BB65..0x3BC66`) with `draw_icon` `0x2E310`:
- type 20 (marker): `draw_icon(x, y + 16, icon)` with icon `0x2C` while HP (`+0x0C`) > 50, else `0x1B8`;
- type 21 (wagon): `draw_icon(x, y − 29, icon)` with icon `0x2B` while HP > 50, else `0x1B7`;
- both skipped when `0xDC83F ≠ 0` (ESC mode).

`draw_icon` fetches the icon (`0x2DD70`) and blits it top-left anchored with `0x13AE0` into the window buffer `0xD500E` at `(x + 0xD5014, (y + 0xD5016 + 0xD501C) mod 0xD501E)` [L for the window terms; they are 0 for the field window in the capture]. With the allocator's `y = row·12 − 28`: marker top `row·12 − 12 = row·12 + 12 − 24`, wagon top `row·12 − 57 = row·12 + 12 − 69`. That is the measured bottom-edge rule exactly, for these two icon heights; the +16/−29 are constants, not a function of the height. x is the stored `col·24 + 1` / `col·24 − 9`.

Corrects `rules-58` §2: the "every type top-left at row·12 − 28" answer is right for men and cannons only; for types 20/21 use the constants above (`rules-51` §1 already named `draw_icon` and the icon ids).

## 7. Report lines without a known EXE text [L]

(a) Refused commands: there is no generic refusal text in the EXE (no "Nicht möglich"/"nicht möglich" string). The original does not print anything for the sim's `Refused` cases; each is a guard in the respective dialog that either leaves the menu item out, ignores the click, or prints the dialog's own text where one exists (those are in the per-screen docs, e.g. the Kontor refusals in `rules-11`, auction `0xBABB0` in `rules-08`). Our `Nicht möglich: …` line has no counterpart; nothing to print is the faithful choice. [G for "each"; not every dialog was re-read this round.]

(b) World fate with a city argument and no property drawn (`rules-06` §3.2, fate 4): the original prints with the index the pick returned (0xFF) and reads past the name table; there is no alternative text. Keep the guard, print nothing.

## 8. Smaller text questions

- `%2N0` in `0x9D605` = the spouse's name in the genitive (`rules-12` §1.5, k = 2 on form 0: name plus `'`/`s`), and `%g` = the wedding city's name with the same genitive marker (`rules-15` §7.1 step 9 args: c, F0, F1, name of W, F2, F3). "Gunst von Ortruds Freunden", "Persönlichkeiten Lichterheides". [C from the format tables]
- `%1N4`: "O in R" with an office, literally "ohne Amt" without (`rules-12` §1.5 table, format strings `0x9D83B..0x9D925` end in "ohne Amt"). Ours matches. [C]
- Erträge heading: the string exists, `0xBD9E0` `$t$|$4H$10>$56<$Z$C^ Produktion _$Z`, printed from `0x4B900` right after the `Erträge` title `0xBD9D8` (the strings dump skips it; read it from the image). [C]
- Spieltipps (`0x61A70`, option bit 8, `rand(5)`): tip 0 is the first branch in property order with a transporter type i (0..5) of count > 0 and condition ≤ 50, printed with `0xC09CC` and arguments (count, text 19 + i, branch location); `%4e` takes (count, noun id) and prints "Eure <noun>" in the number the count implies, without the number (`rules-12` §1.4). Tip 4 walks the production orders in heap order and, per order, tests in this order: `R[g]·14·m < 100·w` (`0xC0B7C`), then `w·140 < R[g]·10·m` (`0xC0BD4`), then `w > 49` (`0xC0C2C`, argument = the branch's city); the first hit prints and ends the tip. [C]
- Revolt (`0x1DE90`, c = 3): with S = 0 the text `0xAB0CE` is printed, `n` stays 0, and the code falls into the shared `n = 0` branch: `0xAB16C`, `favour_decrease`, record deleted. So yes, `0xAB16C` follows `0xAB0CE`. [C]

## 9. Name tables [C]

The LE image has 13 383 internal fixups, all object-relative (`00-loader.md`). In the file's page data the 4-byte slots of the pointer tables hold the target's **offset inside object 2** (e.g. `0xB4378` holds `0x7568, 0x7568, 0x7568, 0x7572` at file offset `0xB4B78`; `0xB6618` holds `0x20D89…`), and the fixup adds the object base `0x80000`. So "offset from linear 0x80000" is exactly right for every pointer whose target is in the data object, which is the case for the location, office, goods, means and title tables. A slot whose fixup targets object 1 would need `+0x10000` instead; none of the name tables do.

## 10. Election vote wait [C]

`menu` `0x35B60` takes the timeout from the high dword of its first argument (`0x35BA4`), sets `deadline = clock() + timeout` where `clock` = `0x6B6E5` → `0x6B67F`, the DOS INT 21h/2Ch time in **hundredths** since start (`rules-01` §5), and returns when `clock() ≥ deadline` (`0x365xx`, the same compare as the auto-repeat deadline of `rules-54` §4). 0xB4 is therefore 1.80 s per computer vote, quantised to BIOS ticks (≈ 55 ms) under real DOS, so 1.81..1.87 s.

## 11. Castle auction lot owner byte [C write side, L read side]

Write side: at `0x69A3C` the owner is `DL = [EBP−0x48]`. Inside `year_end_world` that stack byte is written only at `0x69E04` and `0x69F50`, both in the record pass **after** the lot creation; there is no earlier write in this function, so the byte is whatever a callee left in that slot earlier in the same call (not a fixed value; it is not the previous year's value either, since the slot belongs to this frame).

Read side (`auction settlement 0x27680`): for a type-16 lot the catalog page, the sale and the payment use the **property** record (lot byte 2): the winner pays (`pay(winner)` `0x28835`) and the price goes to the property's owner byte (`cash_add(property.+0x02)` `0x28848`), then `property.+0x02 = winner`. The lot owner byte is read once, into the "a seller is human" OR (`local_38 |= person[lot+1].+0x10`), which decides whether the humans are asked to attend (`0xBB100`) or the auction opens for them directly. A garbage index ≥ 192 reads past the person table (read only, no write). So in practice: payment and ownership are always right; the attendance question can be skipped when the stale byte happens to name a human or a church office holder.

## Corrections to older docs

- `rules-03` §4.4: the yearly `rand(5)` is used: `skill_gain(p, rand(15), 2 + rand(5))`; open question 1 closed. §4.2–4.4 are one per-person loop (order in §1 above). §4.5: the re-initialisation also sets religion to the state religion and `+0xF6 = rand(10)`; branches of the replaced person are not touched.
- `rules-09` §3: shipment source = random branch (one `rand(count)`), not the first branch; case 12 does not count as the action; new offence bit 0x20000 (Eskortierung, counter 3).
- `rules-00` §4: `+0xF6` = fencing skill, start `rand(10)`.
- `rules-58` §2: types 20/21 are drawn by `draw_icon` at `y + 16` / `y − 29`; the fixed-top answer holds for men and cannons only.
