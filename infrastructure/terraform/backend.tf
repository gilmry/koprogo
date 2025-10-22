# Terraform Backend Configuration
# Store state in OVH Object Storage for collaboration and locking

terraform {
  required_version = ">= 1.5"

  required_providers {
    ovh = {
      source  = "ovh/ovh"
      version = "~> 0.35"
    }
    local = {
      source  = "hashicorp/local"
      version = "~> 2.4"
    }
    tls = {
      source  = "hashicorp/tls"
      version = "~> 4.0"
    }
  }

  # Backend configuration for state storage
  # Initialize with: terraform init -backend-config=backend.hcl
  backend "s3" {
    # OVH Object Storage S3-compatible
    bucket = "koprogo-terraform-state"
    key    = "terraform.tfstate"
    region = "gra"

    # OVH S3 endpoint
    endpoint = "s3.gra.io.cloud.ovh.net"

    # Disable AWS-specific features
    skip_credentials_validation = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
    skip_metadata_api_check     = true
    skip_s3_checksum            = true

    # Enable state locking
    dynamodb_table = "terraform-locks"
  }
}

# OVH Provider configuration
provider "ovh" {
  endpoint = var.ovh_endpoint
  # Credentials are read from environment variables:
  # OVH_APPLICATION_KEY, OVH_APPLICATION_SECRET, OVH_CONSUMER_KEY
}
