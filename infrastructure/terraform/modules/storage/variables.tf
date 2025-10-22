# Variables for Storage Module

variable "ovh_service_name" {
  description = "OVH Cloud Project service name"
  type        = string
}

variable "environment" {
  description = "Environment name"
  type        = string
}

variable "cluster_name" {
  description = "Cluster name"
  type        = string
  default     = "koprogo"
}

variable "region" {
  description = "OVH region"
  type        = string
  default     = "GRA"
}

variable "backup_storage_class" {
  description = "Storage class for backups"
  type        = string
  default     = "standard"
  validation {
    condition     = contains(["standard", "high-speed", "archive"], var.backup_storage_class)
    error_message = "Storage class must be standard, high-speed, or archive."
  }
}

variable "enable_versioning" {
  description = "Enable versioning for backup container"
  type        = bool
  default     = true
}

variable "enable_backup_lifecycle" {
  description = "Enable lifecycle policies for backups"
  type        = bool
  default     = true
}

variable "backup_retention_days" {
  description = "Number of days to retain backups"
  type        = number
  default     = 90
}

variable "enable_backup_archiving" {
  description = "Enable automatic archiving of old backups"
  type        = bool
  default     = true
}
