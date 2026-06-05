//! Piper TTS provider — local neural TTS. Offline, fast (~200ms for short
//! lines), good quality. Requires `piper` binary + a `.onnx` voice model.
//!
//! Install (macOS): `brew install piper-tts`.
//! Download voice models from https://huggingface.co/rhasspy/piper-voices

use super::provider::{TtsError, TtsOutput, TtsProvider};
use crate::config::settings::TtsConfig;
use async_trait::async_trait;
use std::path::Path;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub struct Piper {
    cfg: TtsConfig,
}

impl Piper {
    pub fn new(cfg: TtsConfig) -> Self {
        Self { cfg }
    }
}

/// Allowlisted prefixes for an absolute `piper` binary path on Unix.
/// Anything else must use the bare name `"piper"` (resolved via `$PATH`).
#[cfg(unix)]
const SAFE_BINARY_PREFIXES: &[&str] = &[
    "/usr/local/bin/",
    "/opt/homebrew/bin/",
    "/usr/bin/",
    "/opt/local/bin/",
];

/// Validate the user-configured `piper_binary`. The value comes from
/// `config.json` (user-writable, and writable by any process that can drive
/// the settings IPC), and is spawned as an executable — so an unrestricted
/// value is an arbitrary-code-execution lever. Accept only the bare name
/// `"piper"` (PATH lookup) or an absolute path. On Unix (the supported TTS
/// platform) the absolute path must additionally sit under a known-safe
/// install prefix; on other platforms the prefix list does not translate, so
/// requiring an absolute path is the bar (still blocks bare-name $PATH
/// injection of an arbitrary executable).
fn validate_binary(bin: &str) -> Result<&str, TtsError> {
    if bin == "piper" {
        return Ok(bin);
    }
    let p = Path::new(bin);
    if !p.is_absolute() {
        return Err(TtsError::MissingDep(
            "piper_binary must be either \"piper\" or an absolute path".into(),
        ));
    }
    #[cfg(unix)]
    {
        if !SAFE_BINARY_PREFIXES.iter().any(|pre| bin.starts_with(pre)) {
            return Err(TtsError::MissingDep(format!(
                "piper_binary path {bin:?} is outside the allowed install prefixes"
            )));
        }
    }
    Ok(bin)
}

/// Validate the `--model` path. Reject a leading `-` (would be parsed as a
/// flag by piper) and empty values.
fn validate_model(model: &str) -> Result<&str, TtsError> {
    if model.is_empty() {
        return Err(TtsError::MissingDep("piper_model is empty".into()));
    }
    if model.starts_with('-') {
        return Err(TtsError::MissingDep(
            "piper_model may not start with '-'".into(),
        ));
    }
    Ok(model)
}

#[async_trait]
impl TtsProvider for Piper {
    fn name(&self) -> &'static str {
        "piper"
    }

    async fn synthesize(&self, text: &str, _voice: Option<&str>) -> Result<TtsOutput, TtsError> {
        let bin = validate_binary(self.cfg.piper_binary.as_deref().unwrap_or("piper"))?;
        let model = validate_model(
            self.cfg
                .piper_model
                .as_deref()
                .ok_or_else(|| TtsError::MissingDep("piper_model not configured".into()))?,
        )?;

        let out = super::fresh_output_path("wav")?;

        // Piper reads text from stdin, writes WAV to --output_file.
        let mut child = Command::new(bin)
            .arg("--model")
            .arg(model)
            .arg("--output_file")
            .arg(&out)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TtsError::MissingDep(format!("piper binary not found ({bin}): {e}")))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .await
                .map_err(|e| TtsError::Subprocess(format!("piper stdin: {e}")))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| TtsError::Subprocess(format!("piper wait: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TtsError::Subprocess(format!(
                "piper exited {}: {stderr}",
                output.status
            )));
        }

        Ok(TtsOutput {
            path: out,
            mime: "audio/wav",
            provider: "piper",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_bare_piper() {
        // The bare name is accepted on every platform ($PATH lookup).
        assert_eq!(validate_binary("piper").unwrap(), "piper");
    }

    #[cfg(unix)]
    #[test]
    fn accepts_safe_absolute_paths_on_unix() {
        assert_eq!(
            validate_binary("/opt/homebrew/bin/piper").unwrap(),
            "/opt/homebrew/bin/piper"
        );
        assert!(validate_binary("/usr/local/bin/piper").is_ok());
        // Absolute but outside the allowlist is refused on Unix.
        assert!(validate_binary("/tmp/evil").is_err());
        assert!(validate_binary("/Users/me/.cache/payload").is_err());
    }

    #[test]
    fn rejects_relative_and_bare_executable_names() {
        // Relative / bare non-"piper" names would trigger a $PATH lookup of
        // an arbitrary executable — refused on every platform.
        assert!(validate_binary("./piper").is_err());
        assert!(validate_binary("rm").is_err());
        assert!(validate_binary("payload").is_err());
    }

    #[test]
    fn rejects_model_flag_injection() {
        assert!(validate_model("-c").is_err());
        assert!(validate_model("").is_err());
        assert!(validate_model("/path/to/voice.onnx").is_ok());
    }
}
