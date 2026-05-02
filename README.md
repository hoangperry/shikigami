# 式神 · Shikigami

> A summoned desktop companion that reflects your AI agent's real-time state through a reactive 2D character, displayed as a transparent Picture-in-Picture overlay.

**Status**: `v0.1.0-alpha` · planning complete + event engine shipped · character renderer in progress
**Platforms**: macOS (Apple Silicon + Intel, primary) · Windows (alpha — unsigned, transparency unverified) · Linux (alpha — deb/rpm/AppImage, Wayland transparency unverified)
**Current integration**: Claude Code (primary) · Codex CLI (alpha, shared bridge) · Cursor (alpha, 5-event minimal) · Windsurf / Copilot Chat tracked in v0.4 milestone

---

## What It Does

Agentic AI coding sessions generate a lot of events — tool invocations, errors, commits, long-running builds — but the terminal gives you a wall of text and nothing else. Shikigami sits on your desktop as a small transparent window with an animated character who reacts in real time to what your agent is actually doing. A green build makes her smile; a rejected test makes her concerned; a stray `rm -rf` locks her into an alarm state before you press enter.

It is *not* a chatbot, a VTuber rig, or a voice companion. It is **visual proprioception for agentic workflows** — a status indicator with personality.

---

## Why It Exists

Long AI coding sessions are cognitively heavy and visually flat. The existing ecosystem splits into "serious" IDE status bars (functional, no presence) and "fun" desktop pets / VTuber rigs (presence, no agent awareness). Shikigami bridges them: a character grounded in structured events from your AI tool, not cosmetic text patterns.

The design is deliberately narrow so it can be correct. Read [`docs/PRD.md`](docs/PRD.md) for the full product rationale.

---

## Quick Start

```bash
# 1. Clone
git clone https://github.com/hoangperry/shikigami.git
cd shikigami

# 2. Install frontend deps
pnpm install

# 3. Run dev build (opens a transparent always-on-top window)
pnpm tauri:dev

# 4. In another terminal: register Claude Code hooks
python3 scripts/install-hook.py install

# 5. Use Claude Code normally — the character reacts to every tool call
```

Verify health:

```bash
python3 scripts/install-hook.py doctor
```

Test without Claude Code (manual event):

```bash
TOKEN=$(cat ~/.shikigami/token)
PORT=$(jq -r .port ~/.shikigami/config.json)
curl -X POST "http://127.0.0.1:$PORT/v1/events" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"schemaVersion":"1.0","source":"claude-code","type":"git_commit","text":"fix critical bug, finally"}'
# Character should animate into happy_relieved
```

Uninstall hooks:

```bash
python3 scripts/install-hook.py uninstall
```

### Codex CLI (alpha)

OpenAI's Codex CLI ships an identical hook delivery model to Claude
Code, so the same `hooks/shikigami-hook.py` script handles both — only
the EventPayload `source` field changes. Add this snippet to
`~/.codex/config.toml` (replace the absolute path with your clone):

```toml
[hooks]
PreToolUse       = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
PostToolUse      = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
UserPromptSubmit = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
Stop             = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
SessionStart     = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
PermissionRequest = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
```

`PermissionRequest` is Codex-only; it maps to a warning state so the
character signals "agent waiting on your approval" rather than looking
idle while a permission dialog is open. Auto-installer for the Codex
TOML config is tracked but deferred (KISS — manual paste is faster
than introducing a TOML write dependency).

### Cursor (alpha — 5-event minimal scope)

Cursor v1.7+ ships an [extensive hook system](https://cursor.com/docs/hooks)
with 18+ agent events. We map the **5 events that mirror Claude Code's
core lifecycle** today (`sessionStart`, `preToolUse`, `postToolUse`,
`postToolUseFailure`, `stop`); the remaining Cursor-specific events
(`afterMCPExecution`, `preCompact`, `afterAgentThought`, etc.) are
silently skipped until real users tell us which matter. Tolerant
transformer pattern — same playbook as Codex.

Add this to your Cursor hooks config (per project, `.cursor/hooks.json`,
or globally — consult Cursor's current docs for the file location):

```json
{
  "hooks": [
    { "event": "sessionStart",       "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source cursor" },
    { "event": "preToolUse",         "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source cursor" },
    { "event": "postToolUse",        "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source cursor" },
    { "event": "postToolUseFailure", "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source cursor" },
    { "event": "stop",               "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source cursor" }
  ]
}
```

Cursor field names differ slightly from Claude Code's (`conversation_id`
instead of `session_id`, `workspace_roots[0]` instead of `cwd`); the
script's `normalize_cursor()` rewrites them at the edge so the rest
of the pipeline stays source-agnostic.

---

## Installing the .dmg (end users)

The `Shikigami_*.dmg` produced by `pnpm tauri:build` is signed with an
**Apple Development** certificate (suitable for the original author's
test devices) but is **not** notarized through Apple's distribution
service. macOS Gatekeeper will block it on first launch with a
"unidentified developer" prompt.

Two safe ways past it on a fresh Mac:

**Option 1 — system Settings**
1. Double-click the .dmg, drag `Shikigami.app` into `/Applications`
2. Try to open the app — macOS will refuse
3. Open *System Settings → Privacy & Security* → scroll to the
   "Shikigami was blocked" notice → click **Open Anyway**
4. Confirm in the dialog that follows

**Option 2 — terminal (faster)**
```bash
xattr -cr /Applications/Shikigami.app
open /Applications/Shikigami.app
```
`xattr -cr` strips the `com.apple.quarantine` attribute set on every
file downloaded from the internet. The signature itself remains intact
— this only tells Gatekeeper that you trust the source.

A future release with full Apple Developer Program notarization will
remove this step. The signature you see today is genuine; just not yet
notarized:

```bash
codesign -dv /Applications/Shikigami.app
# Authority=Apple Development: Hoang Truong Nhat (Y44JV6U4Q9)
# Authority=Apple Worldwide Developer Relations Certification Authority
# Authority=Apple Root CA
```

---

## Installing on Windows (alpha)

The Windows build ships as **`.msi`** (system-wide, WiX) and **`.exe`**
(per-user, NSIS) installers from the same release tag as the macOS DMG.
At this milestone the Windows binary is **unsigned** — there is no
$400/yr EV cert backing it yet — so SmartScreen will warn on first
launch.

```powershell
# 1. Download Shikigami_*_x64-setup.exe (NSIS) or .msi from the Release page
# 2. Run it. SmartScreen warning → click "More info" → "Run anyway".
# 3. Install Python 3 if you don't have it (Claude Code already requires it)
# 4. Register the hook (uses python on PATH, not python3):
python scripts\install-hook.py install

# 5. Verify
python scripts\install-hook.py doctor
```

⚠️ **Known gaps in the v0.1 alpha Windows build** (tracked in GitHub
Issues — contributions very welcome):
- Transparent always-on-top overlay is **untested on real Windows
  hardware** — Tauri uses WebView2 which has documented quirks around
  DWM composition and per-monitor DPI. May render with a black or
  opaque background until a contributor verifies + tunes.
- No code signing → SmartScreen warning on every fresh install.
- Hook bridge uses the same `shikigami-hook.py` as macOS; a
  PowerShell-native wrapper (`hooks/shikigami-hook.ps1`) ships
  alongside it for users who prefer registering it directly.

For now Windows is best-effort: macOS is the supported alpha target.

---

## Installing on Linux (alpha)

Linux ships three formats from the same release tag — pick whatever
your distro prefers:

```bash
# Debian / Ubuntu
sudo dpkg -i shikigami_*_amd64.deb
sudo apt-get install -f          # pulls in any missing GTK / WebKit deps

# Fedora / RHEL / openSUSE
sudo rpm -i shikigami-*.x86_64.rpm

# Distro-agnostic (no install needed)
chmod +x shikigami_*_amd64.AppImage
./shikigami_*_amd64.AppImage
```

Hook setup is identical to macOS:

```bash
python3 scripts/install-hook.py install
python3 scripts/install-hook.py doctor
```

⚠️ **Known gaps in the v0.1 alpha Linux build** (contributions welcome):

- Transparent always-on-top overlay is **untested on real Linux
  desktops**. X11 with a compositor (Xfwm, Mutter, KWin) is expected
  to work; **Wayland needs verification** — some compositors (older
  GNOME, sway without the right protocols) lack the surface APIs
  Tauri relies on.
- `alwaysOnTop` semantics differ across compositors; behaviour may
  degrade to a regular "above" hint instead of a true topmost layer.
- Click-through requires compositor input-shape support. Tauri's
  abstraction works on X11 + most Wayland compositors, but no
  hardware run has confirmed it for Shikigami yet.

Linux is best-effort at this milestone alongside Windows; macOS
remains the supported alpha target.

---

## Building a signed .dmg yourself

If you have your own Apple signing identity (`security find-identity -v
-p codesigning`), pass it via env:

```bash
APPLE_SIGNING_IDENTITY="Apple Development: <your name> (<team-id>)" \
  pnpm tauri:build
```

For real distribution you need a **Developer ID Application** cert
(Apple Developer Program, $99/year) plus notarization credentials —
see [Tauri's macOS distribution guide](https://v2.tauri.app/distribute/sign/macos/).

---

## How It Works

Events flow through a seven-stage pipeline:

```
Hook → Bridge → Ingest → Segment → Resolve → Emit → Render
 CC     Py      Rust     Rust      Rust      Rust    React+PixiJS
```

- **Bridge** (`hooks/shikigami-hook.py`) transforms Claude Code hook JSON into a typed `EventPayload`
- **Ingest** (`src-tauri/src/event/server.rs`) receives HTTP POST on `127.0.0.1` with bearer auth
- **Segment** (`src-tauri/src/state/dampen.rs`) dedups repeated events in a 2-second sliding window
- **Resolve** (`src-tauri/src/state/machine.rs`) applies Hierarchical Fusion: events drive the dominant state, text modifiers layer on textures, severity scales duration
- **Emit** fires a `state_changed` Tauri event to the frontend
- **Render** (Phase 2) paints the sprite via PixiJS

Full details in [`docs/PIPELINE.md`](docs/PIPELINE.md) · architectural decisions in [`docs/adr/`](docs/adr/).

---

## Features

- 🪶 Lightweight: built on **Tauri 2** (<80 MB RAM idle target)
- 🧠 **Event-driven state**: reactions map to what your agent actually does (tool calls, exit codes, git ops), not prompt-engineered text patterns
- 🛡️ **Severity-aware**: destructive operations like `rm -rf`, `DROP TABLE`, `git push --force` lock the character into a critical warning state with texture suppression
- 🎨 **Dual-layer emotion system**: 9 dominant states × 6 texture modifiers = expressive animation keys like `happy_relieved` or `focused_alarmed`
- 🔌 **Extensible character format**: `.shikigami` zip packages, portable across OSes
- 🔒 **100 % local**: no telemetry, no cloud, no proprietary deps (core)
- 🔁 **Toxic-loop-safe**: dampener prevents strobing when errors repeat

---

## Documentation

| Doc | What it covers |
|-----|----------------|
| [`docs/PRD.md`](docs/PRD.md) | Product requirements v0.2 (post-review) |
| [`docs/TDD.md`](docs/TDD.md) | Technical design mapping PRD to code |
| [`docs/PIPELINE.md`](docs/PIPELINE.md) | Seven-stage data flow narrative |
| [`docs/CHARACTER-PLAYBOOK.md`](docs/CHARACTER-PLAYBOOK.md) | Character production guide + commission strategy |
| [`docs/codex-ui-prompts.md`](docs/codex-ui-prompts.md) | Copy-paste prompts for GPT image-gen |
| [`docs/adr/`](docs/adr/) | Five architecture decision records |
| [`docs/reviews/`](docs/reviews/) | Adversarial-review audit trail |
| [`docs/debates/`](docs/debates/) | Multi-AI tournament decisions |
| [`docs/research/`](docs/research/) | External-repo reusability analyses |

---

## Character Packages

Characters ship as `.shikigami` zip bundles containing a manifest, sprite frames, preview, and license. See [`docs/adr/003-character-package-format.md`](docs/adr/003-character-package-format.md).

### Default Character

| Package | Purpose | License |
|---------|---------|---------|
| `characters/linh-pixel/` | Procedural 8-bit dev fixture | MIT |
| `characters/linh/` | Production Linh (in progress, anime/vector) | CC-BY-SA-4.0 on ship |

The pixel fixture exists so engineering can proceed while the production character is in commission. See `characters/linh-pixel/README.md` for details.

### Make Your Own

A template repo and `shikigami pack` CLI are planned for v0.2. For now, see the manifest schema in [`schemas/manifest.v1.0.json`](schemas/manifest.v1.0.json) and the format spec in [`docs/adr/003-character-package-format.md`](docs/adr/003-character-package-format.md).

Minimum viable character: `idle` + `happy` states. Missing states fall back gracefully.

---

## Project Status

| Phase | Status | Highlights |
|-------|--------|-----------|
| **Planning** | ✅ Complete | PRD + TDD + 5 ADRs + adversarial review + 4-way debate |
| **Phase 0** | ✅ Foundation | Tauri scaffold, transparent overlay, CI workflow |
| **Phase 1** | ✅ Event engine | HTTP server + state machine + texture fusion + hook bridge |
| **Phase 2** | 🛠️ In progress | Character loader, PixiJS sprite renderer |
| **Phase 3** | ✅ Shipped | Settings UI, speech bubble, system tray, .dmg release |
| **v0.2 (Windows scaffolding)** | 🛠️ Alpha | MSI/NSIS bundles, CI matrix, hook script — overlay & signing TBD |
| **v0.3 (Linux scaffolding)** | 🛠️ Alpha | .deb / .rpm / .AppImage bundles, CI release job — Wayland transparency TBD |
| **v0.4+ (adapters)** | 🔬 Researched | Codex CLI (low effort, [#32](https://github.com/hoangperry/shikigami/issues/32)) · Cursor ([#33](https://github.com/hoangperry/shikigami/issues/33)) · Windsurf ([#34](https://github.com/hoangperry/shikigami/issues/34)) · Copilot Chat ([#35](https://github.com/hoangperry/shikigami/issues/35)) — survey at `plans/reports/researcher-260502-0814-ai-tool-adapter-survey.md` |

Progress tracker: [GitHub Issues](https://github.com/hoangperry/shikigami/issues).

---

## Contributing

Open source contributions welcome. Some pointers:

- New adapters (Cursor / Windsurf / ChatGPT): modify only the **Bridge** stage in the pipeline — downstream is identical across tools
- New emotion states / textures: add to `src-tauri/src/state/canonical.rs` and update `schemas/manifest.v1.0.json`
- Character packs: follow [`docs/CHARACTER-PLAYBOOK.md`](docs/CHARACTER-PLAYBOOK.md); any SPDX-compatible permissive license is accepted (CC-BY-SA-4.0 preferred for sprites)

All PRs must pass CI (`cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `pnpm typecheck`, schema validation).

---

## License & Attribution

### Shikigami Source Code

**Code**: MIT (see [`LICENSE`](LICENSE) — to be added).
**Default sprite character `linh-pixel`**: MIT (procedurally generated, our own code).
**Production Linh character** (`characters/linh/`): will ship under CC-BY-SA-4.0 when assets are finalized.

### Dependency Licenses (audited)

All dependencies are **permissive and MIT-compatible**. No GPL, LGPL, or proprietary runtime blobs.

**Rust crates** (`cargo tree --depth 1`):

| Crate | License |
|-------|---------|
| `tauri`, `tauri-plugin-fs` | Apache-2.0 OR MIT |
| `tokio`, `axum`, `tower`, `tower-http`, `tracing`, `tracing-subscriber` | MIT |
| `serde`, `serde_json`, `regex`, `once_cell`, `hex`, `rand`, `anyhow`, `thiserror`, `dirs` | MIT OR Apache-2.0 |
| `subtle` | BSD-3-Clause |

**npm packages** (direct deps):

| Package | License |
|---------|---------|
| `@tauri-apps/api`, `@tauri-apps/cli`, `@tauri-apps/plugin-fs` | Apache-2.0 OR MIT |
| `react`, `react-dom`, `@types/react`, `@types/react-dom` | MIT |
| `@vitejs/plugin-react`, `vite`, `zustand`, `eslint`, `prettier` | MIT |
| `typescript` | Apache-2.0 |

**Python** (`hooks/shikigami-hook.py`, `scripts/install-hook.py`, `characters/linh-pixel/src/generate.py`): uses only Python 3 stdlib + `Pillow` (HPND License — permissive, stdlib-compatible).

### Assets

- **App icon** (`src-tauri/icons/*`): procedurally generated at build time by `src-tauri/icons/` recipe. Renders the Japanese character `式` (ideograph, no copyright) using a system font on macOS (Hiragino Sans, bundled with macOS; output bitmap is a distributable derivative). Replace before v1.0 release.
- **Reference images** under `characters/linh/reference/`: generated via OpenAI's image-generation tools during development. OpenAI Terms of Use grant users ownership of generated outputs with commercial use permitted; used here only as artist reference, not shipped in the runtime bundle.

### Inspirations (referenced, not copied)

Shikigami draws architectural inspiration from several open-source projects:

- **[airi by moeru-ai](https://github.com/moeru-ai/airi)** (MIT) — plugin-protocol identity patterns, `[EMOTION:x]` prompt-tag idea, "Soul vs Stage" separation, pipeline stage naming. See [`docs/research/260422-airi-reusability-analysis.md`](docs/research/260422-airi-reusability-analysis.md) for a full audit. **No code is copied from airi into this repository.** Patterns and concepts are used under independent implementation.
- **VSCode extension format** — inspiration for the `.shikigami` zip package layout.
- **Live2D Cubism SDK** — explicitly kept out of the core runtime (ADR-000); deferred to an optional add-on in a separate repository to preserve truly-OSS status.

### License Compatibility Conclusion

Shikigami can ship and be redistributed under **MIT** without any carve-outs or special attribution beyond standard OSS credit. All dependencies are permissive. All inspirations are pattern-level with independent implementations. All asset pipelines use either our own procedural code or AI tools that grant output ownership.

If you find an attribution gap or a license compatibility concern, please open an issue.

---

## Links

- **Repository**: https://github.com/hoangperry/shikigami
- **Issues**: https://github.com/hoangperry/shikigami/issues
- **CI**: https://github.com/hoangperry/shikigami/actions

---

*"She watches what your agent actually does. She reacts with truth. Summoned by code, grounded in events."*
