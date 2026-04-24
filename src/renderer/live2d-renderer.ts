// Live2D renderer — pixi.js v8 native via untitled-pixi-live2d-engine.
//
// Swapped from @naari3/pixi-live2d-display after the latter silently
// failed to render despite setting window.PIXI. untitled-pixi-live2d-engine
// is also pixi v8 native (fork of pixi-live2d-display-mulmotion),
// maintained by Untitled-Story, supports Cubism 3/4/5.
//
// Same mount / setCharacter / transitionTo / dispose contract as
// SpriteRenderer. Character-stage dispatches based on .model3.json
// presence in frame paths.
//
// Requires live2dcubismcore.min.js from the CDN (see index.html). The
// renderer polls for the global at model-load time and fails fast if
// it never arrives — lets CharacterStage fall back to the sprite path.

import { Application } from "pixi.js";
import { Live2DModel } from "untitled-pixi-live2d-engine/cubism";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ActiveCharacter } from "../ipc/commands";

async function waitForCubismCore(timeoutMs: number): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if ((globalThis as Record<string, unknown>).Live2DCubismCore) return;
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(
    "Live2DCubismCore did not load from CDN — check network / CSP",
  );
}

export class Live2DRenderer {
  private app: Application | null = null;
  private container: HTMLElement | null = null;
  private model: Live2DModel | null = null;

  async mount(container: HTMLElement): Promise<void> {
    this.container = container;
    const app = new Application();
    await app.init({
      resizeTo: container,
      backgroundAlpha: 0,
      antialias: true,
      resolution: window.devicePixelRatio ?? 1,
      autoDensity: true,
      preference: "webgl",
    });
    container.appendChild(app.canvas);
    this.app = app;
    console.log("[live2d] Application init OK", {
      w: app.canvas.width,
      h: app.canvas.height,
    });
  }

  async setCharacter(character: ActiveCharacter): Promise<void> {
    if (!this.app) throw new Error("renderer not mounted");

    const defaultState = character.states[character.default_state];
    if (!defaultState || defaultState.frames.length === 0) {
      throw new Error(
        `Live2D character ${character.id} missing frames for default state`,
      );
    }

    const rawPath = defaultState.frames[0];
    // In a plain browser (no Tauri), the raw path is already a URL from
    // CharacterStage's dev-mode payload (/hiyori/...). Otherwise turn the
    // absolute disk path into an asset:// URL the Tauri WebView can fetch.
    const isTauri = typeof (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ !== "undefined";
    const modelUrl = isTauri ? convertFileSrc(rawPath) : rawPath;
    console.log("[live2d] raw path:", rawPath);
    console.log("[live2d] asset url:", modelUrl, "(tauri:", isTauri, ")");

    console.log("[live2d] waiting for Cubism Core…");
    await waitForCubismCore(8000);
    console.log("[live2d] Cubism Core ready");

    console.log("[live2d] Live2DModel.from…");
    const model = await Live2DModel.from(modelUrl);
    console.log("[live2d] model loaded", { w: model.width, h: model.height });
    this.app.stage.addChild(model);

    this.model = model;
    this.fit();
    console.log("[live2d] ready");

    // Re-centre on container resize (user drags/resizes the overlay).
    const onResize = () => this.fit();
    window.addEventListener("resize", onResize);
    (this.model as unknown as { __onResize?: () => void }).__onResize = onResize;
  }

  private fit(): void {
    if (!this.app || !this.model) return;
    // app.screen is logical CSS pixels. canvas.width is backing-store pixels
    // (= CSS × devicePixelRatio); using it gave a scale factor too large for
    // the visible area and pushed the character past the right edge on retina.
    const w = this.app.screen.width;
    const h = this.app.screen.height;
    const m = this.model;
    const scale = Math.min(w / m.width, h / m.height) * 0.9;
    m.scale.set(scale);
    m.anchor.set(0.5, 0.5);
    m.position.set(w / 2, h / 2);
  }

  transitionTo(animKey: string, _crossfadeMs?: number): void {
    if (!this.model) return;
    const dominant = animKey.split("_")[0];

    // untitled-pixi-live2d-engine API: model.motion(group, index?, priority?)
    try {
      this.model.motion(dominant);
    } catch {
      // no motion group for this state — safe to ignore
    }
    // Expression support varies per model; wrap in try for safety.
    try {
      const expr = (this.model as unknown as { expression?: (name: string) => void })
        .expression;
      expr?.(dominant);
    } catch {
      // no expression
    }
  }

  dispose(): void {
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
