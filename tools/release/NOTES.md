OpenMittelland needs the files of your own copy of *Mittelland AD*; none are included.

On the first start it looks for the game in the usual places (next to the program, your home, Games and Downloads folders, Steam and GOG libraries). If it does not find it, it asks you for the folder: pick the game's install folder or the `MITTELL` folder that holds `MITTELL.EXE`. The choice is remembered.

- **Windows**: unzip, start `OpenMittelland.exe`. SmartScreen may warn about an unknown publisher: "More info" → "Run anyway".
- **macOS** (Apple silicon and Intel, macOS 11+): unzip, move `OpenMittelland.app` where you like. The app is not notarized, so macOS refuses the first start ("Apple could not verify..."): click "Done", then open System Settings → Privacy & Security and click "Open Anyway" next to OpenMittelland. Or in Terminal: `xattr -dr com.apple.quarantine OpenMittelland.app`.
- **Linux** (x86_64, glibc 2.34 or newer, e.g. Ubuntu 22.04, Debian 12): unpack, run `./openmittelland`. Needs ALSA (`libasound2`).

Saves and options go to `%APPDATA%\mittelland-ad` (Windows) or `~/.local/share/mittelland-ad`.
