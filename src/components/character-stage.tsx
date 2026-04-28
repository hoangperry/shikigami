// Character stage — mounts the PixiJS renderer, drives it from
// state_changed events, and hosts the speech bubble + settings modal.
//
// - Entire window is draggable via `data-tauri-drag-region`
// - Diagnostics: hidden by default, Cmd/Ctrl+I to toggle (or tray menu)
// - Preferences: Cmd/Ctrl+, or tray "Preferences…"
// - Speech bubble: event-triggered, near the character, auto-fades

import { useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  getActiveCharacter,
  getSettings,
  listCharacters,
  type ActiveCharacter,
  type CharacterSummary,
  type Settings,
  type SpeakEvent,
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
  const [diag, setDiag] = useState<string[]>(["shikigami ready"]);
  const [showDiag, setShowDiag] = useState<boolean>(false);
  const [showSettings, setShowSettings] = useState<boolean>(false);
  const [spoken, setSpoken] = useState<{ text: string; key: number } | null>(null);
  // Latest TTS volume from settings, kept in a ref so the speak() callback
  // never closes over a stale value (settings changes don't re-bind the
  // listen() handler).
  const ttsVolumeRef = useRef<number>(1.0);

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
        // Browser-only dev mode: synthesize a Hiyori payload pointing at
        // /hiyori/ served by Vite so we can debug Live2D in Safari/Chrome.
        const isTauri =
          typeof (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ !== "undefined";

        let chars: CharacterSummary[];
        let character: ActiveCharacter | null;
        if (isTauri) {
          chars = await listCharacters();
          if (cancelled) return;
          character = await getActiveCharacter();
        } else {
          log("browser dev mode — using public/hiyori payload");
          chars = [
            {
              id: "hiyori",
              name: "Hiyori (browser dev)",
              author: "Live2D Inc.",
              version: "1.0.0",
              is_active: true,
              default_state: "idle",
              state_count: 5,
            },
          ];
          character = {
            id: "hiyori",
            name: "Hiyori (browser dev)",
            default_state: "idle",
            states: Object.fromEntries(
              ["idle", "happy", "focused", "warning", "sleepy"].map((s) => [
                s,
                {
                  fps: 30,
                  loop: true,
                  then: null,
                  duration_ms: null,
                  frames: ["/hiyori/frame_00.model3.json"],
                  textures: [],
                  motion: s === "happy" || s === "warning" ? "TapBody" : "Idle",
                  motions: [],
                  motion_chain: [],
                  expression: null,
                  expressions: [],
                },
              ]),
            ),
          } as ActiveCharacter;
        }
        setAllCharacters(chars);
        log(`found ${chars.length} character(s): ${chars.map((c) => c.id).join(", ") || "(none)"}`);

        if (!containerRef.current) return;
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

        // Apply persisted scale + opacity BEFORE the character loads so
        // the very first fit() inside setCharacter uses the user's
        // configured size — eliminates the "huge initial render then
        // shrink" flicker that occurred when the settings effect raced
        // the mount effect.
        try {
          const initial = await getSettings();
          const r = renderer as unknown as {
            setUserTransform?: (o: { scale?: number; opacity?: number }) => void;
          };
          r.setUserTransform?.({ scale: initial.scale, opacity: initial.opacity });
          ttsVolumeRef.current =
            typeof initial.tts.volume === "number" ? initial.tts.volume : 1.0;
        } catch (e) {
          log(`prefetch settings failed: ${String(e)}`);
        }

        try {
          await renderer.setCharacter(character);
          setActiveCharacter(character);
          log(`ready — default state: ${character.default_state}`);
        } catch (innerErr) {
          // Do NOT silently swap to a different character — that hides the
          // real Live2D failure from the user. Surface the error, open diag
          // automatically, and keep the loading banner up so it's obvious
          // rendering failed (rather than pretending a sprite fallback is
          // what the user wanted).
          log(`setCharacter failed: ${String(innerErr)}`);
          if (innerErr instanceof Error && innerErr.stack) {
            log(`stack: ${innerErr.stack.split("\n").slice(0, 3).join(" | ")}`);
          }
          setShowDiag(true);
          throw innerErr;
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

  // Subscribe to settings_changed → apply scale + opacity to the active
  // renderer without requiring a window reload. Also runs once on mount
  // so the persisted values from settings.json take effect at startup.
  useEffect(() => {
    if (!activeCharacter) return;
    let unlisten: UnlistenFn | null = null;
    let cancelled = false;

    const apply = (s: Settings) => {
      const r = rendererRef.current as unknown as {
        setUserTransform?: (o: { scale?: number; opacity?: number }) => void;
      } | null;
      r?.setUserTransform?.({ scale: s.scale, opacity: s.opacity });
      // Cache volume for next tts:speak event.
      ttsVolumeRef.current = typeof s.tts.volume === "number" ? s.tts.volume : 1.0;
    };

    (async () => {
      try {
        // Initial apply from persisted settings.
        const s = await getSettings();
        if (cancelled) return;
        apply(s);
        // Then subscribe for live changes from the Preferences modal.
        const off = await listen<Settings>("settings_changed", (e) => {
          apply(e.payload);
        });
        if (cancelled) {
          off();
          return;
        }
        unlisten = off;
      } catch (e) {
        log(`settings subscribe failed: ${String(e)}`);
      }
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [activeCharacter]);

  // Subscribe to tts:speak — backend synthesised audio, drive Live2D lipsync.
  // Sprite renderer has no speak(); we no-op silently for that path.
  useEffect(() => {
    if (!activeCharacter) return;
    let unlisten: UnlistenFn | null = null;
    let cancelled = false;

    (async () => {
      try {
        const off = await listen<SpeakEvent>("tts:speak", (e) => {
          const r = rendererRef.current as unknown as {
            speak?: (url: string, opts?: { volume?: number }) => void;
          } | null;
          if (!r?.speak) {
            log(`tts:speak ignored — renderer has no lipsync (${e.payload.provider})`);
            return;
          }
          // Convert disk path → Tauri asset URL so WKWebView can stream it.
          const url = convertFileSrc(e.payload.audio_url);
          const vol = ttsVolumeRef.current;
          log(`tts:speak ${e.payload.provider} (vol=${vol.toFixed(2)}) → ${url.split("/").pop()}`);
          // Volume 0 = mute → skip playback entirely so no audio decode work
          // happens. Lipsync also stays still, which is the right UX.
          if (vol <= 0.001) return;
          r.speak(url, { volume: vol });
          // Surface the spoken text in the bubble so user reads what Hiyori
          // is saying. Use Date.now() as a unique key so identical text
          // back-to-back still re-triggers the bubble fade-in.
          setSpoken({ text: e.payload.text, key: Date.now() });
        });
        if (cancelled) {
          off();
          return;
        }
        unlisten = off;
      } catch (e) {
        log(`tts subscribe failed: ${String(e)}`);
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

      {/* Resize handles — 4 corners. Without native window decorations the
          user has no idea where to drag to resize, so we render small
          interactive corner grips that fade in on hover. Click-and-drag
          on any handle invokes Tauri's startResizeDragging in the right
          direction. */}
      <ResizeHandle corner="topLeft" />
      <ResizeHandle corner="topRight" />
      <ResizeHandle corner="bottomLeft" />
      <ResizeHandle corner="bottomRight" />

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
      <SpeechBubble
        state={lastState}
        lastEventText={lastState?.text ?? undefined}
        spokenText={spoken?.text}
        spokenKey={spoken?.key}
      />

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

// ─────────────────────────────────────────────────────────────────────
// ResizeHandle — small interactive corner grip. The window has
// `decorations: false` (transparent overlay aesthetic) so macOS doesn't
// render its own resize cursor anywhere. We render four 16×16 corners
// that brighten on hover; mousedown invokes Tauri's
// `startResizeDragging` in the matching direction so the OS takes over
// the drag-resize loop.
// ─────────────────────────────────────────────────────────────────────

type Corner = "topLeft" | "topRight" | "bottomLeft" | "bottomRight";

const CORNER_DIR: Record<Corner, string> = {
  topLeft: "NorthWest",
  topRight: "NorthEast",
  bottomLeft: "SouthWest",
  bottomRight: "SouthEast",
};

function ResizeHandle({ corner }: { corner: Corner }) {
  const [hover, setHover] = useState(false);

  // Position the 16×16 hit-target flush in its corner. We don't use
  // padding so the entire box is grabbable.
  const pos: React.CSSProperties = {
    position: "absolute",
    width: 16,
    height: 16,
    zIndex: 25,
    cursor:
      corner === "topLeft" || corner === "bottomRight"
        ? "nwse-resize"
        : "nesw-resize",
  };
  if (corner === "topLeft") Object.assign(pos, { top: 0, left: 0 });
  if (corner === "topRight") Object.assign(pos, { top: 0, right: 0 });
  if (corner === "bottomLeft") Object.assign(pos, { bottom: 0, left: 0 });
  if (corner === "bottomRight") Object.assign(pos, { bottom: 0, right: 0 });

  const onMouseDown = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    try {
      const win = getCurrentWindow();
      // Tauri 2: startResizeDragging takes a ResizeDirection string
      // matching the Rust enum variants (NorthWest / NorthEast / etc.).
      // Cast through unknown — older type defs only accept the v1 enum
      // and reject the string union we pass here.
      await (
        win as unknown as { startResizeDragging: (d: string) => Promise<void> }
      ).startResizeDragging(CORNER_DIR[corner]);
    } catch (err) {
      console.warn("[shikigami] startResizeDragging failed:", err);
    }
  };

  return (
    <div
      style={pos}
      onMouseDown={onMouseDown}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
    >
      {/* Visual indicator — small square that brightens on hover so the
          user can find the grab target without it being intrusive. */}
      <div
        style={{
          position: "absolute",
          inset: 4,
          borderRadius: 2,
          background: hover ? "rgba(255,255,255,0.55)" : "rgba(255,255,255,0.18)",
          boxShadow: hover ? "0 0 6px rgba(0,0,0,0.4)" : "none",
          transition: "background 120ms ease, box-shadow 120ms ease",
          pointerEvents: "none",
        }}
      />
    </div>
  );
}
