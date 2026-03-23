variable "instance_name" {
  description = "Base name for cluster instances"
  type        = string
  default     = "koprogo"
  validation {
    condition     = can(regex("^[a-z0-9-]+$", var.instance_name))
    error_message = "Instance name must contain only lowercase letters, numbers, and hyphens."
  }
}

variable "environment" {
  description = "Environment name"
  type        = string
  validation {
    condition     = contains(["dev", "integration", "staging", "production"], var.environment)
    error_message = "Environment must be dev, integration, staging, or production."
  }
}

variable "k3s_master_flavor" {
  description = "OVH flavor for K3s master node"
  type        = string
  default     = "b2-7"
  validation {
    condition     = contains(["b2-7", "b2-15", "b2-30"], var.k3s_master_flavor)
    error_message = "Master flavor must be b2-7, b2-15, or b2-30."
  }
}

variable "k3s_agent_flavor" {
  description = "OVH flavor for K3s agent nodes"
  type        = string
  default     = "b2-7"
  validation {
    condition     = contains(["b2-7", "b2-15", "b2-30"], var.k3s_agent_flavor)
    error_message = "Agent flavor must be b2-7, b2-15, or b2-30."
  }
}

variable "k3s_agent_count" {
  description = "Number of K3s agent nodes (0 for single-node)"
  type        = number
  default     = 0
  validation {
    condition     = var.k3s_agent_count >= 0 && var.k3s_agent_count <= 10
    error_message = "Agent count must be between 0 and 10."
  }
}

variable "region" {
  description = "OVH region"
  type        = string
  default     = "GRA"
}

variable "ssh_public_key_path" {
  description = "Path to SSH public key"
  type        = string
  default     = "~/.ssh/id_rsa.pub"
}

variable "ssh_private_key_path" {
  description = "Path to SSH private key (for Ansible)"
  type        = string
  default     = "~/.ssh/id_rsa"
}

variable "ovh_endpoint" {
  description = "OVH API endpoint"
  type        = string
  default     = "ovh-eu"
}
