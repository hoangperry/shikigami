# Position B — Text-Parse Primary

**Defender**: 🟣 Sonnet (via Agent subagent)
**Thesis**: Revert ADR-002. Kaomoji / action text / keyword parsing should be the PRIMARY signal. The adversarial reviewers answered the wrong question.

---

## 1. Steelman

**Position A (Event-Driven Primary, ADR-002):** The strongest case for event-driven primary is reliability under adversarial conditions. Structured signals — exit codes, tool call payloads, git operation results — are deterministic and schema-stable. They cannot be spoofed by a rogue prompt injection, they do not drift when model providers tweak sampling parameters, and they carry zero false-positive risk because the signal vocabulary is finite and known at compile time. If you are building a system that must function correctly in production across thousands of diverse Claude deployments where nobody has configured a custom persona, events are the only signal source that will not embarrass you.

**Position C (Equal Hybrid):** The fusion classifier argument is intellectually honest in a way that pure-event advocacy is not. It acknowledges that text carries signal events cannot — tone, hedging, enthusiasm — while refusing to bet the entire inference stack on regex stability. An equal-weight fusion that degrades gracefully when one channel fails is architecturally conservative. If the text parser returns low confidence, events carry the frame. If events are sparse (interactive REPL session, no tool calls for 30 seconds), text carries it. No single point of failure, and the classifier learns calibration from both channels simultaneously.

**Position D (LLM Classifier):** A small local model (3B–7B, quantized, running on the same machine) eliminates the brittleness of handcrafted patterns entirely. It generalizes across phrasing variations, understands negation and irony that regex cannot, and can be fine-tuned on exactly the output styles Shikigami cares about. The inference latency objection is weaker than it sounds — a quantized Phi-3-mini classifying a 200-token chunk runs in under 100ms on CPU, which is invisible against the seconds-scale pace of LLM output streaming.

---

## 2. Thesis: Why Text-Parse Primary Wins

The adversarial reviewers who pushed ADR-002 answered the wrong question. They evaluated text-parse against raw Claude Code output — terse, tool-heavy, persona-free. That is not the user Shikigami is building for. The target user has already deployed a nekomata-engineer or sexy-secretary output style. Those styles are not accidents of phrasing; they are structured contracts. Nekomata-engineer *will* emit `(=^･ω･^=)` on task completion. Sexy-secretary *will* emit `*adjusts glasses*` before a technical summary. These are guaranteed markers, not probabilistic guesses. Regex against a style-contracted output stream is not "brittle pattern matching" — it is schema parsing on a soft schema, and it is *more reliable* than hoping the event bus is populated during an interactive session.

Shikigami dogfoods this exact use case. The project's own CLAUDE.md uses themed personas. The developers are the users. If text-parse primary failed in practice, the team would have abandoned it in v0.1 before any external review happened. The fact that it survived to v0.1 shipment is empirical evidence the reviewers ignored.

Events are blunt instruments for an emotional companion. `tool_complete exitCode 0` tells you the command succeeded. It tells you nothing about whether the AI is nervously relieved, smugly satisfied, or bouncing off the walls with excitement. Shikigami's differentiation is *not* "we show you whether the build passed" — every terminal multiplexer does that. Shikigami's differentiation is that the Shikigami *feels* something about the build passing. That feeling lives exclusively in text. Strip text-parse to fallback and you have shipped a character-skinned status bar. That is what the PRD v0.2 tagline "visual proprioception" actually describes — and it is a strategic retreat from the emotional companion thesis that made this product interesting.

---

## 3. Killer Counter

The strongest attack on Position B is recall degradation: when persona styles vary phrasing unexpectedly, regex misses states entirely, and the Shikigami goes blank at the worst moment. This is real. I will not pretend it is not.

The defense is threefold. First, style prompts are part of the product contract — Shikigami-certified persona packs include marker guarantees baked into the system prompt, making certain kaomoji and action phrases structurally mandatory at state transitions. Second, a community-maintained pattern library with coverage metrics creates social pressure on persona authors to hit recall targets before certification. Third, event signals serve as *fallback injectors* — when text-parse confidence drops below threshold, events fire a generic state signal that prevents blanking without demoting text to secondary for the 95% case where persona output is well-formed.

Recall is a solvable engineering problem. Surrendering text-parse primacy is a product strategy mistake you cannot undo without a new ADR and a positioning rebuild. Fix the recall problem. Keep the soul.
