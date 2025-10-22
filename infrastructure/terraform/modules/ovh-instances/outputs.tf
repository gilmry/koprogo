# Outputs for OVH Instances Module

# Control Plane Outputs
output "control_plane_ids" {
  description = "IDs of control plane instances"
  value       = ovh_cloud_project_compute_instance.control_plane[*].id
}

output "control_plane_names" {
  description = "Names of control plane instances"
  value       = ovh_cloud_project_compute_instance.control_plane[*].name
}

output "control_plane_ips" {
  description = "Public IPs of control plane instances"
  value       = ovh_cloud_project_compute_instance.control_plane[*].ip_address
}

output "control_plane_private_ips" {
  description = "Private IPs of control plane instances"
  value       = [for instance in ovh_cloud_project_compute_instance.control_plane : instance.network[0].ip_address]
}

# Worker Outputs
output "worker_ids" {
  description = "IDs of worker instances"
  value       = ovh_cloud_project_compute_instance.worker[*].id
}

output "worker_names" {
  description = "Names of worker instances"
  value       = ovh_cloud_project_compute_instance.worker[*].name
}

output "worker_ips" {
  description = "Public IPs of worker instances"
  value       = ovh_cloud_project_compute_instance.worker[*].ip_address
}

output "worker_private_ips" {
  description = "Private IPs of worker instances"
  value       = [for instance in ovh_cloud_project_compute_instance.worker : instance.network[0].ip_address]
}

# Longhorn Volumes
output "longhorn_volume_ids" {
  description = "IDs of Longhorn volumes"
  value       = ovh_cloud_project_volume.longhorn[*].id
}

# Ansible Inventory Output
output "ansible_inventory" {
  description = "Ansible inventory in YAML format"
  value = yamlencode({
    all = {
      vars = {
        ansible_user                 = "ubuntu"
        ansible_ssh_private_key_file = "~/.ssh/koprogo-${var.environment}"
        ansible_python_interpreter   = "/usr/bin/python3"
      }
      children = {
        k3s_control_plane = {
          hosts = {
            for idx, instance in ovh_cloud_project_compute_instance.control_plane :
            instance.name => {
              ansible_host = instance.ip_address
              private_ip   = instance.network[0].ip_address
              node_type    = "control-plane"
            }
          }
        }
        k3s_workers = {
          hosts = {
            for idx, instance in ovh_cloud_project_compute_instance.worker :
            instance.name => {
              ansible_host = instance.ip_address
              private_ip   = instance.network[0].ip_address
              node_type    = "worker"
            }
          }
        }
      }
    }
  })
}
