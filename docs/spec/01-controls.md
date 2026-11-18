# 01 Controls (Die Steuerung)

Handbook chapter 2, H p.3-5.

## Input devices
- The game is mouse-driven. Every action is triggered by the left mouse button.
- Keyboard is used only for: name entry, bidding in an auction (via the player's hotkey), changing tactics during a raid/combat (via the player's hotkey). Video sequences can be skipped with Escape.
- Readme additions: Escape during combat auto-resolves the fight (see 06-combat.md); Escape during an auction skips the current lot (see 07-special-events.md). R p.7

## Interaction element types
1. **Icons** (ICONS): click performs an action or opens a new window.
2. **Hotspots** (Kontaktflächen): hovering highlights a label in gold text (example: desk -> "Schreibstube"); moving away hides it. Clicking opens a new game screen.
3. **Numbers** (Zahlen): each digit is edited separately.
   - Cursor in the upper half of a digit shows "+"; left-click increments that digit.
   - Cursor in the lower half shows "-"; left-click decrements that digit.
4. **Texts** (Texte): an editable text highlights on hover (gold text -> yellow-gold, black text -> red).
   - Upper half: cursor shows an up arrow; left-click switches to the next text value.
   - Lower half: cursor shows a down arrow; left-click switches to the previous text value.

### Number example (payment screen)
Office -> chest ("Bezahlung") opens the payment screen with four coin columns. Clicking the upper half of the ones digit under the "Arbeiter" (workers) column improves worker pay "by 2 percent"; then clicking the lower half of the tens digit leaves worker pay at 92 percent of normal wage. H p.4 (see open question 1).

## Closing / leaving
- Right mouse button closes any window and leaves any game screen.
- Example: Office -> score board above the chest opens the score window, which has no close icon; right-click closes it. Office -> cross opens the church; right-click returns to the office. H p.4-5
- Right-click in the office ends the player's round. A confirmation is asked (controlled by option "Runde bestätigen", see 03-banner.md). H p.5, p.8, p.28

## Source map
| Claim | Source |
|---|---|
| Mouse-only except name entry, auction bid, raid tactics | H p.3 |
| Escape skips videos | H p.3 |
| Left button triggers every action | H p.3 |
| Icon/hotspot behaviour, gold label "Schreibstube" | H p.4 |
| Payment screen has four coin columns | H p.4 |
| Digit upper half "+", lower half "-" | H p.4 |
| +2 percent / 92 percent example | H p.4 |
| Text hover colours and arrow behaviour | H p.4 |

| Right-click closes windows/leaves screens | H p.4 |
| Score window has no close icon | H p.4 |
| Right-click in office ends round, with confirmation | H p.5, p.8 |
| ESC in combat / auction | R p.7 |

## Open questions
1. Payment example: one click on the ones digit is said to raise pay by 2 percent, and one tens-digit decrement then yields 92 percent. A +1 ones step from 100 would give 101 -> 91. Is the start value not 100, is the ones step 2, or is the example inaccurate?
2. Do digits wrap (9 -> 0) or clamp, and does a digit increment carry into the next digit?
3. What are the min/max bounds for each editable number (wages, order quantities, bribe amounts)?
4. Do text selectors wrap around at the end of their list?
5. Does a right-click in a sub-window close only that window or the whole screen?
6. Is the confirmation on round end shown only when "Runde bestätigen" is on (H p.8, p.28) or always (H p.5 says "sicherheitshalber gefragt")?
