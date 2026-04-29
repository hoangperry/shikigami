// PixiJS v7 sprite renderer — Phase 2.
// Loads an ActiveCharacter's frame sequences, plays them on a transparent
// canvas, supports crossfade transitions between states.
//
// Pixi v7 chosen so we can share the runtime with `pixi-live2d-display-mulmotion`,
// which is v7-pinned and the battle-tested path for Live2D rendering.

import { Application, Assets, Sprite, Texture } from "pixi.js";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ActiveCharacter } from "../ipc/commands";

type LoadedAnimation = {
  stateKey: string;
  textures: Texture[];
  fps: number;
  loop: boolean;
  then: string | null;
  durationMs: number | null;
};

export class SpriteRenderer {
  private app: Application | null = null;
  private container: HTMLElement | null = null;

  private character: ActiveCharacter | null = null;
  private animations = new Map<string, LoadedAnimation>();

  private currentSprite: Sprite | null = null;
  private currentAnim: LoadedAnimation | null = null;

  private frameIndex = 0;
  private accumulatorMs = 0;
  private elapsedMs = 0;
  // User-tunable transform from Preferences. Applied on top of centerSprite()'s
  // auto-fit. 1.0 = no extra scaling.
  private userScale: number = 1.0;
  // Held so dispose() can detach the listener it added in mount().
  private onResize: (() => void) | null = null;

  async mount(container: HTMLElement): Promise<void> {
    this.container = container;
    // Pixi v7: synchronous Application constructor; canvas exposed as `app.view`.
    const app = new Application<HTMLCanvasElement>({
      resizeTo: container,
      backgroundAlpha: 0,
      antialias: false,
      resolution: window.devicePixelRatio ?? 1,
      autoDensity: true,
    });
    container.appendChild(app.view);
    // Pixi v7 ticker callback signature: `(deltaFrames: number) => void`.
    // Convert frames → ms via the ticker's deltaMS for our frame-rate logic.
    app.ticker.add(() => this.tick(app.ticker.deltaMS));
    this.app = app;

    // Window resize → refit on the next frame so Pixi's resizeTo poll has
    // updated app.screen first. Without rAF, the resize event fires before
    // Pixi sees the new container size, fit() reads stale dims, character
    // briefly snaps to the wrong scale → user-visible flicker.
    let resizePending = false;
    this.onResize = () => {
      if (resizePending) return;
      resizePending = true;
      requestAnimationFrame(() => {
        resizePending = false;
        this.refit();
      });
    };
    window.addEventListener("resize", this.onResize);
  }

  async setCharacter(character: ActiveCharacter): Promise<void> {
    this.character = character;
    this.animations.clear();

    for (const [stateKey, state] of Object.entries(character.states)) {
      // Base animation for the dominant state.
      const baseTextures = await this.loadFrames(state.frames);
      this.animations.set(stateKey, {
        stateKey,
        textures: baseTextures,
        fps: state.fps,
        loop: state.loop,
        then: state.then,
        durationMs: state.duration_ms,
      });

      // Texture variants — registered as compound keys `<state>_<tex>`
      // so resolveAnimKey can pick the variant when the state machine
      // emits e.g. "happy_relieved". Falls back to the base on miss.
      for (const [texName, framePaths] of Object.entries(state.textures ?? {})) {
        if (!framePaths || framePaths.length === 0) continue;
        const variantTextures = await this.loadFrames(framePaths);
        this.animations.set(`${stateKey}_${texName}`, {
          stateKey: `${stateKey}_${texName}`,
          textures: variantTextures,
          fps: state.fps,
          loop: state.loop,
          then: state.then,
          durationMs: state.duration_ms,
        });
      }
    }
    this.transitionTo(character.default_state);
  }

  private resolveAnimKey(requested: string): string {
    if (this.animations.has(requested)) return requested;
    const underscore = requested.indexOf("_");
    if (underscore > 0) {
      const base = requested.substring(0, underscore);
      if (this.animations.has(base)) return base;
    }
    return this.character?.default_state ?? "idle";
  }

  transitionTo(animKey: string, crossfadeMs = 180): void {
    if (!this.app) return;
    const resolved = this.resolveAnimKey(animKey);
    const next = this.animations.get(resolved);
    if (!next) return;
    if (this.currentAnim?.stateKey === next.stateKey) return;

    const nextSprite = new Sprite(next.textures[0]);
    centerSprite(nextSprite, this.app, this.userScale);
    nextSprite.alpha = 0;
    this.app.stage.addChild(nextSprite);

    const outSprite = this.currentSprite;
    const startedAt = performance.now();
    const fadeStep = () => {
      const t = Math.min(1, (performance.now() - startedAt) / crossfadeMs);
      if (outSprite) outSprite.alpha = 1 - t;
      nextSprite.alpha = t;
      if (t < 1) {
        requestAnimationFrame(fadeStep);
      } else {
        if (outSprite && outSprite.parent) outSprite.parent.removeChild(outSprite);
        this.currentSprite = nextSprite;
        this.currentAnim = next;
        this.frameIndex = 0;
        this.accumulatorMs = 0;
        this.elapsedMs = 0;
      }
    };
    requestAnimationFrame(fadeStep);
  }

  /** Mirror of Live2DRenderer.setUserTransform — keeps a uniform contract. */
  setUserTransform(opts: { scale?: number; opacity?: number }): void {
    if (typeof opts.scale === "number" && opts.scale > 0) {
      this.userScale = opts.scale;
      // Re-center the live sprite so the new scale takes effect now.
      if (this.currentSprite && this.app) {
        centerSprite(this.currentSprite, this.app, this.userScale);
      }
    }
    if (typeof opts.opacity === "number" && this.app) {
      this.app.stage.alpha = Math.max(0, Math.min(1, opts.opacity));
    }
  }

  /** Re-center on container resize. Wired by character-stage so window
   *  drag-resize keeps the sprite proportioned. */
  refit(): void {
    if (this.currentSprite && this.app) {
      centerSprite(this.currentSprite, this.app, this.userScale);
    }
  }

  dispose(): void {
    if (this.onResize) {
      window.removeEventListener("resize", this.onResize);
      this.onResize = null;
    }
    this.animations.clear();
    this.currentSprite = null;
    this.currentAnim = null;
    if (this.app) {
      this.app.destroy(true, { children: true, texture: true });
      this.app = null;
    }
    if (this.container) {
      this.container.innerHTML = "";
      this.container = null;
    }
  }

  private async loadFrames(framePaths: string[]): Promise<Texture[]> {
    const urls = framePaths.map((p) => convertFileSrc(p));
    const textures = await Promise.all(
      urls.map(async (u) => (await Assets.load(u)) as Texture),
    );
    return textures;
  }

  private tick(deltaMs: number): void {
    if (!this.currentAnim || !this.currentSprite) return;
    const anim = this.currentAnim;
    if (anim.textures.length <= 1) return;

    this.accumulatorMs += deltaMs;
    this.elapsedMs += deltaMs;
    const framePeriod = 1000 / anim.fps;

    while (this.accumulatorMs >= framePeriod) {
      this.accumulatorMs -= framePeriod;
      this.frameIndex += 1;
      if (this.frameIndex >= anim.textures.length) {
        this.frameIndex = anim.loop ? 0 : anim.textures.length - 1;
      }
      this.currentSprite.texture = anim.textures[this.frameIndex];
    }

    if (!anim.loop && anim.durationMs && this.elapsedMs >= anim.durationMs) {
      const then = anim.then ?? this.character?.default_state ?? "idle";
      this.transitionTo(then);
    }
  }
}

function centerSprite(sprite: Sprite, app: Application, userScale = 1.0): void {
  // Use app.screen (CSS units), NOT app.canvas (backing pixels × DPR).
  // On Retina displays, app.canvas.width is devicePixelRatio× the real render
  // area, which pushes the sprite off-screen when positioned by raw width.
  //
  // Containment guarantee: rendered sprite never exceeds window dims.
  // Default 90% fill, hard cap at 100% on either axis.
  const viewW = app.screen.width;
  const viewH = app.screen.height;
  const tw = sprite.texture.width;
  const th = sprite.texture.height;
  const targetScale = Math.min((viewW * 0.9) / tw, (viewH * 0.9) / th) * userScale;
  const maxScale = Math.min(viewW / tw, viewH / th);
  sprite.anchor.set(0.5, 0.5);
  sprite.position.set(viewW / 2, viewH / 2);
  sprite.scale.set(Math.min(targetScale, maxScale));
}
