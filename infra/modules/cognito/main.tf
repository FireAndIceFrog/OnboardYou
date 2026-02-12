##──────────────────────────────────────────────────────────────
## Cognito module
##
## Creates a User Pool with a custom `organizationId` attribute
## and an App Client for the API.
##──────────────────────────────────────────────────────────────

variable "environment" {
  description = "Deployment environment (dev / staging / prod)"
  type        = string
}

resource "aws_cognito_user_pool" "main" {
  name = "onboardyou-${var.environment}"

  auto_verified_attributes = ["email"]
  username_attributes      = ["email"]

  password_policy {
    minimum_length    = 12
    require_lowercase = true
    require_uppercase = true
    require_numbers   = true
    require_symbols   = false
  }

  schema {
    name                = "organizationId"
    attribute_data_type = "String"
    mutable             = true
    required            = false

    string_attribute_constraints {
      min_length = 1
      max_length = 128
    }
  }

  schema {
    name                = "email"
    attribute_data_type = "String"
    mutable             = true
    required            = true

    string_attribute_constraints {
      min_length = 1
      max_length = 256
    }
  }
}

resource "aws_cognito_user_pool_client" "api" {
  name         = "onboardyou-api-client"
  user_pool_id = aws_cognito_user_pool.main.id

  explicit_auth_flows = [
    "ALLOW_USER_PASSWORD_AUTH",
    "ALLOW_REFRESH_TOKEN_AUTH",
  ]

  read_attributes  = ["email", "custom:organizationId"]
  write_attributes = ["email", "custom:organizationId"]
}

output "user_pool_id" {
  description = "Cognito User Pool ID"
  value       = aws_cognito_user_pool.main.id
}

output "client_id" {
  description = "Cognito App Client ID"
  value       = aws_cognito_user_pool_client.api.id
}
