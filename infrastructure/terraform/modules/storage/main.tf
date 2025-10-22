# Storage Module
# Creates Object Storage containers for backups and artifacts

# Object Storage Container for Backups
resource "ovh_cloud_project_region_storage_container" "backups" {
  service_name = var.ovh_service_name
  region_name  = var.region
  name         = "${var.cluster_name}-${var.environment}-backups"

  # Storage class (standard, high-speed, archive)
  storage_class = var.backup_storage_class

  # Versioning
  versioning = var.enable_versioning
}

# Object Storage Container for Artifacts (Docker images, helm charts)
resource "ovh_cloud_project_region_storage_container" "artifacts" {
  service_name = var.ovh_service_name
  region_name  = var.region
  name         = "${var.cluster_name}-${var.environment}-artifacts"

  storage_class = "high-speed"
  versioning    = false
}

# Object Storage Container for Logs
resource "ovh_cloud_project_region_storage_container" "logs" {
  service_name = var.ovh_service_name
  region_name  = var.region
  name         = "${var.cluster_name}-${var.environment}-logs"

  storage_class = "standard"
  versioning    = false
}

# S3 User for backup access
resource "ovh_cloud_project_user_s3_credential" "backup_user" {
  service_name = var.ovh_service_name
}

# Backup Policy (lifecycle rules)
resource "ovh_cloud_project_region_storage_container_lifecycle" "backup_lifecycle" {
  count = var.enable_backup_lifecycle ? 1 : 0

  service_name   = var.ovh_service_name
  region_name    = var.region
  container_name = ovh_cloud_project_region_storage_container.backups.name

  # Delete objects older than retention period
  rule {
    name    = "delete-old-backups"
    enabled = true

    filter {
      prefix = ""
    }

    expiration {
      days = var.backup_retention_days
    }
  }

  # Transition to archive after 30 days
  rule {
    name    = "archive-old-backups"
    enabled = var.enable_backup_archiving

    filter {
      prefix = ""
    }

    transition {
      days          = 30
      storage_class = "GLACIER"
    }
  }
}
