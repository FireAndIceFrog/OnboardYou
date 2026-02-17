##──────────────────────────────────────────────────────────────
## Demo User — outputs
##──────────────────────────────────────────────────────────────

output "credentials" {
  description = "Map of email → password for every demo user"
  value       = { for email, _ in local.users : email => random_password.this[email].result }
  sensitive   = true
}
