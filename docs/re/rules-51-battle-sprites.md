# Battle sprites: team colour in hicolor, unit facing and cannon frames (rules-51)

Source: blitter `0x13AE0` (window-buffer paths `0x13DC5..0x14052`), battle draw in `activate` `0x389C0` (frame tail `0x3AED4..0x3C1D6`), object allocator `0x3D7C0`, `deploy` `0x3DDA0`, colour setup in `battle_main` `0x3EE60` (decompile `0x3F3E7..`). Tables were dumped from `image.bin`, and `SZENARIO.DAT` was read from the install. Tags: **[C]** instruction level, **[L]** decompiler or partial trace, **[UNVERIFIED]** inferred. Everything is [C] unless marked. This extends and corrects `formats.md` §1, §8.4 and §11.4.

## 1. Team colour

### 1.1 Which file, which flag

- Every battle sprite is a frame of bank icon `0xF1`, loaded with `0x2DD70(0xF1)` from `ICONS<mode>` (`formats.md` §1). In mode 3 that is `ICONS3B/ICONF1.DAT` [L for the directory choice], which holds 8-bit pixels (byte-identical to `ICONS2B`).
- Every blit of the bank passes `flags = 0x80 + 5·colour` (the transform byte is 0):
  - men (unit types 0..11): `0x3B1EC..0x3B259` and `0x3B32E..0x3B39B`, colour = `0xDB7FC[obj+0x10]`;
  - objects drawn with their participant's colour, cannon type 16 among them: `0x3BF94..0x3C04B`;
  - flying projectile (type 17, `0x3BA42`) and the second object path (`0x3BDE2`, explosion overlays): colour byte literal 0, so flags `0x80` (ramp 0).
- No battle sprite is ever drawn with colour 0 (as stored). In 16-bit modes the as-stored path reads 2 bytes per pixel (`0x13F60`). The remap path reads 1 byte per pixel (`0x14034`). **`ICONS3B/ICONF1.DAT` and `ICONS3/ICONP6.DAT` being 8-bit files is therefore required, not an install defect** (correction to `formats.md` §1). ICONP6 is the flag drawn with `0x80 + 5k` (§11.4).
- Goods markers and wagons (types 20, 21) are not bank frames. They are plain icons drawn by `draw_icon` `0x2E310` (ids `0x2C`/`0x1B8`, `0x2B`/`0x1B7`), as stored, with no team colour.

### 1.2 Remap (flags ≥ 0x80), all modes

`k = flags & 0x7F`, `base = 16·k`. For **every opaque source byte p** (skips are transparent, nothing else passes through):
- 8-bit modes (`0x13EB6..0x13F0A`): `out = T8[base + p]`, with `T8` at `0x83B79` (palette indices);
- 16-bit modes 3/4 (`0x13FD4..0x1404E`): `out = lo[base + p] | hi[base + p] << 8`, with `lo` at `0x83D59` and `hi` at `0x83F39`, written as one RGB555 word.

p is the full byte, with no mask. Each table has 480 entries (30 rows of 16). Battle code uses k = 5c, so **index = 80·c + p**.

### 1.3 Table layout and why only pixels 1..10 change colour

The rows repeat with period 5 (checked on `T8`):
- row 5c, entries 0..10: the team ramp of colour c;
- row 5c, entries 11..15: identical for every c;
- rows 5c+1 .. 5c+4: identical for every c.

ICONF1 uses pixel values 2..78 only. All of them are < 80, so a sprite never reaches the next colour's block. Pixels 1..10 take the team ramp. Pixels 11..79 map through shared rows to fixed colours. Those are not the palette colours of p: for example p = 46 is palette (76,0,0) but draws grey, and p = 63 is palette (44,56,12) but draws black.

Check values (16-bit, `lo | hi<<8`, then the RGB 8-bit expansion ×8):

| p | c = 0 | c = 1 | any c |
|---|---|---|---|
| 5 | `0x14DF` (40,48,248) | `0x7000` (224,0,0) | |
| 7 | `0x0492` | `0x4800` (144,0,0) | |
| 9 | `0x0006` | `0x1800` (48,0,0) | |
| 45 / 46 / 47 | | | (144,160,144) / (112,96,112) / (80,96,80) |
| 63 | | | 0 (black) |

Pixels 5..9 cover about 99 k of the ~476 k opaque pixels in the bank. That they form the bodies is [UNVERIFIED], but it fits the capture: with c = 1 they come out red, and pixels 45..47 come out grey (armour). A port that gets near-black bodies is probably not using index `80c + p` with the full byte. For example, `16c + p` lands in the shared row 1, whose entries 2..9 are dark (`T8` = 105, 121, 33, 38, 39, 41, 44, 47) [UNVERIFIED: the port code was not inspected]. Mode 3 tables are RGB555: `0x7000` is pure red only as 555.

### 1.4 Participant colours (`0xDB7FC`, 4 bytes, indexed by participant 0 attacker, 1 defender, 2/3 helpers)

1. For each participant i < n+2 (n = helpers, 0..2), with person p = `0xDB7F4[i]`: if p is human (person +0x10 & 0xF0 ≠ 0), `col[i] = person +0xF4` (the colour chosen at creation). Otherwise `col[i] = 0xFF`.
2. Then for each i in order with `col[i] = 0xFF`: v = the lowest of 0..5 that no `col[j]` (j < n+2) currently holds. Set `col[j] = v` for every j with the same person as i.

So human colours are reserved first. Computer participants then take free colours in participant order, and one person keeps one colour across slots. Capture: human attacker colour 0, computer defender → 1 (red). Colour hues: 0 blue, 1 red, 2 green, 3 yellow, 4 grey, 5 white (`formats.md` §11.4).

The object record byte **+0x10 is the participant** (copied from unit record +1 at `0x3DB4E`), and +0x11 is the side (unit +2). Correction to `formats.md` §11.4, which lists +16 side and +17 owner.

Answer: in mode 3 the unit frames are the 8-bit `ICONS3B/ICONF1.DAT` bank. The blitter gets `0x80 + 5c`, with c = `0xDB7FC[participant]`. It maps every opaque byte p to `lo[80c+p] | hi[80c+p]<<8` (RGB555, tables `0x83D59`/`0x83F39`). No pixel is drawn as stored. Only p = 1..10 differ between colours. Humans keep their own colour, and computers take the lowest free 0..5 in participant order.

## 2. Facing and frames

### 2.1 Facing at deployment (`0x3DDA0`)

- After the anchor is chosen (`rules-07` §5.4), `d = (anchor cell byte & 0xE0) >> 5`. This is the direction bits of the SZENARIO cell (`0x3E046..0x3E05B`).
- Every man of the unit is allocated with facing d (`0x3D7C0(type, unit, x, y, d)`, stored at object +0x13, `0x3DB72`). Further men go every 1 cell (2 for cannoneers) in direction `(d + 2) mod 8`.
- A cannoneer's cannon (object type 16) is allocated by the same allocator with the **same facing d**, on the cell `(x + DX[d], y + DY[d])`. `DX` (`0x84F50`) = 1, 1, 0, −1, −1, −1, 0, 1 and `DY` (`0x84F60`) = 0, 1, 1, 1, 0, −1, −1, −1, which is the `(d+2)` step (`0x3D9AF..0x3D9EB`).
- Action := 0 and frame := 0 for all objects.
- Resting pixel position (`0x3DAB9..0x3DAF8`): x = col·24 − XOFF[type][0], y = row·12 − 28. Markers: x = col·24 + 1. Wagons: x = col·24 − 9.

Cannon anchors in `SZENARIO.DAT` (code 16 = defender class 2, as (col, row, d)):
- field 3 (castle): (5,5,3), (25,6,5), (20,16,5), (24,19,5), (19,21,5), (23,32,6);
- field 4 (camp): (18,6,5), (18,9,5), (16,14,5), (23,25,6), (18,42,7), (22,42,7);
- field 2: (17,19,5), (21,34,6), (22,42,6).

The defender uses the anchor nearest the first entrance square (`rules-07` §5.4). Most castle cannon anchors have d = 5 (SW) [UNVERIFIED: which anchor wins was not computed here]. So the defender's cannoneers and their cannons face down-left.

### 2.2 Frame index (`0x3B143..0x3B19C`, `0x3BEE5..0x3BF8E`)

- Men (type < 12): `frame = BASE[type][action] + DIR4[facing] + 4·anim`.
- Objects (type ≥ 12, drawn when `anim < COUNT[type][action]`): `frame = BASE[type][action] + D + anim`, **without ×4**. Here D = `facing` for type 16 (cannon) and `DIR4[facing]` for the other objects.
- Tables (`i16`, 11 actions per type, 22 bytes per type): BASE `0x85200`, XOFF `0x853E4`, COUNT `0x855C8`. DIR4 is `u16[8]` at `0x857EE` = 0, 1, 1, 1, 2, 3, 3, 3.
- Cannon (type 16): BASE[16][0] = 1474, COUNT[16][0] = 8, XOFF[16][0] = 8, all other actions −1. So a cannon is always frame **1474 + d**: eight pre-drawn facings, with no animation through `anim` at rest. Cannoneers (types 2/7): BASE[·][0] = 244 / 824, COUNT 1, XOFF 4.
- A cannoneer with action 5 is drawn at (x − 6, y − 6) (`0x3B0F5..0x3B13D`). The others are drawn at the stored (x, y).

Answer: deployment gives every man and cannon the anchor cell's direction d. On the castle field that is d = 5 for most cannon anchors. A cannon frame is `1474 + d`, with the raw facing and no DIR4 and no ×4, so d = 5 gives frame 1479, the diagonal down-left cannon. Men use `BASE + DIR4[d] + 4·anim`. An upright cannon means the port either ignores the cannon's facing or routes it through DIR4 or ×4.

## 3. Black tunics: DIR4 is a u16 table (follow-up)

**(a) Indexing [C].** BASE, XOFF and COUNT are `i16[type·11 + action]`, read as `word [0x85200 + 22·type + 2·action]` (`0x3B143..0x3B17C`), with type 0..21 and action 0..10. The port's BASE values per type are correct.

**DIR4 is `u16[8]`, not bytes [C].** The draw code reads `word [0x857EE + 2·facing]` (`0x3B175`/`0x3B183`, `0x3BF1E`/`0x3BF27`). The memory at `0x857EE` is `00 00 01 00 01 00 01 00 02 00 03 00 03 00 03 00`, so DIR4 = 0, 1, 1, 1, 2, 3, 3, 3. Read as bytes, d = 0..7 gives 0, 0, 1, 0, 1, **0**, 1, 0. Every odd facing, d = 5 included, then lands on column 0, the **back view**. That view is the black cloak: mostly p = 63 (black in every colour, §1.3) plus pixels 16, 31, 46, 47 and 59..62. This is why the tunics are black.

**(b) Frame for a standing castle Söldner [C].** Type 0, action 0: `BASE[0][0] + DIR4[5] + 4·0 = 0 + 3 + 0 = 3`. Frames come in groups of four views: back (col 0), right (col 1), front (col 2) and left (col 3), then the next animation step. Rendered with c = 1 (scratch sheet, frames 0..127): frames 1, 2 and 3 show red tunics with grey/white armour, and frames 0, 4, 8, … show the black back view.

Castle field facings of the defender anchors (code 14..19 = defender classes 0..5, as (col, row, d)):
- code 14 (Söldner): (7,12,3), (15,15,4), (13,22,5), (15,24,5), (18,27,5), (22,28,6);
- code 15 (Musketier): (9,6,4), (8,9,4), (21,13,5), (27,18,5), (22,21,5), (21,32,6);
- code 17 (Offizier): (6,9,3), (15,13,4), (22,17,5), (14,20,5), (16,22,5), (19,25,5), (24,26,6);
- code 18 (General): (26,8,5).

d = 3 and 5 give DIR4 = 1 and 3. d = 4 gives 2 (front), and d = 6 gives 3.

**(c) No pixel offset [C].** The blitter takes the raw RLE byte as p (`0x14034`). There is no per-file or per-bank offset, and the decoded indices are right.

Answer: read DIR4 as 16-bit words. With byte reads, every odd facing selects the back view, which is drawn black. The correct standing frame for a castle Söldner with d = 5 is 3 (left view, red tunic).
