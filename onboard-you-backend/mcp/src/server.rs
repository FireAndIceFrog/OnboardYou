use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
    handler::server::wrapper::Parameters,
};

use crate::api::ApiClient;
use crate::models::{ConfigRequest, CreateConfigArgs, ValidateConfigArgs, SaveConfigArgs, FetchConfigArgs, GetSchemaArgs};
use crate::models::schema::{SchemaResource, build_schema_resources};

/* ── Server struct ────────────────────────────────────────── */

#[derive(Clone)]
pub struct OnboardYouMcp {
    api: Arc<ApiClient>,
    tool_router: ToolRouter<Self>,
    schemas: Arc<Vec<SchemaResource>>,
}

#[tool_router]
impl OnboardYouMcp {
    pub fn new(api: ApiClient) -> Self {
        Self {
            api: Arc::new(api),
            tool_router: Self::tool_router(),
            schemas: Arc::new(build_schema_resources()),
        }
    }

    /// Create a new pipeline configuration.
    ///
    /// The config must include: `name` (string), `cron` (schedule expression),
    /// and `pipeline` (the ETL manifest with ingress/actions/egress).
    #[tool(name = "create_config", description = "Create a new pipeline configuration for a customer company")]
    async fn create_config(
        &self,
        Parameters(args): Parameters<CreateConfigArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate against shared models before calling the API
        let _: ConfigRequest = serde_json::from_value(args.config.clone())
            .map_err(|e| McpError::invalid_params(format!("invalid config: {e}"), None))?;
        let body = self
            .api
            .post(&format!("/config/{}", args.customer_company_id), &args.config)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&body).unwrap_or_default(),
        )]))
    }

    /// Dry-run validate a pipeline configuration.
    ///
    /// Parses the pipeline manifest, builds every action, and folds
    /// `calculate_columns` through the chain. Returns per-step column
    /// snapshots without executing any real transformations.
    #[tool(name = "validate_config", description = "Dry-run validate a pipeline configuration (column propagation check)")]
    async fn validate_config(
        &self,
        Parameters(args): Parameters<ValidateConfigArgs>,
    ) -> Result<CallToolResult, McpError> {
        let _: ConfigRequest = serde_json::from_value(args.config.clone())
            .map_err(|e| McpError::invalid_params(format!("invalid config: {e}"), None))?;
        let body = self
            .api
            .post(
                &format!("/config/{}/validate", args.customer_company_id),
                &args.config,
            )
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&body).unwrap_or_default(),
        )]))
    }

    /// List all pipeline configurations for the caller's organization.
    ///
    /// Returns only `customerCompanyId` and `name` for each config to keep
    /// the response lightweight. Use `fetch_config` to retrieve the full
    /// pipeline manifest for a specific company.
    #[tool(name = "list_configs", description = "List all pipeline configurations (company IDs and names only)")]
    async fn list_configs(&self) -> Result<CallToolResult, McpError> {
        let body = self
            .api
            .get("/config")
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Extract only customerCompanyId + name to avoid overloading context
        let slim: Vec<serde_json::Value> = match &body {
            serde_json::Value::Array(items) => items
                .iter()
                .map(|item| {
                    serde_json::json!({
                        "customerCompanyId": item.get("customerCompanyId"),
                        "name": item.get("name"),
                    })
                })
                .collect(),
            other => vec![other.clone()],
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&slim).unwrap_or_default(),
        )]))
    }

    /// Fetch the full pipeline configuration for a specific customer company.
    ///
    /// Use `list_configs` first to discover available `customerCompanyId` values.
    #[tool(name = "fetch_config", description = "Fetch the full pipeline configuration for a given customerCompanyId")]
    async fn fetch_config(
        &self,
        Parameters(args): Parameters<FetchConfigArgs>,
    ) -> Result<CallToolResult, McpError> {
        let body = self
            .api
            .get(&format!("/config/{}", args.customer_company_id))
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&body).unwrap_or_default(),
        )]))
    }

    /// Save (update) an existing pipeline configuration.
    ///
    /// Validates the pipeline manifest, persists to DynamoDB, and updates
    /// the EventBridge schedule.
    #[tool(name = "save_config", description = "Save (update) an existing pipeline configuration")]
    async fn save_config(
        &self,
        Parameters(args): Parameters<SaveConfigArgs>,
    ) -> Result<CallToolResult, McpError> {
        let _: ConfigRequest = serde_json::from_value(args.config.clone())
            .map_err(|e| McpError::invalid_params(format!("invalid config: {e}"), None))?;
        let body = self
            .api
            .put(&format!("/config/{}", args.customer_company_id), &args.config)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&body).unwrap_or_default(),
        )]))
    }

    /// Retrieve pipeline-config JSON schemas.
    ///
    /// Call with no `name` (or empty) to list available schema names.
    /// Call with a specific `name` (e.g. "Manifest") to get the full JSON Schema.
    #[tool(name = "get_schema", description = "List available JSON schemas or retrieve a specific schema by name")]
    async fn get_schema(
        &self,
        Parameters(args): Parameters<GetSchemaArgs>,
    ) -> Result<CallToolResult, McpError> {
        match args.name.as_deref() {
            None | Some("") => {
                // List all available schema names + descriptions
                let listing: Vec<serde_json::Value> = self
                    .schemas
                    .iter()
                    .map(|s| serde_json::json!({ "name": s.name, "description": s.description }))
                    .collect();
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&listing).unwrap_or_default(),
                )]))
            }
            Some(name) => {
                let res = self
                    .schemas
                    .iter()
                    .find(|s| s.name.eq_ignore_ascii_case(name))
                    .ok_or_else(|| {
                        McpError::invalid_params(
                            format!("unknown schema: {name}. Call get_schema with no name to list available schemas."),
                            None,
                        )
                    })?;
                Ok(CallToolResult::success(vec![Content::text(&res.text)]))
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for OnboardYouMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("onboardyou-mcp", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "OnboardYou MCP server.\n\
             Tools: list_configs, fetch_config, create_config, validate_config, save_config, get_schema.\n\
             Workflow: call list_configs first to see available companies, then fetch_config to get the full manifest before editing.\n\
             Call get_schema to understand the expected JSON shapes before creating or editing configs."
                .to_string(),
        )
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async move {
            let resources = self
                .schemas
                .iter()
                .map(|s| {
                    Annotated::new(
                        RawResource::new(&s.uri, s.name)
                            .with_description(s.description)
                            .with_mime_type("application/json"),
                        None,
                    )
                })
                .collect();
            Ok(ListResourcesResult::with_all_items(resources))
        }
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move {
            let uri = &request.uri;
            let res = self
                .schemas
                .iter()
                .find(|s| s.uri == uri.as_str())
                .ok_or_else(|| {
                    McpError::invalid_params(format!("unknown resource: {uri}"), None)
                })?;
            Ok(ReadResourceResult::new(vec![ResourceContents::text(
                &res.text, uri,
            )]))
        }
    }
}
