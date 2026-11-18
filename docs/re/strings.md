# Text in MITTELL.EXE

All game text is in the data object (linear `0x80000`+), as NUL-terminated CP437 strings. There are 3203 strings of 4 or more characters, about 200 KB.

**The full dump is not committed.** It is the original game text, and `AGENTS.md` keeps original assets out of the repo. Regenerate it from your own install:

```
cargo run -p mad-assets --example exe_strings -- /path/to/MITTELL.EXE > strings.tsv   # address<TAB>text
```

A copy for the team is at `<scratch>` (outside the repo). The runtime should load text from the user's EXE the same way, through `mad_assets::le::LeExe`. Do not copy it into source.

## Encoding (verified)
CP437: `0x81` ü, `0x84` ä, `0x94` ö, `0x8E` Ä, `0x99` Ö, `0x9A` Ü, `0xE1` ß. `0x0A` appears only in DOS console messages (setup and low-memory errors). For font glyph codes see `formats.md` §2.

## Markup (frequency over all strings; meanings marked unless obvious)
| code | count | meaning |
|---|---|---|
| `$N` | 2255 | line break (consistent with every use) |
| `$Z` | 765 | [UNVERIFIED] paragraph/blank-line break; used between title and body |
| `$C` / `$L` / `$R` | 657 / 678 / 50 | [UNVERIFIED] centre / left / right alignment |
| `$P`, `$E`, `$h`, `$g`, `$W`, `$H`, `$(`, `$)` … | ≤ 239 | [UNVERIFIED] layout and colour switches |
| `%s`, `%d` | 657, 314 | string / integer argument (printf-like) |
| `%m`, `%M`, `%a`, `%o`, `%v`, `%j`, `%w`, `%l`, `%q`, `%g` | 4-293 | [UNVERIFIED] game-specific substitutions (`%m` appears where a menu choice or amount sits, `%o` before town names) |
| `%<k>N<n>` (e.g. `%1N1`, `%3N5`), `%<k>W`, `%4E`, `%4u` | | [UNVERIFIED] person/name references with grammatical case or variant |
| `%%` | | literal `%` |
| `^ … _` | | title ornaments (font glyphs `0x5E`/`0x5F`) |

## Regions
Address ranges with a content summary. Attribution to concrete screens needs cross-referencing and is [UNVERIFIED].

| linear range | strings | content |
|---|---|---|
| `0x80000-0x84000` | 19 | sound/MIDI driver and kernel error messages (`lapc_prepare…`, `opengraph…`), copyright line, file name templates |
| `0x84000-0x87000` | 81 | setup menu: sound card names, port lists, sample sets, directory names (mixed with binary tables) |
| `0x87000-0x88000` | 171 | short word lists: goods, materials, professions, titles, castle stats (`Kasematten`, `Sicherheit`) |
| `0x88000-0x89700` | 77 | place descriptions and robber-camp texts |
| `0x89700-0x89B00` | 114 | male first names |
| `0x89B00-0x93800` | 99 | historical chronicle entries (17th-century events by year) |
| `0x93800-0x93B00` | 72 | female first names |
| `0x93B00-0x9EAE0` | 892 | insults/quotes, book titles, tavern encounters, court-trial testimony, yearly news and contests, event messages |
| `0x9EAE0-0xA3A00` | 324 | found objects, item descriptions, credits (speakers), trial texts, prophecies |
| `0xA3A00-0xA5C00` | 144 | military: sieges, mercenaries (`Landsknechte`), bombardment menu |
| `0xA5C00-0xB9900` | 652 | event texts: discoveries, miracles, contests, certificates (`Urkunde`), insurance, long narrative events |
| `0xB9900-0xBB000` | 85 | install/copy-to-disk dialogs, soundcheck, MIDI/FLC file names |
| `0xBB000-0xC3000` | 473 | dialogs: battle results, ransom, evidence/crimes, laws (`Gesetze in Mittelland`), quit (`Spiel beenden`), save (`Spielstand gespeichert`), castle extension (`Zollburg erweitern`), privileges, extortion, building menus |
