##──────────────────────────────────────────────────────────────
## CSV Upload Bucket — variables
##──────────────────────────────────────────────────────────────

variable "project_prefix" {
  description = "Naming prefix (e.g. 'onboardyou')"
  type        = string
}

variable "environment" {
  description = "Deployment environment (dev / staging / prod)"
  type        = string
}

variable "env_postfix" {
  description = "Unique postfix for resource names"
  type        = string
}

variable "allowed_origins" {
  description = "CORS allowed origins for presigned uploads"
  type        = list(string)
  default     = ["*"]
}

variable "retention_days" {
  description = "Days before uploaded CSV objects expire"
  type        = number
  default     = 90
}
