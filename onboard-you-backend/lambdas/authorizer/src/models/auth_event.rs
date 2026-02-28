//! Incoming event from API Gateway (TOKEN authorizer).

use serde::Deserialize;

/// API Gateway Token Authorizer request event.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthEvent {
    /// The "Bearer <jwt>" value from the Authorization header.
    pub authorization_token: Option<String>,
    /// The ARN of the API Gateway method being invoked.
    pub method_arn: Option<String>,
}
