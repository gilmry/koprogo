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
