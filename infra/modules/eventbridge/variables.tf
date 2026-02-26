##──────────────────────────────────────────────────────────────
## EventBridge module
##
## Generic helper for creating rules and targets, optionally on a
## custom event bus.  The `rules` argument accepts a list of objects
## so you can instantiate as many rules/targets as you need.  Each
## rule may override the bus name independently, and the module
## optionally creates a non‑default bus when `create_event_bus` is
## true.
##──────────────────────────────────────────────────────────────

variable "event_bus_name" {
  description = "Name of the event bus to associate rules with.  If you don't need a custom bus the default value can be left unchanged."
  type    = string
  default = "default"
}

variable "create_event_bus" {
  description = "When true and `event_bus_name` is not `default` the module will create a custom event bus with the given name."
  type    = bool
  default = false
}

variable "rules" {
  description = "List of rules to create.  Each rule may optionally specify its own `event_bus_name` to use a different bus than the module default."
  type = list(object({
    name               = string
    description        = optional(string)
    schedule_expression = optional(string)
    event_pattern      = optional(any)
    event_bus_name     = optional(string)
    targets = list(object({
      arn      = string
      id       = string
      input    = optional(string)
      role_arn = optional(string)
    }))
  }))
  default = []
}