# KoproGo K8s Multisite - Staging Environment
module "k8s" {
  source = "../../../../_shared/terraform/modules/ovh-k3s"

  instance_name        = var.instance_name
  environment          = var.environment
  k3s_master_flavor    = var.k3s_master_flavor
  k3s_agent_flavor     = var.k3s_agent_flavor
  k3s_agent_count      = var.k3s_agent_count
  region               = var.region
  ssh_public_key_path  = var.ssh_public_key_path
  ssh_private_key_path = var.ssh_private_key_path
  ovh_endpoint         = var.ovh_endpoint
}

provider "openstack" {
  alias  = "ovh"
  region = var.region
}

variable "instance_name"        { type = string }
variable "environment"          { type = string }
variable "k3s_master_flavor"    { type = string; default = "b2-7" }
variable "k3s_agent_flavor"     { type = string; default = "b2-7" }
variable "k3s_agent_count"      { type = number; default = 1 }
variable "region"               { type = string; default = "GRA" }
variable "ssh_public_key_path"  { type = string; default = "~/.ssh/id_rsa.pub" }
variable "ssh_private_key_path" { type = string; default = "~/.ssh/id_rsa" }
variable "ovh_endpoint"         { type = string; default = "ovh-eu" }

output "master_ip"    { value = module.k8s.master_ip_public }
output "cluster_info" { value = module.k8s.cluster_info }
