# Open questions, round 2 (rules-39)

Answers to nine follow-up questions plus the auction Esc key. Conventions as in `rules-38` (header): `rand(n)` = `0x36FF0`, `music(song, mode)` = `0x41E50`, `print` = `0x322C0`, `box` = `0x368E0`, `choose(base)` = `0x36850`, `rec_new` = `0x424A0`, `rec_find`/`rec_next` = `0x426B0`/`0x427B0`, `persons`/next = `0x433E0`/`0x434E0`, `favour_increase(a, b, x)` = `0x43620`, `favour_decrease` = `0x436F0`, `T[i]` = text id i of `0xB4258`. Tags: **[C]** instruction level, **[L]** decompiler only, **[UNVERIFIED]** inferred. Everything is [C] unless marked.

## 1. Map owner flags are animated by a timer interrupt (correction to `rules-38` §10) [C]

`rules-38` §10 is right about the item painter (`0x34BF0` draws frame 0 and bumps the kind from 0x110 to 0x120) and wrong about "never repainted". The repaint is done by an INT 1Ch (BIOS timer tick, 18.2 Hz) handler, not by the item painter:

- `0x2DCA0` saves the INT 1Ch vector (to `0xD6A26`/`0xD6A2A`) and installs the handler `0x2D990`; `0x2DD10` restores it. Only three screens call the pair, each around its `menu` wait: the Handel map `0x4CC90` (install at `0x4D02C`, remove at `0x4D047`, i.e. per click), the Söldner map `0x54570` (`0x5474C`/`0x54767`) and the year-end world `0x66220` (`0x676EA`/`0x682E7`). The player-setup screen never installs it, so its colour buttons stay on frame 0. This matches the live capture.
- Handler `0x2D990`, every tick: `0x84DA0 += 1`; if the draw lock `0x84D6C ≠ 0` or `0x84DA0 < 2` → chain to the old vector. Otherwise `0x84DA0 := 0` (so one animation step every **2 ticks ≈ 110 ms**) and for every item i = 1..`0x84D48` (item table `0xD5030`, 0x24 bytes per item) whose kind `style & 0x1F0` is **0x120** or **0x140**:
  1. lock++; if the lock is now ≥ 2 skip the draw (but still advance the frame, below);
  2. bank = `0xC39A0[4 · item.+0x10]` (icon cache slot of the item's icon number, 0x196 = ICONP6); nothing if the slot is empty;
  3. restore the item rectangle from the background page (`0x13539` from page `0xEF4BA` … item x/y/w/h);
  4. draw frame `item.+0x1C` of the bank at (item x, item y) with `0x13AE0`: remap flag `0x80 + 5·(style & 0xF)` for kind 0x120 (owner colour ramp, `rules-38` §10), flag 0 for kind 0x140;
  5. copy the rectangle to the visible screen (`0x2FDF0`) and back into the work page;
  6. lock−−; `item.+0x1C += 1`; if `≥ item.+0x20` → 0.
- `item.+0x20` (frame count) is set at item creation (`0x34130`): for kinds 0x110 and 0x130 it is `u32[bank] / 4` = the number of frame offsets in the bank header, so ICONP6 cycles 0..5 in order. Kind 0x130 (ICONPC, vote coin) becomes 0x140 after its first paint and is animated the same way with remap 0 wherever the handler is installed (the year-end screen).
- Consequence for the port: on the Handel and Söldner maps every owner flag runs ICONP6 frames 0,1,2,3,4,5,0,… at one frame per 2 clock ticks, all flags in phase (one global tick counter, each item's own frame index but all created with frame 0 in the same screen build). Animation is paused while the draw lock is held (during item creation, `0x49EB0` screen rebuilds and boxes that raise `0x84D6C`). Elsewhere (player setup) the flag is static frame 0.

## 2. Computer insulter, human victim: the duel question (`0x5B200` reaction case 6) [C]

Reaction record: `a` = the computer person (record +1), `b` = the human target (record +4). Preconditions, in this order: `privilege(a, a, 26, 0) ≠ 0` (`0x5E600`, the duelling code 0x1A); `fencing(a) + 20 > fencing(b)` (person +0xF6); and no type-**22** record of owner `a` with `+4 = b` exists yet (`rec_find(22, a, 0xFF)` chain). If any fails: nothing happens, the case falls through.

Otherwise, into the report window of `b` (this runs inside b's turn-end record pass; hot-seat, no separate screen):
1. `r = rand(12)` (taunt index). Speech: `0x15100(1 + rand(2), gender(a), 2, 1)` (voice 1/2 of a, `rules-14` §3), `print(0xBFCF8 "$V")`, `0x154B0(0)` (play).
2. `base = print([0xB5EF0] = 0x9B7D2 "$L%s beleidigt %1N5, %4N0 %s:$Z%s$Z$L%1N0, verlangt Ihr Satisfaktion?$Z%aJawohl!%aNein.$|", occasion, a, b, reason, taunt, b, r + 11)` with
   - `occasion = [0xB5EF8 + 4·rand(6)]`: `0x9B830` Auf einem Fest, `0x9B83F` Während einer Beerdigung, `0x9B858` Beim Kartenspielen, `0x9B86D` Während des Gottesdienstes, `0x9B889` Bei einer Ratssitzung, `0x9B89F` Auf dem Markt;
   - `reason = [0xB5F10 + 4·rand(8)]`: `0x9B8AD` aus Versehen, `0x9B8BA` im Rausch, `0x9B8C4` wegen einer Kleinigkeit, `0x9B8DD` spaßeshalber, `0x9B8EA` vor allen Anwesenden, `0x9B901` dummerweise, `0x9B90D` irrtümlich, `0x9B918` mit lallender Stimme;
   - `taunt = [0xB5434 + 4r]`: the twelve `$C>…<$N` lines `0x93AD6` Ihr seid ein Nichtsnutz!, `0x93AF5` Euer Vater war ein Halsabschneider!, `0x93B20` Ihr seid eine Schande für Mittelland!, `0x93B4E` Eure Schwester ist eine Mätresse!, `0x93B76` Ihr seid ein Fall für die Heilige Inquisition!, `0x93BAE` Ihr habt die Nase eines Bauern!, `0x93BD4` Aus Euren Augen blickt Narrheit!, `0x93BFB` Soll Euch der Teufel holen!, `0x93C1D` Hole Euch der Inquisitor!, `0x93C3D` Ihr gehört an den Pranger!, `0x93C5F` Sieben Jahre Verderben über Euch!, `0x93C88` Ich spucke auf Euch!;
   - the last argument `r + 11` is passed but has no placeholder [L].
   - RNG order: `rand(12)` taunt, `rand(2)` voice, then the argument evaluation order of the print call: `rand(8)` reason, `rand(6)` occasion. (Compare the human-insulter box in `rules-24` §1, text `0x9B99E`, which has no reason word.)
3. `k = choose(base)`:
   - **k = 0 "Jawohl!"**: `rec_new(22, a, 0xFF, 1)` with `+4 = b` — a duel record owned by the computer insulter against the human; the duel itself follows the type-22 handler (`rules-08` §3). No further text.
   - **k = 1 "Nein."** or right click (k ≥ 2): `print(0xBFCB8 "$N")`, `favour_decrease(b, a, 10)`; if `b` holds an office (`b.+0x11 ≠ 15`): for every person `x` at b's place (`persons(b.+0x0F, 0xFF, 0xFF, 0xFF)`): `favour_decrease(b, x, 1 + rand(6))`. No cash effect, no record.
4. In both cases the reaction counts as handled and the speech slot is freed (`0x15480`).

## 3. Report events the port prints nothing for [C, by reference]

| event | original output | where |
|---|---|---|
| transporters returning home (type-9 return / blocked delivery) | **nothing** per record; only the export total line of the arrivals block | `rules-23` §1 (`0x4BB53`) |
| transporters built (type 10) / repaired (type 11) delivered | **nothing** | `rules-23` §1 (`0x4C814`, `0x4C8B5`) |
| cargo damaged (shipment step 2) | **nothing**; the storm text `T[3324]` `0xB264A` and winds `T[3325]` `0xB268C` belong to the year-end fate handler `0x226F0` (`rules-12` §3.9 kinds 61/62), not to shipping | `rules-23` §2 |
| transporter lost (step 3, count > 1) | `$B` then `T[2815 + type]` (table `0xB6E54`), e.g. ship `0xA8771` `[EXE text]`, with the destination name | `rules-23` §2 row 3 |
| ship off course | only Ballon `0xBA380` / Dampfkarren `0xBA3D0` have a text; ships and carts: **nothing** | `rules-23` §2 row 4 |
| residence seized (debt stage 4) | `0xC119C` `[EXE text]` (residence word, city), `wait_click` | `rules-23` §3.3 (`0x643CC..`) |
| branch seized (stage 5) | `0xC11C8` + optional `0xC11E8` `[EXE text]` + `0xC1218` | `rules-23` §3.4 (`0x64C31`) |
| castle sold | **no message to the seller**, no sound | `rules-23` §4 |
| mission done | congratulation `0xBA7E0` `[EXE text]` in the history screen `0x17930` (`0x1878B`), then the success text and the outro; not a report line | `rules-10` §5, §8 below |
| branch bought / sold | immediate confirmation boxes of the city info window, no report line | `rules-23` §7.1 (`0x4F3C0`) |
| residence ordered | no confirmation after the Bauen box (`rec_new(13, …)`), no report line | `rules-35` (`0x52790`) |
| residence delivered (type 13 at turn start) | **nothing**; the residence appears | `rules-23` §1 / line 127 |
| relief with no siege | `T[2484]` `0xA3D7D` `[EXE text]` (city), human owner only, in the owner's turn-end report | `rules-21` §3.1 (`0x1FAF8`) |

So the port is right to stay silent for returns, deliveries, cargo damage, castle sales and residence delivery; it is missing the lost-transporter line, the off-course lines for Ballon/Dampfkarren, the seizure lines and the relief line.

## 4. Kontor music roll (`0x5D610`) [C]

The roll `rand(100) < 25` → `rand(100) < 50` → `music(0x15 / 0x13, 2)` sits at the top of the **outer** loop of the Kontor function, before the room background and the eight menu items are rebuilt. The inner loop (menu wait, dispatch) returns to the outer loop whenever a room that counts as "left the Kontor" was entered: Schreibstube `0x54570`, Spielstand `0x5C450`, Bezahlung `0x57E70`, Hinterzimmer `0x5C140`, Handel `0x4CC90`, Kirche `0x5CCA0` or the confession box (each increments the "rebuild" counter). The first entry (`0x4F370`), the hint `0x4D1E0` and `0x5DC60` stay in the inner loop and do not re-roll. So: once per Kontor (re)entry, i.e. on turn start and after every return from one of those rooms; not once per turn. Mode 2 only starts a song when none is playing, but the two `rand(100)` are drawn regardless.

## 5. Battle music in ESC mode [C]

- `music(0x1B, 4)` at `0x3F8EC` runs unconditionally right after the deployment `0x3DDA0` and before the first `activate` (`0x389C0`), i.e. before ESC can have been pressed. Mode 4 plays only with the build gate `0x86BC8 > 0xC3500`, but `0x41E50` draws the group's `rand(3)` **first**, for every mode, so the draw always happens.
- `music(0x1C, 5)` at `0x40B71` is **inside the per-unit loop**, not once per round: the loop head `0x40B55` loads the first unit `u = 0xDC83C`, and `0x40B5D` (`u < 100`) is the back-edge target. Order per unit: `music(0x1C, 5)` (always draws `rand(4)`; mode 5 starts a song only when none plays and the gate holds) → ambience: if `0xDC83E < 7` → `+1`, else `rand(100)`; `< 20` → `0xDC83E := 0` and, only when displayed (`0xDC83F = 0`), sample 0x4D mode 3 → `activate(u)` → end checks → next unit. Nothing between `0x40B55` and `0x40BC2` tests the ESC flag `0xDC83F`, so both the `rand(4)` of the song group and the ambience `rand(100)` are drawn in ESC mode as well. Correction to `rules-07` §5.11: add the `rand(4)` of `music(0x1C)` before the ambience step of every unit activation.

## 6. Concert scene RNG order (`0x1AEA0`, counter 0) [C]

Record fields: +4 host x, +5 R (required musicians), +6 Q (required quality index), +7 n (hired count), +8 s (hired quality index), +9 u16 F (fee).
1. `print("$V")`, window torn down, picture, caption box `0x36C90(0, 380, 640, 100, 0, 0xAA8DE, x)`.
2. `music(0x1D, 0)` → `rand(4)` (group {1, 4, 0x12, 0x18}). This is the first draw of the handler.
3. Caption `0x36C90(0, 420, 640, 60, 0, 0xAAD2F)`.
4. Outcome, **no roll**: success if `R < n` and `s < Q`; failure if `n < R` and `Q < s`; otherwise middling.
5. Success: for place `L = 0, 1, 2, 3` (the first four cities, literally the loop index [C]), every person `x` there (`persons(L, 0xFF, 0xFF, 0xFF)`): `favour_increase(owner, x, 2 + rand(10))`; then `payout = (100 + rand(100)) · F / 100`. Failure: same loop with `favour_decrease(owner, x, 2 + rand(10))`; `payout = 0`, no roll. Middling: `payout = (80 + rand(40)) · F / 100`, no favour loop.
6. Outcome box `0x36C90(0, 160, 640, 240, 0, [0xB7008 + 4·k], x, payout)` (k = 0 failure, 1 middling, 2 success), `cash_add(owner, payout)`, history entry (`0x2D750`), screen rebuilt, record deleted.
So the full order is: `rand(4)` music, then per success/failure one `rand(10)` per person at places 0..3 in slot order, then the payout roll (`rand(100)` success, `rand(40)` middling, none on failure).

## 7. Courtship gift reaction text (`0x212B0`, `rules-05` §3) [C]

After the gift menu, when a gift `i < 3` was chosen (`choose` result), before paying: `print([0xB5814] = 0x96657 "$Z$L%1N0 ist %s.$N$N", target (record +9), [0xB5818 + 4·rand(15)])`, then `pay(owner, P[i])`, gift total += `P[i]`. Printed into the turn-end report, right after the menu. The 15 strings (table `0xB5818`): `0x9666E` überrascht, `0x96679` dankbar, `0x96682` unglücklich, `0x9668F` schockiert, `0x9669A` glücklich, `0x966A5` überglücklich, `0x966B3` gerührt, `0x966BB` zu Tränen gerührt, `0x966CD` verblüfft, `0x966D7` verzückt, `0x966E0` sich nicht ganz sicher, ob Ihr das ernst meint, `0x9670F` verliebt, `0x96718` entzückt, `0x96721` entrückt, `0x9672B` entsetzt. Refusal prints `0xBA928` instead and draws nothing.

## 8. Mission accomplished, "Weiterspielen? Nein" [C]

In `0x17930` the mission record is deleted (`rec_del` `0x42550` at `0x18843`) **before** the outro and the question. The question is `0x36AC0(80, 180, 480, 120, 0xBA858 "$N$CWollt Ihr weiterspielen?$Z%m  %m$N", 0x1030, "Ja", 0x1030, "Nein")`. Only answer **1 ("Nein")** calls `0x4D880(0)`; "Ja" or a right click returns to the game.
`0x4D880(save)`: `box(160, 160, 320, 160, 0xBDC98 "$Z$C^ Spiel beenden _$ZSeid Ihr Euch sicher?$Z%m  %m$N", 0x1030, "Ja" 0xBDBB8, 0x1030, "Nein" 0xBDBB0)`. Answer 0 ("Ja"): game save `0x42360` only if `save ≠ 0` (it is 0 here, so **no game save**), config save `0x421A0`, items freed, windows closed, credits roll (`rand(5)` chooses the credits song from `0x87064`, two random text pages `rand(…)` of `0xB6560`), exit to DOS. Anything else ("Nein", right click): return → the caller returns → play continues with the mission record already gone (no further mission checks for this human; other humans keep theirs). So "Nein" at the second box simply resumes the game.

## 9. Auction Esc key (`0x26A50`) [C]

Per tick, after `menu` returns: if a key is waiting, `getch`; `0` → the extended code is read and ignored; **0x1B → `esc := 1`**; any other key is matched against the human bidders' hotkeys (person +0xF5). Esc is not a bidder key, so the human-bid path is skipped and `bidder = 0xFF`. Then, unchanged by Esc: `rand(6)` picks a computer bidder candidate (if his limit allows), and the "price ≥ 4 × start" cap clears him. Then `if bidder = 0xFF or esc`: `rand(100)` is drawn; the hammer branch is taken if `(r < 70 and tick > 1) or esc`; inside it, `if no high bidder or esc`: high bidder := none, `print(0xBADA4 "$16H")`, `print(0xBADC4 "Niemand scheint sich für dieses Objekt zu interessieren.$N")`, `wait_click`, `c := 3` → the lot ends **withdrawn**, even if there was a high bidder; the computer bid picked by `rand(6)` in the same tick is discarded (the bid branch is the `else`). So Esc draws the same `rand(6)` and `rand(100)` as a no-bid tick, forces no hammer count and ends the lot at once without a sale. The function returns 0xFF (unsold); the caller `0x27680` is not told about Esc and proceeds with the next lot. Keys typed during the catalogue or the attendance question are flushed before the first tick (`while kbhit: getch` at the start of each lot), and neither the catalogue nor the attendance box reads keys (no `getch` in `0x27680`; `menu` `0x35B60` only uses Esc inside text-input items, where it clears the input). So Esc works only inside a running lot. `rules-08` §1.4 "ESC withdraws/ends immediately" is correct for the lot, not for the whole auction; `rules-25` §1.3 "or on Esc" is correct, with the addition that Esc never sells.

## 10. SPEC-GAP answers (one line each)

- `rng.rs:30` rand(0): `0x36FF0` = `libc_rand() % n` (`0x6B491`); the state advances first, `n = 0` is a division by zero (fault). Never called with 0 in the original as far as seen.
- `heap.rs:46` full heap: `0x424A0` checks `free (0x85A18) < size[type]` and calls the fatal error `0x2FA00` ("error at … Program terminated", exits). Refusing is as good as the original.
- `save.rs:62`: `0x85A18` (free) and `0x85A1C` (used) are both decremented/incremented by the record size in `0x424A0`; `free = 10000 − used` holds as long as compaction keeps it [UNVERIFIED for compaction `0x42580`].
- `social.rs:268` favour cap: `favour_increase(a, b, x)` (`0x43620`) touches `b.+0x20+a` only if it is `≤ 100`, adds `x` (halved when the religions `+3` differ), clamps to 100. Values 101+ (blackmail) untouched. Same for +rand(3)/+rand(6).
- `trials.rs:97` table `0x84B4C`: 30 dwords, entry i = `1 << i` in order (0x1, 0x2, 0x4 … 0x20000000).
- `auction.rs:14` increments: thresholds `0x84BEC` (19 u32: 100, 200, 500, 1 000, 2 000, 5 000, 10 000, 20 000, 50 000, 100 000, 200 000, 500 000, 1 000 000, 2 000 000, 5 000 000, 1 000 000, 2 000 000, 5 000 000, 10 000 000), increments `0x84C38` (19 u32: 5, 10, 25, 50, 100, 250, 500, 1 000, 2 500, 5 000, 10 000, 25 000, 50 000, 100 000, 250 000, 50 000, 100 000, 250 000, 1 000 000). `i` = number of leading thresholds `≤ price`, capped at 18; increment = `inc[i]`. Because entries 15..17 repeat 12..14, any price ≥ 5 000 000 reaches i = 18 → +1 000 000.
- `ai.rs:10`, `career.rs:417` `person_pick_random`: documented in `rules-24` §2 (`0x43560`): candidates in slot order, one `rand(count)`, 0xFF when empty.
- `world.rs:164` emperor number: no initial value found in the code paths read; `rules-00` §records type 1 (+4, +1 in 1612/1619/1637/1657) [UNVERIFIED].
- `career.rs:353` `old = 0`: the original divides without a check (fault); r = 0 is a safe divergence.
- `battle.rs:918`, `castles.rs:370`, `fight.rs:166`, `events.rs:*`, `missions.rs:46`, `family.rs:42`, `director.rs:*`, `subplots.rs:*`, `trials.rs:175`, `debt.rs:143`, `economy.rs:357`, `turn.rs:1347`, `save.rs:21`, `career.rs:97/259/333/399/457/597`, `ai.rs:538/548`, `heap`/`social.rs:164/253`: not re-read here (each needs its own function pass); still open.

## Corrections to other docs
- `rules-38` §10: owner flags **are** animated (timer handler `0x2D990`, 2 ticks per frame, frames 0..5) on the Handel and Söldner maps; static only where the handler is not installed (player setup).
- `rules-07` §5.11: `music(0x1C, 5)` with its `rand(4)` precedes the ambience step of **every unit activation**, in ESC mode too.
- `rules-08` §1.4 / `rules-25` §1.3: Esc ends the current lot unsold (never a sale), draws the normal `rand(6)` and `rand(100)`, has no effect outside a running lot.
