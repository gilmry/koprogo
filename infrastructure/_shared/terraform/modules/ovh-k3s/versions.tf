# Provider requirements for the ovh-k3s module.
# The module consumes an *aliased* OpenStack provider (openstack.ovh) passed in
# by the caller. Pinned to 1.x: compute_secgroup_v2 / compute_floatingip_* were
# removed in openstack provider 2.0 (migration tracked separately).
terraform {
  required_version = ">= 1.5.0"

  required_providers {
    openstack = {
      source                = "terraform-provider-openstack/openstack"
      version               = "~> 1.54"
      configuration_aliases = [openstack.ovh]
    }
  }
}
