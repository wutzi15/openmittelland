# Open questions, round 4 (rules-42)

Eight follow-up questions (text overflow, caption waits, name caret, feast word, quit save, emperor number, battle frame loop, auction timing). Conventions as in `rules-40` (header): `rand(n)` = `0x36FF0`, `wait_click` = `0x36720`, `print` = `0x322C0`, `delay_ms` as in `rules-25`. Tags: **[C]** instruction level, **[L]** decompiler only, **[UNVERIFIED]** inferred. Everything is [C] unless marked. Each section ends with an "Answer:" line.

## 1. Text overflow in scrolling windows

Current-window fields (the copy at `0xD5000`, offsets as in `rules-16b` §1): +0x04 w, +0x06 h, +0x18 cursor x, +0x1A cursor y (window-relative, top of the next line), +0x1C scroll offset `s`, +0x20 `top` = absolute row of the first line printed since the last click wait, +0x25 bit 0x10 "scrolled since the last wait", +0x26 line height (24 at `win_open`), +0x28 scroll step, +0x2A left margin (12). `0x84D9A` = characters waiting in the line buffer, `0x84D9E` = "no line advance" counter.

**Scroll step.** `win_open` `0x2FA80` (`0x2FC25`) sets +0x28 = `W[0x84DF6 + 2·opt]` with `opt` = options byte `0x86A47`, the options screen's text-speed setting (`0x4EC10`, labels table `0xB6E3C`). [C]

| opt | label | step (px) | steps per 24-px line |
|---|---|---|---|
| 0 | Zu schnell | 24 | 1 |
| 1 | Schnell | 12 | 2 |
| 2 | Normal (initial value of `0x86A47`) | 8 | 3 |
| 3 | Geht so | 6 | 4 |
| 4 | Schnarch | 3 | 8 |
| 5 | Tödlich | 1 | 24 |

The options screen cycles the byte with `(opt + 6 − k) mod 6`. Because the step is copied at `win_open`, a change applies to windows opened afterwards. [C]

**End of a line** (`0x31610` tail, `0x31D8C..0x31F75`) [C]:
1. Clear the line buffer (`0x84D9A := 0`, `0xD6A30 := 0`). If `0x84D9E ≠ 0` (code `$E`) stop here: no advance, no scroll, no wait.
2. Cursor x := left margin, `y += line height`. If `y ≤ h`, stop.
3. **Wait test, `0x31E17`:** if `s + y + lineheight > top + h`, call `wait_click` `0x36720`. Both sides are absolute rows in the ring, so this means that the text printed since the last wait, plus the next line, no longer fits the window height. The player has to acknowledge a full window before unread text scrolls off the top. `wait_click` first flips, then sets `top := s + y` and clears bit 0x10 (`0x3673D..0x36750`).
4. **Scroll loop, `0x31E1C`**, while `y > h`:
   - `d = min(y − h, step)`;
   - `y −= d`, `s += d`, and two words of the window-stack entry (`0xD66CA − d`, `0xD66CC + d`, stride 0x30) move with it;
   - set bit 0x10;
   - every item of this window (`0xD5030` table, owner `+0x1A`) moves up by d (`+0x16` and `+0x02` −= d) and loses its "presented" bit (`+0x09 &= 0x7F`);
   - repaint the left 8-px and right 8-px edge strips with the frame painter `0x30880`, because the edge tiles depend on `s` (`rules-16b` §4);
   - repaint the roll strips `0x30670` (style 0x2000), whose tile `k = (s mod 16)/2` makes the rolls turn;
   - `flip` `0x30400`.
   
   So the window **scrolls smoothly and never clears**. A normal line overflows by exactly one line height (24 px; 12 px after `$Y`/`$Z` half-height lines), and after the loop `y = h`, so the line just finished sits at the bottom of the window. The interior is not repainted: the scrolled-in rows come from the ring buffer (surface h + 48), where the line was already drawn below the visible area.
5. Clear `0x84D9F` and return.

**Step timing [L].** The loop body has no timer wait (`0x12DB5`) and no vertical-retrace poll: `0x30400` and `0x2FDF0` contain no port I/O. Each step lasts as long as one edge, roll and flip repaint takes, so the scroll speed is set only by the step size and the machine's speed. For a fixed rate the port needs its own per-step delay; one frame (about 16 to 20 ms) per step makes "Normal" about 50 ms per line [UNVERIFIED as a target; not measured].

**Print control codes** (dispatch in `print` `0x322C0`, `0x32349..0x3247F`) [C]:
- `$V` (0x56) → `0x32720`: flush a pending partial line (end-of-line above), then **wait if anything was printed since the last wait**, i.e. if `s + y ≠ top` or bit 0x10 is set. The call at `0x32758` is this wait. With nothing new (cursor still on the first line after a wait or page start, nothing scrolled), `$V` does nothing. This is the report's "wait for a click when something was printed".
- `$W` (0x57) → `0x32762`: flush, then **always** `wait_click` (`0x32771`).
- No other code waits directly. `$N` (`0x326EA`) and `$X` (`0x3277B`, two line ends) wait only through the overflow test at `0x31E17`. So does `$Y`/`$Z` (`0x32794`/`0x327CD`, a line end at half line height), and so does `$E` (`0x326F4`, a line end with the advance suppressed, so it never waits).
- `$P` (0x50) → `0x325A5`, same as `0x30F10`: flush, then cursor to (12, 12), `top := s + 12`, clear bit 0x10, repaint the rolls and the whole window frame (`0x30880` over w × h). This is a clear without scrolling and **without a wait**. Text that should be acknowledged first needs `$V`/`$W` before `$P`.

Answer: on overflow the window first waits for a click, but only once a full window of unread text has built up (`s + y + lineheight > top + h`). Then it scrolls up smoothly by the overflow (one line, 24 px) in steps of the text-speed setting (default 8 px, i.e. 3 steps, each step repaints the edges and rolls and flips, with no timer delay), keeping its contents (no clear). Only `$P` clears and restarts at the top, without waiting. The two print waits are `$V` at `0x32758` (only if something was printed or scrolled since the last wait) and `$W` at `0x32771` (unconditional).

## 2. Caption and say_box waits

**`sound_idle` `0x12C71` [C].** Returns 1 ("idle") in these cases, tested in this order:
1. no digital sound card (`0x80184 = 0`) → 1;
2. `0x81B2E ≠ 0` → 0 (busy);
3. `0x81B2D ≠ 0` → 1;
4. `0x82126 ≠ 0` → 1;
5. otherwise 1 only if `0x82EA8 = 1`, else 0.

`0x81B2E` counts the mixer voices that hold a **game sample** (kind 0x10). The voice allocator `0x11AC4` increments it together with `0x81B2D` (all voices) for every game sample. The song start `0x120BC` resets `0x81B2D` to `0x81B2E`, so `0x81B2D` is all voices, game samples plus the samples of the sample-synth MIDI player. The interrupt mixer, which is not in the image (`rules-14` §5), decrements both when a voice ends. `0x82126` and `0x82EA8` are DMA/mixer state that the driver resets to 0 in `0x11AC4`/`0x1217F` and that only the mixer sets [UNVERIFIED meaning]. For the port: **idle = sound off, or no game sample still playing**. Any game sample counts, a sound effect as well as speech, so a sample still ringing extends a speech wait [L for steps 3 to 5, which only matter while the mixer is in a transitional state].

Speech that never starts is idle at once. `speech_load` `0x15100` builds a file name only when the speech option bit `0x86AD1 & 0x80` is set, otherwise it loads nothing. `speech_play` `0x154B0` plays only when the slot `0xC22B0[n]` holds a loaded sample [C]. With speech off, a missing file or no sound card, nothing is playing and `sound_idle` is 1 immediately.

**`say_box` `0x36E50` [C].** The steps are:
1. open the window with style 0x8800 (`0x2FA80`);
2. compositor (`0x2FDF0`);
3. `speech_load` (`0x15100`), `speech_play` (`0x154B0`);
4. print `$g` (`0xBB668`), then the text;
5. present (`0x30400`).

Then comes **one** `sound_idle` test (`0x36FB2`):
- **not idle** (speech running): busy-poll `sound_idle` until it is 1 (`0x36FBB..0x36FC2`). There is no click, no timeout and no minimum time. Mouse and keys are ignored, and the wait cursor is not shown (it belongs to `wait_click` only).
- **idle at once** (speech off, missing, no card, or already finished before the test): `wait_click` `0x36720` (`0x36FC6`), with the wheel cursor.

Afterwards `0x153F0` pops the speech and `0x30190` closes the window (erase). No fixed delay is involved anywhere.

**`caption` `0x36C90` [C].** The steps are:
1. open the window with style 0x8800;
2. compositor;
3. print `$g`, then the text;
4. item paint `0x34BF0`;
5. present.

Then the box tests its delay argument (`[EBP+0x18]`, test at `0x36DFC`):
- **> 0**: `delay_ms(arg)` (`0x6B07C` → `0x6C083`, real milliseconds per `rules-25`). No click and no speech test; the input is not read.
- **= 0**: `wait_click` `0x36720`.

Then the items the box created are deleted (`0x34930` until the item count is back to the entry value) and the window is closed. The caption has **no speech of its own** and never calls `sound_idle`.

**Caller-side waits.** Scenes that start speech themselves and then show a caption or plain text wait in their own code, for example the funeral (`rules-17` §4): "if `sound_idle()` is already true → `delay(N)` (N = 2000, 3500 or 4000 ms), else poll until `sound_idle()`", sometimes followed by a further fixed `delay`. With speech running this waits **exactly until the speech ends**, with no minimum. Only when nothing is playing at the test does the fixed N apply.

Answer: `0x12C71` = "sound off or no game sample playing". `say_box` waits until its speech has ended if the speech is running when the text is up; otherwise it waits for a click, never for a fixed time. `caption` waits `delay_ms(n)` when n > 0, otherwise for a click, never for speech. The 2 s comes only from caller code of the form "idle → delay(2000), else until idle", i.e. `speech_running ? speech_rest : 2000 ms`, plus any separate fixed delay. `max(speech, 2 s) + n` is therefore wrong for speech shorter than 2 s: the original waits the speech length only.

## 3. Name-entry caret

**Painting [C].** The item painter `0x34BF0` loads, per item, the text flags `0xD5024 := item.+0x0C` and the letter spacing `0xD502E := item.+0x0E` (`0x34DFC..0x34E10`). If the item's style has bit 0x4000 (byte +9 bit 0x40, `0x34E19`), it toggles the highlight bit 0x8000 of the flags (`0x34E1F`). The menu `0x35B60` sets 0x4000 (and clears 0x8000) on the item it currently handles, and records a kind-0x80 item as the text focus `0x84D50`. For a kind-0x80 item (`0x35636..0x356EE`):
1. print the item's string (`item.+0x10`) at (item x, item y) with those flags;
2. if the highlight bit is now set: measure the string with `0x33F60` (sum of glyph advances plus spacing) and print the fixed string `0xBB664` = `"$$"`, i.e. one literal `$` character (font code 0x24), at (item x + string width, item y).

So the caret is drawn only on the focused field, directly after the last character, on the same baseline. No blink timer exists: the caret is repainted (`0x34BF0`) after each key [L, no timer found in the edit path].

**Glyph [C, rendered from ICONS2/ICONCS.DAT].** Font code 0x24 is not a dollar sign. It is a **left-pointing solid wedge** (◄), 8 px wide and 13 px high (rows 5..17 of the cell), pointing at the text.

**Colour [C].** In the glyph painter `0x31610` a highlighted character is drawn as a **solid silhouette** (blitter colour mode 1..0x7F, `formats.md` §8.4; the table `0x83B3D` is indexed with k·4 at `0x13E6A`):
- dark text (flag 0x40 set, the default `$t`, used in parchment input windows): the dark glyph in solid colour **1** = table entry 1 = palette 0x25 = **RGB (190, 36, 36), red**;
- gold outlined text (after `$g`): outline as usual, centre glyph in solid colour **9** = entry 9 = palette 0x01 = RGB (255, 255, 121), pale yellow.

The same flags apply to the string printed in step 1, so on parchment the **whole name being typed is red, followed by a red ◄**. (Correction to `rules-16b` §3 / `formats.md` §8.4: the highlight value 9 is a solid colour, not "remap ramp 9", because it is below 0x80.)

**Edit rules (same loop, `0x35B60` around `0x36030..`) [L].** Accepted keys: a–z, A–Z, 0–9, space, `- . + & ' : \` and CP437 ä ö ü Ä Ö Ü ß (`0x84 0x94 0x81 0x8E 0x99 0x9A 0xE1`). A key is added only while the length is below the maximum `item.+0x20` and the pixel width (with this glyph's advance plus spacing and margin) still fits the item width `item.+0x04`. Backspace or Left (scan 0x4B) deletes the last character. Esc or Home (scan 0x47) clears the field. Enter is accepted only when the length is at least `item.+0x1C`. With `0x84DF5` set, a validator `0x2D7D0` checks every keystroke and the final Enter.

Answer: the caret is font glyph 0x24 (a solid ◄ wedge, 8×13 px) printed via the string `"$$"` at item x + text width, item y. It is shown only on the focused kind-0x80 item, which has its highlight bit toggled on, so the caret and the typed text are both drawn as solid colour 1 = palette 0x25 (190, 36, 36) red on parchment, or solid colour 9 (pale yellow) in gold text. It does not blink; it is repainted after every key.

## 4. History feast quality word

History screen `0x17930`, code 2 (`0x182C2`) [C]: text `0xBA620` `[EXE text]` with the record's year (+5) and the word `[0xB63AC + 4 · (+7)]`. The table at `0xB63AC` has six entries: 0 `0x9DDB8` furchtbares, 1 `0x9F02C` ziemlich mäßiges, 2 `0x9F03D` durchschnittliches, 3 `0x9F052` nettes, 4 `0x9F059` wirklich gelungenes, 5 `0x9DD8E` grandioses. The feast scene `0x20530` uses the same table at `0x208B5` for its closing line `0x9F015` "... ein %s Fest!". The value it indexes with is the quality q from `rules-08` §2.2: 3 − the two shortage terms + guest byte / 2, held in one local. The history write `0x2D750(owner, 2, q, 0)` at `0x208FA` passes that same local as `a`, so the history byte +7 is q [C]. No range check in either place.

Answer: the table is `0xB63AC` (furchtbares, ziemlich mäßiges, durchschnittliches, nettes, wirklich gelungenes, grandioses), indexed by +7 = q. The sim's guess is correct.

## 5. Spiel beenden and slot 5

`0x4EFC0` is the options-flags screen (Geschichtsschreibung … Spielende um 1800) and has no quit item. "Spiel beenden" is item 2 of Spieloptionen `0x4D1E0`: Laden, Speichern, Spiel beenden, Neuer Spieler, Musik, Grafik, Optionen. That item calls `0x4D880` with save flag 1 (`0x4D476`) [C].

`0x4D880` [C]:
1. Box `0xBDC98` `[EXE text]` (Ja/Nein). Any answer other than "Ja" returns to the game.
2. On "Ja": if the save flag is set, `0x42360(5)` (`0x4D8DC`/`0x4D8E1`) saves the game to slot 5.
3. Then `0x421A0` saves CONFIG.DAT, items are freed, windows closed, and the credits roll. The credits choose their song with `rand(5)` from `0x87064` and the first text page with a `rand`.

The other two callers pass flag 0, so no game save:
- the history screen after a finished mission, "Weiterspielen? Nein" (`0x18A6A`);
- the title/main menu quit (`0x4B11F`).

Note [L]: the game loop at `0x4B145` also calls `0x42360(5)` and `0x421A0` once per pass while year < 2000. So slot 5 is written regularly anyway, which makes it an autosave ("letztes Spiel fortsetzen").

Answer: yes. Spieloptionen → Spiel beenden → "Ja" saves the game to slot 5, then CONFIG.DAT, before the credits. "Nein" saves nothing. The quit paths from the history screen and the main menu do not save the game.

## 6. Initial emperor number

The type-1 record is created by `0x18A80` [C]: if `rec_find(1, 0xC0, 0xFF)` finds nothing, `rec_new(1, 0xC0, 0xFF, 1)` and **+4 := 0**. Its only call site is `0x4A4B5`, the fourth instruction block of `world_init` `0x4A490`. It runs right after year := 1600 (`0xEF4B8 = 0x640`), `0xEF4C5 := 0` and `0xEF4C6 := 0xC0`, and before the property loop and any RNG draw. (Ghidra has no function at `0x4A490`, so its xref list is empty; a byte scan finds this single call.) No other code writes +4 except the year-end increments in `0x18AE0`: +1 in 1612, 1619, 1637, 1657 (`0x64C`, `0x653`, `0x665`, `0x679`). Creating the record draws no random numbers.

Answer: the emperor number starts at 0 (created in `world_init` before any RNG use). The sim's value 0 is correct, and the [UNVERIFIED] in `rules-39` §10 can be dropped.

## 7. Battle frame loop details

Context: `0x389C0`, record table `0xD7DCC` (200 records × 0x21 bytes), men layer `0xDBD6C`, object layer `0xDC2D0`, record create `0x3D7C0`. The frame scan starts at `0x3AED4` (baseline B := −28, i.e. `12·row − 28` for row 0). Before the scan, `0x3AF0C` clears the done flag +0x1D of all 200 records. The unit activation runs **before** this point, so anything it creates (cannon balls `0x3A4B3`, type-12 records `0x3AB53`) is created before the frame's flag reset.

1. **Records created during the frame [C].** `0x3D7C0` sets the new record's done flag +0x1D to 0, its pixel y (+2) to the baseline of its cell row (`12·row − 28`) and its pixel x to `24·x` minus the type's x offset, and puts it into the object layer for types 0x0D–0x0F and 0x11–0x13, or into the men layer otherwise. The only create call inside the scan is the bomb drop (`0x3C379`): a man in state 6 (lay) reaching frame index 6 creates type 0x13 (facing < 4) or 0x12 (facing ≥ 4) in the object layer **at the bomber's own cell** (+4 = x, +6 = row). The scan then sets frame index 0 and length 18. The bomber is not walking, so his pixel y equals the row baseline and he is handled in pass 1 of that row. The bomb sits in the same row, has pixel y = baseline and a clear done flag, so **pass 2 of the same row handles it in the same frame**: it is drawn and its counter advances (18 → 17) in the frame it was dropped. After handling, every record's done flag is incremented, so nothing runs twice in one frame, including a walker whose layer cell moved into a later row at walk end (`0x389C0` case 3).
2. **Cannon balls in flight [C].** The ball (type 0x11) is created at activation in the object layer at its launch cell, with +2 = launch-row baseline, and keeps both for the whole flight. The drawn position is computed each frame from launch cell, target cell (+8/+0x0A), frame index and length (type-0x11 draw branch of the scan, parabola `(len − i)·i·160 / len²`) and is not stored in +0/+2. So a ball in flight is handled in **pass 2 of its launch row**, at the launch x, every frame. When its counter reaches 0 (end-of-animation state 0), it leaves the launch cell, enters the object layer at the target cell with +2 = target-row baseline, and turns into an explosion (type 13 state 8, or type 14/15 with `rand(2)` on a wall/obstacle cell). Its done flag is already set, so the explosion starts the next frame. A counter of −1 (ball drawn off-screen, x outside 0..679 or y outside 0..519) removes it (`0x3DC40`) instead.
3. **Captured man [C].** Capture happens in the activation, when the attacker's unit has tactic 7 (`0xD7542 = 7`) and the hit roll succeeds. The target is put into state 4 (fall) with its HP (+0x0C) untouched, +0x1E := captor record, and the capture counter of the captor's **participant**, `0xDB804[captor +0x10]`, is incremented (`0x397D2..0x397E0`; 4 entries, one per participant 0..3, zeroed in `battle_main`). It does **not** change type yet. During the fall animation it is still a type < 12 record of its old side in the men layer, so path modes 0/4/5 of the captor's side still treat its cell as a goal. When the fall animation ends (end-of-animation case 4), HP = 0 → removed (`0x3DC40`). HP > 0 → it leaves its unit's ten man slots and becomes **type 12**, with participant +0x10 := the captor record's +0x10 (`0x3D084..0x3D0A1`, via +0x1E), i.e. the captor's **participant index**, not a colour (corrected 2026-11-16, see `rules-51` §1.4). It is drawn with `0x80 + 5·0xDB7FC[+0x10]` (`0x3CEAC..0x3CEC5`), so it appears in the captor's team colour, side +0x11 flipped (xor 1), +0x1E := old type, state 0, pixel x += row mod 4. The unit byte +0x0F is **not** changed: it keeps the old (enemy) unit index.
4. **Mode 5 and prisoners [C].** `0x37C00` compares the side byte +0x11 and the unit byte +0x0F. After the conversion a prisoner has the captor's side and a unit index different from every unit of that side. In the base map it blocks its cell (same side), and in the mode-5 pass it counts as a "same side, other unit" record: approach cells next to it are blocked for the captor's Bombenleger. Before the conversion (during the fall) it counts as an enemy fighter, i.e. a goal.

Answer: yes, a bomb dropped in a frame is created with its done flag clear at its row baseline and advances in pass 2 of the same row in that same frame. A cannon ball in flight stays in the object layer at its launch cell and launch-row baseline (pass 2 of the launch row) until it lands. A captured man becomes type 12 of the captor's side only when its fall animation ends, not at the moment of capture; from then on path mode 5 counts it as a same-side record of another unit (it keeps its old unit byte).

## 8. Auction hall timing

Context: `auction_main` `0x26A50`, menu `0x35B60`, `delay_ticks` `0x12DB5`, `delay_ms` `0x6C083` (thunk `0x6B07C`), clock `0x6B6E5`.

1. **Clock [C, L for the DOS call].** `0x6B6E5` returns hundredths of a second since a stored base: `0x6B67F` builds `((h·60 + m)·60 + s)·100 + hundredths` from the DOS time call wrapper `0x6C3AC` (INT 21h AH=2Ch [L]) and adds 8 640 000 on a midnight wrap. So it advances in BIOS-tick steps of 5–6 hundredths (`rules-25` §5).
2. **Tick wait [C].** Each auction tick ends in the menu wait `0x35B60` (`0x26DC5`). Its timeout (EDX) is `t + 150` hundredths, where `t` (`[EBP−0x54]`) is set to 50 before the first tick (`0x26CCA`) and again right after every menu return (`0x26DD0`). The tick body sets it to 0 whenever it played auctioneer frames: after the hammer frames (`0x275AD`), after the return-to-rest frames (`0x272B0`), after the gesture frames (`0x2738A`). So the wait is **200 hundredths = 2.00 s** after a plain tick and **150 hundredths = 1.50 s** after a tick that animated. The menu returns early on a key press (it returns 0xA1 without consuming the key; the auction then reads it with `0x6B081`/`0x6AC77`) or on a click on an item. On a BIOS-tick clock the timeout ends at the first tick with elapsed ≥ the limit, i.e. up to one tick (54.9 ms) later.
3. **Frame steps [C, `rules-25` §1].** Each auctioneer frame step is `delay_ticks(5)`: one BIOS tick, 54.9 ms per frame, inside the tick (not counted in the menu timeout).
4. **Hammer count 2 [C].** After the hammer frames and the 0x4B hammer sample, when c = 2: `flip` `0x30400`, then `delay_ms(200)` (`EAX = 0xC8` at `0x275CC`, call `0x275D1`). `delay_ms` is calibrated at start-up against the DOS clock, so this is 200 real ms. Then the wait-until-mouse-button-or-sound-idle loop (`0x12DE4` / `0x12C71`) runs, for every hammer count.

So one hammer count costs: frames × 54.9 ms + (200 ms if c = 2) + the wait for the speech to end, then up to 1.50 s of menu wait before the next tick. A bid tick without animation costs 2.00 s, with animation frames × 54.9 ms + 1.50 s. There is no 100 ms step anywhere in the auction.

Answer: tick delay = menu timeout of 200 hundredths (2.00 s), or 150 hundredths (1.50 s) after a tick that played auctioneer frames, cut short by a key or click; frames 54.9 ms each (one BIOS tick); hammer count 2 adds `delay_ms(200)` = 200 ms of real time before the speech/click wait. The sim's 100 ms floor has no counterpart.

## Summary

| item | answer | section |
|---|---|---|
| Text overflow in scrolling windows | on overflow the window first waits for a click, but only once a full window of unread text has built up (`s + y + lineheight > top + h`). Then it scrolls up smoothly by the overflow (one line, 24 px) in steps of the text-speed setting (default 8 px, i.e. 3 steps, each step repaints the edges and rolls and flips, with no timer delay), keeping its contents (no clear). Only `$P` clears and restarts at the top, without waiting. The two print waits are `$V` at `0x32758` (only if something was printed or scrolled since the last wait) and `$W` at `0x32771` (unconditional). | §1 |
| Caption and say_box waits | `0x12C71` = "sound off or no game sample playing". `say_box` waits until its speech has ended if the speech is running when the text is up; otherwise it waits for a click, never for a fixed time. `caption` waits `delay_ms(n)` when n > 0, otherwise for a click, never for speech. The 2 s comes only from caller code of the form "idle → delay(2000), else until idle", i.e. `speech_running ? speech_rest : 2000 ms`, plus any separate fixed delay. `max(speech, 2 s) + n` is therefore wrong for speech shorter than 2 s: the original waits the speech length only. | §2 |
| Name-entry caret | the caret is font glyph 0x24 (a solid ◄ wedge, 8×13 px) printed via the string `"$$"` at item x + text width, item y. It is shown only on the focused kind-0x80 item, which has its highlight bit toggled on, so the caret and the typed text are both drawn as solid colour 1 = palette 0x25 (190, 36, 36) red on parchment, or solid colour 9 (pale yellow) in gold text. It does not blink; it is repainted after every key. | §3 |
| History feast quality word | the table is `0xB63AC` (furchtbares, ziemlich mäßiges, durchschnittliches, nettes, wirklich gelungenes, grandioses), indexed by +7 = q. The sim's guess is correct. | §4 |
| Spiel beenden and slot 5 | yes. Spieloptionen → Spiel beenden → "Ja" saves the game to slot 5, then CONFIG.DAT, before the credits. "Nein" saves nothing. The quit paths from the history screen and the main menu do not save the game. | §5 |
| Initial emperor number | the emperor number starts at 0 (created in `world_init` before any RNG use). The sim's value 0 is correct, and the [UNVERIFIED] in `rules-39` §10 can be dropped. | §6 |
| Battle frame loop details | yes, a bomb dropped in a frame is created with its done flag clear at its row baseline and advances in pass 2 of the same row in that same frame. A cannon ball in flight stays in the object layer at its launch cell and launch-row baseline (pass 2 of the launch row) until it lands. A captured man becomes type 12 of the captor's side only when its fall animation ends, not at the moment of capture; from then on path mode 5 counts it as a same-side record of another unit (it keeps its old unit byte). | §7 |
| Auction hall timing | tick delay = menu timeout of 200 hundredths (2.00 s), or 150 hundredths (1.50 s) after a tick that played auctioneer frames, cut short by a key or click; frames 54.9 ms each (one BIOS tick); hammer count 2 adds `delay_ms(200)` = 200 ms of real time before the speech/click wait. The sim's 100 ms floor has no counterpart. | §8 |

## Corrections to other docs

- `rules-16b` §3 / `formats.md` §8.4: highlight value 9 is a solid colour (pale yellow), not "remap ramp 9"; values below 0x80 are solid colours (§3).
- `rules-39` §10 (`world.rs:164`): the emperor number starts at 0, created by `0x18A80` from `world_init` at `0x4A4B5`; drop the [UNVERIFIED] (§6).
