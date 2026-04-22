# ADR-001: Event Transport — Local HTTP with Auth Token

**Status**: ✅ Accepted
**Date**: 2026-04-22
**Deciders**: @hoangperry
**Informed by**: Codex adversarial review (FR-001 WebSocket/POST mismatch)

## Context

Shikigami needs to receive events from the Claude Code hook system (and eventually other AI tools). PRD v0.1 specified:

> FR-001: "WebSocket server on localhost. Accepts POST events on configurable port."

The adversarial review flagged this as internally inconsistent: **a WebSocket server does not accept HTTP POST**. Additionally:

- Localhost listener without auth is a local attack surface (any process on the machine can inject fake state).
- Port conflicts, corporate firewalls, and antivirus can block localhost bindings.
- Hook integration should be trivial (a `curl` one-liner) — WebSocket requires more infrastructure in the hook.

## Decision

Use **local HTTP/1.1 POST** as the transport for v1:

- **Endpoint**: `http://127.0.0.1:<port>/v1/events` — always bound to loopback, never exposed on external interfaces.
- **Default port**: `7796`, configurable via settings or `SHIKIGAMI_PORT` env var.
- **Auth**: each install generates a random bearer token in `~/.shikigami/config.json`. Hook scripts read the token and send it as `Authorization: Bearer <token>`. Requests without a valid token are rejected with 401.
- **Content type**: `application/json`. Payload schema versioned.
- **Port conflict recovery**: on bind failure, app scans next 10 ports and writes chosen port to config. Hook script reads the chosen port on each send (or listens for config file changes).
- **No WebSocket in v1.** The in-app UI communicates with the Tauri backend via IPC, not WebSocket.

### Example Hook Invocation

```bash
curl -s -X POST "http://127.0.0.1:7796/v1/events" \
  -H "Authorization: Bearer $(cat ~/.shikigami/token)" \
  -H "Content-Type: application/json" \
  -d '{"source":"claude-code","type":"tool_complete","tool":"Bash","exitCode":0,"durationMs":1234}'
```

### Payload Schema v1

```json
{
  "schemaVersion": "1.0",
  "source": "claude-code",
  "type": "tool_start|tool_complete|error|session_start|session_end|user_prompt|assistant_message",
  "tool": "Bash|Read|Edit|...",
  "exitCode": 0,
  "durationMs": 1234,
  "severity": "info|warning|error|critical",
  "text": "<optional raw text for fallback parsing>",
  "metadata": {}
}
```

## Consequences

### Positive

- Trivial integration: a `curl` one-liner works from any hook system (Bash, PowerShell, Node, Python).
- Port-conflict recovery gives real deployability.
- Bearer token mitigates local attack surface.
- Standard HTTP is debuggable with curl/browser dev tools.
- Matches event-driven nature (discrete events, not streaming).

### Negative

- Slightly higher overhead per event than a persistent WebSocket connection — acceptable for the low event rates expected (dozens per session, not thousands).
- Hook scripts need to read token file. We ship a helper script so hook authors don't deal with this manually.
- Token rotation strategy TBD for v2.

### Neutral

- If real-time streaming becomes a requirement later (e.g., for live lip-sync), we can add a WebSocket endpoint alongside the HTTP one without breaking existing hooks. Current design does not preclude it.

## Alternatives Considered

1. **Persistent WebSocket connection**
   - ❌ Rejected: overkill for discrete events, harder to integrate from shell hooks, more failure modes (reconnect logic).

2. **Unix domain socket (mac/linux) + named pipe (win)**
   - ❌ Rejected for v1: less universally familiar, more platform-specific bugs, harder for third-party tool adapters. Revisit in v2 as an optimization or as a *second* transport.

3. **gRPC**
   - ❌ Rejected: overengineered for single-host local IPC, schema tooling burden, harder to curl.

4. **No transport — poll log files**
   - ❌ Rejected: racy, OS-specific, can't capture ephemeral state.

## References

- Codex review — FR-001 contradiction
- `docs/reviews/PRD-v0.1-adversarial-review.md`
