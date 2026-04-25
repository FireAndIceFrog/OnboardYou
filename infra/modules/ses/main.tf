##──────────────────────────────────────────────────────────────
## SES email-ingestion module
##
## Creates:
##   • SES domain identity + receipt rule set
##   • ses-inbox S3 bucket (with lifecycle + bucket policy)
##   • SES receipt rule that stores inbound mail in the bucket
##   • EmailRoutes DynamoDB table
##──────────────────────────────────────────────────────────────

# ── SES domain identity ──────────────────────────────────────
# Only created when a domain is configured; skipped in local/staging
# environments where email_ingestion_domain is left as the default "".
resource "aws_ses_domain_identity" "email_ingestion" {
  count  = var.email_ingestion_domain != "" ? 1 : 0
  domain = var.email_ingestion_domain
}

# ── SES receipt rule set ──────────────────────────────────────
resource "aws_ses_receipt_rule_set" "email_ingestion" {
  rule_set_name = "onboardyou-email-ingestion-${var.env_postfix}"
}

resource "aws_ses_active_receipt_rule_set" "email_ingestion" {
  rule_set_name = aws_ses_receipt_rule_set.email_ingestion.rule_set_name
}

# ── ses-inbox S3 bucket ───────────────────────────────────────
resource "aws_s3_bucket" "ses_inbox" {
  bucket = "onboardyou-ses-inbox-${var.env_postfix}"
}

resource "aws_s3_bucket_lifecycle_configuration" "ses_inbox" {
  bucket = aws_s3_bucket.ses_inbox.id

  rule {
    id     = "expire-raw-emails"
    status = "Enabled"

    filter {} # apply to all objects

    expiration {
      days = 7
    }
  }
}

# SES needs to be able to write to the bucket.
resource "aws_s3_bucket_policy" "ses_inbox" {
  bucket = aws_s3_bucket.ses_inbox.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowSESPut"
        Effect = "Allow"
        Principal = {
          Service = "ses.amazonaws.com"
        }
        Action   = "s3:PutObject"
        Resource = "${aws_s3_bucket.ses_inbox.arn}/*"
        Condition = {
          StringEquals = {
            "aws:SourceAccount" = var.aws_account_id
          }
        }
      }
    ]
  })
}

# ── SES receipt rule → store email in ses-inbox ───────────────
resource "aws_ses_receipt_rule" "store_to_s3" {
  name          = "store-to-s3"
  rule_set_name = aws_ses_receipt_rule_set.email_ingestion.rule_set_name
  enabled       = true
  scan_enabled  = true
  recipients    = [] # empty = catch-all for the domain

  s3_action {
    bucket_name       = aws_s3_bucket.ses_inbox.id
    object_key_prefix = "incoming/"
    position          = 1
  }

  depends_on = [aws_s3_bucket_policy.ses_inbox]
}

# ── EmailRoutes DynamoDB table ────────────────────────────────
module "email_routes_table" {
  source      = "../dynamodb"
  table_name  = "EmailRoutes-${var.env_postfix}"
  hash_key    = "sender_email"
  enable_pitr = false
}
