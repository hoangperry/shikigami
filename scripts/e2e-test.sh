#!/usr/bin/env bash
# e2e-test.sh — end-to-end HTTP test against a running Shikigami app.
#
# Run:
#   pnpm tauri dev   # in another terminal
#   ./scripts/e2e-test.sh
#
# Exit 0 on all-pass, 1 on any failure. Designed to be CI-friendly:
# zero deps beyond curl + jq + python3 (jq is standard on macOS via brew,
# python3 ships with macOS).
set -uo pipefail

# ── colours / counters ───────────────────────────────────────────────
GREEN=$'\033[32m'; RED=$'\033[31m'; YELLOW=$'\033[33m'; DIM=$'\033[2m'; NC=$'\033[0m'
PASS=0; FAIL=0; SKIP=0

pass() { printf "  ${GREEN}✓${NC} %s\n" "$1"; PASS=$((PASS+1)); }
fail() { printf "  ${RED}✗${NC} %s\n     ${DIM}%s${NC}\n" "$1" "$2"; FAIL=$((FAIL+1)); }
skip() { printf "  ${YELLOW}-${NC} %s ${DIM}(%s)${NC}\n" "$1" "$2"; SKIP=$((SKIP+1)); }
section() { printf "\n${YELLOW}── %s ──────────────────────────────${NC}\n" "$1"; }

# ── prerequisites ────────────────────────────────────────────────────
section "preflight"

HOME_DIR="${SHIKIGAMI_HOME:-$HOME/.shikigami}"
TOKEN_FILE="$HOME_DIR/token"
CONFIG_FILE="$HOME_DIR/config.json"

if [[ ! -r "$TOKEN_FILE" ]]; then
  fail "token readable" "$TOKEN_FILE"; exit 1
fi
TOKEN=$(cat "$TOKEN_FILE")
[[ ${#TOKEN} -eq 64 ]] && pass "token loaded (64 hex chars)" \
  || { fail "token format" "expected 64 hex chars, got ${#TOKEN}"; exit 1; }

PORT=$(python3 -c "import json; print(json.load(open('$CONFIG_FILE')).get('port',7796))" 2>/dev/null || echo 7796)
URL="http://127.0.0.1:$PORT"

# Server reachable?
if ! curl -s -o /dev/null --max-time 2 "$URL"; then
  fail "server reachable on $URL" "is Tauri running? try: pnpm tauri dev"
  exit 1
fi
pass "server reachable on $URL"

# ── auth ─────────────────────────────────────────────────────────────
section "auth"

# No bearer → 401
code=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$URL/v1/events" \
  -H "Content-Type: application/json" -d '{"schemaVersion":"1.0","source":"generic","type":"session_start"}')
[[ "$code" == "401" ]] && pass "401 without bearer" \
  || fail "401 without bearer" "got $code"

# Wrong bearer → 401
code=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$URL/v1/events" \
  -H "Authorization: Bearer wrong-token" -H "Content-Type: application/json" \
  -d '{"schemaVersion":"1.0","source":"generic","type":"session_start"}')
[[ "$code" == "401" ]] && pass "401 with wrong bearer" \
  || fail "401 with wrong bearer" "got $code"

# Helper: authenticated POST → expects $1 status
post() {
  local expected="$1"; shift
  local payload="$1"; shift
  local label="$1"; shift
  local path="${1:-/v1/events}"
  local code
  code=$(curl -s -o /tmp/shikigami-e2e.body -w "%{http_code}" -X POST "$URL$path" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -d "$payload")
  if [[ "$code" == "$expected" ]]; then
    pass "$label → $code"
  else
    fail "$label" "expected $expected got $code: $(cat /tmp/shikigami-e2e.body)"
  fi
}

# ── schema validation ───────────────────────────────────────────────
section "schema validation"

post 400 '{"foo":"bar"}' "rejects unknown shape"
post 400 '{"schemaVersion":"9.9","source":"generic","type":"session_start"}' "rejects bad schemaVersion"
post 400 '{"schemaVersion":"1.0","source":"generic","type":"not_a_real_type"}' "rejects unknown event type"

# ── event mapping (per-tool, severity, etc.) ────────────────────────
section "event mapping (positive)"

post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"session_start"}' "session_start"
post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"tool_start","tool":"Bash"}' "tool_start Bash → focused"
post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"tool_start","tool":"Task"}' "tool_start Task → confused"
post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"tool_start","tool":"WebFetch"}' "tool_start WebFetch → confused"
post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"tool_complete","tool":"Bash","exitCode":0}' "tool_complete Bash exit=0 → happy"
post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"tool_complete","tool":"Bash","exitCode":1}' "tool_complete Bash exit=1 → warning"
post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"tool_complete","tool":"Read","exitCode":0}' "tool_complete Read exit=0 → idle"
post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"git_commit","text":"finally fix it"}' "git_commit + finally text"
post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"destructive_op_detected","severity":"critical","text":"rm -rf /tmp"}' "destructive op critical"
post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"user_prompt","text":"hello"}' "user_prompt"

# Session-tagged events
post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"tool_start","tool":"Bash","sessionId":"e2e-test-session-1","cwd":"/tmp/repo-a"}' "session-tagged event #1"
post 202 '{"schemaVersion":"1.0","source":"claude-code","type":"tool_start","tool":"Bash","sessionId":"e2e-test-session-2","cwd":"/tmp/repo-b"}' "session-tagged event #2"

# ── /v1/say (TTS) ──────────────────────────────────────────────────
section "TTS /v1/say"

# Discover provider — skip body checks if disabled.
provider=$(python3 -c "import json; print(json.load(open('$CONFIG_FILE'))['tts']['provider'])" 2>/dev/null || echo none)

if [[ "$provider" == "none" || -z "$provider" ]]; then
  skip "/v1/say with provider=none returns 503" "TTS off in config — re-run after enabling"
  # Verify the disabled-path response code anyway:
  code=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$URL/v1/say" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -d '{"text":"hello"}')
  [[ "$code" == "503" ]] && pass "503 when TTS disabled" || fail "503 when TTS disabled" "got $code"
else
  # Empty text → 400
  post 400 '{"text":""}' "rejects empty text" "/v1/say"
  # Real synth call
  code=$(curl -s -o /tmp/shikigami-e2e.body -w "%{http_code}" -X POST "$URL/v1/say" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -d '{"text":"e2e test xin chao"}')
  if [[ "$code" == "200" ]]; then
    audio=$(python3 -c "import json,sys; print(json.load(open('/tmp/shikigami-e2e.body')).get('audio_url',''))" 2>/dev/null)
    if [[ -n "$audio" && -f "$audio" ]]; then
      pass "/v1/say synthesised → $(basename "$audio") ($(stat -f %z "$audio") bytes)"
    else
      fail "/v1/say audio file" "response had no audio_url or path missing: $(cat /tmp/shikigami-e2e.body)"
    fi
  else
    fail "/v1/say 200" "got $code: $(cat /tmp/shikigami-e2e.body)"
  fi
fi

# ── summary ─────────────────────────────────────────────────────────
section "summary"
total=$((PASS+FAIL+SKIP))
printf "  ${GREEN}%d passed${NC}, ${RED}%d failed${NC}, ${YELLOW}%d skipped${NC}  (of %d)\n" \
  "$PASS" "$FAIL" "$SKIP" "$total"

[[ "$FAIL" -eq 0 ]] && exit 0 || exit 1
