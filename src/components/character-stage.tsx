// Character stage — mounts the PixiJS renderer, drives it from
// state_changed events, and hosts the speech bubble + settings modal.
//
// - Entire window is draggable via `data-tauri-drag-region`
// - Diagnostics: hidden by default, Cmd/Ctrl+I to toggle (or tray menu)
// - Preferences: Cmd/Ctrl+, or tray "Preferences…"
// - Speech bubble: event-triggered, near the character, auto-fades

import { useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  getActiveCharacter,
  listCharacters,
  type ActiveCharacter,
  type CharacterSummary,
} from "../ipc/commands";
import { SpriteRenderer } from "../renderer/sprite-renderer";
import { Live2DRenderer } from "../renderer/live2d-renderer";
import { SpeechBubble } from "./speech-bubble";
import { SettingsModal } from "./settings-modal";

type AnyRenderer = SpriteRenderer | Live2DRenderer;

type ResolvedState = {
  dominant: string;
  texture: string | null;
  severity: string;
  duration_ms: number;
  event_id: number;
  text: string | null;
};

function animKey(state: ResolvedState): string {
  return state.texture ? `${state.dominant}_${state.texture}` : state.dominant;
}

// Detect which renderer to use. Live2D characters have a frame pointing to
// a .model3.json file; sprite characters point to .png/.webp frames.
function guessRendererType(
  character: ActiveCharacter,
): "sprite" | "live2d" {
  for (const state of Object.values(character.states)) {
    for (const frame of state.frames) {
      if (frame.endsWith(".model3.json") || frame.endsWith(".moc3")) {
        return "live2d";
      }
    }
  }
  return "sprite";
}

export function CharacterStage() {
  const containerRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<AnyRenderer | null>(null);
  const [activeCharacter, setActiveCharacter] = useState<ActiveCharacter | null>(null);
  const [allCharacters, setAllCharacters] = useState<CharacterSummary[]>([]);
  const [lastState, setLastState] = useState<ResolvedState | null>(null);
  const [diag, setDiag] = useState<string[]>(["boot"]);
  const [showDiag, setShowDiag] = useState<boolean>(false);
  const [showSettings, setShowSettings] = useState<boolean>(false);

  const log = (msg: string) => {
    console.log("[shikigami]", msg);
    setDiag((d) => [...d.slice(-9), msg]);
  };

  // Keyboard shortcuts.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey) {
        if (e.key.toLowerCase() === "i") {
          e.preventDefault();
          setShowDiag((v) => !v);
        } else if (e.key === ",") {
          e.preventDefault();
          setShowSettings((v) => !v);
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Tray menu events.
  useEffect(() => {
    const unsubs: UnlistenFn[] = [];
    (async () => {
      unsubs.push(
        await listen("tray:toggle_diag", () => setShowDiag((v) => !v)),
      );
      unsubs.push(
        await listen("tray:open_settings", () => setShowSettings(true)),
      );
      unsubs.push(
        await listen("tray:position_reset", () => log("window reset to center")),
      );
    })();
    return () => {
      unsubs.forEach((u) => u());
    };
  }, []);

  // Mount the correct renderer for the active character, load its assets.
  useEffect(() => {
    let cancelled = false;
    let renderer: AnyRenderer | null = null;

    (async () => {
      try {
        const chars = await listCharacters();
        if (cancelled) return;
        setAllCharacters(chars);
        log(`found ${chars.length} character(s): ${chars.map((c) => c.id).join(", ") || "(none)"}`);

        if (!containerRef.current) return;
        const character = await getActiveCharacter();
        if (cancelled) return;
        if (!character) {
          log("no active character");
          return;
        }

        const rendererType = guessRendererType(character);
        log(`using ${rendererType} renderer for ${character.id}`);
        renderer =
          rendererType === "live2d" ? new Live2DRenderer() : new SpriteRenderer();
        rendererRef.current = renderer;

        await renderer.mount(containerRef.current);
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
      renderer?.dispose();
      rendererRef.current = null;
    };
  }, []);

  // Subscribe to state_changed events → drive the renderer + bubble.
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
      {/* Drag layer — covers the whole window so the character is draggable. */}
      <div
        data-tauri-drag-region
        style={{
          position: "absolute",
          inset: 0,
          zIndex: 1,
        }}
      />

      {/* PixiJS canvas — above the drag layer, pointer-events:none so drags pass through. */}
      <div
        ref={containerRef}
        style={{
          position: "absolute",
          inset: 0,
          zIndex: 2,
          pointerEvents: "none",
        }}
      />

      {/* Speech bubble — event-triggered, near top of canvas. */}
      <SpeechBubble state={lastState} lastEventText={lastState?.text ?? undefined} />

      {/* Settings modal. */}
      <SettingsModal open={showSettings} onClose={() => setShowSettings(false)} />

      {/* Diagnostic overlay — hidden by default, toggle with Cmd/Ctrl+I. */}
      {showDiag && (
        <div
          style={{
            position: "absolute",
            top: 8,
            left: 8,
            right: 8,
            padding: "8px 10px",
            borderRadius: 8,
            background: "rgba(20,20,30,0.85)",
            backdropFilter: "blur(10px)",
            WebkitBackdropFilter: "blur(10px)",
            color: "#f5f5f5",
            fontSize: 10,
            lineHeight: 1.45,
            fontFamily: "ui-monospace, Menlo, monospace",
            zIndex: 10,
          }}
        >
          <div style={{ fontSize: 11, fontWeight: 600, marginBottom: 4 }}>
            Shikigami v0.1.0-alpha.0 · (⌘I to hide, ⌘, for prefs)
          </div>
          <div>
            characters: {allCharacters.length} · active: {activeCharacter?.id ?? "—"}
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
      )}
    </div>
  );
}
