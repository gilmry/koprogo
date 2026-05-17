# Provider requirements for the ovh-vps module.
# The module consumes an *aliased* OpenStack provider (openstack.ovh) passed in
# by the caller — that alias MUST be declared via configuration_aliases or
# `terraform validate` fails ("provider configuration not present").
terraform {
  required_version = ">= 1.5.0"

  required_providers {
    openstack = {
      source                = "terraform-provider-openstack/openstack"
      version               = ">= 1.53.0"
      configuration_aliases = [openstack.ovh]
    }
  }
}
