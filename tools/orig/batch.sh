#!/bin/zsh
# Runs ab.sh for each line of a spec file: NAME|step|step|...  then writes OUT/diff.txt (pixels differing by >12%).
H=$(cd "$(dirname $0)" && pwd)
OUT=$1 SPEC=$2
while IFS='|' read -r name rest; do
  [ -z "$name" ] || [[ $name == \#* ]] && continue
  steps=("${(@s:|:)rest}")
  "$H/ab.sh" "$OUT" "$name" "${steps[@]}" || echo "$name failed"
done < "$SPEC"
: > "$OUT/diff.txt"
for f in "$OUT"/*-orig.png; do
  n=${f:t:r}; n=${n%-orig}
  d=$(docker run --rm -v "$OUT":/o mad-dosbox compare -metric AE -fuzz 12% /o/$n-orig.png /o/$n-ours.png /o/$n-diff.png 2>&1)
  echo "$n $d" >> "$OUT/diff.txt"
done
