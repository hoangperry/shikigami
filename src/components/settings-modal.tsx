// Settings modal — triggered by tray "Preferences…" or Cmd+,.
// Mirrors the Rust `Settings` struct and persists updates via update_settings.

import { useEffect, useState } from "react";
import { getSettings, updateSettings, type Settings } from "../ipc/commands";

type Props = {
  open: boolean;
  onClose: () => void;
};

export function SettingsModal({ open, onClose }: Props) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    getSettings()
      .then((s) => {
        if (!cancelled) setSettings(s);
      })
      .catch((e) => console.warn("getSettings failed:", e));
    return () => {
      cancelled = true;
    };
  }, [open]);

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
    } catch (e) {
      console.warn("updateSettings failed:", e);
    } finally {
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

        <Row label={`Scale — ${Math.round(settings.scale * 100)}%`}>
          <input
            type="range"
            min={0.5}
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
