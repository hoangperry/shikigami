# 式神 · Shikigami

**Ngôn ngữ**: [English](README.md) · **Tiếng Việt** · [日本語](README.ja.md) · [简体中文](README.zh-CN.md)

> Linh thú desktop được triệu hồi, phản ánh trạng thái real-time của AI agent qua nhân vật 2D phản ứng động, hiển thị dưới dạng overlay Picture-in-Picture trong suốt.

**Trạng thái**: `v0.1.0-alpha` · planning xong + event engine đã ship · character renderer đang làm
**Nền tảng**: macOS (Apple Silicon + Intel, chính) · Windows (alpha — chưa ký, transparency chưa verify) · Linux (alpha — deb/rpm/AppImage, Wayland transparency chưa verify)
**Tích hợp hiện tại**: Claude Code (chính) · Codex CLI (alpha) · Cursor (alpha, 5-event minimal) · Windsurf (alpha, schema từ docs — chưa test với payload thực) · Copilot Chat trong milestone v0.4

---

## Nó làm gì

Phiên code AI agentic sinh ra rất nhiều event — tool invocation, error, commit, build chạy lâu — nhưng terminal chỉ hiện một bức tường text và không gì khác. Shikigami ngồi trên desktop của bạn dưới dạng cửa sổ trong suốt nhỏ với nhân vật animation phản ứng real-time với những gì agent đang thực sự làm. Build pass thì cười; test bị reject thì lo lắng; lỡ tay `rm -rf` thì khóa vào trạng thái cảnh báo trước khi bạn kịp Enter.

Nó *không* phải chatbot, VTuber rig, hay voice companion. Đây là **proprioception trực quan cho workflow agentic** — chỉ báo trạng thái có cá tính.

---

## Tại sao nó tồn tại

Phiên code AI dài thì nặng nhận thức và phẳng về thị giác. Hệ sinh thái hiện có chia thành "nghiêm túc" (status bar IDE — chức năng nhưng không có sự hiện diện) và "vui" (desktop pet / VTuber rig — có hiện diện nhưng không nhận biết agent). Shikigami bắc cầu hai phía: nhân vật grounded trong event có cấu trúc từ AI tool của bạn, không phải pattern text trang trí.

Thiết kế cố ý hẹp để có thể đúng. Đọc [`docs/PRD.md`](docs/PRD.md) cho đầy đủ rationale sản phẩm.

---

## Quick Start

```bash
# 1. Clone
git clone https://github.com/hoangperry/shikigami.git
cd shikigami

# 2. Cài frontend deps
pnpm install

# 3. Chạy dev build (mở cửa sổ trong suốt always-on-top)
pnpm tauri:dev

# 4. Terminal khác: đăng ký Claude Code hooks
python3 scripts/install-hook.py install

# 5. Dùng Claude Code bình thường — nhân vật phản ứng theo từng tool call
```

Kiểm tra health:

```bash
python3 scripts/install-hook.py doctor
```

Test không cần Claude Code (event thủ công):

```bash
TOKEN=$(cat ~/.shikigami/token)
PORT=$(jq -r .port ~/.shikigami/config.json)
curl -X POST "http://127.0.0.1:$PORT/v1/events" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"schemaVersion":"1.0","source":"claude-code","type":"git_commit","text":"fix critical bug, finally"}'
# Nhân vật sẽ chuyển sang happy_relieved
```

Gỡ hooks:

```bash
python3 scripts/install-hook.py uninstall
```

### Codex CLI (alpha)

Codex CLI của OpenAI ship hook delivery model giống hệt Claude Code, nên cùng script `hooks/shikigami-hook.py` xử lý cả hai — chỉ khác field `source` trong EventPayload. Thêm snippet này vào `~/.codex/config.toml` (thay đường dẫn tuyệt đối với clone của bạn):

```toml
[hooks]
PreToolUse       = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
PostToolUse      = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
UserPromptSubmit = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
Stop             = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
SessionStart     = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
PermissionRequest = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
```

`PermissionRequest` chỉ có ở Codex; map sang trạng thái warning để nhân vật báo "agent đang chờ phê duyệt của bạn" thay vì trông idle khi dialog permission đang mở. Auto-installer cho TOML config của Codex được track nhưng defer (KISS — paste tay nhanh hơn việc thêm dependency write TOML).

### Cursor (alpha — 5-event minimal scope)

Cursor v1.7+ ship [hệ thống hook đầy đủ](https://cursor.com/docs/hooks) với 18+ agent event. Hiện tại chúng ta map **5 event mirror lifecycle core của Claude Code** (`sessionStart`, `preToolUse`, `postToolUse`, `postToolUseFailure`, `stop`); các event Cursor-specific còn lại (`afterMCPExecution`, `preCompact`, `afterAgentThought`, v.v.) được skip im lặng cho đến khi user thật bảo cái nào quan trọng. Pattern transformer dung sai — cùng playbook với Codex.

Thêm vào config hook của Cursor (per project, `.cursor/hooks.json`, hoặc global — tham khảo docs Cursor hiện tại cho vị trí file):

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

Tên field của Cursor khác Claude Code chút (`conversation_id` thay `session_id`, `workspace_roots[0]` thay `cwd`); `normalize_cursor()` của script viết lại tại edge để phần còn lại của pipeline giữ source-agnostic.

### Windsurf (alpha — schema từ docs, chưa test payload thực)

[Cascade Hooks](https://docs.windsurf.com/windsurf/cascade/hooks) của Windsurf ship 12 event chia theo loại tool (`pre_run_command`, `pre_read_code`, `pre_write_code`, `pre_mcp_tool_use`, `pre_user_prompt`, `post_cascade_response`, v.v.) với payload `tool_info` lồng nhau. `normalize_windsurf()` flatten chúng thành Claude shape để phần còn lại của bridge giữ source-agnostic. Chúng ta map 11 event phù hợp với taxonomy hiện có của Shikigami.

⚠️ **Trạng thái**: bridge này được implement chỉ từ docs — chưa có sample payload nào từ phiên Windsurf thực để validate tên field. Nếu bạn dùng Windsurf và thấy event bị drop im lặng hoặc field sai tên, vui lòng mở issue.

Thêm vào `~/.codeium/windsurf/hooks.json` (hoặc `.windsurf/hooks.json` per project, hoặc system path docs ở link trên):

```json
{
  "hooks": {
    "pre_user_prompt":     [{ "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source windsurf" }],
    "pre_run_command":     [{ "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source windsurf" }],
    "post_run_command":    [{ "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source windsurf" }],
    "pre_read_code":       [{ "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source windsurf" }],
    "post_read_code":      [{ "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source windsurf" }],
    "pre_write_code":      [{ "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source windsurf" }],
    "post_write_code":     [{ "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source windsurf" }],
    "pre_mcp_tool_use":    [{ "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source windsurf" }],
    "post_mcp_tool_use":   [{ "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source windsurf" }],
    "post_cascade_response":[{ "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source windsurf" }],
    "post_setup_worktree": [{ "command": "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source windsurf" }]
  }
}
```

Muốn giúp validate? Đăng ký `hooks/shikigami-windsurf-dumper.py` song song (hoặc thay) bridge — nó capture raw payload vào `~/.shikigami/windsurf-payloads.jsonl` để chúng ta đối chiếu với schema từ docs. Upload vào issue [#34](https://github.com/hoangperry/shikigami/issues/34).

---

## Cài .dmg (end users)

`Shikigami_*.dmg` build từ `pnpm tauri:build` được ký với cert **Apple Development** (phù hợp với máy test của tác giả gốc) nhưng **không** notarize qua Apple distribution service. Gatekeeper macOS sẽ chặn ở lần mở đầu với prompt "unidentified developer".

Hai cách an toàn để qua trên Mac mới:

**Option 1 — System Settings**
1. Double-click .dmg, kéo `Shikigami.app` vào `/Applications`
2. Thử mở app — macOS sẽ từ chối
3. Mở *System Settings → Privacy & Security* → kéo xuống thông báo "Shikigami was blocked" → click **Open Anyway**
4. Confirm trong dialog tiếp theo

**Option 2 — terminal (nhanh hơn)**
```bash
xattr -cr /Applications/Shikigami.app
open /Applications/Shikigami.app
```
`xattr -cr` strip thuộc tính `com.apple.quarantine` set trên mỗi file download từ internet. Signature vẫn còn nguyên — chỉ báo Gatekeeper rằng bạn tin nguồn.

Release tương lai với notarization Apple Developer Program đầy đủ sẽ bỏ bước này. Signature hiện tại là thật; chỉ chưa notarize:

```bash
codesign -dv /Applications/Shikigami.app
# Authority=Apple Development: Hoang Truong Nhat (Y44JV6U4Q9)
# Authority=Apple Worldwide Developer Relations Certification Authority
# Authority=Apple Root CA
```

---

## Cài trên Windows (alpha)

Build Windows ship dưới dạng **`.msi`** (system-wide, WiX) và **`.exe`** (per-user, NSIS) installer từ cùng release tag với macOS DMG. Tại milestone này binary Windows **chưa ký** — chưa có EV cert $400/năm — nên SmartScreen sẽ cảnh báo ở lần đầu chạy.

```powershell
# 1. Download Shikigami_*_x64-setup.exe (NSIS) hoặc .msi từ Release page
# 2. Chạy. SmartScreen cảnh báo → click "More info" → "Run anyway".
# 3. Cài Python 3 nếu chưa có (Claude Code yêu cầu Python sẵn)
# 4. Đăng ký hook (dùng python trên PATH, không phải python3):
python scripts\install-hook.py install

# 5. Verify
python scripts\install-hook.py doctor
```

⚠️ **Gap đã biết trong build Windows v0.1 alpha** (track trên GitHub Issues — đóng góp rất hoan nghênh):
- Transparent always-on-top overlay **chưa test trên Windows hardware thật** — Tauri dùng WebView2 có quirks documented về DWM composition và per-monitor DPI. Có thể render với background đen hoặc đục cho đến khi contributor verify + tune.
- Không code signing → SmartScreen cảnh báo mỗi lần cài mới.
- Hook bridge dùng cùng `shikigami-hook.py` như macOS; wrapper PowerShell-native (`hooks/shikigami-hook.ps1`) ship kèm cho user thích đăng ký trực tiếp.

Tạm thời Windows là best-effort: macOS là target alpha được hỗ trợ chính.

---

## Cài trên Linux (alpha)

Linux ship 3 format từ cùng release tag — chọn cái phù hợp distro:

```bash
# Debian / Ubuntu
sudo dpkg -i shikigami_*_amd64.deb
sudo apt-get install -f          # kéo về GTK / WebKit deps thiếu

# Fedora / RHEL / openSUSE
sudo rpm -i shikigami-*.x86_64.rpm

# Distro-agnostic (không cần cài)
chmod +x shikigami_*_amd64.AppImage
./shikigami_*_amd64.AppImage
```

Hook setup giống hệt macOS:

```bash
python3 scripts/install-hook.py install
python3 scripts/install-hook.py doctor
```

⚠️ **Gap đã biết trong build Linux v0.1 alpha** (đóng góp hoan nghênh):

- Transparent always-on-top overlay **chưa test trên desktop Linux thật**. X11 với compositor (Xfwm, Mutter, KWin) dự kiến chạy được; **Wayland cần verify** — một số compositor (GNOME cũ, sway thiếu protocol đúng) thiếu surface API Tauri dựa vào.
- Semantic `alwaysOnTop` khác giữa các compositor; behavior có thể degrade thành hint "above" thay vì topmost layer thật.
- Click-through cần compositor hỗ trợ input-shape. Abstraction của Tauri chạy trên X11 + hầu hết Wayland compositor, nhưng chưa có hardware run nào confirm cho Shikigami.

Linux là best-effort tại milestone này cùng Windows; macOS vẫn là target alpha được hỗ trợ chính.

---

## Tự build .dmg đã ký

Nếu bạn có Apple signing identity riêng (`security find-identity -v -p codesigning`), pass qua env:

```bash
APPLE_SIGNING_IDENTITY="Apple Development: <tên bạn> (<team-id>)" \
  pnpm tauri:build
```

Để distribution thật cần cert **Developer ID Application** (Apple Developer Program, $99/năm) cộng credential notarization — xem [hướng dẫn distribution macOS của Tauri](https://v2.tauri.app/distribute/sign/macos/).

---

## Cách hoạt động

Event chảy qua pipeline 7 stage:

```
Hook → Bridge → Ingest → Segment → Resolve → Emit → Render
 CC     Py      Rust     Rust      Rust      Rust    React+PixiJS
```

- **Bridge** (`hooks/shikigami-hook.py`) transform JSON hook của Claude Code thành `EventPayload` typed
- **Ingest** (`src-tauri/src/event/server.rs`) nhận HTTP POST trên `127.0.0.1` với bearer auth
- **Segment** (`src-tauri/src/state/dampen.rs`) dedup event lặp trong sliding window 2 giây
- **Resolve** (`src-tauri/src/state/machine.rs`) áp dụng Hierarchical Fusion: event drive dominant state, text modifier layer trên texture, severity scale duration
- **Emit** fire event `state_changed` Tauri tới frontend
- **Render** (Phase 2) paint sprite qua PixiJS

Chi tiết đầy đủ trong [`docs/PIPELINE.md`](docs/PIPELINE.md) · architectural decisions trong [`docs/adr/`](docs/adr/).

---

## Tính năng

- 🪶 Nhẹ: build trên **Tauri 2** (target <80 MB RAM idle)
- 🧠 **State event-driven**: phản ứng map theo những gì agent thực sự làm (tool call, exit code, git op), không phải pattern text được prompt-engineer
- 🛡️ **Severity-aware**: thao tác phá hủy như `rm -rf`, `DROP TABLE`, `git push --force` khóa nhân vật vào trạng thái cảnh báo critical với texture suppression
- 🎨 **Hệ thống cảm xúc 2-tầng**: 9 dominant state × 6 texture modifier = animation key biểu cảm như `happy_relieved` hoặc `focused_alarmed`
- 🔌 **Format nhân vật mở rộng**: gói zip `.shikigami`, portable across OS
- 🔒 **100% local**: không telemetry, không cloud, không proprietary deps (core)
- 🔁 **Toxic-loop-safe**: dampener ngăn strobing khi error lặp

---

## Documentation

| Doc | Nội dung |
|-----|----------|
| [`docs/PRD.md`](docs/PRD.md) | Product requirement v0.2 (post-review) |
| [`docs/TDD.md`](docs/TDD.md) | Technical design map PRD vào code |
| [`docs/PIPELINE.md`](docs/PIPELINE.md) | Narrative data flow 7-stage |
| [`docs/CHARACTER-PLAYBOOK.md`](docs/CHARACTER-PLAYBOOK.md) | Hướng dẫn production nhân vật + chiến lược commission |
| [`docs/codex-ui-prompts.md`](docs/codex-ui-prompts.md) | Prompt copy-paste cho GPT image-gen |
| [`docs/adr/`](docs/adr/) | Năm architecture decision record |
| [`docs/reviews/`](docs/reviews/) | Audit trail adversarial review |
| [`docs/debates/`](docs/debates/) | Quyết định multi-AI tournament |
| [`docs/research/`](docs/research/) | Phân tích reusability của repo external |

---

## Gói nhân vật

Nhân vật ship dưới dạng bundle zip `.shikigami` chứa manifest, sprite frame, preview, và license. Xem [`docs/adr/003-character-package-format.md`](docs/adr/003-character-package-format.md).

### Nhân vật mặc định

| Gói | Mục đích | License |
|-----|----------|---------|
| `characters/linh-pixel/` | Fixture dev 8-bit thủ tục | MIT |
| `characters/linh/` | Linh production (đang làm, anime/vector) | CC-BY-SA-4.0 khi ship |

Pixel fixture tồn tại để engineering có thể tiếp tục trong khi nhân vật production đang commission. Xem `characters/linh-pixel/README.md` cho chi tiết.

### Tự làm

Template repo và CLI `shikigami pack` được lên kế hoạch cho v0.2. Hiện tại xem manifest schema trong [`schemas/manifest.v1.0.json`](schemas/manifest.v1.0.json) và format spec trong [`docs/adr/003-character-package-format.md`](docs/adr/003-character-package-format.md).

Nhân vật minimum viable: state `idle` + `happy`. State thiếu fall back gracefully.

---

## Trạng thái dự án

| Phase | Trạng thái | Highlight |
|-------|-----------|-----------|
| **Planning** | ✅ Hoàn tất | PRD + TDD + 5 ADR + adversarial review + debate 4-way |
| **Phase 0** | ✅ Foundation | Tauri scaffold, transparent overlay, CI workflow |
| **Phase 1** | ✅ Event engine | HTTP server + state machine + texture fusion + hook bridge |
| **Phase 2** | 🛠️ Đang làm | Character loader, PixiJS sprite renderer |
| **Phase 3** | ✅ Đã ship | Settings UI, speech bubble, system tray, .dmg release |
| **v0.2 (Windows scaffolding)** | 🛠️ Alpha | MSI/NSIS bundle, CI matrix, hook script — overlay & signing TBD |
| **v0.3 (Linux scaffolding)** | 🛠️ Alpha | .deb / .rpm / .AppImage bundle, CI release job — Wayland transparency TBD |
| **v0.4+ (adapter)** | 🔬 Đã research | Codex CLI ([#32](https://github.com/hoangperry/shikigami/issues/32)) · Cursor ([#33](https://github.com/hoangperry/shikigami/issues/33)) · Windsurf ([#34](https://github.com/hoangperry/shikigami/issues/34)) · Copilot Chat ([#35](https://github.com/hoangperry/shikigami/issues/35)) — survey tại `plans/reports/researcher-260502-0814-ai-tool-adapter-survey.md` |

Tracker tiến độ: [GitHub Issues](https://github.com/hoangperry/shikigami/issues).

---

## Đóng góp

Đóng góp open source được hoan nghênh. Một số hướng:

- Adapter mới (Cursor / Windsurf / ChatGPT): chỉ sửa stage **Bridge** trong pipeline — downstream identical across tool
- State / texture cảm xúc mới: thêm vào `src-tauri/src/state/canonical.rs` và update `schemas/manifest.v1.0.json`
- Pack nhân vật: theo [`docs/CHARACTER-PLAYBOOK.md`](docs/CHARACTER-PLAYBOOK.md); accept bất kỳ license SPDX-compatible permissive (CC-BY-SA-4.0 ưu tiên cho sprite)

Tất cả PR phải pass CI (`cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `pnpm typecheck`, schema validation).

---

## License & Attribution

### Source Code Shikigami

**Code**: MIT (xem [`LICENSE`](LICENSE) — sẽ thêm).
**Nhân vật sprite mặc định `linh-pixel`**: MIT (procedurally generated, code chính chúng tôi).
**Nhân vật Linh production** (`characters/linh/`): sẽ ship dưới CC-BY-SA-4.0 khi asset finalized.

### License Dependency (đã audit)

Tất cả dependency đều **permissive và MIT-compatible**. Không GPL, LGPL, hay blob runtime proprietary.

**Rust crate** (`cargo tree --depth 1`):

| Crate | License |
|-------|---------|
| `tauri`, `tauri-plugin-fs` | Apache-2.0 OR MIT |
| `tokio`, `axum`, `tower`, `tower-http`, `tracing`, `tracing-subscriber` | MIT |
| `serde`, `serde_json`, `regex`, `once_cell`, `hex`, `rand`, `anyhow`, `thiserror`, `dirs` | MIT OR Apache-2.0 |
| `subtle` | BSD-3-Clause |

**npm package** (direct deps):

| Package | License |
|---------|---------|
| `@tauri-apps/api`, `@tauri-apps/cli`, `@tauri-apps/plugin-fs` | Apache-2.0 OR MIT |
| `react`, `react-dom`, `@types/react`, `@types/react-dom` | MIT |
| `@vitejs/plugin-react`, `vite`, `zustand`, `eslint`, `prettier` | MIT |
| `typescript` | Apache-2.0 |

**Python** (`hooks/shikigami-hook.py`, `scripts/install-hook.py`, `characters/linh-pixel/src/generate.py`): chỉ dùng Python 3 stdlib + `Pillow` (License HPND — permissive, stdlib-compatible).

### Asset

- **App icon** (`src-tauri/icons/*`): generated thủ tục tại build time bởi recipe `src-tauri/icons/`. Render ký tự Nhật `式` (chữ ý nghĩa, không bản quyền) dùng font hệ thống trên macOS (Hiragino Sans, đi kèm macOS; output bitmap là derivative phân phối được). Replace trước v1.0 release.
- **Reference image** dưới `characters/linh/reference/`: generate qua công cụ image-generation của OpenAI trong dev. Terms of Use OpenAI cho user quyền sở hữu output generated với commercial use cho phép; chỉ dùng làm reference cho artist, không ship trong runtime bundle.

### Inspiration (referenced, không copy)

Shikigami lấy cảm hứng kiến trúc từ một số dự án open source:

- **[airi by moeru-ai](https://github.com/moeru-ai/airi)** (MIT) — pattern identity plugin-protocol, ý tưởng prompt-tag `[EMOTION:x]`, tách "Soul vs Stage", naming pipeline stage. Xem [`docs/research/260422-airi-reusability-analysis.md`](docs/research/260422-airi-reusability-analysis.md) cho audit đầy đủ. **Không có code nào copy từ airi vào repo này.** Pattern và concept dùng dưới implementation độc lập.
- **Format VSCode extension** — cảm hứng cho layout gói zip `.shikigami`.
- **Live2D Cubism SDK** — explicit giữ ngoài runtime core (ADR-000); defer sang add-on optional trong repo riêng để giữ trạng thái truly-OSS.

### Kết luận License Compatibility

Shikigami có thể ship và redistribute dưới **MIT** không có carve-out hay attribution đặc biệt nào ngoài credit OSS chuẩn. Tất cả dependency permissive. Tất cả inspiration ở pattern-level với implementation độc lập. Tất cả pipeline asset dùng code thủ tục của chúng ta hoặc tool AI cấp ownership output.

Nếu bạn thấy gap attribution hay vấn đề license compatibility, vui lòng mở issue.

---

## Link

- **Repository**: https://github.com/hoangperry/shikigami
- **Issues**: https://github.com/hoangperry/shikigami/issues
- **CI**: https://github.com/hoangperry/shikigami/actions

---

*"Cô ấy nhìn những gì agent của bạn thực sự làm. Cô ấy phản ứng với sự thật. Triệu hồi bởi code, grounded trong event."*
