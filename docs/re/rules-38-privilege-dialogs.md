# Privilege dialogs (rules-38)

Source: privileges menu `0x5F080(player)` (called from the Hinterzimmer menu `0x5C450` at `0x5C76A`), privilege check and entry printer `0x5E600(p, viewer, code, show)`, law book `0x49010(mode)` with line printer `0x48690(book, i, ro)`, item create `0x34130`, item painter `0x34BF0`, `menu` `0x35B60`. Effects of the actions are in `rules-04b` §4.3; this document adds what is on the screen, in which order, and which values the player enters. Tags: **[C]** read at instruction level (call arguments checked), **[L]** decompiler only, **[UNVERIFIED]** inferred. Everything is [C] unless marked.

Conventions (as in `rules-26`/`rules-34`):
- `win(x, y, w, h, style)` = `0x2FA80` (`rules-16b` §2). `print` = `0x322C0`, returns the index of the first item it creates ("base"). `flip` = `0x30400`, `win_close` = `0x30190`, `clear_page` = `0x30F10`.
- `box(x, y, w, h, fmt, …)` = `0x368E0` (`rules-15` §0.4): returns `clicked − base`, so the first `%a` line is 0, the second 1. The OK glyph (`0x348D0`, style `0x1690`) is never hit: `menu` skips every item whose style has bit **0x400** (item flag byte +9 bit 2, `0x35B60` hit loop) [C]. A right click returns "last item + 1". So a box with two `%a` lines returns 0, 1, or ≥ 2 (right click = cancel). The same 0x400 rule makes read-only law lines, the cost displays of the siege dialogs and the map flags unclickable (§3, §2.9, §10).
- `menu(base, 0, &delta)` = `0x35B60`: waits for a click, returns the item index, writes the spinner/cycle delta to `delta` (`rules-26` §1: numeric items step `10^d` by digit position, upper half +, lower half −, clamped to the item range; cycling string items (kind 0xA0) ±1). The caller applies the delta.
- `item_set(i, data)` = `0x34B80` (new text pointer or icon), `item_range(i, lo, hi)` = `0x34B00`, `items_paint(base)` = `0x35B00`, `items_free(base)` = `0x34AC0`, `ok_item` = `0x348D0`, `wait_click` = `0x36720`, `choose(base)` = `0x36850` (flip, wait for a click on an item ≥ base, remove those items, return `clicked − base`).
- `pay(p, x)` = `0x43860`, `cash_add(p, x)` = `0x43790`, `rec_find(type, owner, key)` / `rec_next` = `0x426B0`/`0x427B0`, `rec_new(type, owner, key, counter)` = `0x424A0`, `persons(loc, church, office, ctrl)` / next = `0x433E0`/`0x434E0` (0xFF = any), `favour_increase(a, b, x)` = `0x43620`, `favour_decrease(a, b, x)` = `0x436F0` (`rules-05`), `standing(x, k)` = `0x17430` (`rules-04` §1), `office_tree(player, mode)` = `0x45C30` (`rules-29`; returns the chosen person or 0xFF), `rand(n)` = `0x36FF0`.
- `T[i]` = text id i of the pointer table `0xB4258` (string address = `u32[0xB4258 + 4i]`). `L[loc]` = location name `T[1999 + loc]` (`0xB6194`: Fürstenbrück … Auenthal, Mittelland, castles, camps). `B[loc]` = `T[2003 + loc]`, which for a county loc 10..12 is the county's castle name `T[2013..2015]` Hohenfels / Seefels / Waldenfels. Name id of code c = `0x86778[c]` (`rules-04b` §4.1); below `N` = that id, so the entry text is `T[N]`, its description `T[N+1]` and further dialog texts `T[N+2..5]`.
- `player` = the human whose turn it is (menu argument), `holder` = the person whose privileges are shown (§1.4; normally the player), `loc` = holder's office location (person +0x0F), `city` = city record `0x85F04 + 140·loc` for loc < 10. `flags` = the turn flag word `0x85CBA` (`rules-04b` §2.1).

## 1. The privileges menu `0x5F080(player)`

### 1.1 Window
1. If the low-memory build gate `0x86BC8 < 0xC3501` holds: `music(0x15, 1)` (`0x41E50`, `rules-14` §4). Otherwise no music change.
2. `win(100, 40, 440, 400, 0x8810)`: parchment box (`rules-16b` §2).
3. Each redraw starts with `base = print(0xC0584 "$C$(^ %2N0 Privilegien _$)$N$L", holder)` → centred title "Hansens Privilegien" (genitive of the holder's name, `rules-12` §1.5), then left-aligned.

### 1.2 Entries
For every code c = 0..86 in order, `privilege(holder, player, c, show)` (`0x5E600`, `rules-04b` §4.1) is called; `show` = 1 when the entry falls on the current page (§1.3), else 0. A return of 1 prints one line, a return of 2 prints a line with a second item (a "setting"). The menu keeps a table of item codes: entry c contributes code `c` for its bullet item and `c + 0x57` (87) for the setting item. Lines are printed by `0x5E600` itself; every line starts with the bullet item `%m` (style `0x1090`, glyph `0x73`, the "-" of `%a`), then the entry text:

| code | entry text | setting item (second `%m`) |
|---|---|---|
| 1 | `T[2333]` @0xA1DEF `%m Medicus konsultieren$N` | – |
| 4 | `T[2400]` @0xA2C3E `%m %s nehmen$N` with `T[2402 + gender(p)]` Mätresse / Geliebten | – |
| 6 | `T[2857]` @0xA95EF `%m Testament machen$N` | – |
| 10 | `T[3178]` @0xAFE66 `%m Einkommen: %d Taler$N` with the office income `0x5E500(p)` | – |
| 13 | `T[2714]` @0xA705F `%m Handelszertifikat %s$N` with `T[2719 + cert]` I / II / III (person +0x05) | – |
| 14 | `T[2722]` @0xA7255 `%m Lohnniveau: %m$N` | style `0xA0` (cycling text), text `T[2725 + city.+0x7F / 21]` (sehr niedrig, niedrig, normal, hoch, sehr hoch) |
| 15 | `T[3106]` @0xAE8C8 `%m Einkommensniveau: %m$N` | `0xA0`, `T[2725 + city.+0x80 / 21]` |
| 16 | `T[2730]` @0xA7313 `%m Baupreise: %m$N` | `0xA0`, `T[2733 + city.+0x81 / 21]` (sehr günstig, günstig, normal, teuer, sehr teuer) |
| 17 | `T[3109]` @0xAE972 `%m Lagerraumpreise: %m$N` | `0xA0`, `T[2733 + city.+0x82 / 21]` |
| 22 | `T[3112]` @0xAEA14 `%m Umsatzsteuer: %m%%$N` | style `2` (u8 spinner, 2 digits) on `&city.+0x83`, `item_range(5, 33)` |
| 31 | `T[3118]` @0xAEB9B `%m Maximaler Kreditzins: %m%%$N` | style `2` on `&0x85A45[loc−10]`, range 0..60 |
| 32 | `T[3121]` @0xAEC6A `%m Anteil an Zolleinnahmen: %m%%$N` | style `2` on `&0x85A48[loc−10]`, range 0..33 |
| 51 | `T[3127]` @0xAEEB2 `%m Kirchenzehnt: %m%%$N` | style `2` on `&0x85A42[loc−10]`, range 0..25 |
| 61 | `T[2738]` @0xA73CA `%m Rechtsprechung: %m$N` | `0xA0`, `T[2741 + 0x85A4D / 21]` (sehr lasch, lasch, normal, hart, sehr hart) |
| 64 | `T[2978]` @0xAB8B5 `%m Inquisition ist %m$N` | style `0x1034` (plain text item, 8 M-cells wide), `T[2981 + 0x85A4C]` erlaubt / untersagt |
| 71 | `T[2869]` @0xA98C0 `%m Die Staatsreligion ist %m$N` | `0xA0`, `T[2871 + 0x85A4B]` evangelisch / katholisch / frei wählbar |
| all others | `T[N]` (e.g. `%m Ein Bauwerk stiften$N`) | – |

The spinner items show the live value because their data pointer is the game variable itself; the menu redraws them after each click (§1.5). Codes 0, 2, 3, 5, 7 never return anything (dead entries).

### 1.3 Pages
`count` counts every entry the holder has (plus the switch line of §1.4). Page 0 shows entries 0..11, page 1 shows entries 12 and up; there are only two pages (`page = 1 − page`). The item `0x34130(240, 364, 0, 0, 0x41030, 0xC05DC "Mehr Privilegien")` (centred text item at window y 364) is created when page 0 has more than 12 entries, and always on page 1 (where it leads back). It carries code 0xAF. Entries on the hidden page are evaluated with `show = 0` (no items) but still counted.

### 1.4 Other persons' privileges
After the entries, the menu looks for the next person after the holder (cyclic over slots 0..191, starting at holder + 1) who exists (byte 0 ≠ 0xFF) and whose **favour toward the player is above 100** (the player blackmails him, `rules-04` §3.4) or who is the player himself. Call him `cand`:
- `cand ≠ player`: line `0xC05A4 "$N%m %2N1 Privilegien...$N"` (bullet `0x1090`/`0x73`, then "Graf Hansens Privilegien...") with code 0xAE;
- `cand = player` and `holder ≠ player`: line `0xC05C0 "$N%m Eure Privilegien...$N"`, code 0xAE;
- `cand = player = holder`: no line.
Clicking it makes `cand` the holder and goes to page 0. Repeated clicks cycle through all blackmailed persons and back to the player. With another holder the check `privilege(holder, player, c)` is made with viewer ≠ p, so "self" entries (`rules-04b` §4.1: Medicus, Testament, income shares, …) are not listed; the actions use the holder's office location and `standing(holder, k)` for their costs, but the player's cash, insurance byte and fencing skill (`local player record` in §2) and the player's turn flags.

### 1.5 Click handling, redraw, exit
After the items are built: `ok_item`, `flip`. Then, repeatedly: wait until no mouse button is held (`0x12DE4` polled), `r = menu(base, 0, &delta) − base`.
- `r ≥ number of items` (right click, or the unclickable OK glyph never): `items_free(base)`, `clear_page` when `r = count`, `win_close`, return.
- Code 0xAF ("Mehr"): toggle the page, redraw. Code 0xAE: switch holder (§1.4), page 0, redraw.
- Otherwise the code's case runs (§2). Entries with code ≥ 87 are setting items (`setting = 1`, code −= 87). A setting change sets the redraw flag; an info box or a sub-dialog does not, so the menu waits for the next click without rebuilding (sub-dialog windows erase themselves on close, style bit 0x800).
- Redraw = `items_free(base)`, `clear_page`, rebuild from §1.1 step 3 in the same window.

## 2. Actions

`local player record` = person record of the player (`0xE3A7C + 248·player`). Boxes are `box(x, y, w, h, text, args…)`. "info box" means a box with only the OK glyph: it closes on a right click (or on any `%a` line if the text had one; these texts have none).

### 2.1 Information-only entries
| code | box | text, arguments |
|---|---|---|
| 1 Medicus | (120, 160, 400, 160) | `T[2334]` @0xA1E09 `[EXE text]` with `T[2335 + i]`, `i = 11 − min(life/2, 11)`, life = player +0x02: `[EXE text]` (i = 0, life ≥ 22), `[EXE text]`, `[EXE text]`, `>Es geht Euch gut.<`, `[EXE text]`, `[EXE text]`, `[EXE text]`, `[EXE text]`, `[EXE text]`, `>Es sieht nicht gut aus.<`, `[EXE text]`, `>Macht Euer Testament!<` (i = 11, life 0..1) |
| 10 Einkommen | (160, 160, 320, 160) | `T[3179]` @0xAFE7F `[EXE text]` (player, `0x5E500(player)`) |
| 11 Kein Kirchenzehnt | (160, 160, 320, 160) | `T[3181]` @0xAFED9 `[EXE text]` (player) |
| 13 Handelszertifikat | (140, 140, 360, 200) | `T[2716 + cert]` @0xA70B4/@0xA713E/@0xA71D6 (`[EXE text]` / `[EXE text]` / `…III gestattet Euch, jede Ware…`) |
| 14, 15, 16, 17 bullet | (140, 140, 360, 200) | `T[N+2]`: `T[2724]` @0xA7294 `[EXE text]`, `T[3108]` @0xAE915 (Einkommensniveau für Stadtämter), `T[2732]` @0xA7356 (Baupreisniveau), `T[3111]` @0xAE9C2 (Lagerraumpreisniveau); args (player, `L[loc]`) |
| 22 bullet | (140, 120, 360, 240) | `T[3114]` @0xAEA65 `[EXE text]` (player, `L[loc]`, `0x85A22`) |
| 31 bullet | (160, 140, 320, 200) | `T[3120]` @0xAEC09 `[EXE text]` (player) |
| 32 bullet | (160, 140, 320, 200) | `T[3123]` @0xAECED `[EXE text]` (player, `B[loc]`) |
| 41 | (160, 160, 320, 160) | `T[N+1]` = `T[3213]` `[EXE text]` (player, `B[loc]`) |
| 51 bullet | (140, 120, 360, 240) | `T[3129]` @0xAEF11 `[EXE text]` (player, `L[loc]`, `0x85A41`) |
| 61 bullet | (160, 140, 320, 200) | `T[2740]` @0xA7426 `[EXE text]` (player, `L[loc]` swallowed by `%v`) |
| 64 bullet | (160, 140, 320, 200) | `T[2980]` @0xAB8FC `[EXE text]` (player) |
| 71 bullet | (160, 160, 320, 160) | `T[2874]` @0xA9920 `[EXE text]` (player) |
| 18, 19, 20, 21, 23, 24, 30, 33..39, 42..47, 49, 50, 52, 57, 60, 62, 63, 65..68, 70, 72, 73, 75..78, 83 | (140, 120, 360, 240) | `T[N+1]` (the description, e.g. `T[3185]` `[EXE text]`), args (player, `L[loc]`); texts ending in `%v` swallow the location |

### 2.2 Settings (second item clicked)
No box. The clicked item is updated in place and the menu redraws:
- **14, 15, 16, 17**: `v = city.+0x7F/+0x80/+0x81/+0x82`; `v = clamp(v + 21·delta, 0, 100)`; `item_set(item, T[2725 or 2733 + v/21])`. One click on the upper half = +21 (one level), lower half −21.
- **22**: `city.+0x83 += delta` (delta already clamped by the item range 5..33, step 1 or 10 by digit).
- **31**: `0x85A45[loc−10] += delta` (0..60). **32**: `0x85A48[loc−10] += delta` (0..33).
- **51**: `0x85A42[loc−10] += delta` (0..25); if `delta ≠ 0`: `flags |= 0x08`.
- **61**: `0x85A4D = clamp(0x85A4D + 21·delta, 0, 100)`, `item_set(T[2741 + v/21])`.
- **64**: `0x85A4C ^= 1`, `item_set(T[2981 + v])` (no delta; the item is a plain text item).
- **71**: `0x85A4B = (0x85A4B + 3 + delta) mod 3`, `flags |= 0x100000`, `item_set(T[2871 + v])`.

### 2.3 9 Bauwerk stiften
1. `win(120, 120, 400, 240, 0xA810)` (scrolling text window). `base = print(T[2525] @0xA418A "$Z$LWelcher Stadt wollt Ihr ein Bauwerk stiften?$Z%a%m$|$Z", 0xA5, "")`: bullet (item 0), city item (item 1, style 0xA5 = cycling text, 5 cells, created with an empty string). `flip`.
2. Loop: `item_set(base+1, L[c])` (c starts at 0 = Fürstenbrück), `r = menu(base, 0, &delta) − base`; `r = 1` → `c = (c + 10 + delta) mod 10` (all ten cities), repeat; `r = 0` → city chosen; anything else → cancel (close window, back to the menu).
3. `items_free(base)` (no `clear_page`: the question text, its bullet and the chosen city name stay on the window, the prices follow at the cursor; rules-71 §7). Prices are rolled now, k = 0..4: `P[k] = rand(100) + {100000, 50000, 25000, 10000, 5000}[k]` (`0x872EC`), stored at `0xEF4C8[k]`, each printed as `T[2526] @0xA41C5 "%aEin%s für %d Taler.$|"` with `T[2520 + k]` (`e Kathedrale`, `e Kirche`, ` Theater`, ` Hospital`, `en Springbrunnen`) and `P[k]`; then `T[2527] @0xA41DD "%aNichts!$|$Z"`. `flip`, `k = choose(base)`.
4. `k < 5`: `print(T[2528] @0xA41EB "$LIhr stiftet ein%s, vermehrt dadurch den Reichtum von %s und steigert Euer Ansehen bei den Honoratioren der Stadt.$N", T[2520+k], L[c])`; turn flag: `flags |= 0x80000` if the **menu item index** `r_menu < 2`, else `0x2000000` (see Corrections 1); `pay(player, P[k])`; mission record (type 0, owner player) with kind 18 or 28: `+5 += P[k]`; `0x43F30(c, (5−k)·3)`; for every **computer** person x in city c (`persons(c, 0xFF, 0xFF, 0)`, controller 0, … `0x434E0`): `favour_increase(player, x, rand((5−k)·2 + 1))`; `wait_click`. k = 5 or more: nothing.
5. `win_close`.

### 2.4 25 Versicherung abschließen
1. `win(60, 120, 520, 240, 0x8810)`. `base = print(T[2632] @0xA5C2A "$Z$C^ %2N0 Versicherungen _$Z", player)`.
2. Four lines k = 0..3: `print(T[2633] @0xA5C4A "%aGegen %s...$E$R%m$N", T[2634+k], 0x30, T[2638])` → bullet (item `base+2k`), name `Katastrophen` / `Prozesskosten` / `Überfälle` / `Sabotageakte`, right-aligned text item (style 0x30, item `base+2k+1`) initially "Nein".
3. `print(0xC05F0 "$|$Z$CJede Versicherung kostet Euch$Njährlich 1%% Eures Vermögens.$N")`, `ok_item`, `flip`.
4. Loop: for each k `item_set(base+2k+1, T[2638 + ((player.+0x07 & M[k]) != 0)])` with `M = 0x87300 = {1, 2, 4, 8}` → "Nein" / "Ja" (`T[2639]`); `items_paint(base)`; `r = menu(base, 0, …) − base`; `r < 8` → `player.+0x07 ^= M[r/2]` (bullet or text of the line toggles), repeat; else (right click) `items_free`, `win_close`.
So insurance bit value 1 = Katastrophen (city disasters), 2 = Prozesskosten, 4 = Überfälle, 8 = Sabotageakte. This settles `rules-04b` open question 4 (bit 0).

### 2.5 27 Fechtunterricht
`f = player.+0xF6`; `cost = 2·f·f + 302 + rand(100)` (rand drawn on opening); `box(140, 160, 360, 160, T[3117] @0xAEB44 "$ZWollt Ihr für %d Taler Eure Fechtkünste verbessern?$Z%aEn garde!%aKeine Gewalt!$|", cost)`; 0 → `pay(player, cost)`; if `f < 95`: `f += 1 + rand(5)`. Anything else: nothing (the rand(100) was still drawn).

### 2.6 28 Stadt belagern
1. Scan all type-25 records: `besieged[city +4] = 1`, `total++`, `own++` if owner (+1) = player.
2. `total ≥ 10`: info box (160, 140, 320, 200) `T[2156]` @0x9FA16 `$Z$LDer seltene Fall ist tatsächlich eingetreten; damit hätte man als Programmierer rechnen müssen: Alle Städte sind belagert - Pech gehabt.$N$N`; done. Else `own ≥ 3`: info box (140, 180, 360, 120) `T[2144]` @0x9F397 `[EXE text]`; done.
3. `win(120, 120, 400, 240, 0xA810)`; `base = print(T[2145] @0x9F3D5 "$LWelche Stadt wollt Ihr belagern?$Z%a%m$N%aKeine$|", 0xA5, "")`: items 0 bullet, 1 city (cycling, 5 cells), 2 "Keine" bullet. `c` = first unbesieged city. `flip`.
4. Loop: `item_set(base+1, L[c])`, `items_paint`, `r = menu − base`; `r = 1` → `c = (c + 10 + delta) mod 10` repeated until an unbesieged city; `r = 0` → chosen; else cancel (`win_close`).
5. Army: `N0 = min(city[c].+0x0E / 6, 50000)` (u32 field, `rules-00` §6), `N = N0`. `base = print(T[2146] @0x9F409 "$Z$LIhr baucht dafür mindestens$N%d Landsknechte.$ZWollt Ihr$N%a$L$h%m$h Landsknechte für ein Handgeld von %m$Nund einen Sold von %m Talern anheuern oder$N%alieber nicht belagern?$|", N0, 0x15, &N, 0x426, &H, 0x426, &S)`: items 0 bullet (hire), 1 `N` (u16 spinner, 5 digits, highlighted), 2 `H = 10·N`, 3 `S = 5·N` (u32 displays, 6 digits, no zero padding, style 0x400 = unclickable), 4 bullet (not). `flip`; `item_range(base+1, N0, 50000)`.
6. Loop: `H = 10N`, `S = 5N`, `items_paint`, `r = menu − base`; `r = 1` → `N += delta` (clamped N0..50000, step 1/10/100/1000/10000 by digit); else break. `r = 0`: `print(T[2147] @0x9F4C5 "$Z$LEuer Heer wird mit Waffen ausgerüstet und der Tross setzt sich in Bewegung.$N")`, `pay(player, 10·N)` (the Handgeld; the Sold `5N` is only displayed here), `wait_click`, `rec_new(25, player, 0xFF, 4)` with `+4 = c`, `+5 = u16 N`. Else cancel.
7. `win_close`.

### 2.7 29 Stadt entsetzen
Same shape: `besieged[]` from type-25 records whose owner ≠ player (own sieges are excluded); none → info box (160, 160, 320, 160) `T[2483]` @0xA3D37 `[EXE text]`. Window (120, 120, 400, 240, 0xA810), `T[2477]` @0xA3A5A `[EXE text]` (items 0 choose, 1 city cycling over besieged cities, 2 Keine). Army `N0 = min(city.+0x0E / 10, 50000)`; `T[2478]` @0xA3A8E `[EXE text]` with (N0, 0x15, &N, 0x426, &H): items 0 hire, 1 N spinner (range N0..50000), 2 `H = 10N` display, 3 abort. Confirm: `T[2147]` text, `pay(player, 10·N)`, `wait_click`, `rec_new(26, player, 0xFF, 4)`, `+4 = c`, `+5 = N`.

### 2.8 40 Berater vertauschen / 69 Minister vertauschen
`box(140, 100, 360, 280, T[N+2], player)`: `T[3126]` @0xAEDF7 `[EXE text]` / `T[3132]` @0xAF01E `[EXE text]`. 0 → `h[i] = persons(loc, 0, 3+i, 0xFF)` for i = 0..2, then every `h[i] ≠ 0xFF` gets office index `(i+1) mod 3 + 3`. No cost, no flag, no message.

### 2.9 48 / 74 / 79 Arbeiter verschwinden lassen
`pct = 80 / 70 / 60`; `cost = standing(holder, 20)` (drawn first, always). If `0x85E79 ≠ 0` (already used this turn; cleared at turn start `0x4CBAD`): info box (140, 160, 360, 160) `0xC0640 "$N$Z$CIhr könnt dieses Privileg erst im nächsten Jahr wieder verwenden.$N"`. Else `box(140, 120, 360, 240, T[N+2], player, cost)`: `T[2975]` @0xAB7ED `[EXE text]` / `T[2985]` @0xAB9A9 (`30%% … %aHokus Pokus!%aNein.`) / `T[2990]` @0xABA43 (`40%% … %aSim-sala-bim!%aNein.`). 0 → `t = office_tree(player, 8)` (verb "Wählt", `rules-29`); `t ≠ 0xFF` → `flags |= 0x08`, `pay(player, cost)`, every type-8 record of t: `+5 = +5 · pct / 100`, `0x85E79++`. The texts `T[N+3]` "Wo wohnt Euer Opfer?" / `T[N+4]` "Wählt Euer Opfer!" are not referenced by this code.

### 2.10 53 Grafschaft segnen
`cost = standing(holder, 30)`; `k = loc − 10`; `box(120, 100, 400, 280, T[3055] @0xACF9E "$Z$LWollt Ihr die Grafschaft %s für %d Taler segnen und damit die Produktion von Waren in den Städten %s, %s und %s erleichtern?$Z%aJa%aNein$|", L[loc], cost, T[2000+3k], T[2001+3k], T[2002+3k])` (the county's three cities). 0 → info box (160, 180, 320, 120) `T[3056]` @0xAD030 `[EXE text]` (L[loc]); `flags |= 0x08`; `pay(player, cost)`; type-23 record with `+4 = loc` (first found of any owner, else `rec_new(23, player, 0xFF, 5)`); `+3 = 5`, `+4 = loc`.

### 2.11 54 Grafschaft bekehren / 84 Fürstentum bekehren
`cost = standing(holder, 30)`; `r = box(120, 100, 400, 280, T[N+4], cost, L[loc])`: `T[2867]` @0xA97CD `[EXE text]` / `T[2879]` @0xA99B2 `[EXE text]`. `r < 2` → info box (160, 140, 320, 200) `T[N+5]` (`T[2868]` @0xA984F `[EXE text]` / `T[2880]` @0xA9A26) with (L[loc], `T[N+2+r]` evangelischen / katholischen); `flags |= 0x08`; `pay`; type-24 record with `+4 = loc` (reuse or `rec_new(24, player, 0xFF, 5)`), `+3 = 5`, `+4 = loc`, `+5 = r` (0 evangelisch, 1 katholisch). For 84, loc = 13 and L[13] = "Mittelland".

### 2.12 55 / 86 Wunder wirken
`cost = standing(holder, 20)` (55) / `standing(player, 10)` (86); `box(160, 100, 320, 280, T[N+2], cost)`: `T[2349]` @0xA201E `[EXE text]` (`T[3138]` is the same string). 0 → `pay(player, cost)`, then the miracle `0x59940` (`rules-15` §4, `rules-06`).

### 2.13 56 / 59 / 82 Gesetze ändern
`law_book(0)` / `law_book(1)` / `law_book(2)` (§3). No cost, no confirmation; `T[3135]` (`[EXE text]`) is not used.

### 2.14 58 Projekte unterstützen
1. `win(120, 100, 400, 280, 0x8810)`; `X = 0`; `base = print(T[3059] @0xAD0BB "$Z$LWollt Ihr jemanden mit Staatsgeldern unterstützen? Dies würde Euch 20%% der Summe sowie Ansehenseinbußen bei den Konkurrenten des Unterstützten kosten!$Z%a%m Taler >weiterleiten<!$N%aDas Staatssäckel schonen!$|", 0x24, &X)`: items 0 forward, 1 `X` (u32 spinner, 4 digits → default range 0..9999, step 1/10/100/1000), 2 spare. `flip`.
2. Loop: `r = menu − base`; `r = 1` → `X += delta`; else break. `items_free`, `win_close`.
3. `r = 0` and `X ≠ 0`: `t = office_tree(player, 0)` (verb "Wählt"). `t = 0xFF` → nothing. `t = player` → info box (160, 160, 320, 120) `T[3060]` @0xAD199 `$N$CAber %1N1! Wie verwerflich!$N` (player), nothing else. Otherwise `pay(player, X/5)`, `cash_add(t, X)`, and for every person x at t's office location (`persons(loc(t), 0xFF, 0xFF, 0xFF)`) with `x ≠ t`: `favour_decrease(player, x, rand(5))`. So the most that can be forwarded per use is 9999 Taler.

### 2.15 80 Zwietracht säen
`cost = standing(holder, 30)`; `box(160, 120, 320, 240, T[2804] @0xA8614 "$Z$LAls %1N4 könnt Ihr für %d Taler zwischen zwei Personen Feindschaft säen. Wollt Ihr von diesem Privileg Gebrauch machen?$Z%aJa, he he!%aNein.$|", player, cost)`; 0 → `office_tree(player, 9)` (two-step pick, `rules-29` §2/§3, effect `rules-04` §3.6); result ≠ 0xFF → `flags |= 0x08`. **The cost is only displayed; nothing is paid here** [C]. Whether mode 9/10 of the office tree charges it is not covered by `rules-04` §3.6 [UNVERIFIED].

### 2.16 81 Inquisitionsverhör
`cost = standing(holder, 30)`; `box(120, 100, 400, 280, T[3063] @0xAD241 "$Z$LAls %1N4 könnt Ihr für %d Taler ein Inquisitionsverhör ohne Verteidiger anstrengen. Als Richter fungiert Ihr, der Erzbischof und der Erzdiakon.$Z%aEin Opfer wählen$N%aGnade zeigen$|", player, cost)`; 0 → `office_tree(player, 7)` (`rules-04b` §3.1 creates the kind-1 trial and sets flag 0x08). No payment here either [C]; same open point as 2.15.

### 2.17 85 Heiligsprechen
`cost = standing(holder, 35)`; `box(160, 160, 320, 160, T[3067] @0xAD3CF "$Z$LWollt Ihr für %d Taler jemanden heiligsprechen?$Z%aHallelu-ja!%aNö.$|", cost)`; 0 → `t = office_tree(player, 0)`; `t ≠ 0xFF` → `flags |= 0x08`, `pay(player, cost)`; if `t ≠ player`: for every clergy person x (`persons(0xFF, 1, 0xFF, 0xFF)`): `favour_increase(t, x, 10 + rand(15))` (x's favour toward t, `rules-05`). See Corrections 2.

### 2.18 Built elsewhere
4 → `0x20A00(player)` mistress (`rules-05`); 6 → `0x45A70(player, 1)` testament (`rules-17`); 8 → `0x20160(player)` feast (`rules-08`); 12 → `office_tree(player, 5)` trial; 26 → `office_tree(player, 6)` insult/duel (`rules-29` §2).

## 3. Law book `law_book(mode)` `0x49010`

`book = mode` for mode 0..2 (edit finance / criminal / church), `book = 0x85A20` (last viewed) for mode 3 (browser, called from the Hinterzimmer `0x5C45B`). `ro = (mode == 3)`.
1. `win(80, 40, 480, 400, 0x8810)`. `base = print("")` (empty string at `0xBC540`, only to fix the item base).
2. Per page: `v = strictness(book)` (`0x48DE0`, `rules-04b` §1.1, 0..4); `print(0xBD15C "$Z$C^ Gesetze in Mittelland _$Z")`; header `T[807]` @0x895CF `$CFinanzgesetze: %s$Z$C` / `T[2485]` @0xA3DCE `$CStrafgesetze: %s$Z$C` / `T[2496]` @0xA3EDB `$CKirchengesetze: %s$Z$C` with `T[2624 + v]` (Sehr liberal, Liberal, Neutral, Repressiv, Sehr repressiv); then ten lines `0x48690(book, i, ro)`, i = 0..9, each creating exactly one item (index `base + i`):

| book, i | text | item |
|---|---|---|
| flag laws (`rules-04b` §1 table) | `T[808]` `Kredite sind %m$N`, `T[810..812]`, `T[817]`, `T[2486..2495]`, `T[2497..2505]` | style `S | 0x1030`, text `T[2620 + flag]` `erlaubt.` / `verboten.` (finance, criminal) or `T[2622 + flag]` `gottgefällig.` / `lästerlich.` (church) |
| finance 1 | `T[809]` `Größte Umsatzsteuer: %m%%$N` | `S | 2` (u8, 2 digits) on `0x85A22`, range 0..33 |
| finance 5, 6 | `T[813]` `Minimaler Arbeiterlohn: %m%%$N`, `T[814]` `Minimaler Söldnerlohn: %m%%$N` | `S | 3` (u8, 3 digits) on `0x85A26` / `0x85A27`, range 50..150 |
| finance 7 | `T[815]` `Talerhöchstgrenze: %m$N` | `S | 0x27` (u32, 7 digits) on `0x85A28`, **no `item_range` call**: default range 0..9 999 999 (`0x34130`: `10^digits − 1`) |
| finance 8 | `T[816]` `Höchstzahl Faktoreien: %m$N` | `S | 2` on `0x85A2C`, range 2..10 |
| church 9 | `T[2506]` `Maximaler Kirchenzehnt: %m%%$N` | `S | 2` on `0x85A41`, range 0..25 |

`S = 0x1400` when `ro` (browser), `0` when editing. Bit 0x400 makes the browser's items unclickable (§0), bit 0x1000 drops the background buffer. So in edit mode numbers are zero-padded (`0.050.000` style, `formats.md` §8.6) and in the browser they are not.
3. Mode 3 only: `print(0xBD17C "$Z$C%j Finanzen   %j Justiz   %j Kirche$N", 0xB2, 0xB3, 0xB4)`: three icon items (items `base+10..12`). Then `ok_item`, `flip`.
4. `r = menu(base, 0, &delta) − base`.
   - `r < 10` (edit modes; impossible in the browser): flag law → `^= 1`; number law → `+= delta` (`0x85A28` as u32, the rest as bytes). The delta is what `menu` computed: `±10^d` for the digit clicked (upper half +, lower half −), clamped so the value stays inside the item range. The cash cap therefore moves by 1, 10, 100, …, 1 000 000 depending on the digit, within 0..9 999 999. This answers `rules-04b` open question 1.
   - mode 3 and `10 ≤ r ≤ 12`: `book = r − 10`.
   - `items_free(base)`, `clear_page`; repeat while (mode 3 and `r < 13`) or `r < 10`. Anything else (right click) leaves.
5. Mode 3: `0x85A20 = book`. `win_close`.

## 4. Corrections to other docs
1. `rules-04b` §4.3 code 9: the turn flag does not depend on the building. The code tests the **index of the clicked menu entry** (`r` of §1.5, the position of "Ein Bauwerk stiften" on the current page): index 0 or 1 → `0x80000` (church), otherwise `0x2000000` (secular) [C, `0x5F869`]. For the player's own menu Medicus is always entry 0 and Fest geben precedes Bauwerk, so the player's own donations always set `0x2000000`; through a blackmailed holder (no self entries) Bauwerk is entry 1 and sets `0x80000`. The building k still drives price, `0x43F30(c, (5−k)·3)` and the favour roll.
2. `rules-04b` §4.3 code 85: the clergy favour gain is `10 + rand(15)`, not `rand(15)` [C, `0x61384`].
3. `rules-04b` §4.3 codes 80 and 81: the menu shows `standing(holder, 30)` but pays nothing; see §2.15/§2.16.
4. `rules-04b` §4.3 code 25: bit values 1/2/4/8 = Katastrophen/Prozesskosten/Überfälle/Sabotageakte (§2.4); the text says 1 % of net worth per year.
5. `formats.md` §8.6 "Law book": the polarity is reversed. Style flags are `0x1400` in the read-only browser (mode 3) and `0` when the player edits (`0x490E6`: flag = (mode == 3)) [C].
6. `rules-26` §0 "OK glyph practically unclickable because it is 1×1": the real reason is style bit 0x400 (hit test skips it, `0x35B60` item flag +9 bit 2) [C]. Same bit on the siege cost displays and the map flags.
7. `rules-04b` §4.3 code 9: the favour roll runs over the city's computer persons only (controller argument 0), not over every person [C, `0x5F8F9`].
8. `rules-29` §2: modes 0 and 8 are reached from codes 58/85 and 48/74/79 as listed there ✓; mode 7 and 9 callers ✓.

## 5. Open / unverified
- Whether office tree modes 7 and 9/10 charge the displayed `standing` cost (`rules-04` §3.6/§3.7 do not say).
- `0x5E500` (office income shown by code 10) is not re-derived here (`rules-00` §9 lists it as office income).
- Numeric default ranges rely on the `0x84DA8` table being `10^n` (consistent with `rules-26` §1's `0x84DA4 = 0, 1, 10, 100, …`) [L].

## 6. Credits / quit screens `0x4D880(save)` and `0x4DA50()`
Callers: `0x4D880` from the options screen `0x4D1E0` (`0x4D47B`), from `game_main` (`0x4B11F`, after the main menu) and from the history/reminders code `0x17930` (`0x18A6A`, after a caption menu `0x36AC0(80, 180, 480, 120, …)` answered 1) [L for the meaning of the callers]. `0x4DA50` from `game_main` `0x4B3B2`, right after the caption `0xBD5B0 "$CSämtliche Spieler sind ausgeschieden. Damit ist diese Partie >Mittelland AD< leider beendet…"` (box (10, 360, 620, 120), `wait_click`, `0x15480`).
1. `0x4D880` only: `box(160, 160, 320, 160, 0xBDC98 "$Z$C^ Spiel beenden _$ZSeid Ihr Euch sicher?$Z%m  %m$N", 0x1030, "Ja", 0x1030, "Nein")`; anything but 0 returns to the caller. If `save ≠ 0` (AL argument): `0x42360(5)` saves the game to slot 5 (`game5.dat`, the "letztes Spiel fortsetzen" slot [L]); then `0x421A0` saves CONFIG.DAT. `0x4DA50` skips the box and both saves.
2. `items_free(1)`; `win_close` until only one window is open (`0x84D44 ≤ 1`); `music(0x18, 0)` (`rules-14` §4); `pic = {0xA3, 0x102, 0x104}[rand(3)]` (table `0x87064`; `0x4DA50` reads `0x8706A`, the same three values: ICONA3, ICONG2, ICONG4); `draw_icon(0, 0, pic)`; `flip`.
3. Sixteen iterations i = 0..15, each: `music(0x1F, 1)` (re-assert, no-op), `draw_icon(0, 0, pic)` (erases the previous line), text cursor y := 155 (`0xD501A = 0x9B`), then
   - i < 2: `print(T[2242 + rand(2)])` (`[EXE text]` or the two names swapped, @0xA078F / @0xA07DA) and `i++` (so this consumes i = 0 and 1 as one line);
   - else `print(T[2242 + i])`: `T[2244]` Grafisches Konzept und Leitung … `T[2256]` Projektleitung: Bürger Arne Peters, `T[2257]` the same with `$Nohne Amt` (i = 15).
   - i = 15 (last line): `flip`, sample 0xCC loaded (`0x124E1(0xCC)`), played (`0x12698(0xCC, 3)`), `delay(2500 ms)` (`0x6C083`), stopped (`0x12730(0xCC)`). Otherwise the line is revealed with the screen transition `0x2C460(1)` (`rules-16b` §8: rows 155..324, 200 frames, 958 game-RNG draws per line).
   - Then `kbhit` (`0x6B081`); if a key is pending it is read (`0x6AC77`) and **Esc (0x1B) breaks the loop**; other keys are swallowed.
   So 15 lines are shown: 1 random "Idee und Programmierung" + `T[2244..2257]`.
4. Exit: `0x12B65` (sound driver shutdown), `0x2F970` (icon cache reset), `0x131DB` (video off), `exit(1)` (`0x6AF62`). The game never returns to the menu from here.
Saved state: only what step 1 saved (game slot 5 when requested, CONFIG). Nothing else is written.

## 7. Type-55 records (prisoners), turn-end handler `0x63D0D` [C]
Part of the human turn-end record pass (`rules-37` §2 row "55 prisoners"); the record is created by the battle code (`rules-07` §5, battle end: +4 = prisoners taken, key = the participant's home property). Per record:
1. `n = +4` (u8). `n ≤ 0`: delete the record, nothing printed.
2. `prop = property record (key +2)` (`0xDC86C + 96·key`). One draw `t = rand(30)`; `m = add_troops(prop, n/2, 30 + t)` (`0x45240(property, men, training)`, `rules-07` §1 step 3: men capped by free capacity, round-robin over the four slots starting at `rand(4)` inside `add_troops`). So half the prisoners (integer division) are offered as recruits with training 30..59.
3. `m > 0`: castle (prop kind +0 = 2) → `print(0xC0E4C "Euer General auf Burg %o konnte %4u in den Folterkammern davon überzeugen, dass es besser wäre, sich Euren Truppen anzuschließen…", prop.+1 location, m, 42)` (noun 42 Gefangene, `rules-12` §1.4); camp → `0xC0EDC "Euer Räuberhauptmann >überredete< %4u zum Verweilen im schönen %o..."` (m, 42, location). `m ≤ 0`: `0xC0F28 "Euer General auf Burg %o hatte einen schlechten Tag:"` / `0xC0F60 "Euer Räuberhauptmann im %o langweilte sich:"` (location).
4. `n > m`: `n − m > 1` → `0xC0F90 " %d Gefangene verschwanden spurlos..."` (n − m); else `0xC0FBC " Ein Gefangener verschwand spurlos."`. Then `0xC09C4 "$N$N"`.
5. Delete the record (`0x42550`). RNG per record: `rand(30)` once, plus the `rand(4)` inside `add_troops` (only when it is reached: `rules-07` §1 says men are limited first; whether `rand(4)` is drawn when 0 men fit is in that doc, not re-read here [UNVERIFIED]).

## 8. "Produktion" page skill gain (`rules-37` §1 #12) [C]
Code `0x4C3FE..0x4C768` in `turn_start_reports`. `out[g]` = this turn's output per good (local array), `produced` = number of goods with `out[g] > 0`, `cap` = rows that fit (13, or 11 when the "given away" line is present, `rules-02` §3.3).
1. If `produced = 0`: `0xBDA04 "$LIm letzten Jahr habt Ihr keine Waren produziert.$N"`, no draws.
2. First loop, g = 0..14 in index order while `listed < cap`: goods with `out[g] ≤ 0` are skipped (no draw). For each listed good: the row is printed (`0xBDA40 "$L       %s:$E$R%d$Z"` with the good's name and `out[g]`, icon `0x14A + g` at the left), then **one `rand(3)`** and `skill_gain(p, g, 1 + rand(3))` (`0x43BE0`: `person.+0xE0+g = min(old + x, 150)`). After the 8th row the second column starts (`0xBDA58 "$|$4H$56>$10<"`).
3. Second loop only when `listed < produced` (the list filled up before every produced good was shown): starting at `g` = the index after the last listed good, while `g < 15`: `total += out[g]`, `g += 1`, **one `rand(3)`**, `skill_gain(p, g, rand(3))` with the **already incremented** index and amount 0..2. Every remaining index up to 14 is processed regardless of its output; the final iteration (old g = 14) writes index 15 = person byte `+0xEF` (director pay level). Then the "Weiteres" row (`0xBDA68 "$L       Weiteres:$E$R%d$Z"`, total, icon `0x150`).
So: draws = (number of listed goods) + (if the list overflowed: 15 − first unlisted index). `rules-02` §3.3 is right about the amounts (`1 + rand(3)` listed, `rand(3)` shifted for the rest); `rules-37` #12's "one rand(3) per index from the first listed good to 14" is wrong: unlisted zero-output goods inside the listed range draw nothing, and the tail loop exists only when more goods were produced than fit.

## 9. Pranger sentence and loss of office texts (`rules-18` §10) [C]
- Sentence 0 Pranger: `d.+0x02 = d.+0x02 · 9 / 10 + 1` first, then `say_box(…, T[2767] @0xA802D "$C%1N1 wird zu %w Stunden Pranger verurteilt!$N", d, 7)`: the hour count is the constant **7**; nothing else happens (no record). `rules-18` §10 row 0 is correct.
- Sentence 5: the announcement `T[2772] @0xA818A "$C%3N1 wird mit sofortiger Wirkung das Amt als %1N4 entzogen.$N"` with (d, d) is printed **before** the office is removed (`+0x11 = 15`, `+0x0F = 0` follow the `say_box` at `0x2BCEF`), so `%1N4` still names the office. `rules-18` row 5 is correct.

## 10. Blitter remap mode and the Handel map owner flags (`formats.md` §8.4/§8.5) [C]
- Sprite data is run-length coded per row: skip count, run length, run pixels; byte 0xFE ends a row, 0xFF the sprite. In the 8-bit video modes `0x13AE0(frame, x, y, sprite, flags)` draws the run pixels as: `colour = 0` → as stored; `1 ≤ colour < 0x80` → every run pixel becomes the palette index `0x83B3D[4·colour]` (solid silhouette); `colour ≥ 0x80` → every run pixel `p` becomes `byte[0x83B79 + 16·(colour & 0x7F) + p]`. **No masking and no range test on `p`**: the ramp table is indexed by the raw pixel value, and pixels ≥ 16 read past the 16-entry ramp into the following ramps (the 8-bit table is 30 × 16 = 480 bytes; after it, at `0x83D59`, come the low bytes of the 16-bit ramps, then the high bytes at `0x83F39`). So an implementer must copy the whole byte range `0x83B79..0x8411x` and index it linearly: `dst = table[16·ramp + p]` for all p, not only p < 16 and not `p & 15`. This explains the A/B: the pole pixels of ICONP6 have values ≥ 16 and were remapped in the original and left untouched in ours.
- Handel map (`0x4CC90`): for every city c = 0..9 whose rights entry `0x85E08[c]` names an owner: `item(x = city.+0x0A, y = city.+0x0C, w = 40, h = 20, style = 0x510 | colour(owner), data = 0x196)` with `colour = owner.+0xF4` (0..5). Cities without owner get a `%M` placeholder (`0xBDB40 "%M$E"`). Style 0x510 = kind 0x110 (bank item) + 0x400 (unclickable). The Söldner map (`0x5460F`) builds its seven flags the same way (`0x510 | colour`, owner from `0x85E16[k]`) [C].
- Item painter (`0x34BF0`, kind 0x110..0x11F): `bank = icon(0x196)`; draws **frame 0** (`bank + u32[bank]`, the first frame offset) at the item position with `flags = 0x80 + 5·k` where `k = style & 0xF` = the player colour. So the owner flag is ICONP6 frame 0 remapped through **ramp 5k** (ramps 0, 5, 10, 15, 20, 25 for colours 0..5), not frame k, and it is not animated: after painting, the item's style is bumped by 0x10 (kind 0x120) and flag +9 bit 1 is set, which takes it out of the 0x110 branch, so it is never repainted until the screen is rebuilt. The player-setup colour buttons (`formats.md` §8.2, style `0x110 | k`) use the same path, so they show frame 0 in ramp 5k as well. ICONPC (`0x130`, vote coin) goes through the same branch with colour 0.
