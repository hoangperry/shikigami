//! Texture extraction — Stage 2 of the Hierarchical Fusion pipeline.
//!
//! Priority: first-match wins, in the order declared. Character-level
//! `emotionOverrides` are merged above these defaults at character load time
//! (not yet wired — see FR-058 character system work).

use super::canonical::Texture;
use once_cell::sync::Lazy;
use regex::Regex;

static PATTERNS: Lazy<Vec<(Regex, Texture)>> = Lazy::new(|| {
    vec![
        (
            Regex::new(r"(?i)\bfinally\b|phew").expect("relieved regex"),
            Texture::Relieved,
        ),
        (
            Regex::new(r"\(｡•̀ᴗ-\)✧|~$").expect("playful regex"),
            Texture::Playful,
        ),
        (
            Regex::new(r"(?i)\bagain\b|still failing|\bugh\b").expect("exhausted regex"),
            Texture::Exhausted,
        ),
        (
            Regex::new(r"⚠|(?i)\bdangerous\b|\bcareful\b").expect("alarmed regex"),
            Texture::Alarmed,
        ),
        (
            Regex::new(r"\*[^*\n]+\*|♡").expect("cute regex"),
            Texture::Cute,
        ),
        (
            Regex::new(r"(?i)told you|\( ?˶ˆᗜˆ˵ ?\)").expect("smug regex"),
            Texture::Smug,
        ),
    ]
});

pub fn extract(text: &str) -> Option<Texture> {
    PATTERNS
        .iter()
        .find(|(re, _)| re.is_match(text))
        .map(|(_, t)| *t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relieved_from_finally() {
        assert_eq!(
            extract("fix critical bug, finally"),
            Some(Texture::Relieved)
        );
    }

    #[test]
    fn alarmed_from_warning_sign() {
        assert_eq!(extract("⚠️ this is dangerous"), Some(Texture::Alarmed));
    }

    #[test]
    fn cute_from_action_text() {
        assert_eq!(extract("*em cười khẽ*"), Some(Texture::Cute));
    }

    #[test]
    fn none_on_plain_text() {
        assert_eq!(extract("a minor typo"), None);
    }

    #[test]
    fn empty_returns_none() {
        assert_eq!(extract(""), None);
    }
}
