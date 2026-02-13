##──────────────────────────────────────────────────────────────
## API Gateway module — variables  (proxy pattern)
##──────────────────────────────────────────────────────────────

variable "api_name" {
  description = "Name of the REST API"
  type        = string
}

variable "description" {
  description = "API description"
  type        = string
  default     = ""
}

variable "environment" {
  description = "Deployment environment (dev / staging / prod)"
  type        = string
}

variable "stage_name" {
  description = "API Gateway stage name"
  type        = string
  default     = "v1"
}

variable "endpoint_type" {
  description = "API Gateway endpoint type"
  type        = string
  default     = "REGIONAL"
}

variable "base_path_part" {
  description = "Root resource path segment (e.g. 'config')"
  type        = string
  default     = "config"
}

variable "base_methods" {
  description = "HTTP methods on the base resource itself (e.g. ['GET'] for list)"
  type        = list(string)
  default     = []
}

variable "lambda_invoke_arn" {
  description = "Invoke ARN of the Lambda that handles all requests"
  type        = string
}

variable "lambda_function_name" {
  description = "Function name of the Lambda (for invoke permission)"
  type        = string
}

variable "authorization" {
  description = "Default authorization type for all methods (NONE or CUSTOM)"
  type        = string
  default     = "NONE"
}

variable "authorizer_uri" {
  description = "Invoke ARN of the Lambda Authorizer (required when authorization = CUSTOM)"
  type        = string
  default     = null
}

variable "authorizer_function_name" {
  description = "Function name of the Lambda Authorizer (for invoke permission)"
  type        = string
  default     = null
}

variable "xray_enabled" {
  description = "Enable X-Ray tracing on the stage"
  type        = bool
  default     = true
}

# ── CORS defaults ────────────────────────────────────────────

variable "cors_allowed_headers" {
  description = "Value for Access-Control-Allow-Headers"
  type        = string
  default     = "Content-Type,Authorization"
}

variable "cors_allowed_origin" {
  description = "Value for Access-Control-Allow-Origin"
  type        = string
  default     = "*"
}
