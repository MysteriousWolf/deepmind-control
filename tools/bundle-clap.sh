#!/usr/bin/env bash
# Turns the plugin library into a CLAP a host can load.
#
#   cargo build --release --package control-plugin
#   tools/bundle-clap.sh                        # this machine, into dist/
#   tools/bundle-clap.sh aarch64-apple-darwin   # a cross build, into dist/
#
# A CLAP is the library itself under another name on Linux and Windows, and a
# bundle on macOS. There is nothing else to it, so this is a script rather than
# a tool to install: `cargo build` makes the library and this names it.
#
# Where the host looks for it:
#   Linux    ~/.clap  or  /usr/lib/clap
#   macOS    ~/Library/Audio/Plug-Ins/CLAP
#   Windows  %COMMONPROGRAMFILES%\CLAP
set -euo pipefail

name="DeepMind Control"
library="control_plugin"
target="${1:-}"
out="${2:-dist}"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
release="$root/target/${target:+$target/}release"

case "$(uname -s)" in
  Linux)
    from="$release/lib$library.so"
    kind=file
    ;;
  Darwin)
    from="$release/lib$library.dylib"
    kind=bundle
    ;;
  MINGW* | MSYS* | CYGWIN*)
    from="$release/$library.dll"
    kind=file
    ;;
  *)
    echo "no idea how to bundle a CLAP on $(uname -s)" >&2
    exit 1
    ;;
esac

if [ ! -f "$from" ]; then
  echo "$from is not there: run cargo build --release --package control-plugin first" >&2
  exit 1
fi

version=$(grep -m1 -E '^version = ' "$root/Cargo.toml" | cut -d'"' -f2)
mkdir -p "$out"
clap="$out/$name.clap"
rm -rf "$clap"

if [ "$kind" = file ]; then
  cp "$from" "$clap"
else
  mkdir -p "$clap/Contents/MacOS"
  cp "$from" "$clap/Contents/MacOS/$name"
  printf 'BNDL????' > "$clap/Contents/PkgInfo"
  cat > "$clap/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>$name</string>
  <key>CFBundleDisplayName</key><string>$name</string>
  <key>CFBundleExecutable</key><string>$name</string>
  <key>CFBundleIdentifier</key><string>io.github.mysteriouswolf.deepmind-control</string>
  <key>CFBundlePackageType</key><string>BNDL</string>
  <key>CFBundleSignature</key><string>????</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST
fi

echo "$clap"
