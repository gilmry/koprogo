variable "ovh_endpoint" {
  description = "OVH API endpoint (ovh-eu, ovh-ca, etc.)"
  type        = string
  default     = "ovh-eu"
}

variable "ovh_service_name" {
  description = "ID du projet OVH Cloud (obtenu via console OVH)"
  type        = string
}

variable "instance_name" {
  description = "Nom de l'instance VPS"
  type        = string
  default     = "koprogo-vps"
}

variable "region" {
  description = "Région OVH (GRA = Gravelines France, bas carbone)"
  type        = string
  default     = "GRA11" # Datacenter Gravelines (60g CO2/kWh)
}

variable "ssh_public_key_path" {
  description = "Chemin vers votre clé SSH publique"
  type        = string
  default     = "~/.ssh/id_rsa.pub"
}

variable "domain" {
  description = "Nom de domaine (optionnel, pour SSL automatique)"
  type        = string
  default     = ""
}
