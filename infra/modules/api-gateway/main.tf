##──────────────────────────────────────────────────────────────
## API Gateway module
##
## Creates a REST API with dynamic route + method generation.
## All routes live under /{base_path_part}/{route_key}.
## CORS OPTIONS are auto-generated for each route.
##──────────────────────────────────────────────────────────────

locals {
  # ── Flatten routes × methods into a map for for_each ──────
  route_methods = merge([
    for route_key, route in var.routes : {
      for method in route.methods :
      "${route_key}_${lower(method)}" => {
        route_key            = route_key
        method               = upper(method)
        lambda_invoke_arn    = route.lambda_invoke_arn
        lambda_function_name = route.lambda_function_name
      }
    }
  ]...)

  # ── Routes that need CORS ─────────────────────────────────
  cors_routes = { for k, v in var.routes : k => v if v.enable_cors }

  # ── Build per-route Allow-Methods header value ────────────
  cors_methods = {
    for k, v in local.cors_routes : k => join(",", concat(v.methods, ["OPTIONS"]))
  }

  # ── Unique lambdas (for permissions) ──────────────────────
  _lambda_grouped = {
    for k, v in var.routes : v.lambda_function_name => v.lambda_invoke_arn...
  }
  lambda_permissions = { for name, arns in local._lambda_grouped : name => arns[0] }
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

# ── Base path resource (e.g. {organizationId}) ──────────────

resource "aws_api_gateway_resource" "base" {
  rest_api_id = aws_api_gateway_rest_api.this.id
  parent_id   = aws_api_gateway_rest_api.this.root_resource_id
  path_part   = var.base_path_part
}

# ── Child route resources ────────────────────────────────────

resource "aws_api_gateway_resource" "routes" {
  for_each    = var.routes
  rest_api_id = aws_api_gateway_rest_api.this.id
  parent_id   = aws_api_gateway_resource.base.id
  path_part   = each.key
}

# ══════════════════════════════════════════════════════════════
# Methods + Lambda-proxy integrations  (one per route × method)
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_method" "methods" {
  for_each      = local.route_methods
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.routes[each.value.route_key].id
  http_method   = each.value.method
  authorization = var.authorization
}

resource "aws_api_gateway_integration" "methods" {
  for_each                = local.route_methods
  rest_api_id             = aws_api_gateway_rest_api.this.id
  resource_id             = aws_api_gateway_resource.routes[each.value.route_key].id
  http_method             = aws_api_gateway_method.methods[each.key].http_method
  integration_http_method = "POST" # Lambda proxy always uses POST
  type                    = "AWS_PROXY"
  uri                     = each.value.lambda_invoke_arn
}

# ══════════════════════════════════════════════════════════════
# CORS OPTIONS  (one per route with enable_cors = true)
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_method" "cors" {
  for_each      = local.cors_routes
  rest_api_id   = aws_api_gateway_rest_api.this.id
  resource_id   = aws_api_gateway_resource.routes[each.key].id
  http_method   = "OPTIONS"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "cors" {
  for_each    = local.cors_routes
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.routes[each.key].id
  http_method = aws_api_gateway_method.cors[each.key].http_method
  type        = "MOCK"

  request_templates = {
    "application/json" = "{\"statusCode\": 200}"
  }
}

resource "aws_api_gateway_method_response" "cors_200" {
  for_each    = local.cors_routes
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.routes[each.key].id
  http_method = aws_api_gateway_method.cors[each.key].http_method
  status_code = "200"

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = true
    "method.response.header.Access-Control-Allow-Methods" = true
    "method.response.header.Access-Control-Allow-Origin"  = true
  }
}

resource "aws_api_gateway_integration_response" "cors_200" {
  for_each    = local.cors_routes
  rest_api_id = aws_api_gateway_rest_api.this.id
  resource_id = aws_api_gateway_resource.routes[each.key].id
  http_method = aws_api_gateway_method.cors[each.key].http_method
  status_code = aws_api_gateway_method_response.cors_200[each.key].status_code

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = "'${var.cors_allowed_headers}'"
    "method.response.header.Access-Control-Allow-Methods" = "'${local.cors_methods[each.key]}'"
    "method.response.header.Access-Control-Allow-Origin"  = "'${var.cors_allowed_origin}'"
  }
}

# ══════════════════════════════════════════════════════════════
# Deployment + Stage
# ══════════════════════════════════════════════════════════════

resource "aws_api_gateway_deployment" "this" {
  rest_api_id = aws_api_gateway_rest_api.this.id

  depends_on = [
    aws_api_gateway_integration.methods,
    aws_api_gateway_integration.cors,
  ]

  triggers = {
    redeployment = sha1(jsonencode([
      aws_api_gateway_resource.base.id,
      { for k, v in aws_api_gateway_resource.routes : k => v.id },
      { for k, v in aws_api_gateway_method.methods : k => v.id },
      { for k, v in aws_api_gateway_integration.methods : k => v.id },
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

# ══════════════════════════════════════════════════════════════
# Lambda invoke permissions  (de-duplicated by function name)
# ══════════════════════════════════════════════════════════════

resource "aws_lambda_permission" "apigw" {
  for_each      = local.lambda_permissions
  statement_id  = "AllowAPIGatewayInvoke-${each.key}"
  action        = "lambda:InvokeFunction"
  function_name = each.key
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.this.execution_arn}/*/*"
}
