//! Upload-policy tests for initiate (g01.009, retained by g01.013).
//!
//! Initiate still delegates size/MIME enforcement to
//! `initiate_upload_validated`. Live finalisation is covered by the
//! `promote_verified_owned` composition tests.

use crate::config::AcmeConfig;
use underlay_blob::{
    BlobAdapterUploadExt, BlobError, BlobObjectKey, LocalAdapter, LocalConfig, UploadRequest,
    DEFAULT_ALLOWED_CONTENT_TYPES,
};

async fn test_adapter() -> (LocalAdapter, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!(
        "acme-media-validation-{}",
        acme_core::Uuid::new_v7()
    ));
    std::fs::create_dir_all(&dir).expect("create temp media dir");
    let adapter = LocalAdapter::new(LocalConfig::new(&dir, "http://localhost:0/uploads"))
        .await
        .expect("build local adapter");
    (adapter, dir)
}

fn upload_request(key: &str, content_type: &str, size: u64) -> UploadRequest {
    UploadRequest::from_object_key(
        BlobObjectKey::parse(key).expect("valid object key"),
        content_type,
        size,
    )
}

#[test]
fn upload_policy_is_foundation_defaults() {
    let media = AcmeConfig::default().media;

    // Allowlist and size limit come straight from the foundation defaults.
    assert_eq!(
        media.allowed_content_types(),
        DEFAULT_ALLOWED_CONTENT_TYPES
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(media.max_file_size_bytes_limit(), 50 * 1024 * 1024);

    // Active/scriptable content stays excluded.
    for banned in ["image/svg+xml", "text/html", "application/javascript"] {
        assert!(
            !media.is_content_type_allowed(banned),
            "{banned} must not be allowed by default"
        );
    }
    assert!(media.is_content_type_allowed("image/png"));
}

#[tokio::test]
async fn oversized_upload_is_rejected_at_initiate() {
    let (adapter, dir) = test_adapter().await;
    let media = AcmeConfig::default().media;

    let too_big = media.max_file_size_bytes_limit() + 1;
    let err = adapter
        .initiate_upload_validated(
            upload_request("a/oversized.png", "image/png", too_big),
            &media,
        )
        .await
        .expect_err("oversized upload must be rejected");
    assert!(matches!(err, BlobError::TooLarge(_, _)));

    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn disallowed_content_type_is_rejected_at_initiate() {
    let (adapter, dir) = test_adapter().await;
    let media = AcmeConfig::default().media;

    let err = adapter
        .initiate_upload_validated(upload_request("a/sneaky.svg", "image/svg+xml", 128), &media)
        .await
        .expect_err("SVG must be rejected by the default allowlist");
    assert!(matches!(err, BlobError::InvalidContentType(_)));

    let _ = std::fs::remove_dir_all(dir);
}
