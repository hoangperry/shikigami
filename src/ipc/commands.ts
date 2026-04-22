// Typed wrappers around Tauri invoke() so components never touch string commands.
import { invoke } from "@tauri-apps/api/core";

export type Settings = {
  port: number;
  active_character: string | null;
  click_through: boolean;
  opacity: number;
  scale: number;
  auto_hide_during_capture: boolean;
};

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export function updateSettings(patch: Partial<Settings>): Promise<Settings> {
  return invoke<Settings>("update_settings", { patch });
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

export type StatePayload = {
  fps: number;
  loop: boolean;
  then: string | null;
  duration_ms: number | null;
  /** Absolute paths on disk. Convert via convertFileSrc() before rendering. */
  frames: string[];
  textures: string[];
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
