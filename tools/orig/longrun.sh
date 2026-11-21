#!/bin/zsh
# Passive long run of the original: ends every turn ("Ja" on the confirm box), right-clicks through every other
# window and copies each autosave ($INSTALL/MITTELL/CONFI2/GAME5.DAT, written at each year start) to OUT/auto-N.DAT.
#   longrun.sh OUT [max autosaves]     (orig.sh started, a game loaded)
# Screens are told apart by ImageMagick hashes of fixed crops (Kontor banner, confirm box).
# A menu with bullets is answered with its last line (copied to OUT/q-N.png). A screen that stays
# the same through several right clicks gets a left tap, Return, then Escape; still stuck: OUT/stuck.png, exit 3.
# When DOSBox has exited, the screen before is kept as OUT/crash-N.png. That is the game ending (the last human ruined,
# credits), not a crash: the run stops with exit 4. REPLAY=1 restarts from the last autosave instead ("Das letzte Spiel
# fortsetzen"); the original reseeds from the clock and autosaves that year again before its first turn (rules-70 §1,
# rules-69 §7.5), so a restart adds a duplicate auto-N of one year and every later autosave is conditioned on the
# human surviving the replay. Use it only for statistics that do not depend on the human.
H=$(cd "$(dirname $0)" && pwd)
O=$H/orig.sh OUT=$1 MAX=${2:-30}
: ${INSTALL:?} ${DISK:?}
SAVE=$INSTALL/MITTELL/CONFI2/GAME5.DAT
mkdir -p "$OUT"
# crop hashes: "Kontor, A.D." in the banner; the turn-end confirm box text
KONTOR=${KONTOR:-50f319c0f25474f38f774b5d6147d31048acca728b4387f4ea944387f9639440}
CONFIRM=${CONFIRM:-a7ae3b8c085eed603269007d4a4905f74baabe9c0887d285be6fba3165c4fb8c}
sig() { docker exec mad sh -c "cp /tmp/s.png /tmp/prev.png 2>/dev/null; import -window root /tmp/s.png; for c in 640x480+0+0 $1; do convert /tmp/s.png -crop \$c +repage -format '%#\n' info:; done" | tr '\n' ' '; }
alive() { docker exec mad sh -c 'cat /proc/[0-9]*/comm 2>/dev/null' | grep -q dosbox; }
n=$(ls "$OUT"/auto-*.DAT(N) | wc -l | tr -d ' ')
last=$(stat -f %m "$SAVE") prev= same=0 i=0 crash=0 q=$(ls "$OUT"/q-*.png(N) | sed "s/.*q-//; s/.png//" | sort -n | tail -1); q=${q:-0}
while (( n < MAX )); do
  (( i++ % 10 == 0 )) && "$O" shot "$OUT/last.png"
  if ! docker info >/dev/null 2>&1; then echo "$(date +%T) docker down, waiting"; until docker info >/dev/null 2>&1; do sleep 10; done; fi
  if ! alive; then
    crash=$((crash + 1)); docker cp mad:/tmp/prev.png "$OUT/crash-$crash.png" >/dev/null
    if [ -z "$REPLAY" ]; then echo "$(date +%T) game ended after auto-$n (crash-$crash.png)"; exit 4; fi
    echo "$(date +%T) game ended after auto-$n, replay $crash"
    "$O" start; "$O" tap 320 263; sleep 8; continue
  fi
  set -- $(sig "120x24+420+10 200x40+220+190")
  full=$1 kontor=$2 confirm=$3
  # a menu with bullet icons (colour #2929B5): answer with the last line (no gift, decline, nobody)
  b=$(docker exec mad sh -c 'convert /tmp/s.png txt:- | grep -i "#2929B5" | tr ",:" "  " | sort -k2,2n | tail -1' | cut -d' ' -f1,2)
  if [ -n "$b" ] && [ "$kontor" != "$KONTOR" ]; then
    q=$((q + 1)); docker cp mad:/tmp/s.png "$OUT/q-$q.png" >/dev/null; set -- ${=b}; "$O" tap $1 $(($2 - 8)); echo "$(date +%T) menu q-$q: last bullet"
  elif [ "$confirm" = "$CONFIRM" ]; then "$O" tap 289 267
  elif [ "$kontor" = "$KONTOR" ]; then "$O" rclick 320 240
  else "$O" rclick 600 450; fi
  [ "$full" = "$prev" ] && same=$((same + 1)) || same=0
  prev=$full
  case $same in
    4) "$O" tap 600 450 ;;
    6) "$O" key Return ;;
    8) "$O" key Escape ;;
    10) "$O" shot "$OUT/stuck.png"; echo "stuck after auto-$n"; exit 3 ;;
  esac
  sleep 1.5
  m=$(stat -f %m "$SAVE")
  if [ "$m" != "$last" ]; then
    sleep 2; n=$((n + 1)); cp "$SAVE" "$OUT/auto-$n.DAT"; last=$m; echo "$(date +%T) auto-$n"
  fi
done
