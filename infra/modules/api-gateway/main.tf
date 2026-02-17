##──────────────────────────────────────────────────────────────
## API Gateway module  (route-list pattern)
##
## Declare routes as a flat list:
##   "config"   → /config with explicit methods
##   "config/*" → /config/{proxy+} ANY catch-all
##   "auth/*"   → /auth/{proxy+}   unauthenticated
##   "settings" → /settings with explicit methods
##
## The Lambda runs an Axum router, so API Gateway only handles
## auth + CORS; all path matching is done inside the Lambda.
##──────────────────────────────────────────────────────────────

locals {
  # Separate routes into exact and proxy
  exact_routes = { for r in var.routes : r.path => r if !endswith(r.path, "/*") }
  proxy_routes = { for r in var.routes : trimsuffix(r.path, "/*") => r if endswith(r.path, "/*") }

  # All unique parent path segments that need an API Gateway resource
  parent_segments = toset(concat(
    [for r in var.routes : r.path if !endswith(r.path, "/*")],
    [for r in var.routes : trimsuffix(r.path, "/*") if endswith(r.path, "/*")],
  ))

  # Expand exact routes into per-method entries:
  #   "config/GET" → { parent = "config", method = "GET", auth = true }
  exact_method_pairs = merge([
    for k, r in local.exact_routes : {
      for m in r.methods : "${k}/${m}" => {
        parent = k
        method = m
        auth   = r.auth
      }
    }
  ]...)
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
# Path resources  (one per unique parent segment)
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_resource" "path" {
  for_each    = local.parent_segments
  rest_api_id = aws_api_gateway_rest_api.this.id
  parent_id   = aws_api_gateway_rest_api.this.root_resource_id
  path_part   = each.value
}

# ══════════════════════════════════════════════════════════════
# Proxy child resources  ({proxy+} under wildcard paths)
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_resource" "proxy" {
  for_each    = local.proxy_routes
  rest_api_id = aws_api_gateway_rest_api.this.id
  parent_id   = aws_api_gateway_resource.path[each.key].id
  path_part   = "{proxy+}"
}

# ══════════════════════════════════════════════════════════════
# Exact-path methods  (e.g. GET /config, GET /settings)
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_method" "exact" {
  for_each      = local.exact_method_pairs
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.path[each.value.parent].id
  http_method   = each.value.method
  authorization = each.value.auth ? var.authorization : "NONE"
  authorizer_id = each.value.auth && var.authorization == "CUSTOM" ? aws_api_gateway_authorizer.token[0].id : null
}

resource "aws_api_gateway_integration" "exact" {
  for_each                = local.exact_method_pairs
  rest_api_id             = aws_api_gateway_rest_api.this.id
  resource_id             = aws_api_gateway_resource.path[each.value.parent].id
  http_method             = aws_api_gateway_method.exact[each.key].http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = var.lambda_invoke_arn
}

# ══════════════════════════════════════════════════════════════
# Proxy catch-all methods  (ANY /{path}/{proxy+} → Lambda)
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_method" "proxy" {
  for_each      = local.proxy_routes
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.proxy[each.key].id
  http_method   = "ANY"
  authorization = each.value.auth ? var.authorization : "NONE"
  authorizer_id = each.value.auth && var.authorization == "CUSTOM" ? aws_api_gateway_authorizer.token[0].id : null

  request_parameters = {
    "method.request.path.proxy" = true
  }
}

resource "aws_api_gateway_integration" "proxy" {
  for_each                = local.proxy_routes
  rest_api_id             = aws_api_gateway_rest_api.this.id
  resource_id             = aws_api_gateway_resource.proxy[each.key].id
  http_method             = aws_api_gateway_method.proxy[each.key].http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = var.lambda_invoke_arn

  request_parameters = {
    "integration.request.path.proxy" = "method.request.path.proxy"
  }
}

# ══════════════════════════════════════════════════════════════
# CORS OPTIONS — exact paths
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_method" "cors_exact" {
  for_each      = local.exact_routes
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.path[each.key].id
  http_method   = "OPTIONS"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "cors_exact" {
  for_each    = local.exact_routes
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.path[each.key].id
  http_method = aws_api_gateway_method.cors_exact[each.key].http_method
  type        = "MOCK"

  request_templates = {
    "application/json" = "{\"statusCode\": 200}"
  }
}

resource "aws_api_gateway_method_response" "cors_exact_200" {
  for_each    = local.exact_routes
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.path[each.key].id
  http_method = aws_api_gateway_method.cors_exact[each.key].http_method
  status_code = "200"

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = true
    "method.response.header.Access-Control-Allow-Methods" = true
    "method.response.header.Access-Control-Allow-Origin"  = true
  }
}

resource "aws_api_gateway_integration_response" "cors_exact_200" {
  for_each    = local.exact_routes
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.path[each.key].id
  http_method = aws_api_gateway_method.cors_exact[each.key].http_method
  status_code = aws_api_gateway_method_response.cors_exact_200[each.key].status_code

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = "'${var.cors_allowed_headers}'"
    "method.response.header.Access-Control-Allow-Methods" = "'${join(",", concat(each.value.methods, ["OPTIONS"]))}'"
    "method.response.header.Access-Control-Allow-Origin"  = "'${var.cors_allowed_origin}'"
  }
}

# ══════════════════════════════════════════════════════════════
# CORS OPTIONS — proxy paths
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_method" "cors_proxy" {
  for_each      = local.proxy_routes
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.proxy[each.key].id
  http_method   = "OPTIONS"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "cors_proxy" {
  for_each    = local.proxy_routes
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.proxy[each.key].id
  http_method = aws_api_gateway_method.cors_proxy[each.key].http_method
  type        = "MOCK"

  request_templates = {
    "application/json" = "{\"statusCode\": 200}"
  }
}

resource "aws_api_gateway_method_response" "cors_proxy_200" {
  for_each    = local.proxy_routes
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.proxy[each.key].id
  http_method = aws_api_gateway_method.cors_proxy[each.key].http_method
  status_code = "200"

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = true
    "method.response.header.Access-Control-Allow-Methods" = true
    "method.response.header.Access-Control-Allow-Origin"  = true
  }
}

resource "aws_api_gateway_integration_response" "cors_proxy_200" {
  for_each    = local.proxy_routes
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.proxy[each.key].id
  http_method = aws_api_gateway_method.cors_proxy[each.key].http_method
  status_code = aws_api_gateway_method_response.cors_proxy_200[each.key].status_code

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
# Deployment + Stage
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_deployment" "this" {
  rest_api_id = aws_api_gateway_rest_api.this.id

  depends_on = [
    aws_api_gateway_integration.exact,
    aws_api_gateway_integration.proxy,
    aws_api_gateway_integration.cors_exact,
    aws_api_gateway_integration.cors_proxy,
  ]

  triggers = {
    redeployment = sha1(jsonencode([
      { for k, v in aws_api_gateway_resource.path : k => v.id },
      { for k, v in aws_api_gateway_resource.proxy : k => v.id },
      { for k, v in aws_api_gateway_method.exact : k => v.id },
      { for k, v in aws_api_gateway_integration.exact : k => v.id },
      { for k, v in aws_api_gateway_method.proxy : k => v.id },
      { for k, v in aws_api_gateway_integration.proxy : k => v.id },
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
