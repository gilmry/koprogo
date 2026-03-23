output "master_ip_private" {
  description = "K3s master private IP"
  value       = openstack_compute_instance_v2.k3s_master.access_ip_v4
}

output "master_ip_public" {
  description = "K3s master public floating IP"
  value       = openstack_compute_floatingip_v2.k3s_master_fip.address
}

output "agents_ips" {
  description = "K3s agent private IPs"
  value = {
    for idx, agent in openstack_compute_instance_v2.k3s_agents :
    agent.name => agent.access_ip_v4
  }
}

output "cluster_info" {
  description = "Cluster info for Ansible"
  value = {
    cluster_name      = var.instance_name
    k3s_version       = local.k3s_version
    master_ip_private = openstack_compute_instance_v2.k3s_master.access_ip_v4
    master_ip_public  = openstack_compute_floatingip_v2.k3s_master_fip.address
    agents            = [for agent in openstack_compute_instance_v2.k3s_agents : agent.name]
    region            = var.region
  }
}

output "kubectl_command" {
  description = "Command to retrieve kubeconfig"
  value       = "ssh -i ${var.ssh_private_key_path} ubuntu@${openstack_compute_floatingip_v2.k3s_master_fip.address} sudo cat /etc/rancher/k3s/k3s.yaml"
}
