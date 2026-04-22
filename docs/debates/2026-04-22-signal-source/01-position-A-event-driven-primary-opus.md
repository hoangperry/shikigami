# Position A — Event-Driven Primary (Current ADR-002)

**Defender**: 🔵 Claude Opus (moderator defending own position)
**Thesis**: Structured events are the load-bearing signal source. Text-parse is opt-in enhancement, not a peer.

---

## 1. Steelman

**Position B (Text-Parse Primary):** The strongest case is that Shikigami's differentiation comes from *personality*, not *telemetry*. A character that only reacts to `exitCode 0` is a status bar in costume. The emotional payload lives in phrasing — hesitation, enthusiasm, kaomoji markers emitted by style-contracted output. Position B rightly notes that the reviewer pool evaluated text-parse against the *wrong corpus* (raw Claude Code), not the actual user surface (themed output styles where markers are guaranteed). If the project dogfoods themed personas and the markers survive shipment, empirical signal exceeds theoretical fragility.

**Position C (Equal Hybrid):** The fusion-classifier framing is the most architecturally sophisticated move on the table. Events carry ground truth but no tone; text carries tone but no guarantees. Weighted fusion with conflict-detection lets the character express *complex states* (relieved exhaustion, anxious success) that neither pure channel can produce. Treating signals as orthogonal rather than hierarchical is a genuine intellectual advance over both Position A and Position B, and the "tunable emotional mirror" framing gives the product a configuration axis with real user value.

**Position D (LLM Classifier):** A small local classifier eliminates regex brittleness *and* the event-schema adapter burden simultaneously. Modern quantized 0.5B–3B models hit 80% accuracy on sentiment/intent tasks at <100ms CPU latency. The dependency size (~500MB) is a one-time cost amortized across thousands of sessions. Most importantly, Position D is the only approach that scales to unknown AI tools without writing new adapters — a quality Shikigami will desperately want at v0.4 when Cursor and Windsurf enter scope.

---

## 2. Thesis: Why Event-Driven Primary Wins

Three hard facts demolish the alternatives.

**First: the 95% corpus reality.** Shikigami's addressable v0.1 user base is not "users who run nekomata-engineer." It is *every Claude Code user* who installs the hook. Most will run default output. For them, text-parse returns `idle` 95% of the time. A product that only works for users running a custom style the *author also happens to use* is a hobby project, not open source. Position B's "dogfood means it works" argument is survivorship bias: we confirmed it works on a corpus of one. The adversarial reviewers were right to challenge this.

**Second: context-correctness is non-negotiable for a character product.** When Claude runs `rm -rf` inside a tool call, the event bus reports `tool_start: Bash` with a destructive-op flag *before* any text emerges. Event-driven primary lets Shikigami look alarmed *immediately*. Position B cannot do this — the text warning ("⚠️ this is destructive") arrives *after* the action begins, if it arrives at all. Gemini's "smiling while the database is wiped" attack is fatal to text-primary architectures, full stop.

**Third: adapter scalability.** Every new integration (Cursor, Windsurf, future tools) emits *different* text. Regex libraries per tool multiply maintenance burden linearly. Events, by contrast, normalize at the adapter boundary: Cursor's `diagnostics.error` maps to the same `error` event as Claude Code's `PostToolUse` with non-zero exit. Write the adapter once, reuse state machine forever. Position B scales O(tools × personas × locales). Position A scales O(tools).

Text-parse is not *wrong*. It is *not primary*. It is the delight layer — the reason the `nekomata` user sees her Shikigami blush when the code review lands a compliment. The event layer is the load-bearing skeleton that makes sure the blush never happens during a production outage.

---

## 3. Killer Counter

The sharpest attack on Position A is Gemini's: **"Events tell you WHAT but miss HOW — you have shipped a character-skinned status bar."** If a user's only reaction vocabulary is `idle / focused / happy / warning / sleepy`, the Shikigami *is* a status indicator with personality art. Position C is right that there is a reductive collapse happening here.

Defense: Position A does not forbid nuance — it localizes it. The event stream is the *primary* signal, which means it determines the *dominant* state. Within each state, text-parse can modulate *texture*: a `happy` state triggered by `git_commit` is neutral by default, but if the commit message contains `"finally"` or `"phew"`, a `relieved_happy` sub-state activates. The architecture is event-gated, text-textured. This is precisely the skeleton-flesh metaphor Gemini described, but organized as a pipeline rather than a fusion classifier — events fire first and constrain the search space, then text refines within that space. The result: characters express nuance (satisfying Position B and C) without ever expressing the *wrong dominant state* (which is the failure mode Position A exists to prevent).

The equal-hybrid proposal is seductive but exposes the Shikigami to *signal ambiguity catastrophes*: two orthogonal signals of equal weight can deadlock or oscillate. Hierarchy eliminates this. Events win tie-breaks. Text decorates the winner. That is Position A, properly stated.
