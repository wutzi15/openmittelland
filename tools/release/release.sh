#!/bin/zsh
# Builds the downloadable OpenMittelland archives from the committed HEAD on this Mac and the two build machines
# (GitHub CI is not available), into dist/VERSION/:
#   OpenMittelland-VERSION-macos.zip         universal .app (arm64 + x86_64), ad-hoc signed
#   OpenMittelland-VERSION-linux-x86_64.tar.gz   built in rust:1.100-bookworm (binary needs glibc 2.34) on the Linux host
#   OpenMittelland-VERSION-windows-x86_64.zip    MinGW build on the Windows host
#   release.sh VERSION [--publish OWNER/REPO]   (--publish: gh release create vVERSION with the three files)
# Hosts: LINUX (default 192.168.2.244, needs docker), WINDOWS (default mail@192.168.2.107, needs rustup with the
# x86_64-pc-windows-gnu toolchain and WinLibs MinGW from winget).
set -e
V=${1:?version}; PUB=; [ "$2" = --publish ] && PUB=${3:?owner/repo}
: ${LINUX:=192.168.2.244} ${WINDOWS:=mail@192.168.2.107}
R=$(cd "$(dirname $0)/../.." && pwd); cd "$R"
git diff --quiet HEAD || { echo "commit first: the archives are built from HEAD"; exit 1; }
D=$R/dist/$V; N=OpenMittelland-$V; rm -rf "$D"; mkdir -p "$D"
git archive --format=tar.gz -o "$D/src.tgz" HEAD
DOCS=(README.md LICENSE)

linux() {
  ssh $LINUX "rm -rf omt-rel && mkdir omt-rel" && scp -q "$D/src.tgz" $LINUX:omt-rel/
  ssh $LINUX "cd omt-rel && tar xzf src.tgz && docker run --rm -e U=\$(id -u):\$(id -g) -v \$PWD:/s -w /s rust:1.100-bookworm sh -c \
    'apt-get update -qq >/dev/null && apt-get install -y -qq libasound2-dev >/dev/null && cargo build -q --release -p mad-app && chown -R \$U target' \
    && mkdir $N && cp target/release/mad-app $N/openmittelland && cp ${DOCS[*]} $N/ && tar czf $N-linux-x86_64.tar.gz $N"
  scp -q $LINUX:omt-rel/$N-linux-x86_64.tar.gz "$D/"
}

windows() {
  local mg='C:\Users\mail\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin'
  ssh $WINDOWS "rmdir /s /q omt-rel 2>nul & mkdir omt-rel" && scp -q "$D/src.tgz" $WINDOWS:omt-rel/
  ssh $WINDOWS "set PATH=%USERPROFILE%\\.cargo\\bin;$mg;%PATH%&& cd omt-rel && tar xzf src.tgz && cargo build -q --release -p mad-app && mkdir $N && copy target\\release\\mad-app.exe $N\\OpenMittelland.exe >nul && copy README.md $N\\ >nul && copy LICENSE $N\\LICENSE.txt >nul && tar -a -cf $N-windows-x86_64.zip $N"
  scp -q $WINDOWS:omt-rel/$N-windows-x86_64.zip "$D/"
}

macos() {
  local B=$D/mac; mkdir -p $B && tar xzf "$D/src.tgz" -C $B
  (cd $B && for t in aarch64-apple-darwin x86_64-apple-darwin; do cargo build -q --release -p mad-app --target $t; done)
  local A=$D/$N/OpenMittelland.app; mkdir -p $A/Contents/MacOS
  lipo -create -output $A/Contents/MacOS/openmittelland $B/target/{aarch64,x86_64}-apple-darwin/release/mad-app
  sed "s/@VERSION@/$V/g" "$R/tools/release/Info.plist" > $A/Contents/Info.plist
  mkdir -p $A/Contents/Resources && cp "$R/crates/mad-app/icon/icon.icns" $A/Contents/Resources/
  codesign --force --deep -s - $A
  cp ${DOCS[@]} $D/$N/
  (cd $D && ditto -c -k --norsrc --keepParent $N $N-macos.zip)
}

linux & lp=$!
windows & wp=$!
macos
wait $lp && wait $wp
rm -rf "$D/src.tgz" "$D/mac" "$D/$N"
ls -la "$D"
[ -n "$PUB" ] && gh release create v$V "$D"/*.zip "$D"/*.tar.gz --repo $PUB --title "OpenMittelland $V" --notes-file "$R/tools/release/NOTES.md"
true
