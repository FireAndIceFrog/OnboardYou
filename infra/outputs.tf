##──────────────────────────────────────────────────────────────
## Outputs
##──────────────────────────────────────────────────────────────

output "api_url" {
  description = "Config API base URL"
  value       = "${aws_api_gateway_stage.v1.invoke_url}"
}

output "api_endpoint_example" {
  description = "Example: POST a pipeline config"
  value       = "curl -X POST ${aws_api_gateway_stage.v1.invoke_url}/{organizationId}/config -d '{...}'"
}

output "config_table_name" {
  description = "DynamoDB table for pipeline configs"
  value       = aws_dynamodb_table.pipeline_configs.name
}

output "config_table_arn" {
  description = "DynamoDB table ARN"
  value       = aws_dynamodb_table.pipeline_configs.arn
}

output "config_api_lambda_arn" {
  description = "Config API Lambda ARN"
  value       = aws_lambda_function.config_api.arn
}

output "etl_trigger_lambda_arn" {
  description = "ETL Trigger Lambda ARN"
  value       = aws_lambda_function.etl_trigger.arn
}

output "scheduler_role_arn" {
  description = "EventBridge Scheduler execution role ARN"
  value       = aws_iam_role.scheduler_execution.arn
}
