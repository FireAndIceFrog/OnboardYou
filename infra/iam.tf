##──────────────────────────────────────────────────────────────
## IAM — Lambda execution roles + EventBridge Scheduler role
##──────────────────────────────────────────────────────────────

data "aws_caller_identity" "current" {}

# ── Shared assume-role policy for Lambda ─────────────────────
data "aws_iam_policy_document" "lambda_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

# ──────────────────────────────────────────────────────────────
# Config API Lambda role
# ──────────────────────────────────────────────────────────────
resource "aws_iam_role" "config_api_lambda" {
  name               = "onboardyou-config-api-role-${var.environment}"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume.json
}

resource "aws_iam_role_policy_attachment" "config_api_basic" {
  role       = aws_iam_role.config_api_lambda.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

data "aws_iam_policy_document" "config_api_permissions" {
  # DynamoDB read/write on configs table
  statement {
    actions = [
      "dynamodb:PutItem",
      "dynamodb:GetItem",
      "dynamodb:UpdateItem",
      "dynamodb:DeleteItem",
      "dynamodb:Query",
    ]
    resources = [aws_dynamodb_table.pipeline_configs.arn]
  }

  # EventBridge Scheduler management (scoped to onboardyou-* schedules)
  statement {
    actions = [
      "scheduler:CreateSchedule",
      "scheduler:UpdateSchedule",
      "scheduler:DeleteSchedule",
      "scheduler:GetSchedule",
    ]
    resources = [
      "arn:aws:scheduler:${var.aws_region}:${data.aws_caller_identity.current.account_id}:schedule/default/onboardyou-*"
    ]
  }

  # PassRole so the Config Lambda can assign the scheduler role to schedules
  statement {
    actions   = ["iam:PassRole"]
    resources = [aws_iam_role.scheduler_execution.arn]
  }
}

resource "aws_iam_role_policy" "config_api_inline" {
  name   = "config-api-permissions"
  role   = aws_iam_role.config_api_lambda.id
  policy = data.aws_iam_policy_document.config_api_permissions.json
}

# ──────────────────────────────────────────────────────────────
# ETL Trigger Lambda role
# ──────────────────────────────────────────────────────────────
resource "aws_iam_role" "etl_trigger_lambda" {
  name               = "onboardyou-etl-trigger-role-${var.environment}"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume.json
}

resource "aws_iam_role_policy_attachment" "etl_trigger_basic" {
  role       = aws_iam_role.etl_trigger_lambda.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

data "aws_iam_policy_document" "etl_trigger_permissions" {
  # DynamoDB read-only on configs table
  statement {
    actions = [
      "dynamodb:GetItem",
      "dynamodb:Query",
    ]
    resources = [aws_dynamodb_table.pipeline_configs.arn]
  }
}

resource "aws_iam_role_policy" "etl_trigger_inline" {
  name   = "etl-trigger-permissions"
  role   = aws_iam_role.etl_trigger_lambda.id
  policy = data.aws_iam_policy_document.etl_trigger_permissions.json
}

# ──────────────────────────────────────────────────────────────
# EventBridge Scheduler execution role
# (allows Scheduler to invoke the ETL trigger Lambda)
# ──────────────────────────────────────────────────────────────
data "aws_iam_policy_document" "scheduler_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["scheduler.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "scheduler_execution" {
  name               = "onboardyou-scheduler-role-${var.environment}"
  assume_role_policy = data.aws_iam_policy_document.scheduler_assume.json
}

data "aws_iam_policy_document" "scheduler_invoke_etl" {
  statement {
    actions   = ["lambda:InvokeFunction"]
    resources = [aws_lambda_function.etl_trigger.arn]
  }
}

resource "aws_iam_role_policy" "scheduler_invoke_etl" {
  name   = "scheduler-invoke-etl"
  role   = aws_iam_role.scheduler_execution.id
  policy = data.aws_iam_policy_document.scheduler_invoke_etl.json
}
