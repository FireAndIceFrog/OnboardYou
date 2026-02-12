##──────────────────────────────────────────────────────────────
## API Gateway module — outputs
##──────────────────────────────────────────────────────────────

output "api_id" {
  description = "REST API ID"
  value       = aws_api_gateway_rest_api.this.id
}

output "execution_arn" {
  description = "REST API execution ARN (for IAM / permissions)"
  value       = aws_api_gateway_rest_api.this.execution_arn
}

output "invoke_url" {
  description = "Stage invoke URL (base)"
  value       = aws_api_gateway_stage.this.invoke_url
}

output "stage_name" {
  description = "Deployed stage name"
  value       = aws_api_gateway_stage.this.stage_name
}
