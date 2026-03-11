terraform {
  required_providers {
    supabase = {
      source  = "supabase/supabase"
      version = "~> 1.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }
}

resource "random_password" "db" {
  length  = 16
  special = true
}

resource "supabase_project" "this" {
  provider                = supabase
  organization_id         = var.organization_id
  name                    = var.project_name
  database_password       = random_password.db.result
  region                  = var.region
}