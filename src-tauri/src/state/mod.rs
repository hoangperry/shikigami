//! State machine + canonical emotion vocabulary.
//!
//! Implements the Hierarchical Fusion architecture from ADR-002.

pub mod announcements;
pub mod canonical;
pub mod dampen;
pub mod idle_timer;
pub mod machine;
pub mod texture;

pub use canonical::{DominantState, ResolvedState, Severity, Texture};
pub use dampen::Dampener;
pub use idle_timer::IdleTracker;
pub use machine::resolve;
