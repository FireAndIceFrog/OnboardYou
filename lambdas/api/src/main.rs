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
use tower_http::cors::{Any, CorsLayer};
use controllers::{create_config, get_config, list_configs, update_config, validate_config};
use controllers::{get_settings, upsert_settings};
use controllers::login;
use models::{AppState, ConfigRequest, ErrorResponse, LoginRequest, LoginResponse, OrgSettings, PipelineConfig, SettingsRequest};
use tracing_subscriber::{fmt, EnvFilter};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use engine::validation_engine::{StepValidation, ValidationResult};
use onboard_you::{ActionConfig, ActionType, Manifest};

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
        controllers::auth_controller::login,
        controllers::config_controller::list_configs,
        controllers::config_controller::get_config,
        controllers::config_controller::create_config,
        controllers::config_controller::update_config,
        controllers::config_controller::validate_config,
        controllers::settings_controller::get_settings,
        controllers::settings_controller::upsert_settings,
    ),
    components(schemas(
        LoginRequest,
        LoginResponse,
        PipelineConfig,
        ConfigRequest,
        Manifest,
        ActionConfig,
        ActionType,
        ValidationResult,
        StepValidation,
        ErrorResponse,
        OrgSettings,
        SettingsRequest,
    )),
    tags(
        (name = "Authentication", description = "Login and token management"),
        (name = "Configuration", description = "Pipeline configuration CRUD operations"),
        (name = "Validation", description = "Dry-run pipeline validation"),
        (name = "Settings", description = "Organization settings management"),
    ),
    security(
        ("bearer" = []),
    ),
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

/// Adds the Bearer security scheme to the OpenAPI document.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::HttpBuilder::new()
                    .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some(
                        "Cognito JWT — the `custom:organizationId` claim is extracted \
                         by the Lambda authorizer and injected into the request context.",
                    ))
                    .build(),
            ),
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    // Quick escape hatch: `config-api --openapi` dumps the spec to stdout.
    if std::env::args().any(|a| a == "--openapi") {
        println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());
        return Ok(());
    }

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
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/auth/login", post(login))
        .route("/config", get(list_configs))
        .route(
            "/config/{customer_company_id}",
            get(get_config).post(create_config).put(update_config),
        )
        .route(
            "/config/{customer_company_id}/validate",
            post(validate_config),
        )
        .route("/settings", get(get_settings).put(upsert_settings))
        .with_state(state)
        .layer(cors)
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
}
