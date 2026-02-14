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

output "config_table_name" {
  description = "DynamoDB table for pipeline configs"
  value       = module.pipeline_configs_table.name
}

output "config_table_arn" {
  description = "DynamoDB table ARN"
  value       = module.pipeline_configs_table.arn
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
  value       = module.frontend.bucket_name
}

output "frontend_cloudfront_id" {
  description = "CloudFront distribution ID (for cache invalidation)"
  value       = module.frontend.cloudfront_distribution_id
}

output "frontend_url" {
  description = "Frontend website URL"
  value       = module.frontend.website_url
}
