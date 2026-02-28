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
        // ARN format: arn:aws:execute-api:{region}:{account}:{api-id}/{stage}/{method}/{resource}
        // We keep everything up to and including {stage}/ then wildcard the rest.
        let resource = {
            let parts: Vec<&str> = method_arn.splitn(2, ':').collect();
            if method_arn.contains("execute-api") {
                // Split on '/' and take the first 6 colon-delimited + 2 slash-delimited segments
                // i.e. everything through {api-id}/{stage}/*
                let segments: Vec<&str> = method_arn.split('/').collect();
                if segments.len() >= 2 {
                    format!("{}/{}/*", segments[0], segments[1])
                } else {
                    format!("{method_arn}/*")
                }
            } else {
                let _ = parts;
                "*".to_string()
            }
        };

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
