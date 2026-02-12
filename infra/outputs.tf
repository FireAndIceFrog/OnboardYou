##──────────────────────────────────────────────────────────────
## Outputs
##──────────────────────────────────────────────────────────────

output "api_url" {
  description = "Config API base URL"
  value       = module.api.invoke_url
}

output "api_endpoint_example" {
  description = "Example: POST a pipeline config"
  value       = "curl -X POST ${module.api.invoke_url}/{organizationId}/config -d '{...}'"
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
