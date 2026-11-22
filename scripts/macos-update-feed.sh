#!/bin/sh
# Prepare the GitHub release's signed Sparkle feed. Never uploads anything.
# Call after signing/notarizing the final DMG: <dmg> <output-directory>.
set -eu
cd "$(dirname "$0")/.."
dmg=$1
out=$2
sparkle=$(sh scripts/prepare-sparkle.sh)
version=$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' packaging/Info.plist)
public=$(/usr/libexec/PlistBuddy -c 'Print SUPublicEDKey' packaging/Info.plist)
mkdir -p "$out"
stage=$(mktemp -d "$out/.feed.XXXXXX")
trap 'rm -rf "$stage"' EXIT HUP INT TERM
name=$(basename "$dmg")
cp "$dmg" "$stage/$name"
python3 - "$version" "$stage/${name%.dmg}.md" <<'PY'
import json, sys
from pathlib import Path
version, output = sys.argv[1:]
log = json.loads(Path('changelog.json').read_text())
notes = next(n for n in log['releases'] if n['version'] == version)
Path(output).write_text('# Version ' + version + '\n\n' + '\n'.join('- ' + x for x in notes['changes']) + '\n')
PY
python3 scripts/sparkle-sign.py --sparkle-root "$sparkle" --public-key "$public" \
    generate_appcast --embed-release-notes \
    --download-url-prefix "https://github.com/gokcol/Ashina/releases/download/v$version/" \
    --full-release-notes-url https://github.com/gokcol/Ashina/releases \
    "$stage"
python3 scripts/sparkle-sign.py --sparkle-root "$sparkle" --public-key "$public" \
    sign_update --verify "$stage/appcast.xml"
cp "$stage/appcast.xml" "$out/appcast.xml"
echo "Signed feed ready: $out/appcast.xml (upload with the final DMG)"
