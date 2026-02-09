##──────────────────────────────────────────────────────────────
## DynamoDB — PipelineConfigs table
##
## Schema:
##   PK  = organizationId  (String)
##   Attrs: cron, lastEdited, pipeline (JSON string)
##──────────────────────────────────────────────────────────────

resource "aws_dynamodb_table" "pipeline_configs" {
  name         = var.config_table_name
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "organizationId"

  attribute {
    name = "organizationId"
    type = "S"
  }

  point_in_time_recovery {
    enabled = true
  }

  tags = {
    Name = var.config_table_name
  }
}
