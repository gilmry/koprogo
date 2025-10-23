# Production Environment Terraform Configuration
# 3 control plane (HA) + 3 workers for high availability and performance

terraform {
  required_version = ">= 1.5"

  backend "s3" {
    bucket   = "koprogo-terraform-state"
    key      = "prod/terraform.tfstate"
    region   = "gra"
    endpoint = "s3.gra.io.cloud.ovh.net"

    skip_credentials_validation = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
    skip_metadata_api_check     = true
    skip_s3_checksum            = true
  }
}

locals {
  environment  = "prod"
  cluster_name = "koprogo-prod"
  region       = "GRA11"

  tags = {
    Environment = "production"
    Project     = "koprogo"
    ManagedBy   = "terraform"
    CostCenter  = "operations"
    Criticality = "high"
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
  vlan_id      = 30
  subnet_cidr  = "10.30.0.0/24"
  subnet_start = "10.30.0.10"
  subnet_end   = "10.30.0.250"

  # Security - stricter in prod
  admin_ips = var.admin_ips

  # Load balancer enabled in prod
  enable_load_balancer = true
}

# Compute Instances Module
module "instances" {
  source = "../../modules/ovh-instances"

  ovh_service_name = var.ovh_service_name
  environment      = local.environment
  cluster_name     = local.cluster_name
  region           = local.region

  # Control plane: 3 nodes (HA) with b2-15 (4 vCPU, 15GB RAM)
  control_plane_count  = 3
  control_plane_flavor = "b2-15"

  # Workers: 3 nodes with b2-30 (8 vCPU, 30GB RAM)
  worker_count  = 3
  worker_flavor = "b2-30"

  # SSH
  ssh_public_key = var.ssh_public_key

  # Network
  private_network_name = module.networking.private_network_name

  # Anti-affinity enabled for HA
  enable_anti_affinity = true

  # Longhorn storage: 2 volumes per worker, 200GB each
  longhorn_volumes_per_node = 2
  longhorn_volume_size      = 200

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

  # High-speed storage for prod
  backup_storage_class = "high-speed"
  enable_versioning    = true

  # Longer retention for prod
  backup_retention_days   = 90
  enable_backup_archiving = true
  enable_backup_lifecycle = true
}

# DNS Module
module "dns" {
  source = "../../modules/dns"

  domain_name = var.domain_name
  dns_ttl     = 300

  # Use load balancer IP for all services
  api_ip_address        = module.networking.load_balancer_ip
  app_ip_address        = module.networking.load_balancer_ip
  monitoring_ip_address = module.networking.load_balancer_ip
  wildcard_ip_address   = module.networking.load_balancer_ip

  create_api_record         = true
  create_app_record         = true
  create_monitoring_records = true
  create_wildcard_record    = true
  create_caa_records        = true

  verification_txt = var.domain_verification_txt
}
