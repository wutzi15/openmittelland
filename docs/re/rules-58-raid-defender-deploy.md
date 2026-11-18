# Raid defender deployment and sprite rest position (rules-58)

Source: `battle_main` `0x3EE60`, raid defender block `0x3F552..0x3F786`; `deploy(field, type, n, participant, side, training)` `0x3DDA0` (register args EAX, EDX, EBX, ECX, stack side then training; returns the number placed in AL); object allocator `0x3D7C0` (rest position `0x3DA54..0x3DAF8`); object draw in `activate` `0x389C0` (`0x3BEE5..0x3C050`), blitter `0x13AE0`. `S` = shipment record (type 9: +4 kind, +5 u16 qty, +7 good, +8 transporter count, +9 transporter type). Escort count array `E[k]` = `0xDB7F8 + k`, markers count `0xDB801`. Tags: **[C]** instruction level, **[L]** decompiler or partial trace, **[UNVERIFIED]** inferred. Everything is [C] unless marked.

## 1. Defender deployment, codes 0..3

Entered when `code < 4` (`0x3F552`). `E[0..3]` were zeroed at `0x3F0DE..0x3F10E` before this. Every call below uses **participant 1, side 1** (ECX = 1, first stack arg 1), so every defender object takes colour `0xDB7FC[1]` (the defender's colour, `rules-51` §1).

| # | addr | type | n | training | condition |
|---|---|---|---|---|---|
| 1 | `0x3F5A2` | 20 goods markers | `(S.qty + 99) / 100` (signed div) | 0 | always; result stored in `0xDB801` |
| 2a | `0x3F5E7` | 10 carters | `S[+8]` | 50 | code < 3 |
| 2b | `0x3F61B` | 21 wagons | `S[+8]` | 0 | code < 3 |
| 2c | `0x3F656` | 11 sailors | `(5 · S[+8]) & 0xFF` (8-bit `MUL`) | 50 | code = 3 |
| 3 | `0x3F724` | k = 0, 1, 2, 3 in order | `E[k]` | 50 | always called; `E[k]` is 0 unless escorted (§1.1), and `deploy` returns at once for n = 0 (`0x3DDB7`) |
| 4 | `0x3F757` | **4 leader (general)** | **1** | **87** (`0x57`) | **always**, escorted or not |

Then `0xD9799 := 0` and `value := S.qty · (S.good + 8)` (`0x3F763..0x3F783`).

### 1.1 Escort counts (`0x3F663..0x3F6CF`)

Only if `S[+4] & 1`: for k = 0..3, skipping k = 2 unless code = 3: `E[k] = (S.qty + 249) / 250`. Unescorted shipments leave `E[k] = 0`, so step 3 places nothing.

### 1.2 What the port is missing

- The extra man of the capture is step 4: one **type-4 leader with training 87**, placed for every raid, independent of the escort bit. Nothing else is added: no driver per wagon, no +1 in any count. It is the same leader call the castle/camp defender gets (`rules-50` §1.3 step 5), and the attacker gets its own leader after line 4.
- He is drawn with the defender's colour remap like the carters; the grey legs are pixels of the type-4 frames outside the remapped range p = 1..10 (`rules-51` §1) [UNVERIFIED: frame contents are in the icon bank, not the EXE].
- His anchor is the SZENARIO cell with code `8 + class 4 + 6` = 18 on field 1 [C for the code, UNVERIFIED for the cell position; the capture's top-right position is where that anchor lies].
- Marker count: no `min 1`. `qty = 0` gives 0 markers and `deploy` returns 0; `0xDB801` is 0.
- Sailors: `5 · S[+8]` is computed in AL, so 52 or more ships wrap (irrelevant in practice).
- Call order for the port: markers, carters, wagons (or sailors), escort types 0..3 (cannoneers 2 only on ships), leader.

Answer: codes 0..3 deploy markers `ceil(qty/100)`, carters and wagons `S[+8]` (ships: sailors `5·S[+8]`), escort types with `ceil(qty/250)` only when `S[+4]` is odd, and **always one type-4 leader, training 87, participant 1, side 1** at `0x3F757`. That leader is the extra man.

## 2. Rest position and draw y of types 20 and 21

Allocator `0x3D7C0`, after the cell is chosen (`col` = `[EBP−0x24]`, `row` = `[EBP−0x10]`):
- type 20 (`0x3DA54`): `x = col·24 + 1`, `y = row·12 − 28`, HP 100;
- type 21 (`0x3DA87`): `x = col·24 − 9`, `y = row·12 − 28`, HP 100;
- all others (`0x3DAB9`): `x = col·24 − XOFF[type][action 0]` (`0x853E4`), `y = row·12 − 28`.

Draw (`0x3BEE5..0x3C050`): the object's stored `(x, y)` is passed unchanged to `0x13AE0(frame buffer 0xD500E, x, y, sprite, 0x80 + 5·colour)`; only a cannoneer in action 5 gets (x − 6, y − 6) (`rules-51` §2.2). `0x13AE0` places the sprite's first row at `y` (top-left anchor, run-length rows with no height header: `rules-33` §blit, `rules-38` §10). The field buffer is then presented at window offset (40, 40) (`0x3F909..0x3F939`).

So in field coordinates the **top** of a type-20 or type-21 sprite is `row·12 − 28`, with no dependence on the sprite height. The port's rule `top = row·12 + 12 − h` agrees only for h = 40. For a marker or wagon frame of height h the bottom is `row·12 − 28 + h`.

Answer: y is top-anchored at `row·12 − 28` for every object type, markers and wagons included; x is `col·24 + 1` (markers), `col·24 − 9` (wagons), `col·24 − XOFF` (men). Replace the bottom-anchored rule with the fixed top.
