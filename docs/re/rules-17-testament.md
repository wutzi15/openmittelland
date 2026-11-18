# Death, notary, testament and funeral (rules-17)

Source: turn-end settlement tail 0x654A9..0x6613E (inside 0x61A70), heir list 0x44130, testament window 0x45A70, person clear 0x432D0, purge 0x42F60. Confidence tags as in `rules-00-map.md` ([C] instruction level, [L] decompiler reading, [UNVERIFIED]). Expands `rules-05` §7 and `rules-15` §7.4; where they differ, this document wins (differences listed in §8). Primitives (`picture`, `caption`, `say_box`, `film`, `region copy`, `delay`, `tick`) are those of `rules-15` §0; speech file names follow `rules-16` §1. `p` = the dying human (person index), `P` = its record (`0xE3A7C + 248·p`).

## 1. When it runs [C]

Last step of every human's turn-end settlement, after the debt check (`rules-02` §10) and the mission/history check (0x17930), unless the jury court ran this turn (it returns early, `rules-02` §10.3) or the player went bankrupt (§1.1):
1. `life -= 1` if `year < 1620`, else `life -= rand(3)` (one RNG draw only from 1620 on).
2. If `life` (signed byte +0x02) `> 0`: nothing more, phase 6.
3. Otherwise the death sequence below.

Computer persons never die; they are replaced by the turnover rule (`rules-05` §9). There is therefore no "computer heir choice": the only heir choice is the human's own testament (§4), and the default without any edit is the first-born child (the record with bit 7, set at scheduling when `m = 0`, `rules-05` §4).

### 1.1 Bankrupt screen (no branch left), for completeness [C]
Checked before step 1 of the settlement tail: no property of kind 0 → report window closed, items cleared, picture **0xBA**, samples 0x4E..0x51 loaded, `music(0x0B, 0)`, flip; four times: `sample_play(0x4E + rand(4))` (RNG draw), speech `(0 2 2 9 : 951..954)` loaded and started, caption box (flag 0 = click) `(40, 376, 560, 104)` 0xC1544 with (p); `(40, 380, 560, 80)` 0xC15A4; `(40, 380, 560, 100)` 0xC1620; `(40, 380, 560, 100)` 0xC167C with (p); speech popped after each box. Samples 0x51..0x4E unloaded, purge mode 1, person cleared, window closed, phase 6. Life is not touched.

## 2. Candle and gravestone [C]

1. `music(9, 0)` (Trauermarsch).
2. Icon cache reset; `film(kerze, 0, 0, 10)` (`flcfilme\kerze.flc`, 10 fps, full screen); icon cache re-initialised. The film plays over the turn-end report screen; nothing is drawn before it.
3. Picture **0xC7** (B/ICONC7, graveyard with tombstone) at (0, 0) in the still-open full-screen turn-end window.
4. Print `$g$7H$35>$35<$Z` (0xC16E8): gold outlined text, line 7 (y = 12 + 7·24 = 180), left and right margins 35 units = 210 px each, blank line.
5. Epitaph bit: `b := 8`; up to 16 times: `r = rand(8)`; if bit `r` of the deed byte `P+0xF3` is set, `b := r` and stop. (So up to 16 RNG draws; with no deed bit set exactly 16 draws.) Deed bits: 0x01 mistress, 0x02 robber camp, 0x04 saboteur, 0x08 spy, 0x10 debtors' tower, 0x20 siege, 0x40 law violation, 0x80 bribery [G for 0x40/0x80, from the epitaph texts].
6. Print 0x9EB83 `$CHier ruht$Z$N%1N0$Z` with (p), then `T[0xB6330][2·b + gender(p)]`:

| b | male | female |
|---|---|---|
| 0 | ohne seine Mätresse | ohne ihren Geliebten |
| 1 | Seine Räuber taten alles für ihn. | Ihre Räuber taten alles für sie. |
| 2 | Viele Saboteure sind nun arbeitslos. | (same) |
| 3 | Seine Spione trauern. | Ihre Spione trauern. |
| 4 | Der Schuldturm wird lange leerstehen. | (same) |
| 5 | Belagern war seine Passion. | Belagern war ihre Passion. |
| 6 | Das Lesen der Gesetze war seine Stärke nicht. | …ihre Stärke nicht. |
| 7 | Bestechung war sein Leben. | Bestechung war ihr Leben. |
| 8 | Hoffentlich in Frieden | (same) |

Each line ends in `$|` (flush). No flip call here: the text goes to the screen through `wait_click` [L: the print path presents itself].
7. `wait_click`.

## 3. Notary [C]

1. Picture **0xC8** (ICONC8, scholar at a desk) at (0, 0); mouse pointer hidden (`0x2EC40(0)`); flip.
2. `len = heir_list(p)` (§3.1); `NW = max(0, net_worth(p))` (0x44640 then 0x441F0), computed **now**, before anything is paid.
3. **No children (`len = 0`)**: text cursor y := 1; speech `(0 2 2 3 : 908)` loaded and started; print 0x9B6BC `[EXE text]` with (p) (line 2, y = 60, centred, plain text colour); flip; `wait_click`; purge mode 1; person cleared (§6); speech stack popped; phase 6. No funeral.
4. Otherwise two off-screen 640×60 frames are allocated: frame A ("notary strip") receives a copy of screen rows (0, 0, 640, 60), frame B ("mourners' strip") a copy of (0, 420, 640, 60). The pointer hide counter is bumped around the copies.

### 3.1 Heir list (0x44130) [C]
Saves and restores the record-search state (12 bytes at 0xEF490). `h := 10` (global byte `0xEF4C2`). Walks the type-31 records owned by `p` in heap order; every record with counter (+3) `< 45` (born child) is appended to the pointer table `0xDC844[n]`, and if its flag byte (+4) has bit 7, `h := n`. After the walk, if `h >= n` then `h := n`. Returns `n` (= `len`). So `h` is the index of the first born child flagged as main heir, or `len` ("Das Erzbistum") when none is flagged. Unborn children (counter ≥ 45) are not heirs and are not listed.

## 4. The six notary steps [C]

For `i = 0..5`; at the start of each step the text cursor y is forced to 10 (`0xD501A`), so each notary line is printed at the top of the screen over the notary picture, inside the strip saved in frame A.

### 4.1 Steps 0..4: the speech
Speech slot (family 3, narrator) loaded per step, then played:

| i | speech | text `T[0xB5E7C][i]` | print args |
|---|---|---|---|
| 0 | `(0 2 s 3 : 900)`, `s = 0` only if `len = 1` and that child is a son (bit 0 of +4 clear), else `s = 1` | `$C>Liebe Hinterbliebene!<$N` | (p, NW) |
| 1 | `(0 2 2 3 : 901)` | `[EXE text]` | (p, NW) |
| 2 | `(0 2 2 3 : 902 + gender(p))` | `[EXE text]` | (p, NW) |
| 3 | `(0 2 2 3 : 904 + gender(p))` | `[EXE text]` | (p, NW) |
| 4 | `(0 2 2 3 : 906)` | `>Hier das Testament:<$N` | (p, NW) |

Quirk: for `i = 0` the child tested is `0xDC844[h]`; with `len = 1` and no heir flag, `h = 1` and the test reads a stale pointer [C, harmless: only chooses the recording].

All five lines are printed with both arguments on the stack; lines 0, 1, 4 consume none, line 2 consumes `p`, line 3 both. Then flip, then the wait: if `sound_idle()` is already true (speech off, or nothing playing) → `delay(2000)`; otherwise loop until `sound_idle()`. **No click is needed** (correction to `rules-15` §7.4). Then speech popped.

### 4.2 Step 5: the testament
1. flip; **testament window** `0x45A70(p, editable = 0)` (§5), shown read-only; pointer hidden again.
2. If `h >= len` ("alles dem Erzbistum"): speech `(0 2 2 3 : 909)` loaded and started; print 0x9B70C `[EXE text]` with (p) at y = 10; purge mode 1; person cleared; wait: idle → `delay(3500)`, else until idle; speech popped. The player is out. No mourners for this step; the loop ends after the strip restore.
3. Otherwise **succession** (§6), then speech `(0 2 2 3 : 907)` loaded and started; print 0x9B4AF `[EXE text]` with (p) — p's record already carries the heir's name and gender, so the heir is addressed; flip; wait: idle → `delay(3500)`, else until idle; popped. Then, if `len > 1`: `pay(p, NW / 5)` (the compulsory fifth; the money vanishes). Then purge mode 2 (§6.2).

### 4.3 Mourners (after each step)
After step `i`:
- `i = 0`, or (`i = 5` and `len = 1` and `h = 0`, the only child inherits): `delay(4000)`, no heckles.
- otherwise: `delay(2000)`; `k = 1` if `len = 0` else `k = 1 + rand(len)` (so `len` cannot be 0 here; `k` is 1..len). For each of the `k` lines, in this order:
  1. text cursor y := 421 (`0x1A5`);
  2. `a = rand(3)`: 0 → print `$L` (left), 1 → `$R` (right), 2 → `$C` and `y += 30` (centred lines sit at y = 451);
  3. `i < 5`: `line = rand(14)`, `voice = 15 + rand(4)` (18 → 1), `g = rand(2)`; speech `(voice, g, 2, 3 : line)` loaded and started; print `T[0xB5E94][line]`;
     `i = 5`: `line = rand(5)`, `voice = 15 + rand(4)` (18 → 1), `g = rand(2)`; speech `(voice, g, 2, 3 : 14 + line)`; print `T[0xB5ECC][line]`;
     RNG order per line: alignment, line, voice, gender (correction to `rules-16` §4, which has voice before line);
  4. flip; `delay(2000)`; then loop until `sound_idle()`; speech popped; print `$H$C$|` (0xC1704); the mourners' strip (0, 420, 640, 60) is restored from frame B.
- Then the notary strip (0, 0, 640, 60) is restored from frame A (also after the `i = 0` and only-child cases) and the next step begins.

Heckle texts: `0xB5E94` (14, files `voice g 2 3 : 000..013`): >Endlich!< / >Hurra!< / >Das wurde aber auch Zeit.< / >Darauf habe ich schon lange gewartet.< / >Ha-haaa!< / >Ich bin der wahre Erbe!< / >Mir alles!!< / >Das tut mir aber leid.< / >Ooohh...< / >Darauf einen Humpen!< / >Geizkragen!< / >Endlich bin ich reich!< / >Lasst die Vorrede!< / >Zur Sache, Notar!<. `0xB5ECC` (5, files `: 014..018`): >Verrat!< / >MIR ward alles versprochen.< / >Das ist fingiert!< / >Das Testament ist gefälscht!< / >Am Sterbebett wurde mir alles versprochen!<.

Voice digits: 15, 16, 17 → files `F…`, `G…`, `H…`; the roll 18 is mapped to 1 → `1g23nnn`. The gender digit `g` is random, not a person's.

After step 5 both frames are freed; phase := 6; the turn-end window is closed by the caller.

Timing summary for an implementer: step line → speech length (or 2 s), then 4 s (i = 0) or 2 s + heckles (each: speech length, 2 s pause before waiting). Nothing in the funeral waits for a click except the gravestone and the no-children screen.

## 5. Testament window 0x45A70(p, editable) [C]

Called read-only (`editable = 0`) from the funeral, editable (`= 1`) from the privileges menu entry 6 "Testament machen" (0x5F080; offered when p has at least one born child, `rules-04b` §4.3).

1. `win_open(40, 100, 560, 280, 0x8810)` (parchment message window).
2. Loop:
   - print `T[0xB6F00][0]` = 0xA9605 `[EXE text]` with `%m` = (style `0xA8 | (editable ? 0 : 0x400)`, pointer to the empty string 0xBC540): one centred cycling string item (kind 0xA0, `rules-16b` §6), flag 0x400 when read-only [UNVERIFIED: 0x400 = not clickable]. The print returns the item index `it`.
   - `len = heir_list(p)` (sets `h`).
   - `h < len`: the item's label is set (`0x34B80(it, name)`) to the child's name from the human-name table (`0xE377C + 24·slot`, slot = child record +5); then if `len = 2` print 0xA96A9 `[EXE text]`, if `len > 2` print 0xA96F8 `[EXE text]`.
   - `h >= len`: label := 0xA9768 `$hDas Erzbistum$h` (highlighted); no fifth-part line.
   - print 0xA975D `$R%1N0$N$B` with (p): the testator's name right-aligned as signature.
   - editable: an OK item (style 0x1690, glyph 0xCD) is added at (right − 32, bottom − 32) of the window (0x348D0).
   - flip; `c = menu(it, 0, &d) − it`.
   - `c = 0` (the heir item was clicked): if `h < len` clear bit 7 of record `0xDC844[h]` (+4 &= 1); `h := (len + h + 1 + d) mod (len + 1)`; if `h < len` set bit 7 on `0xDC844[h]`; new page (`0x30F10`, clears the window); items of the print removed; loop again.
   - otherwise: items removed, loop ends.
3. `win_close`.

`d` is the menu's step value for the clicked item: `+1` for a click in the upper half of the line, `−1` in the lower half [L, from the spinner rule of `rules-13` §1.3; the testament formula gives next heir for `d = +1`, previous for `d = −1`]. Index `len` is "Das Erzbistum". In the read-only call the item carries flag 0x400 and there is no OK item; the menu returns on any click [L: with no clickable item the menu's "clicked − first" is nonzero]. The funeral therefore shows the window until the player clicks.

What the window reads and writes: reads `P+0x00` gender (for the name), the type-31 records of p (counter, flag byte +4, name slot +5), the human-name table; writes only bit 7 of the children's flag bytes. Nothing else changes until death.

## 6. Succession maths (h < len) [C]

Let `H = 0xDC844[h]` (the heir's child record), `P` the dying person's record.
1. The old human-name slot is freed: the first byte of the name at `P+0x09` (pointer) := 0.
2. `P+0x08` (name slot index) := `H+5`; `0x42C40(p)` recomputes the name pointer `P+0x09` from the slot (humans: `0xE377C + 24·slot`).
3. `P+0x00` gender := `H+4 & 1`.
4. Skills `P+0xE0 .. P+0xEE` (15 bytes): `s := (s + 100) / 2` (C integer division of a positive value).
5. `P+0x04` spouse := 0xFF (the widow's own +0x04 still points at p; quirk kept).
6. `P+0x02` life := `H+3` (the child's counter = 44 − years since birth).
7. `P+0x07` insurance flags := 0.
8. (speech 907 and text, §4.2)
9. `len > 1`: `pay(p, NW / 5)` (0x43860; cash may go negative; nobody receives it). `NW` is the net worth frozen in §3 step 2.
10. Purge mode 2: deletes p's records of types 5, 6, 7, 18, 22, 23, 24, 28, 29, 30, 31, 50, 57 and the referencing records listed in `rules-05` §7.3 (all children records included, the heir's too).

Kept: cash, title (+0x0D), office/location (+0x0F, +0x10, +0x11), properties, favours of everybody toward slot p, religion (+0x03), colour, controller nibble, deed byte +0xF3, mission record (type 0 is not purged). The office stays with the slot; no vacancy is created.

### 6.1 No heir: person clear (0x432D0) [C]
Every property of p passes to a random computer person with location 13, or any computer person if none; the human-name slot is freed; the record is zeroed and byte 0 := 0xFF (slot free). Combined with purge mode 1 (all records of p and all records referencing p). The next-human search then skips the slot; the save continues with the remaining players.

### 6.2 Tax
There is no inheritance tax. The only deduction is the fifth part when 2 or more born children exist (§6 step 9). With one child everything passes untaxed; the Erzbistum option gives everything to a random principality office holder (§6.1), not to a church account.

## 7. What the sim must supply
- `life` decrement (§1), heir list order (heap order of type-31 records with counter < 45) and `h`.
- Net worth `NW` at the moment of the notary (after the debt stages, before the fifth is paid).
- Deed byte for the epitaph; `rand` sequence: epitaph (≤ 16 × `rand(8)`), then per step the heckle draws (`rand(len)`, then per line `rand(3)`, `rand(14|5)`, `rand(4)`, `rand(2)`). No draws in the testament window.
- The succession writes of §6 in that order; purge mode 2 afterwards.
- Presentation: picture ids 0xC7, 0xC8, film `kerze.flc`, strips (0, 0, 640, 60) and (0, 420, 640, 60), text positions (line 7 with 210 px margins; y = 10; y = 421/451), delays 2000/3500/4000 ms, speech files as listed.

## 8. Corrections to earlier docs
- `rules-15` §7.4: notary lines wait for the speech or 2 s (3.5 s for the testament outcome lines), never for a click; the `0x2EC40(0)` is the pointer hide (`rules-16b` §7); the mourner count is `1 + rand(len)`, the alignment roll comes first and the centred variant sits 30 px lower; after step 0 there are no heckles, only a 4 s pause.
- `rules-16` §4: mourner RNG order is line, voice, gender.
- `rules-05` §7.1: the editable testament has an OK item; the heir index moves by the menu step `d`.
- `rules-05` §7 step 4: the epitaph search sets `b = 8` before every retry, i.e. the last successful hit wins and an unsuccessful run ends with 8.

## 9. Open
- Item flag 0x400 semantics in the read-only testament (assumed "display only") [UNVERIFIED].
- The menu step `d` for cycling string items (+1/−1 by half of the line) [L].
- Whether `$|` lines print without an explicit flip before `wait_click` (gravestone) [L].
