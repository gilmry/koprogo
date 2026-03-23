output "vps_ip" {
  description = "Public IP of the VPS"
  value       = openstack_compute_instance_v2.koprogo_vps.access_ip_v4
}

output "vps_id" {
  description = "Instance ID"
  value       = openstack_compute_instance_v2.koprogo_vps.id
}

output "ssh_command" {
  description = "SSH command to connect"
  value       = "ssh ubuntu@${openstack_compute_instance_v2.koprogo_vps.access_ip_v4}"
}
