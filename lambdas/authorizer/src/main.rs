//! Authorizer Lambda
//!
//! API Gateway TOKEN authorizer. In dev mode every request is allowed;
//! in production it validates a Cognito JWT and injects the verified
//! `organizationId` into the API Gateway request context.

mod engine;
mod models;
mod repositories;

use engine::auth_engine::{self, AuthConfig};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use models::{AuthEvent, AuthResponse};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let config = AuthConfig::from_env();

    lambda_runtime::run(service_fn(|event: LambdaEvent<AuthEvent>| {
        let config = &config;
        async move { handler(config, event).await }
    }))
    .await
}

// ── Controller ──────────────────────────────────────────────

/// Receives the authorizer event, delegates to the engine, and returns
/// the IAM policy response.
async fn handler(
    config: &AuthConfig,
    event: LambdaEvent<AuthEvent>,
) -> Result<AuthResponse, Error> {
    let (payload, _ctx) = event.into_parts();

    match auth_engine::authorize(config, &payload).await {
        Ok(response) => Ok(response),
        Err(e) => {
            tracing::warn!(error = %e, "Authorization denied");
            let arn = payload.method_arn.as_deref().unwrap_or("*");
            Ok(AuthResponse::deny(arn))
        }
    }
}
