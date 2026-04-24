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
// NOTE: Live2DRenderer is imported lazily so a broken Live2D dep can never
// take down the sprite path — blank-window symptom root-cause-fix.
import type { Live2DRenderer as Live2DRendererType } from "../renderer/live2d-renderer";
import { SpeechBubble } from "./speech-bubble";
import { SettingsModal } from "./settings-modal";

type AnyRenderer = SpriteRenderer | Live2DRendererType;

function guessRendererType(character: ActiveCharacter): "sprite" | "live2d" {
  for (const state of Object.values(character.states)) {
    for (const frame of state.frames) {
      if (frame.endsWith(".model3.json") || frame.endsWith(".moc3")) {
        return "live2d";
      }
    }
  }
  return "sprite";
}

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

  // Mount the correct renderer for the active character. Live2D characters
  // get a Live2DRenderer; sprite characters (or Live2D fallback after a
  // Cubism Core load failure) get a SpriteRenderer.
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
        if (rendererType === "live2d") {
          try {
            const mod = await import("../renderer/live2d-renderer");
            renderer = new mod.Live2DRenderer();
          } catch (importErr) {
            log(`live2d import failed: ${String(importErr)} — falling back to sprite`);
            renderer = new SpriteRenderer();
          }
        } else {
          renderer = new SpriteRenderer();
        }
        rendererRef.current = renderer;

        await renderer.mount(containerRef.current);
        try {
          await renderer.setCharacter(character);
          setActiveCharacter(character);
          log(`ready — default state: ${character.default_state}`);
        } catch (innerErr) {
          // Live2D init can fail on CDN / model load. Fall back to sprite so
          // the user never ends up with a blank window. Auto-open diag so
          // the user sees the error string without needing ⌘I.
          log(`setCharacter failed: ${String(innerErr)}`);
          if (innerErr instanceof Error && innerErr.stack) {
            log(`stack: ${innerErr.stack.split("\n").slice(0, 3).join(" | ")}`);
          }
          setShowDiag(true);
          if (rendererType === "live2d") {
            log("falling back to sprite renderer");
            renderer.dispose();
            const fallback = new SpriteRenderer();
            renderer = fallback;
            rendererRef.current = fallback;
            if (containerRef.current) {
              await fallback.mount(containerRef.current);
              const spriteChar = chars.find(
                (c) => c.id !== character.id && !c.is_active,
              );
              if (spriteChar) {
                // Best effort: caller can switch via tray; for now, stay on
                // this Live2D id but render as sprite if any frames exist.
                log(`fallback: showing ${spriteChar.id}`);
              }
              // Regardless, mark active so the diag overlay stops saying
              // "loading" and events still flow.
              setActiveCharacter(character);
            }
          } else {
            throw innerErr;
          }
        }
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

      {/* Early-boot banner — always visible until a character is active, so
          the window is never silent. */}
      {!activeCharacter && (
        <div
          style={{
            position: "absolute",
            top: "50%",
            left: "50%",
            transform: "translate(-50%, -50%)",
            padding: "10px 16px",
            borderRadius: 10,
            background: "rgba(20,20,30,0.75)",
            color: "#f5f5f5",
            fontSize: 12,
            fontFamily:
              "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
            zIndex: 5,
            pointerEvents: "none",
          }}
        >
          Shikigami · loading character…
          <div style={{ opacity: 0.6, fontSize: 10, marginTop: 4 }}>
            press ⌘I for diag
          </div>
        </div>
      )}

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
