# Final Synthesis: Are 6 v0.4-Research-Later issues truly infeasible?

**Debate ID**: 2026-05-03-v04-research-later-feasibility
**Rounds completed**: 2 of planned 3 (early termination — see "Round 3 skipped" below)
**Participants**: 🟡 Gemini · 🔴 Codex · 🟠 Sonnet · 🐙 Claude (Opus)
**Resolution**: 2 issues to close · 2 to advance with contributor assets · 2 truly blocked

---

## Per-issue verdicts (final)

| Issue | Title | Verdict | Action |
|---|---|---|---|
| **#29** | Windows transparent overlay verification | **SPLIT** | UTM smoke-test runbook (do now) + hardware verify (keep open) |
| **#30** | Windows EV cert procurement | **CLOSE-WITH-DOC** | Close, attach `docs/WINDOWS-SIGNING.md` as resolution |
| **#31** | Linux transparent overlay verification | **KEEP-OPEN** | No action — genuinely needs hardware |
| **#33** | Cursor adapter | **DO-NOW** | Ship minimal 5-event bridge, close when shipped |
| **#34** | Windsurf adapter | **SPLIT** | Ship contributor payload-dumper (do now) + bridge impl (keep open) |
| **#35** | Copilot Chat adapter | **KEEP-OPEN** | No action — wait for VSCode hooks API GA |

## How positions evolved across rounds

```
                  R1 (independent)         R2 (cross-critique)
#29  Win overlay  G:DO-NOW  3xKEEP-OPEN    Gemini conceded; Codex flipped DO-NOW (UTM); SPLIT consensus
#30  EV cert      4x CLOSE                 Stable
#31  Linux        4x KEEP-OPEN             Stable
#33  Cursor       3 DO-NOW + Gemini        Gemini retracted (docs verified HTTP 200); 4/4 DO-NOW
                  CLOSE (wrong fact)
#34  Windsurf     2 DO-NOW + 2 KEEP-OPEN   All converged on SPLIT pattern
#35  Copilot      4x KEEP-OPEN             Stable
```

## Areas of agreement (all four participants)

- **Engineering time is the right priority filter** — issues that consume hours without shipping anything are anti-value
- **Truly external blockers are real** — EV cert ($300/yr), Windows hardware, Linux compositor variance, Preview API churn
- **Tolerant transformer pattern works** — adapter scope can be narrow (5 events) instead of wide (18+); ship + iterate
- **Contributor-facing assets convert blockers into asynchronous work** — a dumper script + clear instructions unblocks #34 without requiring the maintainer to install Windsurf

## Areas of remaining tension (acceptable)

- **#29 SPLIT** — Codex believes UTM smoke catches 80% of Win32 mechanics; Opus is mildly skeptical that virtualised DWM is a faithful transparency proxy. Compromise: ship the runbook explicitly scoped to "Win32 mechanics, not transparency rendering". Both halves of the issue tracked separately.

## Round 3 skipped — rationale

Round 2 produced strong convergence:
- 5 of 6 issues fully converged (4/4 agreement after fact-corrections + cross-critique)
- The remaining tension on #29 was resolved by adopting the same SPLIT pattern that worked for #34
- The user's primary constraint was **engineering time** — running Round 3 would consume 5+ minutes of CLI calls and ~$0.10 of tokens to confirm what Round 2 already showed
- All four participants explicitly stated their final positions in Round 2; further iteration would be ceremonial

This is documented as an explicit moderator decision, not a missed step.

## Most-wrongly-deferred (consensus across all 4 advisors)

**#33 Cursor adapter.** Gemini, Codex, Sonnet, and Opus independently identified this as the issue Claude was most overcautious about. The pattern (Claude Code → Codex → Cursor) was already proven twice in this codebase; deferring on "config location unresolved" was anchor-bias on the survey's caveat rather than honest engineering judgment.

## Action plan

Executed in the same session as this synthesis:

1. **#33 Cursor bridge** — implement minimal 5-event adapter (`sessionStart`, `preToolUse`, `postToolUse`, `postToolUseFailure`, `stop`). Extends existing `hooks/shikigami-hook.py` with `--source cursor` + Cursor-specific field-name transform. Close when shipped + CI green.

2. **#30 EV cert** — close with `docs/WINDOWS-SIGNING.md` link as resolution.

3. **#34 Windsurf** — ship `hooks/shikigami-windsurf-dumper.py` script + comment instructions on the issue. Issue stays open as "data needed from a contributor with Windsurf".

4. **#29 Win overlay** — write `docs/WINDOWS-UTM-SMOKE-TEST.md` runbook + comment on issue. Issue stays open for the hardware-validation half.

5. **#31, #35** — no action. Comments on each linking this synthesis as the rationale for keeping them deferred.

## Cost & quality summary

- **Total LLM calls**: 4 (2 Gemini + 2 Codex external) + 1 Sonnet agent + 2 Opus rounds = 7 distinct analyses
- **Cost estimate**: ~$0.15 (Gemini + Codex API) + Sonnet/Opus included in subscription
- **Quality**: All 4 advisors produced substantive verdicts; 1 fact error caught + corrected (Gemini's "Cursor docs don't exist" → HTTP 200 verified)
- **Convergence**: 5 of 6 issues 4/4 agreement; 6th has explicit SPLIT compromise
