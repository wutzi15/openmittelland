# Open questions, round 6 (rules-45)

Two follow-up questions: the "schwärzt … an" line in the revenge handler, and whether the 1600 year end reaches the next-year election list. Conventions as in `rules-44`: `rand(n)` = `0x36FF0`, `print` = `0x322C0`. Person records are 0xF8 bytes at `0xE3A7C`; `loc(x)` = person +0x0F. Year word = `0xEF4B8`. Tags: **[C]** instruction level, **[L]** decompiler or xref only, **[UNVERIFIED]** inferred. Everything is [C] unless marked. Each section ends with an "Answer:" paragraph.

## 1. Denunciation line 0xBFCFC

### 1.1 Which revenge case prints it

The try loop of `0x5B200` dispatches `c = rand(10)` through the jump table at `0x5B2A8`:

| c | target |
|---|---|
| 0 | `0x5B305` |
| 1 | `0x5B33D` |
| 2 | `0x5B67A` |
| 3, 4 | `0x5B6A0` |
| 5 | `0x5B83E` |
| 6 | `0x5BCB8` |
| 7 | `0x5BEE8` |
| 8 | `0x5BFC2` |
| 9 | `0x5C096` |

The only reference to `0xBFCFC` is at `0x5C0DB`, inside the **c = 9** code. The c = 3/4 trial path (`0x5B6A0..0x5B839`) calls no `print` at all. So the line has nothing to do with type-18 trial records. It is the text of the c = 9 favour act that `rules-09` §4 lists as "a random computer r at loc (not a or v): `favour_decrease(v, r, rand(30))`". That row was missing the print.

### 1.2 c = 9 in order (a = record +1, v = record +4)

1. `r = 0x43560(loc(a), 0xFF, 0xFF, controller 0)`. This collects every person matched by the filter `0x433E0`/`0x434E0`: any computer person whose +0x0F equals `loc(a)`, with church and office unrestricted. If the list is empty it returns 0xFF **without** a draw. Otherwise it draws `rand(n)` (n = list size, also when n = 1) and returns that entry. a itself is in the list, because a is a computer person at `loc(a)`.
2. If r = 0xFF, r = a or r = v, the try fails. Nothing is printed, the done flag stays clear and the loop rolls a new c (up to the 10-try limit, `rules-09` §4). The `rand(n)` from step 1 has already been drawn.
3. Otherwise `print(0xBFCFC, a, v, r)`. The arguments are pushed in that order, so %1N5 = a (the actor), %4N0 = v (the human victim) and %3N5 = r.
4. Then `favour_decrease(v, r, rand(30))` (`0x436F0`, `rules-03`): r's favour toward v drops by `rand(30)`, doubled when r and v differ in religion, floored at 0. The `rand(30)` is drawn **after** the print.
5. Done flag := 1, so the record's revenge for this year ends.

The live capture fits this. Actor "Ratsherr in Bergstedt" and r "Baumeister in Bergstedt" share `loc` = Bergstedt.

RNG per successful c = 9: `rand(10)` (the c roll), `rand(n)`, `rand(30)`. A failed c = 9 with a non-empty list costs `rand(10)` and `rand(n)`. With an empty list it costs only `rand(10)`.

### 1.3 Position and scope

- `$B` (`0xBFCBC`) is printed once per type-50 record at `0x5B26C`, after the counter-0 test (counter 0 → delete `0x42550` and return, so a deleted record prints nothing) and before the try loop.
- The denunciation line follows inside the loop. At most one per record per year, because c = 9 sets the done flag. A c = 1 toll line (`0xBFCC0`) can precede it in the same record, because c = 1 never sets the done flag (`rules-09` §4). No other line can precede it in the same record, because every other printing case sets the done flag.
- `0x5B200` does not test which human the victim is, and nothing ties the line to a viewer. It runs for every type-50 record in the year-end record pass over computer persons (`rules-32` §2 row 15), whatever the victim. It prints into the single year-end window (`rules-32` §2).

### 1.4 Side finding: trial path c = 3/4 (corrects `rules-04b` §3.1 [L])

Read at instruction level, `0x5B6A0..0x5B839`:
- Level test: fails if `loc(v) ≥ 13` and `loc(a) < 13`, or `loc(v) ≥ 10` and `loc(a) < 10`. A failure ends the try without the done flag.
- Spy contract: first type-6 record owned by a with +4 = v. If there is none, one is created: counter 10, +4 = v, +9 = 1, +0x0A = 0, +5 = 0, +0x0B = 0.
- If the contract's evidence (+0x0B) is 0, `rand(100)` is drawn and a result ≥ 10 ends the try **without** the done flag. Evidence > 0 skips the draw.
- If a already owns a type-18 record with +4 = v (either kind), no record is created, but the done flag **is** set.
- Otherwise a type-18 record is created: owner a, counter **`rand(4) + 1`** (1..4; `inc al` at `0x5B7D9`), +4 = v, +5 = 1 if `privilege(a, a, 81, 0)` (`0x5E600`) else 0. Done flag := 1.

So `rules-04b`'s "counter `rand(4)`" is wrong, and so is its quirk "a counter created as 0 wraps to 255". The counter is never 0 at creation. No text is printed on this path.

Answer: (a) The line is printed by revenge case c = 9 (`0x5C096`), not by the c = 3/4 trial path. The trial path prints nothing. It is printed exactly when the drawn computer person r exists and is neither a nor v. (b) %1N5 = actor a, %4N0 = human victim v, %3N5 = r, a random computer person whose office place equals a's (`0x43560`, one `rand(n)` over all matches, a included). r = a or no match makes the try fail silently. The line is followed by `rand(30)` and `favour_decrease(v, r, rand(30))`. (c) It comes after the record's `$B`, at most once per record per year, and only a c = 1 toll line from the same record can come between them. It is printed for every type-50 record of every computer actor, whatever the human victim, into the one shared year-end window.

## 2. Does the 1600 year end reach 0x6A520?

Control flow of `year_end_world` `0x66220`:
- From the entry to `0x66CC7` there is no `ret` and no jump past `0x66CC7`. The only jump to it is the loop exit at `0x66A21`.
- `0x66CC7`: `if year ≤ 1600 (0x640) goto 0x6884D`. In 1600 this skips the vacancy and election cycle `0x66CD6..0x6884B` **and** the type-5 application cleanup at `0x68815..0x6884B` (which deletes type-5 records with counter ≤ 1). In other years that cleanup falls through into `0x6884D`.
- `0x6884D..0x6890E` is straight-line code: `0x1331E`, window close `0x30190`, `rand(100) < 12` → `0x18BD0`, `0x61410` for every computer person, then `0x27680` on the type-3 world record if one exists, and the year-end window opens at `0x6890E`.
- Scan of `0x6884D..0x6A520`: no `ret`, no indirect jump, and the only jump that leaves the range is `0x69F67 jmp 0x6A520`, the exit of the human tax loop. `0x6A520` stores the city values (`0x85E25`, `0x85E4D`, `0x85E75`) and jumps to `0x6A5AD`, where the `[EXE text]` list starts.
- The other year reads in the function (`0x696FB` against 1608, `0x69A01`, `0x69B26`) are inside the window part and do not branch around `0x6A520`.

Answer: [C] Yes. The 1600 year end skips only `0x66CD6..0x6884B` (vacancy filling, elections and the type-5 cleanup), joins the normal path at `0x6884D` and reaches `0x6A520` and the next-year list at `0x6A5AD`, like every other year. This removes the [L] in `rules-44` §3. Side note: in 1600 the type-5 application records with counter ≤ 1 are not deleted at this point.
