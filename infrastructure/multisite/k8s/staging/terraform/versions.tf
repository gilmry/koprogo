# Provider requirements for this root config.
terraform {
  required_version = ">= 1.10.0" # use_lockfile in the S3 backend

  required_providers {
    openstack = {
      source  = "terraform-provider-openstack/openstack"
      version = "~> 1.54"
    }
  }
}
