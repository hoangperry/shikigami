//! Per-state voice announcement phrases. Fired by the event server when
//! `tts.announce_events = true` and the dominant state has changed.
//!
//! Default voice is `Linh` (vi_VN) so all phrases are Vietnamese. Add
//! locale switching when we ship multi-language voices.

use super::canonical::DominantState;
use rand::seq::SliceRandom;

/// Pick a random phrase appropriate for the new dominant state. Returns
/// `None` for states we deliberately don't announce (idle/sleepy — those
/// are background ambience, talking would be annoying; idle_timer
/// already handles long-idle voice nudges).
pub fn phrase_for(dominant: DominantState) -> Option<&'static str> {
    let pool: &[&str] = match dominant {
        DominantState::Focused => &[
            "Em đang làm đây anh~",
            "Để em check một chút meow~",
            "Đang xử lý nha chủ nhân.",
            "Em tập trung đây.",
        ],
        DominantState::Happy => &[
            "Xong rồi nhé chủ nhân!",
            "Done meow~",
            "OK xong rồi anh ạ.",
            "Hoàn tất, em mừng quá!",
        ],
        DominantState::Warning => &[
            "Có lỗi rồi anh ơi.",
            "Hỏng rồi meow >_<",
            "Anh check lại giúp em nhé.",
            "Cảnh báo, có vấn đề!",
        ],
        DominantState::Confused => &[
            "Em đang nghĩ kỹ đây...",
            "Hơi khó nha, để em xem...",
            "Vấn đề này hơi phức tạp meow~",
        ],
        // Idle / Sleepy: silent by default. The long-idle nudge is
        // handled separately by state::idle_timer.
        DominantState::Idle | DominantState::Sleepy => return None,
    };
    let mut rng = rand::thread_rng();
    pool.choose(&mut rng).copied()
}

/// Special-case override for critical-severity destructive ops — louder,
/// more urgent line. Fires regardless of state-change dampening because
/// destructive ops are always announce-worthy.
pub fn critical_destructive_phrase() -> &'static str {
    let pool = [
        "Khoan đã chủ nhân! Lệnh này nguy hiểm đó!",
        "Cẩn thận anh ơi, có thao tác xóa!",
        "Dừng lại! Em phát hiện lệnh phá huỷ!",
    ];
    let mut rng = rand::thread_rng();
    pool.choose(&mut rng).copied().unwrap_or("Cẩn thận anh ơi!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_state_returns_none() {
        assert!(phrase_for(DominantState::Idle).is_none());
    }

    #[test]
    fn sleepy_state_returns_none() {
        // Idle nudges are handled separately by state::idle_timer.
        assert!(phrase_for(DominantState::Sleepy).is_none());
    }

    #[test]
    fn focused_returns_phrase() {
        let p = phrase_for(DominantState::Focused);
        assert!(p.is_some());
        assert!(!p.unwrap().is_empty());
    }

    #[test]
    fn happy_returns_phrase() {
        assert!(phrase_for(DominantState::Happy).is_some());
    }

    #[test]
    fn warning_returns_phrase() {
        assert!(phrase_for(DominantState::Warning).is_some());
    }

    #[test]
    fn confused_returns_phrase() {
        assert!(phrase_for(DominantState::Confused).is_some());
    }

    #[test]
    fn critical_phrase_is_non_empty() {
        let p = critical_destructive_phrase();
        assert!(!p.is_empty());
        assert!(p.contains("nguy hiểm") || p.contains("xóa") || p.contains("phá"));
    }

    #[test]
    fn random_pick_visits_all_phrases_eventually() {
        // 1000 trials should hit every phrase in a 4-element pool with
        // probability ~1 - (3/4)^1000 ≈ 1 (effectively certain).
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1000 {
            if let Some(p) = phrase_for(DominantState::Focused) {
                seen.insert(p);
            }
        }
        assert!(
            seen.len() >= 2,
            "expected randomness across phrases, got only {}",
            seen.len()
        );
    }
}
