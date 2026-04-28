// Settings modal — triggered by tray "Preferences…" or Cmd+,.
// Mirrors the Rust `Settings` struct and persists updates via update_settings.

import { useEffect, useState } from "react";
import {
  applyRuntimeSettings,
  getSettings,
  listCharacters,
  listSessions,
  setActiveCharacter,
  setSessionAllowed,
  updateSettings,
  type CharacterSummary,
  type SessionInfo,
  type Settings,
  type TtsConfig,
} from "../ipc/commands";

const TTS_PROVIDERS = [
  { value: "none", label: "Off" },
  { value: "say-macos", label: "macOS say (built-in)" },
  { value: "piper", label: "Piper (local neural)" },
  { value: "openai", label: "OpenAI" },
  { value: "elevenlabs", label: "ElevenLabs" },
];

// Voice hint per provider — surfaces sensible defaults so users without
// external doc-diving still get usable output.
const VOICE_PLACEHOLDER: Record<string, string> = {
  "say-macos": "Linh / Samantha / Bubbles…",
  piper: "(set piper_model path in config)",
  openai: "alloy / nova / shimmer / echo",
  elevenlabs: "voice_id (e.g. 21m00Tcm4TlvDq8ikWAM)",
};

type Props = {
  open: boolean;
  onClose: () => void;
};

export function SettingsModal({ open, onClose }: Props) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [characters, setCharacters] = useState<CharacterSummary[]>([]);
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [saving, setSaving] = useState(false);

  // Load settings, character roster, and active session list when the
  // modal opens. The session list is also polled every 3s while open so
  // newly-active Claude tabs show up without re-opening the modal.
  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    Promise.all([getSettings(), listCharacters(), listSessions()])
      .then(([s, chars, sess]) => {
        if (cancelled) return;
        setSettings(s);
        setCharacters(chars);
        setSessions(sess);
      })
      .catch((e) => console.warn("settings init failed:", e));
    const t = window.setInterval(() => {
      listSessions()
        .then((sess) => {
          if (!cancelled) setSessions(sess);
        })
        .catch(() => {});
    }, 3000);
    return () => {
      cancelled = true;
      window.clearInterval(t);
    };
  }, [open]);

  const toggleSession = async (id: string, allowed: boolean) => {
    // Optimistic — the backend write is cheap and reliable; flipping
    // locally first keeps the checkbox snappy.
    setSessions((prev) =>
      prev.map((s) => (s.id === id ? { ...s, allowed } : s)),
    );
    try {
      await setSessionAllowed(id, allowed);
    } catch (e) {
      console.warn("setSessionAllowed failed:", e);
      // Revert on failure.
      setSessions((prev) =>
        prev.map((s) => (s.id === id ? { ...s, allowed: !allowed } : s)),
      );
    }
  };

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open || !settings) return null;

  const update = async (patch: Partial<Settings>) => {
    setSaving(true);
    try {
      const next = await updateSettings(patch);
      setSettings(next);
      // Window-level knobs (click-through) need a runtime apply, otherwise
      // the toggle only takes effect on next launch.
      if ("click_through" in patch) {
        await applyRuntimeSettings().catch((e) =>
          console.warn("applyRuntimeSettings failed:", e),
        );
      }
    } catch (e) {
      console.warn("updateSettings failed:", e);
    } finally {
      setSaving(false);
    }
  };

  // Patch a sub-field of settings.tts in one call.
  const updateTts = (patch: Partial<TtsConfig>) =>
    update({ tts: { ...settings.tts, ...patch } });

  // Switching characters needs a renderer remount (Live2D model + Pixi
  // stage are bound to the active character at mount time). Simplest
  // KISS approach: persist the selection then full-reload — avoids the
  // class of bugs where the WebGL context is reused across model swaps.
  const switchCharacter = async (id: string) => {
    if (id === settings.active_character) return;
    setSaving(true);
    try {
      await setActiveCharacter(id);
      // Reflect locally before reload so the dropdown reads correctly
      // during the brief window before the page tears down.
      setSettings({ ...settings, active_character: id });
      // Full reload remounts CharacterStage with the new active char.
      window.location.reload();
    } catch (e) {
      console.warn("setActiveCharacter failed:", e);
      setSaving(false);
    }
  };

  return (
    <div
      data-tauri-drag-region={false}
      onClick={onClose}
      style={{
        position: "absolute",
        inset: 0,
        background: "rgba(0,0,0,0.42)",
        zIndex: 30,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          width: 320,
          maxWidth: "92%",
          padding: 16,
          borderRadius: 14,
          background: "rgba(22,22,30,0.97)",
          color: "#f5f5f5",
          fontFamily:
            "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
          fontSize: 12,
          boxShadow: "0 16px 48px rgba(0,0,0,0.45)",
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: 10,
          }}
        >
          <strong style={{ fontSize: 14 }}>Preferences</strong>
          <button
            onClick={onClose}
            style={{
              background: "transparent",
              border: "none",
              color: "#f5f5f5",
              opacity: 0.7,
              cursor: "pointer",
              fontSize: 16,
            }}
            aria-label="Close"
          >
            ×
          </button>
        </div>

        <Row label="Character">
          <select
            value={settings.active_character ?? ""}
            onChange={(e) => switchCharacter(e.target.value)}
            disabled={characters.length === 0 || saving}
            style={selectStyle}
          >
            {characters.length === 0 && (
              <option value="">(loading…)</option>
            )}
            {characters.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        </Row>

        <Row label={`Scale — ${Math.round(settings.scale * 100)}%`}>
          <input
            type="range"
            min={0.3}
            max={2.0}
            step={0.05}
            value={settings.scale}
            onChange={(e) => update({ scale: parseFloat(e.target.value) })}
            style={{ width: "100%" }}
          />
        </Row>

        <Row label={`Opacity — ${Math.round(settings.opacity * 100)}%`}>
          <input
            type="range"
            min={0.2}
            max={1.0}
            step={0.02}
            value={settings.opacity}
            onChange={(e) => update({ opacity: parseFloat(e.target.value) })}
            style={{ width: "100%" }}
          />
        </Row>

        <Row label="Click-through">
          <input
            type="checkbox"
            checked={settings.click_through}
            onChange={(e) => update({ click_through: e.target.checked })}
          />
        </Row>

        <Row label="Auto-hide during screen capture">
          <input
            type="checkbox"
            checked={settings.auto_hide_during_capture}
            onChange={(e) =>
              update({ auto_hide_during_capture: e.target.checked })
            }
          />
        </Row>

        <Divider label="Voice (TTS)" />

        <Row label="Provider">
          <select
            value={settings.tts.provider}
            onChange={(e) => updateTts({ provider: e.target.value })}
            style={selectStyle}
          >
            {TTS_PROVIDERS.map((p) => (
              <option key={p.value} value={p.value}>
                {p.label}
              </option>
            ))}
          </select>
        </Row>

        {settings.tts.provider !== "none" && (
          <>
            <Row label="Voice">
              <input
                type="text"
                value={settings.tts.voice ?? ""}
                placeholder={VOICE_PLACEHOLDER[settings.tts.provider] ?? ""}
                onChange={(e) =>
                  updateTts({ voice: e.target.value.trim() || null })
                }
                style={textInputStyle}
              />
            </Row>

            <Row label={`Rate — ${settings.tts.rate.toFixed(2)}×`}>
              <input
                type="range"
                min={0.5}
                max={2.0}
                step={0.05}
                value={settings.tts.rate}
                onChange={(e) =>
                  updateTts({ rate: parseFloat(e.target.value) })
                }
                style={{ width: "100%" }}
              />
            </Row>

            <Row
              label={
                settings.tts.volume <= 0.001
                  ? "Volume — muted"
                  : `Volume — ${Math.round(settings.tts.volume * 100)}%`
              }
            >
              <input
                type="range"
                min={0}
                max={1.0}
                step={0.05}
                value={settings.tts.volume}
                onChange={(e) =>
                  updateTts({ volume: parseFloat(e.target.value) })
                }
                style={{ width: "100%" }}
              />
            </Row>

            <Row label="Announce events">
              <input
                type="checkbox"
                checked={settings.tts.announce_events}
                onChange={(e) =>
                  updateTts({ announce_events: e.target.checked })
                }
              />
            </Row>

            {(settings.tts.provider === "openai" ||
              settings.tts.provider === "elevenlabs") && (
              <Row label="API key (or env)">
                <input
                  type="password"
                  value={settings.tts.api_key ?? ""}
                  placeholder={
                    settings.tts.provider === "openai"
                      ? "OPENAI_API_KEY env wins"
                      : "ELEVENLABS_API_KEY env wins"
                  }
                  onChange={(e) =>
                    updateTts({ api_key: e.target.value.trim() || null })
                  }
                  style={textInputStyle}
                />
              </Row>
            )}

            {settings.tts.provider === "piper" && (
              <>
                <Row label="Piper binary">
                  <input
                    type="text"
                    value={settings.tts.piper_binary ?? ""}
                    placeholder="piper (PATH lookup)"
                    onChange={(e) =>
                      updateTts({
                        piper_binary: e.target.value.trim() || null,
                      })
                    }
                    style={textInputStyle}
                  />
                </Row>
                <Row label="Piper model">
                  <input
                    type="text"
                    value={settings.tts.piper_model ?? ""}
                    placeholder="path/to/voice.onnx"
                    onChange={(e) =>
                      updateTts({
                        piper_model: e.target.value.trim() || null,
                      })
                    }
                    style={textInputStyle}
                  />
                </Row>
              </>
            )}
          </>
        )}

        <Divider label={`Active sessions (${sessions.length})`} />

        {sessions.length === 0 ? (
          <div style={{ fontSize: 10, opacity: 0.5, padding: "4px 0" }}>
            No Claude Code session has sent events yet.
          </div>
        ) : (
          <div style={{ maxHeight: 140, overflowY: "auto", marginTop: 4 }}>
            {sessions.map((s) => (
              <SessionRow
                key={s.id}
                session={s}
                onToggle={(allowed) => toggleSession(s.id, allowed)}
              />
            ))}
          </div>
        )}

        <div
          style={{
            fontSize: 10,
            opacity: 0.5,
            marginTop: 10,
            lineHeight: 1.5,
          }}
        >
          event server port: {settings.port}
          {saving && " · saving…"}
        </div>
      </div>
    </div>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: 12,
        margin: "10px 0",
      }}
    >
      <span style={{ minWidth: 130, opacity: 0.85 }}>{label}</span>
      <span style={{ flex: 1, maxWidth: 160, textAlign: "right" }}>{children}</span>
    </label>
  );
}

function Divider({ label }: { label: string }) {
  return (
    <div
      style={{
        marginTop: 14,
        marginBottom: 4,
        paddingTop: 8,
        borderTop: "1px solid rgba(255,255,255,0.12)",
        fontSize: 10,
        textTransform: "uppercase",
        letterSpacing: 0.6,
        opacity: 0.55,
      }}
    >
      {label}
    </div>
  );
}

const textInputStyle: React.CSSProperties = {
  width: "100%",
  background: "rgba(255,255,255,0.08)",
  color: "#f5f5f5",
  border: "1px solid rgba(255,255,255,0.15)",
  borderRadius: 6,
  padding: "4px 6px",
  fontSize: 11,
  textAlign: "right",
};

const selectStyle: React.CSSProperties = {
  width: "100%",
  background: "rgba(255,255,255,0.08)",
  color: "#f5f5f5",
  border: "1px solid rgba(255,255,255,0.15)",
  borderRadius: 6,
  padding: "4px 6px",
  fontSize: 11,
};

// A single row in the session picker — checkbox to (un)mute that tab,
// label = cwd basename, secondary = event count + relative time.
function SessionRow({
  session,
  onToggle,
}: {
  session: SessionInfo;
  onToggle: (allowed: boolean) => void;
}) {
  const ageMs = Date.now() - session.last_seen_ms;
  const relTime =
    ageMs < 5_000
      ? "now"
      : ageMs < 60_000
        ? `${Math.round(ageMs / 1000)}s ago`
        : ageMs < 3_600_000
          ? `${Math.round(ageMs / 60_000)}m ago`
          : `${Math.round(ageMs / 3_600_000)}h ago`;

  return (
    <label
      style={{
        display: "flex",
        alignItems: "center",
        gap: 10,
        padding: "5px 6px",
        borderRadius: 6,
        background: session.allowed
          ? "rgba(255,255,255,0.04)"
          : "rgba(255,255,255,0.01)",
        marginBottom: 4,
        cursor: "pointer",
        opacity: session.allowed ? 1 : 0.55,
      }}
    >
      <input
        type="checkbox"
        checked={session.allowed}
        onChange={(e) => onToggle(e.target.checked)}
      />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div
          style={{
            fontSize: 11,
            fontWeight: 600,
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
          }}
          title={session.cwd ?? session.id}
        >
          {session.label}
        </div>
        <div style={{ fontSize: 9, opacity: 0.55 }}>
          {session.event_count} events · {relTime}
        </div>
      </div>
    </label>
  );
}
