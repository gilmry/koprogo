# Development Environment Terraform Configuration
# Single control plane + 1 worker for cost optimization

terraform {
  required_version = ">= 1.5"

  backend "s3" {
    bucket   = "koprogo-terraform-state"
    key      = "dev/terraform.tfstate"
    region   = "gra"
    endpoint = "s3.gra.io.cloud.ovh.net"

    skip_credentials_validation = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
    skip_metadata_api_check     = true
    skip_s3_checksum            = true
  }
}

# Local variables
locals {
  environment  = "dev"
  cluster_name = "koprogo-dev"
  region       = "GRA11"

  tags = {
    Environment = "development"
    Project     = "koprogo"
    ManagedBy   = "terraform"
    CostCenter  = "engineering"
  }
}

# Networking Module
module "networking" {
  source = "../../modules/networking"

  ovh_service_name = var.ovh_service_name
  environment      = local.environment
  cluster_name     = local.cluster_name
  region           = local.region

  # Network configuration
  vlan_id      = 10
  subnet_cidr  = "10.10.0.0/24"
  subnet_start = "10.10.0.10"
  subnet_end   = "10.10.0.250"

  # Security
  admin_ips = var.admin_ips

  # No load balancer in dev (cost optimization)
  enable_load_balancer = false
}

# Compute Instances Module
module "instances" {
  source = "../../modules/ovh-instances"

  ovh_service_name = var.ovh_service_name
  environment      = local.environment
  cluster_name     = local.cluster_name
  region           = local.region

  # Control plane: 1 node (non-HA) with b2-7 (2 vCPU, 7GB RAM)
  control_plane_count  = 1
  control_plane_flavor = "b2-7"

  # Workers: 1 node with b2-15 (4 vCPU, 15GB RAM)
  worker_count  = 1
  worker_flavor = "b2-15"

  # SSH
  ssh_public_key = var.ssh_public_key

  # Network
  private_network_name = module.networking.private_network_name

  # Anti-affinity not needed with single worker
  enable_anti_affinity = false

  # Longhorn storage: 1 volume per worker, 50GB
  longhorn_volumes_per_node = 1
  longhorn_volume_size      = 50

  tags = local.tags

  depends_on = [module.networking]
}

# Storage Module
module "storage" {
  source = "../../modules/storage"

  ovh_service_name = var.ovh_service_name
  environment      = local.environment
  cluster_name     = local.cluster_name
  region           = "GRA"

  # Standard storage for dev
  backup_storage_class = "standard"
  enable_versioning    = false

  # Shorter retention for dev
  backup_retention_days   = 30
  enable_backup_archiving = false
}

# DNS Module (optional in dev)
module "dns" {
  source = "../../modules/dns"
  count  = var.create_dns_records ? 1 : 0

  domain_name = var.domain_name
  dns_ttl     = 300

  # Use first worker IP for all services in dev
  api_ip_address        = length(module.instances.worker_ips) > 0 ? module.instances.worker_ips[0] : ""
  app_ip_address        = length(module.instances.worker_ips) > 0 ? module.instances.worker_ips[0] : ""
  monitoring_ip_address = length(module.instances.worker_ips) > 0 ? module.instances.worker_ips[0] : ""

  create_api_record        = true
  create_app_record        = true
  create_monitoring_records = false # No monitoring in dev
  create_wildcard_record   = false
  create_caa_records       = false
}
