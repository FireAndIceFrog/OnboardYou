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
    tags = {
      Project     = "OnboardYou"
      ManagedBy   = "OpenTofu"
      Environment = var.environment
    }
  }
}

data "aws_caller_identity" "current" {}

# ══════════════════════════════════════════════════════════════
# DynamoDB
# ══════════════════════════════════════════════════════════════

module "pipeline_configs_table" {
  source      = "./modules/dynamodb"
  table_name  = "PipelineConfigs"
  hash_key    = "organizationId"
  range_key   = "customerCompanyId"
  enable_pitr = false
}

module "org_settings_table" {
  source      = "./modules/dynamodb"
  table_name  = "OrgSettings"
  hash_key    = "organizationId"
  enable_pitr = false
}

# ══════════════════════════════════════════════════════════════
# Cognito
# ══════════════════════════════════════════════════════════════

module "cognito" {
  source      = "./modules/cognito"
  environment = var.environment
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
  source_binary  = "${path.module}/../target/lambda/etl-trigger/bootstrap"
  memory_size    = 512
  timeout        = 300

  log_retention_days = var.log_retention_days

  environment_variables = {
    CONFIG_TABLE_NAME    = module.pipeline_configs_table.name
    SETTINGS_TABLE_NAME  = module.org_settings_table.name
    RUST_LOG             = "info"
  }

  iam_policy_statements = [
    {
      actions   = ["dynamodb:GetItem", "dynamodb:Query"]
      resources = [module.pipeline_configs_table.arn, module.org_settings_table.arn]
    },
  ]
}

module "authorizer" {
  source         = "./modules/lambda"
  project_prefix = "onboardyou"
  function_name  = "authorizer"
  description    = "Lambda Authorizer — validates Cognito JWTs and injects organizationId into request context"
  environment    = var.environment
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
  source_binary  = "${path.module}/../target/lambda/config-api/bootstrap"
  memory_size    = 256
  timeout        = 30

  log_retention_days = var.log_retention_days

  environment_variables = {
    CONFIG_TABLE_NAME    = module.pipeline_configs_table.name
    SETTINGS_TABLE_NAME  = module.org_settings_table.name
    ETL_LAMBDA_ARN       = module.etl_trigger.arn
    SCHEDULER_ROLE_ARN   = aws_iam_role.scheduler_execution.arn
    COGNITO_CLIENT_ID    = module.cognito.client_id
    RUST_LOG             = "info"
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
  ]
}

# ══════════════════════════════════════════════════════════════
# Frontend Hosting (S3 + CloudFront)
# ══════════════════════════════════════════════════════════════

module "frontend" {
  source         = "./modules/frontend-hosting"
  project_prefix = "onboardyou"
  environment    = var.environment
  api_origin     = module.api.invoke_url
  price_class    = "PriceClass_100"
}

# ══════════════════════════════════════════════════════════════
# API Gateway
# ══════════════════════════════════════════════════════════════
#
# To add a new route, just add an entry to the routes map.
# The module handles resource creation, method + integration,
# CORS OPTIONS, deployment triggers, and lambda permissions.
#
# ──────────────────────────────────────────────────────────────

module "api" {
  source      = "./modules/api-gateway"
  api_name    = "onboardyou-api"
  description = "OnboardYou Config API — manage ETL pipeline configurations"
  environment = var.environment
  stage_name  = "v1"

  # ── Routing ──────────────────────────────────────────────────
  # /config       → GET (list)
  # /config/{…}   → Axum router in the Lambda handles all sub-paths
  base_path_part       = "config"
  base_methods         = ["GET"]
  lambda_invoke_arn    = module.config_api.invoke_arn
  lambda_function_name = module.config_api.function_name

  # ── Auth ─────────────────────────────────────────────────────
  authorization            = "CUSTOM"
  authorizer_uri           = module.authorizer.invoke_arn
  authorizer_function_name = module.authorizer.function_name

  # ── Unauthenticated /auth path (login) ──────────────────────
  auth_path_enabled = true
}
