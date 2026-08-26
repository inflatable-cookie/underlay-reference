//! Acme core primitives.
//!
//! This crate deliberately re-exports Underlay primitives so the app can
//! adopt shared infrastructure without pushing app-specific concepts into Underlay.

pub use underlay_core::{AppError, AppResult, IdGenerator, RawUuid, SystemIdGenerator, Uuid};
