# Remaining [UNVERIFIED] items settled (rules-56)

Answers to the 2026-11-16 batch from `docs/IMPL-GAPS.md`. Tags as in `rules-50`.

### 1. Courtship gift texts (`0x212B0`, `0x2130A..0x2137C`)
- **Prices first** [C]: `P[i] = standing(target, 25)` for i = 0..2 (`0x212CE..0x21308`).
- **Words** [C]: one uniform draw per table, independent of the recipient, made in argument push order:
  1. `rand(6)` → `0xB57FC`;
  2. `rand(7)` → `0xB57E0`;
  3. `rand(7)` → `0xB57C4`.
- **Text** [C]: `0x963C3` with args (target, `B57C4[r]`, P0, `B57E0[r]`, P1, `B57FC[r]`, P2). The format supplies the leading E/e.

| gift (item) | table | entries (ids) |
|---|---|---|
| 1 (0) | `0xB57C4`, 7 | 1371..1377: inen Diamantring, in Paar goldbestickte Pantoffeln, inen Pelzmantel, inige Folterwerkzeuge, ine kostbare Bibel, in Reitpferd, inen edelsteinbesetzten Maßkrug |
| 2 (1) | `0xB57E0`, 7 | 1378..1384: inige Leibeigene, inen Falken, in Nachtgewand, ine Ferienreise ins Kurprinzentum Mallorcien, inen Mietminnesänger unter dem Balkon, inen Zweihänder, ine Streckbank |
| 3 (2) | `0xB57FC`, 6 | 1385..1390: inen Kerzenleuchter, ine chinesische Vase, inen Keuschheitsgürtel mit Gravur, inen Siegelring, in Gemälde von Euch, inen Kupferstich von Eurem Wohnsitz |

**Answer:** gift 1 = `0xB57C4`, gift 2 = `0xB57E0`, gift 3 = `0xB57FC`, each drawn uniformly. The draw order is gift 3, then 2, then 1, after the three prices. rules-05 §2 lists the tables in draw order.

### 2. Birth texts `0xBA6DC` / `0xBA700`
- These are the history-replay captions (history screen `0x17930`, case 4), not the birth scene. The call is `0x36E50(0,2,2,9, slot 0x38F + sex, 80, 200, 480, 160, text, u16 rec+5)` [C].
- The text is chosen by `rec+7` (sex, from `0x2D750(owner, 4, rec+4 & 1, 0)` at `0x22460`): 0 → `0xBA6DC` (Sohn), else `0xBA700` (Tochter).

**Answer:** `%l` is the year of the birth (history record +5), and it is the only argument. Sex selects both the text and the speech slot.

### 3. Mission replay feast words (history case 2)
- The call is `0x36E50(0,2,2,9, 0x38C, 80, 200, 480, 160, 0xBA620 "$C%l - Ihr gabt ein %s Fest...$N", u16 rec+5, 0xB63AC[rec+7])` [C].
- The feast handler stores its quality q (`0x2089E..0x208A8`) both as the caption word `0xB63AC[q]` and as the history `a` (`0x2D750(owner, 2, q, 0)` at `0x208FA`).
- Table `0xB63AC`, q = 0..5 (ids 2133..2138): furchtbares, ziemlich mäßiges, durchschnittliches, nettes, wirklich gelungenes, grandioses.
- For comparison, the concert (case 3) uses `0xBA644` with (year, rec+7, `0xB646C[rec+8]` = gänzlich hervorragender / begnadeter / außergewöhnlicher / recht guter).

**Answer:** the table is `0xB63AC`, indexed by the feast quality at history +7.

### 4. OUTRO vs OUTROB (`0x18884..0x189CE`)
- `cmp byte [0x86BC7], 0` [C]. `0x86BC7` is CONFIG byte `0x273` (Filmgröße, `rules-44`) [L for the option name].
- 0 → `film("flcfilme\outro.flc", 0, 24, 10)`. Anything else → `film("flcfilme\outrob.flc", 96, 100, 10)`. **Only one film plays.** This corrects rules-15 §7.8.
- During the film, either `music(0x1D, 0)` plays (draws `rand(4)`) when `0x86BC8 ≤ 0xC3500` or `0x86AD8 = 0`, or else speech 13 plays and is waited out.

**Answer:** CONFIG 0x273: 0 → OUTRO at (0, 24), non-zero → OUTROB at (96, 100).

### 5. `0xBCEFC` (denounce, angry) args (`0x479D2..0x479F6`)
- Only **L** is pushed [C], into box (160, 140, 320, 200). The text is `[EXE text]`.
- `%2N1` uses L. `%1N0` reads the next stack slot, which no argument fills: it is whatever lies on the stack above the pushed arguments [C for the push count; the value is undefined].
- The other denounce boxes push (L, P), so the original most likely meant P here. Using P is a reasonable choice, but it is not what the code does.

**Answer:** args = (L). The `%1N0` name is unspecified in the original. Use P.

### 6. Inquisition boxes (mode 7, `0x47678..0x4778E`)
- Existing record: `0xBCCD4` `[EXE text]` (T), msgbox **(120, 140, 400, 200)**. Answer 0 deletes the record [C].
- New: type-18 record (+4 T, +5 1), `0x85CBA |= 8`, then `0xBCD6C` `[EXE text]` (T), msgbox **(160, 180, 320, 120)** [C].

### 7. Pre-fight text without `$W` before a `$P` (e.g. `0xBBA98`)
- `$P` (`print` `0x325A5`) draws the pending line into the window buffer and resets the cursor. It then repaints the window buffer (`0x30880`, "partialclearwindow") **without `flip`**.
- `print` never flips by itself. Only `$W`/`$V` (`wait_click` flips first) or an explicit `0x30400` presents the window.
- Between `0xBBA98` and the next `$P` text there is no wait and no flip [C].

**Answer:** the line is never visible. The port should not show it.

### 8. Report/chronicle scrolling (rules-42 §1)
- Scrolling happens in the line painter, when a printed line overflows the window (`0x31E1C..0x31F70`), not in the menu loop. Each step does the following, with **no timer, delay or clock read** [C]:
  1. `d = min(y − h, step)`, `y −= d`;
  2. move the items up by d;
  3. repaint the 8-px edge strips (`0x30880`) and the rolls (`0x30670`);
  4. `flip`.
- The step comes from `W[0x84DF6 + 2·opt]` = 24 / 12 / 8 / 6 / 3 / 1 px, copied by `win_open` (`0x2FC25`). opt is `0x86A47` = CONFIG 0x0F3 (0..5, default 2 = 8 px).

**Answer:** no step timer. CONFIG 0x0F3 sets the step size, and the real speed is the machine's repaint rate. The original has no millisecond value.

### 9. Kontor music roll after a raid (rules-39 §5)
- The roll (`0x5D65F..0x5D6A1`) sits in the Kontor `0x5D610`, which has a single caller, `play_turn` phase 4 (`0x4B276`) [C].
- Human raids and attacks fight in `settlement_pre`, called from `turn_end_settlement` (`0x62A1C`, phase 5, after the Kontor was left). Play then goes to phase 6 and the next player.
- `battle_main` has no music call at its end. Its only draws are `music(0x1B, 4)` (`rand(3)`) and `music(0x1C, 5)` per activation [C].

**Answer:** no. A raid or attack is never followed by a Kontor roll. The next roll comes at that player's next Kontor entry, as usual.

### 10. Film placement and scaling (follow-up to item 4)

**Mode switch around every film [C].** Every film call is bracketed by the mode setter `0x12DFE`. The sequence is:
1. Before: `0x131C4` (mouse off), `mov eax, 1` (**VGA 320×200×8**), `0x12DFE`.
2. Play.
3. After: `0x12DFE(byte [0x86954])` restores the configured mode; then the caller reloads icons and rebuilds its screen.

The only exception is `hochzeit.flc` (`0x21AFA`), which switches to mode **2** (VESA 640×480×8).

**Player `0x14ED6(EAX = x, EDX = y, EBX = d, ECX = stop frame)` [C].**
- Position (`0x14EE4..0x14F1E`): in modes ≤ 1, x := x/2 and y := `0x835D6[y]` (the 480-entry logical-to-200-line table; 24 → 10, 100 → 41). In mode 2, x and y are unchanged. In modes ≥ 3, x is scaled to bytes per pixel.
- The frame is written **1:1** at `y·pitch + x` into video memory. There is no pixel doubling or scaling in the player. The "size" comes from the 320×200 video mode, which the display stretches to the full screen.
- Timing: each frame waits until `(n+1)·d` hundredths have elapsed. Esc returns 1, the normal end returns 0.
- The rest of the screen is whatever the mode switch left, i.e. the cleared (black) VGA screen [L: BIOS mode set clears video memory]. No frame icon is drawn.

**FLC sizes (headers) and content.** OUTROB.FLC and INTROB.FLC hold a single all-zero (black) 320×180 BRUN frame followed by 587/1583 empty frames. "Blackbox" therefore shows **a black screen** for the length of the film, while the sound or speech runs. INFORMNB.FLC is a real 224×116 film.

| film | call site(s) | mode | logical (x, y) → 320×200 | FLC size | d (fps) | result |
|---|---|---|---|---|---|---|
| intro / introb | `0x4ACBB`, `0x4AD81` (start-up) | 1 | (0, 24) → (0, 10) / (96, 100) → (48, 41) | 320×180 / 320×180 black | 10 | full-width film, centred / black screen |
| outro / outrob | `0x188E3`/`0x1890E` / `0x18984`/`0x189B2` | 1 | (0, 24) → (0, 10) / (96, 100) → (48, 41) | 320×180 / 320×180 black | 10 | same |
| informnt / informnb | `0x63A63` / `0x63ACF` | 1 | (0, 24) → (0, 10) / (96, 100) → (48, 41) | 320×180 / 224×116 | 10 | full width / small box centred at (48..271, 41..156) |
| feuer, lawine | `0x18F67` | 1 | (0, 0) | 320×200 | 12 (8.3) | full screen |
| belager | `0x1F4BF` (12), `0x1F901` (10) | 1 | (0, 0) | 320×200 | 12 / 10 | full screen |
| entlager | `0x1FCF7`, `0x1FD2A` (12); `0x1FFFE`, `0x20031` (10) | 1 | (0, 0) | 320×200 | 12 / 10 | full screen, twice |
| kerze | `0x65834` | 1 | (0, 0) | 320×200 | 10 | full screen |
| sabotage | `0x68F57` | 1 | (0, 0) | 320×200 | 10 | full screen |
| hochzeit | `0x21B28` | **2** | (0, 0) | 640×480 | 12 | full screen 1:1 |

For a 640×480 port, a mode-1 film is equivalent to scaling 320×200 to 640×480 (×2 horizontally, ×2.4 vertically). The 320×180 films then cover logical y 24..456 [L: equivalence through the `0x835D6` table].

**Outro speech 13 [C]** (`0x188F4..0x18920`, the same in the B branch).
- Used when `0x86BC8 > 0xC3500` and `0x86AD8 ≠ 0`; otherwise `music(0x1D, 0)` plays.
- Sequence: preload slot 13 (`0x15100`), start it (`0x154B0`), play the film. **If the film ended normally (return 0), the code waits in a loop until `0x12C71` (sound idle) is true**, then stops the speech (`0x15480`).
- After Esc (return 1) there is no wait, and the speech is stopped at once.
- Only then is the mode restored, and the game shows picture 0xBB and the "Wollt Ihr weiterspielen?" box.
