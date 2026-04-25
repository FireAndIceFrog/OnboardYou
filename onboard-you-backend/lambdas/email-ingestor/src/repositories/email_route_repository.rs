use async_trait::async_trait;
use aws_sdk_dynamodb::{types::AttributeValue, Client as DynamoClient};
use std::sync::Arc;

use crate::models::email_route::EmailRoute;

/// Abstraction over the `EmailRoutes` DynamoDB table.
#[async_trait]
pub trait IEmailRouteRepo: Send + Sync {
    async fn lookup(&self, table: &str, sender: &str) -> Result<Option<EmailRoute>, String>;
}

/// DynamoDB-backed implementation of [`IEmailRouteRepo`].
pub struct EmailRouteRepository {
    dynamo: DynamoClient,
}

impl EmailRouteRepository {
    pub fn new(dynamo: DynamoClient) -> Arc<Self> {
        Arc::new(Self { dynamo })
    }
}

#[async_trait]
impl IEmailRouteRepo for EmailRouteRepository {
    async fn lookup(&self, table: &str, sender: &str) -> Result<Option<EmailRoute>, String> {
        let resp = self
            .dynamo
            .get_item()
            .table_name(table)
            .key("sender_email", AttributeValue::S(sender.to_lowercase()))
            .send()
            .await
            .map_err(|e| format!("DynamoDB GetItem failed: {e}"))?;

        let Some(item) = resp.item else {
            return Ok(None);
        };

        let org_id = item
            .get("org_id")
            .and_then(|v| v.as_s().ok())
            .map(String::from)
            .ok_or_else(|| "EmailRoute missing org_id".to_string())?;

        let company_id = item
            .get("company_id")
            .and_then(|v| v.as_s().ok())
            .map(String::from)
            .ok_or_else(|| "EmailRoute missing company_id".to_string())?;

        Ok(Some(EmailRoute { org_id, company_id }))
    }
}

