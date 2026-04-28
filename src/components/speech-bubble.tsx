// Contextual info overlay near the character — anti-UX-fatigue utility.
// Emerges on specific ResolvedState events (destructive op, failed tool,
// commit) with a short message, then fades away.
//
// Trigger rules live here in `pickBubble`. Critical-severity bubbles stay
// until manually dismissed (click anywhere); others auto-fade after a TTL.

import { useEffect, useState } from "react";

type ResolvedState = {
  dominant: string;
  texture: string | null;
  severity: string;
  duration_ms: number;
  event_id: number;
};

type BubbleContent = {
  text: string;
  accent: "info" | "success" | "warning" | "critical";
  /** Milliseconds before auto-dismiss. null = sticky (user must click). */
  ttl: number | null;
  key: number;
};

type Props = {
  state: ResolvedState | null;
  /** Optional extra info fields that future events can carry. */
  lastEventText?: string | undefined;
  /** Currently-spoken TTS text. Takes priority over state-driven bubbles. */
  spokenText?: string | undefined;
  /** Monotonic id so the same text re-fires the bubble effect. */
  spokenKey?: number | undefined;
};

function pickBubble(
  state: ResolvedState | null,
  lastEventText?: string,
  spokenText?: string,
  spokenKey?: number,
): BubbleContent | null {
  // TTS-driven bubble takes priority — Hiyori is "speaking" right now.
  if (spokenText && spokenText.trim().length > 0) {
    return {
      text: spokenText,
      accent: "info",
      // Rough estimate: ~12 chars/sec spoken, min 3s, max 12s.
      ttl: Math.min(12000, Math.max(3000, spokenText.length * 80)),
      key: spokenKey ?? 0,
    };
  }
  if (!state) return null;
  if (state.severity === "critical") {
    return {
      text: `⚠ ${lastEventText?.slice(0, 80) ?? "Destructive operation detected"}`,
      accent: "critical",
      ttl: null,
      key: state.event_id,
    };
  }
  if (state.dominant === "warning") {
    return {
      text: "tool failed — check logs",
      accent: "warning",
      ttl: 7000,
      key: state.event_id,
    };
  }
  if (state.dominant === "happy" && state.texture === "relieved") {
    return {
      text: "✓ finally done",
      accent: "success",
      ttl: 3500,
      key: state.event_id,
    };
  }
  // All other states produce no bubble.
  return null;
}

const accentColors: Record<BubbleContent["accent"], string> = {
  info: "rgba(58, 110, 165, 0.92)",
  success: "rgba(58, 140, 90, 0.92)",
  warning: "rgba(200, 130, 50, 0.94)",
  critical: "rgba(220, 60, 60, 0.95)",
};

export function SpeechBubble({ state, lastEventText, spokenText, spokenKey }: Props) {
  const [current, setCurrent] = useState<BubbleContent | null>(null);
  const [visible, setVisible] = useState(false);

  // React to state changes: pick a bubble, if any.
  useEffect(() => {
    const next = pickBubble(state, lastEventText, spokenText, spokenKey);
    if (!next) {
      // Let existing bubble fade naturally via its own TTL.
      return;
    }
    setCurrent(next);
    setVisible(true);
    if (next.ttl !== null) {
      const t = window.setTimeout(() => setVisible(false), next.ttl);
      return () => window.clearTimeout(t);
    }
    return undefined;
  }, [
    state?.event_id,
    lastEventText,
    state?.severity,
    state?.dominant,
    state?.texture,
    spokenText,
    spokenKey,
  ]);

  if (!current) return null;

  return (
    <div
      onClick={() => setVisible(false)}
      style={{
        position: "absolute",
        top: 20,
        left: "50%",
        transform: `translate(-50%, ${visible ? "0" : "-8px"})`,
        maxWidth: "85%",
        padding: "7px 12px",
        borderRadius: 10,
        background: accentColors[current.accent],
        color: "#fff",
        fontSize: 11,
        lineHeight: 1.45,
        fontFamily:
          "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        boxShadow: "0 4px 14px rgba(0,0,0,0.25)",
        opacity: visible ? 1 : 0,
        transition: "opacity 220ms ease, transform 220ms ease",
        zIndex: 20,
        pointerEvents: visible ? "auto" : "none",
        cursor: current.ttl === null ? "pointer" : "default",
        whiteSpace: "pre-wrap",
      }}
      title={current.ttl === null ? "Click to dismiss" : undefined}
    >
      {current.text}
    </div>
  );
}
