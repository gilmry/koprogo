variable "instance_name" {
  description = "Name of the VPS instance"
  type        = string
}

variable "flavor_name" {
  description = "OVH flavor (d2-2 = 1 vCPU/2GB, b2-7 = 2 vCPU/7GB)"
  type        = string
  default     = "d2-2"
}

variable "image_name" {
  description = "OS image name"
  type        = string
  default     = "Ubuntu 22.04"
}

variable "region" {
  description = "OVH region (GRA11 = Gravelines France, low carbon)"
  type        = string
  default     = "GRA11"
}

variable "environment" {
  description = "Environment name (dev, integration, staging, production)"
  type        = string
}

variable "ssh_public_key_path" {
  description = "Path to SSH public key"
  type        = string
  default     = "~/.ssh/id_rsa.pub"
}

variable "ovh_endpoint" {
  description = "OVH API endpoint"
  type        = string
  default     = "ovh-eu"
}

variable "admin_ssh_cidrs" {
  description = "CIDRs allowed to reach SSH (admin IPs, bastion, VPN). Pass via TF_VAR_admin_ssh_cidrs; keep real values out of git."
  type        = list(string)

  validation {
    condition     = length(var.admin_ssh_cidrs) > 0
    error_message = "admin_ssh_cidrs must contain at least one CIDR."
  }

  validation {
    condition     = alltrue([for c in var.admin_ssh_cidrs : can(cidrhost(c, 0))])
    error_message = "Every entry of admin_ssh_cidrs must be a valid CIDR."
  }

  validation {
    condition     = !anytrue([for c in var.admin_ssh_cidrs : contains(["0.0.0.0/0", "::/0"], c)])
    error_message = "admin_ssh_cidrs must not open SSH to the whole Internet (0.0.0.0/0 or ::/0)."
  }
}
