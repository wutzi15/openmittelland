# Open questions, round 5 (rules-44)

Four follow-up questions: CONFIG.DAT persistence and the sound/film options, the battle tactics hotkey, the election presentation, and when the `%F` rival is computed. Conventions as in `rules-42` (header): `rand(n)` = `0x36FF0`, `wait_click` = `0x36720`, `print` = `0x322C0`, `menu` = `0x35B60`, `box` = `0x368E0`. CONFIG offsets are relative to the image at `0x86954`. Tags: **[C]** instruction level, **[L]** decompiler or xref only, **[UNVERIFIED]** inferred. Everything is [C] unless marked. Each section ends with an "Answer:" paragraph.

Note on tooling: Ghidra's xref list for `0x421A0` misses seven call sites in code it never disassembled. The call lists below come from a scan of the whole code object for `E8` calls to the target.

## 1. CONFIG.DAT persistence

### 1.1 Who writes CONFIG.DAT

All 13 direct calls of `0x421A0`:

| site | context | when |
|---|---|---|
| `0x4EBF5` | "Musik" options screen `0x4E810` | on leaving the screen (any click that is not one of its 9 items, `0x4EBEA..`), whether or not anything changed |
| `0x4D82F` | "Speichern" `0x4D630` | after a confirmed game save (`0x42360`), to store the slot name at config `0x01 + 0x30·slot` |
| `0x4D8E6` | "Spiel beenden" `0x4D880` | on "Ja", before the credits (`rules-42` §5) |
| `0x4B14F` | main game loop | once per pass while year < 2000, right after the autosave `0x42360(5)` (`rules-01`) |
| `0x4B2CA` | main game loop | after a graphics-mode switch requested by the Grafik screen (`0x8692D` ≠ 0): mode byte config `0x000` := `0x8692D`, video and icon re-init, then save |
| `0x4EEEF` | "Grafik" screen `0x4EC10` | after a successful graphics install ("Installieren"), which ORs flags into `0x86BB7..` |
| `0x172EB`, `0x17369` | installer (`0x16420` copy, "Mittelland AD wurde erfolgreich installiert") | after install; `0x17369` follows the sound setup `0x15630` |
| `0x4B5BC`, `0x4B68C`, `0x4B6C0`, `0x4B702`, `0x4B88B` | start-up code `0x4B4E0..0x4B8F0` | start-up branches that force a graphics mode (config `0x000` := 3, 0x93 or 2, plus memory dword `0x86BC8`) or run the first sound setup `0x15630`. Which command-line switch or first-run test picks each branch is [L]. |

The "Grafik" screen (`0x4EC10`, Filmgröße and Scrolling) and the "Optionen" screen (`0x4EFC0`, option bits) never save on exit. Their changes stay in memory and are written by the next save in the table: leaving the Musik screen, saving a game, quitting with "Ja", or the yearly autosave. A change is never saved on its own.

### 1.2 Musik screen `0x4E810` fields

Layout string `0xBDFAC`: rows "Digital:", "Musik:", "Sprache:", each with a "−" item, a level bar and a "+" item, then "Sprachausgabe: Aus/Ein" (labels `0xB7390`), "Soundcheck" and "Soundkarten...". Item indices (`menu` result − first item):

| item | effect |
|---|---|
| 0 / 1 | Digital −/+: byte `0x86AE8` |
| 2 / 3 | Musik −/+: byte `0x86AE9` |
| 4 / 5 | Sprache −/+: byte `0x86BCC`, then copied to `0x8457C` |
| 6 | Sprachausgabe toggle: bit 0x80 of `0x86AD1` |
| 7 | Soundcheck (speech `0227900`, `rules-16`), only if `0x86BC8` > 800000 |
| 8 | Soundkarten: re-detect, `0x15630(0x86AD5)`, driver init `0x12A08(0x86AD5)` |

| setting | CONFIG offset | range | down | up | shipped default |
|---|---|---|---|---|---|
| Digital (sample) volume | `0x194` | 0..127 | v ≤ 8 → 0, else v − 8 | v ≥ 119 → 127, else v + 8 | 0x7F = 127 |
| Musik volume | `0x195` | 0..127 | same | same | 0x7F = 127 |
| Sprache volume | `0x278` | 0..255 | v ≤ 8 → 0, else v − 8 | v ≥ 247 → 255, else v + 8 | 0xAF = 175 |
| Sprachausgabe | `0x17D` bit 0x80 (dword `0x86AD1`) | 0 = Aus, 1 = Ein | toggle | | set (byte 0x17D = 0xFF) |
| Filmgröße (Grafik screen) | `0x273` | 0 = "Lichtspielhaus", 1 = "Blackbox" (labels `0xB7398`) | `xor 1` | | 0 |

Defaults are read from `orig-install/MITTELL/CONFI2/CONFIG.DAT` (633 bytes).

- The level bars draw 0x86AE8 and 0x86AE9 as they are, and the speech level as `0x86BCC / 2`, so all three bars span 0..127.
- With steps of 8 from the default 127, the reachable values are 127, 119, 111, …, 7, then 0. Going up from 0 they are 0, 8, …, 120, then 127. Speech is similar with 255 as the top.
- **Mixer.** After each Digital/Musik change the screen calls `0x10CB7(AL = 0x86AE8, DL = 0x86AE9)`, which writes the Sound Blaster mixer: register 0x04 (voice) from the digital byte and 0x26 (FM/MIDI) from the music byte on the SB-Pro path, or registers 0x32/0x33.. on the other path (`0x801A2` selects the path). The driver init `0x12A08` reads its record from `0x86AD5` (config `0x181`): +0x13 = `0x86AE8` → `0x801AD` and +0x14 = `0x86AE9` → `0x801AE`, then calls `0x10CB7` (`0x12B11`). Normal start-up calls `0x12A08(0x86AD5)` at `0x4B8B5`, so the stored levels are applied at start. This corrects `rules-14` "no volume anywhere": samples have no per-voice volume, but these master levels exist.
- **Speech level.** `loadconfig` copies `0x86BCC` into `0x8457C` (`0x42186`), and the speech preload `0x15100` reads it (`0x1536C`). How the driver uses it is [UNVERIFIED].
- **Speech forced off.** Each redraw of the screen clears bit 0x80 when config byte `0x184` (`0x86AD8`, the card type copied to `0x80184` by `0x12A08`) is 0, i.e. when there is no digital sound card.
- **Filmgröße readers.** Only two: the intro at `0x18884` (0 → `INTRO.FLC` full size, otherwise `INTROB.FLC` small, `formats.md` §9) and one film in `turn_end_settlement` at `0x639FD`. Both test the whole byte for 0. No other film call reads it [L, xref].

Answer: (a) `0x421A0` runs on leaving the Musik screen, after a confirmed game save, on "Spiel beenden" → "Ja", in the yearly autosave pass of the main loop, after a graphics mode switch or install, and in installer and start-up paths. It does not run after each change. Changes on the Grafik and Optionen screens are saved only by the next of these events. (b) Digital volume = byte 0x194 and music volume = byte 0x195, both 0..127 in steps of 8 with a clamp at 0 and 127, default 127. Speech volume = byte 0x278, 0..255 in steps of 8 with a clamp at 0 and 255, default 175, displayed halved. Speech on/off = bit 0x80 of byte 0x17D, default on, forced off without a sound card. Film size = byte 0x273, 0 "Lichtspielhaus" (default) or 1 "Blackbox". The app's 0..16 scale with defaults 16, 16, 11 matches value/8 for digital and music (127/8 = 15.9) and value/16 for speech (175/16 = 10.9), but the original stores the raw bytes. A port that writes CONFIG.DAT must store the bytes.

## 2. Battle tactics hotkey

**Where the poll is.** It sits in the frame loop of `activate` `0x389C0` (`rules-07` §5.11), at the end of every frame (`0x3D5F7..0x3D68C`). Order within one frame:
1. step logic of the unit's men;
2. object scan and draw;
3. in display mode only: status strip, then wait for the next BIOS tick (`0x3D51E`, INT 1Ah loop);
4. the key poll.

ESC mode skips step 3 (`0x3D474` jumps straight to the poll), so the poll runs **once per animation frame in both modes**.
- If a key is waiting (`0x6B081`), it is read with `0x6AC77`. A 0 code (extended key) is read a second time.
- Key 27 sets ESC mode (`0xDC83F := 1`).
- When not in ESC mode, the key is compared with the attacker's hotkey (person `0xDB7F4`, byte +0xF5) and the defender's hotkey (person `0xDB7F5`, +0xF5).
- A matching key is queued in a local byte `q` (cleared at function entry, `0x389D5`): attacker `q := (q << 2) | 1` unless `q & 5`; defender `q := (q << 2) | 2` unless `q & 0xA`. So each side is queued at most once, in press order.

**When the menu opens.** Only after the activation's frame loop ends, i.e. when no object animated in the last frame (`[EBP−0x10]` = 0 at `0x3D68C`). The loop is `while q ≠ 0`:
- if `q & 4`: tactic menu `0x37100(side 0)`;
- if `q & 8`: `0x37100(side 1)`;
- then `q := (q << 2) & 0xC`.

This opens the queued menus in the order the keys were pressed. Afterwards `0xDC83A := 1` (redraw the strip) and `activate` returns. A key pressed in the middle of a unit's activation therefore opens the menu when that unit finishes, before `battle_main` runs its post-activation checks (held entrances, men alive) and before the next unit acts. If ESC is pressed after a hotkey, the queued menu still opens (the menu loop does not test ESC mode).

**When the tactic takes effect.** `0x37100` writes the unit's tactic byte (unit record +0x0E) directly, so the change is immediate. The new tactic is read the next time each changed unit is activated. Units later in this round's list use it this round; units that already acted use it next round. A computer-controlled unit (`rules-07` §5.7) re-chooses its tactic at activation, which overrides a menu change. Whether the menu refuses a side without a living leader is in `rules-07` §5.8 [L].

**Battle start.** `0x37100` has exactly two call sites, both in this poll (`0x3D725`, `0x3D779`). Before round 0 `battle_main` shows only the caption `0xB4219` `[EXE text]` (`0x36C90`). No menu opens on its own. The draw-only calls of `0x389C0` during deployment (`BL` ≠ 0) run the same frame tail, so a hotkey pressed then also opens the menu [L].

**RNG.** The poll, the key compare and `0x37100` call no `rand`. The menu itself draws nothing. The only RNG effect is indirect: the chosen tactic changes later game-rule draws.

Answer: the original polls the keyboard once per animation frame of the current unit's activation, after the frame's draw and tick wait, in display and ESC mode alike. Hotkeys are queued, at most one per side, in press order (one key per frame; if both humans share a key, one press queues both sides, attacker first; rules-71 §5). The menu opens when the current unit's activation ends, not at a round boundary, and never automatically before round 0 (only the "Taktik-Menü: Hot-Key" caption is shown). The new tactic is written at once and is used from each unit's next activation, which can be later in the same round. No RNG draws are involved. Correction: `formats.md` §11.5 "set-up phase before each battle round" is wrong; the menu is hotkey-only.

## 3. Election presentation

What `rules-12` §3.7 and `rules-15` §3.4 already give (texts, arguments, layout) is not repeated. New here are the selection conditions, all inside `year_end_world` `0x66220`.

**Inputs per vacant office** (set up at `0x66DD9..0x672AD`):
- **E** (`[EBP−0x18]`): the OR of the controller nibbles (person +0x10 & 0xF0) of all electors. An elector is each appointer's holder, or the persons blackmailing that holder instead (`rules-03` §4.6). E ≠ 0 means at least one human elector.
- **A** (`[EBP−0x3C]`): the OR of the controller nibbles of every **applicant**. This includes applicants that a privileged candidate shuts out: `0x6723D` ORs the nibble even when `0x67215` skips adding the applicant to the list. Candidates drawn by the computer rule (no applicants) never set A; they are computer persons only.
- **x** (`[EBP−0x4C]`): the privileged candidate, or 0xFF (`rules-03` §4.6: must have applied and not be in prison).
- **n**: the number of candidates.
- **Rival snapshot** (§4): array R of (human h, rival r) pairs built at the start of `0x66220`. Every "is c some human's rival" test below scans R and takes the first matching h.

**Branching at `0x67636`:**
1. `n ≤ 1` and x = 0xFF → the no-election path at `0x68431` (step 6 of `rules-12`):
   - n = 0: the office stays vacant and nothing is printed;
   - n = 1: the winner w is the sole candidate. **0xC1998** "Mangels weiterer Bewerber…" is shown if A ≠ 0 (here that means w is human) or w is in R. Arguments (w, h or 0xFF, office noun, L). Office noun = text id 72 + o if person +0x00 = 0, else 1407 + o.
2. Otherwise an election is held: votes are cast and the tie is broken. It is **displayed** only if `E | A` ≠ 0, i.e. there is a human elector or a human applicant.

**Displayed election** (sequence of `rules-15` §3.4, with these details):
- The place picture `0x66160(L, church, rand(2))` and its `rand(2)` happen only on this path (`0x6765D`). A hidden election does not draw it. This is a presentation draw that matters for RNG parity.
- Candidate lines 0xC17F8, or 0xC1818 when c is in R. After each line the bullet's alternate glyph is set to 0xCD (`0x34B80(item, 0xCD)`, `0x6780C`) for **every** candidate, unconditionally, not only when there is a privileged one.
- **Privileged line 0xC183C** `[EXE text]` with (x, x, x): printed exactly when x ≠ 0xFF (`0x67829`), directly after the candidate list. Only x is listed then, because the other applicants were not added. This line appears even when x is the only candidate (n = 1), because x ≠ 0xFF forces the election path.
- `wait_click` (`0x6784F`) follows the candidate list **in every displayed election**, with or without the privileged line.
- Votes, then the result: if exactly one candidate had the most votes, **0xC1934** "Damit wird %1F %s in %o." (w, h/0xFF, office noun, L); if several tied, **0xC1954** `[EXE text]` (w, h/0xFF). h comes from R in both. The tie-break `rand(k)` (k = number tied) is drawn **always**, also when k = 1 (`0x68159`), and before the display test.

**Hidden election** (`E | A` = 0, `0x68332`): the votes and the `rand(k)` happen as usual and nothing is shown, except that **0xC1978** "%1F wird %s in %o." is printed if the winner is in R. Arguments (w, h, office noun, L). It is preceded by the place picture `0x66160(L, church, religion of w)`, which draws no rand, and followed by flip and `wait_click`. There is no sound. So 0xC1978 means `[EXE text]`. That includes a privileged candidate elected alone. Correction to `rules-12` §3.7 step 6: it is not "one candidate drawn by the computer rule". Drawn candidates (no applicants) reach 0xC1978 only through a hidden election with n ≥ 2, or 0xC1998 when n = 1.

**`[EXE text]` list** (`0x6A5AD..0x6A7E6`). It runs on **every** year end: `0x6A520..0x6A7E6` contains no year test. So it should also run in 1600, when the vacancy cycle itself is skipped (`rules-03` §4.6) [L: that the 1600 year end reaches `0x6A520` was not traced]. Position: in the year-end report window, after the indulgence (Ablass) lines (`0xC1F7C`) and before the chronicle (`0x18AE0`) and the "keine besonderen Vorfälle" line.
- Loop: places 13 down to 0. For each place, office index i from `9 + 3·(L ≥ 10) − 2·(L = 13)` (city 9, county 12, principality 10) down to 0.
- For each (L, i), scan persons 0..191 for the first **human** with `office_eligibility(p, L, i) = 2` (`0x47C20`). The list depends only on eligibility, not on applications.
- If such a human is found:
  - print the header **0xC1FD4** `[EXE text]` once (flag `[EBP−0x28]`);
  - fold grouped seats: while i ∈ {1, 2}, or L < 10 and i = 8, decrement i. The group base is index 0 (3 Ratsherren / Geheimräte / Hofräte) or index 7 in a city (2 Priester);
  - print **0xC2004** `%s in %o$N` with (plural name `0xB4A28[g]`, L) when i = 0, or i = 7 in a city ("Ratsherren in …", "Geheimräte in …", "Hofräte in …", "Priester in …");
  - otherwise print **0xC2010** `Der %s von %o$N` with (singular name `0xB4378[g]`, L);
  - g = `0x43FF0(L, i)`, the global index. The outer decrement then moves past the group, so each office or group is listed at most once.
- If the header was printed: `wait_click`, then `$P`.

This settles the `rules-12` [L] on which table goes with which office kind.

Answer: the privileged line 0xC183C (x, x, x) appears exactly when a privileged candidate exists and the election is displayed (human elector or any human applicant, including applicants the privilege shut out), right after the candidate list and before the click that every displayed election has. Rival variants (0xC1818, and h in 0xC1934, 0xC1954, 0xC1978, 0xC1998) use the first matching human in the year-start rival snapshot. 0xC1978 is the hidden-election message: shown only when no human elects or applies and the winner is some human's rival. 0xC1998 is the single-candidate message: shown when the only candidate is human or a human's rival. The "nächstes Jahr" list is printed every year end, including 1600, for each office or seat group that at least one human could be promoted to (eligibility 2): plural form "%s in %o" for the grouped seats, "Der %s von %o" otherwise, then one click. RNG notes: the header picture's `rand(2)` is drawn only for displayed elections, and the tie-break `rand(k)` always.

## 4. Rival form timing

**Snapshot.** `0x66220` builds R at its very start (`0x6625D..0x66312`), after `0x43E60`. For each human h in slot order:
1. `r = human_rival_of(h)` (`0x44F10`). It draws `rand(10)` only when h has no office and no branch (`rules-40` §3).
2. Rival friction: `rand(3)` and `favour_decrease(h, r, ·)` (`0x436F0`).

R is never recomputed during the year end.

**Dismissal** (0xC170C / 0xC1734 / 0xC1790, plus the "show only if …is a human's rival" test). These read R (`0x66638`, `0x6671A..`); there is no `0x44F10` call. RNG draws between the snapshot of h and the print for office holder p (person loop `0x66338..0x6698D`):
- friction `rand(3)` of every later human (and `rand(10)` from their `human_rival_of`, if it applies);
- for every person before p in index order: favour floor `rand(2)` per low favour (`0x663B1`), the dismissal `favour_decrease` calls, the wealth-drift `rand` calls at `0x668DB` and `0x668ED` plus the calls `0x43BE0` and `0x42EA0` (`rules-03` §4.4; whether those two draw is [UNVERIFIED]), and the `office_vacate` `rand(34)` ×15 of each earlier dismissed computer holder;
- for p itself: its favour-floor `rand(2)` draws.

p's own `office_vacate` draws come **after** the print (`0x66832`).

**Election** (0xC1818, 0xC1934, 0xC1954, 0xC1978, 0xC1998, and adding rivals as candidates when nobody applied). These also read R; there is no `0x44F10` call. Every year-end draw between the start of `0x66220` and the vote intervenes. Besides everything above, that includes computer turnover (`rules-03` §4.5; that it runs before the vacancy cycle is [L], from the order in `rules-03`) and the earlier offices' votes and tie-breaks.

**Sabotage** (branch, 0xC1BB4 / 0xC1C54 vs 3139). Here `human_rival_of(t)` **is** called at print time (`0x68D08`). It comes after the damage rolls (`rand(40)` on +4 if +3 ≠ 0xFFFF, `rand(30)` ×2 on +0x23.., `rand(30)` ×6 on +0x27..) and the value difference, and directly before the message. No draw lies between the call and the print. Its own `rand(10)` cannot fire, because t owns the sabotaged branch, so `0x42B90` finds a branch [L: assumes `0x42B90` returns that branch]. The castle and camp texts use no `%F`.

Answer: sabotage: called at print time, after the damage rolls; no RNG draws between. Dismissal and election: the original never calls `human_rival_of` when printing. It uses one snapshot per human taken at the start of the year end, immediately before that human's friction `rand(3)`. All later year-end draws lie between it and the print: friction, favour floor, wealth drift, turnover, earlier dismissals' `office_vacate` and earlier elections. A port must take the rival snapshot at year-end start for dismissal and election messages, not at event time. Otherwise it can pick a different rival, because the friction and the dismissal favour changes alter the favours `human_rival_of` reads, and it would also move the `rand(10)` draw.

## Summary

| item | answer | section |
|---|---|---|
| CONFIG save points | leaving Musik screen, game save, quit "Ja", yearly autosave, graphics switch/install, installer/start-up; not per change | §1.1 |
| volumes | 0x194 digital, 0x195 music (0..127, step 8, default 127); 0x278 speech (0..255, step 8, default 175, shown /2) | §1.2 |
| speech on/off, film size | 0x17D bit 0x80 (default on); 0x273 0 Lichtspielhaus / 1 Blackbox (default 0) | §1.2 |
| tactics hotkey | polled once per animation frame in `activate`; menu opens when the current unit's activation ends; tactic used from each unit's next activation; no auto menu before round 0; no RNG | §2 |
| election display | displayed iff a human elector or human applicant; 0xC183C iff privileged; click after the list always; 0xC1978 = hidden election won by a human's rival; header `rand(2)` only when displayed | §3 |
| next-year list | every year end incl. 1600; offices with a human at eligibility 2; plural "%s in %o" for seat groups, "Der %s von %o" otherwise | §3 |
| rival timing | sabotage at print, no draws between; dismissal and election from a year-start snapshot | §4 |

## Corrections to other docs

- `formats.md` §3.1: `0x194` = digital volume, `0x195` = music volume (0..127, default 127); `0x278` = speech volume (0..255, default 175), copied to `0x8457C`; `0x273` = the Grafik screen's "Filmgröße" (0 Lichtspielhaus, 1 Blackbox), which is also what picks `INTRO.FLC`/`INTROB.FLC`; `0x17D` bit 0x80 = Sprachausgabe; `0x181..0x196` is the sound-driver record passed to `0x12A08`.
- `formats.md` §11.5: the tactic menu is not a set-up phase before each round. It opens only on a side's hotkey, at the end of the current unit's activation (§2).
- `rules-14` header: a master volume exists (SB mixer via `0x10CB7`, from config 0x194/0x195), set at driver init and on the Musik screen. "No volume, pan or priority" is true only per voice.
- `rules-12` §3.7: the "human involved" condition is [C]: a human elector or any human applicant (§3). Step 6: 0xC1978 is the hidden-election message, not "one candidate drawn by the computer rule". The next-year list's table choice is [C] (§3).
- `rules-15` §3.4: step 3's bullet glyph change to 0xCD applies to every candidate line; the click after the candidate list happens in every displayed election. §3.1: the election header's `rand(2)` is drawn only when the election is displayed.
- `rules-42` §5: `0x421A0` has more callers than `0x4B145`/`0x4D8E6` (§1.1).
