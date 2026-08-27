//! Front family: product-user routes.
//!
//! These are the domain surfaces a signed-in end user drives. They are not
//! operator routes and never appear under `/v1/admin/*`. An app with no
//! product-user API would omit this family rather than register an empty one.

mod router;
pub mod tasks;

pub use router::build_front_router;
