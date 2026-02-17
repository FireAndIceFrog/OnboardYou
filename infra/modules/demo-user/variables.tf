##──────────────────────────────────────────────────────────────
## Demo User module
##
## Creates (or resets) Cognito demo users with randomly
## generated passwords that rotate on every deploy.
##──────────────────────────────────────────────────────────────

variable "user_pool_id" {
  description = "Cognito User Pool ID to create demo users in"
  type        = string
}

variable "aws_region" {
  description = "AWS region the Cognito pool lives in (passed to CLI commands)"
  type        = string
}

variable "users" {
  description = "List of demo users to provision"
  type = list(object({
    email           = string
    organization_id = optional(string, "demo-org")
  }))
}
