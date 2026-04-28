// Live2D renderer — pixi.js v7 + pixi-live2d-display-mulmotion (battle-tested
// combo used in production VTuber apps).
//
// History: v8-native fork (`untitled-pixi-live2d-engine`) loaded models OK
// but never actually drew to the Pixi v8 stage in WKWebView. Downgraded to
// v7 + mulmotion which is the canonical, well-supported path.
//
// Same mount / setCharacter / transitionTo / dispose contract as
// SpriteRenderer. Character-stage dispatches based on .model3.json
// presence in frame paths.
//
// Requires live2dcubismcore.min.js loaded by index.html. The renderer polls
// for the global at model-load time and fails fast if it never arrives.

import * as PIXI from "pixi.js";
import { Application, Ticker } from "pixi.js";
import { Live2DModel } from "pixi-live2d-display-mulmotion/cubism4";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { setCharacterBbox, type ActiveCharacter, type StatePayload } from "../ipc/commands";

// pixi-live2d-display reaches into `window.PIXI` to discover Pixi classes
// (Sprite, Texture, etc.) that aren't passed in directly. Without this the
// plugin throws at model construction time. Required for v7 + mulmotion.
(window as unknown as Record<string, unknown>).PIXI = PIXI;

// CRITICAL: pixi-live2d-display does NOT auto-subscribe to Pixi's ticker.
// Without registerTicker the Cubism update loop never runs and the model
// silently fails to draw.
Live2DModel.registerTicker(Ticker);

async function waitForCubismCore(timeoutMs: number): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if ((globalThis as Record<string, unknown>).Live2DCubismCore) return;
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(
    "Live2DCubismCore did not load — check index.html script tag and CSP",
  );
}

// Why pre-bundle as Blob URLs: WKWebView inside Tauri throws "Network
// error" when the Live2D plugin fetches .moc3 via XHR — even though the
// same URL returns HTTP 200 via fetch() and curl. Blob URLs are same-
// origin synthetic URLs that WKWebView's XHR handles reliably, so we
// pre-fetch every file the model needs (moc3, textures, motions,
// physics, pose, userdata, displayinfo, expressions), wrap each in a
// Blob URL, rewrite the model3.json in memory so all internal paths
// point at Blob URLs, then feed that rewritten JSON to the plugin
// (also via a Blob URL). No plugin XHR hits the network at all.
type ModelSettings = {
  FileReferences?: {
    Moc?: string;
    Textures?: string[];
    Physics?: string;
    Pose?: string;
    UserData?: string;
    DisplayInfo?: string;
    Motions?: Record<string, Array<{ File?: string; Sound?: string }>>;
    Expressions?: Array<{ Name?: string; File?: string }>;
  };
  [k: string]: unknown;
};

function mimeFor(path: string): string {
  const p = path.toLowerCase();
  if (p.endsWith(".json")) return "application/json";
  if (p.endsWith(".png")) return "image/png";
  if (p.endsWith(".jpg") || p.endsWith(".jpeg")) return "image/jpeg";
  if (p.endsWith(".webp")) return "image/webp";
  if (p.endsWith(".mp3")) return "audio/mpeg";
  if (p.endsWith(".wav")) return "audio/wav";
  return "application/octet-stream";
}

async function fetchAsBlobUrl(url: string): Promise<string> {
  const resp = await fetch(url);
  if (!resp.ok) {
    throw new Error(`fetch ${resp.status} ${resp.statusText} @ ${url}`);
  }
  const buf = await resp.arrayBuffer();
  const blob = new Blob([buf], { type: mimeFor(url) });
  return URL.createObjectURL(blob);
}

async function prebundleModelAsBlobUrl(
  modelUrl: string,
  revokeList: string[],
): Promise<string> {
  const resp = await fetch(modelUrl);
  if (!resp.ok) {
    throw new Error(
      `model3.json fetch ${resp.status} ${resp.statusText} @ ${modelUrl}`,
    );
  }
  const settings: ModelSettings = await resp.json();
  const baseUrl = modelUrl.substring(0, modelUrl.lastIndexOf("/") + 1);
  const resolve = (rel: string) => baseUrl + rel;

  const refs = settings.FileReferences ?? {};
  if (refs.Moc) {
    const u = await fetchAsBlobUrl(resolve(refs.Moc));
    revokeList.push(u);
    refs.Moc = u;
  }
  if (refs.Textures) {
    refs.Textures = await Promise.all(
      refs.Textures.map(async (t) => {
        const u = await fetchAsBlobUrl(resolve(t));
        revokeList.push(u);
        return u;
      }),
    );
  }
  for (const key of ["Physics", "Pose", "UserData", "DisplayInfo"] as const) {
    const v = refs[key];
    if (typeof v === "string" && v.length > 0) {
      const u = await fetchAsBlobUrl(resolve(v));
      revokeList.push(u);
      refs[key] = u;
    }
  }
  if (refs.Motions) {
    for (const group of Object.keys(refs.Motions)) {
      const arr = refs.Motions[group];
      for (const m of arr) {
        if (m.File) {
          const u = await fetchAsBlobUrl(resolve(m.File));
          revokeList.push(u);
          m.File = u;
        }
        if (m.Sound) {
          const u = await fetchAsBlobUrl(resolve(m.Sound));
          revokeList.push(u);
          m.Sound = u;
        }
      }
    }
  }
  if (refs.Expressions) {
    for (const e of refs.Expressions) {
      if (e.File) {
        const u = await fetchAsBlobUrl(resolve(e.File));
        revokeList.push(u);
        e.File = u;
      }
    }
  }
  settings.FileReferences = refs;

  const blob = new Blob([JSON.stringify(settings)], {
    type: "application/json",
  });
  const jsonUrl = URL.createObjectURL(blob);
  revokeList.push(jsonUrl);
  return jsonUrl;
}

export class Live2DRenderer {
  private app: Application | null = null;
  private container: HTMLElement | null = null;
  private model: InstanceType<typeof Live2DModel> | null = null;
  private blobUrls: string[] = [];
  // Cached per-state config so transitionTo() can resolve motion groups.
  private states: Record<string, StatePayload> = {};
  // Held only when mulmotion's native speak() is unavailable; lets us
  // interrupt the audio element on the next speak() call.
  private audioFallback: HTMLAudioElement | null = null;
  // User-tunable transform applied on top of the auto-fit scale. Bumped
  // from settings (Preferences slider). 1.0 = no extra scaling.
  private userScale: number = 1.0;
  // Active sequential motion chain timer; cleared if a newer transition
  // arrives so we never have two chains overlapping.
  private motionChainTimer: ReturnType<typeof setTimeout> | null = null;
  // Natural model dimensions snapshotted at load time (with scale = 1).
  // CRITICAL: pixi-live2d-display's `model.width`/`height` return the
  // POST-scale bounds, so re-reading them inside fit() after a previous
  // `scale.set()` returns the already-scaled size and the next fit
  // multiplies on top — character grows on every resize. Cache once and
  // use the cached values for all subsequent fit math.
  private modelNaturalWidth: number = 1;
  private modelNaturalHeight: number = 1;

  async mount(container: HTMLElement): Promise<void> {
    this.container = container;
    // Pixi v7: synchronous Application constructor with options. View is
    // exposed as `app.view` (HTMLCanvasElement). resizeTo handles DPR via
    // autoDensity + resolution.
    const app = new Application<HTMLCanvasElement>({
      resizeTo: container,
      backgroundAlpha: 0,
      antialias: true,
      resolution: window.devicePixelRatio ?? 1,
      autoDensity: true,
    });
    container.appendChild(app.view);
    this.app = app;
  }

  async setCharacter(character: ActiveCharacter): Promise<void> {
    if (!this.app) throw new Error("renderer not mounted");

    this.states = character.states;
    const defaultState = character.states[character.default_state];
    if (!defaultState || defaultState.frames.length === 0) {
      throw new Error(
        `Live2D character ${character.id} missing frames for default state`,
      );
    }

    // Rust loader includes .moc3 alongside .model3.json in `frames`; after
    // alphabetical sort the .moc3 may come first. Explicitly pick the
    // model3.json — that's the settings file the plugin expects.
    const rawPath = defaultState.frames.find((f) => f.endsWith(".model3.json"));
    if (!rawPath) {
      throw new Error(
        `Live2D character ${character.id}: no .model3.json in frames`,
      );
    }

    // URL resolution:
    //   - Browser dev payload: rawPath already shaped like "/hiyori/..." (URL)
    //   - Tauri dev: WebView loads from Vite at http://localhost:1420.
    //     `public/characters/` is symlinked to the repo `characters/` dir,
    //     so Vite serves model + textures + motions under /characters/...
    //     This bypasses asset:// which WKWebView's XHR (used by the Live2D
    //     plugin) treats as a "Network error".
    //   - Tauri prod bundle: fall back to convertFileSrc.
    const isTauri =
      typeof (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ !== "undefined";
    const isDev = import.meta.env.DEV;
    const charsMarker = "/characters/";
    const markerIdx = rawPath.indexOf(charsMarker);
    let modelUrl: string;
    if (!isTauri) {
      modelUrl = rawPath;
    } else if (isDev && markerIdx >= 0) {
      modelUrl = `${window.location.origin}${rawPath.substring(markerIdx)}`;
    } else {
      modelUrl = convertFileSrc(rawPath);
    }

    await waitForCubismCore(8000);

    // Strategy: try direct URL first (mulmotion's pixi-loader handles
    // WKWebView XHR fine for the Hiyori sample). Fall back to blob-URL
    // prebundling if direct fails — that path bypasses any XHR quirks by
    // pre-fetching every asset as a Blob and rewriting model3.json refs.
    let model: InstanceType<typeof Live2DModel>;
    try {
      model = await Live2DModel.from(modelUrl);
    } catch (directErr) {
      let bundledUrl: string;
      try {
        bundledUrl = await prebundleModelAsBlobUrl(modelUrl, this.blobUrls);
      } catch (bundleErr) {
        throw new Error(
          `prebundle failed for ${modelUrl}: ${String(bundleErr)} (direct also failed: ${String(directErr)})`,
        );
      }
      try {
        model = await Live2DModel.from(bundledUrl);
      } catch (bundledErr) {
        throw new Error(
          `Live2DModel.from failed via direct AND bundled. direct: ${String(directErr)} | bundled: ${String(bundledErr)}`,
        );
      }
    }

    this.app.stage.addChild(model);
    this.model = model;
    // Snapshot natural dimensions BEFORE any scale.set() — Pixi/mulmotion
    // returns post-scale bounds so we can never recompute these later.
    model.scale.set(1);
    this.modelNaturalWidth = model.width || 1;
    this.modelNaturalHeight = model.height || 1;

    this.fit();

    // Resize handling: `window.resize` fires synchronously while Pixi's
    // `resizeTo` poll updates `app.screen` on the next frame. If we call
    // fit() immediately we'd compute against stale dimensions and the
    // character snaps to the wrong size, then Pixi's render runs against
    // the new canvas → user sees a one-frame "siêu to" flicker. Defer to
    // rAF so app.screen is settled before we measure.
    let resizePending = false;
    const onResize = () => {
      if (resizePending) return;
      resizePending = true;
      requestAnimationFrame(() => {
        resizePending = false;
        this.fit();
      });
    };
    window.addEventListener("resize", onResize);
    (this.model as unknown as { __onResize?: () => void }).__onResize = onResize;
  }

  // Bottom-anchored fit: Live2D models are typically full-body, the natural
  // pivot is feet-on-floor. We anchor (0.5, 1.0) so feet land at canvas
  // bottom-centre.
  //
  // Containment guarantee: rendered width AND height MUST stay ≤ window
  // width/height — never overflow. We compute the contain-fit scale
  // (smaller axis ratio) at full window (1.0 fill) and CLAMP the final
  // user-multiplied scale so the rendered character can never exceed
  // either window dimension regardless of slider position.
  //
  // Uses cached `modelNaturalWidth/Height` so repeat calls don't compound
  // scaling — see field comment on `modelNaturalWidth` for the trap.
  private fit(): void {
    if (!this.app || !this.model) return;
    const w = this.app.screen.width;
    const h = this.app.screen.height;
    const m = this.model;

    const mw = this.modelNaturalWidth;
    const mh = this.modelNaturalHeight;
    // Default fit: 90% of the shorter axis — leaves a small breathing
    // margin so resize handles + window edge stay visually present.
    const targetScale = Math.min((w * 0.9) / mw, (h * 0.9) / mh) * this.userScale;
    // Hard cap: rendered width ≤ w AND rendered height ≤ h, no exceptions.
    const maxScale = Math.min(w / mw, h / mh);
    const finalScale = Math.min(targetScale, maxScale);
    m.scale.set(finalScale);

    const maybeAnchor = (m as unknown as { anchor?: { set: (x: number, y: number) => void } }).anchor;
    if (maybeAnchor?.set) {
      maybeAnchor.set(0.5, 1.0);
    } else {
      m.pivot.set(mw / 2, mh);
    }
    m.position.set(w / 2, h);

    // Push the rendered character's screen-space AABB to the backend
    // so the smart click-through poller knows where to catch clicks.
    // Fire-and-forget: never block fit() on IPC.
    void this.emitBbox(finalScale, mw, mh, w, h);
  }

  /** Compute screen-space (physical pixels) AABB of the rendered character
   *  and post it to the backend for click-through hit-testing. */
  private async emitBbox(
    scale: number,
    naturalW: number,
    naturalH: number,
    canvasW: number,
    canvasH: number,
  ): Promise<void> {
    if (!this.app) return;
    try {
      // Rendered character size in CSS pixels.
      const cssW = naturalW * scale;
      const cssH = naturalH * scale;
      // Position: anchor (0.5, 1.0) at (canvasW/2, canvasH) → top-left =
      // (centerX - cssW/2, canvasH - cssH).
      const cssX = canvasW / 2 - cssW / 2;
      const cssY = canvasH - cssH;
      // Convert canvas-CSS to screen-physical: get window outer position,
      // multiply local CSS coords by DPR, add window origin (already physical).
      const win = getCurrentWindow();
      const [outerPos, dpr] = await Promise.all([
        win.outerPosition(),
        Promise.resolve(window.devicePixelRatio || 1),
      ]);
      await setCharacterBbox({
        x: outerPos.x + cssX * dpr,
        y: outerPos.y + cssY * dpr,
        width: cssW * dpr,
        height: cssH * dpr,
      });
    } catch {
      // ignore — bbox is best-effort polish
    }
  }

  /**
   * Apply user-tunable transform from Preferences (Scale + Opacity sliders).
   * Idempotent — safe to call repeatedly. Re-runs fit() so the new scale
   * takes effect immediately without waiting for a resize event.
   */
  setUserTransform(opts: { scale?: number; opacity?: number }): void {
    if (typeof opts.scale === "number" && opts.scale > 0) {
      this.userScale = opts.scale;
      this.fit();
    }
    if (typeof opts.opacity === "number" && this.app) {
      // app.stage.alpha multiplies through the entire scene graph — Live2D
      // model + any future overlays inherit the same fade.
      this.app.stage.alpha = Math.max(0, Math.min(1, opts.opacity));
    }
  }

  /**
   * Play TTS audio with mouth-sync. mulmotion's `model.speak(url)` analyses
   * audio amplitude in real time and drives `ParamMouthOpenY`. The Hiyori
   * sample model has the standard mouth params rigged so no extra config is
   * required. Falls back to plain `<audio>` playback if the engine method
   * is missing on the model class.
   *
   * Interrupt semantics: if called while previous audio still plays, the
   * previous one is stopped first — avoids overlapping voices when the user
   * fires multiple AI turns in quick succession.
   */
  speak(audioUrl: string, opts?: { volume?: number; expression?: string }): void {
    if (!this.model) return;

    // Stop any in-flight audio before starting the next one.
    this.stopSpeaking();

    const speakable = this.model as unknown as {
      speak?: (url: string, options?: Record<string, unknown>) => void;
      stopSpeaking?: () => void;
    };
    if (typeof speakable.speak === "function") {
      speakable.speak(audioUrl, {
        volume: opts?.volume ?? 1,
        expression: opts?.expression,
        resetExpression: true,
        crossOrigin: "anonymous",
      });
    } else {
      const a = new Audio(audioUrl);
      a.volume = opts?.volume ?? 1;
      // Self-clean once playback ends so the reference doesn't pin the
      // decoded audio buffer between speaks. Guarded against the next
      // speak() race: if a different audio element has already replaced
      // `audioFallback`, leave that one alone.
      const release = () => {
        if (this.audioFallback === a) {
          this.audioFallback = null;
        }
      };
      a.addEventListener("ended", release);
      a.addEventListener("error", release);
      this.audioFallback = a;
      void a.play();
    }
  }

  /** Schedule a sequence of motion calls with per-step delays. The first
   *  motion fires immediately; subsequent ones wait `delay_ms` from the
   *  PREVIOUS step before triggering. Newer `transitionTo()` cancels the
   *  pending timer. */
  private runMotionChain(chain: { group: string; delay_ms: number }[]): void {
    if (!this.model) return;
    const playStep = (idx: number) => {
      if (idx >= chain.length || !this.model) return;
      const step = chain[idx];
      try {
        this.model.motion(step.group);
      } catch {
        /* motion missing — keep advancing */
      }
      const next = idx + 1;
      if (next < chain.length) {
        this.motionChainTimer = setTimeout(
          () => playStep(next),
          Math.max(0, step.delay_ms),
        );
      } else {
        this.motionChainTimer = null;
      }
    };
    playStep(0);
  }

  /** Cancel the current spoken line, if any. */
  stopSpeaking(): void {
    if (this.audioFallback) {
      try {
        this.audioFallback.pause();
        this.audioFallback.src = "";
      } catch {
        /* ignore */
      }
      this.audioFallback = null;
    }
    if (this.model) {
      const m = this.model as unknown as { stopSpeaking?: () => void };
      try {
        m.stopSpeaking?.();
      } catch {
        /* ignore */
      }
    }
  }

  transitionTo(animKey: string, _crossfadeMs?: number): void {
    if (!this.model) return;
    const dominant = animKey.split("_")[0];
    const state = this.states[dominant];

    // Cancel any in-flight motion chain — newer transition wins.
    if (this.motionChainTimer !== null) {
      clearTimeout(this.motionChainTimer);
      this.motionChainTimer = null;
    }

    // Resolution priority: motion_chain > motions[] > motion > state name.
    const chain = state?.motion_chain ?? [];
    if (chain.length > 0) {
      this.runMotionChain(chain);
    } else {
      const pool = state?.motions ?? [];
      const group =
        pool.length > 0
          ? (pool[Math.floor(Math.random() * pool.length)] ?? dominant)
          : (state?.motion ?? dominant);
      try {
        this.model.motion(group);
      } catch {
        // motion group not present on this model — safe to ignore
      }
    }

    // Expression layer (face overlay, blends with motion). Pool wins over
    // single. Skip silently if model has 0 expressions (e.g. Hiyori sample).
    const exprPool = state?.expressions ?? [];
    const exprName =
      exprPool.length > 0
        ? exprPool[Math.floor(Math.random() * exprPool.length)]
        : (state?.expression ?? null);
    if (exprName) {
      try {
        const expr = (this.model as unknown as { expression?: (name: string) => void })
          .expression;
        expr?.(exprName);
      } catch {
        // no expression — safe to ignore
      }
    }
  }

  dispose(): void {
    if (this.motionChainTimer !== null) {
      clearTimeout(this.motionChainTimer);
      this.motionChainTimer = null;
    }
    this.stopSpeaking();
    for (const u of this.blobUrls) URL.revokeObjectURL(u);
    this.blobUrls = [];
    if (this.model) {
      const ref = this.model as unknown as { __onResize?: () => void };
      if (ref.__onResize) window.removeEventListener("resize", ref.__onResize);
      this.model.destroy({ children: true, texture: false });
      this.model = null;
    }
    if (this.app) {
      this.app.destroy(true, { children: true, texture: true });
      this.app = null;
    }
    if (this.container) {
      this.container.innerHTML = "";
      this.container = null;
    }
  }
}
