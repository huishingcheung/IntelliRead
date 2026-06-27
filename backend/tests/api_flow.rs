use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use intelliread_backend::{app, config::Config, state::AppState};
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn test_app(max_document_bytes: usize) -> axum::Router {
    test_app_with_expiration(max_document_bytes, 3600).await
}

async fn test_app_with_expiration(
    max_document_bytes: usize,
    jwt_expiration_seconds: i64,
) -> axum::Router {
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();
    app(Arc::new(AppState {
        db,
        config: Config {
            database_url: "sqlite::memory:".into(),
            jwt_secret: "test-secret-with-at-least-thirty-two-characters".into(),
            jwt_expiration_seconds,
            server_host: "127.0.0.1".parse().unwrap(),
            server_port: 3000,
            max_document_bytes,
            cors_allowed_origins: vec!["http://localhost:5173".into()],
            ai_provider: "local-deterministic".into(),
            ai_api_base_url: "https://api.deepseek.com".into(),
            ai_api_key: None,
            ai_model: "deepseek-v4-pro".into(),
            ai_timeout_seconds: 30,
            ai_max_output_tokens: 1200,
            ai_thinking: "disabled".into(),
        },
    }))
}

#[tokio::test]
async fn cors_allows_configured_origin_and_rejects_other_origins() {
    let app = test_app(1024).await;
    let allowed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/documents")
                .header(header::ORIGIN, "http://localhost:5173")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(allowed.status(), StatusCode::OK);
    assert_eq!(
        allowed.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(&"http://localhost:5173".parse().unwrap())
    );

    let rejected = app
        .clone()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/documents")
                .header(header::ORIGIN, "https://untrusted.example")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        rejected
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none()
    );
}

async fn json_request(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

async fn register_and_login(app: &axum::Router, suffix: &str) -> String {
    let email = format!("user{suffix}@example.com");
    let (status, registered) = json_request(app, "POST", "/api/v1/auth/register", json!({"username": format!("user{suffix}"), "email": email, "password": "correct-password"}), None).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(registered["success"], true);
    let (_, logged_in) = json_request(
        app,
        "POST",
        "/api/v1/auth/login",
        json!({"email": format!("user{suffix}@example.com"), "password": "correct-password"}),
        None,
    )
    .await;
    logged_in["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn upload(
    app: &axum::Router,
    token: &str,
    filename: &str,
    content: &str,
) -> (StatusCode, Value) {
    upload_bytes(app, token, filename, content.as_bytes()).await
}

async fn upload_bytes(
    app: &axum::Router,
    token: &str,
    filename: &str,
    content: &[u8],
) -> (StatusCode, Value) {
    let boundary = "intelliread-test-boundary";
    let mut body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"title\"\r\n\r\nTest Document\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: text/plain\r\n\r\n"
    )
    .into_bytes();
    body.extend_from_slice(content);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn registration_login_and_auth_failures() {
    let app = test_app(1024).await;
    let token = register_and_login(&app, "one").await;
    let (status, _) = json_request(
        &app,
        "POST",
        "/api/v1/auth/register",
        json!({"username":"userone","email":"other@example.com","password":"correct-password"}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = json_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        json!({"email":"userone@example.com","password":"wrong-password"}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents")
                .header(header::AUTHORIZATION, "Bearer invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(!token.is_empty());
}

#[tokio::test]
async fn document_flow_enforces_ownership_and_updates_progress() {
    let app = test_app(1024).await;
    let owner = register_and_login(&app, "owner").await;
    let other = register_and_login(&app, "other").await;
    let (status, uploaded) = upload(
        &app,
        &owner,
        "paper.md",
        "First paragraph.\n\nSecond paragraph.",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(uploaded["data"]["paragraphs"].as_array().unwrap().len(), 2);
    let id = uploaded["data"]["id"].as_str().unwrap();
    let (status, _) = json_request(
        &app,
        "GET",
        &format!("/api/v1/documents/{id}"),
        Value::Null,
        Some(&other),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, progress) = json_request(
        &app,
        "PUT",
        &format!("/api/v1/documents/{id}/progress"),
        json!({"paragraph_position":1,"progress_percent":100.0}),
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(progress["data"]["progress_percent"], 100.0);
    let (status, persisted) = json_request(
        &app,
        "GET",
        &format!("/api/v1/documents/{id}/progress"),
        Value::Null,
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(persisted["data"]["paragraph_position"], 1);
    assert_eq!(persisted["data"]["progress_percent"], 100.0);
}

#[tokio::test]
async fn document_upload_rejects_type_and_size() {
    let app = test_app(12).await;
    let token = register_and_login(&app, "limits").await;
    let (status, _) = upload(&app, &token, "paper.pdf", "content").await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let (status, _) = upload(&app, &token, "paper.txt", "this content is too large").await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);

    let app = test_app(1024).await;
    let token = register_and_login(&app, "encoding-limits").await;
    let (status, _) = upload_bytes(&app, &token, "empty.txt", &[]).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = upload_bytes(&app, &token, "invalid.txt", &[0xff, 0xfe, 0xfd]).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn document_update_search_archive_and_delete_flow() {
    let app = test_app(2048).await;
    let token = register_and_login(&app, "manage").await;
    let (status, uploaded) = upload(
        &app,
        &token,
        "research.md",
        "Introduction.\n\nA uniquely searchable quasar appears here.",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = uploaded["data"]["id"].as_str().unwrap();

    let (status, searched) = json_request(
        &app,
        "GET",
        "/api/v1/documents?q=quasar",
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(searched["data"]["items"].as_array().unwrap().len(), 1);

    let (status, archived) = json_request(
        &app,
        "PATCH",
        &format!("/api/v1/documents/{id}"),
        json!({"title":"Archived Research","archived":true}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(archived["data"]["archived_at"].is_string());

    let (_, active) =
        json_request(&app, "GET", "/api/v1/documents", Value::Null, Some(&token)).await;
    assert!(active["data"]["items"].as_array().unwrap().is_empty());
    let (_, archived_list) = json_request(
        &app,
        "GET",
        "/api/v1/documents?archived=true",
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(archived_list["data"]["items"].as_array().unwrap().len(), 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/documents/{id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let (status, _) = json_request(
        &app,
        "GET",
        &format!("/api/v1/documents/{id}"),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn tags_notes_and_highlights_enforce_ownership_and_ranges() {
    let app = test_app(2048).await;
    let owner = register_and_login(&app, "annotator").await;
    let other = register_and_login(&app, "outsider").await;
    let (_, uploaded) = upload(
        &app,
        &owner,
        "paper.txt",
        "Alpha paragraph.\n\nBeta paragraph.",
    )
    .await;
    let document_id = uploaded["data"]["id"].as_str().unwrap();
    let paragraph_id = uploaded["data"]["paragraphs"][0]["id"].as_str().unwrap();

    let (status, tag) = json_request(
        &app,
        "POST",
        "/api/v1/tags",
        json!({"name":"Important"}),
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let tag_id = tag["data"]["id"].as_str().unwrap();
    let (status, attached) = json_request(
        &app,
        "PUT",
        &format!("/api/v1/documents/{document_id}/tags"),
        json!({"tag_ids":[tag_id]}),
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(attached["data"].as_array().unwrap().len(), 1);
    let (status, filtered) = json_request(
        &app,
        "GET",
        &format!("/api/v1/documents?tag_id={tag_id}"),
        Value::Null,
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(filtered["data"]["items"].as_array().unwrap().len(), 1);
    let (status, _) = json_request(
        &app,
        "PUT",
        &format!("/api/v1/documents/{document_id}/tags"),
        json!({"tag_ids":[tag_id]}),
        Some(&other),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, note) = json_request(
        &app,
        "POST",
        &format!("/api/v1/documents/{document_id}/notes"),
        json!({"paragraph_id":paragraph_id,"content":"Review this sentence"}),
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let note_id = note["data"]["id"].as_str().unwrap();
    let (status, _) = json_request(
        &app,
        "PUT",
        &format!("/api/v1/notes/{note_id}"),
        json!({"content":"Changed by outsider"}),
        Some(&other),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, highlight) = json_request(
        &app,
        "POST",
        &format!("/api/v1/documents/{document_id}/highlights"),
        json!({"paragraph_id":paragraph_id,"start_offset":0,"end_offset":5,"color":"green"}),
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(highlight["data"]["color"], "green");
    let (status, _) = json_request(
        &app,
        "POST",
        &format!("/api/v1/documents/{document_id}/highlights"),
        json!({"paragraph_id":paragraph_id,"start_offset":0,"end_offset":500}),
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, overview) = json_request(
        &app,
        "GET",
        "/api/v1/statistics/overview",
        Value::Null,
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(overview["data"]["active_documents"], 1);
    assert_eq!(overview["data"]["paragraphs"], 2);
    assert_eq!(overview["data"]["tags"], 1);
    assert_eq!(overview["data"]["notes"], 1);
    assert_eq!(overview["data"]["highlights"], 1);
}

#[tokio::test]
async fn expired_jwt_is_rejected() {
    let app = test_app_with_expiration(1024, -120).await;
    let token = register_and_login(&app, "expired").await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn extractor_and_route_errors_use_json_envelope() {
    let app = test_app(1024).await;
    let token = register_and_login(&app, "error-envelope").await;
    let malformed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{bad json"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        malformed.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );

    let invalid_query = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?archived=not-a-bool")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_query.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        invalid_query.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );

    let not_found = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(not_found.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        not_found.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );

    let method_not_allowed = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(method_not_allowed.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(
        method_not_allowed
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap(),
        "application/json"
    );
}

#[tokio::test]
async fn ai_selection_analysis_returns_translation_terms_and_prompt_metadata() {
    let app = test_app(1024).await;
    let token = register_and_login(&app, "ai-selection").await;

    let (status, body) = json_request(
        &app,
        "POST",
        "/api/v1/ai/selection",
        json!({
            "text": "The algorithm improves neural network performance because the dataset is noisy.",
            "paragraph": "The algorithm improves neural network performance because the dataset is noisy.",
            "document_title": "Machine Learning Notes",
            "source_language": "en",
            "target_language": "zh-CN"
        }),
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["provider"], "local-deterministic");
    assert!(
        body["data"]["original_text"]
            .as_str()
            .unwrap()
            .contains("algorithm")
    );
    assert_eq!(body["data"]["prompt"]["name"], "selection_translate");
    assert!(
        body["data"]["translation"]
            .as_str()
            .unwrap()
            .contains("算法")
    );
    assert!(body["data"]["terms"].as_array().unwrap().len() >= 2);
    assert!(
        body["data"]["sentence_analysis"]["clauses"]
            .as_array()
            .unwrap()
            .len()
            >= 2
    );
}

#[tokio::test]
async fn ai_document_analysis_returns_frequent_words_and_terminology() {
    let app = test_app(1024).await;
    let token = register_and_login(&app, "ai-document").await;

    let (status, body) = json_request(
        &app,
        "POST",
        "/api/v1/ai/document",
        json!({
            "document_id": "doc-1",
            "title": "Neural Network Reading",
            "paragraphs": [
                "The neural network algorithm improves model performance.",
                "The dataset improves model evaluation and the algorithm reduces noisy features.",
                "Performance depends on dataset quality and evaluation design."
            ],
            "target_language": "zh-CN"
        }),
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["provider"], "local-deterministic");
    assert_eq!(body["data"]["prompt"]["name"], "document_summary");
    assert!(
        body["data"]["summary"]
            .as_str()
            .unwrap()
            .contains("Neural Network Reading")
    );
    assert!(body["data"]["frequent_words"].as_array().unwrap().len() >= 3);
    assert!(body["data"]["terminology"].as_array().unwrap().len() >= 2);
    assert!(body["data"]["suggestions"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn ai_selection_analysis_rejects_empty_text() {
    let app = test_app(1024).await;
    let token = register_and_login(&app, "ai-empty").await;

    let (status, body) = json_request(
        &app,
        "POST",
        "/api/v1/ai/selection",
        json!({
            "text": "   ",
            "paragraph": "Context",
            "target_language": "zh-CN"
        }),
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn ai_analysis_requires_authentication() {
    let app = test_app(1024).await;

    let (status, body) = json_request(
        &app,
        "POST",
        "/api/v1/ai/selection",
        json!({
            "text": "The algorithm improves performance.",
            "target_language": "zh-CN"
        }),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn vocabulary_review_flow_enforces_ownership_and_scheduling() {
    let app = test_app(2048).await;
    let owner = register_and_login(&app, "vocab-owner").await;
    let other = register_and_login(&app, "vocab-other").await;
    let (_, uploaded) = upload(
        &app,
        &owner,
        "vocab.txt",
        "The neural network algorithm improves performance.",
    )
    .await;
    let document_id = uploaded["data"]["id"].as_str().unwrap();
    let paragraph_id = uploaded["data"]["paragraphs"][0]["id"].as_str().unwrap();

    let (status, card) = json_request(
        &app,
        "POST",
        "/api/v1/vocabulary",
        json!({
            "document_id": document_id,
            "paragraph_id": paragraph_id,
            "term": "neural network",
            "pronunciation": "",
            "definition": "A model inspired by connected neurons.",
            "example_sentence": "The neural network algorithm improves performance.",
            "source_text": "The neural network algorithm improves performance."
        }),
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(card["success"], true);
    assert_eq!(card["data"]["term"], "neural network");
    assert_eq!(card["data"]["mastery_status"], "new");
    let vocabulary_id = card["data"]["id"].as_str().unwrap();

    let (status, _) = json_request(
        &app,
        "POST",
        "/api/v1/vocabulary",
        json!({
            "document_id": document_id,
            "paragraph_id": paragraph_id,
            "term": "neural network",
            "definition": "Duplicate card"
        }),
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (status, _) = json_request(
        &app,
        "POST",
        "/api/v1/vocabulary",
        json!({"document_id": document_id, "term": "missing definition"}),
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, list) = json_request(
        &app,
        "GET",
        "/api/v1/vocabulary?page=1&page_size=10&sort=term&order=asc",
        Value::Null,
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["data"]["total"], 1);
    assert_eq!(list["data"]["items"].as_array().unwrap().len(), 1);

    let (status, other_list) = json_request(
        &app,
        "GET",
        "/api/v1/vocabulary",
        Value::Null,
        Some(&other),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(other_list["data"]["total"], 0);

    let (status, _) = json_request(
        &app,
        "GET",
        &format!("/api/v1/vocabulary/{vocabulary_id}"),
        Value::Null,
        Some(&other),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, queue) = json_request(
        &app,
        "GET",
        "/api/v1/review/queue?limit=5",
        Value::Null,
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(queue["data"].as_array().unwrap().len(), 1);

    let (status, answer) = json_request(
        &app,
        "POST",
        "/api/v1/review/answer",
        json!({"vocabulary_id": vocabulary_id, "answer_result": "good"}),
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(answer["data"]["answer_result"], "good");
    assert!(answer["data"]["next_review_at"].is_string());

    let (status, updated) = json_request(
        &app,
        "GET",
        &format!("/api/v1/vocabulary/{vocabulary_id}"),
        Value::Null,
        Some(&owner),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["data"]["mastery_status"], "familiar");
    assert!(updated["data"]["next_review_at"].is_string());
}
#[tokio::test]
async fn migration_creates_expected_schema_and_valid_foreign_keys() {
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' ORDER BY name",
    )
    .fetch_all(&db)
    .await
    .unwrap();
    assert_eq!(
        tables,
        [
            "document_paragraphs",
            "document_tags",
            "documents",
            "highlights",
            "notes",
            "reading_progress",
            "review_answers",
            "tags",
            "users",
            "vocabulary_cards",
        ]
    );
    let violations: Vec<(String, i64, String, i64)> = sqlx::query_as("PRAGMA foreign_key_check")
        .fetch_all(&db)
        .await
        .unwrap();
    assert!(violations.is_empty());
}
