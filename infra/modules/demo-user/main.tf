##──────────────────────────────────────────────────────────────
## Demo User — resources
##──────────────────────────────────────────────────────────────

locals {
  users = { for u in var.users : u.email => u }
}

terraform {
  required_providers {
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }
}

resource "random_password" "this" {
  for_each = local.users

  length           = 20
  special          = true
  override_special = "!@#$"
  min_lower        = 2
  min_upper        = 2
  min_numeric      = 2
  min_special      = 1

  keepers = {
    rotate = plantimestamp()
  }
}

resource "terraform_data" "provision" {
  for_each = local.users

  triggers_replace = [random_password.this[each.key].result]

  provisioner "local-exec" {
    interpreter = ["bash", "-c"]
    environment = {
      AWS_DEFAULT_REGION = var.aws_region
      POOL_ID            = var.user_pool_id
      EMAIL              = each.key
      PASSWORD           = random_password.this[each.key].result
      ORG_ID             = each.value.organization_id
    }
    command = <<-EOT
      # Create user (ignore AlreadyExists)
      aws cognito-idp admin-create-user \
        --user-pool-id "$POOL_ID" \
        --username "$EMAIL" \
        --user-attributes \
            Name=email,Value="$EMAIL" \
            Name=email_verified,Value=true \
            Name=custom:organizationId,Value="$ORG_ID" \
        --message-action SUPPRESS 2>/dev/null || true

      # Set permanent password (rotated every deploy)
      aws cognito-idp admin-set-user-password \
        --user-pool-id "$POOL_ID" \
        --username "$EMAIL" \
        --password "$PASSWORD" \
        --permanent
    EOT
  }
}
