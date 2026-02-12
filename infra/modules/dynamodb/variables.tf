##──────────────────────────────────────────────────────────────
## DynamoDB module — variables
##──────────────────────────────────────────────────────────────

variable "table_name" {
  description = "DynamoDB table name"
  type        = string
}

variable "hash_key" {
  description = "Hash (partition) key attribute name"
  type        = string
}

variable "hash_key_type" {
  description = "Hash key type (S = String, N = Number, B = Binary)"
  type        = string
  default     = "S"
}

variable "range_key" {
  description = "Optional range (sort) key attribute name"
  type        = string
  default     = null
}

variable "range_key_type" {
  description = "Range key type (S, N, B)"
  type        = string
  default     = "S"
}

variable "billing_mode" {
  description = "DynamoDB billing mode"
  type        = string
  default     = "PAY_PER_REQUEST"
}

variable "enable_pitr" {
  description = "Enable Point-in-Time Recovery"
  type        = bool
  default     = true
}

# ── Global Secondary Indexes ─────────────────────────────────
# Example:
#   global_secondary_indexes = [
#     {
#       name      = "StatusIndex"
#       hash_key  = "status"
#       range_key = "createdAt"
#       projection_type = "ALL"
#     }
#   ]
# ─────────────────────────────────────────────────────────────

variable "global_secondary_indexes" {
  description = "List of GSI definitions"
  type = list(object({
    name            = string
    hash_key        = string
    hash_key_type   = optional(string, "S")
    range_key       = optional(string)
    range_key_type  = optional(string, "S")
    projection_type = optional(string, "ALL")
  }))
  default = []
}

variable "tags" {
  description = "Additional tags"
  type        = map(string)
  default     = {}
}
