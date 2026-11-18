# RE rules progress, agent C (castles/combat, auctions/feasts/duels, AI, crime/trials/privileges)

Ghidra project copy: `<scratch>`. Helpers in `<scratch>` (run with `<scratch>`):
- `dc2.py addr...` decompile with names to `out/<addr>.c`.
- `gasm2.py addr...` listing to `out/<addr>.s`, annotates string constants and pointer-table loads.
- `rdis.py start end` disassemble a raw range (code outside functions) to `out/r_<start>.s`.
- `ptab.py <tab> <n>` pointer table with strings; `b <addr> <n>` bytes; `w` words; `t <id>...` text ids via `0xB4258`.
- `xr.py addr...` callers/xrefs. `privtexts.txt` privilege names and descriptions.

| doc | status |
|---|---|
| rules-04b-crime-trials-privileges.md | done: laws, offence word, evidence, trials, penalties, privileges table and effects, sabotage execution |
| rules-07-castles-bandits-combat.md | mostly done: castle data, owner actions, computer castles, raids, battle setup/end, terrain codes, tactic AI, movement modes, damage. Open: human order screens, per-tactic step logic, sieges |
| rules-08-auction-feast-duel.md | done: auctions, castle private sale, feasts, duels, church |
| rules-09-ai.md | done: difficulty (missions only), year-end NPC triggers, npc_act, revenge, director AI |
| rules-38-privilege-dialogs.md | done: privileges menu (pages, blackmailed holders, entry/setting items), every privilege dialog (windows, texts, items, ranges), law-book edit mode; appendices: credits/quit screen, type-55 prisoners, Produktion skill draws, Pranger/Amtsentzug texts, blitter remap + map flags |
| rules-39-open-questions-2.md | done: map-flag timer animation (0x2D990), computer-insulter duel question texts, report-event silence table, Kontor music roll timing, battle music draws in ESC mode, concert RNG order, gift reaction table, mission-done "Nein" path, auction Esc, SPEC-GAP one-liners |

## Corrections to other docs (not edited here; reported to the team lead)
- rules-04 3.2: office-tree price multiplier default is 10, not 7; the discounts come from the sabotage privileges 47/73/78.
- rules-02: options byte `0x86AD1` bit 1 = Abenteuer, bit 3 = Spieltipps (the "option bit 4 / bit 8" labels look like a different numbering).
- rules-00 / names.txt: 0x5F080 is the privileges menu (not siege UI); 0x59940 is the miracle event (not computer misfortunes); the castle/county pairing question is settled (record k = location 14+k).

## Next steps
1. rules-07: per-tactic logic in 0x398CB..0x3AC80 (jump table 0x39870), human castle order screens 0x55BC8 / 0x579A1 / 0x57E70, sieges 0x1F2A0 / 0x1FA80, prisoners (type 55).
2. Jury court for debts (0x6449F region).
