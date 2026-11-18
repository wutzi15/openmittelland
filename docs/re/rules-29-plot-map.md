# Plot map: place picker for back-room plots and privileges (rules-29)

Source: `0x45C30(player, mode)` (called `office_tree_view` in `rules-04`; the "office tree" is the person list it opens after a place is chosen). Helpers: `win` = `0x2FA80`, `print` = `0x322C0`, `draw_icon(x, y, id)` = `0x2E310`, `blit(dst, dx, dy, src, sx, sy, w, h)` = `0x13539`, `mouse(&x, &y)` = `0x12DE4` (returns 0 none, 1 left, 2 right), `menu` = `0x35B60`. Surfaces: `0xD500E` = off-screen window buffer, `0xEF4BA` = screen. Tags: **[C]** read at instruction level, **[L]** from decompiler output, **[O]** matches a live observation from the team lead, **[UNVERIFIED]** not confirmed.

The actions run after a person is picked are in `rules-04` §3 and `rules-04b` §4.3; this file covers the map, the hit test, titles and highlights.

## 1. Screen layout [C]

- Window style `0x8910` at screen (20, 40), size 600×400.
- Map `ICON25` (icon `0x25`, 480×360) drawn at window (60, 20) = **screen (80, 60)**. All tables below are in map-local coordinates; add (80, 60) for screen.
- After drawing, the clean map is copied to a 480×360 save buffer (used to erase highlights).
- Title line: drawn with `print` after setting the text row word `0xD501A = 0x20`. The strip erased/refreshed for it is map-local (0, 12)–(480, 36), i.e. screen y 72..95, full map width. `$C` centres it [L: exact glyph baseline not checked].

## 2. Hit test [C]

Mouse position (sx, sy) in screen coordinates. Rects are tested in index order 0..24; **the first match wins**. A rect matches when `x1+80 <= sx < x2+80` and `y1+60 <= sy < y2+60` (left/top inclusive, right/bottom exclusive). Tables: x1 `0x86E88`, y1 `0x86EBA`, x2 `0x86EEC`, y2 `0x86F1E` (25 × u16 each), result `0x86F50` (26 × u32, low word used; entry 25 = `0x15` = no place).

| rect | x1,y1 – x2,y2 (map) | screen | place id | name |
|---|---|---|---|---|
| 0 | 248,152 – 283,182 | 328,212 – 363,242 | 0 | Fürstenbrück |
| 1 | 44,154 – 76,184 | 124,214 – 156,244 | 1 | Finsterwalde |
| 2 | 107,106 – 140,131 | 187,166 – 220,191 | 2 | Moortal |
| 3 | 79,223 – 111,248 | 159,283 – 191,308 | 3 | Lichterheide |
| 4 | 306,68 – 341,101 | 386,128 – 421,161 | 4 | Greifenheim |
| 5 | 410,86 – 441,113 | 490,146 – 521,173 | 5 | Bergstedt |
| 6 | 409,155 – 436,189 | 489,215 – 516,249 | 6 | Falkenhorst |
| 7 | 373,230 – 415,260 | 453,290 – 495,320 | 7 | Waldstein |
| 8 | 257,270 – 286,295 | 337,330 – 366,355 | 8 | Lauenburg |
| 9 | 335,167 – 368,193 | 415,227 – 448,253 | 9 | Kronsbach |
| 10 | 364,114 – 419,168 | 444,174 – 499,228 | 14 | Hohenfels (castle) |
| 11 | 231,198 – 288,242 | 311,258 – 368,302 | 15 | Seefels (castle) |
| 12 | 199,114 – 259,159 | 279,174 – 339,219 | 16 | Waldenfels (castle) |
| 13 | 188,94 – 212,110 | 268,154 – 292,170 | 17 | Meucheltann (camp) |
| 14 | 78,188 – 112,209 | 158,248 – 192,269 | 18 | Henkerswald (camp) |
| 15 | 303,229 – 342,269 | 383,289 – 422,329 | 19 | Dunkelforst (camp) |
| 16 | 389,184 – 415,217 | 469,244 – 495,277 | 20 | Säbeltal (camp) |
| 17 | 32,67 – 171,249 | 112,127 – 251,309 | 10 | Mooringen |
| 18 | 171,67 – 225,217 | 251,127 – 305,277 | 10 | Mooringen |
| 19 | 239,65 – 449,121 | 319,125 – 529,181 | 11 | Bergenhöh |
| 20 | 369,121 – 449,223 | 449,181 – 529,283 | 11 | Bergenhöh |
| 21 | 267,180 – 369,225 | 347,240 – 449,285 | 12 | Auenthal |
| 22 | 201,225 – 449,345 | 281,285 – 529,405 | 12 | Auenthal |
| 23 | 239,147 – 289,181 | 319,207 – 369,241 | 13 | Mittelland |
| 24 | 289,128 – 365,185 | 369,188 – 445,245 | 13 | Mittelland |
| — | no match | | 0x15 | none |

Place ids and names are the location table `0xB6194` (`rules-00` "Locations"). Castle vs camp labels for 14..20 follow `rules-07` [UNVERIFIED here].

- Priority is cities, then castles/camps, then counties, then Mittelland. Counties and Mittelland are plain rectangles; the county sprites (§4) are **not** used for hit testing.
- Overlaps that matter: castle 17 (rect 13) lies inside Mooringen rect 18; city 0 overlaps Mittelland rect 23; city 9 and castle 14 overlap Mittelland rect 24; Auenthal rect 21 covers the bottom strip of Mittelland rect 24 (screen y 240..244 at x 369..444 goes to Auenthal).
- **Mittelland is selectable**: only in the leftover parts of rects 23/24 (roughly screen x 319..444, y 188..239, minus cities 0 and 9 and castle 14).
- Live check [O]: (330,240) → rect 0 Fürstenbrück; (300,150), (130,290), (120,130) → rects 18/17/17 Mooringen; (470,140) → rect 19 and (500,250) → rect 20 Bergenhöh; (460,350), (300,340), (400,380) → rect 22 Auenthal; (160,380), (215,320), (200,100) → no match. All agree with the table.

## 3. Title line [C]

`mode` (second argument) indexes the verb table `0xB673C` (13 pointers):

| mode | verb | caller → plot |
|---|---|---|
| 0 | Wählt | privileges 58 Projekte unterstützen (`0x60F30`), 85 Heiligsprechen (`0x61335`) |
| 1 | Bestechung | trade map gold item (`0x4D083`), Kontor gold display (`0x4F38B`), branch screen (`0x50FA1`) |
| 2 | Erpressung | Hinterzimmer (`0x5C3D6`) |
| 3 | Spionage | Hinterzimmer Spione (`0x5C3B4`) |
| 4 | Sabotage | Hinterzimmer (`0x5C3C5`) |
| 5 | Prozess | privilege 12 trial (`0x5FA58`) |
| 6 | Beleidigung | privilege 26 insult (`0x60091`) |
| 7 | Inquisition | privilege 81 Inquisitionsverhör (`0x612CA`) |
| 8 | Wählt | privileges 48/74/79 Arbeiter verschwinden lassen (`0x60A41`) |
| 9 | Das 1. Opfer | privilege 80 Zwietracht säen (`0x6124F`), step 1 |
| 10 | Das 2. Opfer | Zwietracht säen, step 2 (reached only from mode 9) |
| 11 | Das Opfer | Hinterzimmer Anschwärzen (`0x5C3E7`), step 1 |
| 12 | Anschwärzen | Anschwärzen, step 2 (reached only from mode 11) |

Caller-to-entry mapping for the privilege menu uses its jump table `0x5F4A8` (privilege number → case). Which Hinterzimmer button reaches each call site is inferred from the verb only [L].

Map title, redrawn only when the hit result changes (initial "previous" value is `0x16`, so the first poll always draws):
- place id < 21: `$C^ %s in %o _$N` (verb, place name) → e.g. "^ Spionage in Mooringen _".
- no place: `$C^ %s... _$N` → "^ Spionage... _".

Person-list title (after a place is clicked), each time the list is (re)shown:
- mode 12: `$H$C^ %s bei... _$N` with "Anschwärzen" (no place name).
- otherwise: `$H$C^ %s in %o _$N` (verb, place).

Two-step flows [L]: picking a person in mode 9 or 11 stores them as victim 1 and increments the mode (9→10, 11→12). The list is shown again for the same place with the new title; the player can leave it (close entry) to return to the map, where the title now reads "Das 2. Opfer in …" / "Anschwärzen in …". In mode 10/12 picking victim 1 again is ignored silently. After the second pick resolves, the mode is decremented back (10→9, 12→11) and the list reappears. The function returns only on mode 0 or 8 (the picked person) or on map cancel (0xFF); all other modes loop until the player backs out. Any pick in modes ≠ 0 that is the player themself, or an empty office, does nothing.

## 4. Hover highlight [C]

On a change of hit result from `old` to `new`:
1. Erase (from the save buffer into the window buffer): the title strip; then, if `old` < 21: if `old` is 10..13, the whole map, else the 60×54 rect at `HL[old]`.
2. Print the new title. If `new` < 21: `draw_icon(HL[new] + (60,20) window = HL[new] + (80,60) screen, 0x106 + new)`.
3. Present to screen: the first time after (re)opening, the whole window; otherwise, if neither `old` nor `new` is 10..13: the title strip plus the 60×54 rects at `HL[old]` and `HL[new]` (each if < 21); else the whole map.

Highlight origin table `HL` = x `0x86E34`, y `0x86E5E` (21 × u16). Sprite ids `0x106 + place` (`ICONG6`..`ICONHA`, naming per `formats.md` §1). Sizes are from the extracted icons.

| place | icon | HL (map) | screen | size |
|---|---|---|---|---|
| 0 Fürstenbrück | 0x106 ICONG6 | 248,152 | 328,212 | 35×30 |
| 1 Finsterwalde | 0x107 ICONG7 | 44,154 | 124,214 | 32×30 |
| 2 Moortal | 0x108 ICONG8 | 107,106 | 187,166 | 33×25 |
| 3 Lichterheide | 0x109 ICONG9 | 79,223 | 159,283 | 32×25 |
| 4 Greifenheim | 0x10A ICONGA | 306,68 | 386,128 | 35×33 |
| 5 Bergstedt | 0x10B ICONGB | 410,86 | 490,146 | 31×27 |
| 6 Falkenhorst | 0x10C ICONGC | 415,161 | 495,221 | 15×22 |
| 7 Waldstein | 0x10D ICONGD | 373,230 | 453,290 | 42×30 |
| 8 Lauenburg | 0x10E ICONGE | 257,270 | 337,330 | 29×25 |
| 9 Kronsbach | 0x10F ICONGF | 335,167 | 415,227 | 33×26 |
| 10 Mooringen | 0x110 ICONH0 | 31,65 | 111,125 | 225×190 |
| 11 Bergenhöh | 0x111 ICONH1 | 221,63 | 301,123 | 229×162 |
| 12 Auenthal | 0x112 ICONH2 | 191,167 | 271,227 | 259×179 |
| 13 Mittelland | 0x113 ICONH3 | 31,62 | 111,122 | 419×285 |
| 14 Hohenfels | 0x114 ICONH4 | 364,114 | 444,174 | 55×54 |
| 15 Seefels | 0x115 ICONH5 | 231,198 | 311,258 | 60×45 |
| 16 Waldenfels | 0x116 ICONH6 | 199,114 | 279,174 | 57×44 |
| 17 Meucheltann | 0x117 ICONH7 | 188,94 | 268,154 | 24×16 |
| 18 Henkerswald | 0x118 ICONH8 | 78,188 | 158,248 | 34×21 |
| 19 Dunkelforst | 0x119 ICONH9 | 303,229 | 383,289 | 39×40 |
| 20 Säbeltal | 0x11A ICONHA | 389,184 | 469,244 | 26×33 |

HL equals the hit-rect origin for every place except Falkenhorst (HL 415,161 vs rect 409,155). The sprite is drawn with `draw_icon`; whether transparency is keyed is per `formats.md` [UNVERIFIED here].

## 5. Input loop and person list [C unless marked]

- Wait until all mouse buttons are released, then poll. Right button → cancel (returns 0xFF). Left button on a place → open the person list; left on no place → ignored. The loop also redraws on hover while no button is pressed.
- Person list rows (`office` = row + base; holder from `0x433E0(place, 0xFF, office, 0xFF)`; names from `0xB4378` male / `0xB5854` female, index = name base + row):

| place | worldly level | church level |
|---|---|---|
| city 0..9 | 7 rows, office 0..6, names 0..6 (Ratsherr×3, Baumeister, Richter, Kämmerer, Bürgermeister) | 3 rows, office 7..9, names 7..9 (Priester×2, Domherr) |
| county 10..12 | 8 rows, office 0..7, names 10..17 (Geheimrat×3, Hauptmann, Justizberater, Finanzberater, Kanzler, Vogt) | 5 rows, office 8..12, names 18..22 (Kellermeister, Sakristan, Diakon, Abt, Bischof) |
| Mittelland 13 | 8 rows, office 0..7, names 23..30 (Hofrat×3, Marschall, Justizminister, Finanzminister, Großkanzler, Regent) | 3 rows, office 8..10, names 31..33 (Erzdiakon, Inquisitor, Erzbischof) |
| castle/camp 14..20 | 1 row "Besitzer" (name 34) = byte +2 of the property found by `0x42A50(0xFF, place, 0xFF)` | — |

- The list opens on the worldly level. For places < 14 an extra entry "Zur kirchlichen Ebene..." / "Zur weltlichen Ebene..." toggles the level; castles show `%M$E` instead [L: exact meaning of that entry not checked]. An empty office row reads "unbesetzt". The player's human rival (`0x44F10`) is drawn with the style bit `0x80` flipped (highlighted) [L].
- After an entry beyond the row count and toggle (close), the map is redrawn completely and the hover state is reset.

## Open

1. Exact title glyph position and whether `$C` centres on the window (600) or map (480); both give centre x = 320 on screen. [UNVERIFIED]
2. Which Hinterzimmer button calls which mode is taken from the verb string, not traced to the button index. [UNVERIFIED]
