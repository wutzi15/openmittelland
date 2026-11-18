# Gold-display click (Bestechung) and Schreibstube Bewerbung (rules-64)

Source: `0x4F370(p)` (thunk), `0x45C30(p, mode)` (bribe action `0x46A79..0x46C7A`, exit `0x47BC0`), `office_apply` `0x47F90` (Schreibstube case at `0x5C6FC..0x5C742`). Helpers as in `rules-04`: `msgbox` `0x368E0`, `win_open` `0x2FA80`, `print` `0x322C0`, `menu` `0x35B60`, `pay` `0x43860`, `cash_add` `0x43790`, `favour_add` `0x43620`, `net_worth` `0x44640`, `rec_find/next/new` `0x426B0`/`0x427B0`/`0x424A0`, `office_eligibility` `0x47C20` (`rules-03` §3.2), `human_rival_of` `0x44F10`, OK item `0x348D0`, items free `0x34AC0`. Tags: **[C]** instruction level, **[L]** decompiler or taken from another doc, **[UNVERIFIED]** inferred. Everything is [C] unless marked.

## 1. Gold display → `0x4F370(p)`

`0x4F370` is a 42-byte thunk: **`0x45C30(p, 1)`**, mode 1 = "Bestechung". The user's memory is right: the gold display opens the bribery flow. Callers: Kontor gold item (`0x5D978`), trade map (`0x4D083`), branch screen, Schreibstube, Kirche, wages, Söldner, Hinterzimmer, Kirchgang banners (all nine `xr` hits), each "then stay" on its own screen.

### 1.1 Flow (already specified elsewhere, cited here)

1. **Plot map** (`rules-29`): window `(20, 40, 600, 400)` style `0x8910`, map ICON25 at (60, 20), title `0xBC544` `$C^ %s in %o _` / `0xBC558` `$C^ %s... _` with verb `0xB673C[1]` = "Bestechung". Hit test and highlights as `rules-29` §2/§4. Right click or no-place leaves the screen; the function then returns to the caller, which redraws its own screen.
2. **Person list** (`rules-31`): `0xBC568` `$H$C^ Bestechung in %o _`, rows per office of the chosen place, level toggle "Zur kirchlichen Ebene..." / "Zur weltlichen Ebene...", OK item; a click on an occupied row picks T. `human_rival_of(player)` is evaluated for the row highlight [L: `rules-03` §4.1 says it draws `rand(10)` only when the human has no office and no branch].
3. **Action** (mode 1 → `0x46A79`), `rules-04` §3.1 refined at instruction level:
   - `cash(P) ≤ 0` → `msgbox(160, 160, 320, 160, 0xBC5CC)` `[EXE text]` (T). Nothing else.
   - else `win_open(160, 120, 320, 240, 0x8810)`; `print(0xBC618, T, 0x27, &X := 0, 0, cash(P))` = `[EXE text]`: one `%a` line holding a 7-digit spinner (`%n` style 0x27, range 0..cash); `flip`; `menu` loop: item 1 (the spinner) adds the menu's out-value to X and repeats; item 0 (the bullet) ends the loop with "pay"; anything else (right click) ends it with "cancel".
   - **Pay** only if result = 0 and `X > 0`: turn flag `0x85CBA |= 4`; `pay(P, X)`; `g = X·400 / max(100, net_worth(T))`, capped at 100; if `gender(P) ≠ gender(T)`: `g = g·11/10`; `favour_add(P, T, g)` (religion halving and the >100 guard inside, `rules-04` §3.1); `cash_add(T, X)`; the per-call action counter `[EBP−0x1C]` += 1; if P's type-0 mission record has kind 6, its `+9 += X`.
   - Cancel or X = 0: no message, no payment, no flag.
   - Items freed, `win_close`, back to `0x47BC0`: the plot map is redrawn and the loop continues (another bribe is possible in the same visit).
4. **No RNG** in the bribe action itself. Draws on the way in are only those of the map/list (`rules-29` §4 highlights draw none; the `human_rival_of` case above).

Answer: the gold display is not a finance screen; it is `0x45C30(p, 1)`: plot map → person list → the "^ Bestechen _" spinner window `(160, 120, 320, 240)`, pay X to T for favour `min(100, X·400/max(100, NW(T)))` (+10 % across genders), with the empty-purse box when cash ≤ 0. It returns to the caller's screen when the map is left.

## 2. Schreibstube → Bewerbung (`0x47F90(p)`)

### 2.1 Gate in the Schreibstube (`0x5C6FC`)
`p+0x0D` (citizenship byte) `< 1` → `msgbox(160, 160, 320, 160, 0xBFDC0)` `[EXE text]`. Otherwise `0x47F90(p)`.

### 2.2 Building the list (`0x47FB6..0x48180`)
1. Rival table: for every human h (slot used, `+0x10 & 0xF0`), `R[h] = human_rival_of(h)`.
2. Offices: scan place 13 down to 0; per place idx from `9 + 3·(place > 9) − 2·(place = 13)` (city 9, county 12, Mittelland 10) down to 0. An office is listed when **`office_eligibility(p, place, idx) = 2`** (eligible, vacant and a promotion; `rules-03` §3.2) and fewer than 6 are listed. After listing idx, the rest of its group is skipped (idx 2, 1 → Ratsherr group; city idx 8 → the second Priester), so a group shows its highest vacant seat only. Up to **6** entries.
3. For each entry the player's own type-5 record with `+4 = place`, `+5 = idx` is looked up (`rec_find(5, p, 0xFF)` + `rec_next`) and remembered as `A[i]` (0 = none).
4. No entry → `msgbox(160, 180, 320, 120, 0xBCFF4)` `[EXE text]`, return.

### 2.3 Main window (`0x481B5`)
`win_open(80, 80, 480, 320, 0x8810)`. Loop:
- `print(0xBD048)` `[EXE text]` (the title is "Bewerben", not "Bewerbungen").
- One row per entry i: `print(0xBD09C, name, place, 0x1030, s)` = `%a%s in %o$E$R%m$|`: bullet item (icon 0x73), office name from the name table `0xB4258[base + idx + 10·(place > 9) + 13·(place > 12)]` with `base = 0x48` for a male player and `0x57F` for a female one (gendered office names), "in <place>", and a right-aligned `%m` text item (style `0x1030`) whose text `s` is **"Ja"** if `A[i] ≠ 0` else **"Nein"** (bytes at `0x41E40`: `Nein\0Ja\0`). There is no "(beworben)" marker; the Ja/Nein item is the state.
- `flip`, then `menu` until a result r with `r/2 ≥ count` (right click, or the OK-less window's "nothing") ends the screen. `r` even = the bullet of row r/2, `r` odd = its Ja/Nein item.
  - **Odd (toggle)**: `A = 0` → `rec_new(5, p, 0xFF, counter 2)`, `+4 := place`, `+5 := idx`, `A := record` (apply). `A ≠ 0` → record byte 0 `|= 0x80` (deleted), `A := 0` (withdraw). Then `0x30F10` (window clear [L]), items freed, and the whole list is reprinted with the new Ja/Nein. **No cost, no cash check, no RNG.**
  - **Even (applicant list)**: `win_open(60, 140, 520, 200, 0x8810)`; `print(0xBD0B0)` `[EXE text]`; walk all type-5 records (`rec_find(5, 0xFF, 0xFF)` + next, heap order) with owner ≠ p, `+5 = idx`, `+4 = place`: the first four print `0xBD0DC` `$($a%1N0, %2N0 $hGegenspieler$h$)$|` (male) / `0xBD100` `…$hGegenspielerin$h…` (female) with (applicant, the human h whose `R[h]` is the applicant) when the applicant is some human's current rival, else `0xBD128` `$($a%1N0, %1N4$)$|` (applicant, applicant's title); the fifth prints `0xBD13C` `$a...und andere$|`; none → `0xBD150` `$aNiemand$|`. Then the OK item `0x348D0`, `flip`, one `menu` (any click, bullet or OK, just closes), items freed, `win_close`. The bullets in this window do nothing; applying is only the Ja/Nein toggle of the main list.
- After an applicant window the main `menu` resumes without reprint; after a toggle the list is rebuilt; a result with `r/2 ≥ count` frees the items and closes the main window.

### 2.4 What the sim must do
- Apply: type-5 record (owner p, key 0xFF, counter 2, `+4` place 0..13, `+5` office index within level). Withdraw: delete it. Both free. Lifetime and use: `rules-03` §4.7 (counter 2 → 1 at p's turn end, consumed at year end).
- Listing requires `office_eligibility = 2` at the moment the screen opens (vacancy included), at most 6 entries, highest free seat of a group.
- There is no "Zurück" item in this screen; leaving is a right click (or any click outside the rows). The user's "(beworben)" and "Zurück" are not in this EXE's strings; the applicant window's bullets are inert.

Answer: Bewerbung = window `(80, 80, 480, 320)` titled "^ Bewerben _" with up to six `[EXE text]` rows for vacant promotion offices; the Ja/Nein item toggles the type-5 application record (no cost); the row bullet opens `(60, 140, 520, 200)` `[EXE text]` listing up to four rival applicants (title, or "<human>s Gegenspieler(in)") plus "...und andere" / "Niemand" with an OK item. Right click leaves.
