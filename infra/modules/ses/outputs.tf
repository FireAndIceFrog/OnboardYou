##──────────────────────────────────────────────────────────────
## SES module — outputs
##──────────────────────────────────────────────────────────────

output "ses_inbox_bucket_id" {
  description = "Name / ID of the ses-inbox S3 bucket."
  value       = aws_s3_bucket.ses_inbox.id
}

output "ses_inbox_bucket_arn" {
  description = "ARN of the ses-inbox S3 bucket."
  value       = aws_s3_bucket.ses_inbox.arn
}

output "email_routes_table_name" {
  description = "Name of the EmailRoutes DynamoDB table."
  value       = module.email_routes_table.name
}

output "email_routes_table_arn" {
  description = "ARN of the EmailRoutes DynamoDB table."
  value       = module.email_routes_table.arn
}
