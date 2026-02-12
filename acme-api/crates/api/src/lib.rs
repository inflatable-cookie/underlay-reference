//! Acme HTTP API library.
//!
//! Keep handlers and router construction here so the binary entrypoint stays thin.

mod db_errors;

pub mod config;
pub mod dto;
pub mod openapi;
pub mod routes;
pub mod state;
