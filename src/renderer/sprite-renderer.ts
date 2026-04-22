// PixiJS v8 sprite renderer — Phase 2.
//
// Takes an ActiveCharacter + animation key, plays the corresponding frame
// sequence on a transparent canvas. Supports crossfade transitions between
// states.

import { Application, Assets, Container, Sprite, Texture } from "pixi.js";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ActiveCharacter, StatePayload } from "../ipc/commands";

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
  private nextSprite: Sprite | null = null;
  private currentAnim: LoadedAnimation | null = null;

  private frameIndex = 0;
  private accumulatorMs = 0;
  private elapsedMs = 0;

  async mount(container: HTMLElement): Promise<void> {
    this.container = container;
    const app = new Application();
    await app.init({
      resizeTo: container,
      backgroundAlpha: 0,
      antialias: true,
      resolution: window.devicePixelRatio ?? 1,
      autoDensity: true,
    });
    container.appendChild(app.canvas);
    app.ticker.add((ticker) => this.tick(ticker.deltaMS));
    this.app = app;
  }

  async setCharacter(character: ActiveCharacter): Promise<void> {
    this.character = character;
    this.animations.clear();

    for (const [stateKey, state] of Object.entries(character.states)) {
      const textures = await this.loadFrames(state);
      this.animations.set(stateKey, {
        stateKey,
        textures,
        fps: state.fps,
        loop: state.loop,
        then: state.then,
        durationMs: state.duration_ms,
      });
    }
    // Kick off the default state.
    this.transitionTo(character.default_state);
  }

  /**
   * Resolve animation key with graceful fallback:
   *   happy_relieved → happy (if texture not supported)
   *   unknown        → default_state
   */
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

    // Build the "next" sprite.
    const nextSprite = new Sprite(next.textures[0]);
    centerSprite(nextSprite, this.app);
    nextSprite.alpha = 0;
    this.app.stage.addChild(nextSprite);
    this.nextSprite = nextSprite;

    // Fade current out, next in.
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
        this.nextSprite = null;
        this.currentAnim = next;
        this.frameIndex = 0;
        this.accumulatorMs = 0;
        this.elapsedMs = 0;
      }
    };
    requestAnimationFrame(fadeStep);
  }

  dispose(): void {
    this.animations.clear();
    this.currentSprite = null;
    this.nextSprite = null;
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

  private async loadFrames(state: StatePayload): Promise<Texture[]> {
    const urls = state.frames.map((p) => convertFileSrc(p));
    const textures = await Promise.all(
      urls.map(async (u) => {
        const tex = (await Assets.load(u)) as Texture;
        return tex;
      }),
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
        if (anim.loop) {
          this.frameIndex = 0;
        } else {
          this.frameIndex = anim.textures.length - 1;
        }
      }
      this.currentSprite.texture = anim.textures[this.frameIndex];
    }

    // Auto-transition after non-looping duration.
    if (!anim.loop && anim.durationMs && this.elapsedMs >= anim.durationMs) {
      const then = anim.then ?? this.character?.default_state ?? "idle";
      this.transitionTo(then);
    }
  }
}

function centerSprite(sprite: Sprite, app: Application): void {
  const maxDim = Math.min(app.canvas.width, app.canvas.height);
  const scale = (maxDim * 0.9) / Math.max(sprite.texture.width, sprite.texture.height);
  sprite.anchor.set(0.5, 0.5);
  sprite.position.set(app.canvas.width / 2, app.canvas.height / 2);
  sprite.scale.set(scale);
}
