terraform {
  required_version = ">= 1.0"
  required_providers {
    ovh = {
      source  = "ovh/ovh"
      version = "~> 0.35"
    }
  }
}

# Configure OVH Provider
# Credentials via environment variables:
# - OVH_ENDPOINT (ovh-eu, ovh-ca, etc.)
# - OVH_APPLICATION_KEY
# - OVH_APPLICATION_SECRET
# - OVH_CONSUMER_KEY
provider "ovh" {
  endpoint = var.ovh_endpoint
}

# VPS Value - Le moins cher (1 vCPU, 2GB RAM, 40GB NVMe)
# Prix: ~7€ TTC/mois (OVH France 2025)
resource "ovh_cloud_project_instance" "koprogo_vps" {
  service_name = var.ovh_service_name
  name         = var.instance_name

  # VPS Value (d2-2)
  flavor_name = "d2-2"

  # Ubuntu 22.04 LTS
  image_name = "Ubuntu 22.04"

  # Région France (GRA = Gravelines, datacenter bas carbone)
  region = var.region

  # Clé SSH publique
  ssh_public_key = file(var.ssh_public_key_path)

  # Network configuration
  networks {
    name = "Ext-Net"
  }
}

# Output VPS IP
output "vps_ip" {
  description = "IP publique du VPS KoproGo"
  value       = ovh_cloud_project_instance.koprogo_vps.access_ip_v4
}

output "vps_id" {
  description = "ID de l'instance VPS"
  value       = ovh_cloud_project_instance.koprogo_vps.id
}

output "ssh_command" {
  description = "Commande SSH pour se connecter"
  value       = "ssh ubuntu@${ovh_cloud_project_instance.koprogo_vps.access_ip_v4}"
}
