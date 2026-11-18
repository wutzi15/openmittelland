# Music: song files, playback paths, triggers (rules-20)

Source: `MIDSONGS/SONG1..9,A..O.MID` (parsed with a script in the session scratchpad, `tools/midi.py`), the music front end 0x41E50, song start 0x12BED, the hardware MIDI sequencer 0x101F4..0x10432 (prepare 0x10432, tempo/meta 0x1034B, interrupt handler 0x101F4..0x102F3), the sample-synth song loader 0x11C6B, and the sample directories. Confidence [C] unless marked. Supersedes `rules-14` §1.4 where more precise; the trigger table of `rules-14` §4.1 is repeated in §5 with scene names so one document answers "which song plays here".

## 1. Files [C]

24 Standard MIDI Files, `midsongs\song<c>.mid`, `c = "0123456789abcdefghijklmno"[song]` for song numbers 1..24 (0x01..0x18). All have the same shape:

- format 1, **one track** (all channels interleaved in track 0), division **48 ticks per quarter**.
- Channels 0..9 (SONGK also 10..15, SONGE only 1, 2, 9). Channel 9 is percussion (GM drum map; track name "Rhythm&SE"). Channel 0 carries no notes in most songs (program 0, "Keyboard" track name).
- Events used: note on/off, program change, controllers 1 (modulation), 7 (volume), 10 (pan), 64 (sustain), 123 (SONGK), pitch bend. No sysex.
- Meta: copyright (type 2) with the private instrument chunk (§3), time signature (type 0x58), tempo (0x51, one at tick 0 and repeated every bar with the same value; SONGG and SONGL change tempo), track names (type 3, ten per file: instrument labels "Basso", "Floete", "Harfe", "Pauke", … or "Spur n"), end of track.
- No loop markers, no cue points, no sequencer-specific events.

| song | file | duration s | bpm | time sig. | notes | melodic programs (ch 1..8) |
|---|---|---|---|---|---|---|
| 1 | SONG1 | 101.1 | 144 | 4/4 | 1779 | 86; 72 74 75 86 87; 58 60 82; 78; 43; 3; 110; 122 |
| 2 | SONG2 | 65.9 | 120 | 4/4 | 572 | 29 68 92; 27 38 43 50; 24 95; 78; 43; 3; 110; 122 |
| 3 | SONG3 | 113.2 | 120 | 4/4 | 1274 | 112; 38 50; 95 97 122; 16 50 78; 43; 3; 110; 122 |
| 4 | SONG4 | 74.6 | 144 | 3/4 | 1334 | 68 86; 73 86; 16; 78; 43; 3; 110; 122 |
| 5 | SONG5 | 44.9 | 85 | 4/4 | 266 | 12; 112; 70; 78; 43; 3; 110; 122 |
| 6 | SONG6 | 88.0 | 120 | 4/4 | 626 | 36; 38 41 50 71 72; 38 74 95; 78; 43; 3; 110; 122 |
| 7 | SONG7 | 10.2 | 130 | 6/8 | 153 | 95; 86; 85; 112; 43; 3; 110; 122 |
| 8 | SONG8 | 15.9 | 100 | 6/8 | 201 | 95; 50; 85; 112; 43; 3; 110; 122 |
| 9 | SONG9 | 230.0 | 48 | 4/4 | 1201 | 38 92; 56 86; 13 34; 112; 43; 3; 110; 122 |
| 0x0A | SONGA | 48.8 | 128 | 4/4 | 836 | 17 89; 50 72 82; 86; 16 58 78; 43; 3; 110; 122 |
| 0x0B | SONGB | 50.4 | 100 | 4/4 | 222 | 36; 16; 72; 92; 43; 3; 110; 122 |
| 0x0C | SONGC | 47.3 | 132 | 4/4 | 345 | 68; 50 51; 70 95; 78; 43; 3; 110; 122 |
| 0x0D | SONGD | 42.8 | 80 | 3/4 | 320 | 38; 49; 95; 78; 43; 3; 110; 122 |
| 0x0E | SONGE | 7.6 | 120 | 4/4 | 80 | 95; 86 (channels 1, 2, 9 only) |
| 0x0F | SONGF | 180.9 | 127 | 4/4 | 2153 | 12; 11; 45; 78; 70; 3; 110; 122 |
| 0x10 | SONGG | 134.6 | 75/78/84/93 | 4/4 | 440 | 38; 76; 95; 78; 12; 15; 110; 122 |
| 0x11 | SONGH | 51.0 | 140 | 4/4 | 829 | 48; 48; 22; 49; 45; 3; 110; 122 |
| 0x12 | SONGI | 136.2 | 192 | 4/4 | 3138 | 48 86; 48; 84; 12; 73; 60; 56; 122 |
| 0x13 | SONGJ | 180.9 | 127 | 4/4 | 1752 | 73; 38; 16 17; 72; 86; 3; 110; 122 |
| 0x14 | SONGK | 167.7 | 120 | 4/4 | 916 | 38; 52; 48; 93; 16 54 86; 45; 110; 122 |
| 0x15 | SONGL | 224.7 | 69/84 | 4/4 | 1810 | 15 38; 60; 38 49 54; 48; 60; 57; 45 54 56; 71 |
| 0x16 | SONGM | 39.4 | 105 | 4/4 | 211 | 82; 9; 38; 78; 43; 3; 110; 122 |
| 0x17 | SONGN | 114.1 | 105 | 4/4 | 1102 | 68; 15 54; 48; 38; 86; 3; 110; 122 |
| 0x18 | SONGO | 105.8 | 95 | 4/4 | 2379 | 73; 71; 71; 12; 16; 48; 43; 68 |

Programs are GM numbers 0-based (as in the file). Programs 43 / 3 / 110 / 122 on channels 5..8 recur in most files and look like a template left by the sequencer (few or no notes on those channels [UNVERIFIED per song]). Duration = tempo map applied to the last event tick.

## 2. Playback paths [C]

`music(song, mode)` 0x41E50 resolves random groups (§4), applies the mode rule (`rules-14` §1.4), builds the file name and calls the song start 0x12BED, which branches on the configured devices (`0x80004` MIDI device, `0x80184` digital card):

1. **MIDI device present** (`0x80004 ≠ 0`; MPU-401 General MIDI, or Roland LAPC-I/MT-32, `rules-14` §1.1): the file is loaded (0x1074F), prepared (0x10432: up to 8 tracks, "lapc_prepare: too many tracks/ticks per quarter/tempomap overflow" errors) and played by the timer interrupt 0x101F4: each tick sends the due MIDI bytes to the port; running status is expanded; meta events are skipped except tempo (0x51) and time signature (0x58). Everything in the file goes to the device unchanged, so the device renders the GM programs: **a General MIDI synth reproduces this path exactly**. On the MT-32 the same program numbers map to the MT-32 patch set [UNVERIFIED which sound set the composer intended; the OGG soundtrack is the reference].
2. **No MIDI device, digital card present** (`0x80004 = 0`, `0x80184 ≠ 0`): the song is loaded by 0x11C6B, the *sample synth* loader: it reads the private chunk (§3) and loads one PCM sample file per requested (program, variant) and per requested drum note into the mixer's 32 voices; the interrupt mixer then plays the MIDI through those samples. The mixer code is not in the disassembled image (`rules-14` §5), so how a sample is pitched per note (the variant digit is most likely an octave/zone index) is [UNVERIFIED]. Programs without a sample entry in the note table (§3) are silent on this path. The OPL/AdLib port (0x388/0x389) is touched only by a small detection routine (0x10DE1..0x10E46) and nothing writes FM registers during playback, so there is no FM fallback [L].
3. **No sound** (`0x80004 = 0` and `0x80184 = 0`): nothing; the `rand` draws of the random groups still happen.

Tempo: `tick period = 240 000 000 / (µs per quarter · 2^denominator)` is stored as a 16-bit "beats" value (0x82EA6, default 120 for 500 000 µs and a /4 signature) and the timer is programmed from it [C for the formula, L for the timer reprogramming]; with division 48 this is 96 interrupts per second at 120 bpm. Tempo changes inside a song (SONGG, SONGL) are honoured.

End of song: when a track reaches its End-of-Track meta the handler sends MIDI Stop (0xFC) for that track and the prepare step appends an extra tempo-map track that ends in 0xFC. No code restarts a song or clears the current-song byte `0x86C58`; a song plays **once** and silence follows until the next `music` call [L: the handler's "all tracks done" state was not traced; `rules-14` §5]. For the port: play each track once, no loop. Fanfares (7, 8, 0x0E) are 8–16 s one-shots anyway; the long screen songs (Kontor, trial, battle) simply stop after 2–4 minutes in the original.

There is no volume, fade or crossfade; a new song cuts the old one at the next `music` start (`0x12BED` stops the running song, waits for the mixer voices, loads the new file).

## 3. The private chunk and the instrument samples [C]

Track 0 begins with `00 FF 02 81 2F` + 175 bytes of copyright meta: 31 bytes `(C) 1995 Edelguth Laboratorien` + NUL padding, then a **128-byte bitmap** (meta payload bytes 31..158), then four little-endian u32 **memory sizes** (payload 159..174): PCM bytes needed at 22 kHz 8-bit, 22 kHz 16-bit, 44 kHz 8-bit, 44 kHz 16-bit (SONGJ, SONGL, SONGM carry zeros). The loader compares the first 36 bytes of the track with the template at 0x801CC and requires the match.

Bitmap entry `i` (0..127), bit `k`:
- `k = 0..5`: load `smpXXkhz\smpl_<T[i]><k>.dat` where `T` is the 128-byte table at 0x80477 indexed by **GM program number** `i`; entries are 0xFF except programs 97..104 → 0x61, 0x61, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68 and 112..124 → 0x70, 0x71, 0x72, 0x73, 0x73, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C. A set bit for any other program is skipped (no file). The sample is registered in the note/program-to-slot table 0x80F2D (row `i`, column `k`, 12 bytes per row).
- `k = 6`: load `smpl_<i + 0x80 as two hex digits>0.dat` and register it in row `128 + i` — the **drum** sample for GM drum note `i` (SMPL_A30..EC0 = drum notes 35..108).

The bitmaps of the 24 songs request (program, variant) pairs for programs such as 12, 16, 38, 48, 50, 86, 95 … that have **no table entry**, and the drum files B40, B70, E90 that do not exist; both are skipped silently (a missing file leaves an empty 20-byte descriptor). Across all songs the loader can actually find only eight instrument files (`SMPL_613`, `614`, `701`, `702`, `703`, `7A3`, `7A4`, `7A5`: programs 97 "FX rain", 112 "tinkle bell", 121 "breath noise") and 30 drum files (A40, A60, A70, A90, AB0, AD0, AE0, AF0, B00, B10, B20, B30, B60, B80, BC0, BD0, BE0, BF0, C00, C10, C90, CB0, CC0, CD0, CF0, D30, D60, E30, E70, E80); 74 of the 82 instrument-variant files (0x61..0x7C) and 35 of the 65 drum files in `SMP22KHZ` are referenced by no song [C: generated from the bitmaps and the directory listing]. The sample-synth path therefore renders little more than percussion; it is not a faithful rendering of the songs and the port should not emulate it.

Sample file format: `formats.md` §5.1 (20-byte header, unsigned 8-bit mono PCM). Variant files are instrument zones for this path only.

## 4. Song selection [C]

`music(code, mode)`: codes 0x01..0x18 are songs; 0x19..0x1E random groups (one game-RNG draw each, drawn even without sound):

| code | pick | meaning |
|---|---|---|
| 0x19 | `{0x12, 0x13, 0x14, 0x15, 0x01, 0x04, 0x0A, 0x12, 0x15}[rand(9)]` | trade map, castle order screen |
| 0x1A | `{2, 3}[rand(2)]` | castle screens |
| 0x1B | `{0x10, 6, 3}[rand(3)]` | battle setup |
| 0x1C | `{2, 3, 0x10, 6}[rand(4)]` | battle rounds |
| 0x1D | `{1, 4, 0x12, 0x18}[rand(4)]` | festive: history, outro, concert, wedding |
| 0x1E | `{7, 0x0E}[rand(2)]` | fanfare |
| 0x1F | the current song | re-assert / stop |

Modes: 0 always start; 1 start unless this song is already current; 2 start only if nothing is current; 3 stop (with 0x1F); 4 start, 5 start-if-silent, both only in the large-memory build (`0x86BC8 > 0xC3500`). "Current" = `0x86C58`, set at start, cleared only by an explicit stop (mode 3) or when the driver is inactive, never by the song ending, so mode 2/5 calls after a song has run out do nothing. Implementers should copy this: keep `current` until a stop, even when playback has finished.

## 5. Song per scene trigger [C] (from the call sites; scene references to `rules-15`/`17`/`18`/`19`)

| trigger | call | song(s) |
|---|---|---|
| player setup "Neuer Spieler"; year-start screen after city growth (with speech 0229948 game-over check nearby) | `music(4, 1)` | 4 Händlertanz |
| feast scene (type 23) on picture 0xBB | `music(1, 0)` | 1 |
| concert scene (type 36), after the first caption | `music(0x1D, 0)` | 1 / 4 / 0x12 / 0x18 |
| wedding scene, banquet picture 0xFF | `music(0x1D, 0)` | same group |
| courtship rejected (door picture 0xC0) | `music(5, 0)` | 5 |
| matchmaker (0xBD) | `music(0x11, 1)` | 0x11 |
| birth scene (cradle 0xCE) | `music(0x16, 0)` | 0x16 |
| child death (graveyard 0xC7) | `music(9, 0)` | 9 |
| death: before the candle film | `music(9, 0)` | 9 Trauermarsch |
| bankrupt (0xBA), debtors' tower (0xB9) | `music(0x0B, 0)` | 0x0B |
| jury court acquittal (0xB8), title award, herald, relief army, year-end elections/appointments for humans | `music(0x1E, 0)` | 7 / 0x0E fanfare |
| too many branches (turn-end report) | `music(8, 2)` | 8 |
| siege films, office dismissal | `music(8, 0)` | 8 |
| disaster scene kind 0 / 1 / 2 / 3 | `music(0x0D / 0x0C / 0x0B / 0x0D, 1)` | by kind |
| sabotage film, spy-evidence films, insurance payout, sound check | `music(0x0D, 1)` (sound check mode 0) | 0x0D |
| trade certificate (0xCB) | `music(0x0E, 1)` | 0x0E |
| money lender, church | `music(0x0F, 2)` | 0x0F |
| trial scene at open; re-asserted after speech boxes | `music(0x10, 0)` then `(0x10, 1)` | 0x10 |
| backroom menu | 50 %: `music(0x10, 0)`, then `(0x10, 2)` | 0x10 |
| auction day at open; per lot | `music(0x12, 0)`, `(0x12, 1)` | 0x12 |
| Kontor entry | 25 %: `music(rand(100) < 50 ? 0x15 : 0x13, 2)` | 0x13 / 0x15 |
| privileges menu (low-memory build only) | `music(0x15, 1)` | 0x15 |
| duel scene start | `music(0x17, 0)` | 0x17 |
| credits | `music(0x18, 0)` | 0x18 |
| trade map, castle order screen | `music(0x19, 2)` | group 0x19 |
| castle screen / castle info (kind < 3) | `music(0x1A, 2)` | 2 / 3 |
| camp screen, castle info (kind ≥ 3) | `music(6, 2)` | 6 |
| battle setup / each round | `music(0x1B, 4)` / `(0x1C, 5)` | groups |
| history screen with entries, outro, mission accomplished | `music(0x1D, 0)` | group 0x1D |
| history end, office tree / end-turn dialog / battle start (low-memory build), sound check end | `music(0x1F, 3)` | stop |

Scenes with no music call of their own (they inherit whatever plays): gravestone/notary/funeral (song 9 from the candle), auction catalogue (0x12), place pictures of elections (fanfare only for humans), jury court bribe screen and vote (silent unless something was playing), tornado and divine boxes, fate events, witness/plea parts of the trial (0x10 re-asserted).

## 6. Rendering options for the port

1. **OGG soundtrack** (`rules-14` §4.2 mapping by duration; rows marked ~ still need a listening check): play the track once per `music` start, cut on the next start, no loop. This matches the original's hardware-MIDI path in structure (one-shot, hard cut) and is the recommended default.
2. **Software GM synth** (fluidsynth/TinySoundFont with a GM soundfont): feed the SMF unchanged, honour tempo changes, no loop; identical event stream to path 1 of §2. Use for a "MIDI" option or where the OGG licence is a problem.
3. Emulating the sample-synth path is not worth it (§3).

Caveat for parity: every `music(0x19..0x1E, …)` call draws from the game RNG, so the sim must perform those draws at the listed triggers even when music is off.

## 7. Open
- Mixer internals for the sample-synth path (pitching of variant samples, voice stealing) [UNVERIFIED, not in the image].
- The "song finished" state of the hardware sequencer (whether 0x82E70 bit 0 is cleared) [L].
- Intended sound set for path 1 (GM vs MT-32) [UNVERIFIED]; `rules-14` §4.2 ~ rows.
- Channels 5..8 template programs: whether they carry audible notes in each song [UNVERIFIED].
