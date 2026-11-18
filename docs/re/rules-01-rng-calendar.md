# RNG, calendar and turn order

Confidence tags as in `rules-00-map.md`: [C] confirmed, [L] likely, [G] guess.

## 1. Random number generator [C]

The game uses the Watcom C runtime `rand`/`srand` and one wrapper.

State: one unsigned 32-bit word `S` (at `0xB9284`), initial value 1.

- `srand(x)`: `S = x`.
- `rand()`: `S = S * 1103515245 + 12345 (mod 2^32)`; return `(S >> 16) & 0x7FFF` (0..32767).
- `random_n(n)`: `rand() mod (n mod 65536)`, i.e. the argument is truncated to 16 bits and the remainder is taken with a signed 32-bit division (rand is never negative, so this is the ordinary remainder). Result range `0 .. n-1`. `n = 0` (or a multiple of 65536) is a division by zero and crashes the original; no call site was seen to pass 0 deliberately, but a few pass computed values (for example `random_n(151 - director_pay_level)`), so an implementation should define `random_n(0) = 0` and log it.

All game randomness goes through `random_n` (100 call sites). `rand` is called only from `random_n`. Nothing else touches `S`.

Seeding [C]: `randomize_from_time` (0x37030) calls DOS INT 21h AH=2Ch (get time) and calls `srand(DX)`, where DX = seconds * 256 + hundredths, so at most 6000 distinct seeds, zero-extended to 32 bits. It is called:
1. once at program start, before the title menu, so world generation is seeded from the clock;
2. at the start of every tactical battle (0x3EE60), which re-seeds the global generator in the middle of a game.

The RNG state is not saved. Loading a game continues with whatever state the generator has. Consequences for a faithful reimplementation:
- The original is not reproducible across runs. Bit-exact replay of an original session is impossible unless the seeds are captured from DOSBox.
- A deterministic port should keep the same LCG and the same `random_n` semantics (distribution and modulo bias included), seed it explicitly, store `S` in the save, and treat the two re-seed points as `srand(seed_source())` hooks.

Evidence: `rand` 0x6B491, `srand` 0x6B4B5, `random_n` 0x36FF0, `randomize_from_time` 0x37030; seed state 0xB9284.

## 2. Calendar [C]

- One turn = one year. The year counter is a u16 at `0xEF4B8`.
- A new game starts in 1600 (`world_init`, 0x4A490).
- The main loop stops when the year reaches 2000 (it runs while `year <= 1999`); then the program returns to DOS without further message. There is no other hard end date. The option "Spielende um 1800" (0x400) does not end the game; it only extends the last year in which children can be conceived from 1669 to 1769 (`rules-00` §11), so dynasties die out later.
- Several formulas use the year directly (wage drift, transporter prices, price inflation, production year factor). They are given in the economy doc.

## 3. Program and year loop [C]

Program start (game_main 0x4AB90):
1. Seed RNG from clock.
2. `world_init` builds a complete new world (always, even if a save is loaded afterwards; loading then overwrites it).
3. Title menu: new game / load / continue last game / quit. New game: choose mission difficulty (0..3 = keine, leicht, mittel, schwer, stored at `0x85CBE`), number of players 1..6, then `setup_human_player` once per player in order.

Every human person has controller nibble 1 (high nibble of person byte 0x10; `setup_human_player` always passes 1). Computer persons have 0.

After "new game" the current person is set to the first human. After "load" it comes from the save. Then:

```
loop:
    if year >= 2000: exit program
    save to slot 5 ("letztes Spiel"), save config
    if current < 0xC0:                      -- a human is due
        play_turn(current)
        compact record heap
        current = next_human(current)       -- 0xC0 or more when none is left this year
        continue loop at play_turn without saving again while current < 0xC0
    current = first human (lowest slot)
    if there is none: show `[EXE text]`, back to the title menu
    year_end_world()              (0x66220)
    compact record heap
    year = year + 1
    city_growth()                 (0x4A3C0)
```

So the autosave happens once per year, at the start of the year, and the year-end processing runs after the last human of the year.

`next_human(i)`: the smallest index j > i whose controller nibble is nonzero. Humans play in ascending person-slot order. Starting players are created in order and take the first free slot, so this is normally creation order. A human who joins mid-game ("Neuer Spieler") takes the first free slot (for example a slot freed by a death) and so may play before older players.

### 3.1 One human turn (play_turn) [C]

The phase byte `0xEF4C5` is saved with the game, so a save made inside a turn resumes in the same phase.

1. Prison check: if the person has a record of type 20 (and separately type 21) [types 20/21 = jail and debtors' tower, L]: if its counter byte 3 is 0, delete the record; otherwise decrement it, show the "sitting out" message, and set phase 6 (turn skipped).
2. Phase 0, `turn_start_reports` (0x4B900): results of last year (production, sales, arrivals), new storage offers, new loan offers. Ends by setting phase 4. In 1600 the result screens are skipped (nothing was ordered yet) but the offers are still generated.
3. Phases 1..4, `kontor` (0x5D610): interactive screens. Phase 4 is the Kontor itself, 1..3 are sub-screens that return to 4. Ending the round (right click, optionally confirmed) sets phase 5.
4. Phase 5, `turn_end_settlement` (0x61A70): see economy doc. Ends with phase 6.
5. Death, debt seizure and similar events can set phase 6 early.

**Prison turn in detail (correction: there is no "sitting out" message) [C]** (`0x4B161..0x4B222`, current human `0xEF4C6`):
- Type 20 (Schuldturm), then type 21 (Kerker), are looked up with `rec_find(type, player, 0xFF)`. For each record found:
  - counter `+3` = 0: the record is deleted (bit 7, `0x4B199` / `0x4B1F5`) and **nothing is shown**. The turn is then played normally. This is the release, and it has no message.
  - counter ≠ 0: phase `0xEF4C5 := 6`, counter − 1, and `history(player, 0, 0, 0)` (`0x2D750` at `0x4B1B8` / `0x4B214`). That is a chronicle record only, not a window. Both records are checked in the same turn, so a player holding both writes two entries.
- Phase ≥ 6 jumps to `0x4B2D4`: record compaction `0x42580`, then the next human (`+0x10 & 0xF0`, slot order) with phase 0, and the loop continues. **No window, box, picture, text, music or sound appears, and there is no click wait.** The skipped player's `turn_start_reports` (phase 0, where the turn opening is shown) never runs, so he gets no Erträge page, no offers, no Kontor and no settlement. No RNG is drawn.
- Remaining time is never shown. A counter of N means N lost turns. The record is deleted at the start of the turn after them, and that turn is played. The sentence (`rules-04b` §3, `rules-02` §4: counter 1) therefore costs exactly one turn.
- The only visible trace is in the history/chronicle screen (`0x17930`): code 0 reads `0xBA5C4` `[EXE text]`, with the year of the lost turn [L: code → text mapping from `rules-41` §2]. Kerker (type 21) also writes code 0, so it shows the same "Schuldturm" line.
- The visible prison moments are elsewhere: the sentence itself (trial verdict `0xA8154` / `0xA81CC`, debtors' tower scene `0xC1320` / `0xC13A8`, `rules-15`).

### 3.2 Year end [L]

`year_end_world` runs once after all humans. Coarse order (details in the subsystem docs):
1. Favour adjustments toward humans; every favour value below 10 rises by rand(2).
2. Dismissals of office holders whose net worth fell (career doc).
3. Vacant-office handling and new computer persons; elections and appointments.
4. Ageing, deaths, inheritance, disasters (`disaster_damage`), auctions, trials, duels, sieges, computer-person actions and misfortunes, chronicle of the year.

### 3.3 City growth after the year counter is incremented [C]

For each city c (in order 0..9):
- `population = population * (1000 + random_n(50)) / 1000` (unsigned, truncating): 0.0 % .. 4.9 % growth per year.
- city dword at +0x84 (likely branch-rights price) `+= random_n(200)`.
- city dword at +0x88 `+= 1` if `random_n(100) < 7`.

The RNG calls happen in exactly this order per city (3 calls per city).

## 4. Evaluation-order notes for parity

- Prices are recomputed at the start of every human's settlement, not once per year (economy doc). Players later in the order see prices moved by earlier players' sales.
- Weather (harvest quality) is rolled inside each human's own turn start, so two humans get different harvest quality for the same good in the same year.

## 5. Game clocks and delay resolution [C unless marked]

The game has no clock of its own. Every timer reads the DOS time with **INT 21h AH=2Ch** (CH h, CL min, DH s, DL hundredths). Nothing reads the BIOS tick word `0040:006C` (no reference to 0x46C in the image), and nothing reprograms the PIT (no `out 0x40`/`out 0x43` and no port-0x43 load in the image).

| routine | used by | how it counts | how a wait of n ends |
|---|---|---|---|
| `clock` `0x6AC50` → `0x6B6E5` | `menu` repeat and timeout (`rules-54` §4, `rules-50` §3), auction ticks | `0x6B67F`: `((h·60 + m)·60 + s)·100 + hs` via `0x6C3AC` (INT 21h); minus the base `0xEF4E0` set at start-up (`0x6B6DA`); +8 640 000 on a midnight wrap | caller loops `clock() ≥ start + n` |
| FLC frame pacing (inside `0x14ED6`) | all films (`rules-56` §10) | own INT 21h/2Ch at `0x14F5E`: `(h·60 + m)·6000 + s·100 + hs`; base taken at the first frame (`0x835B7`); +8 640 000 on a wrap | busy-waits until `elapsed ≥ (frame + 1)·d` |
| `delay_ticks(n)` `0x12DB5` | animation frame steps (auction, duel, trial: n = 5) | polls only DL (hundredths); `(DL − last + 100) mod 100 ≥ n`. `last` (`0x83448`) is the DL at the **end of the previous call**, so successive calls pace from each other, not from their own start; n is effectively below 100 | returns on the first poll that satisfies it, then stores DL |
| `delay_ms(n)` `0x6C083` | the few real-ms delays (`rules-25`) | makes `max(1, (n·calls_per_s + 500)/1000)` INT 21h/2Ch calls, with `calls_per_s` calibrated at start-up (`0x6C05A`); counts calls, not time | ≈ wall-clock ms at the calibration's precision |

**Resolution [L: standard DOS behaviour].** DOS computes the 2Ch time from the BIOS tick count (18.2 Hz). The hundredths value therefore advances in steps of 5 or 6 (about 54.9 ms). For every hundredths-based wait above:
- It ends on the first tick at which `elapsed ≥ n`.
- The effective wait is n rounded **up** to whole ticks, measured from where the base fell inside its tick. One wait lasts between ceil(n/5.49)−1 and ceil(n/5.49) ticks, 0..54.9 ms off.
- Examples: `delay_ticks(5)` = one tick (54.9 ms) per frame. The menu repeat delays 17/13/9/5/1 hundredths become 3–4, 2–3, 1–2, 1, 0–1 ticks. The FLC frame pacing at d = 10 averages 10 fps but quantises each frame to 54.9 ms steps (e.g. 2 ticks, then 2, 2, 1, 2 …).
- The menu timeout of 150 hundredths ends after 27–28 ticks.

**PIT and sound [UNVERIFIED].** The image contains no PIT programming. The digital mixer and the MIDI timing run in interrupt code that is not in the image (`rules-14`). If that driver speeds up IRQ 0, a well-behaved driver still chains the BIOS tick at 18.2 Hz, and the DOS time stays at its normal resolution. The game's own INT 1Ch hook (`0x2D990`, flag animation, `rules-39` §1) assumes the 18.2 Hz tick.

**Answer:** all delays use DOS INT 21h/2Ch time in hundredths, at an effective resolution of one BIOS tick (54.9 ms). A wait of n hundredths ends at the first tick with elapsed ≥ n, i.e. it rounds up to ticks. The game does not reprogram the PIT.

## Open questions

1. Phase 1..3 meaning (which Kontor sub-screens) is UI only and not needed for rules.
