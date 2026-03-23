# Terraform Module: Single OVH VPS Instance
# Provisions a single VPS on OVH OpenStack for Docker Compose deployments

resource "openstack_compute_keypair_v2" "koprogo_key" {
  provider   = openstack.ovh
  name       = "${var.instance_name}-key"
  public_key = file(var.ssh_public_key_path)
}

resource "openstack_compute_instance_v2" "koprogo_vps" {
  provider    = openstack.ovh
  name        = var.instance_name
  flavor_name = var.flavor_name
  image_name  = var.image_name
  key_pair    = openstack_compute_keypair_v2.koprogo_key.name

  network {
    name = "Ext-Net"
  }

  metadata = {
    project     = "koprogo"
    environment = var.environment
    managed_by  = "terraform"
  }
}