# Variables for DNS Module

variable "domain_name" {
  description = "Domain name (e.g., koprogo.io)"
  type        = string
}

variable "dns_ttl" {
  description = "TTL for DNS records in seconds"
  type        = number
  default     = 300
}

# API Record
variable "create_api_record" {
  description = "Create DNS record for API"
  type        = bool
  default     = true
}

variable "api_ip_address" {
  description = "IP address for API endpoint"
  type        = string
  default     = ""
}

# App Record
variable "create_app_record" {
  description = "Create DNS record for frontend app"
  type        = bool
  default     = true
}

variable "app_ip_address" {
  description = "IP address for frontend app"
  type        = string
  default     = ""
}

# Monitoring Records
variable "create_monitoring_records" {
  description = "Create DNS records for monitoring stack"
  type        = bool
  default     = true
}

variable "monitoring_ip_address" {
  description = "IP address for monitoring services"
  type        = string
  default     = ""
}

# Wildcard Record
variable "create_wildcard_record" {
  description = "Create wildcard DNS record"
  type        = bool
  default     = false
}

variable "wildcard_ip_address" {
  description = "IP address for wildcard record"
  type        = string
  default     = ""
}

# Verification
variable "verification_txt" {
  description = "TXT record for domain verification"
  type        = string
  default     = ""
}

# CAA Records
variable "create_caa_records" {
  description = "Create CAA records for Let's Encrypt"
  type        = bool
  default     = true
}
