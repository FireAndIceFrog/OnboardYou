##──────────────────────────────────────────────────────────────
## OnboardYou — Root module
## Wires together DynamoDB, both Lambdas, API Gateway,
## EventBridge Scheduler role, and IAM.
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
