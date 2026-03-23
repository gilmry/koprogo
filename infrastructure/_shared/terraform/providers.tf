# Shared Terraform Provider Definitions for KoproGo
# All environments reuse these provider configurations

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