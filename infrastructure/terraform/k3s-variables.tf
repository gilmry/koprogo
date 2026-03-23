# Terraform Variables: K3s Cluster on OVH OpenStack
# Issue #266: K3s Provisioning with Terraform + OpenStack

variable "k3s_master_flavor" {
  description = "OVH flavor for K3s master node (e.g., 'b2-7' = 2 vCPU, 7GB RAM)"
  type        = string
  default     = "b2-7"

  validation {
    condition     = contains(["b2-7", "b2-15", "b2-30"], var.k3s_master_flavor)
    error_message = "Master flavor must be b2-7, b2-15, or b2-30."
  }
}

variable "k3s_agent_flavor" {
  description = "OVH flavor for K3s agent nodes (e.g., 'b2-7' = 2 vCPU, 7GB RAM)"
  type        = string
  default     = "b2-7"

  validation {
    condition     = contains(["b2-7", "b2-15", "b2-30"], var.k3s_agent_flavor)
    error_message = "Agent flavor must be b2-7, b2-15, or b2-30."
  }
}

variable "k3s_agent_count" {
  description = "Number of K3s agent nodes (minimum 1 for single-node, 2+ for HA)"
  type        = number
  default     = 2

  validation {
    condition     = var.k3s_agent_count >= 1 && var.k3s_agent_count <= 10
    error_message = "Agent count must be between 1 and 10."
  }
}

variable "region" {
  description = "OVH region for instances (GRA=Gravelines/Roubaix, BHS=Beauharnois Canada, SGP=Singapore)"
  type        = string
  default     = "GRA"

  validation {
    condition     = contains(["GRA", "BHS", "SGP", "SBG", "DE1", "WAW"], var.region)
    error_message = "Region must be a valid OVH region code."
  }
}

variable "environment" {
  description = "Environment name (production, staging, development)"
  type        = string
  default     = "production"

  validation {
    condition     = contains(["production", "staging", "development"], var.environment)
    error_message = "Environment must be production, staging, or development."
  }
}

variable "instance_name" {
  description = "Base name for all instances (will be suffixed with -master, -agent-1, etc.)"
  type        = string
  default     = "koprogo"

  validation {
    condition     = can(regex("^[a-z0-9-]+$", var.instance_name))
    error_message = "Instance name must contain only lowercase letters, numbers, and hyphens."
  }
}

variable "ssh_public_key_path" {
  description = "Path to SSH public key file for instance access"
  type        = string
  default     = "~/.ssh/id_rsa.pub"
}

variable "ssh_private_key_path" {
  description = "Path to SSH private key file (for Ansible)"
  type        = string
  default     = "~/.ssh/id_rsa"
}

variable "ovh_endpoint" {
  description = "OVH API endpoint (ovh-eu, ovh-ca, ovh-au, ovh-us, ovh-asia)"
  type        = string
  default     = "ovh-eu"

  validation {
    condition     = contains(["ovh-eu", "ovh-ca", "ovh-au", "ovh-us", "ovh-asia"], var.ovh_endpoint)
    error_message = "OVH endpoint must be ovh-eu, ovh-ca, ovh-au, ovh-us, or ovh-asia."
  }
}
