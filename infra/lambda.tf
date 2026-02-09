##──────────────────────────────────────────────────────────────
## Lambda Functions
##
## Both are Rust binaries compiled with cargo-lambda and
## deployed as provided.al2023 (custom runtime).
##
## Build artefacts are expected at:
##   target/lambda/config-api/bootstrap
##   target/lambda/etl-trigger/bootstrap
##──────────────────────────────────────────────────────────────

# ── Config API Lambda ────────────────────────────────────────

data "archive_file" "config_api_zip" {
  type        = "zip"
  source_file = "${path.module}/../target/lambda/config-api/bootstrap"
  output_path = "${path.module}/.build/config-api.zip"
}

resource "aws_lambda_function" "config_api" {
  function_name = "onboardyou-config-api-${var.environment}"
  description   = "POST/PUT /{organizationId}/config → DynamoDB + EventBridge Scheduler"
  role          = aws_iam_role.config_api_lambda.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["x86_64"]
  memory_size   = 256
  timeout       = 30

  filename         = data.archive_file.config_api_zip.output_path
  source_code_hash = data.archive_file.config_api_zip.output_base64sha256

  environment {
    variables = {
      CONFIG_TABLE_NAME  = aws_dynamodb_table.pipeline_configs.name
      ETL_LAMBDA_ARN     = aws_lambda_function.etl_trigger.arn
      SCHEDULER_ROLE_ARN = aws_iam_role.scheduler_execution.arn
      RUST_LOG           = "info"
    }
  }
}

resource "aws_cloudwatch_log_group" "config_api" {
  name              = "/aws/lambda/${aws_lambda_function.config_api.function_name}"
  retention_in_days = var.log_retention_days
}

# ── ETL Trigger Lambda ───────────────────────────────────────

data "archive_file" "etl_trigger_zip" {
  type        = "zip"
  source_file = "${path.module}/../target/lambda/etl-trigger/bootstrap"
  output_path = "${path.module}/.build/etl-trigger.zip"
}

resource "aws_lambda_function" "etl_trigger" {
  function_name = "onboardyou-etl-trigger-${var.environment}"
  description   = "Invoked by EventBridge Scheduler — reads config from DynamoDB, runs the ETL pipeline"
  role          = aws_iam_role.etl_trigger_lambda.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["x86_64"]
  memory_size   = 512
  timeout       = 300 # 5 minutes for pipeline execution

  filename         = data.archive_file.etl_trigger_zip.output_path
  source_code_hash = data.archive_file.etl_trigger_zip.output_base64sha256

  environment {
    variables = {
      CONFIG_TABLE_NAME = aws_dynamodb_table.pipeline_configs.name
      RUST_LOG          = "info"
    }
  }
}

resource "aws_cloudwatch_log_group" "etl_trigger" {
  name              = "/aws/lambda/${aws_lambda_function.etl_trigger.function_name}"
  retention_in_days = var.log_retention_days
}
