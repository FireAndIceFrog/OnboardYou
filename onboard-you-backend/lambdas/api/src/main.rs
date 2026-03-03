//! Config API Lambda
//!
//! Bootstrap + route declarations. Read the router() function to know what this API does.
//! Serves an OpenAPI specification at `/swagger-ui` for interactive API documentation.

mod controllers;
mod dependancies;
mod engine;
mod models;
mod repositories;

use axum::{
    http::HeaderValue,
    routing::{get, post},
    Router,
};
use controllers::login;
use controllers::{
    create_config, delete_config, get_config, list_configs, update_config, validate_config,
};
use controllers::{csv_columns, csv_presigned_upload};
use controllers::{get_settings, upsert_settings};
use dependancies::Dependancies;
use models::{
    ConfigRequest, CsvColumnsResponse, ErrorResponse, LoginRequest, LoginResponse,
    PresignedUploadResponse, SettingsRequest, StepValidation, ValidationResult,
};
use onboard_you_models::{
    ActionConfig, ActionConfigPayload, ActionType, ApiDispatcherConfig, BearerPlacement,
    BearerRepoConfig, ColumnMapping, Manifest, OAuth2GrantType, OAuth2RepoConfig, OAuthRepoConfig, OrgSettings, SchemaDiff,
    PipelineConfig,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{fmt, EnvFilter};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
        controllers::config_controller::delete_config,
        controllers::config_controller::validate_config,
        controllers::csv_upload_controller::csv_presigned_upload,
        controllers::csv_upload_controller::csv_columns,
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
        ActionConfigPayload,
        ActionType,
        ApiDispatcherConfig,
        BearerRepoConfig,
        BearerPlacement,
        OAuthRepoConfig,
        OAuth2RepoConfig,
        OAuth2GrantType,
        ValidationResult,
        StepValidation,
        SchemaDiff,
        ColumnMapping,
        ErrorResponse,
        OrgSettings,
        SettingsRequest,
        PresignedUploadResponse,
        CsvColumnsResponse,
    )),
    tags(
        (name = "Authentication", description = "Login and token management"),
        (name = "Configuration", description = "Pipeline configuration CRUD operations"),
        (name = "Validation", description = "Dry-run pipeline validation"),
        (name = "CSV Upload", description = "CSV file upload and column discovery"),
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

    let state = Dependancies::new(Dependancies::create_env()).await;
    let app = router(state);

    lambda_http::run(app).await
}

// ── Routes ──────────────────────────────────────────────────

fn router(state: Dependancies) -> Router {
    // determine allowed origin from environment; fall back to Any (for local dev).
    let allowed_origin = std::env::var("FRONTEND_URL")
        .ok()
        .filter(|s| !s.is_empty());
    let mut cors_builder = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any);

    match allowed_origin.as_deref() {
        // "*" or missing → allow all origins
        Some("*") | None => {
            cors_builder = cors_builder.allow_origin(Any);
        }
        Some(origin) => {
            if let Ok(val) = origin.parse::<HeaderValue>() {
                cors_builder = cors_builder.allow_origin(val);
            } else {
                cors_builder = cors_builder.allow_origin(Any);
            }
        }
    }
    let cors = cors_builder;

    Router::new()
        .route("/auth/login", post(login))
        .route("/config", get(list_configs))
        .route(
            "/config/{customer_company_id}",
            get(get_config)
                .post(create_config)
                .put(update_config)
                .delete(delete_config),
        )
        .route(
            "/config/{customer_company_id}/validate",
            post(validate_config),
        )
        .route(
            "/config/{customer_company_id}/csv-upload",
            post(csv_presigned_upload),
        )
        .route(
            "/config/{customer_company_id}/csv-columns",
            get(csv_columns),
        )
        .route("/settings", get(get_settings).put(upsert_settings))
        .with_state(state)
        .layer(cors)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
