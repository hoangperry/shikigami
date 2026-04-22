#!/usr/bin/env bash
# fetch-hiyori-sample.sh — download Live2D's free Hiyori sample and wrap it
# into a Shikigami-compatible character package.
#
# Source: https://github.com/Live2D/CubismWebSamples (MIT code; model assets
# shipped under the Live2D Free Material License).
#
# Usage:
#   ./scripts/fetch-hiyori-sample.sh
# Writes: characters/hiyori/
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/characters/hiyori"
REPO="Live2D/CubismWebSamples"
MODEL_PATH="Samples/Resources/Hiyori"
RAW_BASE="https://raw.githubusercontent.com/$REPO/master/$MODEL_PATH"

echo "→ downloading Hiyori from $REPO"
rm -rf "$OUT"
mkdir -p "$OUT/model" "$OUT/model/Hiyori.2048" "$OUT/model/motions"

fetch_dir() {
  local remote="$1"      # path inside the repo
  local local_prefix="$2" # local subdir inside $OUT/model
  gh api "repos/$REPO/contents/$remote" --paginate \
    | python3 -c "
import json, sys
data = json.load(sys.stdin)
if isinstance(data, dict): data = [data]
for item in data:
    if item.get('type') == 'file' and item.get('download_url'):
        print(item['download_url'], item['name'])
"  | while IFS=' ' read -r url name; do
    echo "  ↓ $local_prefix/$name"
    curl -fsSL "$url" -o "$OUT/model/$local_prefix/$name"
  done
}

# Top-level Hiyori files (.model3.json, .moc3, .cdi3.json, etc.)
gh api "repos/$REPO/contents/$MODEL_PATH" --paginate \
  | python3 -c "
import json, sys
data = json.load(sys.stdin)
if isinstance(data, dict): data = [data]
for item in data:
    if item.get('type') == 'file' and item.get('download_url'):
        print(item['download_url'], item['name'])
"  | while IFS=' ' read -r url name; do
    echo "  ↓ $name"
    curl -fsSL "$url" -o "$OUT/model/$name"
  done

fetch_dir "$MODEL_PATH/Hiyori.2048" "Hiyori.2048"
fetch_dir "$MODEL_PATH/motions"     "motions"

cat > "$OUT/manifest.json" <<'JSON'
{
  "$schema": "https://shikigami.dev/schema/manifest/v1.0.json",
  "schemaVersion": "1.0",
  "id": "hiyori",
  "name": "Hiyori (Live2D sample)",
  "description": "Live2D Cubism sample character. Ships under Live2D Free Material License.",
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

# Per-state directories each point at the single model3.json entry.
for state in idle happy focused warning sleepy; do
  d="$OUT/assets/states/$state"
  mkdir -p "$d"
  rm -f "$d/frame_00.model3.json"
  ln -sf "../../../model/Hiyori.model3.json" "$d/frame_00.model3.json"
done

cat > "$OUT/LICENSE" <<'TXT'
Hiyori character model © Live2D Inc.
Distributed under the Live2D Free Material License.
See https://www.live2d.com/en/download/sample-data/ for current terms.
Permits personal and small-scale commercial use (annual revenue
under 10 million JPY).
TXT

cat > "$OUT/README.md" <<'MD'
# Hiyori — Live2D Sample Character

Downloaded from Live2D/CubismWebSamples. Model by Live2D Inc., distributed
under their Free Material License.

Regenerate with:
    ./scripts/fetch-hiyori-sample.sh
MD

echo "✓ done. Hiyori installed at $OUT"
echo "  Restart pnpm tauri:dev — Shikigami should detect the new character."
