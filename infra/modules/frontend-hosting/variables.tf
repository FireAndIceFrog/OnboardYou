##──────────────────────────────────────────────────────────────
## frontend-hosting — input variables
##──────────────────────────────────────────────────────────────

variable "project_prefix" {
  description = "Prefix for resource names (e.g. onboardyou)"
  type        = string
}

variable "environment" {
  description = "Deployment environment (dev / staging / prod)"
  type        = string
}

variable "api_origin" {
  description = "Origin URL for the backend API (used in CORS / CSP headers)"
  type        = string
  default     = ""
}

variable "default_root_object" {
  description = "Default document served by CloudFront"
  type        = string
  default     = "index.html"
}

variable "price_class" {
  description = "CloudFront price class (PriceClass_100 = US/EU only, PriceClass_All = all edge locations)"
  type        = string
  default     = "PriceClass_100"
}
