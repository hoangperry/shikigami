import { useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

type ResolvedState = {
  dominant: string;
  texture: string | null;
  severity: string;
  duration_ms: number;
  event_id: number;
};

export function App() {
  const [state, setState] = useState<ResolvedState | null>(null);
  const [history, setHistory] = useState<ResolvedState[]>([]);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const unlisten = await listen<ResolvedState>("state_changed", (e) => {
          setState(e.payload);
          setHistory((h) => [e.payload, ...h].slice(0, 10));
        });
        if (cancelled) {
          unlisten();
          return;
        }
        unlistenRef.current = unlisten;
      } catch (err) {
        console.warn("failed to subscribe to state_changed:", err);
      }
    })();
    return () => {
      cancelled = true;
      unlistenRef.current?.();
      unlistenRef.current = null;
    };
  }, []);

  const animationKey = state
    ? state.texture
      ? `${state.dominant}_${state.texture}`
      : state.dominant
    : "idle";

  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 12,
        fontFamily:
          "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
      }}
    >
      <div
        style={{
          padding: "18px 24px",
          borderRadius: 16,
          background: "rgba(20, 20, 30, 0.72)",
          backdropFilter: "blur(12px)",
          WebkitBackdropFilter: "blur(12px)",
          color: "#f5f5f5",
          fontSize: 14,
          lineHeight: 1.5,
          boxShadow: "0 4px 24px rgba(0,0,0,0.25)",
          textAlign: "center",
          minWidth: 240,
        }}
      >
        <div style={{ fontSize: 12, opacity: 0.7, marginBottom: 6 }}>
          Shikigami · v0.1.0-alpha.0
        </div>
        <div style={{ fontWeight: 600, fontSize: 20 }}>
          <code>{animationKey}</code>
        </div>
        <div style={{ fontSize: 11, opacity: 0.55, marginTop: 6 }}>
          {state
            ? `severity=${state.severity} · duration=${state.duration_ms}ms · #${state.event_id}`
            : "waiting for events…"}
        </div>
      </div>

      {history.length > 0 && (
        <details
          style={{
            padding: "6px 12px",
            fontSize: 11,
            color: "rgba(240,240,240,0.65)",
            maxWidth: 320,
          }}
        >
          <summary style={{ cursor: "pointer" }}>
            last {history.length} event{history.length > 1 ? "s" : ""}
          </summary>
          <ul style={{ margin: "4px 0 0 0", paddingLeft: 16 }}>
            {history.map((h) => (
              <li key={h.event_id} style={{ fontFamily: "monospace" }}>
                #{h.event_id} — {h.texture ? `${h.dominant}_${h.texture}` : h.dominant}
              </li>
            ))}
          </ul>
        </details>
      )}
    </div>
  );
}
