//! Spec 003 oracle: live finalise through the handler with failure-capable
//! blob and database seams.

use super::*;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
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
    ObjectInfo, OwnedPublicationFacts, SignedUrl, StoredObject, UploadPlan, UploadRequest,
};
use underlay_media::storage::version_object_key;

const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x01];
const FORGED_SHA256: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const WRONG_TOKEN: &[u8] = b"other-token-not-the-first-one!!!!";

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

struct StoredBlob {
    bytes: Vec<u8>,
    content_type: String,
    metadata: HashMap<String, String>,
}

struct FakeAdapter {
    objects: Mutex<HashMap<String, StoredBlob>>,
    swap_after_read: Mutex<Option<(String, Vec<u8>)>>,
    unreadable: Mutex<HashSet<String>>,
    fail_delete: Mutex<HashSet<String>>,
    bounded_reads: AtomicUsize,
    name: &'static str,
    bucket: String,
}

impl Default for FakeAdapter {
    fn default() -> Self {
        Self {
            objects: Mutex::new(HashMap::new()),
            swap_after_read: Mutex::new(None),
            unreadable: Mutex::new(HashSet::new()),
            fail_delete: Mutex::new(HashSet::new()),
            bounded_reads: AtomicUsize::new(0),
            name: "fake",
            bucket: "fake-bucket".to_string(),
        }
    }
}

impl FakeAdapter {
    fn seed(&self, key: &str, bytes: impl Into<Vec<u8>>, content_type: &str) {
        self.objects.lock().expect("fake adapter objects").insert(
            key.to_string(),
            StoredBlob {
                bytes: bytes.into(),
                content_type: content_type.to_string(),
                metadata: HashMap::new(),
            },
        );
    }

    fn stored(&self, key: &str) -> Option<Vec<u8>> {
        self.objects
            .lock()
            .expect("fake adapter objects")
            .get(key)
            .map(|object| object.bytes.clone())
    }

    fn stored_metadata(&self, key: &str) -> Option<HashMap<String, String>> {
        self.objects
            .lock()
            .expect("fake adapter objects")
            .get(key)
            .map(|object| object.metadata.clone())
    }

    fn mark_unreadable(&self, key: &str) {
        self.unreadable
            .lock()
            .expect("fake adapter unreadable")
            .insert(key.to_string());
    }

    fn fail_deletes(&self, key: &str) {
        self.fail_delete
            .lock()
            .expect("fake adapter fail delete")
            .insert(key.to_string());
    }

    fn clear_fail_deletes(&self) {
        self.fail_delete
            .lock()
            .expect("fake adapter fail delete")
            .clear();
    }

    fn remove(&self, key: &str) {
        self.objects
            .lock()
            .expect("fake adapter objects")
            .remove(key);
    }

    fn stored_object(&self, key: &str, data: &[u8], content_type: &str) -> StoredObject {
        StoredObject::new(
            self.name,
            &self.bucket,
            key,
            data.len() as u64,
            content_type,
        )
    }
}

#[async_trait]
impl BlobAdapter for FakeAdapter {
    async fn initiate_upload(&self, _request: UploadRequest) -> BlobResult<UploadPlan> {
        unimplemented!("finalise tests do not initiate")
    }

    async fn finalise_upload(&self, _key: &str) -> BlobResult<StoredObject> {
        unimplemented!("promote_verified_owned must not call mutable finalise_upload")
    }

    fn public_url(&self, key: &str) -> String {
        key.to_string()
    }

    async fn signed_download_url(&self, _request: DownloadRequest) -> BlobResult<SignedUrl> {
        unimplemented!("finalise tests do not sign downloads")
    }

    async fn delete(&self, key: &str) -> BlobResult<()> {
        if self
            .fail_delete
            .lock()
            .expect("fake adapter fail delete")
            .contains(key)
        {
            return Err(BlobError::Internal(format!(
                "injected delete failure for {key}"
            )));
        }
        self.objects
            .lock()
            .expect("fake adapter objects")
            .remove(key);
        Ok(())
    }

    async fn head(&self, key: &str) -> BlobResult<ObjectInfo> {
        let objects = self.objects.lock().expect("fake adapter objects");
        let object = objects
            .get(key)
            .ok_or_else(|| BlobError::NotFound(key.to_string()))?;
        Ok(ObjectInfo {
            key: key.to_string(),
            size: object.bytes.len() as u64,
            content_type: object.content_type.clone(),
            etag: None,
            last_modified: None,
            metadata: object.metadata.clone(),
        })
    }

    async fn get_bytes(&self, _key: &str) -> BlobResult<Vec<u8>> {
        unimplemented!("owned recovery must not call unbounded get_bytes")
    }

    async fn put_bytes(
        &self,
        _key: &str,
        _data: &[u8],
        _content_type: &str,
    ) -> BlobResult<StoredObject> {
        unimplemented!("promote_verified_owned must not call unconditional put_bytes")
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn bucket(&self) -> &str {
        &self.bucket
    }

    async fn get_bytes_bounded(&self, key: &str, max_bytes: u64) -> BlobResult<Vec<u8>> {
        self.bounded_reads.fetch_add(1, Ordering::SeqCst);
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
                .map(|object| object.bytes.clone())
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
                    StoredBlob {
                        bytes: swap_bytes,
                        content_type: "application/octet-stream".to_string(),
                        metadata: HashMap::new(),
                    },
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
        objects.insert(
            key.to_string(),
            StoredBlob {
                bytes: data.to_vec(),
                content_type: content_type.to_string(),
                metadata: HashMap::new(),
            },
        );
        Ok(self.stored_object(key, data, content_type))
    }

    async fn put_bytes_create_only_owned(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
        facts: &OwnedPublicationFacts,
    ) -> BlobResult<StoredObject> {
        let mut objects = self.objects.lock().expect("fake adapter objects");
        if objects.contains_key(key) {
            return Err(BlobError::DestinationExists(key.to_string()));
        }
        objects.insert(
            key.to_string(),
            StoredBlob {
                bytes: data.to_vec(),
                content_type: content_type.to_string(),
                metadata: facts
                    .metadata_pairs()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
            },
        );
        Ok(self.stored_object(key, data, content_type))
    }
}

struct FailAfterVersionReadyStore<'a>(&'a sqlx::PgPool);

#[async_trait]
impl ReadyCurrentStore for FailAfterVersionReadyStore<'_> {
    async fn activate_ready_current(
        &self,
        media_id: Uuid,
        version_id: Uuid,
        promoted: &VerifiedPromotionResult,
    ) -> Result<MediaVersionRow, sqlx::Error> {
        db_media::activate_ready_current_failing_after_version_ready(
            self.0,
            version_id,
            media_id,
            promoted.object.size as i64,
            &promoted.object.content_type,
            &promoted.sha256,
            &promoted.object.provider,
            &promoted.object.bucket,
            &promoted.object.key,
        )
        .await
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
    let staging = version_object_key(media_id, version_id, "photo.png").expect("staging key");
    db_media::create_media_version(
        pool,
        version_id,
        media_id,
        None,
        staging.as_str(),
        content_type,
    )
    .await
    .expect("create version");
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

async fn call_finalise_fault(
    state: &AppState,
    media_id: Uuid,
    version_id: Uuid,
    fault: FinaliseFault,
) -> Result<axum::response::Response, ApiError> {
    finalise_upload_with(
        state,
        PoolStore(state.local_auth.pool()),
        media_id,
        version_id,
        finalise_req(),
        fault,
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

fn assert_token_redacted(version: &MediaVersionRow) {
    let token = version
        .ownership_token
        .as_ref()
        .expect("ownership token must be persisted");
    let rendered = format!("{version:?}");
    assert!(rendered.contains("redacted"));
    assert!(!rendered.contains(&hex::encode(token)));
    assert!(token.len() >= 32);
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
    assert!(body["version"].get("ownership_token").is_none());
    assert!(body["version"].get("published_object_key").is_none());
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
    let err = call_finalise(state, prepared.media_id, prepared.version_id)
        .await
        .expect_err("identical destination must refuse");
    assert_eq!(err.status, StatusCode::CONFLICT);
    assert_eq!(
        identical.stored(prepared.destination.as_str()).as_deref(),
        Some(PNG)
    );
    let version = version_row(&pool, prepared.version_id).await;
    assert_eq!(version.state, "uploading");
    assert_eq!(current_version_id(&pool, prepared.media_id).await, None);
}

#[tokio::test]
async fn occupied_destination_refuses_after_staging_mutates_post_capture() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }
    let db = setup_test_db().await;
    let pool = db.pool_clone();
    let blob = Arc::new(FakeAdapter::default());
    let prepared = prepare_uploading(&pool, &blob, PNG, "image/png").await;
    blob.seed(
        prepared.destination.as_str(),
        b"<html>incumbent</html>",
        "text/html",
    );
    blob.swap_after_read.lock().expect("swap").replace((
        prepared.staging.as_str().to_string(),
        b"<html>incumbent</html>".to_vec(),
    ));
    let state = build_test_state(pool.clone(), blob.clone()).await;
    let err = call_finalise(state, prepared.media_id, prepared.version_id)
        .await
        .expect_err("collision must refuse even if staging later matches incumbent");
    assert_eq!(err.status, StatusCode::CONFLICT);
    assert_eq!(
        blob.stored(prepared.destination.as_str()).as_deref(),
        Some(b"<html>incumbent</html>".as_slice())
    );
    let version = version_row(&pool, prepared.version_id).await;
    assert_eq!(version.state, "uploading");
    assert_eq!(version.sha256, None);
    assert_eq!(current_version_id(&pool, prepared.media_id).await, None);
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
    assert!(version.get("ownership_token").is_none());
    let row = version_row(&pool, prepared.version_id).await;
    assert_token_redacted(&row);
    assert_eq!(
        row.published_object_key.as_ref().map(|key| key.as_str()),
        Some(prepared.destination.as_str())
    );
    assert_eq!(
        blob.stored(prepared.destination.as_str()).as_deref(),
        Some(PNG)
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

    let media_id = prepared.media_id;
    let version_id = prepared.version_id;
    let staging = prepared.staging.clone();
    let destination = prepared.destination.clone();
    drop(prepared);

    let err = finalise_upload_with(
        &state,
        FailAfterVersionReadyStore(&pool),
        media_id,
        version_id,
        finalise_req(),
        FinaliseFault::None,
    )
    .await
    .expect_err("injected in-transaction activation failure");
    assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(blob.stored(destination.as_str()).as_deref(), Some(PNG));
    assert_eq!(blob.stored(staging.as_str()).as_deref(), Some(PNG));

    let version = version_row(&pool, version_id).await;
    assert_eq!(version.state, "uploading");
    assert_eq!(version.sha256.as_deref(), Some(png_digest().as_str()));
    assert_eq!(
        version.object_key.as_ref().map(|key| key.as_str()),
        Some(staging.as_str())
    );
    assert_eq!(
        version
            .published_object_key
            .as_ref()
            .map(|key| key.as_str()),
        Some(destination.as_str())
    );
    assert_token_redacted(&version);
    assert_eq!(current_version_id(&pool, media_id).await, None);

    db_media::update_media(
        &pool,
        media_id,
        "Oracle photo",
        Some("renamed.png"),
        "restricted",
        None,
    )
    .await
    .expect("rename media so retry cannot recompute keys from filename");

    let response = call_finalise(
        build_test_state(pool.clone(), blob.clone()).await,
        media_id,
        version_id,
    )
    .await
    .expect("retry should activate from persisted staging identity");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["version"]["state"], "ready");
    assert_eq!(body["version"]["object_key"], destination.as_str());
    assert_eq!(body["media"]["current_version_id"], version_id.to_string());
    assert_eq!(blob.stored(destination.as_str()).as_deref(), Some(PNG));

    let again = call_finalise(
        build_test_state(pool.clone(), blob.clone()).await,
        media_id,
        version_id,
    )
    .await
    .expect("idempotent retry of a completed finalise");
    assert_eq!(again.status(), StatusCode::OK);
}

#[tokio::test]
async fn pre_create_crash_plus_foreign_incumbent_refuses() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }
    let db = setup_test_db().await;
    let pool = db.pool_clone();
    let blob = Arc::new(FakeAdapter::default());
    let prepared = prepare_uploading(&pool, &blob, PNG, "image/png").await;
    let state = build_test_state(pool.clone(), blob.clone()).await;

    let err = call_finalise_fault(
        &state,
        prepared.media_id,
        prepared.version_id,
        FinaliseFault::BeforeCreate,
    )
    .await
    .expect_err("injected crash after persisting owned authority");
    assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(blob.stored(prepared.destination.as_str()).is_none());

    let version = version_row(&pool, prepared.version_id).await;
    assert_eq!(version.state, "uploading");
    assert_eq!(version.sha256, None);
    assert_token_redacted(&version);
    assert_eq!(
        version
            .published_object_key
            .as_ref()
            .map(|key| key.as_str()),
        Some(prepared.destination.as_str())
    );

    blob.seed(prepared.destination.as_str(), PNG, "image/png");
    let err = call_finalise(
        build_test_state(pool.clone(), blob.clone()).await,
        prepared.media_id,
        prepared.version_id,
    )
    .await
    .expect_err("foreign incumbent must not be adopted from persisted token");
    assert_eq!(err.status, StatusCode::CONFLICT);
    assert_eq!(
        blob.stored(prepared.destination.as_str()).as_deref(),
        Some(PNG)
    );
    assert!(blob
        .stored_metadata(prepared.destination.as_str())
        .expect("foreign metadata")
        .is_empty());
    let version = version_row(&pool, prepared.version_id).await;
    assert_eq!(version.state, "uploading");
    assert_eq!(version.sha256, None);
    assert_eq!(current_version_id(&pool, prepared.media_id).await, None);
}

#[tokio::test]
async fn post_owned_create_crash_recovers_without_staging() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }
    let db = setup_test_db().await;
    let pool = db.pool_clone();
    let blob = Arc::new(FakeAdapter::default());

    let deleted = prepare_uploading(&pool, &blob, PNG, "image/png").await;
    let mutated = prepare_uploading(&pool, &blob, PNG, "image/png").await;
    let hostile = prepare_uploading(&pool, &blob, PNG, "image/png").await;
    let state = build_test_state(pool.clone(), blob.clone()).await;

    for prepared in [&deleted, &mutated, &hostile] {
        let err = call_finalise_fault(
            &state,
            prepared.media_id,
            prepared.version_id,
            FinaliseFault::AfterCreate,
        )
        .await
        .expect_err("injected crash after exclusive owned create");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        let version = version_row(&pool, prepared.version_id).await;
        assert_eq!(version.state, "uploading");
        assert_eq!(version.sha256, None);
        assert_token_redacted(&version);
        assert!(blob
            .stored_metadata(prepared.destination.as_str())
            .expect("owned metadata")
            .contains_key("underlay-owned-v1-verifier"));
    }

    blob.remove(deleted.staging.as_str());
    blob.mark_unreadable(deleted.destination.as_str());
    blob.seed(
        mutated.staging.as_str(),
        b"<html>mutated-staging</html>",
        "text/html",
    );
    blob.seed(
        hostile.staging.as_str(),
        b"<html>hostile-staging</html>",
        "text/html",
    );

    let reads_before = blob.bounded_reads.load(Ordering::SeqCst);
    for prepared in [&deleted, &mutated, &hostile] {
        db_media::update_media(
            &pool,
            prepared.media_id,
            "Oracle photo",
            Some("renamed.png"),
            "restricted",
            None,
        )
        .await
        .expect("rename media");
        let recovered = call_finalise(
            build_test_state(pool.clone(), blob.clone()).await,
            prepared.media_id,
            prepared.version_id,
        )
        .await
        .expect("retry must recover from owned destination without staging");
        assert_eq!(recovered.status(), StatusCode::OK);
        let body = response_json(recovered).await;
        assert_eq!(body["version"]["state"], "ready");
        assert_eq!(body["version"]["sha256"], png_digest());
        assert_eq!(body["version"]["object_key"], prepared.destination.as_str());
        assert_eq!(
            body["media"]["current_version_id"],
            prepared.version_id.to_string()
        );
        assert_eq!(
            blob.stored(prepared.destination.as_str()).as_deref(),
            Some(PNG)
        );
        assert!(body["version"].get("ownership_token").is_none());
    }
    assert_eq!(
        blob.bounded_reads.load(Ordering::SeqCst),
        reads_before,
        "owned recovery must not reread staging or destination bytes"
    );
}

#[tokio::test]
async fn wrong_token_provider_bucket_or_destination_refuses_without_mutation() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }
    let db = setup_test_db().await;
    let pool = db.pool_clone();
    let blob = Arc::new(FakeAdapter::default());
    let prepared = prepare_uploading(&pool, &blob, PNG, "image/png").await;
    let state = build_test_state(pool.clone(), blob.clone()).await;
    call_finalise_fault(
        &state,
        prepared.media_id,
        prepared.version_id,
        FinaliseFault::AfterCreate,
    )
    .await
    .expect_err("owned create crash window");

    let original_meta = blob
        .stored_metadata(prepared.destination.as_str())
        .expect("owned metadata");
    let original_token = version_row(&pool, prepared.version_id)
        .await
        .ownership_token
        .expect("token");

    sqlx::query("UPDATE media.media_version SET ownership_token = $2 WHERE id = $1")
        .bind(prepared.version_id)
        .bind(WRONG_TOKEN)
        .execute(&pool)
        .await
        .expect("tamper token");
    let err = call_finalise(
        build_test_state(pool.clone(), blob.clone()).await,
        prepared.media_id,
        prepared.version_id,
    )
    .await
    .expect_err("wrong token must refuse");
    assert_eq!(err.status, StatusCode::CONFLICT);
    let rendered = format!("{err:?}");
    assert!(!rendered.contains(&hex::encode(WRONG_TOKEN)));
    assert!(!rendered.contains(std::str::from_utf8(WRONG_TOKEN).unwrap()));

    sqlx::query(
        "UPDATE media.media_version SET ownership_token = $2, storage_provider = $3 WHERE id = $1",
    )
    .bind(prepared.version_id)
    .bind(&original_token)
    .bind("s3")
    .execute(&pool)
    .await
    .expect("tamper provider");
    let err = call_finalise(
        build_test_state(pool.clone(), blob.clone()).await,
        prepared.media_id,
        prepared.version_id,
    )
    .await
    .expect_err("wrong provider must refuse");
    assert_eq!(err.status, StatusCode::BAD_REQUEST);

    sqlx::query("UPDATE media.media_version SET storage_provider = $2, bucket = $3 WHERE id = $1")
        .bind(prepared.version_id)
        .bind("fake")
        .bind("other-bucket")
        .execute(&pool)
        .await
        .expect("tamper bucket");
    let err = call_finalise(
        build_test_state(pool.clone(), blob.clone()).await,
        prepared.media_id,
        prepared.version_id,
    )
    .await
    .expect_err("wrong bucket must refuse");
    assert_eq!(err.status, StatusCode::BAD_REQUEST);

    let foreign = BlobObjectKey::parse("media/foreign/published/photo.png").expect("foreign key");
    blob.seed(foreign.as_str(), PNG, "image/png");
    sqlx::query(
        "UPDATE media.media_version SET bucket = $2, published_object_key = $3 WHERE id = $1",
    )
    .bind(prepared.version_id)
    .bind("fake-bucket")
    .bind(foreign.as_str())
    .execute(&pool)
    .await
    .expect("tamper destination");
    let err = call_finalise(
        build_test_state(pool.clone(), blob.clone()).await,
        prepared.media_id,
        prepared.version_id,
    )
    .await
    .expect_err("wrong destination must refuse");
    assert_eq!(err.status, StatusCode::CONFLICT);

    assert_eq!(
        blob.stored(prepared.destination.as_str()).as_deref(),
        Some(PNG)
    );
    assert_eq!(
        blob.stored_metadata(prepared.destination.as_str())
            .expect("owned metadata"),
        original_meta
    );
    assert_eq!(blob.stored(foreign.as_str()).as_deref(), Some(PNG));
    let version = version_row(&pool, prepared.version_id).await;
    assert_eq!(version.state, "uploading");
    assert_eq!(version.sha256, None);
    assert_eq!(current_version_id(&pool, prepared.media_id).await, None);
}

#[tokio::test]
async fn delete_and_purge_blob_failure_retains_row_and_retry_converges() {
    if skip_without_db() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    }
    let db = setup_test_db().await;
    let pool = db.pool_clone();
    let blob = Arc::new(FakeAdapter::default());
    let delete_case = prepare_uploading(&pool, &blob, PNG, "image/png").await;
    let purge_case = prepare_uploading(&pool, &blob, PNG, "image/png").await;
    let state = build_test_state(pool.clone(), blob.clone()).await;

    call_finalise_fault(
        &state,
        delete_case.media_id,
        delete_case.version_id,
        FinaliseFault::AfterCreate,
    )
    .await
    .expect_err("crash window for delete");
    let purged = call_finalise(
        build_test_state(pool.clone(), blob.clone()).await,
        purge_case.media_id,
        purge_case.version_id,
    )
    .await
    .expect("purge case completes");
    assert_eq!(purged.status(), StatusCode::OK);

    blob.fail_deletes(delete_case.destination.as_str());
    blob.fail_deletes(purge_case.destination.as_str());

    let err = super::super::delete_version(
        admin_user(),
        State(build_test_state(pool.clone(), blob.clone()).await),
        Path((delete_case.media_id, delete_case.version_id)),
    )
    .await
    .expect_err("delete must fail when blob cleanup fails");
    assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
    let retained = version_row(&pool, delete_case.version_id).await;
    assert_token_redacted(&retained);
    assert_eq!(
        retained
            .published_object_key
            .as_ref()
            .map(|key| key.as_str()),
        Some(delete_case.destination.as_str())
    );
    assert_eq!(
        blob.stored(delete_case.destination.as_str()).as_deref(),
        Some(PNG)
    );

    let err = super::super::purge_media(
        admin_user(),
        State(build_test_state(pool.clone(), blob.clone()).await),
        Path(purge_case.media_id),
    )
    .await
    .expect_err("purge must fail when blob cleanup fails");
    assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(db_media::get_media(&pool, purge_case.media_id)
        .await
        .expect("load purged media")
        .is_some());
    let purge_row = version_row(&pool, purge_case.version_id).await;
    assert_token_redacted(&purge_row);
    assert_eq!(
        blob.stored(purge_case.destination.as_str()).as_deref(),
        Some(PNG)
    );

    blob.clear_fail_deletes();
    let deleted = super::super::delete_version(
        admin_user(),
        State(build_test_state(pool.clone(), blob.clone()).await),
        Path((delete_case.media_id, delete_case.version_id)),
    )
    .await
    .expect("delete retry after blob recovery");
    assert_eq!(deleted.status(), StatusCode::OK);
    assert!(blob.stored(delete_case.destination.as_str()).is_none());
    assert!(blob.stored(delete_case.staging.as_str()).is_none());
    assert!(db_media::get_media_version(&pool, delete_case.version_id)
        .await
        .expect("load deleted version")
        .is_none());

    let purged = super::super::purge_media(
        admin_user(),
        State(build_test_state(pool.clone(), blob.clone()).await),
        Path(purge_case.media_id),
    )
    .await
    .expect("purge retry after blob recovery");
    assert_eq!(purged.status(), StatusCode::OK);
    assert!(blob.stored(purge_case.destination.as_str()).is_none());
    assert!(blob.stored(purge_case.staging.as_str()).is_none());
    assert!(db_media::get_media(&pool, purge_case.media_id)
        .await
        .expect("load purged media")
        .is_none());
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
