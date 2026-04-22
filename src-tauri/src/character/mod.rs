//! Character system: manifest, filesystem loader, registry.

pub mod loader;
pub mod manifest;
pub mod registry;

pub use loader::{load_from_dir, LoadError, LoadedCharacter};
pub use manifest::{CharacterManifest, StateDef};
pub use registry::{ActiveCharacter, CharacterRegistry, CharacterSummary, StatePayload};
