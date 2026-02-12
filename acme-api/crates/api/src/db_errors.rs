use underlay_http::ApiError;

pub(crate) fn internal_with_diagnostics<E>(code: &'static str, operation: &str, err: &E) -> ApiError
where
    E: std::any::Any + std::fmt::Display,
{
    let sqlx_err = (err as &dyn std::any::Any)
        .downcast_ref::<sqlx::Error>()
        .or_else(|| {
            (err as &dyn std::any::Any)
                .downcast_ref::<Box<dyn std::error::Error + Send + Sync>>()
                .and_then(|boxed| boxed.as_ref().downcast_ref::<sqlx::Error>())
        });

    let message = sqlx_err
        .map(|db_err| underlay_db::describe_db_error(operation, db_err))
        .unwrap_or_else(|| format!("{operation}. Cause: {err}"));

    ApiError::internal(code, message).with_cause(err)
}
