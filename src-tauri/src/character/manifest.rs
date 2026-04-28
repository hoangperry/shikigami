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
    /// Live2D motion group name to play when entering this state. Optional.
    /// If absent, the renderer falls back to using the state name as the
    /// motion group (which often does not exist in community models — Cubism
    /// motion groups are typically `Idle`, `TapBody`, `Tap@<region>`, etc.).
    /// Sprite renderer ignores this field.
    #[serde(default)]
    pub motion: Option<String>,
    /// Pool of motion-group names; renderer picks one at random per
    /// transition. Overrides `motion` when non-empty. Lets the same
    /// dominant state surface different gestures across visits.
    #[serde(default)]
    pub motions: Vec<String>,
    /// Sequential motion chain. Each step plays its group after waiting
    /// `delay_ms` from the previous step. Use for longer dramatic
    /// reactions ("wave hand → return to idle"). Overrides `motion` /
    /// `motions` when non-empty.
    #[serde(default, rename = "motionChain")]
    pub motion_chain: Vec<MotionStep>,
    /// Single Cubism expression name (e.g. "F01"). Blended on top of
    /// motion — face overlay independent of body pose.
    #[serde(default)]
    pub expression: Option<String>,
    /// Pool of expressions; renderer picks one at random per transition.
    /// Overrides `expression` when non-empty.
    #[serde(default)]
    pub expressions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MotionStep {
    pub group: String,
    /// Delay in ms between this step and the next. Last step's delay is
    /// the chain's tail wait before re-asserting the loop motion (or 0).
    #[serde(default, rename = "delayMs")]
    pub delay_ms: u32,
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

    // Schema-extension coverage — make sure motion-pool / motion-chain /
    // expression / expressions all round-trip cleanly. A regression here
    // would silently strip the user's character config at load time.

    const HARU_FRAGMENT: &str = r#"{
      "$schema": "x",
      "schemaVersion": "1.0",
      "id": "test",
      "name": "T",
      "author": "x",
      "version": "1.0.0",
      "license": "X",
      "renderer": "live2d",
      "defaultState": "idle",
      "states": {
        "idle": { "path": "assets/states/idle", "fps": 30,
                  "motion": "Idle", "expressions": ["F01", "F02"] },
        "happy": { "path": "assets/states/happy", "fps": 30,
                   "motionChain": [{"group":"TapBody","delayMs":1500}, {"group":"Idle","delayMs":0}],
                   "expression": "F03" },
        "focused": { "path": "assets/states/focused", "fps": 30,
                     "motions": ["Idle", "TapBody"] }
      }
    }"#;

    #[test]
    fn motion_pool_parses() {
        let m: CharacterManifest = serde_json::from_str(HARU_FRAGMENT).expect("parse");
        let focused = &m.states["focused"];
        assert_eq!(
            focused.motions,
            vec!["Idle".to_string(), "TapBody".to_string()]
        );
        assert!(focused.motion.is_none());
    }

    #[test]
    fn motion_chain_parses_with_delays() {
        let m: CharacterManifest = serde_json::from_str(HARU_FRAGMENT).expect("parse");
        let happy = &m.states["happy"];
        assert_eq!(happy.motion_chain.len(), 2);
        assert_eq!(happy.motion_chain[0].group, "TapBody");
        assert_eq!(happy.motion_chain[0].delay_ms, 1500);
        assert_eq!(happy.motion_chain[1].group, "Idle");
        assert_eq!(happy.motion_chain[1].delay_ms, 0);
    }

    #[test]
    fn expression_pool_parses() {
        let m: CharacterManifest = serde_json::from_str(HARU_FRAGMENT).expect("parse");
        let idle = &m.states["idle"];
        assert_eq!(idle.expressions, vec!["F01".to_string(), "F02".to_string()]);
        assert!(idle.expression.is_none());
    }

    #[test]
    fn expression_single_parses() {
        let m: CharacterManifest = serde_json::from_str(HARU_FRAGMENT).expect("parse");
        let happy = &m.states["happy"];
        assert_eq!(happy.expression.as_deref(), Some("F03"));
        assert!(happy.expressions.is_empty());
    }

    #[test]
    fn manifest_without_extensions_still_parses() {
        // Backward compat: pre-extension manifests must continue to load.
        let m: CharacterManifest = serde_json::from_str(LINH_PIXEL).expect("parse");
        for state in m.states.values() {
            assert!(state.motions.is_empty());
            assert!(state.motion_chain.is_empty());
            assert!(state.expressions.is_empty());
        }
    }
}
