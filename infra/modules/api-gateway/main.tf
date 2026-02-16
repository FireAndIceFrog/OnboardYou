##──────────────────────────────────────────────────────────────
## API Gateway module  (proxy pattern)
##
## Creates a REST API with:
##   /{base_path_part}            — explicit methods (e.g. GET for list)
##   /{base_path_part}/{proxy+}   — ANY catch-all (Lambda framework routes)
##
## The Lambda runs an Axum router, so API Gateway only handles
## auth + CORS; all path matching is done inside the Lambda.
##──────────────────────────────────────────────────────────────

locals {
  base_cors_methods = join(",", concat(var.base_methods, ["OPTIONS"]))
}

# ══════════════════════════════════════════════════════════════
# REST API
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_rest_api" "this" {
  name        = "${var.api_name}-${var.environment}"
  description = var.description

  endpoint_configuration {
    types = [var.endpoint_type]
  }
}

# ── /config ──────────────────────────────────────────────────

resource "aws_api_gateway_resource" "base" {
  rest_api_id = aws_api_gateway_rest_api.this.id
  parent_id   = aws_api_gateway_rest_api.this.root_resource_id
  path_part   = var.base_path_part
}

# ── /config/{proxy+} ────────────────────────────────────────

resource "aws_api_gateway_resource" "proxy" {
  rest_api_id = aws_api_gateway_rest_api.this.id
  parent_id   = aws_api_gateway_resource.base.id
  path_part   = "{proxy+}"
}

# ══════════════════════════════════════════════════════════════
# Lambda Authorizer (only created when authorization = CUSTOM)
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_authorizer" "token" {
  count                            = var.authorization == "CUSTOM" ? 1 : 0
  rest_api_id                      = aws_api_gateway_rest_api.this.id
  name                             = "onboardyou-authorizer"
  type                             = "TOKEN"
  authorizer_uri                   = var.authorizer_uri
  authorizer_result_ttl_in_seconds = 300
  identity_source                  = "method.request.header.Authorization"
}

resource "aws_lambda_permission" "authorizer" {
  count         = var.authorization == "CUSTOM" ? 1 : 0
  statement_id  = "AllowAPIGatewayAuthorizer"
  action        = "lambda:InvokeFunction"
  function_name = var.authorizer_function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.this.execution_arn}/*/*"
}

# ══════════════════════════════════════════════════════════════
# Base path methods  (e.g. GET /config → list)
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_method" "base" {
  for_each      = toset(var.base_methods)
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.base.id
  http_method   = each.value
  authorization = var.authorization
  authorizer_id = var.authorization == "CUSTOM" ? aws_api_gateway_authorizer.token[0].id : null
}

resource "aws_api_gateway_integration" "base" {
  for_each                = toset(var.base_methods)
  rest_api_id             = aws_api_gateway_rest_api.this.id
  resource_id             = aws_api_gateway_resource.base.id
  http_method             = aws_api_gateway_method.base[each.key].http_method
  integration_http_method = "POST" # Lambda proxy always uses POST
  type                    = "AWS_PROXY"
  uri                     = var.lambda_invoke_arn
}

# ══════════════════════════════════════════════════════════════
# Proxy catch-all  (ANY /config/{proxy+} → Lambda)
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_method" "proxy" {
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.proxy.id
  http_method   = "ANY"
  authorization = var.authorization
  authorizer_id = var.authorization == "CUSTOM" ? aws_api_gateway_authorizer.token[0].id : null

  request_parameters = {
    "method.request.path.proxy" = true
  }
}

resource "aws_api_gateway_integration" "proxy" {
  rest_api_id             = aws_api_gateway_rest_api.this.id
  resource_id             = aws_api_gateway_resource.proxy.id
  http_method             = aws_api_gateway_method.proxy.http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = var.lambda_invoke_arn

  request_parameters = {
    "integration.request.path.proxy" = "method.request.path.proxy"
  }
}

# ══════════════════════════════════════════════════════════════
# CORS OPTIONS — base path  (/config)
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_method" "cors_base" {
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.base.id
  http_method   = "OPTIONS"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "cors_base" {
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.base.id
  http_method = aws_api_gateway_method.cors_base.http_method
  type        = "MOCK"

  request_templates = {
    "application/json" = "{\"statusCode\": 200}"
  }
}

resource "aws_api_gateway_method_response" "cors_base_200" {
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.base.id
  http_method = aws_api_gateway_method.cors_base.http_method
  status_code = "200"

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = true
    "method.response.header.Access-Control-Allow-Methods" = true
    "method.response.header.Access-Control-Allow-Origin"  = true
  }
}

resource "aws_api_gateway_integration_response" "cors_base_200" {
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.base.id
  http_method = aws_api_gateway_method.cors_base.http_method
  status_code = aws_api_gateway_method_response.cors_base_200.status_code

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = "'${var.cors_allowed_headers}'"
    "method.response.header.Access-Control-Allow-Methods" = "'${local.base_cors_methods}'"
    "method.response.header.Access-Control-Allow-Origin"  = "'${var.cors_allowed_origin}'"
  }
}

# ══════════════════════════════════════════════════════════════
# CORS OPTIONS — proxy path  (/config/{proxy+})
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_method" "cors_proxy" {
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.proxy.id
  http_method   = "OPTIONS"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "cors_proxy" {
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.proxy.id
  http_method = aws_api_gateway_method.cors_proxy.http_method
  type        = "MOCK"

  request_templates = {
    "application/json" = "{\"statusCode\": 200}"
  }
}

resource "aws_api_gateway_method_response" "cors_proxy_200" {
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.proxy.id
  http_method = aws_api_gateway_method.cors_proxy.http_method
  status_code = "200"

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = true
    "method.response.header.Access-Control-Allow-Methods" = true
    "method.response.header.Access-Control-Allow-Origin"  = true
  }
}

resource "aws_api_gateway_integration_response" "cors_proxy_200" {
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.proxy.id
  http_method = aws_api_gateway_method.cors_proxy.http_method
  status_code = aws_api_gateway_method_response.cors_proxy_200.status_code

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = "'${var.cors_allowed_headers}'"
    "method.response.header.Access-Control-Allow-Methods" = "'GET,POST,PUT,DELETE,OPTIONS'"
    "method.response.header.Access-Control-Allow-Origin"  = "'${var.cors_allowed_origin}'"
  }
}

# ══════════════════════════════════════════════════════════════
# Lambda invoke permission
# ══════════════════════════════════════════════════════════════

resource "aws_lambda_permission" "apigw" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = var.lambda_function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.this.execution_arn}/*/*"
}

# ══════════════════════════════════════════════════════════════
# Public /auth path  (login — no authorizer)
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_resource" "auth" {
  count       = var.auth_path_enabled ? 1 : 0
  rest_api_id = aws_api_gateway_rest_api.this.id
  parent_id   = aws_api_gateway_rest_api.this.root_resource_id
  path_part   = var.auth_path_part
}

resource "aws_api_gateway_resource" "auth_proxy" {
  count       = var.auth_path_enabled ? 1 : 0
  rest_api_id = aws_api_gateway_rest_api.this.id
  parent_id   = aws_api_gateway_resource.auth[0].id
  path_part   = "{proxy+}"
}

# ── POST /auth/{proxy+} → Lambda (NONE auth) ────────────────

resource "aws_api_gateway_method" "auth_proxy" {
  count         = var.auth_path_enabled ? 1 : 0
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.auth_proxy[0].id
  http_method   = "ANY"
  authorization = "NONE"

  request_parameters = {
    "method.request.path.proxy" = true
  }
}

resource "aws_api_gateway_integration" "auth_proxy" {
  count                   = var.auth_path_enabled ? 1 : 0
  rest_api_id             = aws_api_gateway_rest_api.this.id
  resource_id             = aws_api_gateway_resource.auth_proxy[0].id
  http_method             = aws_api_gateway_method.auth_proxy[0].http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = var.lambda_invoke_arn

  request_parameters = {
    "integration.request.path.proxy" = "method.request.path.proxy"
  }
}

# ── CORS OPTIONS — /auth/{proxy+} ───────────────────────────

resource "aws_api_gateway_method" "cors_auth_proxy" {
  count         = var.auth_path_enabled ? 1 : 0
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.auth_proxy[0].id
  http_method   = "OPTIONS"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "cors_auth_proxy" {
  count       = var.auth_path_enabled ? 1 : 0
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.auth_proxy[0].id
  http_method = aws_api_gateway_method.cors_auth_proxy[0].http_method
  type        = "MOCK"

  request_templates = {
    "application/json" = "{\"statusCode\": 200}"
  }
}

resource "aws_api_gateway_method_response" "cors_auth_proxy_200" {
  count       = var.auth_path_enabled ? 1 : 0
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.auth_proxy[0].id
  http_method = aws_api_gateway_method.cors_auth_proxy[0].http_method
  status_code = "200"

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = true
    "method.response.header.Access-Control-Allow-Methods" = true
    "method.response.header.Access-Control-Allow-Origin"  = true
  }
}

resource "aws_api_gateway_integration_response" "cors_auth_proxy_200" {
  count       = var.auth_path_enabled ? 1 : 0
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.auth_proxy[0].id
  http_method = aws_api_gateway_method.cors_auth_proxy[0].http_method
  status_code = aws_api_gateway_method_response.cors_auth_proxy_200[0].status_code

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = "'${var.cors_allowed_headers}'"
    "method.response.header.Access-Control-Allow-Methods" = "'POST,OPTIONS'"
    "method.response.header.Access-Control-Allow-Origin"  = "'${var.cors_allowed_origin}'"
  }
}

# ══════════════════════════════════════════════════════════════
# Deployment + Stage
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_deployment" "this" {
  rest_api_id = aws_api_gateway_rest_api.this.id

  depends_on = [
    aws_api_gateway_integration.base,
    aws_api_gateway_integration.proxy,
    aws_api_gateway_integration.cors_base,
    aws_api_gateway_integration.cors_proxy,
    aws_api_gateway_integration.auth_proxy,
    aws_api_gateway_integration.cors_auth_proxy,
  ]

  triggers = {
    redeployment = sha1(jsonencode([
      aws_api_gateway_resource.base.id,
      aws_api_gateway_resource.proxy.id,
      aws_api_gateway_method.proxy.id,
      { for k, v in aws_api_gateway_method.base : k => v.id },
      { for k, v in aws_api_gateway_integration.base : k => v.id },
      aws_api_gateway_integration.proxy.id,
      var.auth_path_enabled ? aws_api_gateway_resource.auth_proxy[0].id : null,
      var.auth_path_enabled ? aws_api_gateway_integration.auth_proxy[0].id : null,
    ]))
  }

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_api_gateway_stage" "this" {
  deployment_id = aws_api_gateway_deployment.this.id
  rest_api_id   = aws_api_gateway_rest_api.this.id
  stage_name    = var.stage_name

  xray_tracing_enabled = var.xray_enabled
}
