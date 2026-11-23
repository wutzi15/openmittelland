## What's new in 0.2.0

**Extras.** Press **Tab** (or Spieloptionen → Optionen → "Extras...") for optional features the original does not have. All are off by default, so a fresh install still plays like the original, and none of them changes your save files: they are stored in `prefs.txt` next to the saves. The README explains each one.

- Comfort: game speed up to 4x, fullscreen and sharp scaling, autosave history every 1, 5 or 10 years, undo of the last turn (with fresh dice), choosing the game number or a daily game, keyboard shortcuts for the Kontor, and tooltips (as a box at the pointer or a line at the screen edge) that say what a click does and what each Extras option means.
- Skipping: Space skips court scenes, the funeral, elections and dismissals (the outcome is the same as watching), and Tab lets an auction lot run without you.
- Helpers: debt warnings at the start and end of your turn and before buying a branch you cannot afford, a production helper with the best worker ratio and the expected yield, a yearly ledger, a price table of every city (highest price in gold, lowest in red), selling every branch's stock at once, a chronicle of all reports, copying orders between branches, office info that shows who appoints an office and how the vote stands, and safety questions before bribes, blackmail and spies.
- House rules: debt never takes your last branch, up to 255 workers per production order with every worker counting fully, and names for the newspaper contest's unnamed disciplines.

**Closer to the original.**

- The year end matches the original byte for byte on a long-run save from 1619 (before: 40 of 69 319 bytes differed).
- Turn-end reports separate events by blank lines, a new title shows in the banner at once, and short texts before a question no longer wait for an extra click.
- Dismissals, elections without a vote, the creditors' jury and the debtors' tower print their texts on the picture where the original does; the jurors show the original's markers.
- Privilege dialogs print their closing lines inside the window, judge and sentence choices and "Wollt Ihr weiterspielen?" use the original's layout, and the funeral follows the original's timing.
- Battles: captured men stay in their unit as in the original, and in a battle between two human players both players' tactics keys work.

**Fixes.**

- The space bar works: names with spaces, skipping and ending the turn with Space.
- Election screen: your new title no longer shows before the vote, and the privileged candidate's line sits where the original puts it.
- The newspaper contest question fits on screen, and the production helper window grows with its text.

## Recently

- **0.1.7**: computer persons act like in the original; a playtest save from 1619 matched the original's year end except for 40 bytes.
- **0.1.6**: the turn end reads as one window: courtship gifts and adventure offers appear under the report text, answered menus stay in place.
- **0.1.5**: the year end follows the original more closely: one pass per person for favour, dismissals and wealth; computer persons' wealth stays within the cash limits; a replaced person inherits the predecessor's stored wealth; computer trade shipments start from a random branch. No more invented "Nicht möglich" lines.
- **0.1.4**: the bribery map switches to the church level again; election bullets only appear when you vote.
- **0.1.3**: naming a baby no longer jumps to the next window; long adventure menus scroll so every choice is visible; goods a branch holds but its city does not produce (e.g. after a stolen order book) can be sold.
- **0.1.2**: elections as in the original: computer candidates register for vacant offices, so applying means a real vote; adventure dialogs appear in the right order.

## Running it

OpenMittelland needs the files of your own copy of *Mittelland AD*; none are included.

On the first start it looks for the game in the usual places (next to the program, your home, Games and Downloads folders, Steam and GOG libraries). If it does not find it, it asks you for the folder: pick the game's install folder or the `MITTELL` folder that holds `MITTELL.EXE`. The choice is remembered.

- **Windows**: unzip, start `OpenMittelland.exe`. SmartScreen may warn about an unknown publisher: "More info" → "Run anyway".
- **macOS** (Apple silicon and Intel, macOS 11+): unzip, move `OpenMittelland.app` where you like. The app is not notarized, so macOS refuses the first start ("Apple could not verify..."): click "Done", then open System Settings → Privacy & Security and click "Open Anyway" next to OpenMittelland. Or in Terminal: `xattr -dr com.apple.quarantine OpenMittelland.app`.
- **Linux** (x86_64, glibc 2.34 or newer, e.g. Ubuntu 22.04, Debian 12): unpack, run `./openmittelland`. Needs ALSA (`libasound2`).

Saves and options go to `%APPDATA%\mittelland-ad` (Windows) or `~/.local/share/mittelland-ad`.

Found a bug or a difference from the original? Open an issue, with a screenshot and your save if you can.
