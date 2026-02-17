##──────────────────────────────────────────────────────────────
## Lambda module — variables
##──────────────────────────────────────────────────────────────

variable "project_prefix" {
  description = "Naming prefix (e.g. 'onboardyou')"
  type        = string
}

variable "function_name" {
  description = "Short function name (e.g. 'config-api'). Full name = {prefix}-{name}-{env}"
  type        = string
}

variable "description" {
  description = "Lambda function description"
  type        = string
  default     = ""
}

variable "environment" {
  description = "Deployment environment (dev / staging / prod)"
  type        = string
}

variable "source_binary" {
  description = "Absolute path to the compiled bootstrap binary"
  type        = string
}

variable "memory_size" {
  description = "Lambda memory in MB"
  type        = number
  default     = 256
}

variable "timeout" {
  description = "Lambda timeout in seconds"
  type        = number
  default     = 30
}

variable "architectures" {
  description = "Lambda CPU architectures"
  type        = list(string)
  default     = ["x86_64"]
}

variable "runtime" {
  description = "Lambda runtime"
  type        = string
  default     = "provided.al2023"
}

variable "handler" {
  description = "Lambda handler"
  type        = string
  default     = "bootstrap"
}

variable "environment_variables" {
  description = "Environment variables map"
  type        = map(string)
  default     = {}
}

# ── IAM ──────────────────────────────────────────────────────
# Pass a list of IAM statement objects and the module builds
# the policy document dynamically.
#
# Example:
#   iam_policy_statements = [
#     { actions = ["dynamodb:GetItem","dynamodb:Query"], resources = [table_arn] },
#   ]
# ─────────────────────────────────────────────────────────────

variable "iam_policy_statements" {
  description = "Additional IAM policy statements for this Lambda"
  type = list(object({
    effect    = optional(string, "Allow")
    actions   = list(string)
    resources = list(string)
  }))
  default = []
}

variable "log_retention_days" {
  description = "CloudWatch Logs retention in days"
  type        = number
  default     = 14
}

variable "env_postfix" {
  description = "Unique postfix for resource names"
  type        = string
}
