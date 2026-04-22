//! Event ingress: HTTP POST server, bearer auth, JSON payload schema.

pub mod auth;
pub mod schema;
pub mod server;

pub use schema::{EventPayload, EventSource, EventType};
pub use server::{serve, AppState, StateEmitter};
