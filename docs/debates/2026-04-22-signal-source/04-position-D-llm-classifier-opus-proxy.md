# Position D — LLM Classifier

**Defender**: 🔴 Codex CLI *(dispatched but timed out; Claude Opus wrote this proxy position preserving the LLM-classifier arguments from prior context)*
**Thesis**: Reject both event-driven and text-parse. Neither scales. Run a small local quantized LLM to classify agent output semantically — the only approach that generalizes.

---

## 1. Steelman

**Position A (Event-Driven Primary, ADR-002):** The strongest case for events is determinism. A `tool_complete exitCode 0` payload is a cryptographic fact, not an interpretation. It is free of prompt-injection risk, free of locale drift, free of model sampling variance. An event stream with a typed schema gives you *guaranteed* correct state mapping for the 70% of agent activity that is tool-mediated. You cannot argue with an exit code. You can argue with anything parsed from text. For a product whose failure mode is "smiling while the database is wiped," starting from ground-truth signals is the risk-minimizing move.

**Position B (Text-Parse Primary, v0.1 original):** Position B correctly observes that events are emotionally flat. `exitCode 0` tells you success happened; it tells you nothing about the character's *relationship* to the success. Shikigami is a character product, not a Datadog dashboard. The emotional payload lives in phrasing — in the hedging of "I think this is fixed," in the exasperation of "I keep hitting this," in the relief of "finally." A character that cannot read its own speech is a marionette with a teleprompter — animated but not present. Text-parse at least *tries* to honor that.

**Position C (Equal Hybrid):** The fusion-classifier move is architecturally elegant. Events and text carry orthogonal information, and a weighted combiner lets the product express complex states (relieved exhaustion, smug success, nervous completion) that neither pure channel can produce alone. By avoiding both regex fragility and LLM inference cost, Position C hits a performance sweet spot that Position A's reduction and Position D's classifier both sacrifice. The "tunable emotional mirror" framing — where users dial weights toward logical or empathetic — is a real feature, not a hack.

---

## 2. Thesis: Why the LLM Classifier Wins

All three competing positions are handcrafted pattern machines. They will all decay the same way: model providers will update output formats, new AI tools will emit new event schemas, locales and personas will fragment the input space, and the maintainers will spend eternity updating regex libraries and adapter adapters. This is the Clippy failure mode — rule-based systems that were impressive until language moved.

A quantized small LM — Phi-3-mini (3.8B @ 4-bit ≈ 2.2GB), Qwen2.5-0.5B-Instruct (≈ 400MB @ 4-bit), or Gemma-2-2B — collapses all three rival approaches into a single inference call that outputs a canonical state token plus confidence. The model reads the same corpus the user sees and classifies semantically. It disambiguates "I'm sorry, I can't do that" (refusal → `warning`) from "I'm sorry about the bug" (apology → `shy`). It handles negation, sarcasm, irony — things regex literally cannot represent and event streams cannot observe. It generalizes across unseen personas without new pattern libraries. It generalizes across unseen AI tools without new adapters.

**Local-first, no cloud.** Open-weight models ship with permissive licenses (Apache 2.0 Qwen, MIT Phi). No proprietary SDK like Live2D. No vendor lock. No telemetry leakage. The weights live in the add-on package and never touch the network.

**Training-free.** The classifier uses prompt engineering only. A 200-token system prompt teaches the model the 9 canonical states + severity vocabulary. No fine-tuning, no GPU rig, no dataset curation. Ship week one.

**Future-proof.** When Cursor v4 arrives with a new event schema, Position A writes a new adapter. When Cursor v4 arrives with new output phrasing, Position B writes new regex. When Cursor v4 arrives, Position D does nothing — the model already reads natural language.

---

## 3. Killer Counter

The strongest attack is the **dependency-footprint critique**: "A 500MB–2.2GB model violates your <80MB RAM idle / <200MB peak budget by 10×. This is not a desktop companion anymore; it is an inference stack with a cartoon glued on."

This is real. I will not pretend the budget numbers in PRD v0.2 survive Position D as-is. The defense is architectural, not rhetorical.

**Three-tier deployment.** The classifier runs in a *separate sidecar process* (`shikigami-mind`), not in the Tauri window. The overlay stays at <80MB RAM and 0% idle CPU. The sidecar uses llama.cpp with `mlock` off and lazy weight loading — idle footprint ≈ 100MB (weights memory-mapped, not resident). Peak during classification ≈ 1.5GB for ~150ms per inference. Classifications are debounced: no more than 1 per 500ms. Total amortized CPU <3% at normal session rates.

**Optional, not required.** Ship the classifier as `shikigami-mind` add-on, not bundled. Users who want it install it; the core product still works on event-driven primary (Position A) for those who refuse the footprint. This collapses the attack: for your 95% corpus, Position A runs; for the persona-power-user 5%, Position D runs; text-parse becomes obsolete. Position A users pay zero new cost. Position D users pay the cost consciously.

The budget objection evaporates the moment you stop treating the classifier as monolithic. Position A, B, C all require constant pattern maintenance forever. Position D requires one model swap per year. Which engineering burden scales?
