# Windows UTM Smoke-Test Runbook

> **Status**: contributor-facing runbook. Issue [#29](https://github.com/hoangperry/shikigami/issues/29) tracks the full hardware-validation path.
> **Scope**: confirms the Windows MSI / NSIS bundle installs and runs, exercises Win32 windowing/DPI/topmost/click-through mechanics. **Does NOT confirm transparent-background rendering** — that needs bare-metal DWM verification on real hardware.

---

## Why this exists

The release workflow ships unsigned `.msi` and `.exe` installers from CI on every tag. Until a contributor has a real Windows box to test on, the binaries are unverified end-to-end. The four-way debate at `docs/debates/2026-05-03-v04-research-later-feasibility/` reached a SPLIT compromise: a UTM Windows-on-ARM VM catches the **Win32 mechanics** half cheaply (free, runs on any Apple Silicon Mac), and the **transparent-overlay rendering** half stays on the issue tracker waiting for hardware validation.

This runbook covers the UTM half.

---

## Prerequisites

- macOS 13+ on **Apple Silicon** (M1/M2/M3/M4) — Intel Macs cannot run Windows-on-ARM under UTM
- ~30 GB free disk space (Windows 11 ARM Insider Preview + Tauri runtime + dev tools)
- A copy of `Shikigami_*_x64-setup.exe` (NSIS) or `Shikigami_*_x64_en-US.msi` (WiX) from the latest release page

> **Note**: The shipped installers are x64. UTM on Apple Silicon runs Windows-on-ARM, which transparently emulates x64 binaries via Microsoft's built-in WoW64 layer. Performance is fine for smoke testing; not representative of native-x64 hardware performance.

---

## Setup (one-time, ~30 minutes)

### 1. Install UTM

```bash
brew install --cask utm
```

Or download from https://mac.getutm.app/.

### 2. Get a Windows 11 ARM image

Microsoft hosts the ARM Insider Preview at https://www.microsoft.com/en-us/software-download/windowsinsiderpreviewARM64 (free, no licence required for evaluation).

### 3. Create a UTM VM

UTM has a Windows-specific gallery template at https://mac.getutm.app/gallery/. Use the "Windows 11" entry — it pre-configures the right virtio drivers and TPM emulation. Allocate at least 4 GB RAM, 2 vCPU.

### 4. First boot — accept Windows OOBE defaults; skip Microsoft account.

---

## Smoke-test checklist

After installing the Shikigami `.exe` or `.msi` inside the VM, walk through:

### Mechanics (this is what UTM CAN verify)
- [ ] Installer runs without admin prompt errors (NSIS) or installs system-wide cleanly (MSI)
- [ ] App launches without crashing
- [ ] Window appears at the configured 480×640 default size
- [ ] Window is `alwaysOnTop` — opens above File Explorer / Edge / Notepad
- [ ] Window is `skipTaskbar` — does not show in the taskbar list
- [ ] Window is draggable from the character region (no native title bar)
- [ ] Resize handles work (4 corners)
- [ ] Click-through works: cursor over transparent area passes click to whatever is below; cursor over the character catches the click
- [ ] System tray icon appears and the menu opens with the expected items (Toggle Click-Through, Run Demo, Quit)
- [ ] Hook installer runs: `python scripts\install-hook.py install` and `... doctor` both succeed

### Transparency (this is what UTM CANNOT honestly verify)
- [ ] *(Hardware only)* Background is genuinely transparent — character floats over the desktop with no opaque rectangle
- [ ] *(Hardware only)* `alwaysOnTop` interaction with Windows 11 Snap Layouts overlay
- [ ] *(Hardware only)* Per-monitor DPI scaling on multi-monitor setups
- [ ] *(Hardware only)* Behaviour over fullscreen apps (especially fullscreen games — DWM handoff)

---

## Reporting results

After the smoke test, comment on issue [#29](https://github.com/hoangperry/shikigami/issues/29) with:

1. Windows version + build number (`winver` in cmd)
2. Which checklist items in the **Mechanics** section passed / failed
3. Screenshot or screen recording showing the rendered window — even if transparency is broken under UTM, the screenshot tells us whether the window is positioned and sized correctly

If you have access to **bare-metal Windows hardware**, the Transparency checklist items become testable too — those are the ones that matter most for closing the hardware-validation half of #29.

---

## Why UTM transparency results are unreliable

UTM's macOS host runs a **virtio-gpu** display driver inside the guest, which composites via QEMU rather than Windows's native DWM. DWM transparency depends on the Windows compositor having direct access to the GPU's hardware overlay planes — virtio-gpu doesn't expose those, so the alpha channel may be silently dropped or composited against a solid colour. A "transparent works in UTM" signal is weak evidence the binary works on real hardware; a "transparent fails in UTM" signal is also weak (it might work on bare metal).

This is why the runbook scopes UTM to the **mechanics** half and explicitly defers transparency to the hardware track in issue #29.

---

## See also

- [`docs/WINDOWS-SIGNING.md`](./WINDOWS-SIGNING.md) — code-signing procurement runbook (issue #30, closed)
- Issue [#29](https://github.com/hoangperry/shikigami/issues/29) — overarching Windows verification tracker
- [`docs/debates/2026-05-03-v04-research-later-feasibility/synthesis.md`](./debates/2026-05-03-v04-research-later-feasibility/synthesis.md) — debate that produced this SPLIT compromise
