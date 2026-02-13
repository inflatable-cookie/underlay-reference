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

    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::util::ServiceExt;

    // Minimal health handler for demonstration
    async fn health() -> &'static str {
        "OK"
    }

    fn test_router() -> Router {
        Router::new().route("/v1/health", get(health))
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let app = test_router();

        let request = Request::builder()
            .uri("/v1/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_route_returns_not_found() {
        let app = test_router();

        let request = Request::builder()
            .uri("/v1/unknown")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
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
}
