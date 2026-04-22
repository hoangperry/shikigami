// Live2D renderer — loads Cubism 4 models via pixi-live2d-display-lipsyncpatch
// and maps Shikigami animation keys to Live2D expressions + motions.
//
// Requires live2dcubismcore.js to be loaded globally before this renderer is
// instantiated (see index.html — we load it from Live2D's CDN).
//
// Model directory layout (convention):
//   <character-root>/
//     manifest.json           ← "renderer": "live2d", "defaultState": "idle"
//     model/<name>.model3.json
//     model/<name>.moc3
//     model/textures/*.png
//     model/motions/<state>.motion3.json   (one per dominant state)
//     model/expressions/<state>.exp3.json  (optional per state)
//
// Manifest state `path` points to the directory holding motion/expression
// files; the Live2D loader is configured to look up files by name.

import { Application } from "pixi.js";
import { Live2DModel } from "pixi-live2d-display-lipsyncpatch";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ActiveCharacter } from "../ipc/commands";

type Live2DApp = Application;

export class Live2DRenderer {
  private app: Live2DApp | null = null;
  private container: HTMLElement | null = null;
  private model: Live2DModel | null = null;
  private character: ActiveCharacter | null = null;

  async mount(container: HTMLElement): Promise<void> {
    this.container = container;
    const app = new Application({
      resizeTo: container,
      backgroundAlpha: 0,
      antialias: true,
      resolution: window.devicePixelRatio ?? 1,
      autoDensity: true,
    });
    container.appendChild(app.view as HTMLCanvasElement);
    this.app = app;
  }

  async setCharacter(character: ActiveCharacter): Promise<void> {
    if (!this.app) throw new Error("renderer not mounted");
    this.character = character;

    // Expect the default state to include a single .model3.json frame path.
    const defaultState = character.states[character.default_state];
    if (!defaultState || defaultState.frames.length === 0) {
      throw new Error(
        `Live2D character ${character.id} missing frames for default state`,
      );
    }
    // By convention the FIRST frame of defaultState is the model3.json file.
    const modelUrl = convertFileSrc(defaultState.frames[0]);

    const model = await Live2DModel.from(modelUrl);
    this.app.stage.addChild(model as unknown as import("pixi.js").DisplayObject);

    // Fit model to canvas.
    const view = this.app.view as HTMLCanvasElement;
    const scale = Math.min(view.width / model.width, view.height / model.height) * 0.9;
    model.scale.set(scale);
    model.anchor.set(0.5, 0.5);
    model.position.set(view.width / 2, view.height / 2);

    this.model = model;
  }

  transitionTo(animKey: string, _crossfadeMs?: number): void {
    if (!this.model || !this.character) return;
    // Strip optional texture suffix ("happy_relieved" → "happy").
    const dominant = animKey.split("_")[0];

    // Try expression first (non-looping morph), then motion (body animation).
    try {
      const expr = (this.model as unknown as { expression?: (name: string) => void }).expression;
      expr?.(dominant);
    } catch {
      // ignore — character may not have an expression for this state
    }
    try {
      this.model.motion(dominant);
    } catch {
      // ignore — no motion for this state
    }
  }

  dispose(): void {
    if (this.model && this.app) {
      this.app.stage.removeChild(
        this.model as unknown as import("pixi.js").DisplayObject,
      );
      this.model.destroy();
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
    this.character = null;
  }
}
