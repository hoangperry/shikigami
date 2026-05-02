# Debate: Are the 6 v0.4-Research-Later issues truly infeasible?

**Debate ID**: 2026-05-03-v04-research-later-feasibility
**Rounds**: 3 (thorough, cross-critique)
**Participants**: 🟡 Gemini · 🔴 Codex · 🟠 Sonnet · 🐙 Claude/Opus
**Started**: 2026-05-03

## Question

Six issues were moved to the `v0.4 Research Later` GitHub milestone with the rationale "blocked on external resources (hardware, paid cert, upstream API GA, schema verification)". Audit each one: is the infeasibility claim solid, or did Claude (Opus) miss a creative path that could ship?

For each issue, output one of:
- **CLOSE-WITH-DOC** — truly infeasible from the current dev environment; should be closed with the research doc attached as the rationale
- **KEEP-OPEN** — blocked but worth tracking; revisit when blocker clears
- **DO-NOW** — actually feasible right now; Claude missed a path; ship it

## User constraints

- **Goal**: Audit infeasibility claims (stress-test prior triage)
- **Evaluation mode**: Cross-critique (each round sees the others' positions)
- **Priority factor**: Engineering time (minimise hours on tasks that won't ship)

## Issues under audit

## #29 Phase 4: Verify Windows transparent overlay on real hardware

**Blocked from macOS.** Phase 4 scaffolding (commit 77a5e77) lays Tauri MSI/NSIS bundle config, CI windows-latest matrix, and the PowerShell hook wrapper, but the transparent always-on-top window is not yet verified on real Windows hardware.

## What needs verification

Tauri 2 on Windows uses WebView2 with DWM composition for transparency. Known quirks:

- Transparent windows can render with a **black or opaque background** if DWM is disabled or Visual Effects -> "Show shadows under windows" is off.
- Per-monitor DPI awareness: window may shrink or stretch on multi-monitor setups with mixed scaling.
- alwaysOnTop interaction with the Windows 11 Snap Layouts overlay.
- Click-through (passthrough.rs) currently uses CGEventTap semantics; Windows equivalent is WS_EX_TRANSPARENT + WS_EX_LAYERED, not yet wired in.

## Concrete tasks

- [ ] Build .msi from CI artifact on clean Windows 10 + Windows 11 VM
- [ ] Confirm transparent background renders correctly with default DWM settings
- [ ] Confirm transparent background degrades acceptably with minimal Visual Effects
- [ ] Verify alwaysOnTop respects fullscreen apps
- [ ] Test click-through with current Rust code
- [ ] If passthrough.rs is mac-specific, port to Windows SetWindowLongPtr + WS_EX_TRANSPARENT

## Hardware needed

- Windows 10 22H2 (oldest supported)
- Windows 11 23H2+
- Multi-monitor preferred (mixed DPI)

Tracks Phase 4 / v0.2.

## #30 Phase 4: Windows code-signing — EV cert procurement + CI integration

The Phase 4 scaffolding ships **unsigned** Windows binaries because we do not yet have an EV (Extended Validation) code-signing certificate. SmartScreen warns on every fresh install, and Windows Defender SmartScreen reputation does not build for unsigned binaries.

## Cost reality

- **Standard code-signing cert**: ~\\$60-120/yr (Sectigo, DigiCert). Builds reputation gradually; SmartScreen warning persists for weeks until enough installs accumulate.
- **EV code-signing cert**: ~\\$300-400/yr. SmartScreen reputation is **immediate**. Required for kernel drivers (we don't ship one) and for instant trust on enterprise networks. Comes on a hardware token (USB) or requires HSM service from the issuer.

For an alpha repo we can defer indefinitely; users tolerate SmartScreen on alpha software.

## When to revisit

- After v0.2.0 stable release with a non-trivial install base
- When a contributor or maintainer owns a Windows-distributed product and already has an EV setup we can borrow signing cycles from
- If Microsoft Store distribution is targeted (different signing flow)

## CI integration when it happens

Tauri reads ``CERTIFICATE_THUMBPRINT`` from the windows config block; on CI the cert is imported into the runner via signtool. The release.yml job already has a placeholder thumbprint field; flipping it on requires:

1. Cert import step (analogous to the Apple cert import block)
2. ``CERTIFICATE_THUMBPRINT`` set to the imported cert's thumbprint
3. Tauri config ``windows.certificateThumbprint`` matched to the secret

## Out of scope

This issue is **research / planning only** — no code changes until the cert exists. Reopen when revisiting.

## #31 Phase 5: Verify Linux transparent overlay on real hardware

**Blocked from macOS.** Phase 5 scaffolding (commit abc4e5f) lays Tauri's deb/rpm/AppImage bundle config, GTK/WebKit2GTK build deps in the Linux release job, and unverified-but-compile-clean transparent-window settings. End-to-end overlay behaviour is untested on real Linux desktops.

## What needs verification

Tauri 2 on Linux uses **GTK3 + WebKit2GTK** for the window. Transparent + always-on-top semantics depend on the compositor, not Tauri:

- **X11 + compositing WM** (Xfwm, Mutter, KWin, picom on bare i3/sway) — should work; Tauri's transparent window is well-documented here
- **Wayland** — patchier:
  - Older GNOME (Mutter pre-44) doesn't expose layer-shell or always-on-top via standard protocols
  - Sway needs wlroots layer-shell extension; Tauri may or may not request it
  - KDE Plasma 5.27+ Wayland is generally fine
- **Click-through** (\`set_ignore_cursor_events\`) requires the compositor to honour input-shape regions. X11 supports this universally; Wayland requires \`wp_input_method\` or compositor-specific protocols.

## Concrete tasks

- [ ] Build .AppImage from CI release artifact, run on Ubuntu 22.04 + 24.04 (X11 + Wayland sessions)
- [ ] Build + install .deb on Ubuntu 24.04, .rpm on Fedora 40
- [ ] Confirm transparent background on X11 with default compositor
- [ ] Confirm transparent background on Wayland — record which compositors work, which fall back
- [ ] Verify \`alwaysOnTop\` semantics across Xfwm / Mutter / KWin / sway
- [ ] Test click-through end-to-end on at least X11 + KDE Wayland

## Hardware needed

- Ubuntu 22.04 LTS (X11 default)
- Ubuntu 24.04 LTS (Wayland default)
- Fedora 40 / 41 (KDE Plasma Wayland or GNOME Wayland)
- Optional: Arch + sway for the wlroots case

Tracks Phase 5 / v0.3.

## #33 v0.4 adapter: Cursor hook bridge (rich event surface)

**Source:** Adapter survey at \`plans/reports/researcher-260502-0814-ai-tool-adapter-survey.md\`.

Cursor v1.7+ exposes a documented hook system at https://cursor.com/docs/hooks with the **richest event surface of any tool surveyed** — 18+ agent events including \`preToolUse\`, \`postToolUseFailure\`, \`afterFileEdit\`, \`beforeShellExecution\`, \`afterMCPExecution\`. More granular than Claude Code, opens up more reactive states.

## Approach

1. Build \`hooks/shikigami-cursor-hook.py\` parallel to \`shikigami-hook.py\`. Stdin format is JSON; mostly compatible with EventPayload after a key-rename pass
2. Extend \`scripts/install-hook.py\` with a \`--target=cursor\` mode that writes Cursor's hook config file (location confirmed in survey)
3. Map Cursor's richer event taxonomy → existing DominantState enum. Some events (e.g. \`preCompact\`) won't map to a useful state and should be silently ignored
4. README: add \"Installing Cursor hooks\" section parallel to Claude Code

## Effort estimate

**Medium** — 3-5 days. New install path + payload transform per Cursor's payload schema (some keys differ from Claude Code).

## Open questions

- Cursor hook config file location (~/.cursor/settings.json? per-project .cursor/?). Survey didn't fully resolve — confirm before starting
- Whether Cursor hooks fire for **inline Tab completions** as well as Cmd+K agent — if yes, we'd want to gate them via settings to avoid event-storm

## References

- [Cursor hooks docs](https://cursor.com/docs/hooks)
- Survey §1 for full event list

Tracks v0.4 adapter expansion.

## #34 v0.4 adapter: Windsurf Cascade Hooks bridge (schema verification needed)

**Source:** Adapter survey at \`plans/reports/researcher-260502-0814-ai-tool-adapter-survey.md\`.

Windsurf (formerly Codeium) ships **Cascade Hooks** — confirmed exists, payload schema partial. Survey couldn't fully verify the JSON shape from public docs alone. Blocked on confirming the hook payload format before we can write a bridge.

## Approach (when unblocked)

Same pattern as Cursor — separate \`hooks/shikigami-windsurf-hook.py\`, install path via \`install-hook.py --target=windsurf\`, payload transform.

## Pre-work

- [ ] Manually install Windsurf, register a no-op hook script that dumps stdin to disk, capture a few real payloads
- [ ] Document the schema in \`docs/adapter-windsurf-payload.md\`
- [ ] Then file a follow-up implementation issue

## Effort estimate

**Medium-High** until schema is confirmed; then drops to Medium.

## References

- [Windsurf Cascade Hooks docs](https://docs.windsurf.com/docs/agent/cascade-hooks)
- Survey §3

Tracks v0.4 adapter expansion. **Do not start implementation until pre-work above is complete.**

## #35 v0.4 adapter: GitHub Copilot Chat (wait for hooks API GA)

**Source:** Adapter survey at \`plans/reports/researcher-260502-0814-ai-tool-adapter-survey.md\`.

GitHub Copilot Chat in VSCode v1.110+ ships an **agent hooks API** documented at https://code.visualstudio.com/docs/copilot/customization/hooks — currently in **Preview**. Survey recommends waiting for GA before investing in the bridge to avoid rewrites when the API changes.

## Approach (when unblocked)

VSCode-extension based — different shape from the CLI bridges. Need to:

1. Create a new repo or directory \`vscode-shikigami/\` with a small VSCode extension
2. Subscribe to the agent hooks API events
3. POST EventPayload to the same \`http://127.0.0.1:7796/v1/events\` endpoint
4. Distribute via VSCode marketplace

This gives us coverage of any VSCode-based AI tool (Copilot Chat, Continue.dev when it ships, future entrants) through one extension.

## Pre-conditions

- [ ] Hooks API moves from Preview → Stable (track [VSCode release notes](https://code.visualstudio.com/updates))
- [ ] API surface stops changing materially between releases for ~2 minor versions
- [ ] If Continue.dev adopts the same API, prioritise sooner

## Effort estimate

**Medium** when ready — ~1 week including marketplace publishing.

## References

- [VSCode agent hooks docs](https://code.visualstudio.com/docs/copilot/customization/hooks)
- Survey §6

Tracks v0.4 adapter expansion. **Defer until API hits stable.**

