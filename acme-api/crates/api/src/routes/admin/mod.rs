//! Admin routes for the Acme API.
//!
//! All routes in this module require admin privileges (via `AdminUser` extractor).
//! These demonstrate patterns used in production admin interfaces:
//! - Filtering and sorting via query parameters
//! - Soft delete with batch IDs
//! - Reordering endpoints
//! - Field validation endpoints

pub mod categories;
pub mod media;
pub mod projects;
pub mod tasks;
pub mod validation;
