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

  security_groups = [openstack_networking_secgroup_v2.koprogo_vps.name]

  network {
    name = "Ext-Net"
  }

  metadata = {
    project     = "koprogo"
    environment = var.environment
    managed_by  = "terraform"
  }
}

# Network-level filtering in front of the host firewall (ufw), so the VPS is
# not protected by a single layer (CyFun PR.AC-5 / PR.PT-4).
# Egress stays open via the default rules created with the group.
resource "openstack_networking_secgroup_v2" "koprogo_vps" {
  provider    = openstack.ovh
  name        = "${var.instance_name}-sg"
  description = "KoproGo VPS: HTTP/HTTPS public, SSH admin-only"
}

resource "openstack_networking_secgroup_rule_v2" "ssh_admin" {
  provider          = openstack.ovh
  for_each          = toset(var.admin_ssh_cidrs)
  security_group_id = openstack_networking_secgroup_v2.koprogo_vps.id
  direction         = "ingress"
  ethertype         = strcontains(each.value, ":") ? "IPv6" : "IPv4"
  protocol          = "tcp"
  port_range_min    = 22
  port_range_max    = 22
  remote_ip_prefix  = each.value
}

# Public web entrypoint: HTTP (ACME + redirect) and HTTPS must be reachable
# from any address. Only 80/443 are opened this way; SSH stays admin-only.
#trivy:ignore:OPNSTK-0003
resource "openstack_networking_secgroup_rule_v2" "web" {
  provider = openstack.ovh
  for_each = {
    "http-v4"  = { port = 80, ethertype = "IPv4", prefix = "0.0.0.0/0" }
    "https-v4" = { port = 443, ethertype = "IPv4", prefix = "0.0.0.0/0" }
    "http-v6"  = { port = 80, ethertype = "IPv6", prefix = "::/0" }
    "https-v6" = { port = 443, ethertype = "IPv6", prefix = "::/0" }
  }
  security_group_id = openstack_networking_secgroup_v2.koprogo_vps.id
  direction         = "ingress"
  ethertype         = each.value.ethertype
  protocol          = "tcp"
  port_range_min    = each.value.port
  port_range_max    = each.value.port
  remote_ip_prefix  = each.value.prefix
}
