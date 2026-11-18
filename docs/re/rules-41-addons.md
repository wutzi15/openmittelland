# Add-on answers (rules-41)

Four follow-up questions from the round-3 SPEC-GAP pass (`rules-40`). Conventions as in `rules-40` (header): `rand(n)` = `0x36FF0`, `rec_new` = `0x424A0`, `persons` = `0x433E0`, `person_pick_random` = `0x43560`. Tags: **[C]** instruction level, **[L]** decompiler only, **[UNVERIFIED]** inferred. Everything is [C] unless marked.

## 1. Title award pickers (`0x49700`)

The authority is picked inline in `0x49700`, by the title branch after the screen is opened (`0x49912` dispatch on n = title + 1). Two helpers are used [C]:
- `pick_first` = `0x433E0`, called with place in EAX, `0` in EDX, office in EBX, `0xFF` in ECX: the lowest person index whose slot is used (+0x00 ≠ 0xFF), +0x0F = place, low nibble of +0x10 (church flag, `rules-00`) = 0, +0x11 = office (0xFF = any). The controller nibble is not tested (ECX = 0xFF), so humans, including the player himself, qualify. No RNG.
- `pick_random` = `0x43560`, called with all four arguments 0xFF: collects every used person slot in index order (any place, church flag, office, controller; the player is included) and draws **one `rand(count)`**. With all filters open the list is never empty, so this fallback always succeeds and always draws exactly once.

Per title [C]:
- **n = 1** (`0x49936`): `b = branch_random(kind 0, any place, owner player)` (`0x42B90`, one `rand(n_branches)`, drawn also when n = 1), `c` = b's place (property byte +0x01). Mayor = `pick_first(c, 0, 6)`; else `pick_first(c, 0, any)`; else `pick_random`.
- **n = 2** (`0x49A2D`): same branch draw, `k = (c + (c == 0) − 1) / 3 + 10`. Vogt = `pick_first(k, 0, 7)`; else `pick_first(k, 0, any)`; else `pick_random`.
- **n = 3, 4** (`0x49B36`): Regent = `pick_first(13, 0, 7)`; else `pick_first(13, 0, any)`; else `pick_random`. No branch draw.
- **n ≥ 5**: emperor from the type-1 record, no draw.

"Vacant seat" is the same chain: the second step is **any person at that place with church flag 0**, office holder or not (correction to `rules-03` §2.1 "any office holder there"); the third step is any person at all. Since `pick_random` cannot fail, the "none → no text, no seal" path of `rules-27` §1 is unreachable [C for the argument values, L for "never empty": the player's own slot always counts].

RNG order of one award [C]:
1. year = 1600 (`0x49711`): return, nothing drawn.
2. `rand(100)` gate (`0x4973A`, < 6·title → return).
3. `rand(100) < 80` (`0x49838`) only if `cap > title`.
4. n = 1/2: `rand(n_branches)` (branch), then `rand(n_persons)` only when both `pick_first` calls returned 0xFF. n = 3/4: `rand(n_persons)` only on that last fallback.
5. Title byte written (`0x49C7F`), `music(0x1E, 0)` → `rand(2)` (`0x49C8C`), `wait_click`.
6. n = 3: one more independent `rand(n_branches)` (`0x49CA8`) for the Ritter page.

Verdict: `rules-27` §1's order (branch draw, optional `pick_random`, then fanfare `rand(2)`) is right. Fix the second fallback's meaning (any person at the place with church flag 0, controller ignored, so possibly the player or another human) and treat `pick_random` as always succeeding with exactly one draw.

## 2. History record call sites (`0x2D750`)

Writer `0x2D750(person, code, a, b)` (register arguments EAX, EDX, EBX, ECX) [C]: `rec_new(0x39, person, 0xFF, 25)`, then `+4 = code`, `+5` = u16 current year (`0xEF4B8`), `+7 = a`, `+8 = b`. It draws no random numbers.

There are 12 call sites. Ghidra's cross-references and a raw scan of the image for `E8` near calls aimed at `0x2D750` give the same 12. The address never occurs as a dword anywhere in the image, so there are no indirect calls through a table or function pointer [C].

| call | containing function | code | person | a (+7) | b (+8) | condition |
|---|---|---|---|---|---|---|
| `0x4B1B8` | turn start (`0x4AB90` block) | 0 | current player `0xEF4C6` | 0 | 0 | type-20 record (Schuldturm) with counter ≠ 0: the counter is decremented and the turn is lost [C] |
| `0x4B214` | same | 0 | current player | 0 | 0 | same, for the type-21 record (Kerker) [C] |
| `0x21B72` | courtship `0x212B0` | 1 | record owner (+1) | 0 | 0 | wedding [C] |
| `0x208FA` | feast `0x20530` | 2 | record owner (+1) | feast quality q | 0 | after the outcome box [C]; that `a` is the quality comes from `rules-08` §2 [L] |
| `0x1B395` | concert `0x1AEA0` | 3 | record owner (+1) | record +7 | record +8 | after the payout (`0x43790`) [C]; the meanings of +7 and +8 (size, grade) come from `rules-06` [L] |
| `0x22460` | birth `0x221F0` | 4 | record owner (+1) | record +4 & 1 (sex) | 0 | [C] |
| `0x682B9` | year end `0x66220`, office election | **5** | the winner | global office index o | place L | see below [C] |
| `0x192A0` | city disaster `0x18BD0` | 6 | person in the city | location byte (+1) of that person's branch (= the city) | 0 | disaster record +4 = 2 (plague); human only; one entry per branch [C] |
| `0x192C6` | same | 7 | same | same | 0 | disaster record +4 = 1 (storm flood) [C] |
| `0x4001C` | battle `0x3EE60` | **8** | attacker `0xDB7F4` | battle field (`0x85803[code]`) | 0 | see below [C] |
| `0x54F28` | castle info `0x549C0` | 9 | buyer (interest record +1) | castle/camp location | 0 | human buyer only; private castle sale [C] |
| `0x6948C` | year end `0x66220`, computer castle sale | 9 | buyer | location | 0 | human buyer only (`rules-23`) [C] |

**Code 5 is written** at `0x682B9`, in the vacancy election of `year_end_world`. Three conditions must all hold:
- Exactly one candidate has the top vote count. The counter at `[EBP−0x1C]` counts how many candidates are tied at the top, and only a count of 1 reaches this branch.
- The winner is human (`+0x10 & 0xF0`).
- The global office index o is 16 (Kanzler), 22 (Bischof) or ≥ 29 (principality offices).

It is written right after the line `0xC1934` "Damit wird %1F %s in %o." The tie path (`0xC1954` "Das Los entscheidet…", `0x682C0`) writes no history entry. Neither does the computer-rule path (`0xC1978`, `0x68410`). No other promotion path calls the writer.

On the history screen (`0x17930`, at `0x1841F` and `0x18486`), +7 selects the office noun: `0xB4378[a]` = T[72 + a] for a man, `0xB5854[a]` for a woman. +8 is the `%o` place, and the text is `0xBA67C` / `0xBA6AC` `[EXE text]`. The prompt's "0x38E" is not a text id: it is a constant (910) pushed in the box-call argument list at `0x1843C` [C].

**Code 8 is written** at `0x4001C` in `battle_main`, but not for a completed raid. The defender of a raid makes a ransom offer, and a human attacker is then asked `0xAD431` "…soll der Angriff stattfinden? / Gewiss, he he! / Ich lasse Gnade vor Recht ergehen." The entry is written when the attacker picks item 1, "Gnade vor Recht", which means taking the ransom and calling the attack off.
- In that case the battle is skipped (`[EBP−0x38] ≠ 0`, test at `0x40104`). The offered sum moves from the defender to the attacker (`0x43860` / `0x43790` at `0x400F0` / `0x400FF`).
- Item 0 ("Gewiss") zeroes the offer, the battle is fought, and no entry is written.
- A computer attacker never reaches this call.

The screen text `0xBA770` `[EXE text]` (read at `0x186B5`) uses +7 as the index into the background table `0x857FE`. The name in `rules-06` ("robber camp opened") is therefore wrong: code 8 is "ransom accepted instead of a raid" [C for the branch, L for the reading of the menu index: 0 = first item, the same convention as the defender prompt at `0x3FE84`].

RNG: the writer draws nothing. Code 8 sits after the `0x35B60` menu wait, and that branch skips the `rand(100)` at `0x40059`, which belongs only to the computer attacker's path.

## 3. Wait cursor (`0x36720`)

**Mechanism [C].** The mouse cursor is a software sprite. `0x84D68` = requested cursor id, `0x84D64` = id currently drawn, `0x84D5C`/`0x84D60` = mouse x/y. `0x2EC80` restores the 32×40 background under the old position and draws the sprite. `0x2E5B0` does the same from the mouse-driver callback, installed in `0x2F4B0` with INT 33h function 0Ch and mask 1 (movement), so the sprite follows the mouse. The sprite is drawn at (mouse x, mouse y) with its top-left corner there, so the **hotspot is (0,0)**. The save/restore rectangle is 32×40, clipped to `480 − y` rows from y = 441 on.

**Id → icon [C].** Ids 1..5 come from cache slots `0xC399C + 4·id`, i.e. icons 0..4 (ICON00..ICON04). Ids ≥ 6 come from `0xC3F3C + 4·id` = icon `0x167 + id`. In the two-digit file naming (first digit 0-9/A-Z, second 0-F; e.g. `0x196` = ICONP6), ids 6..21 are icons `0x16D..0x17C` = **ICONMD, ICONME, ICONMF, ICONN0 … ICONNC (16 files)**. `0x2F4B0` (graphics init) preloads exactly these 16 icons and pins them (`0x2DD70` loop at `0x2F636`, `0x16D + i` for i = 0..15), together with icons 0..0x21. **ICONND (`0x17D`) is neither preloaded nor reachable as a cursor id**, so the wheel uses 16 frames, not 17. Several scenes draw icon `0x17C` (NC) directly: `0x1AEA0`, `0x2C750`, `0x5CCA0`, `0x61A70`. That use was not checked [L].

**wait_click `0x36720` [C]:**
1. Window refresh (`0x30400`). The text cursor is set to the window bottom. One window item, style `0xC90`, is created (`0x34130`) at (window width − w[0], window height − h[0]), i.e. icon 0 in the window's bottom-right corner [L for what that item shows]. It is removed on exit (`0x34930`).
2. Wait until all buttons are up (`0x12DE4` returns 0).
3. Loop, frame counter i = 0, 1, 2 …, until a button is down:
   - cursor id := `6 + (i mod 16)`;
   - if it differs from the drawn id, redraw (`0x2EC80`);
   - i += 1;
   - redraw again (`0x2EC80`);
   - `0x12DB5(5)` waits until the DOS clock (INT 21h AH = 2Ch) hundredths have advanced by ≥ 5 since the last stamp `0x83448`. That stamp is global and shared with the other `0x12DB5` users, so the first frame can be immediate.
4. On press: `0x12698(0x42, 3)` for button state 1 (left), `0x12698(0x4C, 3)` otherwise. Then wait for release and remove the corner item.

**Frame order and timing.** MD, ME, MF, N0, N1 … NC, then MD again. Each call starts at MD [C]. One frame per ≥ 5 hundredths. On real DOS the hundredths come from the 18.2 Hz BIOS tick in ~5.5 cs steps, so the wheel advances **one frame per timer tick (~55 ms), ~0.88 s per turn** [L: the tick granularity depends on the DOS clock source, so under DOSBox it is ≥ 50 ms]. No game-RNG use.

**After the wait [C].** `0x36720` does not restore the cursor id. The last wheel frame stays as a static cursor until something else writes `0x84D68`:
- `0x35B60` (menu) writes 1 when the id is 0 and 1..5 on hover changes;
- `0x2EC40(id)` call sites always pass 0 or 1;
- `0x2FA80`, `0x30190`/`0x30308` and `0x31610` (the latter only when drawing to the screen page) write 0, which hides the cursor.

**Who uses it [C].** Only `0x36720` ever writes ids ≥ 6, so the wheel appears exactly while `wait_click` runs. Its callers:
- print `0x322C0`: two control-code paths at `0x32758`/`0x32771`, the explicit wait and the full-window wait [L which codes];
- line layout `0x31610`: text overflowing the window bottom, `0x31E17`;
- message boxes: `0x36C90` only when its delay argument is 0 (otherwise a timed delay `0x6B07C` with no wheel), and `0x36E50` at `0x36FC6`. The `0x36E50` call is made only when `0x12C71` is already non-zero on entry; otherwise it polls `0x12C71` with no wheel [L for what `0x12C71` tests];
- the Erträge report `0x4B900` (`0x4BA2E`, `0x4C800`), year end `0x66220`, settlement `0x61A70`, auction `0x26A50` and settlement `0x27680`;
- battle `0x3EE60`, the speech/charter screens `0x494F0` and `0x49700`, Spielstand `0x5C450`, matchmaker `0x20D30`, feast planning `0x20160`, privileges `0x5F080`, births `0x221F0` and `0x4B3A8`.

The menu `0x35B60` uses only ids 1..5 (arrow/hand shapes), so it never shows the wheel. Film and speech playback (`0x15570`) do not write `0x84D68`. Their waits show the wheel only where they end in `wait_click`, as `0x494F0` does.

## 4. Prisoners handler RNG (`rules-38` §7)

`add_troops(prop, men, training)` `0x45240` [C]: first sums the four troop bytes `prop.+0x58..+0x5B`. If sum + men exceeds the capacity byte `prop.+0x04`, men becomes `capacity − sum`. This can be **negative** when the property is already over capacity: the subtraction is signed and nothing clamps it. Only when the capped men is **> 0** does it draw `start = rand(4)` (`0x452C1`). It then adds one man per step round-robin from slot `start`, and each step blends the slot's training byte `+0x5C+s` as (old·count + training) / (count + 1). The return value is the capped men, which may be 0 or negative. So **no `rand(4)` is drawn when 0 men fit, when the property is over capacity, or when `men = 0`.**

Handler `0x63D0D`, per type-55 record [C]:
1. `n = +4`. If `n ≤ 0`, the record is deleted with no draw.
2. `rand(30)` (`0x63D4E`), drawn unconditionally whenever n > 0, before `add_troops` is called.
3. `add_troops(prop, n/2, 30 + t)`, which draws `rand(4)` only when `min(n/2, capacity − sum) > 0`. With n = 1, n/2 = 0 and there is no draw.
4. Texts as in `rules-38` §7. The print calls draw nothing [L: `0x322C0` was not re-read for RNG]. Quirk [C]: when the property is over capacity, `m` is negative. The "schlechten Tag" / "langweilte sich" line is then printed, and the vanished count is `n − m`, which is larger than n.

RNG order per record: `rand(30)`, then `rand(4)` if at least one man is added.

Verdict: the `rules-38` §7 [UNVERIFIED] note is resolved: `rand(4)` is drawn only when the capped men count is positive.

## Summary

| item | answer | section |
|---|---|---|
| title award `0x49700` pickers | `rules-27` §1 RNG order correct; second fallback is any person at the place with church flag 0 (may be a human); `0x43560` always succeeds with one draw | §1 |
| history codes 5 and 8 (`0x2D750`) | code 5 written at `0x682B9` (year-end election, human winner, offices 16, 22, ≥ 29); code 8 at `0x4001C` (`battle_main`, ransom accepted via "Gnade vor Recht"), not "robber camp opened" | §2 |
| wait cursor (`0x36720`) | ICONMD..NC (16 frames, ND unused), ≥ 5 hundredths per frame, hotspot (0,0), only during `wait_click`; no RNG | §3 |
| prisoners `rand(4)` (`rules-38` §7) | drawn only when the capped men count is > 0; the handler draws `rand(30)` first | §4 |

## Corrections to other docs

- `rules-03` §2.1: the title-award fallback is "any person at the place with church flag 0" (office holder or not, human or computer), not "any office holder there"; the "none → no text, no seal" path of `rules-27` §1 is unreachable [L] (§1).
- `rules-06` (history codes): code 8 is the accepted ransom ("Gnade vor Recht"), not "robber camp opened" (§2).
- `rules-38` §7: the `rand(4)` of `add_troops` is drawn only when at least one man fits (§4).
