##──────────────────────────────────────────────────────────────
## EventBridge rule/target helper
##──────────────────────────────────────────────────────────────

locals {
  # index rules by name for easy iteration
  rule_map = { for r in var.rules : r.name => r }

  # flatten targets so they can be iterated with `for_each` later
  rule_targets = flatten([
    for r in var.rules : [
      for t in r.targets : merge(t, {
        rule_name     = r.name
        event_bus_name = lookup(r, "event_bus_name", var.event_bus_name)
      })
    ]
  ])
}

# optional custom bus
resource "aws_cloudwatch_event_bus" "this" {
  count = var.create_event_bus && var.event_bus_name != "default" ? 1 : 0

  name = var.event_bus_name
}

resource "aws_cloudwatch_event_rule" "this" {
  for_each = local.rule_map

  name               = each.value.name
  description        = lookup(each.value, "description", null)
  schedule_expression = lookup(each.value, "schedule_expression", null)
  event_pattern      = each.value.event_pattern != null ? jsonencode(each.value.event_pattern) : null
  event_bus_name     = lookup(each.value, "event_bus_name", var.event_bus_name)
}

resource "aws_cloudwatch_event_target" "this" {
  for_each = { for rt in local.rule_targets : "${rt.rule_name}-${rt.id}" => rt }

  rule      = each.value.rule_name
  arn       = each.value.arn
  target_id = each.value.id
  input     = lookup(each.value, "input", null)
  role_arn  = lookup(each.value, "role_arn", null)
}
