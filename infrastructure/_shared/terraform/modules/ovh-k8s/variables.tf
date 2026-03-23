# Placeholder - use ovh-k3s module with higher agent_count for now
variable "instance_name" {
  type    = string
  default = "koprogo-k8s"
}

variable "environment" {
  type    = string
  default = "production"
}
