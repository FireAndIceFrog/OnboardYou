//! Config API Lambda
//!
//! Bootstrap + route declarations. Read the router() function to know what this API does.
//! Serves an OpenAPI specification at `/swagger-ui` for interactive API documentation.

mod controllers;
mod engine;
mod models;
mod repositories;

use axum::{
    routing::{get, post},
    Router,
};
use controllers::{create_config, get_config, update_config, validate_config};
use models::{AppState, ErrorResponse, PipelineConfig};
use tracing_subscriber::{fmt, EnvFilter};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use engine::validation_engine::{StepValidation, ValidationResult};
use onboard_you::{ActionConfig, Manifest};

/// OpenAPI documentation for the OnboardYou Config API.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "OnboardYou Config API",
        version = "1.0.0",
        description = "REST API for managing OnboardYou ETL pipeline configurations.\n\nSupports CRUD operations on pipeline configs stored in DynamoDB and \ndry-run validation of pipeline manifests via column propagation.",
        license(name = "Proprietary"),
    ),
    paths(
        controllers::config_controller::get_config,
        controllers::config_controller::create_config,
        controllers::config_controller::update_config,
        controllers::config_controller::validate_config,
    ),
    components(schemas(
        PipelineConfig,
        Manifest,
        ActionConfig,
        ValidationResult,
        StepValidation,
        ErrorResponse,
    )),
    tags(
        (name = "Configuration", description = "Pipeline configuration CRUD operations"),
        (name = "Validation", description = "Dry-run pipeline validation"),
    )
)]
struct ApiDoc;

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
            "/{organization_id}/{customer_company_id}/config",
            get(get_config).post(create_config).put(update_config),
        )
        .route(
            "/{organization_id}/{customer_company_id}/config/validate",
            post(validate_config),
        )
        .with_state(state)
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
}
