use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use underlay_http::{
    etag_header_value, if_match_matches, if_none_match_matches, weak_etag_for_bytes, ApiError,
    CACHE_CONTROL_ADMIN_REVALIDATE,
};

pub fn detail_etag(resource_type: &str, resource_id: &str, updated_at: &str) -> String {
    let payload = format!("{resource_type}:{resource_id}:{updated_at}");
    weak_etag_for_bytes(payload.as_bytes())
}

pub fn build_etag_cache_headers(etag: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Some(value) = etag_header_value(etag) {
        headers.insert(header::ETAG, value);
    }
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_CONTROL_ADMIN_REVALIDATE),
    );
    headers
}

pub fn maybe_not_modified(request_headers: &HeaderMap, etag: &str) -> Option<Response> {
    if if_none_match_matches(request_headers, etag) {
        let headers = build_etag_cache_headers(etag);
        return Some((StatusCode::NOT_MODIFIED, headers).into_response());
    }
    None
}

pub fn if_match_mismatch(request_headers: &HeaderMap, etag: &str) -> bool {
    request_headers.contains_key(header::IF_MATCH) && !if_match_matches(request_headers, etag)
}

pub fn precondition_failed_error() -> ApiError {
    ApiError::new(
        StatusCode::PRECONDITION_FAILED,
        "resource.precondition_failed",
        "Resource changed on the server. Reload and retry your update.",
    )
}
