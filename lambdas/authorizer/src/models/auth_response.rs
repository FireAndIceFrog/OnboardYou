//! IAM policy response returned to API Gateway.

use serde::Serialize;
use serde_json::{json, Value};

/// IAM policy document returned by the Lambda Authorizer.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub principal_id: String,
    pub policy_document: Value,
    pub context: Value,
}

impl AuthResponse {
    /// Build an Allow-all policy for the given API, injecting `organizationId`
    /// into the authorizer context so downstream lambdas can read it.
    pub fn allow(principal_id: &str, organization_id: &str, method_arn: &str) -> Self {
        // Widen the resource to the stage root so the cached policy covers every method.
        let resource = method_arn
            .splitn(2, "/*/")
            .next()
            .map(|base| format!("{base}/*"))
            .unwrap_or_else(|| "*".to_string());

        Self {
            principal_id: principal_id.to_string(),
            policy_document: json!({
                "Version": "2012-10-17",
                "Statement": [{
                    "Action": "execute-api:Invoke",
                    "Effect": "Allow",
                    "Resource": resource,
                }]
            }),
            context: json!({
                "organizationId": organization_id,
                "principalId": principal_id,
            }),
        }
    }

    /// Build an explicit Deny policy.
    pub fn deny(method_arn: &str) -> Self {
        Self {
            principal_id: "unauthorized".to_string(),
            policy_document: json!({
                "Version": "2012-10-17",
                "Statement": [{
                    "Action": "execute-api:Invoke",
                    "Effect": "Deny",
                    "Resource": method_arn,
                }]
            }),
            context: json!({}),
        }
    }
}
