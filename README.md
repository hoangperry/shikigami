# 式神 · Shikigami

> A summoned desktop companion that reflects your Claude Code session's emotional state through a 2D animated character, displayed as a Picture-in-Picture overlay.

## ✨ Concept

**Shikigami** (式神) — in Japanese folklore, a spirit summoned to serve its master. Here, she's a 2D animated character who lives on your desktop, reacting in real-time to your AI coding assistant's responses.

- 🎭 **Emotion-reactive** — parses Claude Code output (kaomoji, action text, keywords) and maps to expression states
- 🪟 **Picture-in-Picture** — transparent, always-on-top overlay that doesn't interrupt your workflow
- 🎨 **Custom personas** — swap characters to match different Claude Code output styles
- 🔌 **Hook-driven** — integrates with Claude Code's `Stop` / `PostToolUse` hooks

## 🏗️ Architecture

```
Claude Code Response
        ↓
   [Hooks System]   ← PostToolUse / Stop / UserPromptSubmit
        ↓
  [Emotion Parser]  ← regex detect kaomoji + *action text* + keywords
        ↓
   WebSocket / IPC
        ↓
  [Animation App]   → PiP window with transparent background
```

## 🎭 Emotion States

| State | Triggers | Animation |
|-------|----------|-----------|
| `idle` | default | Breathing loop, blink |
| `happy` | `♡`, `(｡♡‿♡｡)`, `✓ Done` | Smile, sparkle eyes |
| `focused` | `(๑•̀ᴗ-)✧`, `*typing*` | Serious, typing |
| `shy` | `♡///♡`, `(/ω＼)` | Blush, cover face |
| `confused` | `(⊙﹏⊙)`, `(@_@;)` | Tilt head, ? bubble |
| `flirty` | `( ˶ˆᗜˆ˵ )`, wink action | Wink, smirk |
| `warning` | `⚠️`, danger keywords | Raise hand, serious |
| `overloaded` | `(°ー°〃)`, logic conflict | Spinning eyes |

## 🚀 Status

🚧 **Early development** — brainstorming & scaffolding phase.

## 📋 Roadmap

- [ ] **Phase 1 — MVP**: Electron shell + WebSocket server + 3 sprite states
- [ ] **Phase 2 — Emotion Engine**: Regex parser + state machine
- [ ] **Phase 3 — Character Assets**: Live2D rig or sprite sheets
- [ ] **Phase 4 — Polish**: Idle animations, settings UI, multi-persona support

## 📜 License

TBD (currently private — will open-source once MVP is stable)

---

*"She watches. She listens. She reflects. Summoned by code, animated by soul."*
