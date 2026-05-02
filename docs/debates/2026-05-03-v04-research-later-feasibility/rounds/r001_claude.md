# Claude (Opus) — Round 1 Analysis

## My role
I'm the one who triaged these into the milestone in the first place. So this analysis is partially self-critique. I'll be honest about where I was overcautious vs. where the deferral is genuinely justified.

## Per-issue verdicts

**#29 Windows transparent overlay verification** — **KEEP-OPEN**
The CI matrix already confirms the code compiles + tests pass on windows-latest. The actual verification (DWM transparency, alwaysOnTop vs Snap, click-through) requires looking at a running window — no automated test substitutes for human eyeballs on real hardware. UTM Windows-on-ARM (Gemini's suggestion) is a creative path but introduces a virtualisation confound: VM compositors don't always behave like bare-metal DWM, so a "works in UTM" signal is weak evidence the binary works on a contributor's actual Lenovo. Engineering time spent chasing a UTM setup that doesn't reliably reproduce hardware behaviour is exactly the time-sink the user wants to avoid.

**#30 Windows EV cert** — **CLOSE-WITH-DOC**
Unanimous so far. There's no dev-side action; the runbook (`docs/WINDOWS-SIGNING.md`) IS the resolution. Closing with that doc attached as the rationale removes noise from the issue tracker without losing the procurement decision tree.

**#31 Linux transparent overlay verification** — **KEEP-OPEN**
Same reasoning as #29 with one extra dimension: the X11 vs Wayland fragmentation makes it more important to track explicitly. Linux runtime behaviour varies enough between compositors (Mutter / KWin / sway / Xfwm) that even one contributor's verification only covers their stack. KEEP-OPEN as a perpetual platform-readiness tracker.

**#33 Cursor adapter** — **DO-NOW** (revising my prior call)
Initial defer was on "config location unresolved". Codex correctly observes the cursor.com/docs/hooks page documents the hook system today (HTTP 200 just verified — Gemini's "docs don't exist" claim is factually wrong). The implementation pattern is identical to Codex CLI which I just shipped: tolerant transform with key-rename pass, ignore unmapped events, log unknowns. ~150 LOC. The only real unknown is the install file location, which we can document for manual paste like we did for Codex CLI's TOML config. **This was overcautious — should ship.**

**#34 Windsurf adapter** — **KEEP-OPEN**
Gemini's suggestion (write a no-op hook to dump the payload, reverse-engineer) is sound but requires a real Windsurf install + license. I don't have one. The pre-work is small *for someone who uses Windsurf* — but that's the user, not me. Keep-open with a clear next-step ("first contributor with a Windsurf license, run the dump script") is the honest framing.

**#35 Copilot Chat** — **KEEP-OPEN**
Building against a Preview API guarantees rework. The cost (1 week of VSCode extension work + marketplace publishing) is non-trivial and the API will change. Wait-for-GA is genuinely the right call. Three-way agreement here.

## Most-likely-wrong call (self-critique)

I was wrong about **#33 Cursor**. The "config location unresolved" objection treated docs incompleteness as a hard blocker when it's actually a "manual install snippet" tier of friction — exactly what we accepted for Codex CLI. The hook payload differs from Claude Code's but the pattern (tolerant transformer + log-then-skip unknowns) handles that. I deferred a 2-3 hour task to v0.4 because I anchored on the survey's "unresolved" tag instead of comparing to the Codex pattern I just shipped.

The Cursor bridge should ship in this same wave. Closing the gap between Claude Code, Codex, and Cursor in one sprint covers ~80% of the AI-coding market today.

## Cross-critique focus for Round 2

- **Gemini's UTM-VM suggestion for #29**: is virtualised DWM a sufficient proxy for hardware DWM? Skeptical, but worth pressure-testing.
- **Gemini's "Cursor docs don't exist" claim**: factually wrong (verified HTTP 200). Should be retracted.
- **Codex's #33 DO-NOW**: agreed, but want to scope tightly — ~5 most common events, not all 18+.
- **Sonnet's perspective** (pending): especially want a builder's view on whether #33 is really 2-3 hours or 1-2 days.
