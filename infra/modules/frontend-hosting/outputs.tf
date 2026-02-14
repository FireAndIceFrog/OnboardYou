##──────────────────────────────────────────────────────────────
## frontend-hosting — outputs
##──────────────────────────────────────────────────────────────

output "bucket_name" {
  description = "S3 bucket name for frontend artefacts"
  value       = aws_s3_bucket.frontend.id
}

output "bucket_arn" {
  description = "S3 bucket ARN"
  value       = aws_s3_bucket.frontend.arn
}

output "bucket_regional_domain" {
  description = "S3 bucket regional domain name"
  value       = aws_s3_bucket.frontend.bucket_regional_domain_name
}

output "cloudfront_distribution_id" {
  description = "CloudFront distribution ID (used for cache invalidation)"
  value       = aws_cloudfront_distribution.frontend.id
}

output "cloudfront_domain_name" {
  description = "CloudFront domain name (website URL)"
  value       = aws_cloudfront_distribution.frontend.domain_name
}

output "website_url" {
  description = "Full HTTPS URL of the hosted frontend"
  value       = "https://${aws_cloudfront_distribution.frontend.domain_name}"
}
