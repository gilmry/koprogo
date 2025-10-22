# OVH Instances Module
# Creates compute instances for K3s cluster (control plane + workers)

locals {
  # Common tags for all resources
  common_tags = merge(
    var.tags,
    {
      "terraform"   = "true"
      "project"     = "koprogo"
      "environment" = var.environment
      "managed-by"  = "terraform"
    }
  )
}

# SSH Key for instances
resource "ovh_cloud_project_sshkey" "terraform_key" {
  count      = var.ssh_public_key != "" ? 1 : 0
  service_name = var.ovh_service_name
  name         = "koprogo-${var.environment}-key"
  public_key   = var.ssh_public_key
}

# Control Plane Instances
resource "ovh_cloud_project_compute_instance" "control_plane" {
  count = var.control_plane_count

  service_name = var.ovh_service_name
  name         = "${var.cluster_name}-control-plane-${count.index + 1}"

  # Flavor (instance type)
  flavor_name = var.control_plane_flavor

  # Image
  image_name = var.image_name

  # Region
  region = var.region

  # SSH keys
  ssh_key_ids = var.ssh_public_key != "" ? [ovh_cloud_project_sshkey.terraform_key[0].id] : []

  # Network
  network {
    name = var.private_network_name
  }

  # User data for cloud-init
  user_data = templatefile("${path.module}/templates/cloud-init.yaml", {
    hostname     = "${var.cluster_name}-control-plane-${count.index + 1}"
    environment  = var.environment
    node_type    = "control-plane"
    cluster_name = var.cluster_name
  })

  # Lifecycle
  lifecycle {
    create_before_destroy = true
    ignore_changes        = [user_data]
  }
}

# Worker Instances
resource "ovh_cloud_project_compute_instance" "worker" {
  count = var.worker_count

  service_name = var.ovh_service_name
  name         = "${var.cluster_name}-worker-${count.index + 1}"

  # Flavor (instance type)
  flavor_name = var.worker_flavor

  # Image
  image_name = var.image_name

  # Region
  region = var.region

  # SSH keys
  ssh_key_ids = var.ssh_public_key != "" ? [ovh_cloud_project_sshkey.terraform_key[0].id] : []

  # Network
  network {
    name = var.private_network_name
  }

  # User data for cloud-init
  user_data = templatefile("${path.module}/templates/cloud-init.yaml", {
    hostname     = "${var.cluster_name}-worker-${count.index + 1}"
    environment  = var.environment
    node_type    = "worker"
    cluster_name = var.cluster_name
  })

  # Anti-affinity group for workers (spread across different hosts)
  anti_affinity_group_ids = var.enable_anti_affinity ? [ovh_cloud_project_compute_antiaffinity_group.workers[0].id] : []

  # Lifecycle
  lifecycle {
    create_before_destroy = true
    ignore_changes        = [user_data]
  }
}

# Anti-affinity group for workers
resource "ovh_cloud_project_compute_antiaffinity_group" "workers" {
  count = var.enable_anti_affinity ? 1 : 0

  service_name = var.ovh_service_name
  name         = "${var.cluster_name}-workers-antiaffinity"
  region       = var.region
  type         = "anti_affinity"
}

# Additional volumes for Longhorn storage
resource "ovh_cloud_project_volume" "longhorn" {
  count = var.worker_count * var.longhorn_volumes_per_node

  service_name = var.ovh_service_name
  name         = "${var.cluster_name}-longhorn-vol-${floor(count.index / var.longhorn_volumes_per_node) + 1}-${count.index % var.longhorn_volumes_per_node + 1}"
  region       = var.region
  size         = var.longhorn_volume_size
  type         = "high-speed"
}

# Attach volumes to workers
resource "ovh_cloud_project_compute_instance_volume_attach" "longhorn" {
  count = var.worker_count * var.longhorn_volumes_per_node

  service_name = var.ovh_service_name
  instance_id  = ovh_cloud_project_compute_instance.worker[floor(count.index / var.longhorn_volumes_per_node)].id
  volume_id    = ovh_cloud_project_volume.longhorn[count.index].id
}
