##──────────────────────────────────────────────────────────────
## Outputs
##──────────────────────────────────────────────────────────────

output "api_url" {
  description = "Config API base URL"
  value       = module.api.invoke_url
}

output "api_endpoint_example" {
  description = "Example: list all pipeline configs"
  value       = "curl -H 'Authorization: Bearer <jwt>' ${module.api.invoke_url}/config"
}

output "login_endpoint_example" {
  description = "Example: authenticate with email + password"
  value       = "curl -X POST ${module.api.invoke_url}/auth/login -H 'Content-Type: application/json' -d '{\"email\":\"user@example.com\",\"password\":\"…\"}'"
}

# ── Demo user credentials (rotated every deploy) ─────────────

output "demo_credentials" {
  description = "Map of email → password for every demo user (rotated every deploy)"
  value       = var.environment == "local" ? module.demo_user[0].credentials : null
  sensitive   = true
}

output "config_table_name" {
  description = "DynamoDB table for pipeline configs"
  value       = module.pipeline_configs_table.name
}

output "config_table_arn" {
  description = "DynamoDB table ARN"
  value       = module.pipeline_configs_table.arn
}

output "settings_table_name" {
  description = "DynamoDB table for organization settings"
  value       = module.org_settings_table.name
}

output "settings_table_arn" {
  description = "DynamoDB OrgSettings table ARN"
  value       = module.org_settings_table.arn
}

output "config_api_lambda_arn" {
  description = "Config API Lambda ARN"
  value       = module.config_api.arn
}

output "etl_trigger_lambda_arn" {
  description = "ETL Trigger Lambda ARN"
  value       = module.etl_trigger.arn
}

output "scheduler_role_arn" {
  description = "EventBridge Scheduler execution role ARN"
  value       = aws_iam_role.scheduler_execution.arn
}

# ── Frontend Hosting ──────────────────────────────────────────

output "frontend_bucket_name" {
  description = "S3 bucket for frontend build artefacts"
  value       = var.environment == "prod" ? module.frontend[0].bucket_name : null
}

output "frontend_cloudfront_id" {
  description = "CloudFront distribution ID (for cache invalidation)"
  value       = var.environment == "prod" ? module.frontend[0].cloudfront_distribution_id : null
}

output "frontend_url" {
  description = "Frontend website URL (CloudFront when prod, GitHub Pages otherwise)"
  value       = local.frontend_url
}

output "frontend_hosting_mode" {
  description = "Where the frontend is hosted: 'github-pages' or 's3-cloudfront'"
  value       = var.environment == "prod" ? "s3-cloudfront" : "github-pages"
}

output "db_connection_string_pooler" {
  description = "Supabase postgres connection string for lambdas (pooler)."
  value       = module.database.connection_string_pooler
  sensitive   = true
}

output "db_connection_string_direct" {
  description = "Direct Supabase postgres connection string (for migrations)."
  value       = module.database.connection_string_direct
  sensitive   = true
}

output "db_password" {
  description = "Generated postgres password (sensitive)"
  value       = module.database.database_password
  sensitive   = true
}
