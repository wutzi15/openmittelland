# Cash clamp in `cash_add` and `pay` (rules-60)

Source: `cash_add(p, x)` `0x43790`, `pay(p, x)` `0x43860`, cash field = person `+0x18` (`0xE3A94 + 0xF8·p`, s32). Tags: **[C]** instruction level, **[L]** decompiler or partial trace, **[UNVERIFIED]** inferred.

## 1. The two writers [C]

- `cash_add` (`0x437AE..0x437E0`): `cash += x`; then **only** `if cash > 9 999 999: cash = 9 999 999`. No lower bound.
- `pay` (`0x4387E..0x438B0`): `cash −= x`; then **only** `if cash < −9 999 999: cash = −9 999 999`. No upper bound.
- Both then repaint the cash item of the open banner (`0xEF4B6` ≠ 0 and `0xEF4C3` = p): copy cash to `0x86C88`, pick the colour `0x86C84` (`0x86C8C` for ≥ 0 in `cash_add`, `0x86C90` for < 0 in `pay`), `0x34BF0(item)`.
- Neither function clamps on entry, so a value already outside the band stays there until the other bound is crossed. A cash of 50 000 000 minus a 771 bill gives 49 999 229 (`pay` does not look at the top), and the next `cash_add` (toll income) snaps it to 9 999 999. A negative balance below −9 999 999 likewise survives `cash_add` calls that do not bring it back above that line.

So `rules-00` §5's "clamped to ±9 999 999" means: the band is enforced one side at a time, by the operation that moves toward that side.

## 2. Other writers

A scan of the image for the absolute field address `0xE3A94` (26 instructions) finds writes only in `cash_add` and `pay`. The other 22 are reads: auction bid checks (`0x26A50`, `0x27680`), net worth `0x44640` (`0x4465C`), the Hinterzimmer spy screen (`0x45C30`), the banner painter `0x49D60`, the city-info branch opening check (`0x4F3C0`, `0x4F8B9`), and the turn-end ruin test in `0x61A70` (`0x643D7..0x64437`, `cash < 0` and `−net_worth/20 > cash`). The turn-end bill, the battle premium (`rules-59` §6), ransom and bribes (`rules-52`) all go through `pay`/`cash_add`.

Writes through a person pointer (`[reg + 0x18]`), as in savegame load and the new-game initialisation, are not covered by this scan [L: none of the cash-touching rules so far (`rules-02` §9 top-up, `rules-30`) describe a direct write, but the scan cannot exclude one].

Answer: `cash_add` clamps only the top (9 999 999), `pay` only the bottom (−9 999 999); neither re-clamps the other side, which is why an edited 50 000 000 shows 49 999 229 after the bill and 9 999 999 after the first income. By absolute address, no other code writes the field.
