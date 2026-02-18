##──────────────────────────────────────────────────────────────
## CSV Upload Bucket
##
## Private S3 bucket for HRIS CSV uploads.
## Objects are keyed: {organizationId}/{customerCompanyId}/{filename}
##
## CORS allows the frontend to PUT directly via presigned URLs.
## Lifecycle rule expires objects after `retention_days`.
##──────────────────────────────────────────────────────────────

resource "aws_s3_bucket" "csv_uploads" {
  bucket        = "${var.project_prefix}-csv-uploads-${var.env_postfix}"
  force_destroy = var.environment == "dev"

  tags = {
    Name = "${var.project_prefix}-csv-uploads"
  }
}

# ── Block all public access ─────────────────────────────────

resource "aws_s3_bucket_public_access_block" "csv_uploads" {
  bucket = aws_s3_bucket.csv_uploads.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# ── Server-side encryption (AES-256) ────────────────────────

resource "aws_s3_bucket_server_side_encryption_configuration" "csv_uploads" {
  bucket = aws_s3_bucket.csv_uploads.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# ── CORS — allow frontend presigned PUT uploads ─────────────

resource "aws_s3_bucket_cors_configuration" "csv_uploads" {
  bucket = aws_s3_bucket.csv_uploads.id

  cors_rule {
    allowed_headers = ["*"]
    allowed_methods = ["PUT", "GET"]
    allowed_origins = var.allowed_origins
    expose_headers  = ["ETag"]
    max_age_seconds = 3600
  }
}

# ── Lifecycle — auto-expire uploads ─────────────────────────

resource "aws_s3_bucket_lifecycle_configuration" "csv_uploads" {
  bucket = aws_s3_bucket.csv_uploads.id

  rule {
    id     = "expire-csv-uploads"
    status = "Enabled"

    filter {}

    expiration {
      days = var.retention_days
    }
  }
}
