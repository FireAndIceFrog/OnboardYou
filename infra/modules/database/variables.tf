
variable "supabase_access_token" {
  description = "Supabase access token for provisioning and seeding the database"
  type        = string
  sensitive   = true
}

variable "organization_id" {
  description = "Supabase organization slug (from dashboard URL)"
  type        = string
}

variable "project_name" {
  description = "Name of the Supabase project"
  type        = string
  default     = "onboardYou"
}

variable "region" {
  description = "Region where the Supabase project will be created; should match aws_region"
  type        = string
}