# Sim readings settled from the code (rules-55)

Answers to `docs/IMPL-GAPS.md` "Readings chosen", items 1–14 of the 2026-11-16 batch. Tags as in `rules-50`. "Port" = the reading listed in IMPL-GAPS.

### 1. Level A/B roll order in `prices_update` (`0x49FE0`)
For each city c = 0..9 (`0x4A03C`):
1. the 15 goods prices (`0x4A0B9..0x4A292`), which draw no `rand`;
2. A (`rand(12)` at `0x4A2C3`, stored at +0x6F `0x4A2F0`);
3. B (`rand` at `0x4A320`, stored at +0x71 `0x4A34D`);
4. the transporter prices.

The draw sequence is therefore A0 B0 … A9 B9 [C]. **Answer:** port confirmed.

### 2. `add_goods` at a castle or camp (`0x439F0`)
- There is no special case for castles or camps, but `property_create` `0x42870` gives every kind ≠ 0 a wood and stone area of 4999 m² each (`0x4295C`/`0x42965`). The capacity `(4999 + 4999)·17 + rand(100)` is therefore larger than the largest possible stock (15·9999). The give-away loop, with its out-of-table price read, never runs [C].
- Still done on every call:
  - the `rand(100)` (`0x43AB3`);
  - the per-good clamp to 9999, with `0x85CC1 += 9999 − n` [C].

**Answer:** nothing is given away (port confirmed). Add the unconditional `rand(100)` draw and the 9999 clamp/counter.

### 3. Base S of a storage offer (`0x4CA01..0x4CA4D`)
- `S = branch[+0x23] + branch[+0x25]` of the player's branch in c, or 15 without a branch [C].
- Per slot, the residence-offer draws `rand(100)`, `rand(8)`, `rand(10)` come before the size roll (rules-37 step 19).

**Answer:** port confirmed.

### 4. Geheimräte/Hofräte seat groups
Same grouping as the Ratsherren: vacant index 0..2 scans applicants over 2, 1, 0 at every level (rules-40 §1, `0x66220`) [C]. Each vacant seat runs its **own** election, highest index first, with its own draws.

**Answer:** grouping confirmed. Fill each vacant seat by its own election.

### 5. Computer county holder moving within his own county (`office_eligibility` `0x47C20`, rule 3, rules-40 §2)
- City holder (l < 10): place l only. A Bürgermeister (idx 6) or Domherr (idx 9) may also take the county `0x868D4[l]`.
- **County holder (10 ≤ l < 13): place l only.** Kanzler, Vogt and Bischof (idx 6, 7, 12) may also take place 13. City offices, including those of his own county, are refused [C].
- l = 13: no restriction.

**Answer:** port wrong. "Within own county" means the county place only, not its cities.

### 6. Dismissal with cached net worth 0 (`0x663C6..0x66455`)
- `NW = clamp0(net_worth)`, and holders with office 15 are skipped.
- The branch runs if `NW < old` or `+0xF7` is set. It computes `r = NW·100 / old` with an **unguarded** `idiv [person+0x1C]` (`0x66449..0x66455`) [C].
- `old` is written clamped (new game `0x4AAF6`, each year end `0x6686C`), so it can be 0. The branch is then reached only through `+0xF7` (a successful Anschwärzen, `0x47AD0`).
- The result is an integer divide fault. The image contains no handler of its own, so the DOS/4GW default probably aborts the program [UNVERIFIED].
- Side note: `NW·100` overflows i32 above 21 474 836 [C].

**Answer:** the original divides by zero (crash). The port's r = 0 is a non-crashing deviation; mark it as such.

### 7. `person_pick_random` (`0x43560`)
Matches are collected in ascending person index (`0x433E0` first, then `0x434E0`). With no match it returns 0xFF **without a draw**. Otherwise it draws `rand(count)` once (`0x435F4`) [C].

**Answer:** port confirmed.

### 8. Blackmailer standing in for two electors (`0x66E82..0x66FB5`)
- For each appointer seat, every x with `favour(a→x) > 100` adds one entry (voter x, elector a). Only if there is none does a vote himself.
- The vote loop runs once per entry, with one `inc` at `0x67FF6`. A human gets one menu per entry: `0xC18C4` `[EXE text]` [C].

**Answer:** port confirmed, two votes.

### 9. Initial emperor number
`0x18A80` sets type-1 record `+4 := 0`, called only from `world_init` `0x4A4B5`. It is incremented only in 1612/1619/1637/1657 (`0x18AE0`) (rules-42 §6) [C].

**Answer:** port confirmed, 0.

### 10. Certificate award position
`call 0x494F0` at `0x4C9D6` runs **after** the Erträge window is closed (`0x4C9C7`/`0x4C9CC`), i.e. after the reports, arrivals, troop return, production pages and the type-10/11/13 deliveries. It runs before the storage/residence offers (`0x4C9DB`), church prices, lender mood, order refresh and directors. It is outside the `year > 1600` block, so it also runs in 1600 [C].

**Answer:** port wrong. Move it to step 18 of rules-37 §1.

### 11. Delivery of type-10/11 records
- At the owner's turn start, after the Erträge click and only when year > 1600:
  - type 10 (`0x4C80F..0x4C86A`) and type 11 (`0x4C8B0..0x4C94F`) with counter `+3 == 0` are delivered (`0x43930`, cond 100 or 100 − `rand(30)`) and deleted;
  - type 11 draws its `rand(30)` for every due record, before the cartel test (`0x4C8F4`) [C].
- The counter is decremented only in the owner's turn-end record pass (`0x63752`). The type-10/11 handler there (`0x63B5A`) does nothing else [C].
- Quirk: if the turn start is skipped while the counter is 0 (prison), the next turn end wraps it to 255.

**Answer:** port confirmed, with the refinements above.

### 12. Executed orders (status 3) at the next turn start (`0x4CBB4..0x4CC53`)
For every property of p: `0x45820` (refresh). Then for every slot with status 3: `0x49370` (reserve), status := 2, refresh again; an infeasible order becomes 0. This applies to **all** kinds [C].
- Kinds that reach status 3 at the settlement: production 1, transports 2..5, patrol 0x13, raid 0x23.
- Loot fetch (6, 7), attack and transfer (0x11/0x12/0x21/0x22) are set to 1 at execution and never reach status 3 (rules-37 §2).

**Answer:** port wrong for transports. Transports 2..5 also go back to 2 and repeat.

### 13. Bought storage offer (storage screen `0x531A0`, handler `0x5353D..0x5362F`)
- The offers are words in `0x85D01` (10 cities × 3, +10000 = stone), not records. The whole table is regenerated at every turn start (`0x4CAE2..0x4CB37`).
- On buying: let q be the offered area.
  - If area + q > 4999: buy `4999 − area`, and subtract that from the word (`0x535CA`); the rest stays offered.
  - Otherwise the word becomes 0 (`0x535EB`).
  
  Then area += q and `pay(q · price)` [C]. The screen and the director skip 0 words.

**Answer:** port confirmed (removed), except under the 4999 cap, where only the bought part is removed.

### 14. Castle table `0x8647C..0x86531` on loading GAMEn.DAT
- The save/load block list does not contain it (formats §3.2, rules-00 §10).
- No code copies start values back. The only image references are readers or field updates: screens `0x549DB`/`0x556A3`/`0x57450`, toll `0x56D12`, `0x6229B`/`0x6306A`, year end `0x690FC`, xor `+0x19` `0x55AFA`/`0x578B1`, and others.
- None of them is in load `0x42200`, save `0x42360` or new-game setup [C for the absence; L for completeness, which assumes all accesses go through those base loads].

**Answer:** the table is neither saved nor reset. After a load it keeps whatever the running program last had: EXE start values (toll 5 %, accumulator 0) after a fresh program start, otherwise the previous game's values. The port's start values match a load done right after program start.

### Note
`rules-42` §6 point 3 calls a prisoner's `+0x10` the "captor's colour". `rules-51` §1.4 shows that battle object `+0x10` is the participant index into `0xDB7FC`. The two docs should be reconciled [UNVERIFIED which reading rules-42 §6 needs].

### 15. `0x85CC0` (set to 1 at turn start, rules-37 §1 #1)
- **Meaning:** "no branch visited this turn".
- **Writers [C]:** `0x4B9D4` := 1 at turn start, and `0x4D111` := 0 when the trade map `0x4CC90` opens the branch screen `0x4FA70`. That is the only clearing site. Other ways of reaching the Kontor's sub-screens do not clear it.
- **Reader [C]:** Kontor `0x5D610`, on leaving with result 3 or ≥ 0x14 (end of round, right click) (`0x5DB74..0x5DC3C`):
  - Option bit `0x86AD1 & 0x20` ("Runde bestätigen") set: msgbox (160, 160, 320, 160) `0xC02EC` `[EXE text]` (Ja / Nein).
    - Ja (0): the round ends, **and the branch warning below is skipped**.
    - Anything else: phase := 4 (stay in the Kontor).
  - Bit 0x20 clear, `0x85CC0 ≠ 0` and option bit `0x10` ("Warnungen") set: msgbox (140, 160, 360, 160) `0xC0324` `[EXE text]` (Ja / Nein).
    - Ja (0): the round ends.
    - Nein: phase := 4 (stay).
  - Otherwise the round ends with no box.
- No RNG and no other reader.
- **Answer:** the flag only drives the "no branch visited" warning at end of round. That warning appears only when Warnungen is on and Runde bestätigen is off [L for the option bit names, from the option list order `0xBE260`].

### 16. Year-end net-worth cache (`+0x1C`, `0xE3A98 + 0xF8·p`) (`year_end_world` loop `0x6631D..0x6698D`)
- **Loop:** p = 0..191. Persons whose byte `+0x00` = 0xFF (empty slot, `0x66350`) are skipped entirely.
- **Every other person** (human or computer, with or without office) runs the body:
  1. favour drift;
  2. `NW = clamp0(net_worth(p))` (`0x663C6..0x663D0`);
  3. office 15 skips only the dismissal part (`0x663E0` → `0x6685C`);
  4. the dismissal test (the jump at `0x66439` goes to `0x6685C` when NW ≥ old and `+0xF7` = 0);
  5. the dismissal path (`0x45980`, and for a computer `0x42F60(p, 3)`).

  All three paths end at **`0x6685C`: `+0x1C := NW`** [C].
- So the cache is written for every existing person, including the unemployed (office 15) and computers. It holds the NW computed at the start of this person's step, before any dismissal effect. Only empty slots are skipped.
- After the office-income part, `+0xF7 := 0` (`0x6697C`) for every existing person [C].
- **Answer:** written for every person except empty slots. For item 6 this means old = 0 only when the person's clamped net worth was 0 at the previous year end (or at the new game).
