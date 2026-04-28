#!/usr/bin/env bash
# fetch-fili-vtuber.sh — install FILI, a CC0-licensed Filipino VTuber
# Live2D model from MjGjVtube/free-filipino-vtuber-template.
#
# License: CC0 1.0 (public domain dedication) — free to use anywhere.
# Source: https://github.com/MjGjVtube/free-filipino-vtuber-template
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ID="fili"
OUT="$ROOT/characters/$ID"
REPO="MjGjVtube/free-filipino-vtuber-template"
SUBDIR="Character 1_FILI_vts"

IDLE="$OUT/assets/states/idle"
echo "→ downloading FILI → $OUT"
rm -rf "$OUT"
mkdir -p "$IDLE"

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

fetch_recursive "$SUBDIR" "$IDLE"

# Find the actual model3.json and link it as frame_00.model3.json
ORIG=$(ls "$IDLE"/*.model3.json | head -1)
ln -sf "$(basename "$ORIG")" "$IDLE/frame_00.model3.json"

# Symlink other states to idle
cd "$OUT/assets/states"
for s in happy focused warning sleepy; do
  rm -rf "$s"
  ln -s idle "$s"
done
cd - >/dev/null

# Read motion groups for manifest
MOTIONS=$(python3 -c "
import json
d = json.load(open('$ORIG'))
m = (d.get('FileReferences', {}) or {}).get('Motions') or {}
print(','.join(sorted(m.keys())))
")
echo "  · motion groups: ${MOTIONS:-<none>}"

FIRST=$(echo "$MOTIONS" | cut -d, -f1)
emit_motion() {
  if [[ -z "$1" || "$1" == "null" ]]; then echo "null"; else echo "\"$1\""; fi
}

cat > "$OUT/manifest.json" <<JSON
{
  "\$schema": "https://shikigami.dev/schema/manifest/v1.0.json",
  "schemaVersion": "1.0",
  "id": "$ID",
  "name": "FILI (CC0 Filipino VTuber)",
  "description": "CC0 community VTuber model. Free for any use.",
  "author": "MjGjVtube",
  "version": "1.0.0",
  "license": "CC0-1.0",
  "tags": ["live2d", "vtuber", "community", "cc0", "sfw"],
  "renderer": "live2d",
  "defaultState": "idle",
  "states": {
    "idle":    { "path": "assets/states/idle",    "fps": 30, "loop": true,  "motion": $(emit_motion "$FIRST") },
    "happy":   { "path": "assets/states/happy",   "fps": 30, "loop": false, "then": "idle", "durationMs": 2000, "motion": $(emit_motion "$FIRST") },
    "focused": { "path": "assets/states/focused", "fps": 30, "loop": true,  "motion": $(emit_motion "$FIRST") },
    "warning": { "path": "assets/states/warning", "fps": 30, "loop": true,  "motion": $(emit_motion "$FIRST") },
    "sleepy":  { "path": "assets/states/sleepy",  "fps": 30, "loop": true,  "motion": $(emit_motion "$FIRST") }
  }
}
JSON

cat > "$OUT/LICENSE" <<TXT
FILI VTuber model © MjGjVtube, released under CC0 1.0 Universal
(public domain dedication). See:
https://github.com/MjGjVtube/free-filipino-vtuber-template/blob/main/LICENSE
TXT

echo "✓ done. FILI installed at $OUT"
