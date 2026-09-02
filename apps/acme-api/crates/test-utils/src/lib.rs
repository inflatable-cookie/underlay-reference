//! Test utilities for Acme API tests.
//!
//! This crate provides shared test helpers, fixtures, and utilities
//! for writing integration and unit tests across the Acme crates.
//!
//! # Usage
//!
//! ```ignore
//! use acme_test_utils::{fixtures, cleanup};
//!
//! #[tokio::test]
//! async fn test_create_project() {
//!     let pool = setup_test_db().await;
//!     let user = fixtures::create_test_user(&pool).await;
//!     let project = fixtures::create_test_project(&pool, user.id).await;
//!
//!     // ... test logic ...
//!
//!     cleanup::delete_user(&pool, user.id).await;
//! }
//! ```

pub mod cleanup;
pub mod db;
pub mod fixtures;

pub use acme_db::media::activate_ready_current_failing_after_version_ready;
pub use db::{setup_test_db, shared_runtime, TestDb};
pub use fixtures::{TestCategory, TestProject, TestTask, TestUser};
