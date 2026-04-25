##──────────────────────────────────────────────────────────────
## SES module — variables
##──────────────────────────────────────────────────────────────

variable "env_postfix" {
  type        = string
  description = "Environment suffix appended to all resource names (e.g. 'dev', 'prod-abc123')."
}

variable "email_ingestion_domain" {
  type        = string
  description = "The domain registered with SES for inbound email (e.g. 'mail.onboardyou.com')."
}

variable "aws_account_id" {
  type        = string
  description = "AWS account ID — used in the ses-inbox bucket policy to restrict SES write access."
}
