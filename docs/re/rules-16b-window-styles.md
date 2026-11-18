# Window styles, frame tiles, compositor and screen transitions (rules-16b)

Source: `win_open` 0x2FA80, `win_close` 0x30190, frame painter 0x30880, roll painter 0x30670/0x305A0, compositor 0x2FDF0, `flip` 0x30400, item creation 0x34130, blitter 0x13AE0 (transform branch), glyph painter 0x31610, pointer 0x2EC40/0x2EC80/0x2E5B0, transition 0x2C460, tick wait 0x12DB5. Tags: [C] instruction level, [L] decompiler only, [UNVERIFIED] inferred. Resolves `rules-15` §9 "window style bits", the 0x2FDF0/0x2C460/0x2EC40 questions, and `formats.md` §7.2's transform guess.

## 1. The window record [C]
`win_open(x, y, w, h, style)` (EAX, EDX, EBX, ECX, stack) pushes the current window record (48 bytes at `0xD5000`) onto a stack of 16 (`0xD66B0`, depth byte `0x84D44`; more than 15 open windows aborts the program) and fills a new one:

| offset | meaning |
|---|---|
| +0x00/+0x02 | x, y on screen |
| +0x04/+0x06 | w, h |
| +0x08 | style word (bit 0x100 is cleared again after use) |
| +0x0C | handle of the saved-background frame (style 0x4000) |
| +0x0E | drawing surface: the screen (`0xEF4BA`) or the window's own off-screen frame |
| +0x10/+0x12 | the two 16-px roll strips (style 0x2000) |
| +0x14/+0x16 | origin of the window on its surface: (x, y) when drawing on the screen, (0, 0) on an own frame |
| +0x18/+0x1A | text cursor x, y (12, 12) |
| +0x1C | scroll offset (0) |
| +0x1E | surface height: h, or h + 48 for a scrolling window |
| +0x20 | top of the text area = scroll + 12 |
| +0x22 | item spacing 4 |
| +0x24 | text flags, initially 0x4000 (§3) |
| +0x26 | line height 24 |
| +0x28 | `W[0x84DF6][opt]`, opt = options byte `0x86A47` (0..5, cycled by the options screen); table starts 24, 12, 8, 6 — a text-speed value [L] |
| +0x2A/+0x2C | left margin 12, text width w − 24 |
| +0x2E | extra glyph spacing 1 px |

Off-screen frames are allocated with 0x131E9 (w × h bytes of the current pixel format, plus an 8-byte header); the pixel format follows the video mode byte `0x8332F` (1 byte per pixel in the 8-bit modes used here).

## 2. Style bits [C]
| bit | effect at open | effect at close |
|---|---|---|
| 0x8000 | the window draws into its **own off-screen frame**; the compositor (§4) can later copy it to the screen or into other frames. Without it, the window draws straight onto the screen at (x, y) and nothing remembers its pixels | frame freed (or handed back, 0x400) |
| 0x4000 | **saves the screen** rectangle (x, y, w, h) into a backup frame | the backup is copied back to the screen, then freed (plus the roll strips are presented first for 0x2000) |
| 0x2000 | **scrolling text window**: forces 0x8000, surface height h + 48 (the frame is a ring buffer; every copy uses `(scroll + y) mod (h + 48)`), two 16-px roll strips are allocated and painted (§5); the frame painter draws only left edge, interior and right edge | both strips are re-composed from the windows below and freed |
| 0x1000 | never set by callers; `flip` sets it when the low nibble of the style is non-zero, nobody reads it | — |
| 0x0800 | — | **erase on close**: scroll := 0, the whole rectangle is re-composed from the windows below into the screen and flipped. This is how boxes over a buffered backdrop disappear |
| 0x0400 | with 0x8000: **borrow the parent's frame** instead of allocating one; the parent's 0x8000 flag is cleared meanwhile | the frame is given back and the parent's 0x8000 restored. The parent's picture is then whatever the child drew; the caller redraws |
| 0x0200 | set by `flip` after a presentation, cleared when an item is created; never tested | — |
| 0x0100 | **no frame**: the parchment tiles are not drawn; margins and cursor are the same defaults | — |
| 0x0010 | nothing reads it (`0x8810` ≡ `0x8800`, `0x8910` ≡ `0x8900`) | — |
| 0x000F | `flip` presents the frame only when these four bits are 0; none of the game's styles sets them | — |

The styles in use:

| style | bits | meaning | used by |
|---|---|---|---|
| 0x8100 | own frame, no tiles | full-screen backdrop; the caller draws a picture at (0,0) and flips | main menu, year end, disaster, certificate |
| 0x8910 | own frame, no tiles, erase on close | full-screen backdrop that erases itself; also the transparent sub-windows (§3) | auction, player setup, jury bribe window, trial input windows |
| 0x8500 | own frame borrowed from the parent, no tiles | the matchmaker paints its picture into the Kontor's frame | matchmaker |
| 0x8800 | own frame, tiles drawn, erase on close | caption box / caption menu, but see §3: the tiles are overwritten | 0x36C90, 0x36AC0, 0x36E50 |
| 0x8810 | same as 0x8800 | parchment message box with OK glyph | 0x368E0, menu box, options |
| 0xA810 | own frame, tiles, scrolling, erase on close | the report text windows with scroll rolls | turn-end / year-end reports |

`flip` 0x30400: for an own-frame window copies the whole frame (from row `scroll mod (h+48)`) to the screen at (x, y), then paints the roll strips (0x2000) and marks the window's items presented. For a screen-drawn window flip does nothing. A window without 0x8000 and without 0x4000 cannot be erased by the library: the caller must repaint.

## 3. Caption boxes are frameless [C]
`0x36C90`, `0x36AC0` and `0x36E50` open a 0x8800 window (the tiles are painted into its frame), then immediately call the compositor with the window's own frame as destination and its own screen rectangle as source area. The compositor copies the pixels of every buffered window below (§4), so the fresh frame now holds the backdrop picture under the box and the parchment tiles are gone. The text is then printed after `$g`. Result: a caption is text drawn directly over the scene picture, no parchment, no border. The erase at close (0x800) restores the picture. The same trick is used by the trial input windows (`win_open(…, 0x8910)` followed by the compositor over their rectangle) and the office tree window (20, 40, 600, 400): transparent sub-windows over a picture.

`rules-15` §0.1 ("the box has the parchment frame") is wrong; the message box 0x368E0 (0x8810, no compositor call) keeps its parchment.

Text colour: the glyph painter 0x31610 reads the text-flags high byte stored with each character. With bit 0x40 **clear** (after `$g`) each glyph is drawn nine times: the glyph from the second font table (`0xC43A0`, the dark 0x80..0xFF half) at the eight neighbours (±1, ±1) and then the glyph from the first table (`0xC41A0`, the gold 0x00..0x7F half) in the centre — gold letters with a one-pixel dark outline; a highlighted run (`$h`, bit 0x80) draws the centre glyph through remap ramp 9. With bit 0x40 **set** (default, `$t`) only the dark glyph is drawn; highlighted runs are drawn as solid colour 1 (`formats.md` §8.4). Which table holds which half is [L] (the tables are 128 pointers apart and the first is the lower codes); the visual reading — dark on parchment, outlined gold on pictures — matches the shipped screens.

## 4. Compositor 0x2FDF0(dst, x, y, w, h) [C]
Arguments: destination surface in EAX, the screen rectangle in EDX/EBX/ECX/stack. For every window on the stack **below the current one** (indices 1 .. depth − 1) whose style has 0x8000, the intersection of the rectangle with that window is copied from the window's frame (row `(scroll + dy) mod surface height`) into `dst` at the position relative to the rectangle's corner. Scrolling windows also contribute their two roll strips, 16 px above and below their rectangle. Lower windows are copied first, so the top-most buffered window below wins. It is used to:
- restore the background under a frame tile before an edge/corner tile is drawn (§5), so the tiles' transparent pixels show the parent;
- erase a closing window (style 0x800);
- fill the mouse-pointer scratch (32×40) before the pointer is drawn (0x2E5B0/0x2EC80);
- give caption boxes and input windows a transparent background (§3);
- repaint an item's background when it is removed (0x34930).
It does **not** darken, fade or tint anything (`rules-15` §9 item 2 resolved).

## 5. Frame tiles [C]
The window is covered by a 40×40 grid anchored at its top-left corner; cells at the right and bottom are clipped to the window size (a 100-px-wide window shows a 20-px-wide right column). For each cell:
1. The 40×40 scratch surface `0xD6A3C` is filled by the compositor with the backdrop at the cell's screen position (skipped for interior cells).
2. The tile is blitted onto the scratch at (0, 0) with the transform in the high byte; pixel value 0 is transparent in this blit path.
3. The visible part of the cell is region-copied from the scratch into the window surface.

Tile selection, with `cx`/`cy` the cell's x/y inside the window (0, 40, 80, …), `wx`/`wy` the window position on screen and `s` the scroll offset:

| cell | tile id | transform |
|---|---|---|
| top-left (cx < 40, cy < 40) | `0x06 + (wx/40) % 4` | 1 |
| top edge (40 ≤ cx < w − 40, cy < 40) | `0x0E + (cx/40) % 4` | 1 |
| top-right (cx ≥ w − 40, cy < 40) | `0x06 + (wy/40) % 4` | 3 |
| left edge (cx < 40, 40 ≤ cy < h − 40) | `0x0A + (cy/40) % 4` | 1 |
| interior | `0x05`, no compositor fill | 1 |
| right edge (cx ≥ w − 40) | `0x0A + (cy/40) % 4` | 3 |
| bottom-left (cy ≥ h − 40) | `0x06 + (wx/40 + 2) % 4` | 2 |
| bottom edge | `0x0E + (cx/40) % 4` | 2 |
| bottom-right | `0x06 + (wy/40 + 2) % 4` | 4 |

Scrolling windows (0x2000) take only the left-edge / interior / right-edge rows with `cy` replaced by `cy + s`. Note the corner indices depend on the window's screen position, so the same window looks different at another place; edge indices depend on the position inside the window.

Transforms, from the blitter's 8-bit raw path [C]: 1 = as stored; 2 = rows bottom-to-top (vertical flip); 3 = pixels right-to-left (horizontal flip); 4 = both (180° rotation). This confirms the guess in `formats.md` §7.2. The tiles ICON05..ICON11 are 40×40 raw sprites with colour 0 transparent at the torn edge.

Roll strips (0x30670, scrolling windows only): two 16-px strips as wide as the window, `k = (s mod 16) / 2` (0..7): tile `0x12 + k` at x = 0 (transform 1), tile `0x1A + k` at x = 40, 80, … while x < w − 40, tile `0x12 + k` at the last x with transform 3. They are presented at (wx, wy − 16) and (wx, wy + h) (0x305A0), so the rolls animate with the scroll offset.

## 6. Items (0x34130(style, data) with x, y, w, h in EAX, EDX, EBX, ECX) [L]
The style word's bits 0x1F0 select the item kind and the low nibble a count; w = 0 or h = 0 means "compute from the kind":

| kind | meaning | default width / height |
|---|---|---|
| 0x00, 0x10, 0x20 | numeric spinner with `nibble` digits (0 → 3, 5, 10 digits); 0x40, 0x50, 0x60 the same with the alternate digit glyph metrics; 0xC0, 0xD0 with an extra sign glyph | digit widths from the font tables; height 24 (0x00..0x20), 18 (0x40..0x60, 0xC0) |
| 0x30 | text menu line (`%m`) | measured text width; height 24 |
| 0x80 | `nibble` × 2 character cells | height 24 |
| 0x90, 0x100 | icon button; `data` = icon id | icon size from the ICONIN tables (`0xC2DA0` widths, `0xC29A0` heights) |
| 0xA0, 0xB0 | cycling string item | text width; height 24 |
| 0xF0 | like 0x30 | |
| 0x110 + k, 0x130 | animated bank item; `data` = bank id, `k`/frames from the bank header (`0x110 | k` = colour buttons with ICONP6, `0x130` = vote coin ICONPC) | 0x130: 12 high |
| flag 0x1000 | no background backup for the item (otherwise a frame is allocated and flag 0x2000 set) | |
| flag 0x40000 | centre horizontally (x −= w/2) | |
| flag 0x10000 | cleared at creation; skips the initial-value setup | |
| flags 0x200, 0x400 (in 0x1690) | [UNVERIFIED] | |

Item records are 36 bytes at `0xD5030` (count `0x84D48`, max 159). `0x1030` is the plain menu line, `0x1090` an icon button without backup, `0x1690` the OK glyph 0xCD of message boxes.

## 7. Mouse pointer 0x2EC40(k) [C]
Sets the requested pointer shape `0x84D68` and redraws when it differs from the shown one `0x84D64`: 0 = hidden, 1..5 = sprites from table `0xC399C` (ICON00..ICON04: arrow, +, −, up, down per `formats.md` §7.1), ≥ 6 from `0xC3F3C` [UNVERIFIED which]. The pointer is 32×40; its background is kept in the scratch `0xD6A38`, refilled by the compositor at each move (0x2E5B0, called from the input polling). `win_open`/`win_close` hide the pointer during drawing (`0x84D6C` is a hide counter the scenes bump around their own region copies). `rules-15` §7.4's `0x2EC40(0)` at the notary hides the pointer; the battle, trade map, office tree and camp screens call it with 0/1 for the same reason.

## 8. Screen transition 0x2C460(mode) [C]
A per-scanline reveal of the current window's frame onto the screen (the window must be an own-frame window at (0,0), which holds the next picture, not yet flipped):
1. Setup: `centre[0] = 320`, `width[0] = −250`; for y = 1..479: `centre[y] = centre[y−1] + rand(9) − 4`, `width[y] = width[y−1] + rand(25) − 12` (958 game-RNG draws, parity hazard).
2. 200 frames; on every odd frame `delay_ticks(1)` (one hundredth-second tick of the DOS clock; on real DOS hardware the clock advances in 55 ms steps, so the effect lasts a few seconds, on the port use 10 ms).
3. Per frame, for every row y in the range (mode 1: 155..324; otherwise 0..479) that is not finished: if `width < 0` then `width += y/80 + 1` else `width += width/30 + 1`; then `width += frame/20 + 1`. If `width > 0`: copy the span `[centre − width/2, centre + width/2)` clipped to 0..640 of row y from the window frame to the screen; when the clipped span starts at 0 and the unclipped end exceeded 640, the row is finished.
Rows open from a jittered vertical centre line outward, the lower rows faster. Callers: duel (mode 0, revealing the duelling ground), jury court (mode 0, twice: acquittal and tower screens), credits 0x4D880/0x4DA50 (mode 1, middle band only).

## 9. `delay_ticks` 0x12DB5(n) [C]
Reads the DOS time (INT 21h, hundredths) in a loop until it has advanced by n hundredths (mod 100) since the previous call. `rules-14` §1.5 holds; the granularity is the 18.2 Hz DOS clock.

## 10. Open
- Item flag bits 0x200/0x400 and the click behaviour per kind (0x35B60) [UNVERIFIED].
- Which font table is gold vs dark (§3) [L].
- Pointer shapes ≥ 6 [UNVERIFIED].
- `formats.md` §7.2 and `rules-15` §0.1/§0.2 should be amended: caption boxes are frameless (§3), the transform meanings are confirmed (§5).
