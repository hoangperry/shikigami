# Debate Arena — PRD v0.2 Signal Source

**Date**: 2026-04-22
**Topic**: Which signal source should Shikigami prioritize — structured events, output-text parsing, both equally, or something else entirely?
**Format**: Steelman Tournament (4 positions, adversarial tone)
**Rounds**: Single-round per participant (each writes steelman + position + pre-emptive defense)
**Synthesis by**: Claude Opus (moderator)

## Four Positions Assigned

| Position | Thesis | Defender |
|----------|--------|----------|
| **A — Event-Driven Primary** (current ADR-002) | Characters react to tool calls / exit codes / git ops. Text-parse is opt-in fallback only. | 🔵 Claude Opus |
| **B — Text-Parse Primary** (PRD v0.1 original) | Text patterns (kaomoji, action text, keywords) are the soul of the product. Reverting ADR-002. | 🟣 Sonnet Subagent |
| **C — Equal Hybrid** | Neither dominates. A fusion classifier combines structured events + text signals in real time. | 🟡 Gemini CLI |
| **D — Reject Both: LLM Classifier** | Both regex and event-mapping are brittle. Run a small local LLM to classify output semantically. | 🔴 Codex CLI |

## Rules

1. Each defender writes ONE message containing:
   - **Steelman**: articulate the strongest version of EACH opposing position (3 paragraphs, one per opponent)
   - **Thesis**: defend your own position with concrete arguments (1–2 paragraphs)
   - **Killer Counter**: identify the ONE attack that would most damage your position, then pre-emptively defend against it
2. Adversarial tone is encouraged — no politeness at the expense of clarity.
3. Use concrete examples from PRD v0.2 and the adversarial reviews.
4. Word budget: 500–700 words per defender.
5. Moderator (Claude Opus) synthesizes winner, acknowledges valid attacks, updates ADR-002 if needed.
