#!/bin/zsh
# Re-renders only our side of an A/B spec (OUT/NAME-ours.png per line) and prints the diff against the
# existing OUT/NAME-orig.png. Usage: ours.sh OUT SPEC
H=$(cd "$(dirname $0)" && pwd)
OUT=$1 SPEC=$2
while IFS='|' read -r name rest; do
  [ -z "$name" ] || [[ $name == \#* ]] && continue
  tick=100 args=() last=320,240
  for s in "${(@s:|:)rest}"; do
    set -- ${=s}
    case $1 in
      L|S) args+=(--click "$2,$3@$tick"); last=$2,$3 ;;
      R) args+=(--right "$2,$3@$tick"); last=$2,$3 ;;
      M) last=$2,$3 ;;
      T) args+=(--type "$2@$tick") ;;
    esac
    tick=$((tick + 400))
  done
  SD=$(mktemp -d)
  "$H/../../target/debug/mad-app" --data "$DISK/MITTELL" --saves "$INSTALL/MITTELL/CONFI2" --save-dir "$SD" --start menu --mute \
    --screenshot "$OUT/$name-ours.png" --frames $tick --mouse $last "${args[@]}" >/dev/null 2>&1
  echo "$name $(docker run --rm -v "$OUT":/o mad-dosbox compare -metric AE -fuzz 12% /o/$name-orig.png /o/$name-ours.png null: 2>&1)"
done < "$SPEC"
