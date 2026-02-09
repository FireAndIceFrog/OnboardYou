##──────────────────────────────────────────────────────────────
## API Gateway (REST) — /{organizationId}/config
##──────────────────────────────────────────────────────────────

resource "aws_api_gateway_rest_api" "onboardyou" {
  name        = "onboardyou-api-${var.environment}"
  description = "OnboardYou Config API — manage ETL pipeline configurations"

  endpoint_configuration {
    types = ["REGIONAL"]
  }
}

# ── /{organizationId} ───────────────────────────────────────
resource "aws_api_gateway_resource" "org" {
  rest_api_id = aws_api_gateway_rest_api.onboardyou.id
  parent_id   = aws_api_gateway_rest_api.onboardyou.root_resource_id
  path_part   = "{organizationId}"
}

# ── /{organizationId}/config ────────────────────────────────
resource "aws_api_gateway_resource" "config" {
  rest_api_id = aws_api_gateway_rest_api.onboardyou.id
  parent_id   = aws_api_gateway_resource.org.id
  path_part   = "config"
}

# ── POST method ─────────────────────────────────────────────
resource "aws_api_gateway_method" "post_config" {
  rest_api_id   = aws_api_gateway_rest_api.onboardyou.id
  resource_id   = aws_api_gateway_resource.config.id
  http_method   = "POST"
  authorization = "NONE" # TODO: add Cognito / API key auth
}

resource "aws_api_gateway_integration" "post_config" {
  rest_api_id             = aws_api_gateway_rest_api.onboardyou.id
  resource_id             = aws_api_gateway_resource.config.id
  http_method             = aws_api_gateway_method.post_config.http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.config_api.invoke_arn
}

# ── PUT method ──────────────────────────────────────────────
resource "aws_api_gateway_method" "put_config" {
  rest_api_id   = aws_api_gateway_rest_api.onboardyou.id
  resource_id   = aws_api_gateway_resource.config.id
  http_method   = "PUT"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "put_config" {
  rest_api_id             = aws_api_gateway_rest_api.onboardyou.id
  resource_id             = aws_api_gateway_resource.config.id
  http_method             = aws_api_gateway_method.put_config.http_method
  integration_http_method = "POST" # Lambda proxy always uses POST
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.config_api.invoke_arn
}

# ── GET method (fetch current config) ───────────────────────
resource "aws_api_gateway_method" "get_config" {
  rest_api_id   = aws_api_gateway_rest_api.onboardyou.id
  resource_id   = aws_api_gateway_resource.config.id
  http_method   = "GET"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "get_config" {
  rest_api_id             = aws_api_gateway_rest_api.onboardyou.id
  resource_id             = aws_api_gateway_resource.config.id
  http_method             = aws_api_gateway_method.get_config.http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.config_api.invoke_arn
}

# ── CORS OPTIONS ────────────────────────────────────────────
resource "aws_api_gateway_method" "options_config" {
  rest_api_id   = aws_api_gateway_rest_api.onboardyou.id
  resource_id   = aws_api_gateway_resource.config.id
  http_method   = "OPTIONS"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "options_config" {
  rest_api_id = aws_api_gateway_rest_api.onboardyou.id
  resource_id = aws_api_gateway_resource.config.id
  http_method = aws_api_gateway_method.options_config.http_method
  type        = "MOCK"

  request_templates = {
    "application/json" = "{\"statusCode\": 200}"
  }
}

resource "aws_api_gateway_method_response" "options_200" {
  rest_api_id = aws_api_gateway_rest_api.onboardyou.id
  resource_id = aws_api_gateway_resource.config.id
  http_method = aws_api_gateway_method.options_config.http_method
  status_code = "200"

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = true
    "method.response.header.Access-Control-Allow-Methods" = true
    "method.response.header.Access-Control-Allow-Origin"  = true
  }
}

resource "aws_api_gateway_integration_response" "options_200" {
  rest_api_id = aws_api_gateway_rest_api.onboardyou.id
  resource_id = aws_api_gateway_resource.config.id
  http_method = aws_api_gateway_method.options_config.http_method
  status_code = aws_api_gateway_method_response.options_200.status_code

  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = "'Content-Type,Authorization'"
    "method.response.header.Access-Control-Allow-Methods" = "'GET,POST,PUT,OPTIONS'"
    "method.response.header.Access-Control-Allow-Origin"  = "'*'"
  }
}

# ── Deployment + Stage ──────────────────────────────────────
resource "aws_api_gateway_deployment" "main" {
  rest_api_id = aws_api_gateway_rest_api.onboardyou.id

  depends_on = [
    aws_api_gateway_integration.post_config,
    aws_api_gateway_integration.put_config,
    aws_api_gateway_integration.get_config,
    aws_api_gateway_integration.options_config,
  ]

  # Force redeployment when any resource/method/integration changes
  triggers = {
    redeployment = sha1(jsonencode([
      aws_api_gateway_resource.org.id,
      aws_api_gateway_resource.config.id,
      aws_api_gateway_method.post_config.id,
      aws_api_gateway_method.put_config.id,
      aws_api_gateway_method.get_config.id,
      aws_api_gateway_integration.post_config.id,
      aws_api_gateway_integration.put_config.id,
      aws_api_gateway_integration.get_config.id,
    ]))
  }

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_api_gateway_stage" "v1" {
  deployment_id = aws_api_gateway_deployment.main.id
  rest_api_id   = aws_api_gateway_rest_api.onboardyou.id
  stage_name    = "v1"

  xray_tracing_enabled = true
}

# ── Permission: API Gateway → Config Lambda ─────────────────
resource "aws_lambda_permission" "apigw_invoke_config_api" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.config_api.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.onboardyou.execution_arn}/*/*"
}
