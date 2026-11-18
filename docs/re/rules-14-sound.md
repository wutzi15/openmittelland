# Sound: samples, speech, music (rules-14)

Source: MITTELL.EXE disassembly (sound driver 0x10000..0x13200, game-side wrappers 0x15100..0x155F0, music front end 0x41E50) plus the shipped asset directories. Confidence tags as in the other docs: [C] traced in code, [L] read from a listing but not every branch followed, [UNVERIFIED] inferred or not checked. Sample "descriptions" below are inferred from the trigger and the sample length only; nobody listened to them.

Terminology: `rules-12` and `rules-13` write "sound cue n" / "sound 0x12" for calls of the **music** function 0x41E50 with song number n. Sample effects are called by their SMPL number here. Correction to `rules-12` §3.5: the auction screen plays music song 0x12 at open and **sample 0x4B** (hammer) when a lot is sold; "music 0x4B" there is a sample.

## 1. Sound API

### 1.1 Configuration and state [C]
- Sound card type byte `0x80184` (0 = no digital sound; 2/3/5/7 = Sound Blaster family, from `sndsetup`). MIDI device byte `0x80004` (0 = no music; 7 = MPU-401 and others). Set by the configuration record passed to the driver init 0x12A08; `SNDSETUP.BAT` / the "Soundkarten..." option (0x4E810 case 8) re-run detection (0x12237) and re-init.
- Sample rate word `0x8019E` (driver copy) / `0x86ADB` (config): 22050 or 44100. Directory digit: `smp4xkhz`/`spc4xkhz` if the rate is >= 44000, else `smp2xkhz`/`spc2xkhz` (`formats.md` §5.1). An SB16 (DSP version >= 4) is configured at 44100 with the 16-bit flag `0x801A0` bit 0 set; everything else 22050.
- Speech enabled: option dword `0x86AD1` bit 0x80 (toggled by the options screen 0x4E810 case 6, "Sprachausgabe"). With the bit clear every speech load returns an empty slot and the callers fall back to click waits.
- Music driver active: `0x82E70` bit 0 (set when a song has been started, cleared by stop). Current song byte `0x86C58` (0 = none started since the last stop).
- Low-memory build flag: `[0x86BC8] <= 0xC3500` means the "4-MB-Version" (the options screen says `[EXE text]` for the sound check and card setup when this holds). Several music calls are gated on it (1.4).
- Mixer: 32 voices of 0x26 bytes at `0x81B2F`; voice +0x1C = state, +0x1D = kind (0x10 = game sample, other values = samples triggered by the MIDI player), +0x1E = sample number. Counters `0x81B2D` (voices allocated) and `0x81B2E` (voices still playing; decremented by the interrupt mixer, which is not in the disassembled image). No volume, pan or priority exists anywhere in the API: every sample plays at full level, mono, and a request with all 32 voices busy is dropped silently.

### 1.2 Sample effects (SMPL files) [C]
- `sample_load(n)` 0x124E1: no-op if sound is off or slot `n` (table `0x82EA9`, one pointer per number) is already filled. Opens `<path>smpXxkhz\smpl_<xy>0.dat` with `xy` = n as two lowercase hex digits and the **variant digit always 0**. Reads the 20-byte header, allocates DOS memory, copies the PCM. If the file is missing the slot stays empty and every later play of `n` is silent (this happens in the original: see 2.3, samples 0x60..0x63).
- `sample_cmd(n, mode)` 0x12698: no-op if sound is off or slot `n` is empty. `mode < 4`: allocate a voice (0x11AC4), set its state to `mode`, kind 0x10, sample `n`, start pointer = header h0, loop/end pointers = h1..h3, h4 copied. `mode >= 4`: for every voice of kind 0x10 playing sample `n`, set state to `mode - 4`; state 0 stops it and frees the voice.
  - The game only ever uses **mode 3 = one-shot play** and **mode 4 = stop all instances of n**. States 1 and 2 are set only by the MIDI-sample path; their meaning (sustain/loop variants) is [UNVERIFIED] and irrelevant for the port.
- `sample_unload(n)` 0x12730: stop (mode 4), free the DOS block, clear the slot.
- Header: `h3 − h0` is the byte count the driver transfers for an 8-bit card; with the 16-bit flag set it transfers twice that (the file holds `2·(h3−h0)` PCM bytes plus one; `formats.md` §5.1). The port plays the whole PCM at the directory's rate; what the first half alone sounds like on an 8-bit card is [UNVERIFIED] and not needed.
- Variant digits 1..6 (`SMPL_611`, `SMPL_7B3`, ...) are **not** reachable through the sample API. They are instrument samples for the MIDI player: every `SONGx.MID` carries a private chunk (signature `/(C) 1995 Edelguth Laboratorien` right after the header) with a 128-byte bitmap; bit k (0..5) of entry i requests `SMPL_<T[i]><k>` where `T` is the note table at `0x80477` (notes 97..104 -> 0x61, 0x61, 0x63..0x68; notes 112..124 -> 0x70..0x7C, 0x73 twice), and bit 6 requests `SMPL_<80+i>0`. The song loader 0x11C6B loads them into a separate note-to-slot table (`0x80F2D`) when the song starts. A port that plays the OGG soundtrack can ignore this entirely.
- Lifetime: samples are loaded by the screen that needs them and unloaded when the screen closes (2.1). Only the two click samples 0x42 and 0x4C stay loaded for the whole session (loaded in the game init 0x2F4B0, freed in the shutdown 0x2F970 and around a sound-card change).

### 1.3 Speech (SPC WAV files) [C]
- `speech_load(c0, c1, c2, c3, n)` 0x15100 (registers EAX, EDX, EBX, ECX and one stack argument). Returns a slot index and pushes it on a small stack (`0x84578` depth counter, pointers at `0xC22B0`). If speech is off, the slot is empty (and still pushed). File name: `<prefix>spcXxkhz\` + four characters + three decimal digits + `.wav`, with each of the four characters taken from the alphabet `0123456789abcdefghijklmnopqrstuvwxyz` at index c0, c1, c2, c3, and the digits `n/100`, `(n%100)/10`, `n%10`. So (0, 2, 2, 9, 905) -> `SPC22KHZ\0229905.WAV` and (17, 1, 2, 3, 14) -> `H123014.WAV`. 1416 files exist per rate directory (`formats.md` §5.2); the WAV payload becomes a pseudo sample (h2 = h3 = length) in the same slot table as the SMPL samples, at slot index = stack depth (0, 1, 2, ...), so speech never collides with the sample numbers (>= 0x2D).
- `speech_play(i)` 0x154B0: `sample_cmd(i, 3)` on stack entry i if it is loaded.
- `speech_pop()` 0x153F0: stop (mode 4) and free the top entry; `speech_pop_all()` 0x15480.
- `sound_idle()` 0x12C71: true if sound is off, or no voice is still playing. Used as "wait until the speech has finished" in busy loops.
- `speech_say(c0..c3, n)` 0x15570: load, play, then wait until the speech is finished **or a mouse button is pressed**, pop; returns true if no button was pressed (the auction tick lines, the trade-certificate award and two trial lines use it).
- `say_box(c0, c1, c2, c3, n, x, y, w, h, fmt, args...)` 0x36E50: opens a window (style 0x8800), loads and plays the speech, prints `fmt`, then: if speech is playing, waits until it has finished (no click needed); otherwise `wait_click` 0x36720. Then pops the speech and closes the window. All history reminders, the trial narration and the duel dialogue go through it (section 3).
- Meaning of the four name characters, as far as the call sites show it (section 3): c0 is the voice (0 = narrator; 1/2 = two voices chosen by `1 + rand(2)` or by context; 3..10 = the eight witness/juror voices of the trial, `3 + rand(8)`; 11 = 'b' (the judge), 12..14 = 'c'..'e' (trial parties); 15..17 = 'f'..'h' (the tip speakers, `15 + rand(4)` with 18 mapped to 1). For c0 = 0 the pattern is `0 2 2 <group>` where the group selects a text family (0 disasters, 1 office-tree, 2 trial, 3 turn end, 4 certificate, 5 courtship, 6 births, 7 city info / sound check, 8 auction, 9 history and turn-end reports) and `n` = 900 + line. Where the call sites vary c1 or c2 they pass the **gender of the person concerned** (person +0x00, 0 male / 1 female) in that position.

### 1.4 Music (MIDI songs) [C]
`music(song, mode)` 0x41E50 (AL, DL):
1. Song codes 0x19..0x1E are random groups: 0x19 = `{0x12, 0x13, 0x14, 0x15, 0x01, 0x04, 0x0A, 0x12, 0x15}[rand(9)]` (table 0x86C6C), 0x1A = `{2, 3}[rand(2)]`, 0x1B = `{0x10, 6, 3}[rand(3)]`, 0x1C = `{2, 3, 0x10, 6}[rand(4)]`, 0x1D = `{1, 4, 0x12, 0x18}[rand(4)]`, 0x1E = `{7, 0x0E}[rand(2)]`. 0x1F = the current song. These `rand` draws come from the game RNG 0x36FF0 and happen even when no music device exists (parity hazard like `rules-07` §5.13).
2. If the music driver is inactive, current := 0.
3. mode 3 and current != 0: stop (load song 0 = silence, 0x12BED with 0). Mode 3 never starts anything.
4. Start if: mode 0 (always); mode 1 and song != current; mode 2 and current == 0; mode 4 and not the low-memory build; mode 5 and current == 0 and not the low-memory build. Start = build `midsongs\song<c>.mid` with `c` = `"0123456789abcdefghijklmno"[song]` (songs 1..24 -> `SONG1`..`SONG9`, `SONGA`..`SONGO`), stop the running song, load it (which also loads its instrument samples), play, current := song.
- `music_stop_if_lowmem()` 0x41FF0: `music(0x1F, 3)` only in the low-memory build; called when the battle screen and the "end turn" dialog 0x4D1E0 open.
- There is no crossfade and no fade; a new song cuts the old one. Whether a song loops at its end and whether `current` is cleared when it ends is [UNVERIFIED]: the end-of-song handler lives in the interrupt driver, which is not in the image. The byte `0x86C58` has no other writer in the disassembled code, so mode 2/5 calls ("only if nothing plays") fire only when no song was started since the last explicit stop.
- The sound check (options case 7) plays speech (0, 2, 2, 7, 900) and `music(0x0D, 0)` under the caption "Soundcheck läuft...", then `music(0x0D, 3)`. The sound setup 0x15630 uses `SONGA.MID` as its test song.

### 1.5 Timing helpers used around sounds [C]
- `delay_ticks(n)` 0x12DB5: waits n hundredths of a second on the DOS clock (`delay_ticks(5)` = 50 ms between auction/duel animation frames).
- `delay_ms(n)` 0x6B07C (C runtime): milliseconds; the duel uses `delay_ms(100 + rand(300))` between blows.

## 2. Sample effects: every number the game plays

### 2.1 Load sets per screen [C]
| screen / function | loads at open | frees at close |
|---|---|---|
| game init 0x2F4B0 | 0x4C, 0x42 | shutdown 0x2F970 |
| disaster report 0x18BD0 | 0x45 | yes |
| auction screen 0x27680 | 0x4B | yes |
| duel 0x2C750 | the 18 entries of table 0x84D30 (2.4) | yes |
| battle 0x3EE60 (before the first round) | the 35 entries of table 0x858E0: 0x5F 0x59 0x46 0x31 0x33 0x58 0x39 0x43 0x80 0x5A 0x47 0x32 0x34 0x3A 0x44 0x81 0x5B 0x48 0x35 0x3B 0x82 0x5C 0x36 0x3C 0x83 0x5D 0x37 0x3D 0x5E 0x38 0x3E 0x3F 0x40 0x41 0x4D 0x2D | after the battle, once `sound_idle()`; also by 0x38920 |
| credits (quit 0x4D880 / 0x4DA50) | 0xCC (just before playing it) | yes |
| indulgence 0x59720 | 0x4A | yes |
| church 0x5D1F0 | 0x49 | yes |
| turn end 0x61A70, bill page | 0x84 | yes |
| turn end 0x61A70, debtors' tower screen | 0x54..0x57, 0x85, 0x86 | yes |
| turn end 0x61A70, debt seizure screen | 0x4E..0x51 | yes |

0x2D, 0x80..0x83 are loaded for the battle but no battle site plays them (0x80..0x83 are duel sounds; the two tables look like one shared design).

### 2.2 Cue table
Lengths are the 22 kHz file: `(size − 20) / 22050` s; the 44 kHz files have the same durations. "Sounds like" is inferred from use and length only.

| n | file | len s | sounds like [UNVERIFIED] | trigger(s) |
|---|---|---|---|---|
| 0x31, 0x32 | SMPL_310/320 | 1.08, 0.82 | blow landing / cart hit | battle 2.3 (strike by carters/sailors; parry) |
| 0x33..0x38 | SMPL_330..380 | 3.3..5.9 | explosions | battle: cannon fired `0x33 + rand(6)`; explosion `0x33 + rand(6)`, then if `rand(100) < 90` another `0x33 + rand(6)` (2.3) |
| 0x39..0x41 | SMPL_390..410 | 0x39 3.6, rest 0.3..1.3 | screams / grunts on hit | battle: man hit `0x39 + rand(9)`; duel hit tables (2.4) |
| 0x3B..0x3E | (as above) | | | duel hit tables only (2.4) |
| 0x42 | SMPL_420 | 0.24 | short click | `wait_click` 0x36720 and `menu` 0x35B60: left mouse button |
| 0x43, 0x44 | SMPL_430/440 | 1.39, 0.96 | projectile / object sound | battle object step 0x3CFDB: `0x43 + rand(2)` (object kind [UNVERIFIED], see 2.3) |
| 0x45 | SMPL_450 | 6.29 | avalanche rumble | disaster report: right after `flcfilme\lawine.flc` is started (0x18F57) |
| 0x46..0x48 | SMPL_460..480 | 1.3..1.4 | musket shots | battle: shooter strike `0x46 + rand(3)` |
| 0x49 | SMPL_490 | 8.53 | church bells | church 0x5D1F0: after the yes/no box 0x368E0 returns 0 (donation/candle accepted) |
| 0x4A | SMPL_4A0 | 4.32 | organ / choir | indulgence 0x59720 when the buyer is human |
| 0x4B | SMPL_4B0 | 0.65 | hammer blow | auction 0x26A50: lot sold, after the hammer animation loop (0x275A3) |
| 0x4C | SMPL_4C0 | 0.26 | click / thud | `wait_click` and `menu`: any mouse button other than left; battle: banner planted or taken over (0x3AB8D, 0x3D15B) |
| 0x4D | SMPL_4D0 | 14.28 | battle ambience (crowd, wind) | battle loop: ambience counter c (starts 0 at battle start): if c < 7 then c += 1, else if `rand(100) < 20` then c := 0 and, if displayed, play 0x4D (`rules-07` §5.11) |
| 0x4E..0x51 | SMPL_4E0..510 | 0.6..0.8 | hammer / seal | turn end, debt seizure screens (icons 0xBA): `0x4E + rand(4)` after each caption box, three sites 0x65532, 0x655A3, 0x6560E |
| 0x53 | SMPL_530 | 0.71 | sword swing | duel swing table (2.4) |
| 0x54..0x57 | SMPL_540..570 | 0.58..0.77 | chains / drips | turn end, debtors' tower screen: while waiting for the click, every 30 ms `rand(100) < 2` -> `0x54 + rand(4)` (0x652A4) |
| 0x58 | SMPL_580 | 2.00 | body falling | battle: man falls (state 5, frame 5); bomb laid (state 6, frame 6) |
| 0x59..0x5E | SMPL_590..5E0 | 0.5..1.2 | footsteps | battle: walking man, frames 2 and 5, `rand(100) < 60` -> `0x59 + rand(6)` |
| 0x5F | SMPL_5F0 | 0.37 | sword swing | battle melee strike/parry `0x5F + rand(5)`: only 0x5F is in the battle load set, 0x60 and 0x62 have no variant-0 file at all, 0x61/0x63 (`SMPL_610/630`, 0.8 s) exist but are never loaded, so four of the five rolls are silent in the original; duel swing table |
| 0x80..0x83 | SMPL_800..830 | 0.36..0.48 | blade clashes | duel swing table (2.4) |
| 0x84 | SMPL_840 | 1.24 | coins | turn end bill page, right after "Gesamtkosten" is printed (0x62EB2) |
| 0x85, 0x86 | SMPL_850/860 | 26.2, 8.1 | long ambience (wind, rats) | debtors' tower screen: every 30 ms, after the 2 % roll above, `rand(100) < 1` -> `0x85 + rand(2)` (0x652D2) |
| 0xCC | SMPL_CC0 | 1.66 | closing chord / stamp | credits: after the 15th credit line (0x4D9ED, 0x4DB60) |

Everything else in `SMP22KHZ` (0x2D..0x30, 0x3A, 0x3F, 0x40, 0x52, 0x5A..0x5E are covered above; 0x61..0x7C with variant digits, 0xA3..0xEC) is MIDI instrument material or unused by the game code. 0x2E..0x30 and 0x3A are referenced only through the tables (0x3A, 0x3C..0x41 in 2.4; 0x2D loaded, never played).

### 2.3 Battle [C]
Sounds only when the battle is displayed (`0xDC83F == 0`, `rules-07` §5.13); the `rand` for the sample is drawn only then. Man record fields as in `rules-07` §5.9 (+0x0E type, +0x12 action state, +0x14 animation frame, +0x20 hit flag). All plays are mode 3.
- Cannon fired (after the ball object is created): `0x33 + rand(6)`.
- Man animation step (per frame):
  - state 3 (walk), frame 5 or 2, and `rand(100) < 60`: `0x59 + rand(6)`.
  - state 1 (strike/shoot), frame 3: type 1 or 6 (shooters): `0x46 + rand(3)`; type 10 or 11 (carters, sailors): `0x31 + rand(2)`; any other type: `0x5F + rand(5)`.
  - state 2 (parry), frame 3: type in {1, 6, 10, 11, 2, 7}: `0x31 + rand(2)`; others: `0x5F + rand(5)`.
  - state 5 (falling), frame 5: `0x58`.
  - hit flag (+0x20) set: flag cleared, frame −1, remaining frames +1, redraw, `0x39 + rand(9)`.
  - state 6 (laying a bomb), frame 6: bomb object created, `0x58`.
- Explosion (bomb fuse end or cannon ball arrival): `0x33 + rand(6)`; then `rand(100)` (always drawn) and if < 90: `0x33 + rand(6)` again.
- Object step 0x3CFDB after the object sprite is drawn: `0x43 + rand(2)`. Which object kinds reach this step is [UNVERIFIED] (cannon ball in flight is the likely one).
- Banner: planting a new banner object (man becomes type 12, 0x3D15B) and taking over an existing one (0x3AB8D): `0x4C`.
- Ambience 0x4D and music: section 2.2 and 4.

The `rand(count)` sequence of a displayed battle therefore is: the ambience `rand(100)` per activation (both modes), then per sound site one `rand(n)` (displayed only).

### 2.4 Duel 0x2C750 [C]
Table 0x84D30, 7 triples `(swing, hit0, hit1)`: (0x83, 0x39, 0x3A), (0x5F, 0x31, 0x3B), (0x80, 0x32, 0x3C), (0x81, 0x3D, 0x3E), (0x82, 0x3E, 0x3F), (0x83, 0x40, 0x41), (0x53, —, —). Each exchange (two sites, one per duellist, 0x2CF63/0x2CFD3 and 0x2D13E/0x2D1AE): `delay_ms(100 + rand(300))`, swing = `triple[rand(7)].swing`; then `delay_ms(100)`, `b = rand(2)`, `r = rand(6)`, hit = `triple[r].hit[b]`. The 18 bytes of the table (the 7 triples minus the two zero bytes) are all preloaded at duel start and unloaded at the end.

## 3. Speech lines (every `speech_load` site)
Notation `(c0 c1 c2 c3 : n)` -> file. `g` = gender of the person concerned (0/1). "blocking" = `speech_say`; "box" = `say_box`; otherwise `speech_load` + `speech_play` with the caller waiting on `sound_idle()`.

| where | pattern | lines | note |
|---|---|---|---|
| history / reminders 0x17930 (16 boxes) | (0 2 2 9 : n) `0229…` | n = 904 first box; 905 `[EXE text]`; 906 `[EXE text]`; 907 "Ihr hieltet Hochzeit"; 908 `[EXE text]`; 909 concert; 910 (two boxes); 911 + child byte (son/daughter); 913; 914 "Jahrhundertflut"; 915; 916; 900 (0x187C3); mission box: 900 + word from table 0x84AE8 | box |
| outro 0x17930 | (0 0 0 0 : 13) `0000013` | before `outro.flc` / `outrob.flc` | play, no wait |
| disaster report 0x18BD0 | (0 2 2 0 : 900 + kind) and (0 2 2 0 : 910 + x) `0220…` | kinds 0..3 of `rules-06` §2 | box |
| courtship 0x212B0 | (0 2 2 5 : 902, 903, var) `0225…` | gifts/answers | wait |
| births 0x221F0 | (0 2 2 6 : 900 + 2·g(parent) + (record +4 & 1)); then (0 0 0 0 : 14 + x) and (0 0 0 0 : 14 + (x + 1 + rand(4)) % 5) | the two `0000` lines are the name proposals 14..18 [UNVERIFIED meaning] | wait |
| auction 0x26A50 | blocking (0 2 2 8 : 900 + k) for the no-bid tick texts k (`rules-12` §3.5); (0 2 2 8 : 916 + bidder) when the leading bidder changes | `0228900..919` | |
| auction 0x27680 | blocking (0 2 2 8 : 919) | | |
| trial 0x289E0 | boxes (0 2 g 2 : 900, 901+k, 915, 916, 917, 922+, 924, 925+, 927) `0202/0212/0222…`; blocking (0 2 2 2 : 910 + g), (1 g 2 2 : 30 + k); sentences (g? 2 2 : 86, 87, 88, 90, 91, 92) `1022/1122/2022/2122…`; witness statements (3 + rand(8), 0, g, 2 : 39 + 8k + j) `3002..A012`; judge (11, s, s, 2 : 74 + rand(3)) `B002/B112…` with s = a side byte; defence/prosecution lines (c0 = 1 or 2 by controller, c1/c2 from the party records) `C/D/E…` | 30 sites, see `rules-04b` for the trial flow | |
| duel 0x2C750 | boxes (0 2 2 1 : 933+, 941+, 946, 947, 948) `0221…`; and two boxes with c0 = duellist's gender-dependent voice | | box |
| office tree 0x45C30 | (1 + rand(2), g, 2, 1 : 11 + rand(12)) `1021/1121/2021/2121` | random remark when a bar is clicked | wait |
| computer-person reaction 0x5B200 | (1 + rand(2), g, 2, 1 : 11 + rand(12)) | same files | wait |
| trade certificate award 0x494F0 | blocking (0 2 2 4 : 900 + 12·L), (0 2 2 4 : 901 + 12·L), (0 2 2 4 : 902 + 12·L + x) `0224900..917` | L = certificate level 0..1 [L] | |
| year-start screen after `city_growth` (function after 0x4A484, unnamed) | (0 2 2 9 : 948) + `music(4, 1)` | | wait |
| options 0x4E810 | (0 2 2 7 : 900) | sound check | |
| city info 0x4F3C0 | (0 2 2 7 : 900 + k) `0227900..909` | k = a local of the screen (menu index) [UNVERIFIED] | wait |
| turn end 0x61A70 | ruin screens: (1/2, 0/1, 2, 2 : 80) and (… : 81) — all four voice/gender combinations are loaded and the entry `idx` or `idx + 4` is played while the icon animates; (0 2 2 9 : 949..954) `0229…` for the Schuldturm, seizure and title screens; tips (15 + rand(4) -> 15/16/17/1, rand(2), 2, 3 : tip) `F023/F123/G…/H…/1023/1123` 0..18; (0 2 2 3 : 901..909) and (0 2 0 3 / 0 2 1 3 / 0 2 2 3 : 900) `0203/0213/0223…` for the report lines | 24 sites | wait |

Not resolved: which `0000000..012`, `022A900/901`, `022B902..909` lines are used (no constant site found; they may come from the variable trial sites) [UNVERIFIED].

## 4. Music

### 4.1 Song triggers (every `music` call) [C]
| song | screen / event | mode |
|---|---|---|
| 1 | feast 0x20530 (icon 0xBB) | 0 |
| 4 | player setup 0x4DBC0 ("Neuer Spieler"); year-start screen after `city_growth` | 1 |
| 5 | courtship 0x212B0: suitor rejected ("… Eure Werbung abzulehnen", icon 0xC0) | 0 |
| 6 | camp screen 0x56E20; castle info 0x549C0 when the place kind is >= 3 | 2 |
| 8 | siege resolution 0x1F2A0 (two sites); turn end: branch count above `MaxBr` (`rules-12` §3.1); year end: office dismissal report 0x66820 | 0 (turn end: 2) |
| 9 | turn end: the human's life counter reaches 0 (death, 0x657C1); births 0x22559 (second report path) | 0 |
| 0x0B | turn end: `[EXE text]` screens (icons 0xB9 / 0xBA, 0x65195, 0x65511); disaster kind 2 (plague) | 0 |
| 0x0C | disaster kind 1 (storm flood) | 1 |
| 0x0D | disaster kinds 0 (fire) and 3 (avalanche) [table 0x84B30 = 0D 0C 0B 0D]; sabotage film (`rules-12` §3.6); spy evidence report 0x639DA; insurance pays damage 0x68EDA; sound check | 1 / 0 |
| 0x0E | trade certificate award 0x494F0 (icon 0xCB) | 1 |
| 0x0F | money lender 0x5C450; church 0x5CCA0 | 2 |
| 0x10 | trial 0x289E0: at open (icon 0xC1) mode 0, re-asserted with mode 1 after each speech box (10 sites); backroom menu 0x5C140 (Spione/Sabotage/Erpressung/…/Kontor): `rand(100) < 50` -> mode 0, then always mode 2 | |
| 0x11 | matchmaker 0x20D30 (icon 0xBD) | 1 |
| 0x12 | auction screen 0x27680 (icon 0xC1) mode 0; `auction_main` 0x26A50 mode 1 | |
| 0x13 / 0x15 | Kontor 0x5D610 at entry: `rand(100) < 25` -> song = `rand(100) < 50 ? 0x15 : 0x13`, mode 2 | 2 |
| 0x15 | privileges menu 0x5F080, low-memory build only | 1 |
| 0x16 | birth 0x221F0 (first report path, 0x22285) | 0 |
| 0x17 | duel 0x2C750 at start | 0 |
| 0x18 | credits (quit screen 0x4D880 / 0x4DA50), then `music(0x1F, 1)` per line (no-op re-assert) | 0 |
| 0x19 (random: 0x12 0x13 0x14 0x15 1 4 0x0A 0x12 0x15) | trade map 0x4CC90 (icon 0x23 "Handel"); castle order screen 0x57E70 | 2 |
| 0x1A (random: 2, 3) | castle screen 0x55090; castle info 0x549C0 when the place kind is < 3 | 2 |
| 0x1B (random: 0x10, 6, 3) | battle screen setup 0x3F8EC | 4 |
| 0x1C (random: 2, 3, 0x10, 6) | battle loop, each round before the ambience roll (0x40B71) | 5 |
| 0x1D (random: 1, 4, 0x12, 0x18) | history screen 0x17930 when it has entries; outro (two sites); concert 0x1AEA0; wedding 0x212B0 (icon 0xFF) | 0 |
| 0x1E (random: 7, 0x0E) = fanfare | herald announcement (`rules-12` §3.5); relief army 0x1FA80 (two sites); title award 0x49700; turn end title/office screen 0x6539E (icon 0xB8); year end 0x6831E, 0x684DD, 0x6898D when the person is human (elections/appointments) | 0 |
| stop (0x1F, 3) | history screen end 0x1883B; office tree 0x45C30 and "end turn" dialog 0x4D1E0 and battle start, low-memory build only; sound check end | |

### 4.2 Soundtrack mapping [UNVERIFIED]
The OGG tracks in `<local path>` were matched to the MIDI files by duration (MIDI length from the tempo map, OGG length from the last granule position; the renders run 0..3 s longer than the MIDI) and checked against the trigger context. Rows marked ~ have a second candidate within 2 s.

| song | MIDI s | track | OGG s | note |
|---|---|---|---|---|
| 1 | 101.1 | 01 Unsere Zeit | 103.1 | feast / festive group |
| 2 | 65.9 | 13 Der Hauptmann | 68.0 | castle group 0x1A |
| 3 | 113.2 | 14 Hinterhalt ~ | 115.7 | battle groups 0x1B/0x1C; 08 Moveo ergo sum has the same length |
| 4 | 74.6 | 02 Händlertanz | 75.7 | |
| 5 | 44.9 | 16 Das Versprechen ~ | 47.3 | courtship rejected; 19 Ihr auch (44.5) also fits by length |
| 6 | 88.0 | 09 Im Verborgenen | 91.2 | robber camp |
| 7 | 10.2 | 22 Fanfare 1 | 12.0 | |
| 8 | 15.9 | 24 Fanfare 3 | 17.3 | siege / dismissal |
| 9 | 230.0 | 21 Trauermarsch | 231.7 | death |
| 0x0A | 48.8 | 03 De scheresliep ~ | 48.0 | Kontor roll |
| 0x0B | 50.4 | 20 Am Rande des Ruins | 53.6 | the screen text is `[EXE text]` |
| 0x0C | 47.3 | 18 Der Untergang ~ | 48.7 | storm flood; 16 Das Versprechen has the same length |
| 0x0D | 42.8 | 19 Ihr auch ~ | 44.5 | fire, sabotage, sound check; 17 An der Wiege (42.6) also fits |
| 0x0E | 7.6 | 23 Fanfare 2 | 8.7 | |
| 0x0F | 180.9 | 04 Der Kaufmann or 05 Parks und Ländereien | 182.1 / 182.7 | money lender / church; not separable by length |
| 0x10 | 134.6 | 10 Finstere Pläne | 134.2 | trial, backroom |
| 0x11 | 51.0 | 15 Die Kupplerin | 52.9 | matchmaker |
| 0x12 | 136.2 | 11 Höfische Gesellschaft | 137.4 | auction |
| 0x13 | 180.9 | 05 Parks und Ländereien or 04 Der Kaufmann | | Kontor |
| 0x14 | 167.7 | 12 Aufstieg und Glanz | 169.9 | Kontor roll |
| 0x15 | 224.7 | 07 Das Tagewerk | 225.7 | Kontor, privileges |
| 0x16 | 39.4 | 17 An der Wiege ~ | 42.6 | birth |
| 0x17 | 114.1 | 08 Moveo ergo sum ~ | 115.7 | duel; 14 Hinterhalt has the same length |
| 0x18 | 105.8 | 06 Aufspiel zum Fest | 106.9 | credits, festive group |

## 5. Open / unverified
- Mixer internals (voice states 1/2, end-of-song handling, whether songs loop, whether `0x86C58` is cleared at song end): the interrupt code is not in the disassembled image.
- Meaning of the second PCM half for 8-bit cards (`formats.md` §5.1).
- Speech: exact `n` for the variable trial sites (0x29FD2 group) and the duel's gender-dependent boxes; the owners of `0000000..012`, `022A`, `022B`.
- Soundtrack rows marked ~ and the 0x0F/0x13 pair need a listening check.
- Battle object step 0x3CFDB (`0x43 + rand(2)`): which object kinds reach it.

## Corrections (rules-27)
- Trade certificate award: three lines; the third is `902 + place + 2·(place >= 10)` without the `12·L` term. Music `0x1E` draws `rand(2)` from the game RNG on every call (title award); `0x0E` (certificate) draws nothing.
