# Variables for OVH Instances Module

variable "ovh_service_name" {
  description = "OVH Cloud Project service name"
  type        = string
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be dev, staging, or prod."
  }
}

variable "cluster_name" {
  description = "K3s cluster name"
  type        = string
  default     = "koprogo"
}

variable "region" {
  description = "OVH region (e.g., GRA11, SBG5)"
  type        = string
  default     = "GRA11"
}

variable "image_name" {
  description = "OS image name"
  type        = string
  default     = "Ubuntu 22.04"
}

# Control Plane Configuration
variable "control_plane_count" {
  description = "Number of control plane nodes (1 or 3 for HA)"
  type        = number
  validation {
    condition     = var.control_plane_count == 1 || var.control_plane_count == 3
    error_message = "Control plane count must be 1 or 3."
  }
}

variable "control_plane_flavor" {
  description = "Flavor for control plane nodes"
  type        = string
  default     = "b2-15" # 4 vCPU, 15GB RAM
}

# Worker Configuration
variable "worker_count" {
  description = "Number of worker nodes"
  type        = number
}

variable "worker_flavor" {
  description = "Flavor for worker nodes"
  type        = string
  default     = "b2-15" # 4 vCPU, 15GB RAM
}

# SSH Configuration
variable "ssh_public_key" {
  description = "SSH public key for instances"
  type        = string
}

# Network Configuration
variable "private_network_name" {
  description = "Name of the private network"
  type        = string
}

# Anti-affinity
variable "enable_anti_affinity" {
  description = "Enable anti-affinity for workers"
  type        = bool
  default     = true
}

# Longhorn Storage
variable "longhorn_volumes_per_node" {
  description = "Number of Longhorn volumes per worker node"
  type        = number
  default     = 1
}

variable "longhorn_volume_size" {
  description = "Size of each Longhorn volume in GB"
  type        = number
  default     = 100
}

# Tags
variable "tags" {
  description = "Additional tags for resources"
  type        = map(string)
  default     = {}
}
