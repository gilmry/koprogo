terraform {
  required_version = ">= 1.0"
  required_providers {
    ovh = {
      source  = "ovh/ovh"
      version = "~> 0.51"
    }
    openstack = {
      source  = "terraform-provider-openstack/openstack"
      version = "~> 2.1"
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

# Configure OpenStack Provider (OVH Cloud uses OpenStack)
# Authentication via environment variables:
# - OS_AUTH_URL, OS_PROJECT_ID, OS_USERNAME, OS_PASSWORD, OS_REGION_NAME
# These will be loaded from .env or openrc file
provider "openstack" {
  alias  = "ovh"
  region = var.region
}

# SSH Key Pair for instance access
resource "openstack_compute_keypair_v2" "koprogo_key" {
  provider   = openstack.ovh
  name       = "${var.instance_name}-key"
  public_key = file(var.ssh_public_key_path)
}

# VPS Value - Le moins cher (1 vCPU, 2GB RAM, 40GB NVMe)
# Prix: ~7€ TTC/mois (OVH France 2025)
resource "openstack_compute_instance_v2" "koprogo_vps" {
  provider = openstack.ovh
  name     = var.instance_name

  # VPS Value (d2-2)
  flavor_name = "d2-2"

  # Ubuntu 22.04 LTS
  image_name = "Ubuntu 22.04"

  # SSH Key
  key_pair = openstack_compute_keypair_v2.koprogo_key.name

  # Network configuration - Use Ext-Net for public network
  network {
    name = "Ext-Net"
  }

  # Metadata
  metadata = {
    project     = "koprogo"
    environment = "production"
  }
}

# Output VPS IP
output "vps_ip" {
  description = "IP publique du VPS KoproGo"
  value       = openstack_compute_instance_v2.koprogo_vps.access_ip_v4
}

output "vps_id" {
  description = "ID de l'instance VPS"
  value       = openstack_compute_instance_v2.koprogo_vps.id
}

output "ssh_command" {
  description = "Commande SSH pour se connecter"
  value       = "ssh ubuntu@${openstack_compute_instance_v2.koprogo_vps.access_ip_v4}"
}
