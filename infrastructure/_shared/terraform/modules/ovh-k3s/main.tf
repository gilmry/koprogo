# Terraform Module: K3s Cluster on OVH OpenStack
# Provisions master + agent nodes for K3s Kubernetes

locals {
  k3s_version = "v1.29.2+k3s1"
  common_tags = {
    Environment = var.environment
    Project     = var.instance_name
    ManagedBy   = "Terraform"
    GDPR        = "EU-West"
  }
}

# K3s Network
resource "openstack_networking_network_v2" "k3s_network" {
  provider       = openstack.ovh
  name           = "${var.instance_name}-network"
  admin_state_up = true
}

resource "openstack_networking_subnet_v2" "k3s_subnet" {
  provider        = openstack.ovh
  name            = "${var.instance_name}-subnet"
  network_id      = openstack_networking_network_v2.k3s_network.id
  cidr            = "10.0.0.0/24"
  ip_version      = 4
  dns_nameservers = ["1.1.1.1", "8.8.8.8", "9.9.9.9"]
  enable_dhcp     = true
  gateway_ip      = "10.0.0.1"

  allocation_pools {
    start = "10.0.0.10"
    end   = "10.0.0.254"
  }
}

resource "openstack_networking_router_v2" "k3s_router" {
  provider            = openstack.ovh
  name                = "${var.instance_name}-router"
  admin_state_up      = true
  external_network_id = data.openstack_networking_network_v2.ext_net.id
}

resource "openstack_networking_router_interface_v2" "k3s_router_interface" {
  provider  = openstack.ovh
  router_id = openstack_networking_router_v2.k3s_router.id
  subnet_id = openstack_networking_subnet_v2.k3s_subnet.id
}

# Security Group
resource "openstack_compute_secgroup_v2" "k3s_cluster_sg" {
  provider    = openstack.ovh
  name        = "${var.instance_name}-k3s-sg"
  description = "K3s Kubernetes cluster security group"

  rule { from_port = 22;    to_port = 22;    ip_protocol = "tcp"; cidr = "0.0.0.0/0" }
  rule { from_port = 6443;  to_port = 6443;  ip_protocol = "tcp"; cidr = "10.0.0.0/24" }
  rule { from_port = 8472;  to_port = 8472;  ip_protocol = "udp"; cidr = "10.0.0.0/24" }
  rule { from_port = 80;    to_port = 80;    ip_protocol = "tcp"; cidr = "0.0.0.0/0" }
  rule { from_port = 443;   to_port = 443;   ip_protocol = "tcp"; cidr = "0.0.0.0/0" }
  rule { from_port = 10250; to_port = 10250; ip_protocol = "tcp"; cidr = "10.0.0.0/24" }
  rule { from_port = 9100;  to_port = 9100;  ip_protocol = "tcp"; cidr = "10.0.0.0/24" }
  rule { from_port = -1;    to_port = -1;    ip_protocol = "tcp"; cidr = "10.0.0.0/24" }
  rule { from_port = -1;    to_port = -1;    ip_protocol = "udp"; cidr = "10.0.0.0/24" }
}

data "openstack_networking_network_v2" "ext_net" {
  provider = openstack.ovh
  name     = "Ext-Net"
}

data "openstack_images_image_v2" "ubuntu_image" {
  provider    = openstack.ovh
  name        = "Ubuntu 22.04"
  most_recent = true
  visibility  = "public"
}

# SSH Key
resource "openstack_compute_keypair_v2" "koprogo_key" {
  provider   = openstack.ovh
  name       = "${var.instance_name}-key"
  public_key = file(var.ssh_public_key_path)
}

# K3s Master
resource "openstack_compute_instance_v2" "k3s_master" {
  provider        = openstack.ovh
  name            = "${var.instance_name}-master"
  image_id        = data.openstack_images_image_v2.ubuntu_image.id
  flavor_name     = var.k3s_master_flavor
  key_pair        = openstack_compute_keypair_v2.koprogo_key.name
  security_groups = [openstack_compute_secgroup_v2.k3s_cluster_sg.name]

  network {
    uuid = openstack_networking_network_v2.k3s_network.id
  }

  metadata = merge(local.common_tags, {
    role          = "k3s-master"
    k3s_cluster   = var.instance_name
    ansible_group = "k3s_master"
  })

  lifecycle { create_before_destroy = true }
  depends_on = [openstack_networking_router_interface_v2.k3s_router_interface]
}

# K3s Agents
resource "openstack_compute_instance_v2" "k3s_agents" {
  provider        = openstack.ovh
  count           = var.k3s_agent_count
  name            = "${var.instance_name}-agent-${count.index + 1}"
  image_id        = data.openstack_images_image_v2.ubuntu_image.id
  flavor_name     = var.k3s_agent_flavor
  key_pair        = openstack_compute_keypair_v2.koprogo_key.name
  security_groups = [openstack_compute_secgroup_v2.k3s_cluster_sg.name]

  network {
    uuid = openstack_networking_network_v2.k3s_network.id
  }

  metadata = merge(local.common_tags, {
    role          = "k3s-agent"
    k3s_cluster   = var.instance_name
    ansible_group = "k3s_agents"
  })

  lifecycle { create_before_destroy = true }
  depends_on = [openstack_compute_instance_v2.k3s_master]
}

# Floating IP for Master
resource "openstack_compute_floatingip_v2" "k3s_master_fip" {
  provider = openstack.ovh
  pool     = data.openstack_networking_network_v2.ext_net.name
}

resource "openstack_compute_floatingip_associate_v2" "k3s_master_fip_assoc" {
  provider    = openstack.ovh
  floating_ip = openstack_compute_floatingip_v2.k3s_master_fip.address
  instance_id = openstack_compute_instance_v2.k3s_master.id
}
