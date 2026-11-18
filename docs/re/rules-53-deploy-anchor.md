# Deployment anchors and lines (rules-53)

Source: `deploy(field, type, n, participant, side, training)` `0x3DDA0..0x3E293`, object allocator `alloc(type, unit, col, row, d)` `0x3D7C0..0x3DC33`, and the callers in `battle_main` (`0x3F795..0x3F8AB` defender, `0x3FB77..0x3FCD5` attacker). Cell byte `S[f][r][c]` from `SZENARIO.DAT` (low 5 bits = code, high 3 bits = d). Occupancy grid `0xDBD6C` is initialised from `0x85809[code]` (0xFF = free), and the flag grid `0xD979C` from `0x8581D[code]`. Tags as in `rules-50`. Everything is [C] unless marked. This refines `rules-07` §5.4.

## 1. Order of calls

- **Castle/camp defender** (codes 4/5): for k = 0..3, `deploy(field, base + k, n = P[+0x58+k], participant 1, side 1, training P[+0x5C+k])`, with base 0 (castle) or 5 (camp). Then the leader: `deploy(field, base + 4, 1, 1, 1, 87)`. So the order is Söldner/Räuber, Musketier/Flintenschütze, Kanonier, Offizier/Bombenleger, General/Hauptmann.
- **Attacker** (after intro line 4): for k = 0..4, type `(home kind = 2 ? 0 : 5) + k`, `n = k = 4 ? 1 : rec[+5+k]` (computer attackers capped, `rules-07` §5.4), participant 0, side 0, training `home[+0x5C+k]`.
- A count of 0 deploys nothing (`deploy` returns 0 at once).

## 2. One `deploy` call

1. **Unit slot**: the first unit record u (0..99, `0xD7534 + 22u`) whose byte +1 is 0xFF. If there is none, return 0.
2. **Entrance reference** (only side 1 on fields ≥ 2): the first cell in row-major order (row 0..45, col 0..29) whose flag has bit 2 (code 3). This is `(ex, ey)`.
3. **Candidates**: every cell with code `a = 0x85831[type] + 6·side` (= 8 + class + 6·side) that is **free in the occupancy grid**. They are scanned row-major. Two criteria run in the same pass:
   - `score = 4·(c − 15)² + (r − 20)²`; keep the cell if `score > best` (strict, start −1), so the first maximum wins;
   - side 1 on fields ≥ 2 only: `dist = (r − ey)² + (c − ex)²`; keep the cell if `dist < bestd` (strict, start 100000), so the first minimum wins.
4. If no candidate exists (`best < 0`), return 0 and the remaining men are lost. If a `dist` minimum exists, the **entrance-nearest** cell is used. Otherwise the far-from-centre cell is used. There is no `rand` anywhere in deploy.
5. Unit record: +0 type, +1 participant, +2 side, +3 training, +0xE tactic `0x857D8[type]`, +0x0F 0, +0x10 10, and **+0x12 = best**. best is the far-from-centre score maximum, stored even when the entrance-nearest anchor was used. It is the act-order key.
6. `d = anchor >> 5`. The continuation byte is `cont = (anchor & 0xE0) + 7` (code 7 with the same d). The step is `s = 2` for types 2/7 (gunners), otherwise 1. The direction is `(d + 2) mod 8` with `dx = 0,1,1,1,0,−1,−1,−1`, `dy = −1,−1,0,1,1,1,0,−1`.
7. **Line loop**, starting with m = 0 at the anchor:
   - `alloc(type, u, c, r, d)`. If it fails, stop.
   - Step `(c, r) += s·(dx, dy)`, `m += 1`.
   - If `m ≥ n`, stop.
   - Continue only if the new cell's **full byte equals `cont`** and it is free. Otherwise stop.
8. Types 20/21 (markers, wagons): the unit record is released (+0 := 0xFF). Every other type is inserted into the act-order list `0xDC83C` sorted by +0x12 descending. A new unit goes before the first unit with a strictly lower key, so equal keys keep insertion order.
9. If `m < n`: `return m + deploy(field, type, n − m, participant, side, training)`, a new unit at a new anchor. The anchor just used is now occupied, so the next-best anchor of the same code is taken. **No 10-men cap** is applied in `deploy` itself. The line length is limited only by the continuation cells. A unit that is full is reported by `alloc` via the error routine `0x2FA00` ("alloc_Objekt: Einheit full") [L: whether that aborts was not traced].

## 3. `alloc` and the cannon

- A man needs his cell free (occupancy 0xFF). Otherwise `alloc` returns 0xFF and the line stops.
- Gunners (types 2/7): the cannon (object type 16, same unit, same facing d) goes on `(c + DX[d], r + DY[d])`, with `DX` `0x84F50` = 1,1,0,−1,−1,−1,0,1 and `DY` `0x84F60` = 0,1,1,1,0,−1,−1,−1. That is the next cell in the line direction `(d+2)`, the cell the step of 2 skips. The cannon cell needs only to be free; its code is not checked. If it is not free, the gunner is also not placed (his type is reset to 0xFF), and the line stops.
- Quirk: if this happens to the first man (m = 0), the empty unit is still linked, and the recursion picks the same anchor again (still free). This repeats until the 100 unit slots are used up. It does not occur on the shipped fields in the cases below.
- Occupancy marks: a man or cannon occupies its cell. Markers occupy (c, r) and (c+1, r). Wagons occupy a 3×2 block (c..c+2, r−1..r).

## 4. Castle field 3, defender

- Entrance reference: **(22, 14)**.
- Cannon anchors (code 16) by entrance distance: (20,16) d5 → (24,19) d5 → (19,21) d5 → (25,6) d5 → (23,32) d6 → (5,5) d3.
- Simulated with the rules above (offline from `SZENARIO.DAT`; identical with Söldner/Musketier/Offizier counts 0, 20 or 99, so the other slots never block the cannons):

| gunners | units (anchor) | gunners at | cannons at |
|---|---|---|---|
| 1 | (20,16) | (20,16) | (19,15) |
| 2 | (20,16) | (20,16), (18,14) | (19,15), (17,13) |
| 3 | + (24,19) | + (24,19) | + (23,18) |
| 4, 5 | + (19,21) | + (19,21), (17,19) | + (18,20), (16,18) |
| 6 | + (25,6) | + (25,6) | + (24,5) |
| 7, 8 | + (23,32) d6 | + (23,32), (23,30) | + (23,31), (23,29) |
| 9..11 | + (5,5) d3 | + (5,5), (3,7) | + (4,6), (2,8) |

  Line capacities in that order: 2, 1, 2, 1, 2, 2 gunners. Beyond 11 gunners no code-16 anchor is free, and the extra gunners are not deployed.

- Other defender anchors chosen, in order, for large counts: Söldner (code 14) (15,15), (13,22), (15,24), (18,27), (22,28), (7,12). Musketier (15) (21,13), (27,18), (22,21), (8,9), (9,6), (21,32). Offizier (17) (22,17), (15,13), (14,20), (16,22). General (18) (26,8).

Answer: per unit, the anchor is the free cell of code `8 + class + 6·side`. The defender on fields 2..4 takes the cell with the smallest squared distance to the first entrance square in row-major order (first minimum, no rand). Otherwise the cell maximising `4(c−15)² + (r−20)²` (first maximum) is used. The line runs from the anchor in direction `(d+2) mod 8` with step 2 for gunners, while the next cell's byte equals `(anchor & 0xE0) + 7` and is free. Each gunner's cannon sits on the skipped cell `(c + DX[d], r + DY[d])` with facing d. A blocked man or cannon cell ends the line. Leftover men start a new unit at the next anchor, and are dropped when none is left. The slot order is k = 0..3, then the leader. For field 3 the cannons for 1..3 units sit at (19,15)/(17,13), (23,18) and (18,20)/(16,18).

## 5. Unit capacity and "Einheit full" (follow-up)

1. **Capacity [C].** A unit record (`0xD7534 + 22u`) holds **10** member slots, the bytes `+4..+0x0D` (`0xD7538 + 22u + k`, k = 0..9, 0xFF = empty). There is no count field. For a fighter (type < 12) `alloc` scans k = 0..9 for the first 0xFF (`0x3DBCF..0x3DBFA`). If one is found, it stores the object index there (`0x3DC12`). If none is found (k = 10), it calls `0x2FA00("alloc_Objekt: Einheit full")` (`0x3DC1A`). By then the man's object record and occupancy cell are already written (`0x3DA23..0x3DBC5`).
2. **`0x2FA00` is fatal [C].** It restores the video mode (`0x131DB`), prints `**** error at %s. Program terminated. ****` and the `[deb1..deb4]` line, waits for a key (`0x6AC77`) and calls `exit(1)` (`0x6AF62`). It never returns, so `alloc` has no return value in this case.
3. **Caller [C].** `deploy`'s line loop never tests the unit's fill; it stops only on `m ≥ n`, a non-continuation cell, an occupied cell or a failed `alloc` (§2 step 7). A line longer than 10 cells would therefore end the program; leftover men never reach the "new unit at the next anchor" path. The remainder rule (`m < n` → recursive `deploy`) applies only when the line ends for one of the other reasons.
4. **Why it never happens [computed from `SZENARIO.DAT`, all five fields].** The longest possible line from any anchor (codes 8..19, following the continuation cells with step 1, or 2 for gunners, ignoring occupancy, which only shortens it) is:

| field | longest line (anchor code: men) |
|---|---|
| 0 | 5 (codes 8, 9) |
| 1 | 5 |
| 2 | 5 |
| 3 | 6 (code 15, defender Musketier) |
| 4 | 5 |

   Gunner lines are at most 3. So no line reaches 10 for any n, and callers need not cap n. They do not: a human attacker passes the record bytes `+5..+8` (up to 255 each), and the defender passes the garrison counts. The only cap is the computer-raider cap `max(1, value/350)` (codes < 4, computer attacker, `0x3FBC6..0x3FC1D`), and it serves a different purpose (`rules-07` §5.4). Men beyond the free anchors' total line capacity are dropped silently (§2 step 4).
   - Related unchecked limit [C]: `alloc` looks for a free object among 200 (`0x3D7D7..0x3D7FE`) with no failure branch; with none free it would use index 200. The anchor lines of one field add up to at most 83 men plus cannons per side (field 3: 83 + 83), so 200 is not reached by men alone [inferred, ignoring overlaps; raid markers and wagons add a few more].

Answer: a unit holds 10 men in slots `+4..+0x0D`; an 11th `alloc` into the same unit is a fatal error (`0x2FA00` prints and exits) after the man is already on the field. `deploy` does not split lines at 10. This is harmless only because no SZENARIO line is longer than 6 cells (3 gunners). A port can treat the 10-cap as unreachable, or split at 10 for safety, which differs from the original only on modified fields.
