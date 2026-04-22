// Character stage — mounts the PixiJS renderer and keeps it synchronized
// with the latest ResolvedState received from the Rust backend.

import { useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getActiveCharacter } from "../ipc/commands";
import type { ActiveCharacter } from "../ipc/commands";
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
  const [status, setStatus] = useState<string>("initializing...");

  // Mount PixiJS + load character once.
  useEffect(() => {
    let cancelled = false;
    const renderer = new SpriteRenderer();
    rendererRef.current = renderer;

    (async () => {
      try {
        if (!containerRef.current) return;
        await renderer.mount(containerRef.current);

        const character = await getActiveCharacter();
        if (cancelled) return;
        if (!character) {
          setStatus("no character installed");
          return;
        }
        await renderer.setCharacter(character);
        setActiveCharacter(character);
        setStatus("");
      } catch (e) {
        console.error("CharacterStage init failed:", e);
        setStatus(`error: ${String(e)}`);
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
          rendererRef.current?.transitionTo(key);
        });
        if (cancelled) {
          off();
          return;
        }
        unlisten = off;
      } catch (e) {
        console.warn("state_changed subscribe failed:", e);
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
      }}
    >
      <div
        ref={containerRef}
        style={{
          position: "absolute",
          inset: 0,
          pointerEvents: "none",
        }}
      />
      {status && (
        <div
          style={{
            position: "absolute",
            bottom: 12,
            left: "50%",
            transform: "translateX(-50%)",
            padding: "6px 12px",
            borderRadius: 8,
            background: "rgba(20,20,30,0.7)",
            color: "#f5f5f5",
            fontSize: 11,
            fontFamily:
              "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
          }}
        >
          {status}
        </div>
      )}
    </div>
  );
}
