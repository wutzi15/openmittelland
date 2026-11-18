# Running the original under DOSBox-X (macOS)

Reference runs for fidelity checks. Nothing here is committed with game data.

- Install: `brew install dosbox-x cliclick` (the cask is disabled; the formula has a bottle).
- Work on a copy: `cp -R <MittellandAD>/game/install <scratch>/dos/install` (the game writes `CONFI2/`). Mount the CD part directly: `mount e <MittellandAD>/game/disk`.
- Config: the Steam `config/mittelland_ad.conf` cut at the line starting `[autoexec]` (not the first mention in a comment), then `fullscreen=false`, `output=surface`, `autolock=false`, `maximize=false`, `scaler=none`, `captures=<scratch>/dos/capture`, and an autoexec that mounts c and e, `cd \MITTELL`, runs `MITTELL.EXE`.
- Start: `dosbox-x -conf game.conf -noconsole &`.
- Capture only the emulator window (never the whole screen): find its id with a CoreGraphics window list, then `screencapture -x -o -l<id> out.png`. Needs Screen Recording permission for the host app.
- Input: key and mouse injection (`osascript` System Events, `cliclick`) needs Accessibility permission for the host app of the session; not granted as of 2026-11-06.
- The original's random sequence is clock seeded, so compare layouts, sprites and text, not outcomes.

## Cleanup
- Always end a run with `pkill -9 dosbox-x` (a plain `pkill` can leave the window open) and check `pgrep -l dosbox-x` prints nothing. Never leave an emulator window behind.
- Capture the emulator window only (`screencapture -l<id>`), never the whole screen.
- The working copy (`<scratch>/dos`) can stay between runs; delete it when the comparisons are done.
