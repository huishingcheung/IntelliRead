pub mod ai;
pub mod annotations;
pub mod auth;
pub mod config;
pub mod database;
pub mod documents;
pub mod error;
pub mod reading;
pub mod response;
pub mod state;
pub mod statistics;
pub mod tags;

use std::sync::Arc;

use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post, put},
};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(paths(health, auth::register, auth::login, documents::import, documents::list, documents::detail, documents::update, documents::delete, reading::get, reading::update, tags::create, tags::list, tags::update, tags::delete, tags::set_document_tags, tags::get_document_tags, annotations::create_note, annotations::list_notes, annotations::update_note, annotations::delete_note, annotations::create_highlight, annotations::list_highlights, annotations::update_highlight, annotations::delete_highlight, statistics::overview, ai::analyze_selection, ai::analyze_document), components(schemas(HealthResponse, auth::RegisterRequest, auth::LoginRequest, auth::UserResponse, auth::LoginResponse, documents::DocumentSummary, documents::Paragraph, documents::DocumentDetail, documents::DocumentList, documents::ImportDocumentForm, documents::UpdateDocumentRequest, reading::UpdateProgressRequest, reading::ReadingProgress, tags::Tag, tags::TagRequest, tags::SetDocumentTagsRequest, annotations::Note, annotations::CreateNoteRequest, annotations::UpdateNoteRequest, annotations::Highlight, annotations::CreateHighlightRequest, annotations::UpdateHighlightRequest, statistics::LearningOverview, ai::SelectionAnalysisRequest, ai::DocumentAnalysisRequest, ai::PromptInfo, ai::TermInfo, ai::ClauseAnalysis, ai::SentenceAnalysis, ai::SelectionAnalysisResponse, ai::FrequentWord, ai::DocumentAnalysisResponse, response::ErrorBody, response::ErrorDetail)), modifiers(&SecurityAddon), tags((name = "IntelliRead", description = "IntelliRead backend API")))]
struct ApiDoc;

#[derive(serde::Serialize, utoipa::ToSchema)]
struct HealthResponse {
    status: &'static str,
}

struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

pub fn app(state: Arc<AppState>) -> Router {
    let allowed_origins: Vec<axum::http::HeaderValue> = state
        .config
        .cors_allowed_origins
        .iter()
        .map(|origin| {
            origin
                .parse()
                .expect("validated CORS origin must be a valid header value")
        })
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ]);
    let api = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/documents", post(documents::import).get(documents::list))
        .route(
            "/documents/{id}",
            get(documents::detail)
                .patch(documents::update)
                .delete(documents::delete),
        )
        .route(
            "/documents/{id}/progress",
            get(reading::get).put(reading::update),
        )
        .route(
            "/documents/{id}/tags",
            get(tags::get_document_tags).put(tags::set_document_tags),
        )
        .route(
            "/documents/{id}/notes",
            get(annotations::list_notes).post(annotations::create_note),
        )
        .route(
            "/documents/{id}/highlights",
            get(annotations::list_highlights).post(annotations::create_highlight),
        )
        .route("/tags", get(tags::list).post(tags::create))
        .route("/tags/{id}", put(tags::update).delete(tags::delete))
        .route(
            "/notes/{id}",
            put(annotations::update_note).delete(annotations::delete_note),
        )
        .route(
            "/highlights/{id}",
            put(annotations::update_highlight).delete(annotations::delete_highlight),
        )
        .route("/ai/selection", post(ai::analyze_selection))
        .route("/ai/document", post(ai::analyze_document))
        .route("/statistics/overview", get(statistics::overview));
    Router::new()
        .nest("/api/v1", api)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[utoipa::path(get, path = "/api/v1/health", responses((status = 200, body = HealthResponse)))]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn not_found() -> crate::error::AppError {
    crate::error::AppError::NotFound
}

async fn method_not_allowed() -> (StatusCode, Json<crate::response::ErrorBody>) {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        Json(crate::response::ErrorBody {
            success: false,
            error: crate::response::ErrorDetail {
                code: "METHOD_NOT_ALLOWED",
                message: "method not allowed".into(),
            },
        }),
    )
}
