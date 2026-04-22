# Position C — Equal Hybrid

**Defender**: 🟡 Gemini CLI
**Thesis**: Neither dominates. A fusion classifier combines structured events + text signals in real time. Events = skeleton (factual), Text = flesh (emotional).

---

In this tournament of architectural trade-offs, Position C rejects the reductionist hierarchy of "Primary/Secondary" signals. We argue that the Shikigami's "soul" — its ability to react with high-fidelity empathy to a developer's workflow — is lost the moment you prioritize the dry "what" of an event over the nuanced "how" of the text, or vice-versa.

## 1. Steelman

**Position A (Event-Driven Primary): The Logic of Ground Truth.** The strongest argument for Position A is its unassailable reliability and zero-latency execution. By hooking into git operations, tool exit codes, and LSP diagnostics, the system operates on "ground truth." If a test suite fails with exit code 1, that is a binary reality. Position A eliminates the fragility of "hallucinated" intent; it ensures the character never dances while the build is burning. It is the most computationally efficient path, perfectly honoring the <80MB RAM idle budget by avoiding expensive string processing until absolutely necessary.

**Position B (Text-Parse Primary): The Direct Line to Personality.** Position B correctly identifies that the LLM's output is the only place where the "persona" actually resides. If a model emits a sigh or a specific kaomoji, that is a deliberate expressive choice that transcends a tool call. The strongest version of this argument is that a character reacting only to events is just a glorified status bar with a face. Text-parsing captures the *intentionality* of the agent, allowing the Shikigami to mirror the character's internal state — be it snarky, helpful, or exhausted — which no git-hook can ever convey.

**Position D (LLM Classifier): The Semantic Sentinel.** Position D argues that human (and AI) interaction is too complex for regex or exit codes. A local LLM classifier is the only way to detect "the vibe." The steelman here is that "I fixed it" and "I think I finally fixed it, maybe?" represent radically different emotional states despite both being associated with a successful commit. Position D provides a high-fidelity emotional vector that understands sarcasm, frustration, and relief, ensuring the Shikigami's reactions are never "uncanny" or misaligned with the developer's actual experience.

## 2. Thesis: Why Equal Hybrid Wins

Position C, the **Equal Hybrid**, is the only architecture that recognizes that events and text are not redundant; they are orthogonal. Events provide the **Skeleton** (the factual reality), while text provides the **Flesh** (the emotional context).

Consider a `git commit` event. Position A sees `exit 0` and triggers a "Success" animation. But if the commit message is *"I hate this codebase, fixing this stupid bug for the fifth time,"* a "Success" animation is a failure of empathy. Position B might catch the "hate" but miss the fact that the commit actually cleared a critical blocker.

We propose a **Fusion Classifier**. When an event is detected, it generates an `Event_Vector`. Simultaneously, the text stream is scanned for `Sentiment_Keywords`. We apply an equal weighting (e.g., 0.5/0.5). If they align (Success Event + Positive Text), the reaction is intensified. If they conflict (Success Event + Negative Text), we trigger a **Complex State**: the Shikigami looks relieved but exhausted, perhaps wiping sweat from its brow. This creates a "living" character rather than a binary toggle.

This hybrid approach is vastly more robust than Position B's brittle regex, yet far more grounded than Position A's "soulless" event tracking. It respects the performance budget by avoiding Position D's LLM inference, using a lightweight "Fusion Weight" scheme that can be calculated in microseconds, providing 80% of the semantic depth of an LLM classifier at 1% of the CPU cost.

## 3. Killer Counter

The most damaging attack against Position C is the **"Magic Number Fallacy"**: that our fusion weights (50/50 or 60/40) are arbitrary "magic numbers" that will lead to unpredictable, "mushy" character behavior that is impossible to debug. Critics will say that when signals conflict, the hybrid approach will result in a character that looks confused or does nothing, frustrating the user.

**The Defense:** Ambiguity is not a bug; it is a feature of high-fidelity character design. A developer's life is rarely 100% "Happy" or "Sad." It is a spectrum of "Relieved Frustration" and "Anxious Progress." By using a Fusion Classifier, we move away from "Reaction A vs. Reaction B" and toward a **State-Space Model**. Our "magic numbers" are actually **Calibration Parameters**. If a user wants a more "logical" Shikigami, they tilt the weight toward Events; if they want a more "empathetic" companion, they tilt toward Text. This flexibility transforms the Shikigami from a hard-coded script into a tunable emotional mirror, making it the only position that actually fulfills the PRD's vision of a "companion" rather than a tool.
