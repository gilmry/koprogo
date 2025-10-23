# Networking Module
# Creates private network, firewall rules, and load balancer

# Private Network (vRack)
resource "ovh_cloud_project_network_private" "k3s_network" {
  service_name = var.ovh_service_name
  name         = "${var.cluster_name}-${var.environment}-network"
  regions      = [var.region]
  vlan_id      = var.vlan_id
}

# Private Subnet
resource "ovh_cloud_project_network_private_subnet" "k3s_subnet" {
  service_name = var.ovh_service_name
  network_id   = ovh_cloud_project_network_private.k3s_network.id
  region       = var.region

  # Subnet configuration
  start = var.subnet_start
  end   = var.subnet_end
  network = var.subnet_cidr
  dhcp    = true
  no_gateway = false
}

# Security Group for Control Plane
resource "ovh_cloud_project_network_private_security_group" "control_plane" {
  service_name = var.ovh_service_name
  name         = "${var.cluster_name}-${var.environment}-control-plane-sg"
}

# K3s API Server (6443)
resource "ovh_cloud_project_network_private_security_group_rule" "k3s_api" {
  service_name       = var.ovh_service_name
  security_group_id  = ovh_cloud_project_network_private_security_group.control_plane.id
  direction          = "ingress"
  protocol           = "tcp"
  port_range_min     = 6443
  port_range_max     = 6443
  remote_ip_prefix   = "0.0.0.0/0"
}

# Kubelet API (10250)
resource "ovh_cloud_project_network_private_security_group_rule" "kubelet_api" {
  service_name       = var.ovh_service_name
  security_group_id  = ovh_cloud_project_network_private_security_group.control_plane.id
  direction          = "ingress"
  protocol           = "tcp"
  port_range_min     = 10250
  port_range_max     = 10250
  remote_ip_prefix   = var.subnet_cidr
}

# etcd server (2379-2380)
resource "ovh_cloud_project_network_private_security_group_rule" "etcd" {
  service_name       = var.ovh_service_name
  security_group_id  = ovh_cloud_project_network_private_security_group.control_plane.id
  direction          = "ingress"
  protocol           = "tcp"
  port_range_min     = 2379
  port_range_max     = 2380
  remote_ip_prefix   = var.subnet_cidr
}

# Security Group for Workers
resource "ovh_cloud_project_network_private_security_group" "workers" {
  service_name = var.ovh_service_name
  name         = "${var.cluster_name}-${var.environment}-workers-sg"
}

# NodePort Services (30000-32767)
resource "ovh_cloud_project_network_private_security_group_rule" "nodeport" {
  service_name       = var.ovh_service_name
  security_group_id  = ovh_cloud_project_network_private_security_group.workers.id
  direction          = "ingress"
  protocol           = "tcp"
  port_range_min     = 30000
  port_range_max     = 32767
  remote_ip_prefix   = "0.0.0.0/0"
}

# HTTP (80)
resource "ovh_cloud_project_network_private_security_group_rule" "http" {
  service_name       = var.ovh_service_name
  security_group_id  = ovh_cloud_project_network_private_security_group.workers.id
  direction          = "ingress"
  protocol           = "tcp"
  port_range_min     = 80
  port_range_max     = 80
  remote_ip_prefix   = "0.0.0.0/0"
}

# HTTPS (443)
resource "ovh_cloud_project_network_private_security_group_rule" "https" {
  service_name       = var.ovh_service_name
  security_group_id  = ovh_cloud_project_network_private_security_group.workers.id
  direction          = "ingress"
  protocol           = "tcp"
  port_range_min     = 443
  port_range_max     = 443
  remote_ip_prefix   = "0.0.0.0/0"
}

# SSH (22) - restricted to admin IPs
resource "ovh_cloud_project_network_private_security_group_rule" "ssh" {
  for_each = toset(var.admin_ips)

  service_name       = var.ovh_service_name
  security_group_id  = ovh_cloud_project_network_private_security_group.control_plane.id
  direction          = "ingress"
  protocol           = "tcp"
  port_range_min     = 22
  port_range_max     = 22
  remote_ip_prefix   = each.value
}

# Load Balancer for Ingress
resource "ovh_cloud_project_network_loadbalancer" "ingress" {
  count = var.enable_load_balancer ? 1 : 0

  service_name = var.ovh_service_name
  name         = "${var.cluster_name}-${var.environment}-lb"
  region       = var.region
  network_id   = ovh_cloud_project_network_private.k3s_network.id
  subnet_id    = ovh_cloud_project_network_private_subnet.k3s_subnet.id
}

# Load Balancer Listener HTTP
resource "ovh_cloud_project_network_loadbalancer_listener" "http" {
  count = var.enable_load_balancer ? 1 : 0

  service_name   = var.ovh_service_name
  loadbalancer_id = ovh_cloud_project_network_loadbalancer.ingress[0].id
  name           = "http-listener"
  protocol       = "tcp"
  port           = 80
}

# Load Balancer Listener HTTPS
resource "ovh_cloud_project_network_loadbalancer_listener" "https" {
  count = var.enable_load_balancer ? 1 : 0

  service_name   = var.ovh_service_name
  loadbalancer_id = ovh_cloud_project_network_loadbalancer.ingress[0].id
  name           = "https-listener"
  protocol       = "tcp"
  port           = 443
}

# Load Balancer Pool for Ingress
resource "ovh_cloud_project_network_loadbalancer_pool" "ingress_pool" {
  count = var.enable_load_balancer ? 1 : 0

  service_name   = var.ovh_service_name
  loadbalancer_id = ovh_cloud_project_network_loadbalancer.ingress[0].id
  name           = "ingress-pool"
  protocol       = "tcp"
  algorithm      = "round_robin"
}

# Health Monitor for Ingress Pool
resource "ovh_cloud_project_network_loadbalancer_pool_health_monitor" "ingress" {
  count = var.enable_load_balancer ? 1 : 0

  service_name   = var.ovh_service_name
  loadbalancer_id = ovh_cloud_project_network_loadbalancer.ingress[0].id
  pool_id        = ovh_cloud_project_network_loadbalancer_pool.ingress_pool[0].id

  type           = "http"
  delay          = 10
  max_retries    = 3
  timeout        = 5
  url_path       = "/healthz"
  http_method    = "GET"
  expected_codes = ["200", "204"]
}
