# Round 1 — Sonnet (Pragmatic Implementer)

**Role**: The person who'd actually build these. Cares about what ships, hidden gotchas, 2am debug sessions.

---

## #29 — Verify Windows transparent overlay on real hardware

**KEEP-OPEN**

The code reality here is better than the issue implies: `passthrough.rs` already uses Tauri's cross-platform `set_ignore_cursor_events()` abstraction with zero `cfg(target_os)` branches — it's not mac-specific. The concern in the issue about "CGEventTap semantics" is already solved in the implementation. What's genuinely unverifiable without hardware is whether DWM actually renders transparency correctly, and whether WebView2 + DWM composition holds up when Visual Effects are dialed down. A UTM/Parallels Windows ARM VM on Mac can get you 80% of this (DWM composition, rendering) but cannot prove multi-monitor DPI or Snap Layout edge cases. Worth keeping open as a validation gate before any public Windows release rather than closing as research-only.

---

## #30 — Windows code-signing EV cert

**CLOSE-WITH-DOC**

No creative path exists here. The blocker is literally a USD transaction and a hardware token in someone's hand. A standard $60-120/yr cert is technically viable and partially addresses SmartScreen, but the issue as written specifically frames EV as the target. For an alpha with a tiny install base, unsigned + SmartScreen click-through is the honest UX. Document the CI wiring (already mapped out in the issue) and close. The placeholder thumbprint field in release.yml means the mechanical work is already done — this is purely a procurement gate.

---

## #31 — Verify Linux transparent overlay on real hardware

**KEEP-OPEN**

This one is more legitimately blocked than #29. Linux compositing is not "run a VM and you're done" — Wayland protocol support depends on which compositor, which version, and what session type, and GPU passthrough in VMs is flaky enough that you can't trust a green result. The X11 path is probably fine (Tauri's docs say so), but Wayland click-through via `set_ignore_cursor_events` is the genuine unknown. No creative path exists that gives trustworthy results without real hardware running real Wayland sessions. Keep open, do not close, wait for a Linux contributor or cheap VPS with GPU.

---

## #33 — Cursor hook bridge

**DO-NOW**

This is the clearest miss in the triage. The implementation plan is already written in the issue itself. The existing `shikigami-hook.py` + `install-hook.py` is the template — 95% of what needs to exist is already there from the Claude Code + Codex bridges. Cursor hooks are JSON on stdin, documented at cursor.com/docs/hooks, with a richer event surface that maps naturally onto the existing `DominantState` enum (most new events collapse to existing states, unknowns get silently ignored). The two open questions (config file location, Tab completion event storm) are resolvable in a single session with Cursor installed locally — that's not a blocker, it's 20 minutes of local testing. This ships in a day, not 3-5 days, because Codex bridge reuse is near-total. Waiting for "survey certainty" when you can just open Cursor and run the hook yourself is exactly the kind of analysis-paralysis that burns engineering time.

---

## #34 — Windsurf Cascade Hooks bridge

**DO-NOW**

The issue literally contains the unblocking path in its own pre-work checklist: install Windsurf, register a no-op script that dumps stdin to disk, capture payloads. That is 30 minutes of work if you have Windsurf installed (it's free). The "blocked on schema verification" framing treats an afternoon of empirical testing as if it were a multi-month upstream dependency. The payload capture produces the schema doc directly, after which the bridge implementation is identical to the Cursor pattern. This should not be in Research Later — it should be split into two issues: a "capture Windsurf payload schema" task (DO-NOW, 1-2 hours) and a "implement bridge" follow-up. Deferring both because the second depends on the first is lazy triage.

---

## #35 — GitHub Copilot Chat hooks API

**KEEP-OPEN**

The Preview → GA concern is legitimate in a way #33 and #34 are not. VSCode agent hooks API breaking changes between preview releases would mean rewriting the extension, not just adjusting a JSON key mapping. Additionally, this requires building and distributing a VSCode marketplace extension — a distinct engineering surface (TypeScript extension API, packaging, publisher account, review process) compared to the Python hook scripts. The upside of waiting is real: if Continue.dev or another tool adopts the same VSCode hooks API, one extension covers multiple tools. Worth tracking, not worth starting yet.

---

## Most Wrongly Deferred: #33 (Cursor)

**#33 is the clearest engineering mistake in this triage.** Every other deferred issue has a genuine external blocker — hardware, money, an unstable upstream API, or VM-untrustworthy compositor behavior. Cursor blocks on none of these. The docs exist, the app is free to install on the dev machine, the implementation pattern is already proven twice (Claude Code, Codex), and the issue's own approach section is a complete implementation spec. The "survey didn't fully resolve config file location" open question is a `find ~/.cursor -name '*.json' | xargs grep -l hook` away from an answer. If this were a task estimate in a sprint, it's a 4-hour ticket that was tagged Research Later because the researcher wanted to be thorough instead of being done. Ship it.
