//! macOS `say(1)` provider — zero install, zero key. Best for MVP / fallback.
//!
//! Strategy:
//!   1. Spawn `say -v <voice> -r <rate-wpm> -o <out>.aiff <text>`
//!   2. AIFF is uncompressed PCM and decodable by Web Audio in WKWebView,
//!      so we keep it as-is (no transcode dep on `afconvert`).

use super::provider::{TtsError, TtsOutput, TtsProvider};
use crate::config::settings::TtsConfig;
use async_trait::async_trait;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub struct SayMacos {
    cfg: TtsConfig,
}

impl SayMacos {
    pub fn new(cfg: TtsConfig) -> Self {
        Self { cfg }
    }
}

/// Validate a `say -v` voice name. The voice arrives from the (token-gated)
/// `/v1/say` API, so it is attacker-influenced. `Command` uses execvp (no
/// shell), so the only injection surface is argument-smuggling: a value
/// beginning with `-` would be parsed by `say` as an extra flag (e.g.
/// `-o /etc/something`). Reject leading dashes, control characters, and
/// absurd lengths. Returns the validated voice, or an error.
fn validate_voice(voice: &str) -> Result<&str, TtsError> {
    let v = voice.trim();
    if v.is_empty() {
        return Err(TtsError::Subprocess("empty voice name".into()));
    }
    if v.len() > 100 {
        return Err(TtsError::Subprocess("voice name too long".into()));
    }
    if v.starts_with('-') {
        return Err(TtsError::Subprocess(
            "voice name may not start with '-'".into(),
        ));
    }
    if v.chars().any(|c| c.is_control()) {
        return Err(TtsError::Subprocess(
            "voice name contains control characters".into(),
        ));
    }
    Ok(v)
}

#[async_trait]
impl TtsProvider for SayMacos {
    fn name(&self) -> &'static str {
        "say-macos"
    }

    async fn synthesize(&self, text: &str, voice: Option<&str>) -> Result<TtsOutput, TtsError> {
        let out = super::fresh_output_path("aiff")?;
        let voice = validate_voice(voice.or(self.cfg.voice.as_deref()).unwrap_or("Samantha"))?;
        // `say -r` expects words-per-minute (180 ≈ natural speech). Map our
        // 1.0 = normal multiplier into a reasonable WPM band.
        let wpm = (180.0 * self.cfg.rate.clamp(0.5, 2.0)).round() as u32;

        // The spoken text is piped via stdin rather than passed as a CLI
        // argument. `say` reads stdin when no message operand is given, so
        // there is no positional arg for the text to be mis-parsed as a
        // flag (e.g. text = "-f /etc/passwd" would otherwise make `say`
        // read a file). This closes the argument-injection surface for the
        // text payload entirely; `voice` is separately validated above.
        let mut child = Command::new("say")
            .arg("-v")
            .arg(voice)
            .arg("-r")
            .arg(wpm.to_string())
            .arg("-o")
            .arg(&out)
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| TtsError::Subprocess(format!("spawn say: {e}")))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .await
                .map_err(|e| TtsError::Subprocess(format!("say stdin: {e}")))?;
            // Drop stdin so `say` sees EOF and begins synthesis.
        }

        let status = child
            .wait()
            .await
            .map_err(|e| TtsError::Subprocess(format!("say wait: {e}")))?;

        if !status.success() {
            return Err(TtsError::Subprocess(format!(
                "say exited with {status} (voice={voice})"
            )));
        }

        Ok(TtsOutput {
            path: out,
            mime: "audio/aiff",
            provider: "say-macos",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_voice_names() {
        assert_eq!(validate_voice("Samantha").unwrap(), "Samantha");
        assert_eq!(validate_voice("Linh").unwrap(), "Linh");
        assert_eq!(validate_voice("  Daniel  ").unwrap(), "Daniel");
    }

    #[test]
    fn rejects_flag_injection_via_leading_dash() {
        assert!(validate_voice("-o").is_err());
        assert!(validate_voice("--rate").is_err());
        assert!(validate_voice("-o /tmp/pwned").is_err());
    }

    #[test]
    fn rejects_empty_and_control_chars() {
        assert!(validate_voice("").is_err());
        assert!(validate_voice("   ").is_err());
        assert!(validate_voice("Sam\nantha").is_err());
    }

    #[test]
    fn rejects_absurd_length() {
        assert!(validate_voice(&"a".repeat(101)).is_err());
    }
}
