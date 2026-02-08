//! Task and project database operations.
//!
//! Example domain queries demonstrating common patterns including:
//! - Filtering and sorting via QueryParams
//! - Soft delete with batch IDs
//! - Relations (projects -> tasks, tasks -> labels)
//! - Admin queries with counts

mod comments;
mod crud;
mod labels;
mod projects;

pub use comments::*;
pub use crud::*;
pub use labels::*;
pub use projects::*;
