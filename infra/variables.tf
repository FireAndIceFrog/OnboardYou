##──────────────────────────────────────────────────────────────
## Input variables
##──────────────────────────────────────────────────────────────

variable "aws_region" {
  description = "AWS region to deploy into"
  type        = string
  default     = "eu-west-1"
}

variable "environment" {
  description = "Deployment environment (dev / staging / prod)"
  type        = string
  default     = "dev"
}

variable "config_table_name" {
  description = "DynamoDB table name for pipeline configs"
  type        = string
  default     = "PipelineConfigs"
}

variable "log_retention_days" {
  description = "CloudWatch log retention in days"
  type        = number
  default     = 14
}

# ── Cognito (authorizer) ─────────────────────────────────────

variable "cognito_user_pool_id" {
  description = "Cognito User Pool ID for JWT validation (leave empty in dev — AUTH_DEV_MODE takes over)"
  type        = string
  default     = ""
}

variable "cognito_client_id" {
  description = "Cognito App Client ID for audience validation"
  type        = string
  default     = ""
}
