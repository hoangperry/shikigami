#!/usr/bin/env bash
# fetch-live2d-sample.sh — download any Live2D Inc. free sample (Haru,
# Mao, Mark, Natori, Ren, Rice, Wanko, Hiyori) and wrap it into a
# Shikigami-compatible character package.
#
# Source: https://github.com/Live2D/CubismWebSamples
# License: Live2D Free Material License (personal + small commercial).
#
# Usage:
#   ./scripts/fetch-live2d-sample.sh <ModelName> [character-id]
#
# Layout produced (mirrors Hiyori — assets/states/idle holds the actual
# model files; other states symlink back to idle so they share the rig
# but can carry their own motion-group selection via manifest.json):
#
#   characters/<id>/
#     manifest.json
#     LICENSE
#     README.md
#     assets/states/idle/
#         frame_00.model3.json   (renamed from <Model>.model3.json)
#         <Model>.moc3
#         <Model>.physics3.json
#         <Model>.pose3.json
#         <Model>.cdi3.json
#         <Model>.userdata3.json
#         <Model>.2048/texture_*.png
#         motions/*.motion3.json
#         expressions/*.exp3.json (if present)
#         sounds/*.wav (if present)
#     assets/states/{happy,focused,warning,sleepy} → ../idle  (symlink)
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <ModelName> [character-id]" >&2
  echo "available: Haru Hiyori Mao Mark Natori Ren Rice Wanko" >&2
  exit 1
fi

MODEL="$1"
ID="${2:-$(echo "$MODEL" | tr '[:upper:]' '[:lower:]')}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/characters/$ID"
REPO="Live2D/CubismWebSamples"
MODEL_PATH="Samples/Resources/$MODEL"

# Where the actual model lives. Other states symlink to this dir.
IDLE="$OUT/assets/states/idle"

echo "→ downloading $MODEL → $OUT"
rm -rf "$OUT"
mkdir -p "$IDLE"

# Recursively fetch every file under a remote directory into a local one.
fetch_recursive() {
  local remote="$1"
  local local_dir="$2"
  mkdir -p "$local_dir"
  gh api "repos/$REPO/contents/$remote" --paginate \
    | python3 -c "
import json, sys
data = json.load(sys.stdin)
if isinstance(data, dict): data = [data]
for item in data:
    print(item['type'], item.get('download_url') or '-', item['name'])
" \
  | while IFS=' ' read -r type url name; do
      if [[ "$type" == "file" ]]; then
        echo "  ↓ $local_dir/$name"
        curl -fsSL "$url" -o "$local_dir/$name"
      elif [[ "$type" == "dir" ]]; then
        fetch_recursive "$remote/$name" "$local_dir/$name"
      fi
    done
}

# Pull everything into the idle state directly. mulmotion's loader
# resolves all FileReferences relative to the model3.json's URL, so the
# textures/motions/etc must live in the same directory tree.
fetch_recursive "$MODEL_PATH" "$IDLE"

# The model3.json is named "<Model>.model3.json" by Live2D; our renderer
# expects to see a "frame_00.model3.json" in the state dir (that's what
# the Rust loader picks up via its frames glob). Symlink instead of
# rename so the original filename is still resolvable for tooling.
ORIG_MODEL3=$(ls "$IDLE"/*.model3.json | head -1)
if [[ -z "${ORIG_MODEL3:-}" ]]; then
  echo "✗ no .model3.json found in $IDLE" >&2
  exit 1
fi
ln -sf "$(basename "$ORIG_MODEL3")" "$IDLE/frame_00.model3.json"
echo "  · model3.json: $(basename "$ORIG_MODEL3")"

# Other states are pure symlinks to idle — they share the rig but can
# select different motion groups via manifest.json's `motion` field.
cd "$OUT/assets/states"
for state in happy focused warning sleepy; do
  rm -rf "$state"
  ln -s idle "$state"
done
cd - >/dev/null

# Read motion groups so manifest can map states → real motion names.
MOTIONS=$(python3 -c "
import json
d = json.load(open('$ORIG_MODEL3'))
m = (d.get('FileReferences', {}) or {}).get('Motions') or {}
print(','.join(sorted(m.keys())))
")
echo "  · motion groups: ${MOTIONS:-<none>}"

# Pick a motion group matching the keyword (case-insensitive ERE), else
# fall back to the first available group. Bash regex doesn't support PCRE
# (?i) — we lowercase both sides and use a plain substring/ERE match.
pick_motion() {
  local pattern_lc="$1"   # lowercase ERE pattern
  IFS=',' read -ra arr <<< "$MOTIONS"
  for g in "${arr[@]}"; do
    local g_lc
    g_lc="$(echo "$g" | tr '[:upper:]' '[:lower:]')"
    if [[ "$g_lc" =~ $pattern_lc ]]; then
      echo "$g"
      return
    fi
  done
  echo "${arr[0]:-null}"
}

IDLE_M=$(pick_motion 'idle')
HAPPY_M=$(pick_motion 'tap|happy|active')
FOCUS_M=$(pick_motion 'idle')
WARN_M=$(pick_motion 'tap|surprise|warn')
SLEEP_M=$(pick_motion 'idle')

emit_motion() {
  local m="$1"
  if [[ "$m" == "null" || -z "$m" ]]; then echo 'null'; else echo "\"$m\""; fi
}

NAME="$MODEL (Live2D sample)"
cat > "$OUT/manifest.json" <<JSON
{
  "\$schema": "https://shikigami.dev/schema/manifest/v1.0.json",
  "schemaVersion": "1.0",
  "id": "$ID",
  "name": "$NAME",
  "description": "Live2D Cubism sample character. Live2D Free Material License.",
  "author": "Live2D Inc.",
  "version": "1.0.0",
  "license": "CC-BY-4.0",
  "tags": ["live2d", "anime", "sample", "sfw"],
  "renderer": "live2d",
  "defaultState": "idle",
  "states": {
    "idle":    { "path": "assets/states/idle",    "fps": 30, "loop": true,  "motion": $(emit_motion "$IDLE_M") },
    "happy":   { "path": "assets/states/happy",   "fps": 30, "loop": false, "then": "idle", "durationMs": 2000, "motion": $(emit_motion "$HAPPY_M") },
    "focused": { "path": "assets/states/focused", "fps": 30, "loop": true,  "motion": $(emit_motion "$FOCUS_M") },
    "warning": { "path": "assets/states/warning", "fps": 30, "loop": true,  "motion": $(emit_motion "$WARN_M") },
    "sleepy":  { "path": "assets/states/sleepy",  "fps": 30, "loop": true,  "motion": $(emit_motion "$SLEEP_M") }
  }
}
JSON

cat > "$OUT/LICENSE" <<TXT
$MODEL character model © Live2D Inc.
Distributed under the Live2D Free Material License.
See https://www.live2d.com/en/download/sample-data/ for current terms.
TXT

cat > "$OUT/README.md" <<MD
# $MODEL — Live2D Sample Character

Downloaded from Live2D/CubismWebSamples. Model © Live2D Inc., Free Material License.

Regenerate:
\`\`\`bash
./scripts/fetch-live2d-sample.sh $MODEL $ID
\`\`\`
MD

echo "✓ done. $MODEL installed at $OUT (motion: idle=$IDLE_M, happy=$HAPPY_M)"
echo "  Switch active in ~/.shikigami/config.json or via tray."
