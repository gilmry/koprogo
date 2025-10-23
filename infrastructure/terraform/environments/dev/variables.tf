# Variables for Development Environment

variable "ovh_service_name" {
  description = "OVH Cloud Project service name"
  type        = string
}

variable "ovh_endpoint" {
  description = "OVH API endpoint"
  type        = string
  default     = "ovh-eu"
}

variable "ssh_public_key" {
  description = "SSH public key for instances"
  type        = string
}

variable "admin_ips" {
  description = "List of admin IP addresses for SSH access"
  type        = list(string)
  default     = []
}

variable "domain_name" {
  description = "Domain name for DNS records"
  type        = string
  default     = "koprogo.dev"
}

variable "create_dns_records" {
  description = "Create DNS records"
  type        = bool
  default     = false
}
