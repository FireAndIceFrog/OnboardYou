##──────────────────────────────────────────────────────────────
## Input variables
##──────────────────────────────────────────────────────────────

variable "aws_region" {
  description = "AWS region to deploy into"
  type        = string
  default     = "eu-east-1"
}

variable "environment" {
  description = "Deployment environment: local (CORS=*), staging (GitHub Pages), prod (S3+CloudFront)"
  type        = string
  default     = "staging"

  validation {
    condition     = contains(["local", "staging", "prod"], var.environment)
    error_message = "environment must be one of: local, staging, prod"
  }
}

variable "log_retention_days" {
  description = "CloudWatch log retention in days"
  type        = number
  default     = 14
}

variable "csv_upload_retention_days" {
  description = "Days before uploaded objects in the CSV upload bucket expire"
  type        = number
  default     = 90
}

variable "env_postfix" {
  description = "Unique postfix appended to all resource names (avoids naming collisions on destroy/recreate)"
  type        = string
}

variable "tags" {
  description = "Tags applied to every resource via the AWS provider default_tags"
  type        = map(string)
  default     = {}
}

variable "demo_users" {
  description = "Demo users provisioned on every deploy (password auto-rotated)"
  type = list(object({
    email           = string
    organization_id = optional(string, "demo-org")
  }))
  default = []
}

variable "gh_token" {
  description = "GitHub token with access to GH Models API"
  type        = string
  sensitive = true
}

variable "github_pages_url" {
  description = "GitHub Pages URL used as the frontend origin when environment = staging"
  type        = string
  default     = "https://fireandicefrog.github.io"
}

variable "supabase_access_token" {
  description = "Supabase access token for provisioning and seeding the database"
  type        = string
  sensitive   = true
}

variable "supabase_organization_id" {
  description = "Slug for the Supabase organization (found in dashboard URL)"
  type        = string
}}

# ── Email Ingestion ────────────────────────────────────────────

variable "email_ingestion_domain" {
  description = "Domain to register with SES for inbound email ingestion (e.g. 'onboard.acme.com')"
  type        = string
  default     = ""
}
