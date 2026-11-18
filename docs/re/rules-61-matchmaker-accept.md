# Matchmaker accept path (rules-61)

Source: `0x20D30` (Kirche → Hochzeitsglocken), accept/decline block `0x210CA..0x2129B`. Helpers: caption menu `0x36AC0`, `caption` `0x36C90`, `rand(n)` `0x36FF0`, `pay` `0x43860`, `rec_new(type, owner, key, counter)` `0x424A0`, `draw_icon(x, y, id)` `0x2E310`, `flip` `0x30400`, `win_close` `0x30190`. `c` = chosen candidate (`[EBP−0x1C]`), `g` = c's gender byte (person +0x00, `0xE3A7C + 0xF8·c`, 0 male, 1 female per `rules-00` §5), `W` = wedding city (`[EBP−0x18]`), player p. Tags: **[C]** instruction level, **[L]** decompiler or partial trace, **[UNVERIFIED]** inferred. This corrects `rules-15` §7.2.

## 1. Draws, in order [C]

1. Candidate pick: `rand(n)` over the n eligible persons (`0x20F3F`), after the eligibility loop. No draw in the price computation or before the menu.
2. Accept only: `rand(3)` at `0x21135`, after `pay`.
There are **no other `rand` calls** in `0x20D30`. The portrait is not random (§3).

## 2. Accept vs decline [C]

Price question = caption menu `(120, 390, 400, 90)` `0x9DB78` with the price. Result 0 ("das Angebot annehmen") = accept; any other result (item 1 or right click) = decline.

Decline (`0x210F9`): caption `(0, 420, 640, 60, 0)` `0x9DAB2` `[EXE text]`, then `win_close`. **Not shown on accept.**

Accept (`0x21123`), in this order:
1. `pay(p, price)`.
2. `k = 4 + rand(3)`.
3. `rec_new(30, p, 0xFF, k)`: `+9 := c`, `+4..+7 := 0` (gifts), `+8 := k − 2`, `+0xA := W`.
4. Portrait: `draw_icon(20, 390, id)` with `id = 0x18B + c mod 3` if `g = 0`, else `0x186 + c mod 5` (`0x21188..0x211EF`). Then `flip`.
5. caption `(100, 390, 540, 90, 0)` `0x9DAD4` `[EXE text]` with (c).
6. caption `(100, 360, 540, 120, 0)` `0x9DBD0` `[EXE text]` with (`0xB6178[g]` "ihm"/"ihr", `0xB6180[g]` "seine"/"ihre", W).
7. `win_close`.

Every caption on both paths has delay 0, so each waits for a click. The portrait is drawn and flipped **before** the first caption and stays under both.

## 3. Portrait selection [C for the code]

- Test: c's gender byte `g` (person +0x00), the candidate's, not the player's. `g = 0` (male) → icons `0x18B..0x18D` by `c mod 3`; `g ≠ 0` (female) → icons `0x186..0x18A` by `c mod 5`. The index is the candidate's slot number modulo the set size, so the same candidate always gets the same portrait.
- `rules-15` §7.2 had `rand` here and labelled `0x186..0x18A` male and `0x18B..0x18D` female from the rendered sheets. The code assigns the 5-icon set to female candidates and the 3-icon set to male ones. Which set depicts which sex was not re-checked [UNVERIFIED: visual]; the code mapping above is what the port must use.

Answer: draws are `rand(n)` (candidate) then, on accept, `rand(3)` after the fee is paid; the portrait is `c mod 3`/`c mod 5`, no draw. The sex test is c's +0x00 byte, 0 = male → `0x18B + c mod 3`, 1 = female → `0x186 + c mod 5`. `0x9DAB2` appears only on decline. Accept: pay, record, portrait + flip, caption 0x9DAD4 (click), caption 0x9DBD0 (click), close.
