##──────────────────────────────────────────────────────────────
## OnboardYou — Root module
##
## All concrete resources live in ./modules/*
## This file wires them together with a single routes map
## so adding a new endpoint is a one-liner.
##──────────────────────────────────────────────────────────────

terraform {
  required_version = ">= 1.6.0" # OpenTofu 1.6+

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }

  # Uncomment and configure for remote state:
  # backend "s3" {
  #   bucket         = "onboardyou-tfstate"
  #   key            = "infra/terraform.tfstate"
  #   region         = "eu-west-1"
  #   dynamodb_table = "onboardyou-tflock"
  #   encrypt        = true
  # }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = merge(var.tags, {
      Environment = var.environment
    })
  }
}

data "aws_caller_identity" "current" {}

locals {
  dynamic_api_event_stream_name = "${var.environment}-DynamicApiEvents"
}

# ══════════════════════════════════════════════════════════════
# DynamoDB
# ══════════════════════════════════════════════════════════════

module "pipeline_configs_table" {
  source      = "./modules/dynamodb"
  table_name  = "PipelineConfigs-${var.env_postfix}"
  hash_key    = "organizationId"
  range_key   = "customerCompanyId"
  enable_pitr = false
}

module "org_settings_table" {
  source      = "./modules/dynamodb"
  table_name  = "OrgSettings-${var.env_postfix}"
  hash_key    = "organizationId"
  enable_pitr = false
}

# ══════════════════════════════════════════════════════════════
# Cognito
# ══════════════════════════════════════════════════════════════

module "cognito" {
  source      = "./modules/cognito"
  environment = var.environment
  env_postfix = var.env_postfix
}

# ══════════════════════════════════════════════════════════════
# Demo user — password rotated every deploy
# ══════════════════════════════════════════════════════════════

module "demo_user" {
  source       = "./modules/demo-user"
  user_pool_id = module.cognito.user_pool_id
  aws_region   = var.aws_region
  users        = var.demo_users
}

# ══════════════════════════════════════════════════════════════
# CSV Upload Bucket (S3)
# ══════════════════════════════════════════════════════════════

module "csv_upload_bucket" {
  source         = "./modules/csv-upload-bucket"
  project_prefix = "onboardyou"
  environment    = var.environment
  env_postfix    = var.env_postfix
}

# ══════════════════════════════════════════════════════════════
# Lambdas
# ══════════════════════════════════════════════════════════════

module "etl_trigger" {
  source         = "./modules/lambda"
  project_prefix = "onboardyou"
  function_name  = "etl-trigger"
  description    = "Invoked by EventBridge Scheduler — reads config from DynamoDB, runs the ETL pipeline"
  environment    = var.environment
  env_postfix    = var.env_postfix
  source_binary  = "${path.module}/../target/lambda/etl-trigger/bootstrap"
  memory_size    = 512
  timeout        = 300

  log_retention_days = var.log_retention_days

  environment_variables = {
    CONFIG_TABLE_NAME    = module.pipeline_configs_table.name
    SETTINGS_TABLE_NAME  = module.org_settings_table.name
    CSV_UPLOAD_BUCKET    = module.csv_upload_bucket.bucket_name
    RUST_LOG             = "info"
    GITHUB_TOKEN         = var.gh_token
  }

  iam_policy_statements = [
    {
      actions   = ["dynamodb:GetItem", "dynamodb:Query"]
      resources = [module.pipeline_configs_table.arn, module.org_settings_table.arn]
    },
    {
      actions   = ["s3:GetObject"]
      resources = ["${module.csv_upload_bucket.bucket_arn}/*"]
    },
  ]
}

module "authorizer" {
  source         = "./modules/lambda"
  project_prefix = "onboardyou"
  function_name  = "authorizer"
  description    = "Lambda Authorizer — validates Cognito JWTs and injects organizationId into request context"
  environment    = var.environment
  env_postfix    = var.env_postfix
  source_binary  = "${path.module}/../target/lambda/authorizer/bootstrap"
  memory_size    = 128
  timeout        = 10

  log_retention_days = var.log_retention_days

  environment_variables = {
    AUTH_DEV_MODE        = var.environment == "dev" ? "true" : "false"
    COGNITO_USER_POOL_ID = module.cognito.user_pool_id
    COGNITO_CLIENT_ID    = module.cognito.client_id
    RUST_LOG             = "info"
  }
}

module "config_api" {
  source         = "./modules/lambda"
  project_prefix = "onboardyou"
  function_name  = "config-api"
  description    = "Config API — GET /config, CRUD /config/{id}, POST /config/{id}/validate"
  environment    = var.environment
  env_postfix    = var.env_postfix
  source_binary  = "${path.module}/../target/lambda/config-api/bootstrap"
  memory_size    = 256
  timeout        = 30

  log_retention_days = var.log_retention_days

  environment_variables = {
    CONFIG_TABLE_NAME                     = module.pipeline_configs_table.name
    SETTINGS_TABLE_NAME                   = module.org_settings_table.name
    ETL_LAMBDA_ARN                        = module.etl_trigger.arn
    SCHEDULER_ROLE_ARN                    = aws_iam_role.scheduler_execution.arn
    COGNITO_CLIENT_ID                     = module.cognito.client_id
    CSV_UPLOAD_BUCKET                     = module.csv_upload_bucket.bucket_name
    AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH  = "true"
    RUST_LOG                              = "info"
    DYNAMIC_API_EVENT_STREAM_NAME        = local.dynamic_api_event_stream_name
  }

  iam_policy_statements = [
    {
      actions   = ["dynamodb:PutItem", "dynamodb:GetItem", "dynamodb:UpdateItem", "dynamodb:DeleteItem", "dynamodb:Query"]
      resources = [module.pipeline_configs_table.arn, module.org_settings_table.arn]
    },
    {
      actions   = ["scheduler:CreateSchedule", "scheduler:UpdateSchedule", "scheduler:DeleteSchedule", "scheduler:GetSchedule"]
      resources = ["arn:aws:scheduler:${var.aws_region}:${data.aws_caller_identity.current.account_id}:schedule/default/onboardyou-*"]
    },
    {
      actions   = ["iam:PassRole"]
      resources = [aws_iam_role.scheduler_execution.arn]
    },
    {
      actions   = ["cognito-idp:InitiateAuth"]
      resources = ["*"]
    },
    {
      actions   = ["s3:PutObject", "s3:GetObject"]
      resources = ["${module.csv_upload_bucket.bucket_arn}/*"]
    },
    {
      # allow publishing custom events (used by schedule_repository)
      actions   = ["events:PutEvents"]
      resources = [
        # if the bus is custom the module will emit an ARN; fall back to
        # wildcard when using the default bus (the module returns empty
        # string in that case).
        module.eventbridge.event_bus_arn != "" ? module.eventbridge.event_bus_arn : "*"
      ]
    },
  ]
}

# ───────────────────────────────────────────────────────────────────────────
# EventBridge rules & targets (used by the API to trigger ETL on demand)
# ───────────────────────────────────────────────────────────────────────────

module "eventbridge" {
  source             = "./modules/eventbridge"
  create_event_bus   = true
  event_bus_name     = local.dynamic_api_event_stream_name

  rules = [
    {
      name          = "etl-on-dynamic-event"
      description   = "forward ScheduledDynamicApiEvent events from the config API bus to the ETL lambda"
      event_pattern = {
        "detail-type" = ["ScheduledDynamicApiEvent"]
      }
      targets = [
        {
          arn = module.etl_trigger.arn
          id  = "etl-lambda"
        }
      ]
    }
  ]
}


# ══════════════════════════════════════════════════════════════
# Frontend Hosting (S3 + CloudFront)
# ══════════════════════════════════════════════════════════════

module "frontend" {
  source         = "./modules/frontend-hosting"
  project_prefix = "onboardyou"
  environment    = var.environment
  env_postfix    = var.env_postfix
  api_origin     = module.api.invoke_url
  price_class    = "PriceClass_100"
}

# ══════════════════════════════════════════════════════════════
# API Gateway
# ══════════════════════════════════════════════════════════════
#
# To add a new route, just add an entry to the routes list.
# Paths ending with /* create a {proxy+} catch-all.
#
# ──────────────────────────────────────────────────────────────

module "api" {
  source      = "./modules/api-gateway"
  api_name    = "onboardyou-api"
  description = "OnboardYou Config API — manage ETL pipeline configurations"
  environment = var.environment
  env_postfix = var.env_postfix
  stage_name  = "v1"

  lambda_invoke_arn    = module.config_api.invoke_arn
  lambda_function_name = module.config_api.function_name

  # ── Auth ─────────────────────────────────────────────────────
  authorization            = "CUSTOM"
  authorizer_uri           = module.authorizer.invoke_arn
  authorizer_function_name = module.authorizer.function_name

  # ── Routes ──────────────────────────────────────────────────
  routes = [
    { path = "config",   methods = ["GET"] },
    { path = "config/*" },
    { path = "auth/*",   auth = false },
    { path = "settings", methods = ["GET", "PUT"] },
  ]
}
