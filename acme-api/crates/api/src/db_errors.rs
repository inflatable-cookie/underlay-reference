use underlay_http::ApiError;

/// Build a 500 ApiError for a failed database operation.
///
/// Thin wrapper over the canonical underlay implementation: the client-facing
/// message is the static `operation` string only; schema diagnostics stay in
/// the log-only error context.
pub(crate) fn internal_with_diagnostics<E>(code: &'static str, operation: &str, err: &E) -> ApiError
where
    E: std::any::Any + std::fmt::Display,
{
    underlay_http::internal_db_error(code, operation, err)
}
