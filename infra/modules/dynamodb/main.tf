##──────────────────────────────────────────────────────────────
## DynamoDB module
##
## Creates a table with:
##   • configurable hash key + optional range key
##   • optional GSIs (dynamic block)
##   • PAY_PER_REQUEST billing by default
##   • Point-in-Time Recovery enabled by default
##──────────────────────────────────────────────────────────────

locals {
  # Collect all GSI attributes so they can be declared on the table
  gsi_hash_attrs = [
    for gsi in var.global_secondary_indexes : {
      name = gsi.hash_key
      type = gsi.hash_key_type
    }
  ]
  gsi_range_attrs = [
    for gsi in var.global_secondary_indexes : {
      name = gsi.range_key
      type = gsi.range_key_type
    } if gsi.range_key != null
  ]

  # Merge table-level + GSI attributes, deduplicate by name
  all_extra_attrs = concat(
    var.range_key != null ? [{ name = var.range_key, type = var.range_key_type }] : [],
    local.gsi_hash_attrs,
    local.gsi_range_attrs,
  )
  unique_extra_attrs = {
    for attr in local.all_extra_attrs : attr.name => attr.type
  }
}

resource "aws_dynamodb_table" "this" {
  name         = var.table_name
  billing_mode = var.billing_mode
  hash_key     = var.hash_key
  range_key    = var.range_key

  # ── Hash key attribute (always present) ────────────────────
  attribute {
    name = var.hash_key
    type = var.hash_key_type
  }

  # ── Extra attributes (range key + GSI keys) ────────────────
  dynamic "attribute" {
    for_each = local.unique_extra_attrs
    content {
      name = attribute.key
      type = attribute.value
    }
  }

  # ── Global secondary indexes ───────────────────────────────
  dynamic "global_secondary_index" {
    for_each = var.global_secondary_indexes
    content {
      name            = global_secondary_index.value.name
      hash_key        = global_secondary_index.value.hash_key
      range_key       = global_secondary_index.value.range_key
      projection_type = global_secondary_index.value.projection_type
    }
  }

  point_in_time_recovery {
    enabled = var.enable_pitr
  }

  tags = merge(var.tags, {
    Name = var.table_name
  })
}
