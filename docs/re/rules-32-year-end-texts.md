# Year-end window: every computer-person and world line, in print order

Sources: `year_end_world` 0x66220 (window part 0x6890E..0x6A84E), `npc_act` 0x59F80, mistress acquire 0x20A00, mistress yearly 0x20B90, divine event 0x59940, message helper 0x368E0, world fate 0x26640, aftermath 0x19360, contest judging 0x19890, contest start 0x19480, auction announcement 0x269D0, chronicle 0x18AE0, revenge 0x5B200, person filter 0x433E0 / 0x43560. Tags as in `rules-00-map.md`: **[C]** read at instruction level (argument push order checked), **[L]** read from a helper dump or decompiler only. Text ids are entries of the pointer table at 0xB4258 (`rules-12` §0). `%`-code semantics: `rules-12` §1. No live run was made for this document.

Notation: p = the acting computer person; `loc(x)` = person byte +0x0F (office level location; a human without office has his city there, `rules-05` §2); `human_at(L)` = "some person with controller nibble 1 (human) and +0x0F = L exists" (0x433E0 with controller 1; `rules-00` §2 row 0x10); `rights(L)` = rights table 0x85E08[L] != 0xFF (a human owns any property at L, `rules-11`).

## 1. Correction to rules-19 §5 / rules-12 §3.10: divine events at year end are inline [C]

The message helper 0x368E0(x, y, w, h, fmt, args…) opens its own box (style 0x8810), prints, waits for OK and closes, **only when byte 0x84D46 is 0**. When 0x84D46 is non-zero it prints `fmt` into the **current** window and returns at once (no box, no click).

Both year-end callers of 0x59940 increment 0x84D46 around the call (0x5AA0A..0x5AA15 inside npc_act, 0x69DCF..0x69DDA for the prayer miracle). So at year end every divine-event text, the tornado included, is printed **inline into the "Ereignisse in Mittelland" text window**, and there is no box and no click. The box coordinates in `rules-12` §3.10 / `rules-19` §5 apply only to the privilege "Wunder wirken" (0x60E76, flag 0).

## 2. Window and order [C]

Full-screen window 0x8910, picture 0xB8, banner(0xFF), then text window `(100, 126, 440, 300)` style 0xA810 (scrolling, `rules-16b`). Everything below goes into that one window, in this order. Lines from other docs are listed so the order is complete; the computer/world lines this document adds are in §3..§7.

| # | site | what prints | condition |
|---|---|---|---|
| 1 | 0x68998 | herald 0xC19D0 `[EXE text]`, fanfare `music(0x1E, 0)` first | world auction record (type 3, owner 0xC0) exists with counter = 1 |
| 2 | 0x689E3..0x68FE3 | sabotage lines (`rules-12` §3.6); 10 % film | per type-7 record, `rand(100) < 25` |
| 3 | 0x68FE8 | (no text) average-shipment update | |
| 4 | 0x69082..0x69A41 | per **computer-owned** castle/camp property, in property-table order: §6 | |
| 5 | 0x69A5F | world fate 0x26640, §5.1 | option Schicksal (0x86AD1 & 4) and `rand(100) < 15` |
| 6 | 0x69A64..0x69B07 | world records owned by 0xC0, heap order: type 2 aftermath §5.2, type 4 contest judging §5.3 | every year |
| 7 | 0x69B48 | contest announcement 0x19480 (§5.4) | option Abenteuer (& 2), `year > 1605 + rand(5)`, `rand(100) < 4` |
| 8 | 0x69B6E | `[EXE text]` (0xBAB68), arg year + k + 1 | counter 0x85E06 = 0 (else it is decremented) and `rand(100) < 20` |
| 9 | 0x69B73 | 10 × mistress acquire for a random **computer** person (0x43560 controller 0), §4 | |
| 10 | 0x69BB8 | npc_act for each human h (index order) holding an office (+0x11 != 15): `rand(100) < 40` → random computer at `loc(h)` | §3 |
| 11 | 0x69C45 | npc_act for counties 10, 11, 12: `rand(100) < 50` → random computer of that county | §3 |
| 12 | 0x69CA6 | npc_act for the principality: `rand(100) < 90` → random computer at 13 | §3 |
| 13 | 0x69CDF | npc_act for the target of every type-6 (spy) record with +9 != 0, human owner, computer target: `rand(100) < 50` | §3 |
| 14 | 0x69D83 | prayer miracle: `rand(50) < 0x85CBF` (rolled always) → 0x85CBF := 0; if a computer Erzbischof exists (0x433E0(13, church 1, office 10, computer)): `[EXE text]` (0xC1E90), divine event inline (§7), `$N` (0xC1ED0) | |
| 15 | 0x69DEE..0x69F53 | record pass over **computer** persons (0x433E0 controller 0, person order), each person's records in heap order: type 18 trial 0x289E0, 22 duel 0x2C750, 25 siege 0x1F2A0, 26 relief 0x1FA80, 29 mistress 0x20B90 (silent for computer keepers, §4), 50 revenge 0x5B200 (§6.2); other types: delete when counter 0 | |
| 16 | 0x69F58 | `flip` (0x30400); the window stays open | |
| 17 | 0x69F5D..0x6A5AD | per human office holder: `[EXE text]` (0xC1ED4) / `… aus Rom.` (0xC1F30) / `… aus Ablassverkäufen.` (0xC1F7C), args (h, h, amount) | human only |
| 18 | 0x6A5AD..0x6A7E3 | `[EXE text]` (0xC1FD4) once, then `%s in %o` (0xC2004) / `Der %s von %o` (0xC2010) per office; then click and `$P` (0xC2020) | only if a human is concerned (0x47C20 returns 2) |
| 19 | 0x6A7E6 | **chronicle** 0x18AE0 with the type-1 record (owner 0xC0): `$V$P$C^ Historie _$Z$C` (0xBA880) then text id `year − 627` | option Geschichtsschreibung (& 1), `year < 1700`, and the entry text does not start with `?` |
| 20 | 0x6A814 | `$V` (0xC1CF4) | |
| 21 | 0x6A822 | `[EXE text]` (0xC2024) + click | cursor y (0xD501A) = 12, i.e. nothing printed since the last page start |
| 22 | 0x6A83F | window closed (0x30190), then the full-screen window | |

Paging [C for the escapes, formats.md §8.3 for their meaning]: the window scrolls; it only starts a new page at an explicit `$P`. Within the world part (#1..#15) there is no `$P`; the first `$P` is the `$V$P` of the first human tax line (#17), then the election header (#18), the `$P` after the election click, the chronicle header `$V$P` (#19), and #21. `$V` waits for a click only when the cursor is not at the top. So the chronicle always starts on its own page, after the world lines, after the human tax lines and after the next-year election list. Quirk: when #18 printed (it ends with `$P`) and the chronicle is skipped, the cursor is at the top again and #21 prints "keine besonderen Vorfälle" even though the window showed events earlier [C for the code path].

Whether the scrolling window waits for a click when text runs past its bottom (between explicit `$P`s) is [UNVERIFIED]; it was not traced here.

## 3. npc_act (0x59F80): every line [C]

All lines go to the current window (the year-end window; npc_act has no other caller). "Always" = printed whenever the branch runs, independent of any human. Bit/state effects: `rules-09` §3.

### 3.1 Step A, law change (`rand(100) < 40`)
The line is printed **before** `k` is drawn and before the law is touched, so it also appears for k = 8 (no change).

| privilege tried | site | text (id, address) | args | condition |
|---|---|---|---|---|
| 56, `rand(100) < 80` | 0x5A00E | 3233 (0xB0A87) `[EXE text]` | p | always; then k = rand(10) |
| else 59, `rand(100) < 90` | 0x5A127 | 3237 (0xB0B39) `[EXE text]` | p | always; k = 10 + rand(10) |
| else 82 | 0x5A16F | 3134 (0xAF0E7) `[EXE text]` | p | always; k = 20 + rand(10) |

There is one text per law book, not per law; the changed law and its new value are never printed.

### 3.2 Step B
`c = rand(24)`, up to 10 tries (`rules-09` §3). Lines in case order:

| c | branch | site | text (id, address) | args in print order | printed when |
|---|---|---|---|---|---|
| 0 | indulgence (priv 5, never granted) | | none | | |
| 1 | feast (priv 8) | 0x5A347 | 2021 (0x9DD6A) `[EXE text]` | p, adjective text id 2022 + b (`grandioses`, `schönes`, `mittelmäßiges`, `mäßiges`, `furchtbares`), city | `human_at(city)` (0x433E0(city, ff, ff, human)); b = rand(5), city = rand(10), both drawn first |
| 2 | building donation (priv 9) | 0x5A454 | 2519 (0xA4127) `[EXE text]` | p, city, suffix text id 2520 + b (`e Kathedrale`, `e Kirche`, ` Theater`, ` Hospital`, `en Springbrunnen`) | `rights(city)`; b = rand(5), city = rand(10) |
| 3 | building price (priv 16) | 0x5A4FE | 2731 (0xA7326) `[EXE text]` | p, loc(p) | always (printed before the change) |
| 3 | max. interest (else priv 31) | 0x5A5CE | 3119 (0xAEBBB) `[EXE text]` | p, loc(p), **new** rate | always |
| 3 | advisers (else priv 40) | 0x5A611 | 3125 (0xAED7A) `[EXE text]` | p, loc(p) | always |
| 3 | bless (else priv 53) | 0x5A6DF | 3054 (0xACF76) `$L%1N5, segnet die Grafschaft %o.$N$N` | p, loc(p) | always |
| 3 | project (else priv 58) | 0x5A7C9 | 3058 (0xAD06F) `[EXE text]` | p, r, amount | r = random **human** (0x43560 controller 1); amount = `standing(r, 20)` credited to r. If no human exists: no line, no action. Correction to `rules-09` ("a random person"): r is always a human |
| 4 | toll share (priv 32) | 0x5A865 | 3122 (0xAEC8E) `[EXE text]` | p, loc(p) + 4 (the county's castle location), **new** share | always |
| 4 | conversion (else not 54 and priv 84) | 0x5A8F5 | 2864 (0xA9794) `[EXE text]` | p, loc(p), text id 2865 + (religion(p) mod 2) (`evangelischen` / `katholischen`) | always |
| 4 | harshness (else priv 61) | 0x5A9A0 | 2739 (0xA73E2) `[EXE text]` | p | always (before the new value is drawn) |
| 5 | miracle (priv 55 or 86) | 0x5AA02 | 2348 (0xA1FFE) `$L%1N5, wirkt ein Wunder:$Z` | p | always; then the divine event inline (§7), then `$N` (0xBFCB8) |
| 5 | inquisition (else 64) | 0x5AA82 | 2979 (0xAB8D0) `$L%1N5, %s das Inquisitionsverhör.$N$N` | p, text id 2981 + new flag 0x85A4C (0 `erlaubt`, 1 `untersagt`) | always, after the toggle |
| 5 | ministers (else 69) | 0x5AABB | 3131 (0xAEFAF) `[EXE text]` | p | always; then jumps into the adviser rotation code with place = loc(p) (offices 3..5 at loc(p)) |
| 5 | state religion (else 71) | 0x5AB30 | 2870 (0xA98DF) `[EXE text]` | p, text id 2871 + new 0x85A4B (`evangelisch`, `katholisch`, `frei wählbar`) | always, after the change |
| 6 | canonise (priv 85) | 0x5ABAF | 3066 (0xAD376) `[EXE text]` | p, r, r | r = random **human** (0x43560 controller 1); none → no line, no action (correction to `rules-09`) |
| 7..14 | trade shipments | | none | | |
| 12, 17..23 | offence bits only | | none | | |
| 15 | fencing (priv 27, fencing < 80) | 0x5AD44 | 3116 (0xAEB23) `$L%1N5, lernt fechten.$N$N` | p | `human_at(loc(p))`; the fencing gain happens either way |
| 16 | insurance (priv 25) | 0x5AE5C | 2631 (0xA5BF2) `[EXE text]` | p, text id 2634 + i | i = `2·rand(2) + 1`: i = 1 bit 0x02 `Prozesskosten`, i = 3 bit 0x08 `Sabotageakte` (bits from table 0x84B4C = 1, 2, 4, 8); printed only if the bit is **set** after the toggle and `human_at(loc(p))` |

Duplicate: text id 3137 at 0xAF17A is the same string as id 2348 and is not referenced anywhere in the code; the game prints id 2348 (0xA1FFE).

## 4. Computer mistress / lover lines [C]

0x20A00(x), called 10 times with x = random computer person (0x43560(ff, ff, ff, controller 0)):
1. Always prints `$B` (0xBA37C, justified alignment, no visible text).
2. If x already has a type-29 record: nothing more (the human message box branches are unreachable here because x is a computer).
3. Else, if `rights(loc(x))` and `rand(100) < 6`: 2401 (0xA2C4E) `$L%1N5, %s.$N$N` with (x, text id 2408 + rand(5)): 2408 `geht fremd` (0xA2D3A), 2409 `[EXE text]`, 2410 `[EXE text]`, 2411 `[EXE text]`, 2412 `feiert wüste Orgien`. The `rand(100)` is drawn only when the rights test passes; the `rand(5)` only when the line prints.
4. Then always: type-29 record for x with counter `2 + rand(5)`.

Mistress yearly handler 0x20B90 (record pass, #15): both its texts (2856 "… verlangte …", 2851 "… verlässt Euch!") are guarded by "keeper is human". The year-end record pass only visits computer persons, so it prints nothing at year end.

## 5. World lines

### 5.1 World fate 0x26640 [C]
Always prints `$B` first, then exactly one line (k = rand(5), `rules-06` §3.2). Every kind prints; no human condition.

| k | text (id, address) | args |
|---|---|---|
| 0 | 3360 (0xB373E) `[EXE text]` | good name `0xB6618[g]`, g = rand(15) |
| 1 | 3361 (0xB37B1) `[EXE text]` | good name |
| 2 | 3362 (0xB3823) `[EXE text]` | city name `0xB6194[c]` |
| 3 | 3363 (0xB392B) `[EXE text]` | city name |
| 4 | 3079 (0xAD653) `[EXE text]` then 3080 (0xAD6A1) `$LSämtliche $hWarentransporte müssen unverrichteter Dinge wieder umkehren$h.$N$N` or 3081 (0xAD6F8) `[EXE text]` | besieger `0xB686C[rand(12)]`, city name; 3081: the same besieger |

`$h…$h` is the highlight toggle (the red words).

### 5.2 Disaster aftermath 0x19360 (type 2, owner 0xC0) [C]
Creation year (counter bit 7): silent. Otherwise, with `rand(10) < 2`: `$B`, then text id 1797 + d with (record +6 year as `%l`, city name string = text id [city +0]), then `$N$N` (0xBA8CC). d = 0 fire `[EXE text]`; 1 flood `[EXE text]`; 2 plague `[EXE text]`; 3 avalanche `[EXE text]`. No human condition.

### 5.3 Contest judging 0x19890 (type 4, owner 0xC0) [L for the loop structure, C for the ids]
`$B`, 2760 (0xA7D49) `[EXE text]` (city name, judge = record +5); per category 2761 `[EXE text]` (adjective `0xB5D58[…]`, good name); places 2762 / 2763 / 2764 (each ends with `$W`, a click); 2765 `[EXE text]`. Conditions and prize rules: `rules-06` §6.

### 5.4 Contest announcement 0x19480 [L]
1718 (0x9A0C8) `[EXE text]` (newspaper name, fee), human entry menus 1725..1727 / 1743 / 1744, or `[EXE text]` (0xBA8D4), else 1745 `[EXE text]` (city name, judge). Details: `rules-06` §6.

## 6. Computer castles and revenge

### 6.1 Per computer-owned castle/camp (#4) [C for texts and the conditions shown]
In property-table order; properties owned by humans are skipped entirely.

| text | args | condition |
|---|---|---|
| `%1N3 erhält %d Taler Zolleinnahmen aus Burg %o.$N$N` (0xC1D14) | h, amount, castle location | castles only; h = human holding office 7 (non-church) of the county `castle location − 14 + 10`; amount = 30 % of the accumulated toll; printed if h exists and amount > 0 |
| `[EXE text]` (0xC1D4C) + menu | buyer h, owner, location, price | no type-16 record for the property; a buyer was chosen from the type-17 interest records (n = their count, max 6; choice rule [L], `rules-07`); `rand(100) < 80·n/6`; buyer is human |
| `[EXE text]` (0xC1DD8) | location, price, buyer | same roll, buyer is a computer person; always printed |
| `$L%1N5, saniert Burg %o.$N$N` (0xC1E00) | owner, location | not sold, condition (+3) < 50 and `rand(100) < 30`, castle only (a camp is repaired silently); always printed |
| `[EXE text]` (0xC1E20) | owner, location | +0x5C < 70 and `rand(100) < 5`; castles and camps; always printed |
| `[EXE text]` (0xC1E60) | location, year + world auction record counter | castle, world auction record exists, no type-16 record for it, `rand(100) < 5`; always printed |

The sale mechanics (prices, interest records) are in `rules-07`; only the print conditions were checked here.

### 6.2 Revenge records 0x5B200 (#15) [C for ids, L for the tithe line's arguments]
Victim is always human (`rules-09` §4). Lines: `$B` (0xBFCBC) first; c = 1 toll: `[EXE text]` (0xBFCC0) with (actor, castle name, new rate); c = 5: 2723 `[EXE text]`, 3107 `… das Einkommensniveau in %o.`, 3110 `… das Lagerraumpreisniveau in %o.` (actor, place), 3113 `[EXE text]` (actor, place, new tax), 2974 `[EXE text]` (actor, 100 − kept %, victim), 3128 `[EXE text]` [L], 2803 `[EXE text]` (actor, victim, f); c = 6 duel challenge 1830 with `$V` before it; trial denunciation `$L$h%1N5, schwärzt %4N0$h bei %3N5, an.$Z` (0xBFCFC). Trials, duels, sieges and relief bring their own scenes or texts (`rules-18`, `rules-22`, `rules-21`).

## 7. Divine event texts at year end (0x59940, inline) [C]

Preceded by `$L%1N5, wirkt ein Wunder:$Z` (npc_act) or `[EXE text]` (prayer), followed by `$N`. Effects and k selection: `rules-06` §3.1. Every text is text id 2350 + k and starts with `$Z$L`:

| k | id, address | args |
|---|---|---|
| 0 | 2350 (0xA205D) `[EXE text]` | city name, city name |
| 1 | 2351 (0xA2104) `[EXE text]` | city name, illness (swallowed) |
| 2 | 2352 (0xA2161) `[EXE text]` | city name, city name |
| 3 | 2353 (0xA220E) `[EXE text]` | x, x |
| 4 | 2354 (0xA228B) `[EXE text]` | city name |
| 5 | 2355 (0xA22EC) `[EXE text]` | city name |
| 6 | 2356 (0xA2388) `[EXE text]` | city name, illness (swallowed) |
| 7 | 2357 (0xA241C) `[EXE text]` | none |
| 8 | 2358 (0xA2475) tornado | see below |
| 9 | 2359 (0xA25FF) `Unerklärlicherweise leiden viele Bewohner der Stadt %s unter einer Krankheit. Sie haben %s; ihre Gesundheit sinkt.` | city name, illness `0xB683C[rand(12)]` |
| 10 | 2360 (0xA267D) `[EXE text]` | none |

Tornado (k = 8), settles `rules-12` §4's open point. Draw order: A = rand(10), B = (A + 1 + rand(9)) mod 10, n = 50 + rand(50), then six cosmetic numbers in this order: `rand(3)+2`, `rand(4)+2`, `rand(5)+2`, `rand(3)+4`, `rand(3)+5`, `rand(3)+6` (`rules-06` §3.1 lists four; there are six). Arguments in print order: A name, churches `rand(3)+6`, museums `rand(3)+5`, hospitals `rand(3)+4`, inmates n, A name, churches `rand(5)+2`, museums `rand(4)+2`, hospitals `rand(3)+2`, inhabitants `n/2 + 1`, B name. The effects (wealth, population) are applied after the print.

## 8. Open
- Auto-paging of the scrolling 0xA810 window when text overflows between `$P`s [UNVERIFIED].
- Contest judging/announcement loop details (§5.3, §5.4) were taken from the print dump only [L].
- Revenge tithe line (3128) argument sources [L].
