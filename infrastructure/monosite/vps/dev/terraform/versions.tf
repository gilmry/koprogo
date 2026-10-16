# Provider requirements for the production VPS root config.
terraform {
  required_version = ">= 1.10.0" # use_lockfile in the S3 backend

  required_providers {
    ovh = {
      source  = "ovh/ovh"
      version = ">= 0.40.0"
    }
    openstack = {
      source  = "terraform-provider-openstack/openstack"
      version = ">= 1.53.0"
    }
  }
}
