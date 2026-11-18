# Data file formats

Scope: every non-FLC data file in the DOS install (`game/disk/MITTELL`, plus `install/MITTELL/CONFI2`). Parsers live in `crates/mad-assets/src/`. Each section gives the format, how it was established, and a confidence level:
- **verified**: decoder output checked against the data (sizes, checksums, visual) and/or the EXE loader code;
- **observed**: consistent with all files, no code confirmation;
- **[UNVERIFIED]**: a guess, labelled as such.

All integers are little endian. "Linear" addresses refer to `MITTELL.EXE` as loaded (see `00-loader.md`).

## 1. Icons: `ICONS2`, `ICONS2B`, `ICONS3`, `ICONS3B` (`mad_assets::sprite`, `iconin`)

### Directories and pixel formats (verified)
- `ICONS2`, `ICONS2B`: 8-bit palette indices (the palette is in §1.6).
- `ICONS3`, `ICONS3B`: 15-bit RGB555 (`0RRRRRGGGGGBBBBB`), 2 bytes per pixel. No pixel in any RLE file has bit 15 set. The installer text asks for `[EXE text]`.
- Both sets hold the same images at the **same resolution (640×480 screens)**. The 2/3 suffix is colour depth, not 640×480 versus 800×600 (`ARCHITECTURE.md` gets this wrong). The 8-bit art is a separately quantised rendition, and some pictures differ in detail (for example, the map lettering).
- `B` directories hold the large pictures. CD installs read them from the CD (§3.1). The game uses `ICONS<n>` where `n` = the video mode in CONFIG byte 0, so a user picks 8-bit or hicolor art by picking the mode.

### File naming (verified from the loader at `0x2DD70`)
Icon id `0..511` maps to `ICON<hi><lo>.DAT`, where `hi` = `id / 16` as a base-36 digit (`0-9A-Z`, at most `V`) and `lo` = `id % 16` as a hex digit. Example: `ICONR9` = 27·16+9 = 441. Bit `0x80` of the id's index flags picks the `B` directory. Special names: `ICONIN.DAT` (index), `ICONIN.BCK` (older 320-entry index, same layout, unused by the code paths seen), `ICONCS.DAT` (font).

### ICONIN.DAT: index, 4608 bytes = 512 × 9 (verified)
| offset | type | meaning |
|---|---|---|
| `0x000` | `u32[512]` | file size in bytes (0 = no such icon). Loaded to `0xC31A0`; used for memory management. |
| `0x800` | `u8[512]` | flags, loaded to `0xC27A0`. `0x80` = file is in the `B` directory (code). `0x01` = set on exactly the raw-format files (observed). `0x02` = set on the bank files, plus `ICONC6` in the 8-bit index (observed, meaning [UNVERIFIED]). |
| `0xA00` | `u16[512]` | nominal width, loaded to `0xC2DA0` (the destination order in code is size, flags, width table, height table) |
| `0xE00` | `u16[512]` | nominal height, loaded to `0xC29A0` |

Check: the `ICONS2` index matches the size and directory of all 442 files. Every single-image file decodes to a height equal to the index height and a width no greater than the index width. RLE widths are extents, and trailing transparent columns are not stored. Exception: `ICONC6`, whose index entry is 0×0.

The `ICONS3` index disagrees with 4 files. These are install defects:
- `ICONS3/ICONP6.DAT` and `ICONS3B/ICONF1.DAT` are byte-identical copies of the 8-bit files (md5 equal to `ICONS2*/`).
- `ICONS3/ICON25.DAT` is 344806 bytes, but the index says 345790. It still decodes.
- `ICONS3B/ICONG3.DAT` is truncated (617430 of 617760 bytes, in both the disk and install copies). It does not decode.

### Container 1: raw (verified)
`u16 width, u16 height, width·height pixels` (1 or 2 bytes each), row-major, all opaque. Recognised when `4 + w·h·bpp == file size`. Used by ids `0x05-0x21` (40×40 and 40×16 frame tiles) in both depths.

### Container 2: RLE sprite (verified)
Rows of segments:
```
segment := skip:u8 (0..0xFD)  count:u8  pixel[count]   ; x += skip, then count opaque pixels
0xFE    := end of row (x = 0, y += 1)
0xFF    := end of image (also ends the last row)
```
- Rows = number of `0xFE` + 1. A 1-byte file `FF` is an empty 0×1 placeholder (`ICONCD`, `ICONIA-ID`, `ICONJF`, `ICONK0-K2`).
- Skips longer than 253 are chained with `count = 0` (`FD 00 …`).
- Leading `FE` bytes and leading skips position a sprite inside a larger frame. Overlay frames in banks are therefore stored at screen coordinates.
- Example: a 640-wide opaque row is `00 FF <255 px> 00 FF <255 px> 00 82 <130 px> FE`, which is 647 bytes. A 640×480 8-bit screen is 480·647 = **310560** bytes. In 16-bit it is 480·(6+1280+1) = **617760**. Smaller full-rectangle sizes follow the same rule: 78000 = 320×240, 37440 = 192×192, 2448 = 48×48, 4288 = 64×64.
- Verification: compositing an overlay frame of bank `ICOND5` onto its 640×480 base frame is seamless. Totals: 8-bit set = 29 raw + 373 RLE + 40 bank, all decoding; 16-bit set = 29 raw + 372 RLE + 38 bank, plus the 3 defective files below.

### Container 3: bank (verified)
`u32 offset[n]` with `n = offset[0] / 4`, then `n` RLE images back to back. Image `k` spans `offset[k]..offset[k+1]` (the last one ends at end of file). Empty entries are a single `FF`. Each entry decodes to exactly its span in all 78 bank files (40 in the 8-bit set, 38 in the 16-bit set, where the other two are the defective 8-bit copies).

Detection order used by `decode_icon_file`: raw, then single RLE spanning the file, then bank. No file is ambiguous.

### 1.6 Palette (verified)
The 8-bit palette is not in any data file. It is a 768-byte table of 6-bit VGA DAC values at linear `0x8383D` in `MITTELL.EXE`. The graphics init (`0x13142`) uploads all 256 entries with INT 10h AX=1012h. Entry 0 = black, entry 255 = white. At `0x83B3D` the next tables are blitter colour tables, not a second palette (§8.4).
`mad_assets::palette::from_exe` reads it and expands 6→8 bit as `(c<<2)|(c>>4)`.
Check: across 5 icon pairs (352062 pixels), the mean absolute channel difference between palette colours and the RGB555 renditions of the same images is 14/255. With RGB565 interpretation it is 51/255. Rendered output looks correct throughout.
FLC files carry their own palettes. Whether the game reloads the EXE palette after a FLC is [UNVERIFIED].

### 1.7 Catalogue (visual identification from rendered 8-bit sheets; ids in hex, `B` = in B directory)
| ids | size | content |
|---|---|---|
| 00-04 | 32×40 | mouse pointers: arrow, arrow+plus, arrow+minus, arrow+up, arrow+down |
| 05-11 | 40×40 raw | parchment window frame tiles (corners, edges, fill) |
| 12-21 | 40×16 raw | horizontal bar tiles |
| 22 | 640×72 | parchment banner strip |
| 23, 24 / 25 | 640×480 / 480×360 | map "Circulus Germaniae" (two variants, one small) |
| 2E-30, 6A-72, 75-7B, 82-85, 8F, 90, 92, A7, AE-B6 | 32×32 | framed small action icons (coins, hand, hammer, houses, cross, …) |
| 31 | 640×480 | stone wall with ivy |
| 32-40 / 41-4F | 48×48 | trade goods icons, normal / dark (disabled) variants; 50 warehouse |
| 51-5F | 64×64 | buildings (hut → houses → manors → castles, store), ring, wagon, ship, balloon |
| 73, 74 | 15×18 | small two-state marker |
| 7C, 7E, 80; B7D, B7F, B81 | 192×192 | transport pictures (wagon, ship, balloon; steam cart, submarine, glider) |
| 87-8A, A4-A6 | 64×64 | castle, tavern sign, cannon, scales; bed+cross, house+cross, bomb |
| 8B-8E | 320×240 | castles, forest |
| A9, AB, AC, B7, B8, BC, C0-C3, C8, D4 | 640×480 | hall, Kontor rooms, desk, landscape, interiors, portals, churches, scholar, open book |
| AA, AD | 48×14, 44×36 | coin, scroll |
| CB, CC | 336×70 | seal ribbon |
| F2-FD | small | UI bits (buttons, checkboxes, 1×1 colour dots); FD 40×40 dither |
| G0, G1 | 640×480 | soldiers group |
| G6-GF, H0-HA, I0-J4, IE, IF | various | yellow outline masks (building silhouettes; H0-H3 large map regions) |
| J5-K9 | ~93-171×37 | ribbon/label outlines |
| KA-L9 | ~30×36 | small trade goods sprites (transparent) |
| MD-ND | 31×32 | wheel/compass animation (17 frames as separate files) |
| P6 | bank 6 × 33×40 | flags |
| PC | bank 14 × 16×12 | spinning coin |
| PD-PF | 640×480 | castles |
| R5, R6 | 18×18 | crossed swords |
| R9 | 599×400 | parchment with coat of arms |
| B26-B2A | 640×480 | landscape backgrounds (fields, river, ship, road) |
| B2B, B2C, BR7, BR8 | | covered wagon, gold bag |
| B2D | 480×360 | town square scene |
| B60-B69, BM0-BMC | 320×240, 160×120 | residence buildings by rank (large / small) |
| BLA-BLF | 144×144 | transport pictures (small) |
| B86, B91, B93, BA3, BA8, BB9-BD3 | 320×240 / 640×480 | scenes: wall, tavern, candle, forest, tower, dungeon, lute, woman at desk, chapels, castles, court room, graveyard, reaper, cradle, … |
| B94-BA2 | 276×240 | balance scale animation, 15 frames |
| BD5-BDC | banks of 63-105 | talking-head animation (judge/official) over a 640×480 base frame |
| BDD-BE4 | banks of 19 | fencing duel animation over a 640×480 base frame |
| BE5-BF0 | banks of 134 | character animations (~56-88 × 141-166) |
| BF1 | bank of 1485 × ~25×40 | battle unit sprites (all unit types, directions, death frames) |
| BFE | bank of 26 × ~21×11 | battle UI glyphs (letters S M K B G F, arrows) |
| BFF, BG2-BG5 | 640×480 | banquet, duel scenes, abstract map |
| BHB-BHF | various | bomb, cannon, hat, shield with swords, musket |
| BNE-BOD | 64×64 | 16 portraits |
| BOE-BP5 | banks of 23 × 64×64 | portraits with mouth/eye frames |
| BP7-BPB | 320×240 | soldier scenes |
| BQ0-BR3, BR4 | ~16-39 × 26-60 | hand gestures; coin |

Which screen uses which id needs code cross-referencing and is [UNVERIFIED] beyond the visual content.

## 2. Font: `ICONCS.DAT` (`mad_assets::font`) (verified)
| offset | type | meaning |
|---|---|---|
| `0x000` | `u8[256]` | advance width per code |
| `0x100` | `u32[256]` | glyph offset relative to `0x500` |
| `0x500` | | RLE glyphs (§1), 8- or 16-bit like the directory, 24 or 28 rows each |

43643 bytes (8-bit), 67064 bytes (16-bit). The init routine `0x2F4B0` reads it and rebases the 256 offsets. Every glyph spans exactly its offset range.
Code map (from the rendered sheet): `0x20-0x5A`, `0x61-0x7A` are ASCII. `0x5B-0x5D` = Ä Ö Ü, `0x7B-0x7D` = ä ö ü, `0x7F` = ß. `0x5E`/`0x5F` are left/right ornaments: menu titles are written `^ Title _`. `0x0D-0x0F` are radio/checkbox states, `0x10-0x19` small digits, `0x80-0x89` a slider/gauge sequence. `0x80-0xFF` repeat the set in a dark colour. Text in the EXE is CP437 (`0x81` ü, `0x84` ä, `0x94` ö, `0x8E` Ä, `0x99` Ö, `0x9A` Ü, `0xE1` ß). `font::glyph_for_cp437` maps it. Where the game does this translation is [UNVERIFIED].

## 3. Configuration and saves

### 3.1 `CONFIG/CONFIG.DAT` (default) and `CONFI2/CONFIG.DAT` (live), 633 bytes (`mad_assets::config`)
Read whole (0x279 bytes) into linear `0x86954` by `0x42030` and written back by `0x421A0`. A 0x278-byte file (older version) is accepted: bytes `0x199..` move up by one and byte `0x199` is set to 1 (code).

| offset | size | meaning | confidence |
|---|---|---|---|
| `0x000` | 1 | low bits = **video mode**, passed to the mode setter `0x12DFE` and used as the icon directory digit: 1 = VGA 320×200 (used for the intro), 2 = VESA 101h 640×480×8 → `ICONS2`, 3 = VESA 110h 640×480×15 → `ICONS3` (the default), 4/5 = VESA 111h/112h (16/32 bpp, present in the setter, no matching icon sets). In modes 2/3, `ICONS?B` is read from the CD path unless byte `0x26C` (mode 2) / `0x26D` (mode 3) is set; those bytes record that the B icons were copied to disk | code |
| `0x000 + 0x30·k`, k=0..4 | 48 | byte 0 (k=0: the install-type byte above), bytes 1..: NUL-terminated name, `"^ Leer _"` when empty | observed; save-slot labels [UNVERIFIED] |
| `0x0F0` | 4 | `00 00 00 02` | observed |
| `0x0F4` | 1 | last saved slot (`FF` = none; the live sample says `05` = autosave). Save writes it; "Das letzte Spiel fortsetzen" loads `game<[0xF4]>.dat` and is offered only when it is < 6 | code (re-rules) |
| `0x0F5` | `u32[34]` | 5000 … 500000. Byte-identical to the static EXE table at linear `0x86BD0` (just past the 0x279-byte config image). That table holds the office wealth targets (index 0 = Ratsherr … 33 = Erzbischof), which world init copies into the save globals (re-rules, `rules-03-career.md`). No absolute code reference into config `0xF5..0x17C` was found, so this copy looks like an unread template. An access relative to the config base cannot be ruled out | code search |
| `0x17D` | 4 | `FF FF FF FF` | observed |
| `0x181` | | `02`, `u16 0x330` (MIDI port?), `07`, `u16 0x220` (SB port?), `u32 44100` (sample rate?), `02 00 02 00 00 00 10 00 00 7F 7F 00 03 08 01` | sound setup, field meanings [UNVERIFIED] |
| `0x199` | 1 | inserted byte (set to 1 when upgrading an old file) | code |
| `0x19A` | ≤ 0xC8 | CD path, e.g. `E:\MITTELL` | code |
| `0x262` | 1 | CD path length (the game appends `\`) | code |
| `0x267-0x269`, `0x26E`, `0x26F` | 1 each | option flags, each passed to an option setter after loading | code, meanings [UNVERIFIED] |
| `0x26C`, `0x26D` | 1 each | "B icons available locally" for install types 2/3 | code |
| `0x184` | 1 | if 0, the intro plays without sampled effects and uses MIDI song 3 instead (main menu code `0x4ABD0`); default `07` | code, meaning partly [UNVERIFIED] |
| `0x273` | 1 | intro choice: 0 → `INTRO.FLC` drawn at (0,24), else `INTROB.FLC` at (96,100). Default 0 | code |
| `0x274` | `u32` | compared with 800000 (`0xC3500`): at or below it, the intro uses the simple path (song 3, no synced samples). Default 2500000. A memory/speed measure [UNVERIFIED] | code |
| `0x278` | 1 | copied to `0x8457C` after loading | code |

The default and the live `CONFIG.DAT` differ in one byte (`0x0F4`: `FF` vs `05`).

### 3.2 `CONFI2/GAMEn.DAT`, 79703 bytes (`mad_assets::save`) (layout verified, contents mostly not)
The file name is built from `confi2\game0.dat` with the slot digit at position 11. Load (`0x42200`) and save (`0x42360`) read/write 12 memory blocks in this order, with no header and no checksum. The sizes sum exactly to the file size.

| file offset | length | memory | notes |
|---|---|---|---|
| `0x00000` | `0x300` | `0xE377C` | 32 × 24-byte NUL-padded names (the sample has player names) |
| `0x00300` | `0xBA00` | `0xE3A7C` | persons: 192 × 248 (`0xF8`) bytes; after loading, the game runs a per-person fix-up (`0x42C40`) over all 192 |
| `0x0BD00` | `0x4800` | `0xDC86C` | properties: 192 × 96 bytes (re-rules) |
| `0x10500` | `0x2710` | `0xE106C` | record heap, 10000 bytes (below) |
| `0x12C10` | 2 | `0xEF4B8` | year (sample: 1712) (re-rules) |
| `0x12C12` | 1 | `0xEF4C6` | current person (re-rules) |
| `0x12C13` | 1 | `0xEF4C5` | phase (re-rules) |
| `0x12C14` | 4 | `0x85A18` | heap bytes free (re-rules) |
| `0x12C18` | 4 | `0x85A1C` | heap bytes used (re-rules) |
| `0x12C1C` | `0x578` | `0x85F04` | cities: 10 × 140 bytes (re-rules) |
| `0x13194` | `0xE1` | `0x86532` | goods table: 15 × 15 bytes (re-rules) |
| `0x13275` | `0x4E2` | `0x85A20` | miscellaneous globals (re-rules) |

Record heap (re-rules): variable-length records. Byte 0 = type (0..57, bit 7 = deleted), byte 1 = owner person, byte 2 = key, byte 3 = years-left counter. The size of each type comes from the table at `0x85980`.
Slot 5 is the autosave ("letztes Spiel"), written at the start of each year (re-rules). The RNG state (`0xB9284`) is **not** saved, so a reloaded game does not repeat the original random sequence.
The block meanings come from re-rules' code analysis; I have not checked them independently. Field-level meanings: `rules-00-map.md`.

## 4. `SZENARIO.DAT`, 6900 bytes (`mad_assets::szenario`) (layout verified from code, codes partly)
Loaded whole to `0xD9D00` by `0x3E2A0`. Indexing in the battle set-up (`0x3DDA0`): `scenario·0x564 + row·0x1E + col`. The file is therefore **5 battlefields × 46 rows × 30 columns of one byte each**. These are not 2-byte cells; the repeating `02 02` / `01 01` are pairs of equal bytes.
- The low 5 bits of a cell are compared against a deployment code (`table[unit_type] + 6·side`). The set-up picks the matching free cell that maximises a distance score, then places a line of units from it.
- The high 3 bits (`cell & 0xE0`) give a direction 0..7, used to step along the line through two 8-entry `i16` direction tables (`0x84F2C`, `0x84F3E`).
- Observed low-5-bit codes: `0-6` occur only with facing 0. `2` frames every layout (border), `0` and `1` are the most common open cells, and `3-6` are scattered. Codes `7-0x13` (no `0x0D`) occur with several facings, which fits deployment slots whose 6-code stride matches the `+ 6·side` term: `7-0x0C` for one side, `0x0D-0x13` for the other ([UNVERIFIED] split). Terrain meaning of `0-6` is [UNVERIFIED].

## 5. Sound
### 5.1 `SMP22KHZ/SMPL_xyz.DAT`, `SMP44KHZ/SMPL_xyz.DAT` (`mad_assets::smpl`)
- Name built from `smpxxkhz\smpl_xxx.dat` (code `0x11E61`). The directory digit is `4` if the configured rate ≥ 44000 (`[0x8019E] >= 0xABE0`), else `2`. `xy` = the sound number as two hex digits, `z` = a variant digit from a separate table. 203 files (22 kHz) and 205 (44 kHz; the extras are `SMPL_560`, `SMPL_570`).
- Header: 20 bytes = `u32 h[5]`, then unsigned 8-bit mono PCM to end of file (verified: the 22 and 44 kHz versions have equal durations within 1%, and the data centres on 0x80).
- The loader reads the header to `0x804F8` and adds the PCM base address to `h[0]..h[3]`, so they are offsets into the PCM. `h[3] − h[0]` becomes the transfer length. `h[4]` is used only when bit 0 of `[0x801A0]` is set (code).
- Observed: `h[0]=h[1]=0`, `h[2]` = 0 or a loop start, `h[3]=h[4]`. The values are about half the PCM length in both rate directories (for example 22 kHz `SMPL_2D0`: 15361 PCM bytes, `h[3]=0x1E00`). The exception is `SMPL_2F0`, where `h[3]` equals the PCM length and `h[4]=0`. Units and loop semantics are [UNVERIFIED]. `Sample::to_wav` plays the whole payload.
- Mapping of sound numbers to game events: [UNVERIFIED], not analysed.
### 5.2 `SPC22KHZ`, `SPC44KHZ` speech
1416 standard RIFF WAVs each (8-bit mono 22050 / 44100 Hz), playable as-is. Names are 7 characters: digit- or letter-prefixed groups (`0xxxxxx`, `1xxxxxx`, `2xxxxxx`, `022Axxx`, `022Bxxx`, `A…`–`H…`, for example `H123014`). The path prefix is `spc22khz\` / `spc44khz\` plus `.wav`. How names map to texts/events is [UNVERIFIED].
### 5.3 `MIDSONGS/SONGx.MID`
24 Standard MIDI Files, `x` = `1-9`, `A-O`. Name template `midsongs\song?.mid` (linear `0x86C59`). The game has its own MIDI drivers (LAPC-I/MPU and SB16 code paths, `lapc_prepare`/`sb16_prepare` messages). Which song plays where is [UNVERIFIED].

## 6. Strings in the EXE
See `strings.md`.

## 7. Start-up, main menu and new-game screens (code `0x4AB90`)
Coordinates are pixels on the 640×480 screen. "(code)" items come from the call-site arguments. The drawing primitives are `win_open(x, y, w, h, style)` = `0x2FA80`, `draw_icon(x, y, id)` = `0x2E310` (x/y relative to the current window) and `print(fmt, …)` = `0x322C0`.

### 7.1 Sequence (code)
1. **Intro.** The game first switches to video mode 1 (VGA 320×200). Byte `0x273` of `CONFIG.DAT` picks the film: 0 (the default) → `FLCFILME/INTRO.FLC` at logical (0,24), otherwise `INTROB.FLC` at logical (96,100). All drawing coordinates are logical 640×480. In modes 0/1 the blitter and FLC player map x → x/2 and y → table[y] (480-entry table at `0x835D6`, about y·200/480; 24 → 10, 100 → 41). So INTRO.FLC (320×180) lands 1:1 at (0,10) in 320×200, i.e. vertically centred and **not scaled**. INTROB (also 320×180) would land at (48,41), which overflows 320 px; why is [UNVERIFIED].
   - Full path, when the `u32` at config `0x274` is above 800000 and config byte `0x184` is non-zero (true for the defaults): 13 speech files are preloaded into slots 0-12 (§8.1), then the film plays in 24 segments synced to them (§8.1).
   - Simple path otherwise: MIDI `SONG3.MID` (song 3, mode 0), then the film plays straight through.
2. Graphics, icon index and font are (re)initialised.
3. **Main screen:**
   - `win_open(0,0,640,480, 0x8100)` opens the full-screen window.
   - `draw_icon(0, 40, 0xD4)` draws **`ICOND4`**, the 640×480 open book. It is drawn at y = 40 (code). Its bottom 40 rows wrap to frame rows 0..39 (§10.10).
   - Music: **`SONG4.MID`** (song 4, mode 1 = keep playing if already running). Song number n → `SONG<"0123456789abcdef…"[n]>.MID`; numbers `0x19-0x1F` choose randomly from small tables at `0x86C6C…0x86C82`.
   - Each time the menu is shown: `draw_icon(0, 1, 0x22)` draws **`ICON22`**, the 640×72 parchment banner, then `print("$H$Z$C^ Mittelland AD _$N")` writes the title, which presumably lands on the banner ([UNVERIFIED] that `$H` targets it).
4. **Menu box:** `win_open(140, 140, 360, 200, 0x8810)`, a framed window (§7.2), shows `"^ Spielstart _"` and the items "Ein neues Spiel beginnen", "Einen Spielstand laden" (only if some CONFIG slot name is not `^ Leer _`) and "Das letzte Spiel fortsetzen" (only if config `0xF4` < 6). Unavailable items print `%M$E` instead. The `0x1030` argument passed with every `%m` item is an item style [UNVERIFIED]. If no saves exist and there is no last game, the box is skipped and "new game" is chosen directly.
5. **New game:**
   - Redraw the banner (`ICON22` at (0,1)) and print the title `"^ Ein neues Spiel beginnen _"`.
   - Choice dialog `0x368E0` at (140,100), size 360×280: `[EXE text]` with Leicht / Mittel / Schwer / Keine Aufträge. The result is stored at `0x85CBE`.
   - Then (140,140), size 360×200: `"^ Spielerzahl _ … Wieviele Spieler sollen mitspielen?"` with 1-6. Player creation (`0x4DBC0`) runs once per player.
6. Choice 1 → load dialog (`0x4D530`); choice 2 → load `game<[0xF4]>.dat`.
- Mouse pointer: ids `0x00-0x04` (arrow, +, −, up, down) are loaded at icon init together with the frame tiles (ids `0x00-0x21` are preloaded). Which pointer the menu uses: [UNVERIFIED], probably 0.
- Text colour: the font has a gold half (codes `0x00-0x7F`) and a dark half (`0x80-0xFF`). Which one menus use, and whether the `$` codes switch between them, is [UNVERIFIED]. Rendering parchment windows with the dark half and the banner title with gold matches the art, but that is a guess.

### 7.2 Framed window (code `0x30880`)
Windows are tiled with the 40×40 raw tiles `ICON05-ICON11`. The tile code is `id | transform << 8`. The transform byte takes values 1-4; from its use (left vs right, top vs bottom) it is most likely 1 = as stored, 2 = flipped vertically, 3 = flipped horizontally, 4 = rotated 180° ([UNVERIFIED]; the blitter `0x13AE0` was not traced). `wx`/`wy` = window position, `x`/`y` = tile position inside the window, all in pixels:

| cell | tile id | transform |
|---|---|---|
| top-left | `0x06 + (wx/40) % 4` | 1 |
| top edge | `0x0E + (x/40) % 4` | 1 |
| top-right | `0x06 + (wy/40) % 4` | 3 |
| left edge | `0x0A + (y/40) % 4` | 1 |
| interior | `0x05` (plain parchment) | 1 |
| right edge | `0x0A + (y/40) % 4` | 3 |
| bottom-left | `0x06 + (wx/40 + 2) % 4` | 2 |
| bottom edge | `0x0E + (x/40) % 4` | 2 |
| bottom-right | `0x06 + (wy/40 + 2) % 4` | 4 |

Partial tiles at the right and bottom edges are clipped to the window size. A scrolling variant (window flag `0x20`) draws only left edge, interior and right edge.
Horizontal bars (`0x30670`) are drawn into two 16-pixel strips (probably the top and bottom rolls of a scroll, [UNVERIFIED]). `k = (offset % 16) / 2`. Left cap = `0x12 + k`, middle tiles every 40 px = `0x1A + k`, right cap = `0x12 + k` with transform 3.

## 8. Follow-ups for mad-app (code)

### 8.1 Intro sound sync (`0x4ACCA`, preload `0x15100`)
- **Preload.** `0x15100(slot)` builds a speech path, not an SMPL path: `<base>spc22khz\` or `spc44khz\` (44k when the rate word at config `0x187` ≥ 44000), then four digit characters from its four byte arguments (all 0 here), the slot as three decimal digits, then `.wav`. Slots 0-12 therefore load **`SPCxxKHZ/0000000.WAV` … `0000012.WAV`**. The volume/priority global `0x8457C` is 160 for slots 0-11 and 100 for slot 12.
- **Playback.** For segment `i` = 0..23: let `v = table_0x8700C[i]`, slot = `v % 100`, then start the sample with play mode 3 if `v < 100`, mode 1 if `v < 200`, mode 7 otherwise (the mode is passed to `0x12698`; meaning [UNVERIFIED], plausibly a flags/channel value). Then play the FLC with `flc_play(x, y, delay=10, until_frame = table_0x86FDC[i])`. Esc aborts.
  - `0x8700C` = 4, 3, 3, 3, 5, 12, 111, 9, 6, 7, 0, 8, 8, 8, 8, 0, 0, 102, 110, 210, 1, 3, 211, 202
  - `0x86FDC` = 72, 88, 109, 135, 173, 1020, 1069, 1092, 1118, 1143, 1150, 1159, 1165, 1171, 1182, 1241, 1242, 1295, 1389, 1408, 1510, 1580, 1581, 0
  - The last entry 0 means play to the end (INTRO.FLC has 1590 frames). The frame counter is cumulative across segments.
- **Frame timing (`0x14ED6`).** It waits on the DOS clock until `(frames_shown + 1) · delay` hundredths of a second have elapsed since the first frame. With delay 10 that is **10 fps**, regardless of the FLC header speed.

### 8.2 Player creation (`0x4DBC0`, once per player)
1. Choice dialog at (140,140), 360 wide (if the player is not human-created by default; a 320-wide variant at (160,160) is also used, branch not traced).
2. Full-screen `win_open(0,0,640,480, 0x8910)`. `ICOND4` at (0,40), `ICON22` at (0,1), music `SONG4.MID`.
3. Text: `$|$H$Z$CEin neuer Spieler$Z`, then `$|$H$Z$C^ Spieler Nr. %d _$Z`.
4. **Left column** (`$t$|$4H$10>$56<`: line 4, left margin 60, right margin 336):
   - `^ Name _`: text input. Names are stored in save block `0xE377C` (24 bytes each).
   - `^ Geschlecht _`: `%m` items Männlich / Weiblich (style `0x1030`).
   - `^ Glaube _`: Evangelisch / Katholisch.
   - `^ Farbe _`: six 40×40 buttons in a row, centred `(text_width − n·40) / 2`. Each button is created with style `0x110 | k` (k = 0..5) and icon `0x196` = **ICONP6**, the flag bank (§8.5).
5. **Right column** (`$|$4H$56>$10<`): `^ Hotkey _` followed by a menu of hotkey letters (`- %c -`).
6. Inheritance message (`[EXE text]`) and `Seid Ihr zufrieden? %m %m` (Ja / Nein).

### 8.3 Text layout (`print` = `0x322C0`)
- **Inset.** A window opened with `$H`/`$P` gets left and right margins of 12 px (text width = window width − 24) and starts at y = 12. Text is **not** vertically centred: `$nH` puts the cursor on line `n` (y = 12 + n·24).
- **Line height** = 24 px (initialised at `0x2FBB5`; this equals the 24-row glyphs).
- **Escape syntax.** `$` + optional decimal number `n` + code letter. The table below lists every code the parser handles. "Flush" means the pending line is emitted first.

| code | effect |
|---|---|
| `$$` | literal `$` |
| `$N` | new line |
| `$Z` | half-height line break (line height temporarily halved, then new line) |
| `$Y` | half-height break followed by a full new line |
| `$L` `$R` `$C` `$B` | alignment left / right / centred / justified (`B` = Blocksatz; justified is the window default) |
| `$E` | flush the current line without a line break, so text after it continues on the same y. Used as `left$E$Rright` for two-column rows (inferred from usage) |
| `$H` (`$nH`) | home: margins 12, cursor to line n |
| `$P` | new page: reset margins and cursor, redraw the window frame (§7.2) |
| `$n>` / `$n<` / `$n=` | left margin = n·6 px / right margin = n·6 px from the right / width = n·6 px (6 = line height ÷ 4) |
| `$k/m` | column k of m (m defaults to 2): column width = (w − 24)/m, x = 12 + k·width |
| `$\|` | shorthand for `$2>$2<` (12 px extra margin each side) |
| `$h` | toggle highlight (flag `0x80`) |
| `$t` / `$g` | set / clear flag `0x40` |
| `$d` / `$n` | set / clear flag `0x20` (`$n` without a digit prefix differs from `$N`; case matters) |
| `$(` / `$)` | set / clear flag `0x08` (keep-together block [UNVERIFIED]) |
| `$W`, `$V` | wait for click (`$V` only if not at the top of the window) |
| `$X` | new line + new line |
| `$s` | set item spacing to n |
| `$a` | expands to the fixed string at `0xBB594` (left-aligned label with a hanging indent) |

  Which colour each flag selects is [UNVERIFIED] (§8.4). `%m` creates a clickable menu item. `%M` is a placeholder for an unavailable item.
- **Word spacing.** At font load (`0x2F7C8`) the game overrides the advance of code `0x20` (space) to **5 px**; the file's value of 1 is not used. It also copies the advances of codes `0xA2`, `0xA3`, `0xAA`, `0xAF`, `0xBD`, `0xC0`, `0xE0` over those of `0x22`, `0x23`, `0x2A`, `0x2F`, `0x3D`, `0x40`, `0x60`. Justified lines stretch the spaces (`$B`).

### 8.4 Blitter colour modes (`0x13AE0`)
The draw flags word passed with every sprite is `transform << 8 | colour`.
- `colour = 0`: draw pixels as stored.
- `colour = 1..0x7F`: draw the sprite as a **solid silhouette** in colour `k`. The colour table at `0x83B3D` has 4 bytes per entry (`u8` palette index, `u16` RGB555, pad), 15 entries: 
  `26/6000 25/60A5 20/7D4A 39/3604 30/6FE9 91/10B5 8B/255F 43/7320 40/7FE0 01/7FF0 BA/7FFF E5/3521 45/6A63 05/7F70 BB/739C`.
  Format per entry: palette index, then the RGB555 value as a u16 at entry+1. Which colour belongs to which player is [UNVERIFIED].
- `colour ≥ 0x80`: per-pixel remap through 16-entry ramp `k & 0x7F` (8-bit: table at `0x83B79`; 16-bit: low/high bytes at `0x83D59`/`0x83F39`). Source pixel values index the ramp, so this applies only to sprites drawn with small index values. 30 ramps exist; which sprites use them is [UNVERIFIED].

### 8.5 Player colours (resolved in §11.4)
The colour picker shows ICONP6 (6-frame gold flag) on six buttons tagged `k` = 0..5. Nothing in the files is pre-tinted. The likely mechanism is the blitter's solid or remap modes (§8.4), keyed by the player's colour, but the call that draws a player's flag/marker was not traced. Until it is, render ICONP6 frame `k` tinted with a per-player colour of your choice and mark it as a placeholder.

### 8.6 Glyph spacing on lines with items
Code: line painter `0x31610`, glyph append `0x30FB0`, `print` `0x322C0`, item create `0x34130`, string width `0x33F60`, item painter `0x34BF0`, number writer `0x32050`. `sp` = window word `+0x2E`; `adv[c]` = font advance (`0xD4A38[c]`) after the font-load overrides of §8.3.
- **One spacing value for everything.** `win_open` sets `sp = 1`. The only other writer is `$ns`: it flushes the pending line, then sets `sp = n` (`$s` alone gives 0). §8.3's table is wrong here: `$s` sets the extra glyph spacing, not item spacing. In the shipped strings only the office tree uses it (`$0s … $1s`, `0x86C94..0x86CF4`). No per-line, per-item or per-window-style switch exists. Every glyph advances `adv[c] + sp`. A space advances `adv[0x20] + sp` = 6. Line width (`0xD6A30`) adds the same amounts.
- **Items are not glyphs in the line.** `%m` / `%n` / `%M` create an item record and put one marker in the line buffer. When the line is measured, the item counts as its stored **width `w`** (record `+4`). The painter advances x by `w` too, then stores the item's final x. The item's own text is drawn later (`0x34BF0`, on flip) left-aligned at that x. It uses the `sp` captured when the item was created (record `+0x0E`), so in the law book that is 1 px. The window state is saved and restored around it.
- **Centring.** `x = left + (text_width − L) / 2`, truncated, where `L` = Σ(`adv+sp`) of the line's glyphs and spaces + Σ item `w`. The last glyph's `sp` and any trailing space are included. Right alignment uses `text_width − L`.
- **Item width `w`** (`n` = style & 0xF; class = style & 0x1F0; integer division of the whole product):
  - class 0x00/0x10/0x20 (u8/u16/u32 number): `w = ((n−1)·(adv['.']+sp)) / 3 + n·(adv['0']+sp)`
  - class 0xC0 (signed): same + `adv['+'] + sp`
  - class 0x40/0x50/0x60: same formula with `adv[0x1F]` and `adv[0x10]` (the alternate digit set written by `0x32130`); 0xD0 adds `adv['+']+sp`
  - class 0x30/0xA0/0xF0 with `n = 0`: string width = Σ(`adv[c]+sp`) of `data` (0xB0: of `*data`). `~` counts 0. Umlauts are remapped as in §8.3.
  - class 0x80, or 0x30/0xA0/0xB0/0xF0 with `n > 0`: `sp + 2n·(adv['M']+sp)`
  - class 0x90/0x100: `W[0xC2DA0][data]`
- **Number text.** Writer `0x32050(value, n)`. With `n > 0` it writes exactly `n` digits, **zero-padded**, with `.` before every remaining group of 3 (7 digits: `0.050.000`). Style bit `0x400` forces `n = 0`. Then it writes only the significant digits (0 → `0`) and the same separators. The text is left-aligned in `w` and never stretched. Text after the item starts at `item_x + w`. So the gap after a number is `w − drawn_width`. Even a full-width 2- or 3-digit value gets `((n−1)(adv['.']+1))/3` px of phantom separator slot.
- **Law book (`0x48690`).** Style flags are 0 when read-only and `0x1400` when the player can edit. `0x400` means no zero padding. `0x1000` means no background buffer. The items are: Umsatzsteuer `0x02` (u8, n=2), Arbeiterlohn / Söldnerlohn `0x03` (n=3), Talerhöchstgrenze `0x27` (u32, n=7), Faktoreien `0x02`. The yes/no lines (`Kredite sind %m` etc.) use `0x1030` (string, width = text width).
- **What explains the A/B.** [UNVERIFIED] None of this code makes an item line tighter than a plain line. All of it uses `adv+sp` with `sp = 1`. What does differ on item lines is the following: `L` uses the reserved `w`, not the drawn number, so the centring offset moves by `(w − drawn)/2`; there is an extra `w − drawn` gap before `%`; and read-only values are zero-padded. Check these against the capture before you assume a per-glyph change.

### 8.7 Menu hover highlight (`menu` `0x35B60`) [C]
- **Entry**: `menu` paints all items once (`0x34BF0` at `0x35B85`), before any hover test, so they are drawn unlit.
- **Every poll**: the item under the pointer is found (first hit in creation order, `0x35C10..0x35CA5`; kind-0xE0 items pass the hit to the item before them, `0x35D00..0x35D26`). Its style word gets 0x4000 set and 0x8000 cleared (`0x35D28..0x35D37`). This only changes the record. Nothing is drawn at this point.
- **Repaint rule** (`0x365A0..0x365DC`): the screen is repainted **only when the hovered index differs from the global `0x84D4C`** (the last hovered index; initial value 160). With no item under the pointer, the index is the item count. When it differs:
  1. clear 0x40|0x80 in byte +9 of item `0x84D4C`, if that index is still below the current item count (un-light);
  2. repaint all items (`0x34BF0`), so the new item shows lit;
  3. set `0x84D4C` := the new index, and turn the repeat timer off.
- `0x84D4C` is written nowhere else (only `0x365D6`). It survives window changes and `menu` returns. So when a new window's item under the resting pointer has the **same item index** as the last hovered item of the previous window, its 0x4000 flag is set but no repaint happens. The item stays unlit until the pointer moves onto another item or onto empty space (index change), or until some other code repaints the items. If the index differs, it lights on the first poll, i.e. immediately and without waiting for movement.
- The only release wait is at entry, and only when the pointer flag `0x84D68` is 0 (`rules-54` §4). It comes before the first poll, so the first highlight appears after the buttons are released.
- **Un-light**: only by the index change above. The item under the pointer changes, or the pointer goes to empty space (index = count). There is no timeout.
- Answer to the live cases: in the office-tree → "Anschwärzen bei..." case, Benno's new item evidently has the same index as the previously hovered item (both trees rebuild their item list from the same base), so it is not lit until the pointer moves [L: index equality inferred from the observation]. In the tree-under-a-message-box case, the box's OK/line items were hovered last. Back in the tree the index differs, so the hovered tree item lights at once.

## 9. In-game screens (code)
Same conventions as §7: `item(x, y, w, h, style, data)` = `0x34130` creates a clickable item, and `menu` = `0x35B60` waits and returns the clicked item's index.

### 9.1 Banner (`0x49D60(player, title, no_redraw)`), drawn on every turn screen
- `ICON22` at (1,0) unless `no_redraw`.
- Text: `$t$|$H$3<$R%s, A.D. %l$|$C$(%1N1, %1N4$)$g$d`, which on line 0 gives:
  - right-aligned `<title>, A.D. <year>` (title = screen name such as "Kontor" or "Erträge"; year from `0xEF4B8`);
  - centred `<name>, <office/rank>` of the current player (`%1N1`, `%1N4`; person-name codes, see `strings.md`).
- **Gold display:** item at (18,13), size 220×24, style `0x200B0`. Its text is `$t$L%i Taler$N`, or `$t$L$h%i Taler$h$N` (highlighted) when cash < 0. Cash is the `i32` at person + `0x18` (`0xE3A94 + p·0xF8`).
- **Banner itself:** item style `0x1290`, data = icon `0x22` (whole banner clickable).
- Then `%M%M$E` adds two invisible placeholder items.
- Returned item indices: 0 = gold display, 1 = banner, 2, 3 = placeholders. The function returns the base index for the screen's own items.
- For the "Ereignisse" (player = `0xFF`) variant the text is `[EXE text]` with no gold display.

### 9.2 Kontor (`0x5D610`, turn phase 4)
- Full-screen window `0x8100`.
- Music: with 25% chance per entry, song `0x15` or `0x13` (50/50), mode 2.
- Background: **`ICONAB`** (Kontor room, 640×480) at (0,0). Banner with title "Kontor".
- Then nine label + hotspot pairs. Each label is gold text (item style `0x404F0`, size 0) at the label position. The hotspot (style `0x12E0`) is the clickable rectangle.

| idx | label (pos) | hotspot x,y,w,h | action |
|---|---|---|---|
| 4 | Söldner & Räuber (364,178) | 337,204,54,74 | `0x54570`, phase 2 |
| 6 | Schreibstube (502,442) | 311,377,300,100 | `0x5C450` |
| 8 | Spielstand (200,176) | 161,200,78,50 | `0x5DC60` (save/options) |
| 10 | Bezahlung (201,283) | 143,307,112,78 | `0x57E70`, phase 3 |
| 12 | Hinterzimmer (86,254) | 39,254,84,70 | `0x5C140` |
| 14 | Handel (64,366) | 0,350,280,130 | `0x4CC90`, phase 1 |
| 16 | Kirche, or Kircheneintritt if confession byte (person + 3) = 2 (488,169) | 449,192,77,108 | church menu `0x5CCA0`; if confessionless, a choice dialog at (160,140,320,200) to pick a confession |
| 18 | Geld zum Fenster hinauswerfen (464,128) | 576,180,62,154 | confirm dialog at (100,140,440,200) "Achtung! Wollt Ihr aufgeben?"; yes → the player retires (`0x42F60`, `0x432D0`), phase 6 |

- **Index mapping.** The switch only has even cases, which matches the label indices above. Whether a hotspot click returns its own index (odd, which would hit the `default` case = phase 5, end of Kontor) or its label's index is [UNVERIFIED]. The game is presumably playable by clicking the room objects, so hotspots likely report the paired label's index.
- Index 0 (gold display) → `0x4F370`; index 1 (banner) → `0x4D1E0`. Index 3 or ≥ 20 → "end turn".
- **End turn.** If config flag `0x17D` bit `0x20` is set, ask `[EXE text]` at (160,160,320,160). Otherwise, with bit `0x10` set and a flag at `0x85CC0`, ask `[EXE text]` at (140,160,360,160). "No" returns to the Kontor.
- The submenu functions are listed by address. Their layouts are not analysed yet.

### 9.3 Turn start (`0x4B900`, phase 0)
- **"Nächster Spieler"** is shown only when config byte `0x17E` bit 0 is set (`0x86AD2`): a full-screen window style `0x8910` with **`ICONBB`** (lute still life) at (0,0). It prints `$8H$Z$C$gNächster Spieler:$Z%1N1, %1N4$N` (line 8, centred), waits for a click (`0x36720`), and closes.
- **Report screen:** full-screen window `0x8100`, `ICOND4` (book) at (0,40), banner titled "Erträge".
  - Left page (`$t$|$4H$10>$56<`, i.e. line 4, margins 60 | 336): `^ Produktion _`. One row per produced good: `$L       %s:$E$R%d$Z` (good name left, amount right, half-line spacing). The goods sprite **`0x14A + good`** (`ICONKA`…, §1.7) is drawn at (text x, line y − 9).
    - With nothing produced it shows `[EXE text]`
    - "Weiteres" uses icon `0x150`.
  - Right page (`$|$4H$56>$10<`): `^ Exporte _` with `[EXE text]` or `[EXE text]`. It may add `[EXE text]`.
  - **Paging:** none. Both pages are one book spread; the screen waits for a click (`0x36720`) and closes.
  - Further phase-0 effects (record heap events) follow but are rules (re-rules).

### 9.4 Load dialog (`0x4D530`)
- Window at (20,140), size 600×200, style `0x8810`.
- Text: `$Z$C^ Spiel Laden _$Z$C`, then for each of the 5 slots `%d.%m$N` with the slot number and its name from CONFIG (`0x00 + 0x30·k + 1`, `"^ Leer _"` if empty). Item style `0x103D`; empty slots are disabled (`0x348D0`).
- A click loads `game<k>.dat` (`0x42200`).

## 10. Kontor sub-screens (code)
Conventions as in §7 and §9:
- `ICON(x, y, id)` positions are relative to the current window.
- Each labelled hotspot pairs a gold label item (style `0x404F0`, size 0) with a click rectangle (style `0x12E0`).
- Most screens open a full-screen window with style `0x8500` and draw the banner (§9.1) with the given title.

### 10.1 Handel (trade map) `0x4CC90`
- Background: **`ICON23`** (map, 640×480) at (0,0). Banner "Handel", drawn without redrawing ICON22. Music song `0x19` (random pick from a 9-entry table at `0x86C6C`), mode 2.
- Cities: 10 records of 140 bytes at `0x85F04` (save block "cities"), in name order Fürstenbrück, Finsterwalde, Moortal, Lichterheide, Greifenheim, Bergstedt, Falkenhorst, Waldstein, Lauenburg, Kronsbach (string list at `0x9DC4E`). Each record holds three `i16` point pairs at +2, +6, +10:

| city | P1 (+2) | P2 (+6) | P3 (+10) |
|---|---|---|---|
| Fürstenbrück | 323,180 | 329,201 | 284,189 |
| Finsterwalde | 81,197 | 58,207 | 36,193 |
| Moortal | 119,163 | 144,145 | 134,112 |
| Lichterheide | 115,281 | 106,296 | 74,282 |
| Greifenheim | 352,122 | 410,91 | 374,84 |
| Bergstedt | 497,91 | 549,113 | 583,129 |
| Falkenhorst | 503,199 | 553,220 | 569,228 |
| Waldstein | 471,279 | 500,309 | 551,314 |
| Lauenburg | 302,334 | 346,359 | 385,364 |
| Kronsbach | 410,239 | 449,224 | 387,215 |

  These match the town drawings on ICON23. Per city the screen creates:
  - a 40×20 item at P3, filled with a `%M` placeholder (only when a per-player byte at person + `0xF4` says so; [UNVERIFIED] meaning, probably "branch present");
  - an item of style `0x100` and size 50×40 at P2 (the click target);
  - a size-0 item at P1 (the name label).
  The crossed-swords markers `ICONR6` (`0x1B6`) and `ICONR5` (`0x1B5`) are drawn at computed positions in a loop. They are most likely route or occupation markers ([UNVERIFIED]).
- Branch screen, storage and transporter menus: reached from the city items, not yet traced ([UNVERIFIED]).

### 10.2 Schreibstube `0x5C450`
- `ICONC8` (scholar at desk) at (0,28). Banner "Schreibstube". Music song `0x0F`, mode 2.
- Hotspots (label pos → rect x,y,w,h):

| label | label pos | rect |
|---|---|---|
| Bewerbung | 277,324 | 200,357,150,120 |
| Geldleiher | 95,290 | 1,300,170,140 |
| Gesetze | 580,230 | 500,250,138,130 |
| Privilegien | 438,332 | 360,350,200,128 |
| Kreditbuch | 90,150 | 0,80,200,200 |
| Direktoren | 550,150 | 440,80,200,200 |

- **Bewerbung** without citizenship shows `[EXE text]`.
- **Kreditbuch:** `ICOND4` (book) at (0,40), banner "Kreditbuch". Left page (`$t$|$4H$10>$56<`): one entry per loan, `[EXE text]` (the year is highlighted with `$h` when overdue). Right page (`$|$4H$56>$10<`): `[EXE text]` when there are no loans.
- **Direktoren:** window at (40, y) width 560, style `0x8810`. Title `$Z$C^ Eure Direktoren _$Z`, one row per director `$L%1N0 in %o:$E$R%m Taler`, footer `[EXE text]`. With no directors it shows `[EXE text]`.

### 10.3 Bezahlung (wages) `0x57E70`
- `ICONA9` (hall, 640×480) at (0,0). Banner "Löhne". Music song `0x19`, mode 2.
- Text rows use per-row format strings from the table at `0xB6FA0`. Before printing, the text cursor is set directly (cursor y = 90, line width 140), and a second block starts at y = 408.
- Coin sprites **`ICONAA`** (48×14) are drawn at y = 386 with a computed x, one stack per wage row.
- Exact row formulas: [UNVERIFIED].

### 10.4 Spielstand `0x5DC60`
- Window at (40, y), width 560, height 240 (style `0x8810`). Title `$C^ Spielstand _$Z`.
- One row per player: `%a%1N0$E$36>%*$E$R%d$|` (name, a bar `%*` from x = 216, score right-aligned).
- Clicking a row opens a choice at (160,…) or (140,…) showing "%1N0 ist zur Zeit %1N4." or `[EXE text]`.

### 10.5 Hinterzimmer `0x5C140`
- `ICONAC` (fireplace room) at (0,0). Banner "Hinterzimmer". Music song `0x10` (mode 0 or 2).

| label | label pos | rect |
|---|---|---|
| Spione | 425,188 | 401,225,46,128 |
| Sabotage | 52,181 | 21,216,60,35 |
| Erpressung | 147,402 | 1,416,368,63 |
| Anschwärzen | 207,289 | 153,317,92,68 |
| Kontor (back) | 530,218 | 483,187,90,215 |

### 10.6 Kirche (church square) `0x5CCA0(p)`
Read at instruction level [C] unless marked. Reached from the Kontor only while confession byte (p + 3) ≠ 2 (§9.2 idx 16).
- Full-screen window `0x8500`; `sound(0x0F, 2)`. Background **`ICONC2`** (village church) if (p + 3) = 0 (evangelisch), else **`ICONC3`** (twin-tower church, katholisch), at (0,52). Banner `0x49D60(p, "Kirche", 0)`, base B.
- Four label + hotspot pairs. **The item list does not depend on the picture or the confession**; only the icon id differs. Labels: style `0x404F0` (gold text, size 0, centred on x), hotspots: style `0x12E0`. Created in this order:

| idx | label | label pos | hotspot x,y,w,h |
|---|---|---|---|
| 4 / 5 | Hochzeitsglocken | 318,108 | 250,130,145,111 |
| 6 / 7 | Konvertieren | 540,380 | 460,325,180,155 |
| 8 / 9 | Austreten | 100,380 | 0,325,180,155 |
| 10 / 11 | Kirchgang | 318,331 | 242,315,152,141 |

- `menu(B, 0, &v)` (`0x35B60`). A hotspot click reports its label's index (rules-26 §0). Dispatch on `i = result − B` (jump table `0x5CE44`, 11 entries): 0 gold → `0x4F370(p)`; 1 banner → options `0x4D1E0`, result r; 2, 3, 5, 7, 9 and i > 10 (right click returns count + 1) → leave. 4, 6, 8, 10 below.
- Loop: after a case, if i = 3, i ≥ 12, phase byte `0xEF4C5` ≠ 4, r ≠ 0 or the leave flag is set, the church closes (`0x49EB0`, `win_close`) and returns r. Otherwise, if the case set "redraw" the whole screen is rebuilt (window, picture, banner, items), else `menu` is called again on the same items.

**Hochzeitsglocken (i = 4).**
1. If spouse byte (p + 4) ≠ `0xFF`: message box `(120, 180, 400, 120)` `[EXE text]` (`0xC0034`, p + 0 = 0, male) or "…einen Geliebten." (`0xC0074`). No redraw.
2. Else: `0x49EB0`, matchmaker `0x20D30(p)` (rules-05 §2, rules-15 §7.2), then redraw. Every path of `0x20D30` shows something and waits for a click:
   - p owns a type-30 record (courtship): message box `(160, 100, 320, 280)` `0x9D96C` `[EXE text]`; answer 0 deletes the courtship.
   - proposal counter `0x85E07` ≥ 2: Kupplerin picture `0xBD`, caption menu `(40, 400, 560, 80)` `0xBA95C` `[EXE text]`, `wait_click`.
   - empty candidate list: picture `0xBD`, caption `(0, 390, 640, 90, 0)` `0x9DB01` `[EXE text]` (waits for a click). The counter is not incremented.
   - otherwise `0x85E07 += 1` and the offer of rules-05 §2 step 6.
   There is **no age test, no turn flag, no office or branch test, and no silent return** in `0x5CCA0` case 4 or in `0x20D30`. A click that reaches case 4 always changes the screen.
   - Candidate filter quirk: the "slot used" test reads the **human's** byte 0, not the candidate's. A free slot (byte 0 = `0xFF`) therefore passes the gender test; zeroed free slots (`person_clear`, rules-11 §6) still fail the spouse test (+4 = 0). A free slot that was never zeroed [UNVERIFIED].

**Konvertieren (i = 6).** fee = `clamp0(net_worth(p)) / 30`. Message box `(160, 140, 320, 200)` `0xA8527` `[EXE text]` with (fee, `0xB6E14[(p+3) xor 1]`: 0 "evangelisch", 1 "katholisch"); answers "Amen!" / "Um Gottes Willen, nein!". Answer 0: `pay(p, fee)` (`0x43860`, no cash check), (p + 3) ^= 1, turn flag byte `0x85CBC |= 0x10`, redraw (the picture switches C2 ↔ C3). Answer 1: nothing, no redraw.

**Austreten (i = 8).** fee = `clamp0(net_worth(p)) / 10`. Message box `(140, 120, 360, 240)` `0xAB732` `[EXE text]` with (`0xB70C0[(p+0x10) & 0x0F]`, fee); prefix 0 `[EXE text]`, 1 `[EXE text]`; answers "Jawohl!" / "Niemals.". Answer 1: nothing. Answer 0:
1. `pay(p, fee)`; (p + 3) := 2.
2. If (p + 0x10) & 0x0F ≠ 0: (p + 0x0F) := 0, (p + 0x11) := 15, (p + 0x10) &= 0xF0.
3. Every type-5 record of p with +5 > 7, or +5 = 7 and +4 < 10, is deleted.
4. The **first** type-30 record of p only: message box `(160, 180, 320, 120)` `0xC00B4` `[EXE text]` with (record + 9), `favour_decrease(p, record + 9, 20)` (`0x436F0`), record deleted.
5. Leave flag set: the church closes.

**Kirchgang (i = 10).** `0x49EB0`, then `r = 0x5D1F0(p)` (rules-08 §4), redraw; r ≠ 0 closes the church and is returned. No preconditions in `0x5CCA0`.

**Live report: click on Hochzeitsglocken does nothing** (one human, Bergstedt, no office, unmarried, picture ICONC2, hover at (322,185) lights the label, two clicks 10 s apart, no reaction). The code above has no branch that explains this: the point is inside hotspot 5, no earlier item of the screen overlaps it (gold display (18,13,220,24); banner item = ICON22 at (1,0), assumed to end above y = 52 where the picture starts [UNVERIFIED]), the hover highlight proves `menu` resolved it to label 4, and a left click on a resolved item returns at once (`0x36640`..`0x366E8`). Possible explanations, none confirmed [UNVERIFIED]:
- `mouse` (`0x12DE4`) is INT 33h function 3, a **poll** of the current button state. A press and release that both fall between two polls are lost; the poll loop redraws all items when the hovered item changes (`0x365CF`), so a click sent right after moving the pointer is the most exposed. The second click 10 s later would not be exposed in this way.
- Buttons value 3 (both) or 2 counts as a right click: `menu` returns count + 1 and the church closes (visible, so not this report).
- The screen did change and the observation missed it. Re-check with a screenshot about 1 s after the click.

### 10.7 Söldner & Räuber `0x54570`
- Background **`ICON24`** (map variant) at (0,0). Banner "Söldner & Räuber", drawn without redrawing ICON22.
- 7 camp records of 26 bytes at `0x8647E` (three `i16` point pairs, then 7 more words, the last being an id `2014+k`). Points:
  (396,172 / 494,155 / 539,143), (347,260 / 311,266 / 349,242), (301,148 / 279,147 / 285,139), (223,109 / 251,125 / 234,98), (144,234 / 103,250 / 135,227), (376,288 / 403,306 / 449,307), (447,261 / 516,246 / 505,227).
- Items per camp follow the same pattern as §10.1: a 40×20 `%M` item at the third point, size-0 items at the second and first.

### 10.8 Spieloptionen (banner click, `0x4D1E0`)
- Window (40,20,560,440), style `0x8810`. **`ICON2D`** (town square, 480×360) at window-relative (40,50). Title `$C^ Spieloptionen _$N`.

| label | label pos | rect |
|---|---|---|
| Laden | 443,149 | 420,161,42,42 |
| Speichern | 97,152 | 64,157,56,59 |
| Spiel beenden | 169,240 | 140,174,56,61 |
| Neuer Spieler | 358,378 | 316,264,98,98 |
| Musik | 180,371 | 158,250,74,98 |
| Grafik | 348,182 | 326,180,64,69 |
| Optionen | 255,159 | 212,179,88,72 |

- "Neuer Spieler" asks `[EXE text]` (Ja/Nein) at (160,…).

### 10.9 Battle (`0x3EE60`)
- **Set-up.** Battle type `t` (0-5, from the caller) selects:
  - the background icon `0x857FE[t]` = `0x26, 0x2A, 0x29, 0x28, 0x27` → **`ICONB26`, `ICONB2A`, `ICONB29`, `ICONB28`, `ICONB27`** for t = 0-4 (the 640×480 landscapes; t = 5 reads past the table, [UNVERIFIED]);
  - the SZENARIO layout `0x85803[t]` = 0, 1, 1, 2, 3, 4.
- The field is a **720×560** window (style `0x8100`) that is larger than the screen. A 640×480 off-screen buffer is allocated and the background is blitted into it.
- 720 = 30 columns × 24 px, and 46 rows fit into 560 at about 12 px. Cell geometry is [UNVERIFIED].
- Per cell, the low 5 bits index two 20-entry tables:
  - `0x85809` (occupancy init: `FF` free, `FE`/`FB` blocked);
  - `0x8581D` (terrain class 0/1/2).
- **Music:** song `0x1B` (random from 3 at `0x86C75`), mode 4 at start; song `0x1C` (random from 4 at `0x86C78`), mode 5 at the end.
- **Pre-battle dialogs:** message boxes at (40,180,560,160) / (40,160,560,160) (`0x36C90`) and a window at (120,120,400,240) (negotiation: "…bietet Euch…", "Wollt Ihr…").
- **Unit sprites:** the 1485-frame bank `ICONBF1` and the glyph bank `ICONBFE` (§1.7). The drawing code and the tactic menu are **not yet traced**.
- **After-battle report:** window (80,80,480,320), style `0x8810`, title `^ Nach einer langen Schlacht... _`. Paged with `$W$P` (wait for click, new page). Sections:
  - `^ Gewinne... _` (rows `$L%1N0$E$30<$R%d%I%I$E$2<%d%I$|`);
  - `^ Beschädigungen... _`;
  - `^ Verluste... _` (rows `$L%1N0$E$R$38<%d%I$E$26<%d%I$E$14<%d%I$E$2<%d%I$|`);
  - `^ Bezahlung... _`;
  - then castle/insurance messages. `%I` is an inline icon code ([UNVERIFIED] which icons).

### 10.10 Pictures drawn below y = 0 wrap around the frame (what shows under the banner)
Source [C]: `draw_icon` `0x2E310`, blitter `0x13AE0` off-screen branch `0x13DC5..0x14018`; call sites enumerated over every `0x2E310` call.
- `draw_icon(x, y, id)` passes the start row `(window origin y + y + scroll) mod surface height` to the blitter. Nothing else is clipped.
- **Off-screen frame (every window style with 0x8000):** the blitter sets a row counter to `frame height − y`. When a row end decrements it to 0, the write pointer jumps back to **row 0 of the frame** at the same x, and the picture continues from there. A 640×480 picture drawn at (0, y) into a 480-row frame therefore fills the whole frame: picture rows `0..479−y` land at frame rows `y..479`, picture rows `480−y..479` land at frame rows `0..y−1`. Every pixel path in the off-screen branch (plain, silhouette, remap, 8/16/32-bit) has the same reset.
- **Screen destination (`0xEF4BA`):** the counter is `480 − y` and the blit stops when it reaches 0. Clipped, no wrap. None of the screens below draw straight to the screen.
- Consequence: the picture's own bottom strip sits at the top of the screen and shows through the transparent pixels of the banner `ICON22` (§9.1, drawn after the picture). The Kontor frame borrowed by style `0x8500` (rules-16b §2) is fully overwritten. There is no clear, fill, fade, second picture or transition. Full-screen pictures are assumed to have no transparent runs [UNVERIFIED]; if one had, the borrowed Kontor frame would show there.
- Checked against the team lead's captures at (300, 0): `ICONC8` row 452 = (41,24,24) = Schreibstube capture; `ICONC2` row 428 = (8,16,0) = Kirche capture. `ICONAB` row 0 = (66,33,16) = Kontor capture.

| screen | function | window | picture at | rows 0..y−1 behind the banner |
|---|---|---|---|---|
| Kontor | `0x5D610` | `0x8100` | `ICONAB` (0,0) | `ICONAB` rows 0.. (no wrap) |
| Schreibstube | `0x5C450` (draw `0x5C4B3`) | `0x8500` | `ICONC8` (0,28) | `ICONC8` rows 452..479 |
| Kreditbuch (Schreibstube) | `0x5C450` (draw `0x5C785`) | same frame | `ICOND4` (0,40) | `ICOND4` rows 440..479 |
| Kirche | `0x5CCA0` (draw `0x5CD1F`) | `0x8500` | `ICONC2`/`ICONC3` (0,52) | `ICONC2`/`ICONC3` rows 428..479 |
| Kirchgang | `0x5D1F0` (draw `0x5D244`) | `0x8500` | `ICONBC` (0,60) | `ICONBC` rows 420..479 |
| Erträge report | `0x4B900` (draw `0x4BA83`) | `0x8100` | `ICOND4` (0,40) | `ICOND4` rows 440..479 |
| Player setup | `0x4DBC0` (draw `0x4DD30`) | `0x8910` | `ICOND4` (0,40) | `ICOND4` rows 440..479 |
| Auction settlement | `0x27680` (`0x27A84`, `0x27BEF`, `0x27D03`) | `0x8910` | `ICOND4` (0,40) | `ICOND4` rows 440..479 |
| Bezahlung (Löhne) | `0x57E70` | `0x8500` | `ICONA9` (0,0) | `ICONA9` rows 0.. (no wrap) |
| Hinterzimmer | `0x5C140` | `0x8500` | `ICONAC` (0,0) | `ICONAC` rows 0.. |
| Handel | `0x4CC90` | `0x8500` | `ICON23` (0,0) | `ICON23` rows 0.. |
| Söldner & Räuber | `0x54570` | — | `ICON24` (0,0) | `ICON24` rows 0.. (§10.7) |
| castle/camp pictures | `0x549C0`, `0x56080`, `0x56760`, `0x56CF0` | — | 320×240 at (80,102) / (60,92) | no wrap (bottom edge ≤ 342); they do not reach the banner |

- The "Bezahlung ICOND4 at (0,40)" in the A/B request does not match the code: Bezahlung draws `ICONA9` at (0,0). The ICOND4 screens are Erträge, Kreditbuch, player setup and auction settlement. `ICOND4` rows 440..441 at x 300 are (8,0,0), near black, which fits "black or identical".
- This also resolves the [UNVERIFIED] note in §8.2 step 3 (ICOND4 at y = 40 wraps).

## 11. Branch screen, battle sprites, tactic menu (code)

### 11.1 City click on the trade map (`0x4CC90`)
- The banner gold item → `0x45C30`; the banner → Spieloptionen (`0x4D1E0`); index 2 → back to the Kontor.
- City items come in pairs from index 14: even indices are the 50×40 targets, odd ones the labels.
  - **Odd index, or even index without a branch there:** **city info** `0x4F3C0`.
  - **Even index on a city where the player has a branch** (`0x42A50` finds it): the **branch screen** `0x4FA70`.
- City item data is `0x120 + city` (icon id; Moortal uses `0x126`); label data is `0x135 + city`.
- Cities with a branch get a flag item with style `0x510 | colour`, data `0x196` (`ICONP6`, team-tinted; §11.3).
- **Re-check 2026-11-13 [C]** (0x4CC90 dispatch `0x4D0AA..0x4D16C`, `menu` 0x35B60, item create 0x34130, painter 0x34BF0). The rule above holds. A live report said a label click on a branch city "opens nothing"; the code has no such path.
  - Index `n = menu result − base` (base = first item of the banner, 0x49D60). 0 → `0x45C30(player, 1)`; 1 → options `0x4D1E0`; 2 → back to the Kontor (`0xEF4C5 = 4`); 3 → leave the screen; 4..13 = the ten flags/`%M` placeholders; 14..33 = towns; ≥ 34 (right click returns count + 1) → back to the Kontor.
  - Flags (`0x510`) and `%M` placeholders (`0x1630`) carry style bit `0x400`. `menu` skips such items in its hit test, so they can never be clicked.
  - City `c = (n − 14) / 2`, rounded toward zero:

    | item | branch of this player at c (`0x42A50(0, c, player)`) | result |
    |---|---|---|
    | drawing (even n) | yes | branch screen `0x4FA70(rec)`; then `0xEF4B6 = base`, `0x85CC0 = 0`, pointer `0x2EC40(0)`, map redrawn |
    | drawing (even n) | no | city info `0x4F3C0(player, c)` |
    | label (odd n) | either | city info `0x4F3C0(player, c)`; no branch lookup |

    `0x4F3C0` always opens its window (80,20,480,440) and waits in `menu`. It returns 1 after a Lager was opened or closed, and then the map is redrawn. Otherwise the window closes and `menu` resumes on the old map without a redraw.
  - Hit test (0x35C2A): items are walked from `base` upward in creation order; the **first** item whose rect holds the pointer wins. The rect is top-left (x, y) plus (w, h) from 0x34130 (no centring for style `0x100`). The drawing is created before its label, so it wins where the two overlap.
  - **Rects** (style `0x100`, size = nominal `ICONIN` width/height): label `0x135 + c` at P1, drawing `0x120 + c` at P2 (rules-30 §1). Falkenhorst's drawing is an explicit 40×50, because `ICONI6` is only 17×20.

    | c | label w×h | drawing w×h |
    |---|---|---|
    | 0 | 171×38 | 48×41 |
    | 1 | 145×40 | 42×38 |
    | 2 | 93×37 | 45×29 |
    | 3 | 118×35 | 38×35 |
    | 4 | 141×41 | 38×42 |
    | 5 | 103×44 | 33×37 |
    | 6 | 111×38 | 40×50 (explicit) |
    | 7 | 125×41 | 51×39 |
    | 8 | 105×37 | 32×30 |
    | 9 | 105×37 | 37×30 |

    Bergstedt: label (497,91)–(599,134), drawing (549,113)–(581,149). The live points (520,100) and (340,190) hit only the label. (560,125) is in both, so the drawing wins.
  - **Orange frame = hover feedback**, which has nothing to do with branches or clicks. Kind `0x100` items draw nothing while idle (painter `0x353F9`). `menu` marks the item under the pointer with flag `0x4000` (`0x35D28`) and repaints. The painter then draws that item's icon at its rect (`0x35985..0x35AA3`). `ICONI0..I9` and `ICONJ5..JE` are outline sprites (yellow/orange border on transparent; `ICONJA` checked with `icon_dump`). The frame shows on hover before any click.
  - Click acceptance (0x365A0..0x36705): `menu` polls the **current** button state (0x12DE4, no latched presses). It returns on the first poll that sees the left button down over a hittable item (sound `0x42`). A press is ignored only when the button is already down at menu entry and is still held before `0x84D58` ticks have passed (auto-repeat). A short click that starts and ends between two polls is lost, for example during the repaint that follows a hover change. The live "frame but nothing" result fits a click lost this way [UNVERIFIED: retest with the pointer resting on the label first, then a press of ≥ 100 ms; expected result: city info for Bergstedt].
  - **Re-check 2026-11-16 [C]** (`0x4CF29..0x4D006` creation, `0x4D02C..0x4D1DD` loop). **How to open the city info of an own-branch city:** click its **label** (P1 rect) anywhere outside the drawing's rect. Odd n → `0x4F3C0(player, c)` (`0x4D14C..0x4D167`), and there is no ownership test on this path. The same rule applies to every city, whatever its owner: own branch, no branch, or a branch of another human. The drawing (even n) is the only item that tests for a branch, and only for *this* player's branch (`0x42A50(0, c, player)`, `0x4D0D2..0x4D0FB`). Another human's branch counts as "no branch" and gives city info.
    - Creation order per city c (second loop, after the ten flag/`%M` items): drawing `item(P2, 0, 0, 0x100, 0x120 + c)` (Falkenhorst 40×50), then label `item(P1, 0, 0, 0x100, 0x135 + c)`. So n = 14 + 2c is the drawing and 15 + 2c the label.
    - The flags at P3 (style `0x510 | colour`, ICONP6) belong to the person `0x85E08[c]` (`0x4CD86`; drawn when ≠ 0xFF). They are not branch markers, and they are not clickable (bit 0x400).
    - Right click: `menu` returns count + 1 = 34 → `0xEF4C5 := 4` (Kontor, `0x4D171`). No key is handled. `menu` has no timeout here, and keys reach only text-field items, of which there are none.
    - Loop (`0x4D178..0x4D19F`): after a city info that returns 0, the screen goes back to `menu` only while phase `0xEF4C5` = 1 and no options result is pending. Otherwise the map is rebuilt (`0x4CD30`, after a branch screen or a Lager change) or left.
    - Each `menu` call is wrapped by `0x2DCA0`/`0x2DD10` (INT 1Ch flag-animation handler, `rules-39` §1), which does not consume input.
    - The live "Bergstedt label (548,113), outline shown, nothing after 12 s" contradicts this path. (548,113) is inside the label rect (497..599, 91..134) and 1 px left of the drawing rect (x ≥ 549). The likely cause is the lost-click case above, where the press falls into the hover repaint [UNVERIFIED]. Retest: move the pointer onto the label, wait ≥ 0.5 s, then hold the button ≥ 150 ms.

### 11.2 City info `0x4F3C0`
- Window (80,20,480,440), style `0x8810` (own frame, so surface origin is (0,0)). Text area: left margin 12, width 456. `C` = city record `0x85F04 + 0x8C·c`. All coordinates below are window-relative; add (80,20) for the screen.
- **Alignment never changes.** The first string sets `$C`. Nothing later sets `$L`/`$R`/`$B`, and a line flush does not reset alignment (§8.3). So **every line is centred**: `x0 = 12 + (456 − L)/2`, where `L` is the full line width per §8.6. That includes item widths and **trailing spaces**: a space is a buffer entry worth `adv[0x20] + sp` = 6, and it is added to `L` even at the end of the line.
- **Print sequence** (`print` calls in order; `y` = text-cursor top of the line; glyphs are drawn at `y`):

| # | string | args | line y (normal / besieged) |
|---|---|---|---|
| 1 | `$Z$C^ Information %s _$N$N` | `T[C+0]` (text id → `0xB4258`) | 24 (leading `$Z` on an empty buffer advances 12) |
| 2 | `Einwohner: %d$Z` | `C+0x0E` (i32) | 72 |
| 3 | `Reichtum: $g` | none | 108 |
| 4 | `$14c ` × n, n = `C+0x12`/10 + 1 (1..10) | none | same line |
| 5 | `$t$Z` | none | ends the line |
| 6a | not besieged: `Steuer: %d%%       Lohnniveau: %s$Z` | `C+0x83`, `T[0xB6CEC][C+0x7F / 21]` = sehr niedrig / niedrig / normal / hoch / sehr hoch | 144 |
| 6b | besieged: `$hBelagerung durch:$h$N%3N5$Z` (this **replaces** the Steuer line) | `rec+1` of the first type-`0x19` record (`rec_find(0x19,FF,FF)` then `rec_next`) whose byte +4 = c | 144 and 168 |
| 7 | `$h` (only if a disaster record exists) | | |
| 8 | `Katastrophe: ` | | 180 / 204 |
| 9 | `$h` (same condition) | | |
| 10 | `T[0xB7054][k]` | `u16 rec+6` (year; when there is no record, the arg is read through a null pointer and ignored by `%v`) | same line |
| 11 | `$N$N$Z` | | |
| 12 | cursor y += 9; `Hauptproduktion: `; cursor y −= 9 | | 240 / 264 |
| 13 | `%j ` × 3 | `0x14A + P[k]` | same line |
| 14 | `$Z` | | |
| 15 | cursor y += 9; `Begehrte Güter:    ` (4 trailing spaces); cursor y −= 9 | | 276 / 300 |
| 16 | `%j ` × 3 | `0x14A + D[k]` | same line |
| 17 | `$N$N$Z` | | |
| 18 | one Lager line (below) | | 336 / 360 |

  - Line advance: `$N` = 24. `$Z` after text = 24 + 12. `$N$N$Z` after text = 24 + 24 + 12.
- **(1) Wealth rings.** `$14c` appends font glyph code 14 (`$<n>c` = glyph n, handler at `0x32A20`). Each ring is that glyph plus one space: advance `adv[14] + sp` = 21 + 1 = 22, then 6 for the space, so **28 px per ring**. Nothing else sits between rings, and there is no `$ns`. The `$g` before the rings clears text flag `0x40`, so the rings use the outlined style. Glyphs without flag `0x40` are drawn 8 times at the (±1, ±1) neighbours from glyph table `0xC43A0`, then once at the centre from `0xC41A0`. The label and everything after `$t` use the plain single draw. The advance is unchanged. The line is `L = W("Reichtum: ") + 28·n` = 119 + 28n with the shipped font (sp = 1, label sum computed from `ICONCS.DAT`). The last ring's trailing space is included, so it shifts the line 3 px left.
- **Disaster.** `rec = rec_find(2, 0xFF, c)` (first type-2 record keyed by the city, rules-06). `k = rec ? rec[+4] + 1 : 0`. The table at `0xB7054` holds: 0 `keine%v`, 1 `der Brand von %l`, 2 `die Sturmflut von %l`, 3 `die Pest von %l`, 4 `die Lawine von %l`, where `%l` is the year without separators. When a record exists, only the label `Katastrophe: ` is highlighted (`$h` on, label, `$h` off). The disaster word itself is never highlighted.
- **(2) Goods icons.** Groups of five goods, k = 0..2, cover g = 5k … 5k+4.
  - `P[k]` (Hauptproduktion) = the g with the largest yield `C+0x13+g`.
  - `D[k]` (Begehrte Güter) = the g with the largest `C[+0x22+g]·100 / V[g]` (integer division; `V[g]` = goods-table byte 12 = 7 + g, at `0x8653E + 15g`).
  - Both use a strict `>` against a running best that starts at 0, so ties keep the lower g. If nothing beats 0, the default is g = 5k.
  - `%j` creates an item at (0,0) with style `0x1090` and data = icon id. Its width and height come from the ICONIN tables: **36 × 36** for `0x14A..0x158` in both icon sets. The item puts a kind-2 marker in the line.
  - At flush the painter places the item at the current pen x and at `y − 6` (kind-2 markers lower the cursor by 6 only while the item is placed). The pen then advances by the item width, 36. The following space adds 6, so the icons are **42 px apart**. The icon is drawn at the item's (x, y), top-left (`0x2E310`).
  - Centring uses `L = W(label) + 3·(36 + 6)`, which includes the trailing space after the third icon. Label widths: `Hauptproduktion: ` = 202, `Begehrte Güter:    ` = 192.
  - Derived from the formulas above [UNVERIFIED against a capture]: Haupt `x0` = 76, icons at x = 278 / 320 / 362. Begehrte `x0` = 81, icons at x = 273 / 315 / 357. Icon top = line y − 6 (234 and 270 when not besieged).
  - **The ±9 on cursor y (`0xD501A`) around the labels has no visible effect** by the code reading. The label glyphs are only buffered (`0x30FB0` never reads cursor y). The line is painted at the later `$Z`, after the −9 has been applied. So the label is drawn at the line's own y, like every other line [UNVERIFIED live].
- **(4) Lager line.** It is centred like the rest. Which variant appears:
  - `B` = the player's branch in c (`0x42A50(0,c,player)`).
  - **No B:** `P = 0x42870(0,c,player)`.
    - If `P` exists and `0x85E08[c]` = `0xFF`: `[EXE text]` with (icon `0x2F`, price `0x441F0(0x44340(P))`).
    - Otherwise: `[EXE text]`.
  - **B exists:**
    - If `0x42B10(0x42A50(0,0xFF,player))` is non-zero: `[EXE text]` with (icon `0x30`, refund = price·2/3) [UNVERIFIED: that `0x42B10` means "more than one branch"].
    - Otherwise: `[EXE text]`.
  - Geometry:
    - `%j` icon (32 × 32, `ICON2F`/`ICON30`): item width 32, top-left at (pen x, **line y − 6**).
    - `%M` (style `0x1630`, empty string): width 0, kind-1 marker, no y shift. So in `%M%j` the icon starts at `x0`.
    - The text glyphs that follow start at icon x + 32. The space before "Lager" is a normal 6 px space.
  - Menu result 6 = the open icon, 7 = the close icon. The `%M` items exist only to keep the item indices aligned.
- Confirmations are choice dialogs at x = 140: `[EXE text]`, `[EXE text]`, `[EXE text]`.

### 11.3 Branch (Lager) screen `0x4FA70`
- Full-screen window `0x8500`. Background icon at (0,0) is computed; the value seen at the call site is `0x31` (`ICON31`, stone wall) [UNVERIFIED for other cases]. Banner titled with the city name (table `0xB6194`).
- **Goods grid.** 16 slots.
  - Slot 0 is the warehouse (`ICON50`, style `0x1090`).
  - Slots 1-6 hold goods with a city entry ≥ 0x65 ([UNVERIFIED] meaning: produced here).
  - Slots 7+ hold other goods with a nonzero stock.
  - **Positions:**
    - first 7 slots: x = 80 + 80·k;
    - further slots: centred with 64 px spacing, x = 64·k − 104 − (n·64 − 464)/2;
    - y: 176 when there are only 7 slots, else 125 for the first row and 227 for the second.
  - **Each good:**
    - icon `0x32 + good` (normal) or `0x41 + good` (dark, when the player cannot yet trade it), style `0x41090`;
    - a stock-number item (style `0x40054`);
    - an amount line `$g$d$C%B` in an 80 px box at (x − 39, y − 41);
    - a price-trend marker at (x − 18, y − 48), chosen from 10 preloaded sprites by `(value − 50)/10`, clamped 0..9.
- **Bottom row (y = 408):**
  - building icon `0x51 + level` at x = 38 (`ICON51`…, the branch building);
  - `ICON5B` at 138;
  - a gold/value text at (170,412) (data `0xEF4B4`);
  - `ICON5C` at 238;
  - three transporter buttons `0x5D + k` (`ICON5D` wagon, `ICON5E` ship, `ICON5F` balloon) at x = 338 + 100·k, with their name item at y = 412 and count item at y = 456 (style `0x40442`; `0x1630` placeholder when the type is not owned).
- **Transport panel.** A window (0,278,640,120), style `0x8010`, holds the order list, saved and restored behind the goods grid. Rows by order kind:
  - none: `Kein Auftrag`;
  - one-way: `$78<$R%m%M %m$E$|$30>$Lmit %m %s$E$|$62>$7=$C%s$E$|$70>$L%m%M %s…`;
  - route: `%m %m$E$30>$14=$C%m$E$|$44>$Lauf %m %m in %m…` / `…mit %m %m n. %m…`, plus `%J` variants;
  - buy: `$78<$R%m%M%M$E$|$30>$Lmit %m %m aus %m`.
  Each panel ends with `Auslastung: %d%%` and `Kosten: %d Taler`.
- Sub-actions dispatched from this screen: `0x52240`, `0x53A60`, `0x531A0`, `0x53680`, `0x53990` (transporter purchase, loading and route set-up). Their layouts are not traced; impl-app can reuse the panel above.

### 11.4 Battle unit sprites (`ICONBF1`, drawing in `0x389C0`)
- **Grid cell = 24 × 12 px** (code: x = col·24, y = row·12; mouse to cell = ((mx+40)/24, (my+40)/12)). The 30×46 field is 720 × 552 px, drawn into the 720×560 window with a 40 px offset (the visible 640×480 view starts at field pixel (40,40)).
- **Unit record.** 100 units × 33 bytes at `0xD7DCC`:

| off | type | field |
|---|---|---|
| +0 / +2 | i16 | pixel x / y |
| +4 / +6 | i16 | col / row |
| +8 / +10 | i16 | target col / row |
| +12 | u8 | strength |
| +14 | u8 | unit type (0-21) |
| +15 | u8 | group |
| +16 | u8 | side |
| +17 | u8 | owner |
| +18 | u8 | action |
| +19 | u8 | facing 0-7 |
| +20 | u8 | frame counter |
| +21 | i16 | ticks left |
| +23 | i16 | move duration |

  Resting position: x = col·24 − xoff[type][action], y = row·12 − 28.
- **Frame index** into `ICONBF1`: `base[type][action] + dir4[facing] + 4·frame`. Frame runs 0 … `count[type][action]` − 1. Type 16 uses `facing` (0-7) directly in place of `dir4` and no ×4 [UNVERIFIED for the ×4].
  - `dir4` (`0x857EE`, facing N, NE, E, SE, S, SW, W, NW): 0, 1, 1, 1, 2, 3, 3, 3. Columns are 0 = back/up, 1 = right, 2 = front/down, 3 = left.
  - Direction step tables: dx `0x84F2C` = 0,1,1,1,0,−1,−1,−1; dy `0x84F3E` = −1,−1,0,1,1,1,0,−1.
  - Tables are `i16[22][11]`: `base` at `0x85200`, `count` at `0x855C8`, `xoff` at `0x853E4`. −1 = action unused.

| type | base (action 0..5) | count (action 0..5) |
|---|---|---|
| 0 | 0, 0, 32, 60, 84 | 1, 8, 7, 6, 7 |
| 1 | 196, 112, 168, 196, 220 | 1, 14, 7, 6, 6 |
| 2 | 244, –, –, 244, 268, 300 | 1, –, –, 6, 8, 14 |
| 3 | 356, 356, 388, 420, 444 | 1, 8, 8, 6, 6 |
| 4 | 468, 468, 500, 532, 556 | 1, 8, 8, 6, 6 |
| 5 | 580, 580, 612, 640, 664 | 1, 8, 6, 6, 7 |
| 6 | 776, 692, 748, 776, 800 | 1, 14, 7, 6, 6 |
| 7 | 824, –, –, 824, 848, 880 | 1, –, –, 6, 8, 14 |
| 8 | 936, 936, 964, 996, 1020, –, 1060 (action 6) | 1, 7, 8, 6, 10, –, 10 |
| 9 | 1100, 1100, 1132, 1160, 1184 | 1, 8, 7, 6, 7 |
| 10 | 1212, 1212, 1252, 1280, 1304 | 1, 10, 7, 6, 6 |
| 11 | 1328, 1328, 1368, 1396, 1420 | 1, 10, 7, 6, 6 |
| 12-16 | single-sequence effects at 1444, 1450, 1458, 1466, 1474 (6 or 8 frames) | |
| 17-19 | 1482, 1483, 1484 (single frames; type 17 is the flying projectile) | |

  - xoff is 4 for most actions, 8 for action 4 and for types 1/6 action 0-1, 11 for action 5.
  - Action meanings, read from the frame art: 0 = stand, 1 = walk, 2 and 3 = attack/fire variants, 4 = hit or death, 5 = long death or fire sequence (types 2/7). Exact names are [UNVERIFIED].
  - Unit types 0-11 are fighters. The class table `0x85847` maps them to classes 0,1,2,3,4,0,1,2,3,4,5,5, and the type-name string ids are at `0x8585E`.
- **Team colour.** Every unit and flag blit uses colour flag `0x80 + 5·c`, where `c` = `0xDB7FC[side]` (the player's colour 0-5), i.e. remap ramp `5c` (§8.4).
  - The art uses palette indices 2-10 as the recolourable band. Rendered: c = 0 blue, 1 red, 2 green, 3 yellow, 4 grey, 5 white (verified visually on `ICONBF1` and `ICONP6`; the 16-bit ramps at `0x83D59`/`0x83F39` give the same hues).
  - This also answers §8.5: player colour k = ramp 5k; the colour buttons are `ICONP6` drawn with `0x80 + 5k`.
- Projectiles (type 17) fly along a parabola: y offset = −(d − t)·t·160 / d², from (col,row) to (target col,row).

### 11.5 Tactic menu (`0x37100`, set-up phase before each battle round)
- Opening message box (0,180,640,120): `[EXE text]`.
- **Status bar:** the top 640×40 strip, saved and restored.
  - Hovering a friendly unit shows its group:
    - flag sprite `ICONBF1[1444]` in team colour at (304,0);
    - text `$|$0>$69<$C$g%1S (%d%%)$E$|$59>$11<$C%s$E$|`, i.e. left half "<n> <unit type> (<avg strength>%)" and right half the group's current tactic name.
  - Without a unit under the mouse: `$|$0>$69<$C$g%1N0$E$|$59>$11<$CWählt eine Einheit!$E$|`.
  - The hovered group's cells are framed with the corner sprites `ICONF4`…`ICONF7` (8×5) on each 24×12 cell.
- **Left click on a unit** cycles its group's tactic to the next allowed one: `t = (t + 1) mod 11`, repeated until `mask[t]` (`u16` table at `0x84E38`) contains all three bits:
  - `1 << (side + 6)`;
  - `1 << class(type)`;
  - `1 << (battle_kind + 8)`.
  
  Masks: `1FFB, 1FC4, 1C44, 077B, 07BB, 187B, 18BB, 1FFB, 1FFF, 1FFB, 1FFF`.
  Tactics 0-10 (strings at `0xB69DC`): Gegner angreifen, Gegner beschießen (%d), Sturmreif schießen (%d), Waren erbeuten, Waren zurückerobern, Eingang stürmen, Eingang verteidigen, Gefangene machen, Abwarten, Rückzug, Flucht. The tactic byte is group record + 14 (100 groups × 22 bytes at `0xD7534`).
- **Right click** ends set-up. If config `0x17D` bit `0x10` is set, it first asks "Seid Ihr zufrieden?" (Ja/Nein) at (160,180,320,120).
- With no group of the player present, a choice box at (160,180,320,120) shows the message at `0xBB66C` and returns.
