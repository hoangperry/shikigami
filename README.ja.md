# 式神 · Shikigami

**言語**: [English](README.md) · [Tiếng Việt](README.vi.md) · **日本語** · [简体中文](README.zh-CN.md)

> AIエージェントのリアルタイム状態を反応的な2Dキャラクターで表現し、透明なPicture-in-Pictureオーバーレイとして表示する、召喚されたデスクトップコンパニオン。

**ステータス**: `v0.1.0-alpha` · 計画完了 + イベントエンジン出荷済み · キャラクターレンダラー進行中
**プラットフォーム**: macOS (Apple Silicon + Intel、メイン) · Windows (alpha — 未署名、透明性未検証) · Linux (alpha — deb/rpm/AppImage、Wayland透明性未検証)
**現在の統合**: Claude Code (メイン) · Codex CLI (alpha) · Cursor (alpha、5イベント最小) · Windsurf (alpha、ドキュメント由来スキーマ — ライブペイロード未検証) · Copilot Chatはv0.4マイルストーンで追跡中

---

## 何をするのか

エージェント型AIコーディングセッションは大量のイベント — ツール呼び出し、エラー、コミット、長時間ビルド — を生成しますが、ターミナルはテキストの壁しか提供しません。Shikigamiはあなたのデスクトップに小さな透明ウィンドウとして座り、エージェントが実際に何をしているかにリアルタイムで反応するアニメーションキャラクターを表示します。グリーンビルドなら微笑み、テスト失敗なら心配し、うっかり`rm -rf`したら、Enterを押す前にアラーム状態にロックします。

これはチャットボット、VTuberリグ、音声コンパニオンでは*ありません*。**エージェント型ワークフローのための視覚的固有感覚** — 個性のあるステータスインジケーターです。

---

## なぜ存在するのか

長時間のAIコーディングセッションは認知的に重く、視覚的に平坦です。既存エコシステムは「真面目」(機能的だが存在感ゼロのIDEステータスバー) と「楽しい」(存在感はあるがエージェント認識ゼロのデスクトップペット / VTuberリグ) に分かれています。Shikigamiはこれらを橋渡しします: AIツールからの構造化イベントに根ざしたキャラクター、装飾的なテキストパターンではなく。

正しさのために設計は意図的に狭めています。製品の完全な根拠は[`docs/PRD.md`](docs/PRD.md)を参照してください。

---

## クイックスタート

```bash
# 1. クローン
git clone https://github.com/hoangperry/shikigami.git
cd shikigami

# 2. フロントエンド依存をインストール
pnpm install

# 3. 開発ビルドを実行 (透明な常時最前面ウィンドウが開く)
pnpm tauri:dev

# 4. 別のターミナル: Claude Code フックを登録
python3 scripts/install-hook.py install

# 5. Claude Code を通常通り使用 — キャラクターがツールコールごとに反応
```

ヘルスチェック:

```bash
python3 scripts/install-hook.py doctor
```

Claude Codeなしでテスト (手動イベント):

```bash
TOKEN=$(cat ~/.shikigami/token)
PORT=$(jq -r .port ~/.shikigami/config.json)
curl -X POST "http://127.0.0.1:$PORT/v1/events" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"schemaVersion":"1.0","source":"claude-code","type":"git_commit","text":"fix critical bug, finally"}'
# キャラクターが happy_relieved にアニメーション
```

フックをアンインストール:

```bash
python3 scripts/install-hook.py uninstall
```

### Codex CLI (alpha)

OpenAIのCodex CLIはClaude Codeと同一のフックデリバリーモデルを出荷しているため、同じ`hooks/shikigami-hook.py`スクリプトが両方を処理します — EventPayloadの`source`フィールドだけが変わります。次のスニペットを`~/.codex/config.toml`に追加 (絶対パスはあなたのクローンで置き換え):

```toml
[hooks]
PreToolUse       = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
PostToolUse      = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
UserPromptSubmit = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
Stop             = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
SessionStart     = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
PermissionRequest = "python3 /absolute/path/to/shikigami/hooks/shikigami-hook.py --source codex"
```

`PermissionRequest`はCodex専用; warning状態にマップされ、permissionダイアログが開いている間アイドルに見える代わりに「エージェントが承認待ち」とキャラクターが示します。Codex TOML configの自動インストーラーは追跡中だが延期 (KISS — TOML書き込み依存性を導入するより手動ペーストの方が速い)。

### Cursor (alpha — 5イベント最小スコープ)

Cursor v1.7+は18+のエージェントイベントを持つ[包括的なフックシステム](https://cursor.com/docs/hooks)を出荷。現在は**Claude Codeのコアライフサイクルをミラーする5イベント**(`sessionStart`、`preToolUse`、`postToolUse`、`postToolUseFailure`、`stop`)をマップ; 残りのCursor固有イベント (`afterMCPExecution`、`preCompact`、`afterAgentThought` など) は実ユーザーがどれが重要か教えてくれるまで黙ってスキップ。寛容なトランスフォーマーパターン — Codexと同じ手法。

Cursorのフック設定に追加 (プロジェクトごと、`.cursor/hooks.json`、またはグローバル — ファイル位置は現在のCursorドキュメント参照):

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

Cursorのフィールド名はClaude Codeと若干異なる(`session_id`の代わりに`conversation_id`、`cwd`の代わりに`workspace_roots[0]`); スクリプトの`normalize_cursor()`がエッジで書き換え、パイプラインの残りはsource-agnosticのまま。

### Windsurf (alpha — ドキュメント由来スキーマ、ライブペイロード未検証)

Windsurfの[Cascade Hooks](https://docs.windsurf.com/windsurf/cascade/hooks)はツールタイプで分割された12イベント (`pre_run_command`、`pre_read_code`、`pre_write_code`、`pre_mcp_tool_use`、`pre_user_prompt`、`post_cascade_response` など) をネストした`tool_info`ペイロードと共に出荷。`normalize_windsurf()`がこれらをClaude形式にフラット化し、ブリッジの残りはsource-agnosticのまま。Shikigamiの既存タクソノミーに合う11イベントをマップ。

⚠️ **ステータス**: このブリッジはドキュメントのみから実装 — フィールド名を検証するためのライブWindsurfセッションからのペイロードサンプルは未使用。Windsurfを使用してイベントが黙ってドロップされたりフィールド名が間違っているのに気付いたら、Issueを立ててください。

`~/.codeium/windsurf/hooks.json` (またはプロジェクトごとの`.windsurf/hooks.json`、または上記リンクで文書化されているシステムパス) に追加:

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

検証を手伝いたい? `hooks/shikigami-windsurf-dumper.py`をブリッジと並行して(または代わりに) 登録 — 生のペイロードを`~/.shikigami/windsurf-payloads.jsonl`にキャプチャし、ドキュメント由来スキーマと比較できます。Issue [#34](https://github.com/hoangperry/shikigami/issues/34)にアップロード。

---

## .dmg のインストール (エンドユーザー)

`pnpm tauri:build`で作られる`Shikigami_*.dmg`は**Apple Development**証明書で署名されている (オリジナル作者のテストデバイスに適している) が、Appleのディストリビューションサービスでnotarizeされて**いません**。macOS Gatekeeperは初回起動時に「unidentified developer」プロンプトでブロックします。

新しいMacで通り抜ける2つの安全な方法:

**オプション1 — システム設定**
1. .dmgをダブルクリック、`Shikigami.app`を`/Applications`にドラッグ
2. アプリを開こうとする — macOSが拒否
3. *システム設定 → プライバシーとセキュリティ*を開く → 「Shikigami was blocked」通知までスクロール → **このまま開く**をクリック
4. 続くダイアログで確認

**オプション2 — ターミナル (より速い)**
```bash
xattr -cr /Applications/Shikigami.app
open /Applications/Shikigami.app
```
`xattr -cr`はインターネットからダウンロードされたすべてのファイルにセットされる`com.apple.quarantine`属性を削除します。署名自体は無傷 — これはGatekeeperにあなたがソースを信頼していると伝えるだけです。

将来のリリースでApple Developer Programの完全notarizationが入ればこのステップは不要になります。今日見える署名は本物; ただしまだnotarizeされていません:

```bash
codesign -dv /Applications/Shikigami.app
# Authority=Apple Development: Hoang Truong Nhat (Y44JV6U4Q9)
# Authority=Apple Worldwide Developer Relations Certification Authority
# Authority=Apple Root CA
```

---

## Windowsへのインストール (alpha)

WindowsビルドはmacOS DMGと同じリリースタグから**`.msi`** (システム全体、WiX) と**`.exe`** (ユーザーごと、NSIS) インストーラーとして出荷されます。このマイルストーンではWindowsバイナリは**未署名** — まだ$400/年のEV証明書のバックアップがない — ためSmartScreenは初回起動時に警告します。

```powershell
# 1. ReleaseページからShikigami_*_x64-setup.exe (NSIS) または .msi をダウンロード
# 2. 実行。SmartScreen警告 → 「詳細情報」 → 「実行」をクリック。
# 3. Python 3を持っていなければインストール (Claude Codeは既にPythonを必要とする)
# 4. フックを登録 (PATH上のpythonを使用、python3ではない):
python scripts\install-hook.py install

# 5. 検証
python scripts\install-hook.py doctor
```

⚠️ **v0.1 alpha Windowsビルドの既知のギャップ** (GitHub Issuesで追跡 — 貢献歓迎):
- 透明な常時最前面オーバーレイは**実Windowsハードウェアで未テスト** — TauriはWebView2を使用、DWMコンポジションとモニターごとのDPIに関する文書化されたquirksがあります。コントリビューターが検証 + チューニングするまで、黒色または不透明な背景でレンダーされる可能性があります。
- コード署名なし → 新規インストールごとにSmartScreen警告。
- フックブリッジはmacOSと同じ`shikigami-hook.py`を使用; PowerShellネイティブラッパー (`hooks/shikigami-hook.ps1`) も同梱、直接登録を好むユーザー向け。

現時点ではWindowsはベストエフォート: macOSがサポートされているalphaターゲット。

---

## Linuxへのインストール (alpha)

Linuxは同じリリースタグから3フォーマットを出荷 — distroの好みを選んでください:

```bash
# Debian / Ubuntu
sudo dpkg -i shikigami_*_amd64.deb
sudo apt-get install -f          # 不足しているGTK / WebKit依存を引き込む

# Fedora / RHEL / openSUSE
sudo rpm -i shikigami-*.x86_64.rpm

# distro非依存 (インストール不要)
chmod +x shikigami_*_amd64.AppImage
./shikigami_*_amd64.AppImage
```

フックセットアップはmacOSと同一:

```bash
python3 scripts/install-hook.py install
python3 scripts/install-hook.py doctor
```

⚠️ **v0.1 alpha Linuxビルドの既知のギャップ** (貢献歓迎):

- 透明な常時最前面オーバーレイは**実Linuxデスクトップで未テスト**。コンポジター (Xfwm、Mutter、KWin) のあるX11は動作する見込み; **Waylandは検証が必要** — 一部のコンポジター (古いGNOME、正しいプロトコルのないsway) はTauriが依存するサーフェスAPIを欠きます。
- `alwaysOnTop`セマンティクスはコンポジター間で異なる; 真の最上層レイヤーの代わりに通常の「above」ヒントに劣化する可能性があります。
- クリックスルーはコンポジターのinput-shapeサポートを必要とします。Tauriの抽象化はX11 + ほとんどのWaylandコンポジターで動作しますが、Shikigamiについてハードウェアrunで確認されたものはまだありません。

このマイルストーンではLinuxはWindowsと並んでベストエフォート; macOSがサポートされているalphaターゲット。

---

## 自分で署名済み.dmgをビルド

自分のApple署名IDを持っている場合 (`security find-identity -v -p codesigning`)、envで渡してください:

```bash
APPLE_SIGNING_IDENTITY="Apple Development: <あなたの名前> (<team-id>)" \
  pnpm tauri:build
```

実際のディストリビューションには**Developer ID Application**証明書 (Apple Developer Program、$99/年) とnotarization資格情報が必要 — [TauriのmacOSディストリビューションガイド](https://v2.tauri.app/distribute/sign/macos/)を参照。

---

## 仕組み

イベントは7段階のパイプラインを流れます:

```
Hook → Bridge → Ingest → Segment → Resolve → Emit → Render
 CC     Py      Rust     Rust      Rust      Rust    React+PixiJS
```

- **Bridge** (`hooks/shikigami-hook.py`) はClaude CodeフックJSONを型付き`EventPayload`に変換
- **Ingest** (`src-tauri/src/event/server.rs`) はベアラー認証で`127.0.0.1`でHTTP POSTを受信
- **Segment** (`src-tauri/src/state/dampen.rs`) は2秒スライディングウィンドウで繰り返しイベントをdedup
- **Resolve** (`src-tauri/src/state/machine.rs`) はHierarchical Fusionを適用: イベントがdominant stateを駆動、テキストモディファイアがテクスチャに重ねる、severityが持続時間をスケール
- **Emit** はフロントエンドに`state_changed` Tauriイベントを発火
- **Render** (Phase 2) はPixiJS経由でスプライトを描画

完全な詳細は[`docs/PIPELINE.md`](docs/PIPELINE.md) · アーキテクチャ決定は[`docs/adr/`](docs/adr/)。

---

## 機能

- 🪶 軽量: **Tauri 2**上に構築 (アイドル時 <80 MB RAMターゲット)
- 🧠 **イベント駆動状態**: 反応はエージェントが実際に行うこと (ツールコール、終了コード、git op) にマップされ、プロンプトエンジニアリングされたテキストパターンではない
- 🛡️ **重大度認識**: `rm -rf`、`DROP TABLE`、`git push --force` のような破壊的操作は、テクスチャ抑制を伴うクリティカル警告状態にキャラクターをロックする
- 🎨 **2層感情システム**: 9 dominant state × 6 テクスチャモディファイア = `happy_relieved` や `focused_alarmed` のような表現的アニメーションキー
- 🔌 **拡張可能なキャラクター形式**: `.shikigami` zipパッケージ、OS間でポータブル
- 🔒 **100%ローカル**: テレメトリーなし、クラウドなし、proprietary依存なし (コア)
- 🔁 **トキシックループセーフ**: ダンパーがエラー繰り返し時のストロボを防ぐ

---

## ドキュメント

| Doc | 内容 |
|-----|------|
| [`docs/PRD.md`](docs/PRD.md) | プロダクト要件 v0.2 (post-review) |
| [`docs/TDD.md`](docs/TDD.md) | PRDをコードにマッピングする技術設計 |
| [`docs/PIPELINE.md`](docs/PIPELINE.md) | 7段階データフローのナラティブ |
| [`docs/CHARACTER-PLAYBOOK.md`](docs/CHARACTER-PLAYBOOK.md) | キャラクター制作ガイド + コミッション戦略 |
| [`docs/codex-ui-prompts.md`](docs/codex-ui-prompts.md) | GPT image-gen用のコピペプロンプト |
| [`docs/adr/`](docs/adr/) | 5つのアーキテクチャ決定レコード |
| [`docs/reviews/`](docs/reviews/) | Adversarialレビュー監査トレイル |
| [`docs/debates/`](docs/debates/) | Multi-AIトーナメント決定 |
| [`docs/research/`](docs/research/) | 外部リポジトリ再利用性分析 |

---

## キャラクターパッケージ

キャラクターはマニフェスト、スプライトフレーム、プレビュー、ライセンスを含む`.shikigami` zipバンドルとして出荷されます。[`docs/adr/003-character-package-format.md`](docs/adr/003-character-package-format.md)を参照。

### デフォルトキャラクター

| パッケージ | 目的 | ライセンス |
|-----------|------|----------|
| `characters/linh-pixel/` | 手続き型8ビット開発フィクスチャ | MIT |
| `characters/linh/` | プロダクションLinh (進行中、anime/vector) | 出荷時にCC-BY-SA-4.0 |

ピクセルフィクスチャは、プロダクションキャラクターがコミッション中の間、エンジニアリングが進められるように存在します。詳細は`characters/linh-pixel/README.md`参照。

### 自分で作る

テンプレートリポジトリと`shikigami pack` CLIはv0.2に計画されています。今のところ、[`schemas/manifest.v1.0.json`](schemas/manifest.v1.0.json)のマニフェストスキーマと[`docs/adr/003-character-package-format.md`](docs/adr/003-character-package-format.md)のフォーマット仕様を参照してください。

最小限のキャラクター: `idle` + `happy` 状態。欠落した状態は優雅にフォールバックします。

---

## プロジェクトステータス

| フェーズ | ステータス | ハイライト |
|---------|-----------|-----------|
| **Planning** | ✅ 完了 | PRD + TDD + 5 ADR + adversarialレビュー + 4-way debate |
| **Phase 0** | ✅ 基盤 | Tauri scaffold、透明オーバーレイ、CIワークフロー |
| **Phase 1** | ✅ イベントエンジン | HTTPサーバー + state machine + texture fusion + フックブリッジ |
| **Phase 2** | 🛠️ 進行中 | キャラクターローダー、PixiJSスプライトレンダラー |
| **Phase 3** | ✅ 出荷済み | Settings UI、speech bubble、システムトレイ、.dmgリリース |
| **v0.2 (Windows scaffolding)** | 🛠️ Alpha | MSI/NSISバンドル、CIマトリックス、フックスクリプト — オーバーレイ&署名はTBD |
| **v0.3 (Linux scaffolding)** | 🛠️ Alpha | .deb / .rpm / .AppImageバンドル、CIリリースジョブ — Wayland透明性TBD |
| **v0.4+ (アダプター)** | 🔬 リサーチ済み | Codex CLI ([#32](https://github.com/hoangperry/shikigami/issues/32)) · Cursor ([#33](https://github.com/hoangperry/shikigami/issues/33)) · Windsurf ([#34](https://github.com/hoangperry/shikigami/issues/34)) · Copilot Chat ([#35](https://github.com/hoangperry/shikigami/issues/35)) — サーベイは`plans/reports/researcher-260502-0814-ai-tool-adapter-survey.md` |

進捗トラッカー: [GitHub Issues](https://github.com/hoangperry/shikigami/issues)。

---

## 貢献

オープンソースの貢献を歓迎します。いくつかのポインター:

- 新しいアダプター (Cursor / Windsurf / ChatGPT): パイプラインの**Bridge**ステージのみ修正 — 下流はツール間で同一
- 新しい感情状態 / テクスチャ: `src-tauri/src/state/canonical.rs`に追加し、`schemas/manifest.v1.0.json`を更新
- キャラクターパック: [`docs/CHARACTER-PLAYBOOK.md`](docs/CHARACTER-PLAYBOOK.md)に従う; SPDX互換のpermissiveライセンスは何でも受け入れ可能 (スプライトはCC-BY-SA-4.0優先)

すべてのPRはCIをパスする必要があります (`cargo fmt`、`cargo clippy -D warnings`、`cargo test`、`pnpm typecheck`、スキーマ検証)。

---

## ライセンス & 帰属

### Shikigamiソースコード

**コード**: MIT ([`LICENSE`](LICENSE)参照 — 追加予定)。
**デフォルトスプライトキャラクター `linh-pixel`**: MIT (手続き型生成、私たち独自のコード)。
**プロダクションLinhキャラクター** (`characters/linh/`): アセットがファイナライズされた時点でCC-BY-SA-4.0で出荷。

### 依存ライセンス (監査済み)

すべての依存は**permissiveでMIT互換**。GPL、LGPL、proprietaryランタイムblobなし。

**Rust crate** (`cargo tree --depth 1`):

| Crate | License |
|-------|---------|
| `tauri`, `tauri-plugin-fs` | Apache-2.0 OR MIT |
| `tokio`, `axum`, `tower`, `tower-http`, `tracing`, `tracing-subscriber` | MIT |
| `serde`, `serde_json`, `regex`, `once_cell`, `hex`, `rand`, `anyhow`, `thiserror`, `dirs` | MIT OR Apache-2.0 |
| `subtle` | BSD-3-Clause |

**npm パッケージ** (直接依存):

| Package | License |
|---------|---------|
| `@tauri-apps/api`, `@tauri-apps/cli`, `@tauri-apps/plugin-fs` | Apache-2.0 OR MIT |
| `react`, `react-dom`, `@types/react`, `@types/react-dom` | MIT |
| `@vitejs/plugin-react`, `vite`, `zustand`, `eslint`, `prettier` | MIT |
| `typescript` | Apache-2.0 |

**Python** (`hooks/shikigami-hook.py`、`scripts/install-hook.py`、`characters/linh-pixel/src/generate.py`): Python 3 stdlib + `Pillow` (HPND License — permissive、stdlib互換) のみ使用。

### アセット

- **アプリアイコン** (`src-tauri/icons/*`): `src-tauri/icons/`レシピによりビルド時に手続き型生成。macOSのシステムフォント (Hiragino Sans、macOSにバンドル; 出力ビットマップは配布可能なderivative) を使用して日本の文字`式` (表意文字、著作権なし) をレンダー。v1.0リリース前に置き換え。
- **リファレンス画像** `characters/linh/reference/`下: 開発中にOpenAIの画像生成ツールで生成。OpenAI利用規約は商用利用許可された生成出力の所有権をユーザーに付与; アーティストリファレンスとしてのみ使用、ランタイムバンドルには出荷しない。

### インスピレーション (参照、コピーではない)

Shikigamiはいくつかのオープンソースプロジェクトからアーキテクチャインスピレーションを得ています:

- **[airi by moeru-ai](https://github.com/moeru-ai/airi)** (MIT) — プラグインプロトコルアイデンティティパターン、`[EMOTION:x]`プロンプトタグアイデア、「Soul vs Stage」分離、パイプラインステージ命名。完全な監査については[`docs/research/260422-airi-reusability-analysis.md`](docs/research/260422-airi-reusability-analysis.md)参照。**airiからこのリポジトリにコードはコピーされていません。** パターンとコンセプトは独立した実装で使用。
- **VSCode拡張機能フォーマット** — `.shikigami` zipパッケージレイアウトのインスピレーション。
- **Live2D Cubism SDK** — コアランタイムから明示的に除外 (ADR-000); truly-OSSステータスを保つために別リポジトリのオプショナルアドオンに延期。

### ライセンス互換性結論

Shikigamiは標準OSSクレジットを超えるカーブアウトや特別な帰属なしに**MIT**で出荷および再配布可能です。すべての依存はpermissive。すべてのインスピレーションは独立した実装によるパターンレベル。すべてのアセットパイプラインは私たち独自の手続き型コードまたは出力所有権を付与するAIツールを使用。

帰属ギャップやライセンス互換性の懸念があれば、Issueをオープンしてください。

---

## リンク

- **リポジトリ**: https://github.com/hoangperry/shikigami
- **Issues**: https://github.com/hoangperry/shikigami/issues
- **CI**: https://github.com/hoangperry/shikigami/actions

---

*"彼女はあなたのエージェントが実際に何をするかを見ている。彼女は真実をもって反応する。コードによって召喚され、イベントに根ざして。"*
