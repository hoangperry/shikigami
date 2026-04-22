#!/usr/bin/env bash
# fetch-hiyori-sample.sh — download Live2D's free Hiyori sample model and
# wrap it into a Shikigami-compatible character package.
#
# Hiyori ships under Live2D's Free Material License:
# https://www.live2d.com/en/download/sample-data/
# Commercial use permitted for organizations earning < $10M JPY / year.
# User is responsible for complying with the license when redistributing.
#
# Usage:
#   ./scripts/fetch-hiyori-sample.sh
#   → writes characters/hiyori/
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/characters/hiyori"
TMP="$(mktemp -d)"
trap "rm -rf $TMP" EXIT

# Cubism Sample Hiyori model from Live2D's CubismWebSamples repo (MIT).
REPO="Live2D/CubismWebSamples"
MODEL_PATH="Samples/TypeScript/Demo/Resources/Hiyori"

echo "→ downloading Hiyori from $REPO/$MODEL_PATH"
mkdir -p "$OUT/model"

gh api "repos/$REPO/contents/$MODEL_PATH" --paginate \
  | python3 -c "
import json, sys
data = json.load(sys.stdin)
if isinstance(data, dict): data = [data]
for item in data:
    print(item['download_url'] or '', item['path'] or '', item['type'] or '')
" | while IFS=' ' read -r url path typ; do
  [ "$typ" = "file" ] || continue
  [ -n "$url" ] || continue
  rel="${path#$MODEL_PATH/}"
  mkdir -p "$OUT/model/$(dirname "$rel")"
  echo "  ↓ $rel"
  curl -fsSL "$url" -o "$OUT/model/$rel"
done

# Some subdirs (motions, expressions) need recursive fetch.
for sub in motions expressions; do
  mkdir -p "$OUT/model/$sub"
  gh api "repos/$REPO/contents/$MODEL_PATH/$sub" --paginate 2>/dev/null \
    | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    if isinstance(data, dict): data = [data]
    for item in data:
        if item.get('type') == 'file' and item.get('download_url'):
            print(item['download_url'], item['name'])
except Exception:
    pass
" | while IFS=' ' read -r url name; do
    echo "  ↓ $sub/$name"
    curl -fsSL "$url" -o "$OUT/model/$sub/$name"
  done
done

cat > "$OUT/manifest.json" <<'JSON'
{
  "$schema": "https://shikigami.dev/schema/manifest/v1.0.json",
  "schemaVersion": "1.0",
  "id": "hiyori",
  "name": "Hiyori (Live2D sample)",
  "description": "Live2D Cubism sample character — free material use.",
  "author": "Live2D Inc.",
  "version": "1.0.0",
  "license": "CC-BY-4.0",
  "tags": ["live2d", "anime", "sample", "sfw"],
  "renderer": "live2d",
  "defaultState": "idle",
  "states": {
    "idle":    { "path": "assets/states/idle",    "fps": 30, "loop": true },
    "happy":   { "path": "assets/states/happy",   "fps": 30, "loop": false, "then": "idle", "durationMs": 2000 },
    "focused": { "path": "assets/states/focused", "fps": 30, "loop": true },
    "warning": { "path": "assets/states/warning", "fps": 30, "loop": true },
    "sleepy":  { "path": "assets/states/sleepy",  "fps": 30, "loop": true }
  }
}
JSON

# Stage directories point to the same single .model3.json entry.
mkdir -p "$OUT/assets/states"/{idle,happy,focused,warning,sleepy}
MODEL3_JSON="$(ls "$OUT/model"/*.model3.json | head -1)"
MODEL3_NAME="$(basename "$MODEL3_JSON")"
for state in idle happy focused warning sleepy; do
  ln -sf "../../../model/$MODEL3_NAME" "$OUT/assets/states/$state/frame_00.model3.json"
done

cat > "$OUT/LICENSE" <<'TXT'
Hiyori character model © Live2D Inc.

This character is distributed under the Live2D Free Material License.
See https://www.live2d.com/en/download/sample-data/ for current terms.
TL;DR: free for personal and small-scale commercial use (annual revenue
below 10 million JPY).
TXT

cat > "$OUT/README.md" <<'MD'
# Hiyori — Live2D Sample Character

Bundled for Shikigami dev fixtures. Model files by Live2D Inc., distributed
under their Free Material License.

Regenerate with:
    ./scripts/fetch-hiyori-sample.sh
MD

echo "✓ done. Hiyori installed at $OUT"
echo "  Restart pnpm tauri:dev — Shikigami should detect the new character."
