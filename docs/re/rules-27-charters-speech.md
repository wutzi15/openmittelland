# Charters, certificate speech, castle condition line (rules-27)

Sources: title award `0x49700`, certificate award `0x494F0`, `speech_say` `0x15570`, `wait_click` `0x36720`, music `0x41E50`, picker helpers `0x42B90`/`0x433E0`/`0x43560`, castle expansion `0x56080`, manoeuvre `0x56950`. All instruction-level [C] unless marked. No live DOSBox run was made for this document (the harness scripts `start.sh`/`shot.sh` were gone from the scratch directory); the layout position in §3 is derived from text-layout rules and marked. Notation as in `rules-15` §0, `rules-16` §1.

Helpers: `rand(n)` = `0x36ff0` = one C `rand()` draw `% n`. It **always consumes one draw, also for n = 1**. `pic(x, y, id)` = `0x2E310`. `win` = `0x2FA80`. `print` = `0x322C0`. `wait_click` = `0x36720`: waits until no mouse button is held, then for a press of **either** button (left and right differ only in the click sample), then for the release. `win_close` = `0x30190`.

## 1. Title award `0x49700(player)`

Gate and title choice are in `rules-03` §2.1. After the new title `n = title + 1` is decided (n = 1..9):

1. `win(0, 0, 640, 480, 0x8100)`, `pic(0, 0, 0xB7)`, flip, `win(80, 20, 400, 440, 0x8810)` (parchment).
2. Title name = `0xB4258[0xA00 + n]` (male, person +0x00 = 0) or `0xB4258[0xA0A + n]` (female). Charter text, by n:

| n | text | args after the format, in the order the text consumes them |
|---|---|---|
| 1 | `0xB6ECC` (council, 0xA914C) | city name `0xB6194[c]`, player, title name, mayor, mayor |
| 2 | `0xB6ED0` (Vogt, 0xA921C) | Vogt, player, title name, Vogt, Vogt |
| 3, 4 | `0xB6ED4` (Regent, 0xA9298) | Regent, player, title name, Regent, Regent |
| 5..9 | `0xB6ED8` (Emperor, 0xA9315) | emperor name, player, title name, emperor name |

   Emperor name = `0xB707C[rec + 4]`, `rec` = type-1 world record `0x426B0(1, 0xC0, 0xFF)`.
3. Right after the print: `pic(26, 360, 0xCB)` (seal), on every page. Not an item.
4. Person title byte (+0x0D) := n. Then `music(0x1E, 0)`: fanfare, which draws **`rand(2)`** from the game RNG (`0x41F17`, song 7 or 0x0E; unconditional, before the sound-option test). Then `wait_click`.
5. Extra pages, selected **only by the new title**:

| n | page | args |
|---|---|---|
| 3 (Ritter) | county offices `0xBD1A8`, `$P` repaints the same window | player (`%1N1`); `%o` = county `0x868D4[loc]`, `loc` = location (+1) of a **random** own branch (`0x42B90(0, 0xFF, player)`, an independent second draw) |
| 6 (Baron) | principality offices `0xBD298`, same window | player only |
| others | none | |

   Each extra page: print, seal `pic(26, 360, 0xCB)`, `wait_click`. No music, no speech. Order: charter, seal, music, click; extra page, seal, click.
6. `win_close` twice. There is no speech anywhere in the title award.

Authority lookups (person table of 192 slots, a slot with +0x00 = 0xFF is empty and skipped):
- `pick_first(place, church, office)` = `0x433E0`: lowest person index with +0x0F = place, +0x10 low nibble = church flag, +0x11 = office index (0xFF in an argument = don't care). `pick_random` = `0x43560`: same filter, then one `rand(count)` over all matches. `branch_random` = `0x42B90`: random own branch, one `rand(count)` if there is one.
- n = 1: `c` = location of a **random** own branch (correction to `rules-03` §2.1 "first branch city"). Mayor = `pick_first(c, 0, 6)`; fallback `pick_first(c, 0, any)`; fallback `pick_random(any)`. If still none, **no text and no seal are drawn**; the title is still set and music and click still run.
- n = 2: `c` as above, `t = c + (c == 0)`, county `k = (t − 1) / 3 + 10` (c 0..3 → 10, 4..6 → 11, 7..9 → 12). Vogt = `pick_first(k, 0, 7)`; fallbacks `pick_first(k, 0, any)`, `pick_random(any)`; none → no text.
- n = 3, 4: Regent = `pick_first(13, 0, 7)`; fallbacks `pick_first(13, 0, any)`, `pick_random(any)`; none → no text. No branch draw.
- n >= 5: no lookup, no draw.
- RNG order of a title award: `rand(100)` gate (always), `rand(100) < 80` (only if `cap > title`), then n = 1/2: branch draw and, only on the last fallback, `pick_random`; n = 3/4: `pick_random` only on the last fallback; `rand(2)` music; n = 3: one more branch draw.
- A player with no branch at n = 1/2/3 dereferences a null pointer [UNVERIFIED outcome; the residence level comes from branches, so it does not occur in normal play].

## 2. Certificate award `0x494F0(player)` (human turn start)

Gate: person +0x06 != 0. Effects first: +0x06 := 0, +0x05 += 1 (new level `lv` = 1 or 2).

1. Same two windows as §1.1. Text `T[0xB6ED8 + 4·lv]`: lv 1 = `0xA93EF` (Handelszertifikat II), lv 2 = `0xA94B9` (III). Args: player (`%3N1` / `%1N3`), place name `0xB6194[place]` (`%s`). The **whole certificate is printed at once as text**; nothing is split on screen. Seal `pic(26, 360, 0xCB)`.
2. `music(0x0E, 1)`: fixed song, no RNG draw.
3. `L = (lv == 2) ? 1 : 0`. Flip. Then three `speech_say(0, 2, 2, 4, n)` calls, each executed only if the **previous one returned non-zero**:
   - line 1: `n = 900 + 12·L` (`0224900` / `0224912`);
   - line 2: `n = 901 + 12·L` (`0224901` / `0224913`);
   - line 3: `n = 902 + place + 2·(place >= 10)`: `0224902..911` for places 0..9, `0224914..917` for 10..13. **No `12·L` term in line 3** (corrects the task statement and `rules-14` §3). The `+2` skips files 912/913, which are certificate III lines 1 and 2.
   - `place` = person byte **+0x0F** (office level location: city 0..9, county 10..12, principality 13), the same byte that selects the printed name `0xB6194[place]`.
4. If line 3 returned non-zero: `wait_click`. Otherwise no click wait.
5. `win_close` twice.

`speech_say(c0, c1, c2, c3, n)` `0x15570` [C]: loads `<c0 c1 c2 c3><n>.WAV` (`rules-16` §1), starts it, loops until **a mouse button is down or the sound engine reports idle**, stops and unloads the sample, returns **1 if no button is down at that moment (played to the end), 0 if a button was pressed**. Consequences:
- Lines play back to back with no pause. Speech is audio only; the caption is the full certificate already on screen.
- A press during line 1 skips lines 2 and 3 and the click wait; a press during line 2 skips line 3 and the wait; a press during line 3 skips the wait. A button already held at entry makes that call return 0 at once. Any button, press (not release) ends the line.
- No sound: the idle test (`0x12C71`) is true at once, so each call returns 1 immediately and only the final `wait_click` waits. The speech-off option path is as in `rules-16` §1 [not re-traced here].
- Sample lengths (44 kHz files): 900 = 2.32 s, 901 = 8.82 s, 912 = 2.19 s, 913 = 7.63 s, 902..911 and 914..917 = 1.8 to 2.3 s. So line 1 is a short lead-in, line 2 the long remainder, line 3 a ~2 s place line. **Where the text splits in the recordings is [UNVERIFIED]** (nobody listened; no transcript exists).

## 3. Castle/camp condition window `0x56080`: "Waren vor Ort"

- Text `0xBF734` `Waren vor Ort: %d$N`. One `%d` (thousands dots as in all `%d`): the **sum of the 15 u16 stock words at property +0x05 + 2i, i = 0..14, all goods**, in units, unweighted. No per-good list.
- Condition: sum > 0 **or** the property is a camp (type byte +0 = 1; castle = 2). A camp always shows "Waren vor Ort: 0". A castle with sum 0 shows `0xBF748` `Baujahr:%l$N` (u16 at static record R+0x12) instead.
- Position: a second `print` right after the title print, so it continues the title's text flow in `win(100, 20, 440, 440, 0x8810)`. By the layout rule (`formats.md` §9: margin 12, `$Z` = half line, line 24): title starts after a `$Z` (y 24), title line 24 high, trailing `$Z` +12, so the line sits at window y ≈ 60..84 (screen y 80..104), above the picture at window y 92. Horizontal alignment (centred as inherited from the title's `$C`, or left) is [UNVERIFIED]; no screenshot taken.
- Item 0 (live text at window (0, 354), 440×24) is separate: `Sicherheit`/`Kasematten` (castle) or `Tarnung`/`Strohmatten` (camp), values +3 and +4.

## 4. Manoeuvre picture RNG `0x56950`

Yes, game RNG. Exactly one `rand(3)` per **opening** of the manoeuvre window, right after `win_open` and before the title print (`0x56984`). Picture id `0x199 + r` is kept in a local and reused on every pass of that opening. Not redrawn per pass or per click. Per opening the draw order is `rand(3)` once, then per manoeuvre click `rand(4)`, `rand(10)`, `rand(m)` (`rules-26` §5.3). A deterministic sim must consume the `rand(3)` at window open even when the picture is not shown.

## Corrections to earlier docs

- `rules-19` §3 title award: extra pages are **n = 3 (0xBD1A8) and n = 6 (0xBD298)**, same window, seal at (26, 360), click after each, no music. Charter music `0x1E` comes after the charter's seal, before its click. The awarding person for n = 1/2 comes from a **random** own branch.
- `rules-14` §3 and `rules-16` row `0224900/901`: three lines; the third is `902 + place + 2·(place >= 10)` without `12·L`; click-away by `speech_say` return value (§2).
- `rules-14` music table: `0x1E` draws `rand(2)` from the game RNG on every call, including the title award. `0x0E` (certificate) draws nothing.
- `rules-26` §5.1: "Waren vor Ort" also shows for camps with 0 stock.
