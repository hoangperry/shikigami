# Claude (Opus) — Round 2 Analysis

## Round 2 verdict matrix update

| Issue | R1 | R2 (post-cross-critique) |
|---|---|---|
| #29 Win overlay | 1 DO-NOW vs 3 KEEP-OPEN | Gemini conceded KEEP-OPEN, Codex flipped to DO-NOW (UTM smoke) → SPLIT compromise emerged |
| #33 Cursor | 3 DO-NOW vs 1 wrong-fact CLOSE | Gemini retracted on verified docs → **4/4 DO-NOW with 5-event minimal scope** |
| #34 Windsurf | Split 2-2 | Gemini + Codex both proposed SPLIT → **4/4 SPLIT** when combined with Sonnet's R1 split proposal |

## My final positions

**#29 — SPLIT**
Codex's R2 push for a UTM smoke-test ticket changed my mind. The 80/20 here is: virtualised DWM catches Win32 windowing/DPI/topmost/passthrough mechanics (everything except the GPU-driver-touching transparency layer). A UTM smoke test is **honest scope** — explicitly "does not claim hardware coverage". Real hardware validation stays as a separate ticket. This is the same pattern we adopted for `passthrough.rs` doc clarity — code can be cross-platform-correct on principle, runtime verification is its own track.

**#33 — DO-NOW, 5-event minimal scope**
Unanimous after fact-check. Map only `sessionStart`, `preToolUse`, `postToolUse`, `postToolUseFailure`, `stop` — the events Cursor inherits from the same model as Claude Code / Codex. Skip the 13+ Cursor-specific events (`afterMCPExecution`, `preCompact`, `afterAgentThought`, etc) until real Cursor users report which ones matter. Tolerant transformer + log-then-skip unknowns.

**#34 — SPLIT**
Two child issues:
1. **(DO-NOW today)** Write `hooks/shikigami-windsurf-dumper.py` + contributor-facing README section: "If you use Windsurf, paste this snippet in your hooks config, then upload the resulting JSON to issue #34". I can ship the dumper script.
2. **(KEEP-OPEN)** Implement the actual bridge once a contributor uploads a real payload sample.

This converts the "no Windsurf license" objection from a hard blocker to a 30-min asset I produce today + an asynchronous contributor task.

## Round 3 unnecessary

We have convergence. All 4 participants (after the Sonnet R1 split proposal cascaded through R2) agree on:
- #30, #31, #35 → already converged in R1
- #33 → 4/4 after Gemini's fact-correction
- #34 → 4/4 split pattern
- #29 → 3 DO-NOW (UTM smoke ticket) + 1 mild-skeptical (me), with everyone agreeing the hardware-validation half stays open

Running Round 3 would burn ~$0.20 + 5 minutes of CLI calls to confirm what Round 2 already shows. The user's "engineering time" priority says to stop and synthesise.

## Action plan I'll execute

1. **#30** → Close, link `docs/WINDOWS-SIGNING.md` as the resolution
2. **#33** → Implement minimal 5-event Cursor bridge (hooks/shikigami-hook.py extends with `--source cursor` + Cursor-specific transform), close when shipped
3. **#34** → Write `hooks/shikigami-windsurf-dumper.py` + comment on issue with contributor-action prompt; keep issue open
4. **#29** → Write `docs/WINDOWS-UTM-SMOKE-TEST.md` runbook for contributors with Apple Silicon Macs to spin up a UTM Windows-on-ARM VM; comment on issue; keep open for the hardware portion
5. **#31** → No action; keep open
6. **#35** → No action; keep open

This converts 6 deferred issues into 2 closed (#30, #33) + 2 advanced-with-contributor-asset (#29, #34) + 2 truly-blocked-keep-open (#31, #35).
