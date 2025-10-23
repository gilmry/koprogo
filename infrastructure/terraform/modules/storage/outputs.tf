# Outputs for Storage Module

output "backup_container_name" {
  description = "Name of the backup container"
  value       = ovh_cloud_project_region_storage_container.backups.name
}

output "backup_container_url" {
  description = "URL of the backup container"
  value       = ovh_cloud_project_region_storage_container.backups.storage_url
}

output "artifacts_container_name" {
  description = "Name of the artifacts container"
  value       = ovh_cloud_project_region_storage_container.artifacts.name
}

output "logs_container_name" {
  description = "Name of the logs container"
  value       = ovh_cloud_project_region_storage_container.logs.name
}

output "s3_access_key" {
  description = "S3 access key for backups"
  value       = ovh_cloud_project_user_s3_credential.backup_user.access_key_id
  sensitive   = true
}

output "s3_secret_key" {
  description = "S3 secret key for backups"
  value       = ovh_cloud_project_user_s3_credential.backup_user.secret_access_key
  sensitive   = true
}
