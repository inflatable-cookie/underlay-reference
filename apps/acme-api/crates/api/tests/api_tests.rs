//! API Integration Tests
//!
//! These tests demonstrate patterns for testing HTTP endpoints.
//!
//! # Test Setup
//!
//! Due to the complexity of the full application state (auth providers,
//! email services, blob storage), these tests focus on patterns that
//! consuming applications can adapt.
//!
//! For full E2E testing, see the manual test instructions in README.md.

mod health_tests {
    //! Health endpoint tests - simplest integration test example.

    use axum::{routing::get, Router};
    use underlay_testing::TestServer;

    async fn health() -> &'static str {
        "OK"
    }

    fn test_server() -> TestServer {
        TestServer::new(Router::new().route("/v1/health", get(health)))
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let response = test_server().get("/v1/health").send().await;
        response.assert_ok();
        assert_eq!(response.text(), "OK");
    }

    #[tokio::test]
    async fn unknown_route_returns_not_found() {
        let response = test_server().get("/v1/unknown").send().await;
        response.assert_not_found();
    }
}

mod api_version_tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::util::ServiceExt;

    async fn ping() -> &'static str {
        "pong"
    }

    /// The version vocabulary comes from typed config, resolved once at
    /// bootstrap. Tests build the same state from the committed defaults
    /// rather than setting env vars.
    fn test_router() -> Router {
        let versions = acme_api::routes::ApiVersionState::from_behavior(
            &acme_infra::AppBehaviorConfig::default().api,
        );

        Router::new()
            .route("/v1/ping", get(ping))
            .layer(axum::middleware::from_fn_with_state(
                versions,
                acme_api::routes::api_version_middleware,
            ))
    }

    #[tokio::test]
    async fn accepts_default_api_version_when_header_missing() {
        let app = test_router();

        let request = Request::builder()
            .uri("/v1/ping")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("x-api-version")
                .and_then(|v| v.to_str().ok()),
            Some("2025-01-01")
        );
    }

    #[tokio::test]
    async fn rejects_unsupported_api_version() {
        let app = test_router();

        let request = Request::builder()
            .uri("/v1/ping")
            .header("x-api-version", "1900-01-01")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_text.contains("api.unsupported_version"));
    }
}

mod auth_boundary_router_tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::util::ServiceExt;
    use underlay_auth::{AuthError, AuthProvider, HasAuthProvider, Principal, RoleSet};

    use acme_api::state::AdminUser;
    use acme_core::Uuid;

    #[derive(Clone)]
    struct MockAuthProvider;

    #[async_trait]
    impl AuthProvider for MockAuthProvider {
        async fn authenticate_bearer(
            &self,
            bearer_token: &str,
        ) -> underlay_auth::AuthResult<Principal> {
            let principal = match bearer_token {
                "admin-token" => Principal {
                    user_id: Uuid::new_v7(),
                    roles: RoleSet::new(["admin"]),
                },
                "user-token" => Principal {
                    user_id: Uuid::new_v7(),
                    roles: RoleSet::new(["user"]),
                },
                _ => return Err(AuthError::TokenInvalid),
            };

            Ok(principal)
        }
    }

    #[derive(Clone)]
    struct TestState {
        auth_provider: Arc<dyn AuthProvider>,
    }

    impl HasAuthProvider for TestState {
        fn auth_provider(&self) -> &dyn AuthProvider {
            self.auth_provider.as_ref()
        }
    }

    async fn admin_route(_admin: AdminUser) -> &'static str {
        "ok"
    }

    fn test_router() -> Router {
        let state = TestState {
            auth_provider: Arc::new(MockAuthProvider),
        };

        Router::new()
            .route("/v1/admin-only", get(admin_route))
            .with_state(state)
    }

    #[tokio::test]
    async fn admin_route_rejects_missing_bearer_token() {
        let app = test_router();

        let request = Request::builder()
            .uri("/v1/admin-only")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn admin_route_rejects_non_admin_principal() {
        let app = test_router();

        let request = Request::builder()
            .uri("/v1/admin-only")
            .header(header::AUTHORIZATION, "Bearer user-token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn admin_route_accepts_admin_principal() {
        let app = test_router();

        let request = Request::builder()
            .uri("/v1/admin-only")
            .header(header::AUTHORIZATION, "Bearer admin-token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}

mod json_response_tests {
    //! Tests demonstrating JSON response handling patterns.

    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        routing::get,
        Json, Router,
    };
    use serde::{Deserialize, Serialize};
    use tower::util::ServiceExt;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct HealthResponse {
        status: String,
        version: String,
    }

    async fn health_json() -> Json<HealthResponse> {
        Json(HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
        })
    }

    fn test_router() -> Router {
        Router::new().route("/v1/health", get(health_json))
    }

    #[tokio::test]
    async fn health_returns_json() {
        let app = test_router();

        let request = Request::builder()
            .uri("/v1/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        // Read and parse body
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, "1.0.0");
    }
}

mod request_validation_tests {
    //! Tests demonstrating request validation patterns.

    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        routing::post,
        Json, Router,
    };
    use serde::{Deserialize, Serialize};
    use tower::util::ServiceExt;

    #[derive(Debug, Deserialize)]
    struct CreateProjectRequest {
        name: String,
        #[allow(dead_code)]
        description: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct CreateProjectResponse {
        id: String,
        name: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ErrorResponse {
        ok: bool,
        error: ErrorDetail,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ErrorDetail {
        code: String,
        message: String,
    }

    async fn create_project(
        Json(payload): Json<CreateProjectRequest>,
    ) -> Result<Json<CreateProjectResponse>, (StatusCode, Json<ErrorResponse>)> {
        // Validation
        if payload.name.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    ok: false,
                    error: ErrorDetail {
                        code: "validation.name_required".to_string(),
                        message: "Name is required".to_string(),
                    },
                }),
            ));
        }

        if payload.name.len() > 100 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    ok: false,
                    error: ErrorDetail {
                        code: "validation.name_too_long".to_string(),
                        message: "Name must be 100 characters or less".to_string(),
                    },
                }),
            ));
        }

        Ok(Json(CreateProjectResponse {
            id: "test-id".to_string(),
            name: payload.name,
        }))
    }

    fn test_router() -> Router {
        Router::new().route("/v1/projects", post(create_project))
    }

    #[tokio::test]
    async fn create_project_with_valid_data() {
        let app = test_router();

        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/projects")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"name": "My Project"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let project: CreateProjectResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(project.name, "My Project");
    }

    #[tokio::test]
    async fn create_project_rejects_empty_name() {
        let app = test_router();

        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/projects")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"name": ""}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert!(!error.ok);
        assert_eq!(error.error.code, "validation.name_required");
    }

    #[tokio::test]
    async fn create_project_rejects_long_name() {
        let app = test_router();
        let long_name = "x".repeat(101);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/projects")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(format!(r#"{{"name": "{}"}}"#, long_name)))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(error.error.code, "validation.name_too_long");
    }

    #[tokio::test]
    async fn create_project_rejects_invalid_json() {
        let app = test_router();

        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/projects")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"name": }"#)) // Invalid JSON
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Axum returns 400 Bad Request for JSON parse errors
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

mod state_extraction_tests {
    //! Tests demonstrating state extraction patterns.

    use axum::{
        body::Body,
        extract::State,
        http::{Request, StatusCode},
        routing::get,
        Json, Router,
    };
    use std::sync::Arc;
    use tower::util::ServiceExt;

    // Example application state
    #[derive(Clone)]
    struct AppState {
        app_name: String,
        version: String,
    }

    async fn get_info(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "app": state.app_name,
            "version": state.version
        }))
    }

    fn test_router(state: Arc<AppState>) -> Router {
        Router::new()
            .route("/v1/info", get(get_info))
            .with_state(state)
    }

    #[tokio::test]
    async fn handler_accesses_state() {
        let state = Arc::new(AppState {
            app_name: "Acme API".to_string(),
            version: "1.0.0".to_string(),
        });
        let app = test_router(state);

        let request = Request::builder()
            .uri("/v1/info")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let info: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(info["app"], "Acme API");
        assert_eq!(info["version"], "1.0.0");
    }
}

mod pagination_tests {
    //! Tests demonstrating pagination patterns.

    use axum::{
        body::Body,
        extract::Query,
        http::{Request, StatusCode},
        routing::get,
        Json, Router,
    };
    use serde::{Deserialize, Serialize};
    use tower::util::ServiceExt;

    #[derive(Debug, Deserialize)]
    struct ListParams {
        limit: Option<i64>,
        offset: Option<i64>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ListResponse<T> {
        data: Vec<T>,
        has_more: bool,
        total: i64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Item {
        id: i64,
        name: String,
    }

    async fn list_items(Query(params): Query<ListParams>) -> Json<ListResponse<Item>> {
        let limit = params.limit.unwrap_or(10).min(100);
        let offset = params.offset.unwrap_or(0);

        // Simulate 25 total items
        let total = 25i64;
        let all_items: Vec<Item> = (1..=25)
            .map(|i| Item {
                id: i,
                name: format!("Item {}", i),
            })
            .collect();

        let data: Vec<Item> = all_items
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();

        let has_more = offset + (data.len() as i64) < total;

        Json(ListResponse {
            data,
            has_more,
            total,
        })
    }

    fn test_router() -> Router {
        Router::new().route("/v1/items", get(list_items))
    }

    #[tokio::test]
    async fn list_with_defaults() {
        let app = test_router();

        let request = Request::builder()
            .uri("/v1/items")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let list: ListResponse<Item> = serde_json::from_slice(&body).unwrap();

        assert_eq!(list.data.len(), 10); // Default limit
        assert!(list.has_more);
        assert_eq!(list.total, 25);
    }

    #[tokio::test]
    async fn list_with_pagination() {
        let app = test_router();

        let request = Request::builder()
            .uri("/v1/items?limit=5&offset=20")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let list: ListResponse<Item> = serde_json::from_slice(&body).unwrap();

        assert_eq!(list.data.len(), 5);
        assert!(!list.has_more); // Last page
        assert_eq!(list.data[0].id, 21);
    }
}

// ============================================================================
// Database Integration Tests
// ============================================================================

mod database_tests {
    //! Database integration tests - require DATABASE_URL to be set.
    //!
    //! These tests demonstrate patterns for testing database operations.
    //! They use the test-utils crate for fixtures and cleanup.

    use std::env;

    fn skip_without_db() -> bool {
        env::var("DATABASE_URL").is_err() && env::var("TEST_DATABASE_URL").is_err()
    }

    #[tokio::test]
    async fn test_fixture_creation_pattern() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        // Import test utilities
        use acme_test_utils::{
            cleanup,
            fixtures::{create_test_project, create_test_task, create_test_user},
            setup_test_db,
        };

        let db = setup_test_db().await;

        // Create test data using fixtures
        let user = create_test_user(db.pool(), Default::default()).await;
        let project = create_test_project(db.pool(), user.id, Default::default()).await;
        let task = create_test_task(db.pool(), project.id, Default::default()).await;

        // Verify relationships
        assert_eq!(project.owner_id, user.id);
        assert_eq!(task.project_id, project.id);

        // Clean up (in reverse order of creation)
        cleanup::delete_user(db.pool(), user.id)
            .await
            .expect("cleanup should succeed");
    }

    #[tokio::test]
    async fn test_query_pattern() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        use acme_test_utils::{
            cleanup,
            fixtures::{
                create_test_project, create_test_task, create_test_user, CreateTaskOptions,
            },
            setup_test_db,
        };

        let db = setup_test_db().await;

        // Create user and project
        let user = create_test_user(db.pool(), Default::default()).await;
        let project = create_test_project(db.pool(), user.id, Default::default()).await;

        // Create tasks with specific statuses
        let _pending = create_test_task(
            db.pool(),
            project.id,
            CreateTaskOptions {
                status: Some("pending".to_string()),
                ..Default::default()
            },
        )
        .await;

        let _completed = create_test_task(
            db.pool(),
            project.id,
            CreateTaskOptions {
                status: Some("completed".to_string()),
                ..Default::default()
            },
        )
        .await;

        // Query to count tasks by status
        let pending_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM acme.tasks WHERE project_id = $1 AND status = 'pending'",
        )
        .bind(project.id)
        .fetch_one(db.pool())
        .await
        .expect("query should succeed");

        let completed_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM acme.tasks WHERE project_id = $1 AND status = 'completed'",
        )
        .bind(project.id)
        .fetch_one(db.pool())
        .await
        .expect("query should succeed");

        assert_eq!(pending_count.0, 1);
        assert_eq!(completed_count.0, 1);

        // Cleanup
        cleanup::delete_user(db.pool(), user.id)
            .await
            .expect("cleanup should succeed");
    }

    #[tokio::test]
    async fn task_update_requires_matching_project_scope() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        use acme_db::tasks;
        use acme_test_utils::{
            cleanup,
            fixtures::{create_test_project, create_test_task, create_test_user},
            setup_test_db,
        };

        let db = setup_test_db().await;

        let user = create_test_user(db.pool(), Default::default()).await;
        let project_a = create_test_project(db.pool(), user.id, Default::default()).await;
        let project_b = create_test_project(db.pool(), user.id, Default::default()).await;
        let task = create_test_task(db.pool(), project_a.id, Default::default()).await;

        let updated = tasks::update_task(
            db.pool(),
            task.id,
            project_b.id,
            Some("unauthorized update"),
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("update query should succeed");

        assert!(updated.is_none());

        let title: (String,) = sqlx::query_as("SELECT title FROM acme.tasks WHERE id = $1")
            .bind(task.id)
            .fetch_one(db.pool())
            .await
            .expect("task should still exist");

        assert_ne!(title.0, "unauthorized update");

        cleanup::delete_user(db.pool(), user.id)
            .await
            .expect("cleanup should succeed");
    }

    #[tokio::test]
    async fn task_delete_requires_matching_project_scope() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        use acme_db::tasks;
        use acme_test_utils::{
            cleanup,
            fixtures::{create_test_project, create_test_task, create_test_user},
            setup_test_db,
        };

        let db = setup_test_db().await;

        let user = create_test_user(db.pool(), Default::default()).await;
        let project_a = create_test_project(db.pool(), user.id, Default::default()).await;
        let project_b = create_test_project(db.pool(), user.id, Default::default()).await;
        let task = create_test_task(db.pool(), project_a.id, Default::default()).await;

        let deleted = tasks::delete_task(db.pool(), task.id, project_b.id)
            .await
            .expect("delete query should succeed");

        assert!(!deleted);

        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM acme.tasks WHERE id = $1")
            .bind(task.id)
            .fetch_one(db.pool())
            .await
            .expect("existence query should succeed");

        assert_eq!(exists.0, 1);

        cleanup::delete_user(db.pool(), user.id)
            .await
            .expect("cleanup should succeed");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn nightfire_media_usage_sync_stores_nested_block_id_locators() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        use acme_db::media;
        use acme_test_utils::{cleanup, setup_test_db};
        use underlay_media::nightfire::{NightfireFieldNameMatcher, NightfireMediaUsageExtractor};
        use underlay_media::MediaUsageProvenanceKind;
        use underlay_nightfire::NightfireValue;
        use uuid::Uuid;

        let db = setup_test_db().await;

        let user_id = Uuid::now_v7();
        let project_id = Uuid::now_v7();
        let task_id = Uuid::now_v7();

        sqlx::query(
            r#"
            INSERT INTO auth.users (id, email, role, status, display_name)
            VALUES ($1, $2, 'user', 'active', 'Nightfire Test User')
            "#,
        )
        .bind(user_id)
        .bind(format!("nightfire-{}@example.com", user_id.simple()))
        .execute(db.pool())
        .await
        .expect("user insert should succeed");

        sqlx::query(
            r#"
            INSERT INTO acme.projects (id, owner_id, name, status)
            VALUES ($1, $2, 'Nightfire Test Project', 'active')
            "#,
        )
        .bind(project_id)
        .bind(user_id)
        .execute(db.pool())
        .await
        .expect("project insert should succeed");

        sqlx::query(
            r#"
            INSERT INTO acme.tasks (id, project_id, title, status, priority, position)
            VALUES ($1, $2, 'Nightfire Test Task', 'pending', 'medium', 0)
            "#,
        )
        .bind(task_id)
        .bind(project_id)
        .execute(db.pool())
        .await
        .expect("task insert should succeed");

        let media_id = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO media.media (id, kind, visibility, title, created_by, updated_by)
            VALUES ($1, 'image', 'restricted', 'Nightfire Test Media', $2, $2)
            "#,
        )
        .bind(media_id)
        .bind(user_id)
        .execute(db.pool())
        .await
        .expect("media insert should succeed");

        let notes: NightfireValue = serde_json::from_value(serde_json::json!({
            "schema": "acme:task/notes@1",
            "block": {
                "id": "gallery_01",
                "type": "notes.gallery",
                "version": "initial",
                "hash": "",
                "data": {
                    "pages": [
                        {
                            "title": "Cover",
                            "imageId": media_id.to_string(),
                            "caption": "Nested media reference"
                        }
                    ]
                }
            }
        }))
        .expect("nightfire value should deserialize");

        let extractor = NightfireMediaUsageExtractor::new(
            "task",
            Some(task_id),
            "notes",
            MediaUsageProvenanceKind::ContentSync,
            NightfireFieldNameMatcher::with_common_media_fields(),
        );
        let repo = media::AcmeMediaUsageSyncRepo::new(db.pool());

        let report = extractor
            .extract_and_sync(&repo, &notes)
            .await
            .expect("media usage sync should succeed");

        assert_eq!(report.inserted, 1);
        assert_eq!(report.retained, 0);
        assert_eq!(report.removed, 0);

        let usages = media::list_usages_by_entity(db.pool(), "task", task_id, "notes")
            .await
            .expect("usage rows should load");

        assert_eq!(usages.len(), 1);
        let usage = &usages[0];
        assert_eq!(usage.media_id, media_id);
        assert_eq!(usage.used_by_type, "task");
        assert_eq!(usage.used_by_id, Some(task_id));
        assert_eq!(usage.owner_field.as_deref(), Some("notes"));
        assert_eq!(usage.content_kind, "structured_content");
        assert_eq!(usage.locator_kind, "block_id");
        assert_eq!(usage.locator_key, "gallery_01#/pages/0/imageId");
        assert_eq!(usage.usage_role, "embedded");
        assert_eq!(usage.provenance_kind, "content_sync");

        sqlx::query("DELETE FROM media.media WHERE id = $1")
            .bind(media_id)
            .execute(db.pool())
            .await
            .expect("media cleanup should succeed");

        cleanup::delete_user(db.pool(), user_id)
            .await
            .expect("cleanup should succeed");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn task_notes_locator_resolver_reads_current_nested_media_value() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        use acme_db::tasks;
        use acme_test_utils::{cleanup, setup_test_db};
        use uuid::Uuid;

        let db = setup_test_db().await;

        let user_id = Uuid::now_v7();
        let project_id = Uuid::now_v7();
        let task_id = Uuid::now_v7();
        let media_id = Uuid::now_v7();

        sqlx::query(
            r#"
            INSERT INTO auth.users (id, email, role, status, display_name)
            VALUES ($1, $2, 'user', 'active', 'Locator Test User')
            "#,
        )
        .bind(user_id)
        .bind(format!("locator-{}@example.com", user_id.simple()))
        .execute(db.pool())
        .await
        .expect("user insert should succeed");

        sqlx::query(
            r#"
            INSERT INTO acme.projects (id, owner_id, name, status)
            VALUES ($1, $2, 'Locator Test Project', 'active')
            "#,
        )
        .bind(project_id)
        .bind(user_id)
        .execute(db.pool())
        .await
        .expect("project insert should succeed");

        sqlx::query(
            r#"
            INSERT INTO acme.tasks (id, project_id, title, status, priority, position, notes)
            VALUES ($1, $2, 'Locator Test Task', 'pending', 'medium', 0, $3::jsonb)
            "#,
        )
        .bind(task_id)
        .bind(project_id)
        .bind(serde_json::json!({
            "schema": "acme:task/notes@1",
            "block": {
                "id": "nf_locator_demo",
                "type": "notes.gallery",
                "version": "initial",
                "hash": "",
                "data": {
                    "pages": [
                        {
                            "title": "Lookup test",
                            "imageId": media_id.to_string(),
                            "caption": "Current nested reference"
                        }
                    ]
                }
            }
        }))
        .execute(db.pool())
        .await
        .expect("task insert should succeed");

        let resolved = tasks::resolve_task_notes_locator(
            db.pool(),
            task_id,
            "block_id",
            "nf_locator_demo#/pages/0/imageId",
        )
        .await
        .expect("locator resolve should succeed");

        assert_eq!(resolved, Some(serde_json::json!(media_id.to_string())));

        cleanup::delete_user(db.pool(), user_id)
            .await
            .expect("cleanup should succeed");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn project_description_media_usage_sync_stores_nested_block_id_locators() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        use acme_db::media;
        use acme_test_utils::{cleanup, setup_test_db};
        use underlay_media::nightfire::{NightfireFieldNameMatcher, NightfireMediaUsageExtractor};
        use underlay_media::MediaUsageProvenanceKind;
        use underlay_nightfire::NightfireValue;
        use uuid::Uuid;

        let db = setup_test_db().await;

        let user_id = Uuid::now_v7();
        let project_id = Uuid::now_v7();
        let media_id = Uuid::now_v7();

        sqlx::query(
            r#"
            INSERT INTO auth.users (id, email, role, status, display_name)
            VALUES ($1, $2, 'user', 'active', 'Project Description Test User')
            "#,
        )
        .bind(user_id)
        .bind(format!(
            "project-description-{}@example.com",
            user_id.simple()
        ))
        .execute(db.pool())
        .await
        .expect("user insert should succeed");

        sqlx::query(
            r#"
            INSERT INTO acme.projects (id, owner_id, name, status)
            VALUES ($1, $2, 'Project Description Test', 'active')
            "#,
        )
        .bind(project_id)
        .bind(user_id)
        .execute(db.pool())
        .await
        .expect("project insert should succeed");

        sqlx::query(
            r#"
            INSERT INTO media.media (id, kind, visibility, title, created_by, updated_by)
            VALUES ($1, 'image', 'restricted', 'Project Description Media', $2, $2)
            "#,
        )
        .bind(media_id)
        .bind(user_id)
        .execute(db.pool())
        .await
        .expect("media insert should succeed");

        let description: NightfireValue = serde_json::from_value(serde_json::json!({
            "schema": "acme:project/description@1",
            "blocks": [
                {
                    "id": "project_gallery_01",
                    "type": "notes.gallery",
                    "version": "initial",
                    "hash": "",
                    "data": {
                        "pages": [
                            {
                                "title": "Overview",
                                "imageId": media_id.to_string(),
                                "caption": "Nested project description media reference"
                            }
                        ]
                    }
                }
            ]
        }))
        .expect("nightfire value should deserialize");

        let extractor = NightfireMediaUsageExtractor::new(
            "project",
            Some(project_id),
            "description",
            MediaUsageProvenanceKind::ContentSync,
            NightfireFieldNameMatcher::with_common_media_fields(),
        );
        let repo = media::AcmeMediaUsageSyncRepo::new(db.pool());

        let report = extractor
            .extract_and_sync(&repo, &description)
            .await
            .expect("media usage sync should succeed");

        assert_eq!(report.inserted, 1);
        assert_eq!(report.retained, 0);
        assert_eq!(report.removed, 0);

        let usages = media::list_usages_by_entity(db.pool(), "project", project_id, "description")
            .await
            .expect("usage rows should load");

        assert_eq!(usages.len(), 1);
        let usage = &usages[0];
        assert_eq!(usage.media_id, media_id);
        assert_eq!(usage.used_by_type, "project");
        assert_eq!(usage.used_by_id, Some(project_id));
        assert_eq!(usage.owner_field.as_deref(), Some("description"));
        assert_eq!(usage.content_kind, "structured_content");
        assert_eq!(usage.locator_kind, "block_id");
        assert_eq!(usage.locator_key, "project_gallery_01#/pages/0/imageId");
        assert_eq!(usage.usage_role, "embedded");
        assert_eq!(usage.provenance_kind, "content_sync");

        sqlx::query("DELETE FROM media.media WHERE id = $1")
            .bind(media_id)
            .execute(db.pool())
            .await
            .expect("media cleanup should succeed");

        cleanup::delete_user(db.pool(), user_id)
            .await
            .expect("cleanup should succeed");
    }
}
