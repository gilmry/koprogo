# Variables for Networking Module

variable "ovh_service_name" {
  description = "OVH Cloud Project service name"
  type        = string
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
}

variable "cluster_name" {
  description = "K3s cluster name"
  type        = string
  default     = "koprogo"
}

variable "region" {
  description = "OVH region"
  type        = string
  default     = "GRA11"
}

# vRack/Private Network Configuration
variable "vlan_id" {
  description = "VLAN ID for private network"
  type        = number
  default     = 0
}

variable "subnet_cidr" {
  description = "CIDR block for the private subnet"
  type        = string
  default     = "10.0.0.0/24"
}

variable "subnet_start" {
  description = "First IP of the subnet range"
  type        = string
  default     = "10.0.0.2"
}

variable "subnet_end" {
  description = "Last IP of the subnet range"
  type        = string
  default     = "10.0.0.254"
}

# Security Configuration
variable "admin_ips" {
  description = "List of admin IP addresses for SSH access"
  type        = list(string)
  default     = []
}

# Load Balancer Configuration
variable "enable_load_balancer" {
  description = "Enable load balancer for ingress"
  type        = bool
  default     = false
}
