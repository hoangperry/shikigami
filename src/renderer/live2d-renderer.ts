// Live2D renderer — pixi.js v8 native via @naari3/pixi-live2d-display.
//
// This is the pixi v8 compatible branch of pixi-live2d-display maintained by
// naari3. Sibling package to SpriteRenderer; same lifecycle contract
// (mount → setCharacter → transitionTo → dispose) so CharacterStage can
// swap implementations based on the manifest's frame type.
//
// Requires live2dcubismcore.min.js to be loaded globally before instantiating
// (see index.html — async-loaded from Live2D's CDN). The renderer polls for
// the global at model-load time and fails fast if it never arrives, which
// lets CharacterStage fall back to the sprite path.

import { Application, Ticker } from "pixi.js";
import { Live2DModel } from "@naari3/pixi-live2d-display";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ActiveCharacter } from "../ipc/commands";

// Wire the Live2D display plugin to pixi's Ticker (required by v8 port).
// Safe to call multiple times.
Live2DModel.registerTicker(Ticker);

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
      // Force WebGL — Cubism Core vertex math is unreliable on WebGPU.
      preference: "webgl",
    });
    container.appendChild(app.canvas);
    this.app = app;
  }

  async setCharacter(character: ActiveCharacter): Promise<void> {
    if (!this.app) throw new Error("renderer not mounted");

    const defaultState = character.states[character.default_state];
    if (!defaultState || defaultState.frames.length === 0) {
      throw new Error(
        `Live2D character ${character.id} missing frames for default state`,
      );
    }
    // Convention: the first frame of defaultState points at .model3.json.
    const modelUrl = convertFileSrc(defaultState.frames[0]);

    await waitForCubismCore(8000);

    const model = await Live2DModel.from(modelUrl, { autoInteract: false });
    this.app.stage.addChild(model);

    // Fit model inside the current canvas.
    const canvas = this.app.canvas;
    const scale =
      Math.min(canvas.width / model.width, canvas.height / model.height) * 0.9;
    model.scale.set(scale);
    model.anchor.set(0.5, 0.5);
    model.position.set(canvas.width / 2, canvas.height / 2);

    this.model = model;
  }

  transitionTo(animKey: string, _crossfadeMs?: number): void {
    if (!this.model) return;
    // Strip texture suffix ("happy_relieved" → "happy").
    const dominant = animKey.split("_")[0];

    // Try expression first (non-looping morph), then motion group.
    try {
      // @naari3's API exposes expression() and motion() directly on the model.
      const expr = (this.model as unknown as { expression?: (name: string) => void }).expression;
      expr?.(dominant);
    } catch {
      // ignore — model may not declare an expression for this state
    }
    try {
      this.model.motion(dominant);
    } catch {
      // ignore — no motion group for this state
    }
  }

  dispose(): void {
    if (this.model) {
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
