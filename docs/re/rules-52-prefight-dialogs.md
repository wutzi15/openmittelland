# Pre-fight dialogs: ransom, bribes, helper sides (rules-52)

Source: `battle_main` `0x3EE60`, listing `0x3FB77..0x40AD2`. Helpers: `print` `0x322C0`, `menu` `0x35B60` (returns the absolute item index; a right click returns item count + 1), item pop `0x34AC0`, `win_open` `0x2FA80`, `win_close` `0x30190`, `caption` `0x36C90`, `pay` `0x43860(p, amount)`, `cash_add` `0x43790(p, amount)`, `favour_add` `0x43620(from, to, d)`, `net_worth` `0x44640`, `strength(side)` `0x38730`, history `0x2D750`. `A` = attacker `0xDB7F4`, `D` = defender `0xDB7F5`, `value` = battle value (`rules-07` §5.4, local `[EBP−0x70]`), `n` = helpers (0..2), `H[h]` = helper owner `0xDB7F6 + h`, helper record `0xD6A4C[h]`. Tags as in `rules-50`. Everything is [C] unless marked. This refines `rules-07` §5.5.

## 0. Where it happens

1. Codes 4/5: after the intro captions, line 4, the **attacker deployment** (`0x3FB77..0x3FCD5`), a draw-only `activate` and a present of the field (`0x3FCDA..0x3FD22`). So the attackers are visible.
2. Raids (code < 4) only: an extra caption `(40, 180, 560, 160, 0)`. When the good `rec+7` ≠ 14 it reads `0xBBA04` `[EXE text]`. Otherwise it reads `0xBBA48` `… haben …`. Args: (D, good name `0xB6618[g]`, value).
3. **One framed message window** `win_open(120, 120, 400, 240, 0x8810)` (`0x3FDE6`). This is a parchment frame (`rules-16b`), not a caption. Every text below is printed into this window. `$W` waits for a click, and `$P` starts a new page in the same window. The window is closed at `0x40A44`.
4. If the battle goes on, the caption `(0, 180, 640, 120, 0)` `0xB4219` `[EXE text]` is shown, then round 0 starts (`rules-44` §2).

## 1. Defender's offer

**Human D** (`0x3FE2E`). `offer := 0`. Text 2881 `0xA9A89`:

`[EXE text]`

with (D, A, style `0x25`, &offer).

- Items: 0 = "… Taler." line, 1 = the 5-digit u32 spinner (0..99999, preset 0, `rules-13` §1.3), 2 = "Niemals, ich gewinne!".
- The `menu` loop repeats while the result is item 1. On item 1 the menu's out-value `[EBP−0x90]` is added to `offer`: it is the clamped digit step `±10^k` of the click; `menu` never writes the spinner's variable itself (rules-71 §6, [C]).
- Result 0 keeps `offer`. Any other result (item 2 or a right click) sets `offer := 0`.
- No cap against D's cash is applied here.

**Computer D** (`0x3FEBF`). `s = strength(1)`:
- `s ≥ 3`: `offer = (80 + rand(40)) · value · (s − 1) / 400 + 1`. Text 2882 `0xA9B11` `[EXE text]` with (D, A, offer). `%d` prints with dot groups, e.g. "8.083". For s = 3 this is the `rules-07` formula. For s = 4 the factor is 3.
- `s < 2`: `offer = 0`. Text 2883 `0xA9B64` `[EXE text]` with (D, `0xB6F68[gender(D)]` = "er" / "sie").
- `s = 2`: `offer = 0`. Text 2886 `0xA9BA5` `[EXE text]` with (D).

All three end with `$W` (click).

## 2. Attacker's decision

**Human A** (`0x3FFB2`). Text 3069 `0xAD431`:

`[EXE text]`

with (A). This is a new page, then `menu`.
- **Item 0 "Gewiss, he he!" → fight**, and `offer := 0`, so nothing is paid.
- Any other result (item 1 "Gnade", or a right click) → **withdraw**: history entry `0x2D750(A, 8, field, 0)` and the offer is paid.
- No outcome line follows (3070..3072 are not printed for a human A).

**Computer A** (`0x40031`). These are the outcome lines, printed into the window, each ending with `$W`:
- `offer = 0` → 3072 `0xAD549` `[EXE text]` (A), fight.
- otherwise `r = rand(100)`. If `value · r > offer · 200` → 3071 `0xAD4F6` `[EXE text]` (A): money taken, then fight.
- else → 3070 `0xAD497` `$P$N$N$N$C%1N0 entschließt sich, von einem Angriff abzusehen; die %s ziehen sich zurück.$N$W` (A, `0xB601C[k]` Söldner/Räuber, with k = 1 unless A's home kind is 2): withdraw.

Then always `pay(D, offer)` and `cash_add(A, offer)` (`0x400E6`).

## 3. Helpers (only if fighting and n > 0)

Text 2887 `0xA9BE3` `[EXE text]`. Then for each briber side i = 0 (A), 1 (D), with briber `b = 0xDB7F4[i]`:

**Human briber** (`0x401E1`):
1. Header 2888 `0xA9C26` `[EXE text]` (b).
2. Per helper h:
   - if `H[h] = b`: `0xBBA90` `%M%M$E`, two invisible placeholder items;
   - else 2889 `0xA9C6D` `%a%m Taler für %2N0 %s$|` with (style `0x25`, &amt[h] preset 0, `H[h]`, troop word: helper record type 53 → "Söldner", otherwise "Räuber").
3. Then 2890 `0xA9C87` `%aGenug bestochen!$|`.

`menu` loop: `j = result`, `h = j / 2`:
- `h < n` and j even (the "… Taler für …" line): commit.
  - `paid[i][h] += amt[h]`, `pay(b, amt[h])`, `cash_add(H[h], amt[h])`.
  - `favour_add(b, H[h], 2 · min(100, amt[h] · 400 / max(100, net_worth(H[h]))))`. This is doubled for a human, and the `ADD AL,AL` is byte arithmetic, so it wraps above 127.
  - `amt[h] := 0`.
- j odd (spinner): `amt[h] += out` [UNVERIFIED as in §1].
- The loop ends when `h ≥ n` ("Genug bestochen!" or a right click).

**Computer briber** (`0x404A9`): per helper, `x = value / (4n) + rand(100)` (always drawn). `x := 0` if `strength(i) ≤ 1`, or `x < 500`, or `0xDB7F4[h] = b` or `0xDB7F4[h]` is human (`rules-07` §5.5 index quirk). Text into the window:
- `H[h] = b`: `0xBBA98` `[EXE text]` (b);
- else 2891 `0xA9C9C` `$P$N$N$N$C%1N1 besticht %2N1 %s, damit sie zur Hülfe eilen.$N$W` (x ≠ 0), or 2892 `0xA9CDC` `[EXE text]` (x = 0), with (b, `H[h]`, troop word).

Then in every case: `paid[i][h] += x`, `pay`, `cash_add`, and `favour_add(b, H[h], min(100, x·400 / max(100, net_worth)))`. These run with x = 0 too.

**Side choice** per helper h (`0x406D8`), owner `o = H[h]`:
- **Human o**: 3142 `0xAF2CC` `[EXE text]` with (o, A, D). Result 0 = A's side, 1 = D's side, anything else (item 2 or a right click) = stays out. No outcome text follows.
- **Computer o**:
  - if `o = A` or `o = D`: that side;
  - else `fa = favour(o→A)`, `fd = favour(o→D)` (person `+0x20 + target`). A's side if `fa > fd && fa > 50 && paid[0][h] > value/7`. Else D's side if `fd > fa && fd > 45 && paid[1][h] > value/7`. Else out.
  - Text: the side's participant is o → `0xBBAE0` `[EXE text]` (o). Other side → 3143 `0xAF323` `[EXE text]` (o, side person). Out → 3144 `0xAF36D` `[EXE text]` (o).
- The choice is stored at `0xD9796 + h`. Joining helpers deploy their four record troop counts as participant h + 2 on the chosen side, with home training, no leader (`0x408FC..0x40A35`).

## 4. Banner gold

The gold item text is `$t$L%i Taler$N` (`formats.md` §9.1). `%i` = `%d` with dot groups plus a leading `-` when negative (`rules-12` §1) [L]. So 50 000 000 shows as "50.000.000 Taler", which matches the capture.

Answer: all pre-fight dialogs are printed into one framed 400×240 window at (120,120), style 0x8810, over the field after the attackers are deployed. They are paged with `$P` and click-waited with `$W`. Answer encoding:
- 3069: item 0 = fight; anything else = withdraw and take the offer.
- 2881: item 0 = pay the spinner amount; item 2 or a right click = 0.
- 2889: per helper, even item = pay that row, odd item = spinner, "Genug" or a right click ends.
- 3142: 0 = attacker, 1 = defender, other = out.

## 5. "Wohlan" caption position (follow-up)

- **Call [C]** (`0x40AB0..0x40ACD`): `caption 0x36C90(x 0, y 180, w 640, h 120, delay 0, [0xB7708] = 0xB4219)`. It runs after the message window is closed (`0x40A44`), a draw-only `activate` and a full-field present (`0x40A61..0x40AA2`). The text is `[EXE text]`. Nothing is prepended, and the text starts directly with `$C`.
- **Caption path [C]** (`0x36D4A..0x36DE1`), in order:
  1. `win_open(x, y, w, h, 0x8800)` sets cursor y 12, margins 12, line height 24, and `0xD501C` 0. Because of the 0x8000 bit it gives the window its own buffer with text origin `0xD5014`/`0xD5016` = 0, 0.
  2. Compositor `0x2FDF0`.
  3. `print("$g")` (`0xBB668`, flag only, no line).
  4. **`0xD501A := 2`**.
  5. `print(fmt, …)`.
  
  No `$Z` or half line is added by the caption.
- **Glyph y [C]**: the line painter `0x31610` draws every glyph at `y = (0xD501C + 0xD501A) mod 0xD501E + 0xD5016` (`0x31639..0x31668`), which is 2 here. There is no baseline or centring term. The blit adds no offset either (`0x13DC5`). The ICONCS glyph `W` has ink from its first row (rows 0..17 of 24; `T` likewise). The gold glyph is drawn at y and its dark outline copies at y ± 1 (`rules-16b`).
- So this path predicts glyph tops at screen **y 181** (outline) / **182** (gold), and the second line 36 px lower, the same as the port. The code does not explain the capture's ~9 px offset [UNVERIFIED: whether the capture's window or crop is offset; checking the intro captions at (40, 180), which use the same function and the same cursor rule, would separate a capture offset from a code difference].
