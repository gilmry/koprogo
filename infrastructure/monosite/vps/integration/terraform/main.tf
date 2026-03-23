# KoproGo VPS - Integration Environment
module "vps" {
  source = "../../../_shared/terraform/modules/ovh-vps"

  instance_name       = var.instance_name
  flavor_name         = var.flavor_name
  environment         = var.environment
  region              = var.region
  ssh_public_key_path = var.ssh_public_key_path
  ovh_endpoint        = var.ovh_endpoint
}

provider "ovh" {
  endpoint = var.ovh_endpoint
}

provider "openstack" {
  alias  = "ovh"
  region = var.region
}

variable "instance_name" { type = string }
variable "flavor_name" { type = string; default = "d2-2" }
variable "environment" { type = string }
variable "region" { type = string; default = "GRA11" }
variable "ssh_public_key_path" { type = string; default = "~/.ssh/id_rsa.pub" }
variable "ovh_endpoint" { type = string; default = "ovh-eu" }

output "vps_ip" { value = module.vps.vps_ip }
output "ssh_command" { value = module.vps.ssh_command }
