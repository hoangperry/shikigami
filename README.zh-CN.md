# 式神 · Shikigami

**语言**: [English](README.md) · [Tiếng Việt](README.vi.md) · [日本語](README.ja.md) · **简体中文**

> 一个被召唤的桌面伙伴 —— 通过反应式 2D 角色实时反映你 AI agent 的状态，以透明的画中画形式叠加显示。

**状态**: `v0.1.0-alpha` · 规划完成 + 事件引擎已发布 · 角色渲染器进行中
**平台**: macOS (Apple Silicon + Intel,主要) · Windows (alpha — 未签名,透明度未验证) · Linux (alpha — deb/rpm/AppImage,Wayland 透明度未验证)
**当前集成**: Claude Code (主要) · Codex CLI (alpha) · Cursor (alpha,5 事件最小集) · Windsurf (alpha,基于文档推导的 schema — 未在真实 payload 上测试) · Copilot Chat 在 v0.4 milestone 跟踪

---

## 它做什么

Agentic AI 编程会话产生大量事件 —— 工具调用、错误、commit、长时间构建 —— 但终端只给你一面文字墙,什么都没有。Shikigami 作为一个小的透明窗口坐在你的桌面上,显示一个动画角色,实时对你的 agent 实际所做的事情做出反应。绿色构建让她微笑;测试被拒让她担忧;一个错乱的 `rm -rf` 会在你按下回车前把她锁定到警报状态。

它*不是*聊天机器人、VTuber rig 或语音伙伴。它是 **agentic 工作流的视觉本体感觉** —— 一个带个性的状态指示器。

---

## 为什么存在

长 AI 编程会话在认知上沉重而在视觉上单调。现有生态分为"严肃" (功能性的、无存在感的 IDE 状态栏) 和"有趣" (有存在感、但无 agent 感知的桌面宠物 / VTuber rig)。Shikigami 桥接两者:角色基于来自 AI 工具的结构化事件,而非装饰性文本模式。

设计有意收窄以确保正确。完整产品依据见 [`docs/PRD.md`](docs/PRD.md)。

---

## 快速开始

```bash
# 1. Clone
git clone https://github.com/hoangperry/shikigami.git
cd shikigami

# 2. 安装前端依赖
pnpm install

# 3. 运行 dev 构建 (打开一个透明的总在最前窗口)
pnpm tauri:dev

# 4. 另一个终端: 注册 Claude Code hooks
python3 scripts/install-hook.py install

# 5. 正常使用 Claude Code —— 角色对每次工具调用做出反应
```

健康检查:

```bash
python3 scripts/install-hook.py doctor
```

不用 Claude Code 测试 (手动事件):

```bash
TOKEN=$(cat ~/.shikigami/token)
PORT=$(jq -r .port ~/.shikigami/config.json)
curl -X POST "http://127.0.0.1:$PORT/v1/events" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"schemaVersion":"1.0","source":"claude-code","type":"git_commit","text":"fix critical bug, finally"}'
# 角色应该动画到 happy_relieved
```

卸载 hooks:

```bash
python3 scripts/install-hook.py uninstall
```

### Codex CLI (alpha)

OpenAI 的 Codex CLI 提供与 Claude Code 相同的 hook 投递模型,所以同一个 `hooks/shikigami-hook.py` 脚本处理两者 —— 只有 EventPayload 的 `source` 字段不同。把这段加到 `~/.codex/config.toml` (用你的 clone 替换绝对路径):

```toml
[hooks]
PreToolUse       = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
PostToolUse      = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
UserPromptSubmit = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
Stop             = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
SessionStart     = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
PermissionRequest = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
```

`PermissionRequest` 仅 Codex 有;映射到 warning 状态,角色会发出"agent 等待你批准"信号,而不是在 permission 对话框打开时看起来空闲。Codex TOML config 的自动安装器在跟踪但延后 (KISS —— 手动粘贴比引入 TOML 写依赖更快)。

### Cursor (alpha — 5 事件最小集)

Cursor v1.7+ 提供包含 18+ agent 事件的 [完整 hook 系统](https://cursor.com/docs/hooks)。我们今天映射 **5 个镜像 Claude Code 核心生命周期的事件** (`sessionStart`、`preToolUse`、`postToolUse`、`postToolUseFailure`、`stop`);其余 Cursor 特有事件 (`afterMCPExecution`、`preCompact`、`afterAgentThought` 等) 默默跳过,直到真实用户告诉我们哪些重要。容错 transformer 模式 —— 与 Codex 同剧本。

加到 Cursor 的 hook 配置 (按项目 `.cursor/hooks.json`,或全局 —— 文件位置参考 Cursor 当前文档):

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

Cursor 字段名与 Claude Code 略有不同 (`conversation_id` 替代 `session_id`、`workspace_roots[0]` 替代 `cwd`);脚本的 `normalize_cursor()` 在边界重写,流水线其余部分保持 source-agnostic。

### Windsurf (alpha — 基于文档推导的 schema,未在真实 payload 上测试)

Windsurf 的 [Cascade Hooks](https://docs.windsurf.com/windsurf/cascade/hooks) 提供按工具类型分割的 12 个事件 (`pre_run_command`、`pre_read_code`、`pre_write_code`、`pre_mcp_tool_use`、`pre_user_prompt`、`post_cascade_response` 等),带嵌套的 `tool_info` payload。`normalize_windsurf()` 把它们摊平到 Claude 形状,bridge 其余部分保持 source-agnostic。我们映射 11 个适合 Shikigami 现有分类的事件。

⚠️ **状态**: 此 bridge 仅根据文档实现 —— 没有来自真实 Windsurf 会话的 payload 样本来验证字段名。如果你使用 Windsurf 并发现事件被默默丢弃或字段名错误,请提交 issue。

加到 `~/.codeium/windsurf/hooks.json` (或按项目的 `.windsurf/hooks.json`,或上述链接中文档化的系统路径):

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

想帮助验证? 与 (或代替) bridge 一起注册 `hooks/shikigami-windsurf-dumper.py` —— 它会把原始 payload 捕获到 `~/.shikigami/windsurf-payloads.jsonl`,我们可以与文档推导的 schema 对比。上传到 issue [#34](https://github.com/hoangperry/shikigami/issues/34)。

---

## 安装 .dmg (终端用户)

`pnpm tauri:build` 产生的 `Shikigami_*.dmg` 用 **Apple Development** 证书签名 (适合原作者的测试设备),但**未**通过 Apple 分发服务做 notarize。macOS Gatekeeper 会在首次启动时以"unidentified developer"提示阻止它。

在新 Mac 上通过的两种安全方式:

**选项 1 —— 系统设置**
1. 双击 .dmg,把 `Shikigami.app` 拖到 `/Applications`
2. 试图打开 app —— macOS 会拒绝
3. 打开 *系统设置 → 隐私与安全* → 滚动到 "Shikigami was blocked" 通知 → 点击 **仍要打开**
4. 在弹出对话框中确认

**选项 2 —— 终端 (更快)**
```bash
xattr -cr /Applications/Shikigami.app
open /Applications/Shikigami.app
```
`xattr -cr` 剥离每个从互联网下载的文件上设置的 `com.apple.quarantine` 属性。签名本身保持完整 —— 这只是告诉 Gatekeeper 你信任来源。

未来带完整 Apple Developer Program notarization 的发行版会移除这一步。今天看到的签名是真的;只是还未 notarize:

```bash
codesign -dv /Applications/Shikigami.app
# Authority=Apple Development: Hoang Truong Nhat (Y44JV6U4Q9)
# Authority=Apple Worldwide Developer Relations Certification Authority
# Authority=Apple Root CA
```

---

## 在 Windows 上安装 (alpha)

Windows 构建从与 macOS DMG 同一个 release tag 提供 **`.msi`** (系统级,WiX) 和 **`.exe`** (用户级,NSIS) 安装器。这个 milestone 上 Windows 二进制**未签名** —— 还没有 $400/年的 EV 证书 —— 所以 SmartScreen 在首次启动时会警告。

```powershell
# 1. 从 Release 页下载 Shikigami_*_x64-setup.exe (NSIS) 或 .msi
# 2. 运行。SmartScreen 警告 → 点击"更多信息"→"仍要运行"。
# 3. 如果没有 Python 3 就装 (Claude Code 已经要求 Python)
# 4. 注册 hook (用 PATH 上的 python,不是 python3):
python scripts\install-hook.py install

# 5. 验证
python scripts\install-hook.py doctor
```

⚠️ **v0.1 alpha Windows 构建已知缺口** (在 GitHub Issues 跟踪 —— 非常欢迎贡献):
- 透明的总在最前 overlay **未在真实 Windows 硬件上测试** —— Tauri 用 WebView2,在 DWM 合成和按显示器 DPI 方面有文档化的 quirk。在贡献者验证 + 调试之前,可能渲染为黑色或不透明背景。
- 无代码签名 → 每次新安装都有 SmartScreen 警告。
- Hook bridge 与 macOS 用同一个 `shikigami-hook.py`;PowerShell 原生封装 (`hooks/shikigami-hook.ps1`) 一并出货,供偏好直接注册的用户。

目前 Windows 是尽力而为: macOS 是受支持的 alpha 目标。

---

## 在 Linux 上安装 (alpha)

Linux 从同一 release tag 提供三种格式 —— 选择你的 distro 偏好:

```bash
# Debian / Ubuntu
sudo dpkg -i shikigami_*_amd64.deb
sudo apt-get install -f          # 拉取缺失的 GTK / WebKit 依赖

# Fedora / RHEL / openSUSE
sudo rpm -i shikigami-*.x86_64.rpm

# 与 distro 无关 (无需安装)
chmod +x shikigami_*_amd64.AppImage
./shikigami_*_amd64.AppImage
```

Hook 设置与 macOS 相同:

```bash
python3 scripts/install-hook.py install
python3 scripts/install-hook.py doctor
```

⚠️ **v0.1 alpha Linux 构建已知缺口** (欢迎贡献):

- 透明的总在最前 overlay **未在真实 Linux 桌面上测试**。带合成器 (Xfwm、Mutter、KWin) 的 X11 预期可工作;**Wayland 需验证** —— 一些合成器 (旧 GNOME、缺少正确协议的 sway) 缺少 Tauri 依赖的 surface API。
- `alwaysOnTop` 语义在不同合成器上不同;行为可能降级为常规"above"提示而非真正的最顶层。
- 点击穿透需要合成器的 input-shape 支持。Tauri 的抽象在 X11 + 大多数 Wayland 合成器上工作,但还没有 Shikigami 的硬件运行确认。

这个 milestone 上 Linux 与 Windows 一起是尽力而为;macOS 仍是受支持的 alpha 目标。

---

## 自己构建签名的 .dmg

如果你有自己的 Apple 签名 identity (`security find-identity -v -p codesigning`),通过 env 传递:

```bash
APPLE_SIGNING_IDENTITY="Apple Development: <你的名字> (<team-id>)" \
  pnpm tauri:build
```

真正的分发需要 **Developer ID Application** 证书 (Apple Developer Program,$99/年) 加上 notarization 凭证 —— 见 [Tauri 的 macOS 分发指南](https://v2.tauri.app/distribute/sign/macos/)。

---

## 工作原理

事件流过 7 阶段流水线:

```
Hook → Bridge → Ingest → Segment → Resolve → Emit → Render
 CC     Py      Rust     Rust      Rust      Rust    React+PixiJS
```

- **Bridge** (`hooks/shikigami-hook.py`) 把 Claude Code hook JSON 转换成有类型的 `EventPayload`
- **Ingest** (`src-tauri/src/event/server.rs`) 在 `127.0.0.1` 上以 bearer 认证接收 HTTP POST
- **Segment** (`src-tauri/src/state/dampen.rs`) 在 2 秒滑动窗口内对重复事件去重
- **Resolve** (`src-tauri/src/state/machine.rs`) 应用 Hierarchical Fusion: 事件驱动 dominant state、文本修饰符叠加在纹理上、severity 缩放持续时间
- **Emit** 向前端触发 `state_changed` Tauri 事件
- **Render** (Phase 2) 通过 PixiJS 绘制 sprite

完整细节见 [`docs/PIPELINE.md`](docs/PIPELINE.md) · 架构决策见 [`docs/adr/`](docs/adr/)。

---

## 特性

- 🪶 轻量: 基于 **Tauri 2** 构建 (空闲 <80 MB RAM 目标)
- 🧠 **事件驱动状态**: 反应映射到 agent 实际做的事 (工具调用、退出码、git 操作),而非 prompt 工程的文本模式
- 🛡️ **严重度感知**: 像 `rm -rf`、`DROP TABLE`、`git push --force` 这样的破坏性操作把角色锁定到关键警告状态并抑制纹理
- 🎨 **双层情绪系统**: 9 个 dominant state × 6 个纹理修饰符 = 富表现力的动画键如 `happy_relieved` 或 `focused_alarmed`
- 🔌 **可扩展角色格式**: `.shikigami` zip 包,跨 OS 可移植
- 🔒 **100% 本地**: 无遥测、无云、无 proprietary 依赖 (核心)
- 🔁 **Toxic-loop 安全**: 阻尼器在错误重复时防止闪烁

---

## 文档

| Doc | 涵盖内容 |
|-----|----------|
| [`docs/PRD.md`](docs/PRD.md) | 产品需求 v0.2 (post-review) |
| [`docs/TDD.md`](docs/TDD.md) | 把 PRD 映射到代码的技术设计 |
| [`docs/PIPELINE.md`](docs/PIPELINE.md) | 7 阶段数据流叙述 |
| [`docs/CHARACTER-PLAYBOOK.md`](docs/CHARACTER-PLAYBOOK.md) | 角色制作指南 + commission 策略 |
| [`docs/codex-ui-prompts.md`](docs/codex-ui-prompts.md) | GPT image-gen 的复制粘贴 prompt |
| [`docs/adr/`](docs/adr/) | 五个架构决策记录 |
| [`docs/reviews/`](docs/reviews/) | Adversarial review 审计追踪 |
| [`docs/debates/`](docs/debates/) | 多 AI 锦标赛决策 |
| [`docs/research/`](docs/research/) | 外部仓库可复用性分析 |

---

## 角色包

角色作为 `.shikigami` zip 包发布,包含 manifest、sprite 帧、preview 和 license。见 [`docs/adr/003-character-package-format.md`](docs/adr/003-character-package-format.md)。

### 默认角色

| 包 | 用途 | License |
|----|------|---------|
| `characters/linh-pixel/` | 程序化 8-bit 开发 fixture | MIT |
| `characters/linh/` | 生产 Linh (进行中,anime/vector) | 发布时 CC-BY-SA-4.0 |

像素 fixture 存在的目的是让工程能在生产角色 commission 期间继续推进。详情见 `characters/linh-pixel/README.md`。

### 自己制作

模板 repo 和 `shikigami pack` CLI 计划在 v0.2。当前见 [`schemas/manifest.v1.0.json`](schemas/manifest.v1.0.json) 中的 manifest schema 和 [`docs/adr/003-character-package-format.md`](docs/adr/003-character-package-format.md) 中的格式规范。

最小可行角色: `idle` + `happy` 状态。缺失状态优雅回退。

---

## 项目状态

| Phase | 状态 | 亮点 |
|-------|------|------|
| **Planning** | ✅ 完成 | PRD + TDD + 5 ADR + adversarial review + 4-way debate |
| **Phase 0** | ✅ 基础 | Tauri scaffold、透明 overlay、CI workflow |
| **Phase 1** | ✅ 事件引擎 | HTTP server + state machine + texture fusion + hook bridge |
| **Phase 2** | 🛠️ 进行中 | 角色加载器、PixiJS sprite 渲染器 |
| **Phase 3** | ✅ 已发布 | Settings UI、speech bubble、系统托盘、.dmg 发布 |
| **v0.2 (Windows scaffolding)** | 🛠️ Alpha | MSI/NSIS 包、CI 矩阵、hook 脚本 —— overlay 和签名待定 |
| **v0.3 (Linux scaffolding)** | 🛠️ Alpha | .deb / .rpm / .AppImage 包、CI release job —— Wayland 透明度待定 |
| **v0.4+ (适配器)** | 🔬 已研究 | Codex CLI ([#32](https://github.com/hoangperry/shikigami/issues/32)) · Cursor ([#33](https://github.com/hoangperry/shikigami/issues/33)) · Windsurf ([#34](https://github.com/hoangperry/shikigami/issues/34)) · Copilot Chat ([#35](https://github.com/hoangperry/shikigami/issues/35)) —— 调研在 `plans/reports/researcher-260502-0814-ai-tool-adapter-survey.md` |

进度跟踪: [GitHub Issues](https://github.com/hoangperry/shikigami/issues)。

---

## 贡献

欢迎开源贡献。一些指引:

- 新适配器 (Cursor / Windsurf / ChatGPT): 只修改 pipeline 的 **Bridge** 阶段 —— 下游在工具间相同
- 新情绪状态 / 纹理: 加到 `src-tauri/src/state/canonical.rs` 并更新 `schemas/manifest.v1.0.json`
- 角色包: 遵循 [`docs/CHARACTER-PLAYBOOK.md`](docs/CHARACTER-PLAYBOOK.md);接受任何 SPDX 兼容的 permissive license (sprite 优先 CC-BY-SA-4.0)

所有 PR 必须通过 CI (`cargo fmt`、`cargo clippy -D warnings`、`cargo test`、`pnpm typecheck`、schema 验证)。

---

## License & 归属

### Shikigami 源码

**代码**: MIT (见 [`LICENSE`](LICENSE) —— 待添加)。
**默认 sprite 角色 `linh-pixel`**: MIT (程序化生成,我们自己的代码)。
**生产 Linh 角色** (`characters/linh/`): 资产定型时以 CC-BY-SA-4.0 发布。

### 依赖 License (已审计)

所有依赖都**permissive 且 MIT 兼容**。无 GPL、LGPL 或 proprietary 运行时 blob。

**Rust crate** (`cargo tree --depth 1`):

| Crate | License |
|-------|---------|
| `tauri`, `tauri-plugin-fs` | Apache-2.0 OR MIT |
| `tokio`, `axum`, `tower`, `tower-http`, `tracing`, `tracing-subscriber` | MIT |
| `serde`, `serde_json`, `regex`, `once_cell`, `hex`, `rand`, `anyhow`, `thiserror`, `dirs` | MIT OR Apache-2.0 |
| `subtle` | BSD-3-Clause |

**npm 包** (直接依赖):

| Package | License |
|---------|---------|
| `@tauri-apps/api`, `@tauri-apps/cli`, `@tauri-apps/plugin-fs` | Apache-2.0 OR MIT |
| `react`, `react-dom`, `@types/react`, `@types/react-dom` | MIT |
| `@vitejs/plugin-react`, `vite`, `zustand`, `eslint`, `prettier` | MIT |
| `typescript` | Apache-2.0 |

**Python** (`hooks/shikigami-hook.py`、`scripts/install-hook.py`、`characters/linh-pixel/src/generate.py`): 仅使用 Python 3 stdlib + `Pillow` (HPND License —— permissive、stdlib 兼容)。

### 资产

- **应用图标** (`src-tauri/icons/*`): 由 `src-tauri/icons/` recipe 在构建时程序化生成。在 macOS 上用系统字体 (Hiragino Sans,与 macOS 捆绑;输出位图是可分发 derivative) 渲染日文字符 `式` (表意字,无版权)。v1.0 release 前替换。
- **参考图像** 在 `characters/linh/reference/` 下: 开发期间通过 OpenAI 图像生成工具产生。OpenAI 服务条款赋予用户对生成输出的所有权,允许商用;此处仅作艺术家参考,不在运行时 bundle 中发布。

### 灵感 (参考,非复制)

Shikigami 在架构上从一些开源项目获得灵感:

- **[airi by moeru-ai](https://github.com/moeru-ai/airi)** (MIT) —— plugin-protocol 身份模式、`[EMOTION:x]` prompt-tag 想法、"Soul vs Stage" 分离、pipeline 阶段命名。完整审计见 [`docs/research/260422-airi-reusability-analysis.md`](docs/research/260422-airi-reusability-analysis.md)。**没有代码从 airi 复制到本仓库。** 模式和概念在独立实现下使用。
- **VSCode 扩展格式** —— `.shikigami` zip 包布局的灵感。
- **Live2D Cubism SDK** —— 显式排除在核心运行时之外 (ADR-000);延后到独立 repo 中的可选 add-on,以保持 truly-OSS 状态。

### License 兼容性结论

Shikigami 可以在 **MIT** 下发布和重新分发,无需任何 carve-out 或超出标准 OSS 致谢的特殊归属。所有依赖都 permissive。所有灵感都是独立实现的模式级别。所有资产 pipeline 都使用我们自己的程序化代码或授予输出所有权的 AI 工具。

如果你发现归属缺口或 license 兼容性问题,请提 issue。

---

## 链接

- **仓库**: https://github.com/hoangperry/shikigami
- **Issues**: https://github.com/hoangperry/shikigami/issues
- **CI**: https://github.com/hoangperry/shikigami/actions

---

*"她看着你的 agent 实际做什么。她以真实回应。由代码召唤,扎根于事件。"*
