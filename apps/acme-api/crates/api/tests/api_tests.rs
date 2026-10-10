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
            "blocks": [{
                "id": "gallery_01",
                "type": "notes.gallery",
                "version": "initial",
                "data": {
                    "pages": [
                        {
                            "title": "Cover",
                            "image_id": media_id.to_string(),
                            "caption": "Nested media reference"
                        }
                    ]
                }
            }]
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
        assert_eq!(usage.locator_key, "gallery_01#/pages/0/image_id");
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
            "blocks": [{
                "id": "nf_locator_demo",
                "type": "notes.gallery",
                "version": "initial",
                "data": {
                    "pages": [
                        {
                            "title": "Lookup test",
                            "image_id": media_id.to_string(),
                            "caption": "Current nested reference"
                        }
                    ]
                }
            }]
        }))
        .execute(db.pool())
        .await
        .expect("task insert should succeed");

        let resolved = tasks::resolve_task_notes_locator(
            db.pool(),
            task_id,
            "block_id",
            "nf_locator_demo#/pages/0/image_id",
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
            "blocks": [{
                    "id": "project_gallery_01",
                    "type": "notes.gallery",
                    "version": "initial",
                    "data": {
                        "pages": [
                            {
                                "title": "Overview",
                            "image_id": media_id.to_string(),
                                "caption": "Nested project description media reference"
                            }
                        ]
                    }
                }]
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
        assert_eq!(usage.locator_key, "project_gallery_01#/pages/0/image_id");
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

mod bounded_collection_tests {
    use std::env;

    use acme_db::{media, tasks};
    use acme_test_utils::{
        cleanup,
        fixtures::{create_test_project, create_test_task, create_test_user},
        setup_test_db,
    };
    use chrono::Utc;
    use underlay_media::{sync::sync_media_usages_for_record, MediaUsageProvenanceKind};
    use uuid::Uuid;

    const PAGE_SIZE: i64 = 100;

    fn skip_without_db() -> bool {
        env::var("DATABASE_URL").is_err() && env::var("TEST_DATABASE_URL").is_err()
    }

    #[tokio::test]
    async fn front_project_and_task_pages_are_stable_and_scoped() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let owner = create_test_user(db.pool(), Default::default()).await;
        let other_owner = create_test_user(db.pool(), Default::default()).await;
        let project = create_test_project(db.pool(), owner.id, Default::default()).await;
        let other_project = create_test_project(db.pool(), owner.id, Default::default()).await;
        let other_owner_project =
            create_test_project(db.pool(), other_owner.id, Default::default()).await;

        let project_ids = (0..205).map(|_| Uuid::now_v7()).collect::<Vec<_>>();
        let mut expected_project_ids = project_ids.clone();
        expected_project_ids.sort();
        let same_created_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO acme.projects (id, owner_id, name, status, weight, created_at)
            SELECT id, $1, 'equal sort project', 'active', 7, $3
            FROM UNNEST($2::uuid[]) AS project_ids(id)
            "#,
        )
        .bind(owner.id)
        .bind(&project_ids)
        .bind(same_created_at)
        .execute(db.pool())
        .await
        .expect("project page fixtures should insert");
        let archived_project = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO acme.projects (id, owner_id, name, status) VALUES ($1, $2, 'archived', 'archived')",
        )
        .bind(archived_project)
        .bind(owner.id)
        .execute(db.pool())
        .await
        .expect("archived project fixture should insert");

        let project_page_1 =
            tasks::list_projects_for_user(db.pool(), owner.id, false, PAGE_SIZE, 0)
                .await
                .expect("first project page should load");
        let project_page_2 =
            tasks::list_projects_for_user(db.pool(), owner.id, false, PAGE_SIZE, PAGE_SIZE)
                .await
                .expect("second project page should load");
        let project_page_3 =
            tasks::list_projects_for_user(db.pool(), owner.id, false, PAGE_SIZE, PAGE_SIZE * 2)
                .await
                .expect("last project page should load");
        assert_eq!(project_page_1.total, 207);
        assert_eq!(project_page_2.total, 207);
        assert_eq!(project_page_3.total, 207);
        assert_eq!(project_page_1.data.len(), 100);
        assert_eq!(project_page_2.data.len(), 100);
        assert_eq!(project_page_3.data.len(), 7);
        let returned_project_ids = project_page_1
            .data
            .into_iter()
            .chain(project_page_2.data)
            .chain(project_page_3.data)
            .filter(|row| row.name == "equal sort project")
            .map(|row| row.id)
            .collect::<Vec<_>>();
        assert_eq!(returned_project_ids, expected_project_ids);
        assert!(!returned_project_ids.contains(&other_owner_project.id));

        let task_ids = (0..205).map(|_| Uuid::now_v7()).collect::<Vec<_>>();
        let mut expected_task_ids = task_ids.clone();
        expected_task_ids.sort();
        sqlx::query(
            r#"
            INSERT INTO acme.tasks (id, project_id, title, status, priority, position, created_at)
            SELECT id, $1, 'equal sort task', 'pending', 'medium', 4, $3
            FROM UNNEST($2::uuid[]) AS task_ids(id)
            "#,
        )
        .bind(project.id)
        .bind(&task_ids)
        .bind(same_created_at)
        .execute(db.pool())
        .await
        .expect("task page fixtures should insert");
        sqlx::query(
            "INSERT INTO acme.tasks (id, project_id, title, status, priority, position) VALUES ($1, $2, 'completed', 'completed', 'medium', 4)",
        )
        .bind(Uuid::now_v7())
        .bind(project.id)
        .execute(db.pool())
        .await
        .expect("completed task fixture should insert");
        let other_project_task =
            create_test_task(db.pool(), other_project.id, Default::default()).await;

        let task_page_1 = tasks::list_tasks_for_project(db.pool(), project.id, false, PAGE_SIZE, 0)
            .await
            .expect("first task page should load");
        let task_page_2 =
            tasks::list_tasks_for_project(db.pool(), project.id, false, PAGE_SIZE, PAGE_SIZE)
                .await
                .expect("second task page should load");
        let task_page_3 =
            tasks::list_tasks_for_project(db.pool(), project.id, false, PAGE_SIZE, PAGE_SIZE * 2)
                .await
                .expect("last task page should load");
        assert_eq!(task_page_1.1, 205);
        assert_eq!(task_page_2.1, 205);
        assert_eq!(task_page_3.1, 205);
        assert_eq!(task_page_1.0.len(), 100);
        assert_eq!(task_page_2.0.len(), 100);
        assert_eq!(task_page_3.0.len(), 5);
        let returned_task_ids = task_page_1
            .0
            .into_iter()
            .chain(task_page_2.0)
            .chain(task_page_3.0)
            .map(|row| row.id)
            .collect::<Vec<_>>();
        assert_eq!(returned_task_ids, expected_task_ids);
        assert!(!returned_task_ids.contains(&other_project_task.id));

        cleanup::delete_user(db.pool(), owner.id)
            .await
            .expect("owner fixture cleanup should succeed");
        cleanup::delete_user(db.pool(), other_owner.id)
            .await
            .expect("other owner fixture cleanup should succeed");
    }

    #[tokio::test]
    async fn media_pages_and_reconciliation_traverse_every_bounded_batch() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let media_id = Uuid::now_v7();
        let owner_id = Uuid::now_v7();
        let other_owner_id = Uuid::now_v7();
        let created_at = Utc::now();
        sqlx::query(
            "INSERT INTO media.media (id, kind, visibility, title) VALUES ($1, 'image', 'restricted', 'bounded collection fixture')",
        )
        .bind(media_id)
        .execute(db.pool())
        .await
        .expect("media fixture should insert");

        let edge_ids = (0..205).map(|_| Uuid::now_v7()).collect::<Vec<_>>();
        let mut expected_usage_ids = edge_ids.clone();
        let foreign_edge_id = Uuid::now_v7();
        let other_provenance_edge_id = Uuid::now_v7();
        expected_usage_ids.push(foreign_edge_id);
        expected_usage_ids.push(other_provenance_edge_id);
        expected_usage_ids.sort_by(|left, right| right.cmp(left));

        sqlx::query(
            r#"
            INSERT INTO media.media_usage (
                id, media_id, used_by_type, used_by_id, owner_field,
                content_kind, locator_kind, locator_key, usage_role, provenance_kind, created_at
            )
            SELECT id, $1, 'task', $2, 'notes', 'structured_content',
                   'block_id', id::text, 'embedded', 'content_sync', $3
            FROM UNNEST($4::uuid[]) AS edge_ids(id)
            "#,
        )
        .bind(media_id)
        .bind(owner_id)
        .bind(created_at)
        .bind(&edge_ids)
        .execute(db.pool())
        .await
        .expect("usage edge fixtures should insert");
        sqlx::query(
            r#"
            INSERT INTO media.media_usage (
                id, media_id, used_by_type, used_by_id, owner_field,
                content_kind, locator_kind, locator_key, usage_role, provenance_kind, created_at
            ) VALUES
                ($1, $3, 'task', $4, 'notes', 'structured_content', 'block_id', 'other-owner', 'embedded', 'content_sync', $5),
                ($2, $3, 'task', $6, 'notes', 'structured_content', 'block_id', 'manual-edge', 'embedded', 'manual', $5)
            "#,
        )
        .bind(foreign_edge_id)
        .bind(other_provenance_edge_id)
        .bind(media_id)
        .bind(other_owner_id)
        .bind(created_at)
        .bind(owner_id)
        .execute(db.pool())
        .await
        .expect("out-of-scope usage fixtures should insert");

        let page_1 = media::list_media_usages(db.pool(), media_id, PAGE_SIZE, 0)
            .await
            .expect("first usage page should load");
        let page_2 = media::list_media_usages(db.pool(), media_id, PAGE_SIZE, PAGE_SIZE)
            .await
            .expect("second usage page should load");
        let page_3 = media::list_media_usages(db.pool(), media_id, PAGE_SIZE, PAGE_SIZE * 2)
            .await
            .expect("last usage page should load");
        assert_eq!(page_1.1, 207);
        assert_eq!(page_2.1, 207);
        assert_eq!(page_3.1, 207);
        assert_eq!(page_1.0.len(), 100);
        assert_eq!(page_2.0.len(), 100);
        assert_eq!(page_3.0.len(), 7);
        let returned_usage_ids = page_1
            .0
            .into_iter()
            .chain(page_2.0)
            .chain(page_3.0)
            .map(|row| row.id)
            .collect::<Vec<_>>();
        assert_eq!(returned_usage_ids, expected_usage_ids);

        let entity_usages = media::list_usages_by_entity(db.pool(), "task", owner_id, "notes")
            .await
            .expect("entity usages should traverse every batch");
        assert_eq!(entity_usages.len(), 206);
        assert!(entity_usages
            .iter()
            .all(|row| row.used_by_id == Some(owner_id)));

        let repo = media::AcmeMediaUsageSyncRepo::new(db.pool());
        let report = sync_media_usages_for_record(
            &repo,
            "task",
            owner_id,
            &[],
            &MediaUsageProvenanceKind::ContentSync,
        )
        .await
        .expect("empty desired set should reconcile the full content-sync scope");
        assert_eq!(report.removed, 205);
        let content_sync_edges = media::list_usage_edges_for_owner(
            db.pool(),
            "task",
            owner_id,
            &MediaUsageProvenanceKind::ContentSync,
        )
        .await
        .expect("remaining content-sync edges should load");
        assert!(content_sync_edges.is_empty());
        let manual_edges = media::list_usage_edges_for_owner(
            db.pool(),
            "task",
            owner_id,
            &MediaUsageProvenanceKind::Manual,
        )
        .await
        .expect("manual provenance should remain outside content reconciliation");
        assert_eq!(manual_edges.len(), 1);
        let other_owner_edges = media::list_usage_edges_for_owner(
            db.pool(),
            "task",
            other_owner_id,
            &MediaUsageProvenanceKind::ContentSync,
        )
        .await
        .expect("other owner scope should remain isolated");
        assert_eq!(other_owner_edges.len(), 1);

        sqlx::query("DELETE FROM media.media WHERE id = $1")
            .bind(media_id)
            .execute(db.pool())
            .await
            .expect("media fixture cleanup should succeed");
    }

    #[tokio::test]
    async fn admin_session_pages_and_complete_list_preserve_revoked_rows() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let user = create_test_user(db.pool(), Default::default()).await;
        let session_ids = (0..205).map(|_| Uuid::now_v7()).collect::<Vec<_>>();
        let mut expected_ids = session_ids.clone();
        expected_ids.sort_by(|left, right| right.cmp(left));
        let created_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO auth.sessions (
                id, user_id, roles, is_active,
                access_token_fingerprint, refresh_token_fingerprint,
                refresh_token_id, refresh_token_version,
                access_token_expires_at, refresh_token_expires_at,
                created_at, updated_at, last_used_at, status
            )
            SELECT id, $1, '[]'::jsonb, TRUE,
                   'access-fingerprint', 'refresh-fingerprint', id, 1,
                   $3, $4, $2, $2, $2, 'active'
            FROM UNNEST($5::uuid[]) AS session_ids(id)
            "#,
        )
        .bind(user.id)
        .bind(created_at)
        .bind(created_at + chrono::Duration::hours(1))
        .bind(created_at + chrono::Duration::days(1))
        .bind(&session_ids)
        .execute(db.pool())
        .await
        .expect("session fixtures should insert");

        assert!(acme_db::users::revoke_session_admin(
            db.pool(),
            user.id,
            session_ids[0],
            "bounded list test",
        )
        .await
        .expect("session revocation should succeed"));

        let first_page =
            acme_db::users::list_sessions_for_user_page(db.pool(), user.id, PAGE_SIZE, 0)
                .await
                .expect("first session page should load");
        let last_page = acme_db::users::list_sessions_for_user_page(
            db.pool(),
            user.id,
            PAGE_SIZE,
            PAGE_SIZE * 2,
        )
        .await
        .expect("last session page should load");
        assert_eq!(first_page.1, 205);
        assert_eq!(last_page.1, 205);
        assert_eq!(first_page.0.len(), 100);
        assert_eq!(last_page.0.len(), 5);

        let all_sessions = acme_db::users::list_sessions_for_user(db.pool(), user.id)
            .await
            .expect("complete internal session list should traverse all batches");
        assert_eq!(all_sessions.len(), 205);
        let all_ids = all_sessions.iter().map(|row| row.id).collect::<Vec<_>>();
        assert_eq!(all_ids, expected_ids);
        assert!(all_sessions
            .iter()
            .any(|row| row.id == session_ids[0] && row.status == "revoked"));

        cleanup::delete_user(db.pool(), user.id)
            .await
            .expect("session fixture cleanup should succeed");
    }

    #[tokio::test]
    async fn media_versions_renditions_unused_media_comments_and_labels_traverse_all_batches() {
        if skip_without_db() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let owner = create_test_user(db.pool(), Default::default()).await;
        let project = create_test_project(db.pool(), owner.id, Default::default()).await;
        let task = create_test_task(db.pool(), project.id, Default::default()).await;
        let same_created_at = Utc::now();

        let media_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO media.media (id, kind, visibility, title) VALUES ($1, 'image', 'restricted', 'versions fixture')",
        )
        .bind(media_id)
        .execute(db.pool())
        .await
        .expect("media version fixture should insert");

        let version_ids = (0..205).map(|_| Uuid::now_v7()).collect::<Vec<_>>();
        let mut expected_version_ids = version_ids.clone();
        expected_version_ids.sort_by(|left, right| right.cmp(left));
        sqlx::query(
            r#"
            INSERT INTO media.media_version (id, media_id, state, mime_type, created_at)
            SELECT id, $1, 'ready', 'image/png', $3
            FROM UNNEST($2::uuid[]) AS version_ids(id)
            "#,
        )
        .bind(media_id)
        .bind(&version_ids)
        .bind(same_created_at)
        .execute(db.pool())
        .await
        .expect("version fixtures should insert");
        let versions = media::list_media_versions(db.pool(), media_id)
            .await
            .expect("complete version listing should traverse every batch");
        assert_eq!(versions.len(), 205);
        assert_eq!(
            versions.iter().map(|row| row.id).collect::<Vec<_>>(),
            expected_version_ids
        );

        let rendition_ids = (0..205).map(|_| Uuid::now_v7()).collect::<Vec<_>>();
        let mut expected_renditions = rendition_ids
            .iter()
            .enumerate()
            .map(|(index, id)| (format!("rendition-{index:03}"), *id))
            .collect::<Vec<_>>();
        expected_renditions.sort();
        sqlx::query(
            r#"
            INSERT INTO media.media_rendition (
                id, media_version_id, kind, byte_size, mime_type,
                storage_provider, bucket, object_key
            )
            SELECT id, $1, 'rendition-' || lpad((ordinal - 1)::text, 3, '0'), 1, 'image/png',
                   'test', 'test', 'renditions/' || id::text
            FROM UNNEST($2::uuid[]) WITH ORDINALITY AS rendition_ids(id, ordinal)
            "#,
        )
        .bind(version_ids[0])
        .bind(&rendition_ids)
        .execute(db.pool())
        .await
        .expect("rendition fixtures should insert");
        let renditions = media::list_media_renditions(db.pool(), version_ids[0])
            .await
            .expect("complete rendition listing should traverse every batch");
        assert_eq!(renditions.len(), 205);
        assert_eq!(
            renditions
                .iter()
                .map(|row| (row.kind.clone(), row.id))
                .collect::<Vec<_>>(),
            expected_renditions
        );

        let unused_media_ids = (0..205).map(|_| Uuid::now_v7()).collect::<Vec<_>>();
        let unused_version_ids = (0..205).map(|_| Uuid::now_v7()).collect::<Vec<_>>();
        let mut expected_unused_ids = unused_media_ids.clone();
        expected_unused_ids.sort_by(|left, right| right.cmp(left));
        sqlx::query(
            r#"
            INSERT INTO media.media (id, kind, visibility, title, created_at)
            SELECT id, 'image', 'restricted', 'unused batch fixture', $2
            FROM UNNEST($1::uuid[]) AS media_ids(id)
            "#,
        )
        .bind(&unused_media_ids)
        .bind(same_created_at)
        .execute(db.pool())
        .await
        .expect("unused media fixtures should insert");
        sqlx::query(
            r#"
            INSERT INTO media.media_version (id, media_id, state, mime_type, created_at)
            SELECT media_ids.version_id, media_ids.media_id, 'ready', 'image/png', $3
            FROM UNNEST($1::uuid[], $2::uuid[]) AS media_ids(media_id, version_id)
            "#,
        )
        .bind(&unused_media_ids)
        .bind(&unused_version_ids)
        .bind(same_created_at)
        .execute(db.pool())
        .await
        .expect("unused media versions should insert");
        sqlx::query(
            r#"
            UPDATE media.media AS media
            SET current_version_id = fixtures.version_id
            FROM UNNEST($1::uuid[], $2::uuid[]) AS fixtures(media_id, version_id)
            WHERE media.id = fixtures.media_id
            "#,
        )
        .bind(&unused_media_ids)
        .bind(&unused_version_ids)
        .execute(db.pool())
        .await
        .expect("unused media should reference its current versions");
        let unused = media::list_unused_media(db.pool())
            .await
            .expect("complete unused media listing should traverse every batch");
        let returned_unused_ids = unused
            .iter()
            .filter(|row| unused_media_ids.contains(&row.id))
            .map(|row| row.id)
            .collect::<Vec<_>>();
        assert_eq!(returned_unused_ids, expected_unused_ids);

        let comment_ids = (0..205).map(|_| Uuid::now_v7()).collect::<Vec<_>>();
        let mut expected_comment_ids = comment_ids.clone();
        expected_comment_ids.sort();
        sqlx::query(
            r#"
            INSERT INTO acme.task_comments (id, task_id, author_id, body, created_at)
            SELECT id, $1, $2, 'bounded traversal comment', $4
            FROM UNNEST($3::uuid[]) AS comment_ids(id)
            "#,
        )
        .bind(task.id)
        .bind(owner.id)
        .bind(&comment_ids)
        .bind(same_created_at)
        .execute(db.pool())
        .await
        .expect("comment fixtures should insert");
        let comments = tasks::list_task_comments(db.pool(), task.id)
            .await
            .expect("complete comment listing should traverse every batch");
        assert_eq!(comments.len(), 205);
        assert_eq!(
            comments.iter().map(|row| row.id).collect::<Vec<_>>(),
            expected_comment_ids
        );

        let label_ids = (0..205).map(|_| Uuid::now_v7()).collect::<Vec<_>>();
        let mut expected_label_ids = label_ids
            .iter()
            .enumerate()
            .map(|(index, id)| (format!("bounded-label-{index:03}"), *id))
            .collect::<Vec<_>>();
        expected_label_ids.sort();
        sqlx::query(
            r#"
            INSERT INTO acme.labels (id, project_id, name, color, weight, created_at)
            SELECT id, $1, 'bounded-label-' || lpad((ordinal - 1)::text, 3, '0'), '#6366f1', 7, $3
            FROM UNNEST($2::uuid[]) WITH ORDINALITY AS label_ids(id, ordinal)
            "#,
        )
        .bind(project.id)
        .bind(&label_ids)
        .bind(same_created_at)
        .execute(db.pool())
        .await
        .expect("label fixtures should insert");
        let labels = tasks::list_labels_for_project(db.pool(), project.id)
            .await
            .expect("complete label listing should traverse every batch");
        assert_eq!(labels.len(), 205);
        assert_eq!(
            labels
                .iter()
                .map(|row| (row.name.clone(), row.id))
                .collect::<Vec<_>>(),
            expected_label_ids
        );

        sqlx::query("DELETE FROM media.media WHERE id = $1 OR id = ANY($2)")
            .bind(media_id)
            .bind(&unused_media_ids)
            .execute(db.pool())
            .await
            .expect("media traversal fixtures should clean up");
        cleanup::delete_user(db.pool(), owner.id)
            .await
            .expect("task reader fixtures should clean up");
    }
}

mod bounded_route_tests {
    use std::{
        env,
        ffi::{OsStr, OsString},
        sync::{Arc, OnceLock},
    };

    use acme_api::{config::AcmeConfig, routes, state::AppState};
    use acme_auth::{AcmeLocalAuthService, EmailTotpService};
    use acme_test_utils::{
        cleanup,
        fixtures::{create_test_project, create_test_task, create_test_user},
        setup_test_db,
    };
    use async_trait::async_trait;
    use axum::{body::Body, http::Request, Router};
    use serde_json::Value;
    use tower::util::ServiceExt;
    use underlay_auth::{AuthError, AuthProvider, Principal, RoleSet};
    use underlay_blob::NoopAdapter;
    use underlay_jobs_postgres::JobRepository;
    use underlay_observability::Environment;

    static TEST_ENV_ONCE: OnceLock<()> = OnceLock::new();
    static ENVIRONMENT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    #[derive(Clone)]
    struct FixedUserAuthProvider {
        user_id: acme_core::Uuid,
    }

    #[async_trait]
    impl AuthProvider for FixedUserAuthProvider {
        async fn authenticate_bearer(
            &self,
            bearer_token: &str,
        ) -> underlay_auth::AuthResult<Principal> {
            if bearer_token != "bounded-route-test-token" {
                return Err(AuthError::TokenInvalid);
            }
            Ok(Principal {
                user_id: self.user_id,
                roles: RoleSet::new(["user"]),
            })
        }
    }

    fn ensure_test_environment() {
        TEST_ENV_ONCE.get_or_init(|| {
            let (jwt_cfg, _) =
                underlay_auth_jwt::JwtConfig::generate().expect("test JWT keys should generate");
            env::set_var("AUTH_JWT_PRIVATE_KEY", jwt_cfg.private_key_b64());
            env::set_var("AUTH_JWT_PUBLIC_KEY", jwt_cfg.public_key_b64());
            env::set_var("ENVIRONMENT", "test");
            env::set_var("WEBAUTHN_RP_ID", "localhost");
            env::set_var("WEBAUTHN_RP_ORIGIN", "http://localhost:3000");
            env::set_var("WEBAUTHN_RP_NAME", "Reference API tests");
        });
    }

    async fn test_state(pool: sqlx::PgPool, user_id: acme_core::Uuid) -> AppState {
        ensure_test_environment();
        let local_auth = Arc::new(
            AcmeLocalAuthService::from_env(pool.clone()).expect("test auth service should build"),
        );
        let auth_provider: Arc<dyn AuthProvider> = Arc::new(FixedUserAuthProvider { user_id });
        let app_config = acme_infra::AppConfig::from_env().expect("test config should load");
        let email_manager = Arc::new(
            acme_infra::create_email_manager(&app_config.email)
                .expect("test email manager should build"),
        );
        let email_templates = Arc::new(
            acme_infra::create_template_engine(&app_config.email)
                .expect("test email templates should build"),
        );
        let email_totp = Arc::new(EmailTotpService::new(
            pool.clone(),
            email_manager.clone(),
            email_templates.clone(),
            app_config.email.clone(),
        ));

        AppState {
            local_auth,
            auth_provider,
            cookie_config: underlay_http::AuthCookieConfig::default(),
            email_manager,
            email_templates,
            email_totp,
            email_config: app_config.email,
            blob_adapter: Arc::new(NoopAdapter::new()),
            job_repository: Some(Arc::new(JobRepository::new(pool))),
            config: AcmeConfig::default(),
        }
    }

    async fn json_body(response: axum::response::Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("route response body should be readable");
        serde_json::from_slice(&body).expect("route response should be JSON")
    }

    async fn get_page(app: Router, path: &str) -> (axum::http::StatusCode, Value) {
        let response = app
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", "Bearer bounded-route-test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("front list route should respond");
        let status = response.status();
        (status, json_body(response).await)
    }

    #[tokio::test]
    async fn project_and_task_http_routes_return_bounded_page_contracts() {
        if env::var("DATABASE_URL").is_err() && env::var("TEST_DATABASE_URL").is_err() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let db = setup_test_db().await;
        let owner = create_test_user(db.pool(), Default::default()).await;
        let first_project = create_test_project(db.pool(), owner.id, Default::default()).await;
        let second_project = create_test_project(db.pool(), owner.id, Default::default()).await;
        let tasks = [
            create_test_task(db.pool(), first_project.id, Default::default()).await,
            create_test_task(db.pool(), first_project.id, Default::default()).await,
        ];
        let environment_guard = ENVIRONMENT_LOCK.lock().await;
        let state = test_state(db.pool_clone(), acme_core::Uuid(owner.id)).await;
        drop(environment_guard);
        let app = routes::build_router_for_environment(Environment::Test).with_state(state);

        let (status, first_projects) = get_page(app.clone(), "/v1/projects?page=1&limit=1").await;
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(first_projects["data"].as_array().unwrap().len(), 1);
        assert_eq!(first_projects["total"], 2);
        assert!(first_projects["has_more"].as_bool().unwrap());

        let (status, last_project_page) =
            get_page(app.clone(), "/v1/projects?page=2&limit=1").await;
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(last_project_page["data"].as_array().unwrap().len(), 1);
        assert_eq!(last_project_page["total"], 2);
        assert!(!last_project_page["has_more"].as_bool().unwrap());

        let task_path = format!("/v1/projects/{}/tasks?page=1&limit=1", first_project.id);
        let (status, first_task_page) = get_page(app.clone(), &task_path).await;
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(first_task_page["data"].as_array().unwrap().len(), 1);
        assert_eq!(first_task_page["total"], 2);
        assert!(first_task_page["has_more"].as_bool().unwrap());

        let task_path = format!("/v1/projects/{}/tasks?page=2&limit=1", first_project.id);
        let (status, last_task_page) = get_page(app, &task_path).await;
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(last_task_page["data"].as_array().unwrap().len(), 1);
        assert_eq!(last_task_page["total"], 2);
        assert!(!last_task_page["has_more"].as_bool().unwrap());
        let returned_task_ids = first_task_page["data"]
            .as_array()
            .unwrap()
            .iter()
            .chain(last_task_page["data"].as_array().unwrap())
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>();
        let expected_task_ids = tasks
            .iter()
            .map(|task| task.id.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            returned_task_ids,
            expected_task_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        );

        cleanup::delete_user(db.pool(), owner.id)
            .await
            .expect("route test data should clean up");
        assert_ne!(first_project.id, second_project.id);
    }

    struct RestoreEnvironment(Vec<(&'static OsStr, Option<OsString>)>);

    impl Drop for RestoreEnvironment {
        fn drop(&mut self) {
            for (name, prior_value) in &self.0 {
                if let Some(value) = prior_value {
                    env::set_var(name, value);
                } else {
                    env::remove_var(name);
                }
            }
        }
    }

    #[tokio::test]
    async fn public_build_router_resolves_docs_environment_and_fails_closed() {
        if env::var("DATABASE_URL").is_err() && env::var("TEST_DATABASE_URL").is_err() {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }

        let _environment_guard = ENVIRONMENT_LOCK.lock().await;
        let _restore = RestoreEnvironment(vec![
            (OsStr::new("ENVIRONMENT"), env::var_os("ENVIRONMENT")),
            (OsStr::new("ACME_ENV"), env::var_os("ACME_ENV")),
        ]);
        let db = setup_test_db().await;
        let owner = create_test_user(db.pool(), Default::default()).await;
        let state = test_state(db.pool_clone(), acme_core::Uuid(owner.id)).await;

        env::set_var("ENVIRONMENT", "dev");
        env::remove_var("ACME_ENV");
        let development = routes::build_router()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("development public router should answer");
        assert_eq!(development.status(), axum::http::StatusCode::OK);

        for environment in ["production", "staging", "unknown"] {
            env::set_var("ENVIRONMENT", environment);
            let response = routes::build_router()
                .with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .uri("/api/openapi.json")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .expect("non-development public router should answer");
            assert_eq!(
                response.status(),
                axum::http::StatusCode::NOT_FOUND,
                "OpenAPI must be absent for {environment}"
            );
        }

        env::remove_var("ENVIRONMENT");
        env::set_var("ACME_ENV", "unknown");
        let unknown_alias = routes::build_router()
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("unknown alias environment should answer");
        assert_eq!(unknown_alias.status(), axum::http::StatusCode::NOT_FOUND);

        cleanup::delete_user(db.pool(), owner.id)
            .await
            .expect("route test user should clean up");
    }
}
