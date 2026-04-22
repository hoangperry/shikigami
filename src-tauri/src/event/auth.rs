//! Bearer token generation, storage, and constant-time verification.

use rand::RngCore;
use subtle::ConstantTimeEq;
use std::path::Path;

pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Write `token` to `path` with owner-only permissions (mode 0600 on Unix).
pub fn write_token(path: &Path, token: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, token)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

/// Read existing token or generate + persist a new one.
pub fn load_or_create_token(path: &Path) -> std::io::Result<String> {
    if path.exists() {
        let t = std::fs::read_to_string(path)?.trim().to_owned();
        if t.len() == 64 {
            return Ok(t);
        }
        tracing::warn!("token at {} was malformed, regenerating", path.display());
    }
    let t = generate_token();
    write_token(path, &t)?;
    Ok(t)
}

/// Constant-time comparison to prevent timing attacks.
pub fn verify(expected: &str, provided: &str) -> bool {
    if expected.len() != provided.len() {
        return false;
    }
    expected.as_bytes().ct_eq(provided.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_64_hex_chars() {
        let t = generate_token();
        assert_eq!(t.len(), 64);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn verify_accepts_identical() {
        assert!(verify("abc123", "abc123"));
    }

    #[test]
    fn verify_rejects_different() {
        assert!(!verify("abc123", "abc124"));
    }

    #[test]
    fn verify_rejects_different_length() {
        assert!(!verify("abc", "abc123"));
    }
}
