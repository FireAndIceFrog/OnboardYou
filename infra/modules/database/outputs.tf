# Outputs exported by the database module

output "database_password" {
  description = "Randomly generated password for the Supabase postgres user"
  value       = random_password.db.result
  sensitive   = true
}

output "project_id" {
  description = "Supabase project identifier used to build the host name"
  value       = supabase_project.this.id
}

output "connection_string_pooler" {
  description = "Postgres connection string for serverless lambdas (via pooler)."
  # pooler host format: aws-1-<region>.pooler.supabase.com:6543
  # urlencode the password so special chars (? $ @ : etc.) don't break the URI
  value       = "postgresql://postgres.${supabase_project.this.id}:${urlencode(random_password.db.result)}@aws-1-${var.region}.pooler.supabase.com:6543/postgres?pgbouncer=true"
  sensitive   = true
}

output "connection_string_direct" {
  description = "Direct Postgres connection string (useful for migrations)."
  value       = "postgres://postgres:${urlencode(random_password.db.result)}@db.${supabase_project.this.id}.supabase.co:5432/postgres"
  sensitive   = true
}