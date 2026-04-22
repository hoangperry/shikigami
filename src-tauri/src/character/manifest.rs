//! Character manifest parsing. Mirrors `schemas/manifest.v1.0.json`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterManifest {
    pub schema_version: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub author: serde_json::Value, // string or { name, url }
    pub version: String,
    pub license: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub renderer: String,
    pub default_state: String,
    pub states: BTreeMap<String, StateDef>,
    #[serde(default)]
    pub emotion_overrides: BTreeMap<String, EmotionOverride>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateDef {
    pub path: String,
    pub fps: u32,
    #[serde(default)]
    pub r#loop: bool,
    #[serde(default)]
    pub then: Option<String>,
    #[serde(default, rename = "durationMs")]
    pub duration_ms: Option<u32>,
    #[serde(default)]
    pub textures: BTreeMap<String, String>,
    #[serde(default = "default_blendable")]
    pub blendable: bool,
}

fn default_blendable() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmotionOverride {
    #[serde(default)]
    pub texture: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
}

impl CharacterManifest {
    /// Basic structural validation on top of JSON Schema.
    /// Returns a list of human-readable issues; empty = all good.
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.schema_version != "1.0" {
            issues.push(format!(
                "schemaVersion {:?} unsupported, expected \"1.0\"",
                self.schema_version
            ));
        }
        if !self
            .id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            issues.push(format!("id {:?} has invalid characters", self.id));
        }
        if self.renderer != "sprite" && self.renderer != "live2d" {
            issues.push(format!(
                "renderer {:?} not supported in v1.0 (only \"sprite\" or \"live2d\")",
                self.renderer
            ));
        }
        if !self.states.contains_key("idle") {
            issues.push("missing required state: idle".into());
        }
        if !self.states.contains_key("happy") {
            issues.push("missing required state: happy".into());
        }
        if !self.states.contains_key(&self.default_state) {
            issues.push(format!(
                "defaultState {:?} is not defined in states",
                self.default_state
            ));
        }
        for (name, state) in &self.states {
            if !state.path.starts_with("assets/states/") {
                issues.push(format!(
                    "state {:?} has invalid path {:?} (must be under assets/states/)",
                    name, state.path
                ));
            }
            if state.fps == 0 || state.fps > 60 {
                issues.push(format!(
                    "state {:?} has fps {} out of [1,60]",
                    name, state.fps
                ));
            }
        }

        issues
    }

    /// Author field may be a plain string or an object with `name` + `url`.
    pub fn author_name(&self) -> String {
        match &self.author {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(o) => o
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>")
                .to_owned(),
            _ => "<unknown>".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINH_PIXEL: &str = include_str!("../../../characters/linh-pixel/manifest.json");

    #[test]
    fn linh_pixel_manifest_parses() {
        let m: CharacterManifest = serde_json::from_str(LINH_PIXEL).expect("parse");
        assert_eq!(m.id, "linh-pixel");
        assert_eq!(m.renderer, "sprite");
        assert_eq!(m.default_state, "idle");
        assert!(m.validate().is_empty(), "validation: {:?}", m.validate());
    }

    #[test]
    fn missing_idle_state_fails_validation() {
        let mut m: CharacterManifest = serde_json::from_str(LINH_PIXEL).unwrap();
        m.states.remove("idle");
        let issues = m.validate();
        assert!(issues.iter().any(|i| i.contains("idle")));
    }

    #[test]
    fn invalid_renderer_fails_validation() {
        let mut m: CharacterManifest = serde_json::from_str(LINH_PIXEL).unwrap();
        m.renderer = "unicorn".into();
        let issues = m.validate();
        assert!(issues.iter().any(|i| i.contains("renderer")));
    }

    #[test]
    fn live2d_renderer_is_valid() {
        let mut m: CharacterManifest = serde_json::from_str(LINH_PIXEL).unwrap();
        m.renderer = "live2d".into();
        // live2d accepted; other validation issues may remain but none
        // should mention renderer
        let issues = m.validate();
        assert!(
            !issues.iter().any(|i| i.contains("renderer")),
            "unexpected renderer validation issue: {issues:?}"
        );
    }
}
