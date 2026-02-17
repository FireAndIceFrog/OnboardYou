##──────────────────────────────────────────────────────────────
## IAM — EventBridge Scheduler execution role
##
## Lambda IAM roles are managed by the ./modules/lambda module.
## This file only contains the Scheduler role which is unique
## (it allows EventBridge Scheduler to invoke the ETL Lambda).
##──────────────────────────────────────────────────────────────

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
  name               = "onboardyou-scheduler-role-${var.environment}-${var.env_postfix}"
  assume_role_policy = data.aws_iam_policy_document.scheduler_assume.json
}

data "aws_iam_policy_document" "scheduler_invoke_etl" {
  statement {
    actions   = ["lambda:InvokeFunction"]
    resources = [module.etl_trigger.arn]
  }
}

resource "aws_iam_role_policy" "scheduler_invoke_etl" {
  name   = "scheduler-invoke-etl"
  role   = aws_iam_role.scheduler_execution.id
  policy = data.aws_iam_policy_document.scheduler_invoke_etl.json
}
