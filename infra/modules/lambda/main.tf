##──────────────────────────────────────────────────────────────
## Lambda module
##
## Creates a Rust Lambda (provided.al2023) together with:
##   • IAM execution role + optional inline policy (dynamic)
##   • CloudWatch log group
##   • zip archive from the bootstrap binary
##──────────────────────────────────────────────────────────────

locals {
  full_function_name = "${var.project_prefix}-${var.function_name}-${var.environment}"
  has_custom_policy  = length(var.iam_policy_statements) > 0
}

# ══════════════════════════════════════════════════════════════
# IAM role
# ══════════════════════════════════════════════════════════════

data "aws_iam_policy_document" "assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "this" {
  name               = "${var.project_prefix}-${var.function_name}-role-${var.environment}"
  assume_role_policy = data.aws_iam_policy_document.assume.json
}

resource "aws_iam_role_policy_attachment" "basic_execution" {
  role       = aws_iam_role.this.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# ── Dynamic custom policy (only when statements are provided) ─

data "aws_iam_policy_document" "custom" {
  count = local.has_custom_policy ? 1 : 0

  dynamic "statement" {
    for_each = var.iam_policy_statements
    content {
      effect    = statement.value.effect
      actions   = statement.value.actions
      resources = statement.value.resources
    }
  }
}

resource "aws_iam_role_policy" "custom" {
  count  = local.has_custom_policy ? 1 : 0
  name   = "${var.function_name}-permissions"
  role   = aws_iam_role.this.id
  policy = data.aws_iam_policy_document.custom[0].json
}

# ══════════════════════════════════════════════════════════════
# Lambda function
# ══════════════════════════════════════════════════════════════

data "archive_file" "this" {
  type        = "zip"
  source_file = var.source_binary
  output_path = "${path.root}/.build/${var.function_name}.zip"
}

resource "aws_lambda_function" "this" {
  function_name = local.full_function_name
  description   = var.description
  role          = aws_iam_role.this.arn
  handler       = var.handler
  runtime       = var.runtime
  architectures = var.architectures
  memory_size   = var.memory_size
  timeout       = var.timeout

  filename         = data.archive_file.this.output_path
  source_code_hash = data.archive_file.this.output_base64sha256

  dynamic "environment" {
    for_each = length(var.environment_variables) > 0 ? [1] : []
    content {
      variables = var.environment_variables
    }
  }
}

# ══════════════════════════════════════════════════════════════
# CloudWatch log group
# ══════════════════════════════════════════════════════════════

resource "aws_cloudwatch_log_group" "this" {
  name              = "/aws/lambda/${aws_lambda_function.this.function_name}"
  retention_in_days = var.log_retention_days
}
