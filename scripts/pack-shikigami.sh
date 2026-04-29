#!/usr/bin/env bash
# pack-shikigami.sh — build a `<id>.shikigami` zip archive from an
# unpacked character directory.
#
# The output matches the format the in-app installer (#14) consumes:
# `manifest.json` at the archive root + `assets/states/...` (sprite) or
# Live2D model files. Validates the manifest before packing so we never
# ship a bundle the installer would reject.
#
# Usage:
#   ./scripts/pack-shikigami.sh <character-dir>
#   ./scripts/pack-shikigami.sh characters/hiyori           # → hiyori.shikigami
#   ./scripts/pack-shikigami.sh characters/hiyori out/      # → out/hiyori.shikigami
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <character-dir> [out-dir]" >&2
  exit 1
fi

SRC="${1%/}"
OUT_DIR="${2:-.}"

if [[ ! -d "$SRC" ]]; then
  echo "✗ not a directory: $SRC" >&2
  exit 1
fi
if [[ ! -f "$SRC/manifest.json" ]]; then
  echo "✗ $SRC/manifest.json missing — not a valid character package" >&2
  exit 1
fi

# Cheap manifest sanity check (full validation runs in the Rust
# installer; this catches the common typos before we burn time zipping).
ID=$(python3 -c "
import json, sys
m = json.load(open('$SRC/manifest.json'))
required = ['schemaVersion','id','name','author','version','license','renderer','defaultState','states']
missing = [k for k in required if k not in m]
if missing:
    print(f'missing required keys: {missing}', file=sys.stderr); sys.exit(1)
if m['schemaVersion'] != '1.0':
    print(f'unsupported schemaVersion {m[\"schemaVersion\"]!r}', file=sys.stderr); sys.exit(1)
import re
if not re.fullmatch(r'[a-z0-9][a-z0-9-]{2,63}', m['id']):
    print(f'id {m[\"id\"]!r} has invalid characters', file=sys.stderr); sys.exit(1)
for req_state in ['idle','happy']:
    if req_state not in m['states']:
        print(f'missing required state: {req_state}', file=sys.stderr); sys.exit(1)
print(m['id'])
")

mkdir -p "$OUT_DIR"
# Resolve to an absolute path so the `cd $SRC` below doesn't break the
# zip target — `pwd -P` follows symlinks too, matching what the
# installer expects.
OUT_FILE="$(cd "$OUT_DIR" && pwd -P)/$ID.shikigami"
rm -f "$OUT_FILE"

# Pack via `zip -r` from the character dir so manifest.json sits at
# archive root (the location the installer probes first). Excludes the
# usual junk: macOS .DS_Store, editor temp files, raw asset workspaces.
( cd "$SRC" && zip -qr "$OUT_FILE" . \
    -x ".DS_Store" \
    -x "*.swp" \
    -x "*~" \
    -x "raw/*" "raw" \
    -x "psd/*" "psd" \
    -x ".tmp/*" )

SIZE=$(stat -f%z "$OUT_FILE" 2>/dev/null || stat -c%s "$OUT_FILE")
echo "✓ packed $ID → $OUT_FILE ($SIZE bytes)"
echo "  install via: tauri command \`install_character_zip\` or"
echo "               Settings → Install .shikigami package… → Browse"
