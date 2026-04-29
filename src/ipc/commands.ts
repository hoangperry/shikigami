// Typed wrappers around Tauri invoke() so components never touch string commands.
import { invoke } from "@tauri-apps/api/core";

export type TtsConfig = {
  /** "none" | "say-macos" | "piper" | "openai" | "elevenlabs" */
  provider: string;
  voice: string | null;
  api_key: string | null;
  piper_binary: string | null;
  piper_model: string | null;
  rate: number;
  /** 0.0 = mute, 1.0 = full. Applied client-side at audio playback. */
  volume: number;
  /** When true, Hiyori speaks a short Vietnamese phrase on state
   *  transitions (focused/happy/warning/critical/etc) — opt-in. */
  announce_events: boolean;
};

export type Settings = {
  port: number;
  active_character: string | null;
  click_through: boolean;
  opacity: number;
  scale: number;
  auto_hide_during_capture: boolean;
  tts: TtsConfig;
};

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export function updateSettings(patch: Partial<Settings>): Promise<Settings> {
  return invoke<Settings>("update_settings", { patch });
}

/** Re-applies window-level settings (click-through, etc) without restart. */
export function applyRuntimeSettings(): Promise<void> {
  return invoke<void>("apply_runtime_settings_cmd");
}

/** Screen-space (physical pixels) AABB of the rendered character. The
 *  backend uses this for smart click-through hit-testing. */
export type CharacterBBox = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export function setCharacterBbox(bbox: CharacterBBox): Promise<void> {
  return invoke<void>("set_character_bbox", { bbox });
}

/** Install a `.shikigami` zip package. Returns the new character id on
 *  success; the filesystem watcher then triggers a registry reload. */
export function installCharacterZip(path: string): Promise<string> {
  return invoke<string>("install_character_zip", { path });
}

export type CharacterSummary = {
  id: string;
  name: string;
  author: string;
  version: string;
  is_active: boolean;
  default_state: string;
  state_count: number;
};

export type MotionStep = {
  group: string;
  delay_ms: number;
};

export type StatePayload = {
  fps: number;
  loop: boolean;
  then: string | null;
  duration_ms: number | null;
  /** Absolute paths on disk. Convert via convertFileSrc() before rendering. */
  frames: string[];
  /** Texture variant name → frame paths. Renderer plays these when the
   *  resolved animation key is `<state>_<texture-name>`. */
  textures: Record<string, string[]>;
  /**
   * Live2D motion group name. Resolution priority on entry:
   *   motion_chain (non-empty) > motions (non-empty) > motion > state name.
   */
  motion: string | null;
  motions: string[];
  motion_chain: MotionStep[];
  /**
   * Cubism expression overlay. `expressions` (pool) overrides `expression`
   * (single) when non-empty.
   */
  expression: string | null;
  expressions: string[];
};

export type ActiveCharacter = {
  id: string;
  name: string;
  default_state: string;
  states: Record<string, StatePayload>;
};

export function listCharacters(): Promise<CharacterSummary[]> {
  return invoke<CharacterSummary[]>("list_characters");
}

export function getActiveCharacter(): Promise<ActiveCharacter | null> {
  return invoke<ActiveCharacter | null>("get_active_character");
}

export function setActiveCharacter(id: string): Promise<void> {
  return invoke<void>("set_active_character", { id });
}

export type SessionInfo = {
  id: string;
  /** Short label — basename of cwd, or first 8 chars of session id. */
  label: string;
  cwd: string | null;
  event_count: number;
  /** Unix epoch millis when last event was observed. */
  last_seen_ms: number;
  /** When false, events from this session are dropped at the server. */
  allowed: boolean;
};

export function listSessions(): Promise<SessionInfo[]> {
  return invoke<SessionInfo[]>("list_sessions");
}

export function setSessionAllowed(id: string, allowed: boolean): Promise<void> {
  return invoke<void>("set_session_allowed", { id, allowed });
}

/** Payload of the Tauri `tts:speak` event emitted after `/v1/say` synth. */
export type SpeakEvent = {
  /** Absolute path to the generated audio file on disk. Use convertFileSrc(). */
  audio_url: string;
  mime: string;
  provider: string;
  text: string;
};
