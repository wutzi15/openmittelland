# Auctioneer animation, duel sound, animation frame timing (rules-25)

Source: `auction_main` 0x26A50..0x2767F, `duel_main` 0x2C750..0x2D747, `delay_ticks` 0x12DB5, `delay_ms` 0x6C083 and its calibration 0x6C05A, speech load 0x15100 with the gain table 0x11C1B/0x11C58, trial frame loops 0x29E64..0x2B6BD. Bank pixel checks: own decoder over `ICONS2B/ICOND5..D8.DAT`. Tags: **[C]** read at instruction level or measured, **[UNVERIFIED]** inferred. Notation as in `rules-22`.

## 1. Auctioneer figure (resolves `rules-15` §1.4 step 6)

### 1.1 State [C]
Per call of `auction_main` (one call per lot): `bank := 0` (none), `f := 1` (next frame to draw), `c := 0` (hammer count), `spoken := 1` (last hammer count spoken), `tick := 0`. The lot setup (0x27F8D) has already copied the frame-0 patch (196, 28, 217, 164) into the window surface, so every lot starts from the base pose.

Table `T` at 0x84C84, five dwords per bank (bank `b` = 0xD5 + high bidder's person index mod 4):

| bank | T0 | T1 | T2 | T3 | T4 | frames in file |
|---|---|---|---|---|---|---|
| 0xD5 | 27 | 39 | 63 | 14 | 56 | 63 |
| 0xD6 | 31 | 43 | 67 | 19 | 60 | 67 |
| 0xD7 | 35 | 47 | 71 | 22 | 64 | 71 |
| 0xD8 | 35 | 47 | 71 | 24 | 64 | 71 |

One "frame step" = `bank_frame(0, 0, bank, k)` into the window surface, `region_copy(screen, 196, 28, surface, 196, 28, 217, 164)`, `delay_ticks(5)`. Ranges below are half-open: `a..b` draws `a, a+1, …, b−1`.

### 1.2 Bid tick [C]
Order after the bid is accepted (price raised, "bietet" line printed, text area copied to the screen):
1. **Return to rest**: if `f > 1` and `0xD5 + (bidder mod 4) ≠ bank`: frames `T4..T2` of the **old** `bank`; then `f := 1`. (Same bank or `f ≤ 1`: nothing.)
2. **Gesture roll**: only if `f = 1` now: `rand(100)`; if `< 20`: `bank := 0xD5 + (bidder mod 4)`, load it, frames `1..T3`, and `f := T3` (the loop leaves `f` at T3; the figure is **not** reset to 1). If `f ≠ 1`, no `rand(100)` is drawn.
3. `c := 0`.

So the roll happens on the first bid of a lot, and after any bid that triggered a return to rest. Draw order inside the tick: `rand(6)` computer bidder pick (only when no human bid by key), then `rand(100)` gesture roll.

### 1.3 Hammer tick [C]
A no-bid tick draws `rand(100)`; a hammer step happens if `r < 70` and `tick > 1` (the first two ticks of a lot never hammer), or on Esc. With no high bidder: unsold text, `wait_click`, lot ends. Otherwise:
1. Text: `T[0xB6EB4][2c + rand(2)]` with the price; text area copied to the screen.
2. If `spoken ≠ c`: speech `(0 2 2 8 : 916 + c)` loaded and started (not waited for here).
3. `bank := 0xD5 + (high bidder mod 4)`, load it; frames `f..T[c]`, `f := T[c]` (nothing is drawn if `f ≥ T[c]` already).
4. `sample_cmd(0x4B, 3)` (hammer blow) after **every** hammer count, 0, 1 and 2.
5. If `c = 2`: `flip`, `delay_ms(200)`.
6. Wait until a mouse button is down (INT 33h AX=3, 0x12DE4) or `sound_idle()`.
7. If `spoken ≠ c`: `speech_pop_all`, `spoken := c`. Then `c += 1`; `c = 3` ends the lot.

Effects:
- The frame strip is one continuous sequence: `1..T3` gesture, `T3..T0` "zum Ersten", `T0..T1` "zum Zweiten", `T1..T2` "zum Dritten", with `T4..T2` doubling as the return segment (the tail of the third hammer).
- Without a gesture, "zum Ersten" plays `1..T0`.
- After a bid that keeps the same bank, `f` stays where it was. The next "zum Ersten" then draws nothing if `f ≥ T0`. Later counts continue from `f`.
- A hammer count equal to `spoken` is not spoken again (e.g. a bid right after "zum Ersten", then "zum Ersten" again: silent). The first "zum Ersten" of a lot is always spoken because `spoken` starts at 1 and c = 0.

### 1.4 Delta frames [C, measured on ICONS2B]
- `bank_frame` only blits frame k's opaque pixels onto the surface; nothing restores frame 0 between frames. The only reset is the per-lot patch copy.
- Frames 1..n−1 are deltas against the **predecessor**. Compositing frames 1..k in order differs from "frame 0 + frame k" in up to 4336 (D5), 2725 (D6), 5638 (D7), 6081 (D8) pixels. Empty frames (e.g. D5 frames 3, 4, 23, 24) are holds. All overlays lie inside x 204..379, y 33..183.
- The pose after the full strip (`1..T2`) equals the cumulative state after frames 1..12 (D5), 1..16 (D6), 1..18 (D7, D8). It differs from frame 0 by 2410 / 1427 / 1887 / 1887 px.
- Playing `T4..T2` from the end of hammer 1 or 2 lands exactly on that pose for D5/D6. D7/D8 from the end of hammer 2: 139 px off. From the end of a gesture: 1 / 11 / 13 / 297 px off.
- Restarting at frame 1 from that pose leaves 1508 / 1426 / 667 / 667 px different from the intended frame-1 state. These are original glitches. A port that composites cumulatively onto a persistent surface reproduces them.

Corrections to `rules-15` §1.4: the gesture leaves `f = T3`, not at rest. The pause at count 2 is **200 ms**, not 2 s. The 0x4B hammer sample follows every hammer count (also `rules-14` §2: not "when sold"). Speech starts **before** the frames.

## 2. Duel sound (resolves `rules-22` §5 question)

- **Fight bank loop** (0x2CCDA, frames 0..104) and **result bank loop** (0x2D36F, frames 0..18): no `sample_cmd`, no speech. Both are silent [C].
- **No frame-tied sounds exist.** There are no miss or fall samples [C].
- Every effect sample plays in the exchange halves, after the taunt `say_box` has closed, over the static fog picture 0xD1 [C].

Per half [C]:
- `s = 4 + rand(5)` swings. Each one: `delay_ms(100 + rand(300))`, then `sample_cmd(byte[0x84D30 + 3·rand(7)], 3)` = one of **0x83, 0x5F, 0x80, 0x81, 0x82, 0x83, 0x53**. 0x53 is never loaded, so it is silent.
- `h = 1 + rand(2)` hits. Each one: `delay_ms(100)`, `c = rand(2)`, `r = rand(6)`, `sample_cmd(byte[0x84D31 + 3r + c], 3)`. Pairs by r: **(0x39, 0x3A), (0x31, 0x3B), (0x32, 0x3C), (0x3D, 0x3E), (0x3E, 0x3F), (0x40, 0x41)**.
- Mode 3 = one-shot. The sample API has no volume argument. SMPL effects are not passed through any gain table, so all play at file level [C].
- Music: `music(0x17, 0)` at setup [C].

**Speech gain** [C]:
- `0x8457C` is the speech volume in percent (options menu ±8, range 0..255; image default 160; saved as CONFIG byte 0x278).
- At every speech load, `0x11C1B(V)` builds a 256-entry table: `out(s) = clamp(128 + trunc((s − 128)·V / 100), 0, 255)`. It is applied to the 8-bit unsigned PCM (speech WAVs are 8-bit mono, checked on `SPC44KHZ/0228916.WAV`).
- The duel sets `V := min(255, floor(V·11/10))` before the first caption. Default 160 → 176. All duel speech (captions, answers, taunts, fog lines, result lines) is loaded with that gain.
- At the end, `V := CONFIG[0x278]` (from 0x86BCC).
- Port equivalent: speech gain ×1.1 during the duel, with the absolute gain V/100 clipped at the 8-bit range.

## 3. Frame timing [C for code, UNVERIFIED for emulator clock]

`delay_ticks(n)` polls INT 21h AH=2Ch until `(hundredths_now − hundredths_at_last_return) mod 100 ≥ n`, then stores the new value.
- It paces from the **previous `delay_ticks` return**, so drawing time is absorbed. The first wait of a sequence measures against a stale stamp and may return at once.
- DOS hundredths come from the 18.2 Hz BIOS tick: they advance in steps of 5 or 6, and any 3 consecutive steps sum to ≥ 16. A wait therefore lasts a whole number of BIOS ticks (54.9 ms each).

| animation | call | BIOS ticks | ms per frame | frames | total |
|---|---|---|---|---|---|
| auctioneer (§1) | `delay_ticks(5)` | 1 | 54.9 | per segment | e.g. D5 `1..T0` = 26 frames ≈ 1.43 s |
| duel fight 0xD9..0xDC | `delay_ticks(5)` | 1 | 54.9 | 105 | ≈ 5.77 s |
| duel result 0xDD..0xE4 | `delay_ticks(5)` | 1 | 54.9 | 19 | ≈ 1.04 s |
| witness head 0x18E+rand(8) (0x29E64) | `delay_ticks(12)` | 3 | 164.8 | 23 | ≈ 3.79 s |
| court plea / figure segments (0x2A0DB … 0x2ABD4, six sites) | `delay_ticks(15)` | 3 | 164.8 | per segment | 16 frames ≈ 2.64 s |
| verdict frames 100..133 (0x2B6B8) | `delay_ticks(10)` | 2 | 109.9 | 34 | ≈ 3.74 s |
| wipes 0x2C140 / 0x2C460 | `delay_ticks(1)` on odd steps | 1 | 54.9 per 2 steps | up to 200 steps | up to ≈ 5.5 s (whether the loop exits early is [UNVERIFIED]) |

- Nominal values (n × 10 ms: 50 / 120 / 150 / 100 ms) are what `rules-14`, `rules-18` and `rules-22` state. On a standard DOS clock the real values are the ones above.
- Whether the DOSBox-X build feeds INT 21h/2Ch from the BIOS tick (as above) or from a finer host clock is [UNVERIFIED]. On a finer clock the nominal values apply.
- Either way, 100 ms is wrong for all of these except the verdict loop (nominal 100 ms, 109.9 ms on a BIOS-tick clock).

`delay_ms(n)` (0x6C083, Watcom runtime): at start-up 0x6C05A counts INT 21h/2Ch calls per one-second change. `delay_ms(n)` then makes `max(1, (n·count + 500) / 1000)` such calls, so it approximates wall-clock milliseconds [C]. The duel's `100 + rand(300)` and `100` ms gaps and the auction's 200 ms are real milliseconds.

Corrections:
- `rules-14` §1.5, `rules-15` §0 `tick` and `rules-22` §4/§7: replace "n hundredths" with the pacing rule above.
- `rules-22` §5 "~1 s" per wipe is too short if all 200 steps run.
