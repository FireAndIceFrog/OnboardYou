output "event_bus_name" {
  description = "Name of the event bus in use (may be default)."
  value       = var.event_bus_name
}

output "event_bus_arn" {
  description = "ARN of the custom event bus, or empty string when using default."
  value = var.create_event_bus && var.event_bus_name != "default" ? aws_cloudwatch_event_bus.this[0].arn : ""
}

output "rule_names" {
  description = "Names of all rules created by this module."
  value       = [for r in aws_cloudwatch_event_rule.this : r.name]
}

output "rule_arns" {
  description = "ARNs of all rules created by this module."
  value       = [for r in aws_cloudwatch_event_rule.this : r.arn]
}