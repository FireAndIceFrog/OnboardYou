//! JWT claims extracted from the API Gateway authorizer context.
//!
//! The Lambda authorizer validates the Cognito JWT and injects
//! `organizationId` into the API Gateway request context.  This
//! extractor pulls it out so handlers never touch raw headers.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use lambda_http::request::RequestContext;

use crate::models::ApiError;

/// Claims extracted from the authorizer context.
///
/// Axum handlers can add `claims: Claims` to their signature and the
/// `organization_id` will be resolved automatically from the Lambda
/// authorizer context injected by API Gateway.
#[derive(Debug, Clone)]
pub struct Claims {
    /// The organization ID from the `custom:organizationId` Cognito claim,
    /// forwarded by the Lambda authorizer.
    pub organization_id: String,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let ctx = parts
                .extensions
                .get::<RequestContext>()
                .ok_or_else(|| {
                    ApiError::Unauthorized("Missing API Gateway request context".into())
                })?;

            let organization_id = match ctx {
                RequestContext::ApiGatewayV1(gw) => gw
                    .authorizer
                    .fields
                    .get("organizationId")
                    .and_then(|v: &serde_json::Value| v.as_str())
                    .map(|s: &str| s.to_string()),
                _ => None,
            };

            let organization_id = organization_id.ok_or_else(|| {
                ApiError::Unauthorized("organizationId not found in authorizer context".into())
            })?;

            Ok(Claims { organization_id })
        }
    }
}
