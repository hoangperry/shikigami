// Character stage — mounts the PixiJS renderer and keeps it synchronized
// with the latest ResolvedState received from the Rust backend.
//
// Phase 2 includes a visible DIAGNOSTIC PANEL that always shows the loaded
// character + last event so we can debug white-screen issues without
// opening DevTools. Remove once the PixiJS canvas is visibly rendering.

import { useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getActiveCharacter, listCharacters } from "../ipc/commands";
import type { ActiveCharacter, CharacterSummary } from "../ipc/commands";
import { SpriteRenderer } from "../renderer/sprite-renderer";

type ResolvedState = {
  dominant: string;
  texture: string | null;
  severity: string;
  duration_ms: number;
  event_id: number;
};

function animKey(state: ResolvedState): string {
  return state.texture ? `${state.dominant}_${state.texture}` : state.dominant;
}

export function CharacterStage() {
  const containerRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<SpriteRenderer | null>(null);
  const [activeCharacter, setActiveCharacter] = useState<ActiveCharacter | null>(null);
  const [allCharacters, setAllCharacters] = useState<CharacterSummary[]>([]);
  const [lastState, setLastState] = useState<ResolvedState | null>(null);
  const [diag, setDiag] = useState<string[]>(["boot"]);

  const log = (msg: string) => {
    console.log("[shikigami]", msg);
    setDiag((d) => [...d.slice(-9), msg]);
  };

  // Mount PixiJS + load character once.
  useEffect(() => {
    let cancelled = false;
    const renderer = new SpriteRenderer();
    rendererRef.current = renderer;

    (async () => {
      try {
        log("listing characters…");
        const chars = await listCharacters();
        if (cancelled) return;
        setAllCharacters(chars);
        log(`found ${chars.length} character(s): ${chars.map((c) => c.id).join(", ") || "(none)"}`);

        if (!containerRef.current) {
          log("container ref missing");
          return;
        }
        log("mounting pixi app…");
        await renderer.mount(containerRef.current);
        log("pixi mounted");

        log("fetching active character…");
        const character = await getActiveCharacter();
        if (cancelled) return;
        if (!character) {
          log("no active character");
          return;
        }
        log(`loading character ${character.id} (${Object.keys(character.states).length} states)`);
        await renderer.setCharacter(character);
        setActiveCharacter(character);
        log(`ready — default state: ${character.default_state}`);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        console.error("CharacterStage init failed:", e);
        log(`error: ${msg}`);
      }
    })();

    return () => {
      cancelled = true;
      renderer.dispose();
      rendererRef.current = null;
    };
  }, []);

  // Subscribe to state_changed events → drive the renderer.
  useEffect(() => {
    if (!activeCharacter) return;
    let unlisten: UnlistenFn | null = null;
    let cancelled = false;

    (async () => {
      try {
        const off = await listen<ResolvedState>("state_changed", (e) => {
          const key = animKey(e.payload);
          setLastState(e.payload);
          rendererRef.current?.transitionTo(key);
        });
        if (cancelled) {
          off();
          return;
        }
        unlisten = off;
        log("subscribed to state_changed");
      } catch (e) {
        log(`subscribe failed: ${String(e)}`);
      }
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [activeCharacter]);

  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        position: "relative",
        overflow: "hidden",
        background: "transparent",
      }}
    >
      {/* PixiJS canvas mounts here */}
      <div
        ref={containerRef}
        style={{
          position: "absolute",
          inset: 0,
          pointerEvents: "none",
        }}
      />

      {/* Diagnostic overlay — always visible in dev */}
      <div
        style={{
          position: "absolute",
          top: 8,
          left: 8,
          right: 8,
          padding: "8px 10px",
          borderRadius: 8,
          background: "rgba(20,20,30,0.78)",
          backdropFilter: "blur(8px)",
          WebkitBackdropFilter: "blur(8px)",
          color: "#f5f5f5",
          fontSize: 10,
          lineHeight: 1.45,
          fontFamily: "ui-monospace, Menlo, monospace",
          pointerEvents: "none",
        }}
      >
        <div style={{ fontSize: 11, fontWeight: 600, marginBottom: 4 }}>
          Shikigami v0.1.0-alpha.0
        </div>
        <div>
          characters: {allCharacters.length} · active:{" "}
          {activeCharacter?.id ?? "—"}
        </div>
        <div>
          state:{" "}
          {lastState
            ? `${animKey(lastState)} [sev=${lastState.severity}] #${lastState.event_id}`
            : "idle (waiting for events)"}
        </div>
        <details style={{ marginTop: 4 }}>
          <summary style={{ cursor: "pointer", opacity: 0.7 }}>
            diag log ({diag.length})
          </summary>
          <pre style={{ margin: "4px 0 0 0", fontSize: 9, opacity: 0.8, whiteSpace: "pre-wrap" }}>
            {diag.join("\n")}
          </pre>
        </details>
      </div>
    </div>
  );
}
