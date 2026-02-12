##──────────────────────────────────────────────────────────────
## API Gateway module — variables
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
  description = "First path segment shared by all routes (e.g. {organizationId})"
  type        = string
  default     = "{organizationId}"
}

variable "authorization" {
  description = "Default authorization type for all methods"
  type        = string
  default     = "NONE"
}

variable "xray_enabled" {
  description = "Enable X-Ray tracing on the stage"
  type        = bool
  default     = true
}

# ── Route definitions ────────────────────────────────────────
# Each key becomes a child path under /{base_path_part}/{key}
#
# Example:
#   routes = {
#     config   = { methods = ["GET","POST","PUT"], lambda_invoke_arn = "…", lambda_function_name = "…" }
#     validate = { methods = ["POST"],             lambda_invoke_arn = "…", lambda_function_name = "…" }
#   }
#
# Produces:
#   /{organizationId}/config   — GET, POST, PUT, OPTIONS (CORS)
#   /{organizationId}/validate — POST, OPTIONS (CORS)
# ─────────────────────────────────────────────────────────────

variable "routes" {
  description = "Map of route_key → route definition. Key = child path part."
  type = map(object({
    methods              = list(string)
    lambda_invoke_arn    = string
    lambda_function_name = string
    enable_cors          = optional(bool, true)
  }))
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
