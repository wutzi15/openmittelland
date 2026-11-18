#!/bin/zsh
# A/B capture: runs the same input script on the original (orig.sh, Docker) and on mad-app (headless),
# from the start menu with the save in $INSTALL/MITTELL/CONFI2. Writes OUT/NAME-orig.png and OUT/NAME-ours.png.
#   ab.sh OUT NAME "L x y" "R x y" "T text" ...   (each step is followed by a 4 s / 400 tick wait)
# The original keeps running between calls when KEEP=1 (then the first steps should navigate from where it is).
set -e
H=$(cd "$(dirname $0)" && pwd)
OUT=$1 NAME=$2; shift 2
: ${INSTALL:?} ${DISK:?}
docker ps -q --filter name=^mad$ | grep -q . || "$H/orig.sh" start
args=() tick=100 last=320,240
for s in "$@"; do
  set -- ${=s}
  case $1 in
    L) "$H/orig.sh" click $2 $3; args+=(--click "$2,$3@$tick"); last=$2,$3 ;;
    S) "$H/orig.sh" tap $2 $3; args+=(--click "$2,$3@$tick"); last=$2,$3 ;;
    R) "$H/orig.sh" rclick $2 $3; args+=(--right "$2,$3@$tick"); last=$2,$3 ;;
    M) "$H/orig.sh" move $2 $3; last=$2,$3 ;;
    T) "$H/orig.sh" type "$2"; args+=(--type "$2@$tick") ;;
  esac
  sleep 4; tick=$((tick + 400))
done
"$H/orig.sh" shot "$OUT/$NAME-orig.png"
[ "$KEEP" = 1 ] || "$H/orig.sh" stop
SD=$(mktemp -d)
"$H/../../target/debug/mad-app" --data "$DISK/MITTELL" --saves "$INSTALL/MITTELL/CONFI2" --save-dir "$SD" --start menu --mute \
  --screenshot "$OUT/$NAME-ours.png" --frames $tick --mouse $last "${args[@]}" >/dev/null 2>&1
rm -rf "$SD"
