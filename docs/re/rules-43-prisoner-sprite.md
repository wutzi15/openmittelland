# Prisoner sprite and text speed (rules-43)

Two follow-up questions: how the battle scan draws a prisoner (type-12 record made from a captured man, `rules-42` §7.3), and where the text-speed byte `0x86A47` is stored. Conventions as in `rules-42` (header). Battle record layout as in `formats.md` §11.4 / `rules-42` §7: record table `0xD7DCC` (0x21 bytes), +0 / +2 pixel x / y, +4 / +6 cell x / row, +0x0E type, +0x10 colour slot, +0x11 side, +0x12 state, +0x13 facing, +0x14 frame index, +0x15 frame counter, +0x1E link byte. Sprite bank = icon 0xF1 (`ICONBF1`, fetched with `0x2DD70(0xF1)`), blitter `0x13AE0`. Tags: **[C]** instruction level, **[L]** decompiler or xref only, **[UNVERIFIED]** inferred. Everything is [C] unless marked.

## 1. How a prisoner is drawn

**The conversion (`0x3D084..0x3D15B`, end-of-fall case of the frame scan in `0x389C0`).** When a captured man's fall ends with HP > 0:
- colour slot +0x10 := the captor record's +0x10 (+0x1E held the captor during the fall);
- side +0x11 xor 1;
- +0x1E := old type, type +0x0E := 12;
- pixel x +0 += row mod 4 (row = +6); pixel y is not touched;
- state +0x12 := 0, frame index +0x14 := 0, counter +0x15 := `count[12][0]` = **6**, facing +0x13 := **0**;
- if the battle is displayed: sample `0x12698(0x4C, 3)`. This is the same call the banner plant makes at `0x3AB8D`.

The record stays in the men layer at the man's cell.

**The draw (`0x3BC6B..0x3C050`).** The scan sends types < 12 to the man branch (`0x3B0A3`). Types 0x11, 0x14 and 0x15 have their own branches. Every other type ≥ 12 goes to the generic object branch, and type 12 is one of them. The generic branch reads only +0x0E, +0x12, +0x13, +0x14 and +0x10; it never reads +0x1E. So a prisoner is drawn exactly like a banner:
- **bank / frame:** `ICONBF1` frame `base[12][state] + dir4[facing] + frame index` = 1444 + 0 + i, i = 0..5. The banner sequence (frames 1444..1449) is used. The old type's bank frames are not used, and no dedicated prisoner sprite exists. Unlike the man branch, the generic branch does not multiply the frame index by 4. Type 16 uses the raw facing in place of `dir4`.
- **position:** (pixel x, pixel y) as stored. A banner gets 24·col − 8 when it is created (`0x3D7C0`, `xoff[12][0]` = 8). A prisoner keeps the falling man's pixel x plus row mod 4, and the man's row baseline 12·row − 28. Whether the fall state re-bases the man's pixel x is [L]; the fall `xoff` is 8 for every man type.
- **colour remap:** `0x80 + 5 · [0xDB7FC + colour slot]`, the captor's team ramp (same rule as men and banners).
- **drawn while:** frame index < `count[12][0]` (6). This holds every frame, also after the counter has run out (next point), so the prisoner is redrawn each frame.

**Animation.** The scan draws first and then, if the counter is > 0, increments the frame index and decrements the counter (`0x3C1D6..0x3C214`). With counter 6 the prisoner plays frames 1444..1449 once, one per scan frame. When the counter reaches 0, the end-of-animation switch (`0x3C43C`, jump table `0x3C410`, state 0 → `0x3C46E`) sets the frame index to **5** for type 12 (0 for other types). The counter stays 0, so the record then shows frame 1449 for the rest of the battle. A displayed frame waits one BIOS tick (~55 ms, `rules-07` §5.11), so the wave lasts about 6 ticks.

**Banner vs prisoner.** Both are type 12, state 0, facing 0, and both are in the men layer. The only field that tells them apart is **+0x1E** (and only until the prisoner's old unit is next activated: its reset loop writes +0x1E := 0xFF into every slot, rules-71 §4):
- **banner:** 0xFF, because `0x3D7C0` writes +0x1E := 0xFF on every create (`0x3DBAA`);
- **prisoner:** its old type, 0..11.

The banner plant (`0x3AB53..0x3AC49`) creates the banner with facing 0. On every plant or recapture it sets the banner's colour slot and side to the planter's, and sets frame index 0 and counter 6. So a banner replays the same 6-frame wave each time it changes hands, then holds frame 1449. All reads of `0xD7DEA` (+0x1E) in the program are in `0x389C0` and `0x3D7C0`, and apart from the conversion they sit on man (type < 12) paths. Nothing reads +0x1E of a type-12 record [L, xref-based].

Answer: a prisoner is drawn with the banner's sprite, `ICONBF1` frames 1444..1449 (`base[12][0]` + frame index, facing forced to 0), in the captor's team ramp `0x80 + 5·0xDB7FC[colour]`, at the man's old pixel position shifted right by row mod 4. It waves once (6 frames, one per scan frame) after the conversion and then holds frame 1449. It is not drawn with its old type's frames. Banner and prisoner differ only in +0x1E (0xFF for a banner, the old type 0..11 for a prisoner), which the drawing and the battle logic never test. On screen a prisoner looks like a planted flag in the captor's colour.

## 2. Text speed in CONFIG.DAT

`loadconfig` `0x42030` reads 0x279 bytes into `0x86954` (`0x42064..0x42071`), and `0x421A0` writes the same 0x279 bytes back (`0x421D9..0x421DE`). `0x86A47 − 0x86954` = **0xF3**, so the text-speed byte is CONFIG.DAT offset 0x0F3. The upgrade path for a 0x278-byte file only moves bytes from 0x199 on, so 0x0F3 is unaffected.

- Readers: `win_open` `0x2FA80` (`0x2FC27`, scroll step `W[0x84DF6 + 2·value]`, `rules-42` §1) and the options screen `0x4EC10` (`0x4EC62` label, `0x4EF76` change).
- Writer: only the options screen at `0x4EF8E`. It sets `(value + 6 − d) mod 6`, where d is the step direction of the click, so the value cycles through 0..5.
- The shipped default and live CONFIG.DAT both have bytes `00 00 00 02` at 0x0F0..0x0F3, so the stored value is **2 = "Normal"** (8 px steps), the initial value named in `rules-42` §1.
- It is saved with the config: by "Spiel beenden" (`rules-42` §5) and wherever else `0x421A0` is called. It is not saved with the game.

Answer: yes. The text speed is CONFIG.DAT byte **0x0F3** (0..5, scroll step 24, 12, 8, 6, 3, 1 px; default 2), loaded with the rest of the 0x279-byte config image at start-up.

## Summary

| item | answer | section |
|---|---|---|
| prisoner sprite | banner frames `ICONBF1` 1444..1449 (facing forced 0), captor's ramp `0x80 + 5·0xDB7FC[colour]`, man's old pixel position + row mod 4 | §1 |
| animation | one 6-frame wave after conversion, then frame 1449 held (end-of-animation sets index 5 for type 12) | §1 |
| banner vs prisoner | only +0x1E: 0xFF banner, old type 0..11 prisoner; never tested by draw or logic [L] | §1 |
| text speed in CONFIG.DAT | yes, offset 0x0F3, values 0..5, default 2 "Normal" | §2 |

## Corrections to other docs

- `formats.md` §3.1: the field `0x0F0` (4 bytes, `00 00 00 02`) holds the text-speed byte at **0x0F3** (`0x86A47`, 0..5, default 2). The meaning of 0x0F0..0x0F2 is still unknown.
- `formats.md` §11.4: the ×4 is confirmed for men (types 0..11: `base + dir4[facing] + 4·frame`, `0x3B197`). The generic object branch (types 12..16, 18, 19) uses `base + dir4[facing] + frame` with **no** ×4, and type 16 uses the raw facing (`0x3BF04..0x3BF8E`). Drop the [UNVERIFIED].
- `rules-42` §7.3: the conversion also sets facing := 0 and counter := 6 and plays sample 0x4C when displayed. The prisoner is drawn as a banner (§1 here).
