import { useEffect, useState } from "react";

// Phase 0 scaffold — minimal debug panel for the transparent overlay.
// Phase 2 will replace this with the PixiJS sprite renderer.
export function App() {
  const [state, setState] = useState<string>("idle");
  const [lastEventAt, setLastEventAt] = useState<string | null>(null);

  useEffect(() => {
    // TODO (Phase 1): subscribe to Tauri 'state_changed' event
    // import { listen } from "@tauri-apps/api/event";
    // const unlisten = await listen("state_changed", (e) => setState(e.payload as string));
  }, []);

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
          <code>{state}</code>
        </div>
        <div
          style={{ fontSize: 11, opacity: 0.5, marginTop: 6 }}
        >
          {lastEventAt
            ? `last event @ ${lastEventAt}`
            : "waiting for events…"}
        </div>
      </div>

      {/* DEV: inline dev-tester buttons. Remove once Phase 1 transport lands. */}
      <div style={{ display: "flex", gap: 6, opacity: 0.7 }}>
        {["idle", "happy", "focused", "warning", "sleepy"].map((s) => (
          <button
            key={s}
            onClick={() => {
              setState(s);
              setLastEventAt(new Date().toLocaleTimeString());
            }}
            style={{
              padding: "4px 10px",
              fontSize: 11,
              background: "rgba(240,240,240,0.18)",
              color: "#f0f0f0",
              border: "1px solid rgba(240,240,240,0.25)",
              borderRadius: 6,
              cursor: "pointer",
            }}
          >
            {s}
          </button>
        ))}
      </div>
    </div>
  );
}
