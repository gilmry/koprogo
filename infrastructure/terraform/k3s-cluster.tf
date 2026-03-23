# Terraform: OVH OpenStack Provisioning for K3s Cluster
# Issue #266: K3s Provisioning with Terraform + OpenStack
# Belgian GDPR compliance: EU datacenter (OVH Gravelines/Roubaix region)
# Multi-node K3s cluster with master + agents

terraform {
  required_providers {
    openstack = {
      source  = "terraform-provider-openstack/openstack"
      version = "~> 2.1"
    }
  }
}

# Local variables for K3s cluster configuration
locals {
  k3s_environment = var.environment
  k3s_region      = var.region
  k3s_project     = var.instance_name
  k3s_version     = "v1.29.2+k3s1"

  common_tags = {
    Environment = local.k3s_environment
    Project     = local.k3s_project
    Region      = local.k3s_region
    ManagedBy   = "Terraform"
    GDPR        = "EU-West"
    IssueID     = "266"
  }
}

# K3s Network
resource "openstack_networking_network_v2" "k3s_network" {
  provider       = openstack.ovh
  name           = "${local.k3s_project}-network"
  admin_state_up = true
}

# K3s Subnet
resource "openstack_networking_subnet_v2" "k3s_subnet" {
  provider            = openstack.ovh
  name                = "${local.k3s_project}-subnet"
  network_id          = openstack_networking_network_v2.k3s_network.id
  cidr                = "10.0.0.0/24"
  ip_version          = 4
  dns_nameservers     = ["1.1.1.1", "8.8.8.8", "9.9.9.9"]
  enable_dhcp         = true
  gateway_ip          = "10.0.0.1"

  allocation_pools {
    start = "10.0.0.10"
    end   = "10.0.0.254"
  }
}

# Router for external network access
resource "openstack_networking_router_v2" "k3s_router" {
  provider                = openstack.ovh
  name                    = "${local.k3s_project}-router"
  admin_state_up          = true
  external_network_id     = data.openstack_networking_network_v2.ext_net.id
}

# Router interface for K3s subnet
resource "openstack_networking_router_interface_v2" "k3s_router_interface" {
  provider    = openstack.ovh
  router_id   = openstack_networking_router_v2.k3s_router.id
  subnet_id   = openstack_networking_subnet_v2.k3s_subnet.id
}

# Security Group: K3s Cluster
resource "openstack_compute_secgroup_v2" "k3s_cluster_sg" {
  provider    = openstack.ovh
  name        = "${local.k3s_project}-k3s-sg"
  description = "K3s Kubernetes cluster security group (Issue #266)"

  # SSH access from anywhere
  rule {
    from_port   = 22
    to_port     = 22
    ip_protocol = "tcp"
    cidr        = "0.0.0.0/0"
  }

  # K3s API server (port 6443) - internal only
  rule {
    from_port   = 6443
    to_port     = 6443
    ip_protocol = "tcp"
    cidr        = "10.0.0.0/24"
  }

  # Flannel VXLAN overlay network
  rule {
    from_port   = 8472
    to_port     = 8472
    ip_protocol = "udp"
    cidr        = "10.0.0.0/24"
  }

  # HTTP ingress
  rule {
    from_port   = 80
    to_port     = 80
    ip_protocol = "tcp"
    cidr        = "0.0.0.0/0"
  }

  # HTTPS ingress
  rule {
    from_port   = 443
    to_port     = 443
    ip_protocol = "tcp"
    cidr        = "0.0.0.0/0"
  }

  # Kubelet API (10250) - internal
  rule {
    from_port   = 10250
    to_port     = 10250
    ip_protocol = "tcp"
    cidr        = "10.0.0.0/24"
  }

  # Prometheus metrics (9100) - internal
  rule {
    from_port   = 9100
    to_port     = 9100
    ip_protocol = "tcp"
    cidr        = "10.0.0.0/24"
  }

  # Allow all internal traffic within cluster
  rule {
    from_port   = -1
    to_port     = -1
    ip_protocol = "tcp"
    cidr        = "10.0.0.0/24"
  }

  rule {
    from_port   = -1
    to_port     = -1
    ip_protocol = "udp"
    cidr        = "10.0.0.0/24"
  }
}

# Data source: External network (OVH Ext-Net)
data "openstack_networking_network_v2" "ext_net" {
  provider = openstack.ovh
  name     = "Ext-Net"
}

# Data source: Ubuntu 22.04 LTS image
data "openstack_images_image_v2" "ubuntu_image" {
  provider    = openstack.ovh
  name        = "Ubuntu 22.04"
  most_recent = true
  visibility  = "public"
}

# K3s Master Node Instance
resource "openstack_compute_instance_v2" "k3s_master" {
  provider            = openstack.ovh
  name                = "${local.k3s_project}-master"
  image_id            = data.openstack_images_image_v2.ubuntu_image.id
  flavor_name         = var.k3s_master_flavor # e.g., "b2-7"
  key_pair            = openstack_compute_keypair_v2.koprogo_key.name
  security_groups     = [openstack_compute_secgroup_v2.k3s_cluster_sg.name]

  network {
    uuid = openstack_networking_network_v2.k3s_network.id
  }

  metadata = merge(
    local.common_tags,
    {
      role            = "k3s-master"
      k3s_cluster     = local.k3s_project
      hostname        = "${local.k3s_project}-master"
      ansible_group   = "k3s_master"
    }
  )

  lifecycle {
    create_before_destroy = true
  }

  depends_on = [openstack_networking_router_interface_v2.k3s_router_interface]
}

# K3s Agent Nodes (Workers)
resource "openstack_compute_instance_v2" "k3s_agents" {
  provider            = openstack.ovh
  count               = var.k3s_agent_count
  name                = "${local.k3s_project}-agent-${count.index + 1}"
  image_id            = data.openstack_images_image_v2.ubuntu_image.id
  flavor_name         = var.k3s_agent_flavor # e.g., "b2-7"
  key_pair            = openstack_compute_keypair_v2.koprogo_key.name
  security_groups     = [openstack_compute_secgroup_v2.k3s_cluster_sg.name]

  network {
    uuid = openstack_networking_network_v2.k3s_network.id
  }

  metadata = merge(
    local.common_tags,
    {
      role            = "k3s-agent"
      k3s_cluster     = local.k3s_project
      hostname        = "${local.k3s_project}-agent-${count.index + 1}"
      ansible_group   = "k3s_agents"
    }
  )

  lifecycle {
    create_before_destroy = true
  }

  depends_on = [openstack_compute_instance_v2.k3s_master]
}

# Floating IP for K3s Master (public access)
resource "openstack_compute_floatingip_v2" "k3s_master_fip" {
  provider = openstack.ovh
  pool     = data.openstack_networking_network_v2.ext_net.name
}

# Associate Floating IP to Master
resource "openstack_compute_floatingip_associate_v2" "k3s_master_fip_assoc" {
  provider    = openstack.ovh
  floating_ip = openstack_compute_floatingip_v2.k3s_master_fip.address
  instance_id = openstack_compute_instance_v2.k3s_master.id
}

# Outputs for K3s Cluster
output "k3s_master_ip_private" {
  description = "K3s master node private IP"
  value       = openstack_compute_instance_v2.k3s_master.access_ip_v4
}

output "k3s_master_ip_public" {
  description = "K3s master node public floating IP"
  value       = openstack_compute_floatingip_v2.k3s_master_fip.address
}

output "k3s_agents_ips" {
  description = "K3s agent nodes private IPs"
  value = {
    for idx, agent in openstack_compute_instance_v2.k3s_agents :
    agent.name => agent.access_ip_v4
  }
}

output "k3s_cluster_info" {
  description = "K3s cluster information for Ansible"
  value = {
    cluster_name     = local.k3s_project
    k3s_version      = local.k3s_version
    master_hostname  = openstack_compute_instance_v2.k3s_master.name
    master_ip_private = openstack_compute_instance_v2.k3s_master.access_ip_v4
    master_ip_public = openstack_compute_floatingip_v2.k3s_master_fip.address
    agents           = [for agent in openstack_compute_instance_v2.k3s_agents : agent.name]
    network_cidr     = openstack_networking_subnet_v2.k3s_subnet.cidr
    region           = var.region
  }
}

output "ansible_inventory_ini" {
  description = "Ansible inventory in INI format"
  value = <<-EOT
[k3s_master]
${openstack_compute_instance_v2.k3s_master.name} ansible_host=${openstack_compute_instance_v2.k3s_master.access_ip_v4} ansible_user=ubuntu

[k3s_agents]
%{for agent in openstack_compute_instance_v2.k3s_agents~}
${agent.name} ansible_host=${agent.access_ip_v4} ansible_user=ubuntu
%{endfor~}

[k3s_cluster:children]
k3s_master
k3s_agents

[k3s_cluster:vars]
ansible_ssh_private_key_file=${var.ssh_private_key_path}
k3s_version=${local.k3s_version}
k3s_cluster_name=${local.k3s_project}
EOT
}

output "kubectl_command" {
  description = "Command to retrieve kubeconfig from master"
  value       = "ssh -i ${var.ssh_private_key_path} ubuntu@${openstack_compute_floatingip_v2.k3s_master_fip.address} sudo cat /etc/rancher/k3s/k3s.yaml"
}
