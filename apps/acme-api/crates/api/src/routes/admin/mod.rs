//! Admin routes for the Acme API.
//!
//! All routes in this module require admin privileges (via `AdminUser` extractor).
//! These demonstrate patterns used in production admin interfaces:
//! - Filtering and sorting via query parameters
//! - Soft delete with batch IDs
//! - Reordering endpoints
//! - Field validation endpoints

mod router;

pub use router::build_admin_router;

pub mod activity;
pub mod categories;
pub mod dashboard;
pub mod error_logs;
pub mod freshness;
pub mod jobs;
pub mod media;
pub mod projects;
pub mod reorder_conflict;
pub mod scheduled_tasks;
pub mod tasks;
pub mod users;
pub mod validation;
