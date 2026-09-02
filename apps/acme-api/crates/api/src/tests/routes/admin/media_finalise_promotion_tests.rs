//! Spec 003 oracle: live finalise through the handler with failure-capable
//! blob and database seams.

use super::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use acme_auth::{
    AcmeLocalAuthProvider, AcmeLocalAuthService, EmailTotpService, UserId, UserPrincipal, UserRole,
};
use acme_db::media::{self as db_media, MediaVersionRow};
use acme_test_utils::setup_test_db;
use async_trait::async_trait;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use sha2::{Digest, Sha256};
use underlay_auth::AuthProvider;
use underlay_blob::{
    BlobAdapter, BlobError, BlobObjectKey, BlobResult, BlobUploadConfig, DownloadRequest,
    ObjectInfo, SignedUrl, StoredObject, UploadPlan, UploadRequest,
};
use underlay_media::storage::version_object_key;

const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x01];
const FORGED_SHA256: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn skip_without_db() -> bool {
    std::env::var("DATABASE_URL").is_err() && std::env::var("TEST_DATABASE_URL").is_err()
}

fn ensure_test_env() {
    static ONCE: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| {
        let (jwt_cfg, _) =
            underlay_auth_jwt::JwtConfig::generate().expect("should generate test JWT keys");
        std::env::set_var("AUTH_JWT_PRIVATE_KEY", jwt_cfg.private_key_b64());
        std::env::set_var("AUTH_JWT_PUBLIC_KEY", jwt_cfg.public_key_b64());
        std::env::set_var("ENVIRONMENT", "test");
        std::env::set_var("WEBAUTHN_RP_ID", "localhost");
        std::env::set_var("WEBAUTHN_RP_ORIGIN", "http://localhost:3000");
        std::env::set_var("WEBAUTHN_RP_NAME", "Acme Test");
    });
}

fn admin_user() -> AdminUser {
    AdminUser(UserPrincipal {
        user_id: UserId(acme_core::Uuid::new_v7()),
        roles: vec![UserRole::Admin],
        email: Some("admin@example.com".to_string()),
        display_name: Some("Admin".to_string()),
    })
}

#[derive(Default)]
struct FakeAdapter {
    objects: Mutex<HashMap<String, (Vec<u8>, String)>>,
    swap_after_read: Mutex<Option<(String, Vec<u8>)>>,
    unreadable: Mutex<HashSet<String>>,
}

impl FakeAdapter {
    fn seed(&self, key: &str, bytes: impl Into<Vec<u8>>, content_type: &str) {
        self.objects
            .lock()
            .expect("fake adapter objects")
            .insert(key.to_string(), (bytes.into(), content_type.to_string()));
    }

    fn stored(&self, key: &str) -> Option<Vec<u8>> {
        self.objects
            .lock()
            .expect("fake adapter objects")
            .get(key)
            .map(|(bytes, _)| bytes.clone())
    }

    fn mark_unreadable(&self, key: &str) {
        self.unreadable
            .lock()
            .expect("fake adapter unreadable")
            .insert(key.to_string());
    }
}

#[async_trait]
impl BlobAdapter for FakeAdapter {
    async fn initiate_upload(&self, _request: UploadRequest) -> BlobResult<UploadPlan> {
        unimplemented!("finalise tests do not initiate")
    }

    async fn finalise_upload(&self, _key: &str) -> BlobResult<StoredObject> {
        unimplemented!("promote_verified must not call mutable finalise_upload")
    }

    fn public_url(&self, key: &str) -> String {
        key.to_string()
    }

    async fn signed_download_url(&self, _request: DownloadRequest) -> BlobResult<SignedUrl> {
        unimplemented!("finalise tests do not sign downloads")
    }

    async fn delete(&self, _key: &str) -> BlobResult<()> {
        Ok(())
    }

    async fn head(&self, _key: &str) -> BlobResult<ObjectInfo> {
        unimplemented!("finalise tests do not head objects")
    }

    async fn get_bytes(&self, _key: &str) -> BlobResult<Vec<u8>> {
        unimplemented!("promote_verified must not call unbounded get_bytes")
    }

    async fn put_bytes(
        &self,
        _key: &str,
        _data: &[u8],
        _content_type: &str,
    ) -> BlobResult<StoredObject> {
        unimplemented!("promote_verified must not call unconditional put_bytes")
    }

    fn name(&self) -> &'static str {
        "fake"
    }

    fn bucket(&self) -> &str {
        "fake-bucket"
    }

    async fn get_bytes_bounded(&self, key: &str, max_bytes: u64) -> BlobResult<Vec<u8>> {
        if self
            .unreadable
            .lock()
            .expect("fake adapter unreadable")
            .contains(key)
        {
            return Err(BlobError::Unsupported(format!(
                "non-regular or unreadable source: {key}"
            )));
        }

        let bytes = {
            let objects = self.objects.lock().expect("fake adapter objects");
            objects
                .get(key)
                .cloned()
                .map(|(bytes, _)| bytes)
                .ok_or_else(|| BlobError::NotFound(key.to_string()))?
        };

        if bytes.len() as u64 > max_bytes {
            return Err(BlobError::TooLarge(bytes.len() as u64, max_bytes));
        }

        let swap = self
            .swap_after_read
            .lock()
            .expect("fake adapter swap")
            .take();
        if let Some((swap_key, swap_bytes)) = swap {
            if swap_key == key {
                self.objects.lock().expect("fake adapter objects").insert(
                    swap_key,
                    (swap_bytes, "application/octet-stream".to_string()),
                );
            }
        }

        Ok(bytes)
    }

    async fn put_bytes_create_only(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> BlobResult<StoredObject> {
        let mut objects = self.objects.lock().expect("fake adapter objects");
        if objects.contains_key(key) {
            return Err(BlobError::DestinationExists(key.to_string()));
        }
        objects.insert(key.to_string(), (data.to_vec(), content_type.to_string()));
        Ok(StoredObject::new(
            "fake",
            "fake-bucket",
            key,
            data.len() as u64,
            content_type,
        ))
    }
}

struct FailingStore;

#[async_trait]
impl ReadyCurrentStore for FailingStore {
    async fn activate_ready_current(
        &self,
        _media_id: Uuid,
        _version_id: Uuid,
        _promoted: &VerifiedPromotionResult,
    ) -> Result<MediaVersionRow, sqlx::Error> {
        Err(sqlx::Error::Protocol(
            "injected activation failure".to_string(),
        ))
    }
}

async fn build_test_state(pool: sqlx::PgPool, blob: Arc<FakeAdapter>) -> AppState {
    ensure_test_env();

    let local_auth = Arc::new(
        AcmeLocalAuthService::from_env(pool.clone()).expect("should create local auth service"),
    );
    let auth_provider: Arc<dyn AuthProvider> =
        Arc::new(AcmeLocalAuthProvider::new(local_auth.clone()));

    let app_cfg = acme_infra::AppConfig::from_env().expect("should load app config");
    let email_manager = Arc::new(
        acme_infra::create_email_manager(&app_cfg.email).expect("should create email manager"),
    );
    let email_templates = Arc::new(
        acme_infra::create_template_engine(&app_cfg.email)
            .expect("should create email template engine"),
    );
    let email_totp = Arc::new(EmailTotpService::new(
        pool.clone(),
        email_manager.clone(),
        email_templates.clone(),
        app_cfg.email.clone(),
    ));

    AppState {
        local_auth,
        auth_provider,
        cookie_config: underlay_http::AuthCookieConfig::default(),
        email_manager,
        email_templates,
        email_totp,
        email_config: app_cfg.email,
        blob_adapter: blob,
        job_repository: None,
        config: crate::config::AcmeConfig::default(),
    }
}

struct PreparedUpload {
    media_id: Uuid,
    version_id: Uuid,
    staging: BlobObjectKey,
    destination: BlobObjectKey,
}

async fn prepare_uploading(
    pool: &sqlx::PgPool,
    blob: &FakeAdapter,
    bytes: &[u8],
    content_type: &str,
) -> PreparedUpload {
    let media_id = acme_core::Uuid::new_v7().into_inner();
    let version_id = acme_core::Uuid::new_v7().into_inner();
    db_media::create_media(
        pool,
        media_id,
        "image",
        "restricted",
        "Oracle photo",
        Some("photo.png"),
        None,
    )
    .await
    .expect("create media");
    db_media::create_media_version(pool, version_id, media_id, None)
        .await
        .expect("create version");
    let staging = version_object_key(media_id, version_id, "photo.png").expect("staging key");
    let destination = published_object_key(&staging).expect("destination key");
    blob.seed(staging.as_str(), bytes, content_type);
    PreparedUpload {
        media_id,
        version_id,
        staging,
        destination,
    }
}

fn finalise_req() -> FinaliseUploadRequest {
    FinaliseUploadRequest {
        sha256: FORGED_SHA256.to_string(),
        content_type: "image/png".to_string(),
    }
}

async fn call_finalise(
    state: AppState,
    media_id: Uuid,
    version_id: Uuid,
) -> Result<axum::response::Response, ApiError> {
    finalise_upload(
        admin_user(),
        State(state),
        Path((media_id, version_id)),
        Json(finalise_req()),
    )
    .await
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response body should be valid json")
}

fn png_digest() -> String {
    hex::encode(Sha256::digest(PNG))
}

async fn version_row(pool: &sqlx::PgPool, version_id: Uuid) -> MediaVersionRow {
    db_media::get_media_version(pool, version_id)
        .await
        .expect("load version")
        .expect("version exists")
}

async fn current_version_id(pool: &sqlx::PgPool, media_id: Uuid) -> Option<Uuid> {
    db_media::get_media(pool, media_id)
        .await
        .expect("load media")
        .expect("media exists")
        .current_version_id
}

#[tokio::test]
async fn captured_bytes_are_published_when_staging_mutates_after_capture() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }
    let db = setup_test_db().await;
    let pool = db.pool_clone();
    let blob = Arc::new(FakeAdapter::default());
    let prepared = prepare_uploading(&pool, &blob, PNG, "image/png").await;
    blob.swap_after_read.lock().expect("swap").replace((
        prepared.staging.as_str().to_string(),
        b"<html>swapped</html>".to_vec(),
    ));
    let state = build_test_state(pool.clone(), blob.clone()).await;

    let response = call_finalise(state, prepared.media_id, prepared.version_id)
        .await
        .expect("finalise should succeed");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;

    assert_eq!(
        blob.stored(prepared.destination.as_str()).as_deref(),
        Some(PNG)
    );
    assert_eq!(
        blob.stored(prepared.staging.as_str()).as_deref(),
        Some(b"<html>swapped</html>".as_slice())
    );
    assert_eq!(body["version"]["sha256"], png_digest());
    assert_eq!(body["version"]["object_key"], prepared.destination.as_str());
    assert_ne!(body["version"]["sha256"], FORGED_SHA256);
}

#[tokio::test]
async fn oversized_or_unreadable_source_refuses_before_publication() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }
    let db = setup_test_db().await;
    let pool = db.pool_clone();

    let oversized = Arc::new(FakeAdapter::default());
    let prepared = prepare_uploading(&pool, &oversized, PNG, "image/png").await;
    let mut state = build_test_state(pool.clone(), oversized.clone()).await;
    state.config.media = BlobUploadConfig::default().max_file_size_bytes((PNG.len() - 1) as u64);
    let err = call_finalise(state, prepared.media_id, prepared.version_id)
        .await
        .expect_err("oversized source must refuse");
    assert_eq!(err.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(oversized.stored(prepared.destination.as_str()).is_none());
    let version = version_row(&pool, prepared.version_id).await;
    assert_eq!(version.state, "uploading");
    assert_eq!(current_version_id(&pool, prepared.media_id).await, None);

    let unreadable = Arc::new(FakeAdapter::default());
    let prepared = prepare_uploading(&pool, &unreadable, PNG, "image/png").await;
    unreadable.mark_unreadable(prepared.staging.as_str());
    let state = build_test_state(pool.clone(), unreadable.clone()).await;
    let err = call_finalise(state, prepared.media_id, prepared.version_id)
        .await
        .expect_err("unreadable source must refuse");
    assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(unreadable.stored(prepared.destination.as_str()).is_none());
    let version = version_row(&pool, prepared.version_id).await;
    assert_eq!(version.state, "uploading");
    assert_eq!(current_version_id(&pool, prepared.media_id).await, None);
}

#[tokio::test]
async fn occupied_destination_preserves_incumbent_even_with_identical_or_forged_bytes() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }
    let db = setup_test_db().await;
    let pool = db.pool_clone();

    let forged = Arc::new(FakeAdapter::default());
    let prepared = prepare_uploading(&pool, &forged, PNG, "image/png").await;
    forged.seed(
        prepared.destination.as_str(),
        b"incumbent-forged",
        "application/octet-stream",
    );
    let state = build_test_state(pool.clone(), forged.clone()).await;
    let err = call_finalise(state, prepared.media_id, prepared.version_id)
        .await
        .expect_err("forged destination must refuse");
    assert_eq!(err.status, StatusCode::CONFLICT);
    assert_eq!(
        forged.stored(prepared.destination.as_str()).as_deref(),
        Some(b"incumbent-forged".as_slice())
    );
    let version = version_row(&pool, prepared.version_id).await;
    assert_eq!(version.state, "uploading");
    assert_eq!(current_version_id(&pool, prepared.media_id).await, None);

    let identical = Arc::new(FakeAdapter::default());
    let prepared = prepare_uploading(&pool, &identical, PNG, "image/png").await;
    identical.seed(prepared.destination.as_str(), PNG, "image/png");
    let state = build_test_state(pool.clone(), identical.clone()).await;
    let response = call_finalise(state, prepared.media_id, prepared.version_id)
        .await
        .expect("identical destination converges on retry");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        identical.stored(prepared.destination.as_str()).as_deref(),
        Some(PNG)
    );
}

#[tokio::test]
async fn forged_client_metadata_is_ignored_and_persisted_facts_are_server_derived() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }
    let db = setup_test_db().await;
    let pool = db.pool_clone();
    let blob = Arc::new(FakeAdapter::default());
    let prepared = prepare_uploading(&pool, &blob, PNG, "image/png").await;
    let state = build_test_state(pool.clone(), blob.clone()).await;

    let response = call_finalise(state, prepared.media_id, prepared.version_id)
        .await
        .expect("finalise should succeed");
    let body = response_json(response).await;
    let version = &body["version"];
    assert_eq!(version["sha256"], png_digest());
    assert_ne!(version["sha256"], FORGED_SHA256);
    assert_eq!(version["storage_provider"], "fake");
    assert_eq!(version["bucket"], "fake-bucket");
    assert_eq!(version["object_key"], prepared.destination.as_str());
    assert_eq!(version["mime_type"], "image/png");
    assert_eq!(version["byte_size"], PNG.len() as i64);
    assert_eq!(version["state"], "ready");
    assert_eq!(
        body["media"]["current_version_id"],
        prepared.version_id.to_string()
    );
}

#[tokio::test]
async fn activation_failure_keeps_identities_and_retry_does_not_duplicate() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }
    let db = setup_test_db().await;
    let pool = db.pool_clone();
    let blob = Arc::new(FakeAdapter::default());
    let prepared = prepare_uploading(&pool, &blob, PNG, "image/png").await;
    let state = build_test_state(pool.clone(), blob.clone()).await;

    let err = finalise_upload_with(
        &state,
        FailingStore,
        prepared.media_id,
        prepared.version_id,
        finalise_req(),
    )
    .await
    .expect_err("injected activation failure");
    assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        blob.stored(prepared.destination.as_str()).as_deref(),
        Some(PNG)
    );
    assert_eq!(blob.stored(prepared.staging.as_str()).as_deref(), Some(PNG));
    let version = version_row(&pool, prepared.version_id).await;
    assert_eq!(version.state, "uploading");
    assert_eq!(current_version_id(&pool, prepared.media_id).await, None);

    let response = call_finalise(state, prepared.media_id, prepared.version_id)
        .await
        .expect("retry should converge");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["version"]["state"], "ready");
    assert_eq!(
        body["media"]["current_version_id"],
        prepared.version_id.to_string()
    );
    assert_eq!(
        blob.stored(prepared.destination.as_str()).as_deref(),
        Some(PNG)
    );

    let again = call_finalise(
        build_test_state(pool.clone(), blob.clone()).await,
        prepared.media_id,
        prepared.version_id,
    )
    .await
    .expect("idempotent retry of a completed finalise");
    assert_eq!(again.status(), StatusCode::OK);
}

#[tokio::test]
async fn mismatched_declared_mime_refuses_before_publication() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }
    let db = setup_test_db().await;
    let pool = db.pool_clone();
    let blob = Arc::new(FakeAdapter::default());
    let prepared = prepare_uploading(
        &pool,
        &blob,
        b"<html><body>not a png</body></html>",
        "image/png",
    )
    .await;
    let state = build_test_state(pool.clone(), blob.clone()).await;
    let err = call_finalise(state, prepared.media_id, prepared.version_id)
        .await
        .expect_err("mismatched bytes must refuse");
    assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(blob.stored(prepared.destination.as_str()).is_none());
    let version = version_row(&pool, prepared.version_id).await;
    assert_eq!(version.state, "uploading");
    assert_eq!(current_version_id(&pool, prepared.media_id).await, None);
}
