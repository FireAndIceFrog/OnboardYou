##──────────────────────────────────────────────────────────────
## Lambda module — outputs
##──────────────────────────────────────────────────────────────

output "arn" {
  description = "Lambda function ARN"
  value       = aws_lambda_function.this.arn
}

output "function_name" {
  description = "Lambda function name"
  value       = aws_lambda_function.this.function_name
}

output "invoke_arn" {
  description = "Lambda invoke ARN (for API Gateway integration)"
  value       = aws_lambda_function.this.invoke_arn
}

output "role_arn" {
  description = "IAM execution role ARN"
  value       = aws_iam_role.this.arn
}

output "role_name" {
  description = "IAM execution role name"
  value       = aws_iam_role.this.name
}
