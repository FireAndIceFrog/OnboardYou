##──────────────────────────────────────────────────────────────
## CSV Upload Bucket — outputs
##──────────────────────────────────────────────────────────────

output "bucket_name" {
  description = "Name of the CSV upload bucket"
  value       = aws_s3_bucket.csv_uploads.bucket
}

output "bucket_arn" {
  description = "ARN of the CSV upload bucket"
  value       = aws_s3_bucket.csv_uploads.arn
}

output "bucket_domain_name" {
  description = "Domain name of the CSV upload bucket"
  value       = aws_s3_bucket.csv_uploads.bucket_regional_domain_name
}
