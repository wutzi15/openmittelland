# Feature inventory

Evidence = spec file + source page (`H` = handbook `docs/handbook/handbuch.md`, `R` = readme `docs/handbook/readme.md`). Confidence: **stated** = rule explicitly in the source; **example** = only illustrated by an example or in-world prose; **conflict** = sources disagree. Every rule is `[UNVERIFIED]` against the executable until `docs/re/` confirms it. Formulas and numbers not in the sources are `[NEEDS SOURCE]`.

## Rules and UI (from handbook)
| Area | Observed behavior (per source) | Evidence | Confidence | Status | Tests | Known discrepancies |
|---|---|---|---|---|---|---|
| Hotseat rounds | up to 6 human players play their year in turn; then >100 NPCs act | 02 H p.5; 04 H p.9 | stated | not started | - | - |
| Controls | left-click acts; right-click closes/leaves; digit +/- by upper/lower half; text cycle by up/down arrow | 01 H p.3-5 | stated | not started | - | +2 %/92 % example arithmetic unclear |
| Setup | players, difficulty, name/sex/faith (Cath./Prot.), colour, hotkey, uncle's branch, confirm, mission code | 02 H p.5-6 | stated | not started | - | - |
| Missions | 3 difficulties, cards with 3 secret codes; completion detected by game | 02 H p.5-6, R p.5 | stated | not started | - | mission list and predicates [NEEDS SOURCE] |
| Start state | 15 m² storage, no goods, 4/6 goods locked, 1600 | 02 H p.11, p.15, p.38 | stated | not started | - | start gold 750 (H p.38) vs 500 (H p.47) |
| Game end year | - | - | [NEEDS SOURCE] | not started | - | previous FEATURES.md listed "game end 1800" from readme; not in readme or handbook |
| Banner | shows year, cash, title, office; gold -> bribery; elsewhere -> options | 03 H p.6-7 | stated | not started | - | - |
| Bribery | map target (10 cities, 3 counties, principality via lake, castle, camp) -> secular/ecclesiastical -> office -> amount; same faith bigger effect | 03 H p.6-7 | stated | not started | - | effect formula [NEEDS SOURCE] |
| Save/load | 5 slots with comment; continue last game; no autosave | 03 H p.7-9 | stated | not started | - | format unanalysed |
| Options | history, adventures, fate, tips, warnings, confirm round, announce player, witnesses & pleas; all on by default | 03 H p.8, R p.7 | stated | not started | - | - |
| Graphics/music options | 256/32,768 colours; video size; scroll speed; 3 volume sliders (music inert) | 03 H p.8, R p.4 | stated | not started | - | - |
| New player mid-game | allowed if start slot and free city remain; average wealth | 03 H p.8 | stated | not started | - | - |
| Trade map | flags per owner; city name -> info; graphic -> branch or buy rights | 04 H p.9-10 | stated | not started | - | price of rights [NEEDS SOURCE] |
| Branch stock | 6 goods/city, 2-digit price, skill ring, sell 1/lot/all | 04 H p.11 | stated | not started | - | per-city goods list [NEEDS SOURCE] |
| Order book | 2 orders/branch/year; 4 types; status crossed/plain/broken; persists; costs at round end | 04 H p.12 | stated | not started | - | - |
| Production order | workers + means; ratios App. II; slack rule | 04 H p.12-13, 08 H p.36, R p.7 | stated | not started | - | output formula [NEEDS SOURCE] |
| Sell / move / fetch-loot orders | templates and semantics | 04 H p.13-14 | stated | not started | - | - |
| Transports | capacities cart 100, ship 1,000, balloon 50, steam cart 80, submarine 700, flying machine 30 | 04 H p.14 | stated | not started | - | how special transports are obtained [NEEDS SOURCE] |
| Ship reachability | not Moortal, Finsterwalde, Falkenhorst, Waldstein, camps | 04 H p.14 | stated | not started | - | - |
| Escort | any transport order; crossed swords; higher cost | 04 H p.14-15 | stated | not started | - | cost [NEEDS SOURCE] |
| Residence | buy/build/sell/auction/renovate/extend; favour bonus; title prerequisite | 04 H p.15, R p.9 | stated | not started | - | - |
| Storage | 15-20 units/m²; overflow donated; wood vs stone; yearly offers; sell anytime | 04 H p.15 | stated | not started | - | per-good units [NEEDS SOURCE] |
| Directors | hire via seal ring; % of budget; budgets in counting room; may quit | 04 H p.16-19, R p.8-9 | stated | not started | - | AI behaviour [NEEDS SOURCE] |
| Applications for office | needs citizenship; Nein/Ja; crystal shows rivals | 04 H p.17 | stated | not started | - | - |
| Privileges | office-specific actions (examples only) | 04 H p.18, p.26-27, p.50, p.54, R p.8-9 | example | not started | - | full mapping [NEEDS SOURCE] |
| Moneylender / loan book | high interest (>30 % common); repay; expiry -> jury court | 04 H p.18-19 | stated | not started | - | - |
| Payment | 4 columns; wages affect worker output and soldier fighting | 04 H p.4, p.19 | stated | not started | - | column names [NEEDS SOURCE] |
| Church | leave, convert, matchmaker, indulgence, donation (secular favour), candles (church favour), pray | 04 H p.20-21, R p.9 | stated | not started | - | prices [NEEDS SOURCE] |
| Toll castles / robber camps | 3 + 4; buy via interest; 4 troop types; orders Überfallt/Schickt/Plündert/Überwacht; recruit; manoeuvre; toll; hide book | 04 H p.21-25 | stated | not started | - | Schickt slot order conflict |
| Toll by destination | table of 10 cities | 04 H p.25 | stated | not started | - | - |
| Back room | spy, denounce, blackmail, sabotage | 04 H p.26-27 | stated | not started | - | formulas [NEEDS SOURCE] |
| Score window | humans + selected NPCs; crystal for details | 04 H p.28 | stated | not started | - | - |
| End of round | title award, cost report, fate, own-action sequences, adventures, jury court | 05 H p.28-29 | stated | not started | - | exact order [NEEDS SOURCE] |
| World turn | NPC deaths/dismissals/promotions; election screens by background; NPC actions; insults; history | 05 H p.29-30 | stated | not started | - | - |
| Fate events | examples only | 05 H p.28, p.64 | example | not started | - | table [NEEDS SOURCE] |
| Adventures | multi-year sub-plots; examples only | 05/18 H p.29, p.64 | example | not started | - | list and count [NEEDS SOURCE]; previous FEATURES.md "30+" unsourced |
| Combat | ransom offer, patrol mercenaries choose side, units 1-5, general/captain, hotkey tactics, 11 tactics, 3-field conquest, summary | 06 H p.30-33 | stated | not started | - | stats [NEEDS SOURCE] |
| Combat auto-resolve | ESC computes outcome in background | 06 R p.7 | stated | not started | - | - |
| Auction | announced years ahead; catalogue; opt-out unless selling; hotkey bid; ESC skips lot | 07 H p.33-34, R p.7 | stated | not started | - | - |
| Trial | charges, response, witnesses, pleas, judge bribes, verdict; 6 penalty kinds | 07 H p.34 | stated | not started | - | verdict formula [NEEDS SOURCE] |
| Duel | standing reset to average; health loss by fencing skill | 07 H p.34 | stated | not started | - | - |
| Office appointments | Appendix I table | 08 H p.35 | stated | not started | - | abbot/sacristan circularity |
| Certificates | 3 groups of 5 goods | 08 H p.36 | conflict | not started | - | Tuch III (H) vs II (R p.8) |
| Abbreviations | Appendix III | 08 H p.37 | stated | not started | - | - |
| Geography | 3 counties, 10 cities, capital Fürstenbrück | 09 H p.40 | stated | not started | - | coordinates [NEEDS SOURCE] |
| Pricing | yield and popularity rules; saturation | 10 H p.16, p.41-42 | stated | not started | - | formula [NEEDS SOURCE] |
| Laws | 3 books x 10; changed by finance/justice minister, archbishop; judges by accused's level | 13 H p.54 | stated | not started | - | current law list [NEEDS SOURCE] |
| Debt | forced loans, seizure escalation, jury court, dungeon | 13 H p.54-55 | stated | not started | - | thresholds [NEEDS SOURCE] |
| Family | matchmaker, gifts, canon veto, wedding in partner's city, children, baptism, will, heir | 14 H p.56-57, R p.9 | stated | not started | - | - |
| Ageing/health | life limited; illness, duels, pillory; quack | 14 H p.34, p.56 |
 stated | not started | - | model [NEEDS SOURCE] |
| Mistress/lover | standing bonus with like-minded; offence | 14/16 H p.54, p.61 | stated | not started | - | acquisition [NEEDS SOURCE] |
| Insurance, storage rental, feasts, gambling | appear only in narrative | 15/16 H p.58-61 | example | not started | - | existence unconfirmed |

## Assets and engine
| Area | Observed behavior | Evidence | Confidence | Status | Tests | Known discrepancies |
|---|---|---|---|---|---|---|
| FLC cutscenes (13) | Autodesk FLC, 320x180/320x200 | files + decoder | verified | decoder done | unit + ffmpeg cross-check | `FLI_COPY` chunks in INTRO are 2 bytes short; we copy what exists |
| ICON bitmaps | `ICONxx.DAT`, several sub-formats | files | low | not started | - | formats unanalysed |
| WAV speech/samples, OGG soundtrack, MIDI | standard formats | files | high (standard) | not started | - | - |
| Save/load, config | `CONFI2/GAME5.DAT`, `CONFIG.DAT`; readme save path `...\MITTELL\CONFIG\` | files, R p.4 | none | not started | - | format unanalysed |

Detailed rules, source maps and open questions: `docs/spec/INDEX.md`.
