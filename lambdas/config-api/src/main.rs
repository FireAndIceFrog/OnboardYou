//! Config API Lambda
//!
//! Bootstrap + route declarations. Read the router() function to know what this API does.

mod engine;
mod models;
mod repositories;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use models::{AppState, PipelineConfig};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let state = AppState::from_env().await;
    let app = router(state);

    lambda_http::run(app).await
}

// ── Routes ──────────────────────────────────────────────────

fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/{organization_id}/config",
            get(get_config).post(upsert_config).put(upsert_config),
        )
        .with_state(state)
}

// ── Handlers ────────────────────────────────────────────────

/// GET /{organizationId}/config
async fn get_config(
    State(state): State<AppState>,
    Path(organization_id): Path<String>,
) -> Result<impl IntoResponse, models::ApiError> {
    let config = engine::config_engine::get(&state, &organization_id).await?;
    Ok(Json(config))
}

/// POST/PUT /{organizationId}/config
async fn upsert_config(
    State(state): State<AppState>,
    Path(organization_id): Path<String>,
    Json(body): Json<PipelineConfig>,
) -> Result<impl IntoResponse, models::ApiError> {
    let saved = engine::config_engine::upsert(&state, &organization_id, body).await?;
    Ok((StatusCode::OK, Json(saved)))
}
