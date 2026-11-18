# New-game initial state: what a fresh save must contain

Scope: why a save from a new game must carry certain non-zero bytes that `world_init` (0x4A490) and `setup_human_player` (0x4DBC0) never write. Tags as in `rules-00-map.md`: **[C]** read at instruction level, **[L]** likely, **[UNVERIFIED]**.

## 0. Key point [C]

The city block (`0x85F04`, 0x578 bytes) and the misc-globals block (`0x85A20`, 0x4E2 bytes) sit in the **initialised data object** of MITTELL.EXE. Their start values come from the EXE image itself. No new-game code writes them; the program simply starts with them in memory. `world_init` writes only the fields listed in `rules-00-map.md` §11. Persons (`0xE3A7C`), properties (`0xDC86C`) and the record heap are BSS (zero), and `world_init`/setup fill them completely.

A port must therefore start every new game from the static image values below, then run `world_init` and setup.

Side effect: `world_init` has a single call site (0x4ADF1, in game_main before the title window opens). The only global game_main writes on new game is `0x85CBE` difficulty [C]. So if the title menu is reached again in the same program run (all players out), a new game there does not reset cities or globals, and they keep the previous game's values [L: no backward jump to before 0x4ADF1 was found; the return path to the menu was not traced].

## 1. City record +0x00..+0x0D (static, never written by code) [C]

`+0x00` u16 = index into the text pointer table `0xB4258` (1999 + c, which is `0xB4258 + 4*1999 = 0xB6194`, the city name table). Used by setup to print the start city (0x4E745) and in several other places.

Three i16 points, values from the image (identical to `formats.md` §10.1):

| c | city | P1 +2,+4 | P2 +6,+8 | P3 +0x0A,+0x0C |
|---|---|---|---|---|
| 0 | Fürstenbrück | 323,180 | 329,201 | 284,189 |
| 1 | Finsterwalde | 81,197 | 58,207 | 36,193 |
| 2 | Moortal | 119,163 | 144,145 | 134,112 |
| 3 | Lichterheide | 115,281 | 106,296 | 74,282 |
| 4 | Greifenheim | 352,122 | 410,91 | 374,84 |
| 5 | Bergstedt | 497,91 | 549,113 | 583,129 |
| 6 | Falkenhorst | 503,199 | 553,220 | 569,228 |
| 7 | Waldstein | 471,279 | 500,309 | 551,314 |
| 8 | Lauenburg | 302,334 | 346,359 | 385,364 |
| 9 | Kronsbach | 410,239 | 449,224 | 387,215 |

**Only reader: the trade map `0x4CC90`** [C for the reads; L for "only": a scan of the whole code for displacements `0x85F06..0x85F10` hits only this function, and the 8 code sites that load the base `0x85F04` into a register use offsets ≥ 0x0E]. In item creation order (`item(x, y, w, h, style, data)` = 0x34130):

1. `0x43E60` recomputes the rights table (§4).
2. For c = 0..9: if `rights[c] != 0xFF`: **flag** item at **P3**, size 40×20, style `0x510 | person[rights[c]].+0xF4` (owner's colour), data `0x196` (ICONP6). Otherwise a `%M$E` placeholder. So P3 is the owner flag position. (This corrects `formats.md` §10.1, which ties the flag to "person + 0xF4 says so"; the condition is the rights entry, and +0xF4 only gives the colour.)
3. Crossed-swords markers ICONR6/ICONR5 at `0x8703C[c]`, `0x87050[c]` (separate tables, not city record) for cities that have a type-0x19 record.
4. For c = 0..9: **town drawing** item at **P2**, style `0x100`, data `0x120 + c` (ICONI0..ICONI9), size from the icon, except c = 6 (Falkenhorst) which gets an explicit 40×50. Then **name label** item at **P1**, style `0x100`, data `0x135 + c` (ICONJ5..ICONJE), size from the icon. These are the click items (index 14 + 2c and 15 + 2c, `formats.md` §11.1). (This corrects `formats.md` §10.1, which gives P2 a 50×40 size and P3 a 40×20 `%M` item. It also corrects §11.1, where "Moortal uses 0x126" should read Falkenhorst, index 6, explicit 40×50 size, data still `0x120 + 6`. Why Falkenhorst needs the explicit size is [UNVERIFIED].) Note 2026-11-13 [C]: `ICONI6` has a nominal size of only 17×20 (`ICONIN` width/height, ICONS2 and ICONS3 alike), so the override gives a usable target. Flags (style `0x510`) carry bit `0x400` and are not clickable. For item sizes, hover frames and the click dispatch see `formats.md` §11.1.

With zeros in +2..+0x0D all 20 click items and all flags are placed at (0,0). That is the reported symptom.

## 2. Static start values the port must copy (beyond `rules-00` §6 and `rules-02` §1.3/1.4)

### 2.1 City block [C]
Image bytes of all 10 records were checked. Apart from +0x00..+0x0D above, every non-zero byte is already listed in `rules-02-economy.md` §1.3/§1.4 and "Other start values" (+0x0E pop, +0x12 wealth, +0x13..+0x21 yields, +0x6E, +0x6F=20, +0x71=25, +0x73..+0x7E = 100 each, +0x7F..+0x82 = 50, +0x83 = 15, +0x84, +0x88). +0x22..+0x6D and +0x86..+0x8B are 0 in the image.

+0x6F / +0x71 differ between saves because `prices_update` (0x49FE0, called by `world_init`) rolls them with the RNG (2 calls per city, `rules-02` §2.2). They are not static.

### 2.2 Misc-globals block `0x85A20..0x85F01` [C]
In the image the only non-zero bytes are `0x85A22..0x85A61`:

- `0x85A20..0x85A4D`: the law and county tables, all in `rules-04b` (values there match the image byte for byte). `0x85A28` u32 = 500000. `0x85A2C` = 5 in the image, overwritten by setup (`(n + 9) / n`, written at 0x4E7CF after each confirmed player).
- `0x85A4E` u16[4] = 100, 125, 125, 150 (mercenary wage per troop slot); `0x85A56` u16[4] = 75, 100, 100, 125 (robber wage). Readers: castle/camp value in net worth (0x44597, 0x44605) and wage payment in the settlement (0x61C17, 0x61C6A). Already named in `rules-02` §8, but the port must initialise them.
- `0x85A5E` u16 = 250, `0x85A60` u16 = 500. No code reference found (no displacement or pointer to either address) [UNVERIFIED: probably unused, but copy them for byte parity].
- Everything else in the block is 0 in the image. `world_init` then writes `0x85E06`, `0x85E1D`, `0x85E25[10]`, `0x85E4D[10]`, `0x85E75` and `0x85E7A[34]` (`rules-00` §11 step 7/9). game_main writes `0x85CBE`. `0x85E08[21]` is written by 0x43E60 (§4).

### 2.3 Persons and properties
No static data (BSS). Everything is in `rules-00` §11 and `rules-02` §9, plus these setup details [C, setup 0x4E5B0..0x4E734]:

- `person_init` (0x42CF0) with controller 1, then person +0x03 = religion choice, +0xF5 = hotkey, +0xF4 = colour index (0..5).
- Start branch (`property_create(0, city, p)`), order slot 0: +0x33 = 1, +0x36/+0x37/+0x38 = uncle's good/workers/means (`0x87070/7A/84[city]`), +0x3A = 2 (confirmed).
- Year > 1600 only: cash top-up (`rules-02` §9).
- **Person +0x1C = max(0, net_worth(p))** (0x44640 then 0x441F0), written for the new human after the branch and top-up. `world_init` step 8 already did this for all 192 slots, but before the human existed. So a port must recompute +0x1C for each human at the end of setup.
- Person +0x02 = 25 + rand(15) (from `person_init`). 0x20 vs 0x1E between two saves is just a different RNG draw.
- "Nein" at "Seid Ihr zufrieden?" calls `person_clear` (0x432D0) and restarts setup. The branch property is freed; consumed RNG draws stay consumed.

## 3. Order of the new-game writes (summary) [C]

program start: image data → `srand(clock)` → `world_init` → title menu → difficulty `0x85CBE` → n × setup (each: `0x43E60` rights recompute, start city, dialogs, person, branch, +0x1C, then `0x85A2C`, mission `0x17690`) → main loop saves slot 5.

## 4. Rights table `0x85E08[0..20]`: must it be 0xFF in a save? [C]

No. It is a cache. `0x43E60` sets all 21 entries to 0xFF, then for every human (controller nibble ≠ 0, index order) writes the human's index at the location of each property it owns. It is called at the start of each of its readers' flows:

| recompute call | flow | readers of `0x85E08` in that flow |
|---|---|---|
| 0x4CD64 | trade map | 0x4CD88 (flag), city info 0x4F7DB (rights check when buying) |
| 0x545DA | castle map | `0x85E16..` (locations 14..20) at 0x54602, 0x5461C |
| 0x4DC02 | setup | start-city choice 0x4DCBC |
| 0x66258 | `year_end_world` (0x66220) start | 0x20B12 mistress (computer), 0x5A427 (inside 0x59F80), 0x5B88B / 0x5B9B3 (inside 0x5B200); all called from year end (0x69C3E..0x69F02) |

These are all the code references to `0x85E08..0x85E1C` [C: pointer scan of the code object]. So the loaded value is never read before a recompute. Any value in the save works.

Byte parity: the original's new-game save holds what the **last setup's** recompute left. That covers every earlier human's properties, but not the last human's own new branch (created after the recompute). With 1 player all 21 bytes are 0xFF. With n players the earlier humans' start cities hold their person index. The image value is 0. So 0xFF only appears because setup ran 0x43E60.
