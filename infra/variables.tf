##──────────────────────────────────────────────────────────────
## Input variables
##──────────────────────────────────────────────────────────────

variable "aws_region" {
  description = "AWS region to deploy into"
  type        = string
  default     = "eu-east-1"
}

variable "environment" {
  description = "Deployment environment (dev / staging / prod)"
  type        = string
  default     = "dev"
}

variable "log_retention_days" {
  description = "CloudWatch log retention in days"
  type        = number
  default     = 14
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