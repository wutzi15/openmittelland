## What's new in 0.1.7

- **Computer persons act like in the original.** A computer person who changes his insurance still gets his action that year (trade shipments, projects and so on), as in the original; before, it used up his turn. Measured side by side on a playtest save from 1619 (with the random generator re-synchronised at the year end, since the original keeps drawing numbers while the debtors' tower waits for a click): the resulting save matches the original's except for 40 bytes of 69 319, with every computer shipment, office change, dismissal and election identical.
- Smaller fixes from the same comparison: the right human is picked for computer projects and the saint's blessing, only computer clergy are blessed, and shipment sizes and cart counts follow the original's arithmetic.

## Recently

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
