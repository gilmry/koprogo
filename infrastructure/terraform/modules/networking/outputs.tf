# Outputs for Networking Module

output "private_network_id" {
  description = "ID of the private network"
  value       = ovh_cloud_project_network_private.k3s_network.id
}

output "private_network_name" {
  description = "Name of the private network"
  value       = ovh_cloud_project_network_private.k3s_network.name
}

output "subnet_id" {
  description = "ID of the private subnet"
  value       = ovh_cloud_project_network_private_subnet.k3s_subnet.id
}

output "subnet_cidr" {
  description = "CIDR block of the subnet"
  value       = ovh_cloud_project_network_private_subnet.k3s_subnet.network
}

output "control_plane_security_group_id" {
  description = "ID of control plane security group"
  value       = ovh_cloud_project_network_private_security_group.control_plane.id
}

output "workers_security_group_id" {
  description = "ID of workers security group"
  value       = ovh_cloud_project_network_private_security_group.workers.id
}

output "load_balancer_id" {
  description = "ID of the load balancer (if enabled)"
  value       = var.enable_load_balancer ? ovh_cloud_project_network_loadbalancer.ingress[0].id : null
}

output "load_balancer_ip" {
  description = "IP address of the load balancer (if enabled)"
  value       = var.enable_load_balancer ? ovh_cloud_project_network_loadbalancer.ingress[0].ip_address : null
}
