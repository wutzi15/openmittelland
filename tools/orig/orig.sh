#!/bin/zsh
# Drives the original game headless: DOSBox-X on Xvfb in Docker, input by xdotool, 640x480 screenshots.
# Game coordinates are screen coordinates. Nothing touches the host mouse.
#   orig.sh start | click x y | rclick x y | move x y | key <xdotool key> | type text | shot file.png | stop
# INSTALL: the installed game dir (holds MITTELL/CONFI2 saves), DISK: the original disk, CONF: dosbox-x config
# whose autoexec mounts c=/dos/install and e=/disk.
set -e
H=$(cd "$(dirname $0)" && pwd)
: ${INSTALL:?set INSTALL} ${DISK:?set DISK} ${CONF:=$H/game.conf}
x() { docker exec mad sh -c "$1"; }
case $1 in
  start)
    docker image inspect mad-dosbox >/dev/null 2>&1 || docker build -q -t mad-dosbox "$H" >/dev/null
    docker rm -f mad >/dev/null 2>&1 || true
    docker run -d --name mad -v "$INSTALL":/dos/install -v "$DISK":/disk:ro -v "$CONF":/k/game.conf:ro mad-dosbox >/dev/null
    sleep 2; docker exec -d mad sh -c 'dosbox-x -conf /k/game.conf > /tmp/dosbox.log 2>&1'
    sleep 18; x 'xdotool key Escape'; sleep 5 ;;
  click|rclick) b=1; [ $1 = rclick ] && b=3
    x "xdotool mousemove $(($2+6)) $(($3+6)) sleep 0.2 mousemove $2 $3 sleep 0.3 mousedown $b sleep 0.4 mouseup $b" ;;
  # short press (120 ms): long enough for the polled button state (formats §11.1), below the 170 ms auto-repeat (rules-26 §7)
  tap) x "xdotool mousemove $(($2+6)) $(($3+6)) sleep 0.2 mousemove $2 $3 sleep 0.3 mousedown 1 sleep 0.12 mouseup 1" ;;
  move) x "xdotool mousemove $(($2+6)) $(($3+6)) sleep 0.2 mousemove $2 $3" ;;
  key) x "xdotool key $2" ;;
  type) x "xdotool type --delay 150 '$2'" ;;
  shot) x 'import -window root /tmp/s.png'; docker cp mad:/tmp/s.png "$2" >/dev/null ;;
  stop) docker rm -f mad >/dev/null ;;
esac
