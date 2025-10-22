# DNS Module
# Creates DNS records for the cluster

# Data source for the domain zone
data "ovh_domain_zone" "main" {
  name = var.domain_name
}

# A Record for API (K3s API Server)
resource "ovh_domain_zone_record" "api" {
  count = var.create_api_record ? 1 : 0

  zone      = data.ovh_domain_zone.main.name
  subdomain = "api"
  fieldtype = "A"
  ttl       = var.dns_ttl
  target    = var.api_ip_address
}

# A Record for Frontend Application
resource "ovh_domain_zone_record" "app" {
  count = var.create_app_record ? 1 : 0

  zone      = data.ovh_domain_zone.main.name
  subdomain = "app"
  fieldtype = "A"
  ttl       = var.dns_ttl
  target    = var.app_ip_address
}

# A Record for Grafana Monitoring
resource "ovh_domain_zone_record" "grafana" {
  count = var.create_monitoring_records ? 1 : 0

  zone      = data.ovh_domain_zone.main.name
  subdomain = "grafana"
  fieldtype = "A"
  ttl       = var.dns_ttl
  target    = var.monitoring_ip_address
}

# A Record for Prometheus
resource "ovh_domain_zone_record" "prometheus" {
  count = var.create_monitoring_records ? 1 : 0

  zone      = data.ovh_domain_zone.main.name
  subdomain = "prometheus"
  fieldtype = "A"
  ttl       = var.dns_ttl
  target    = var.monitoring_ip_address
}

# Wildcard A Record (*.koprogo.io)
resource "ovh_domain_zone_record" "wildcard" {
  count = var.create_wildcard_record ? 1 : 0

  zone      = data.ovh_domain_zone.main.name
  subdomain = "*"
  fieldtype = "A"
  ttl       = var.dns_ttl
  target    = var.wildcard_ip_address
}

# TXT Record for domain verification
resource "ovh_domain_zone_record" "verification" {
  count = var.verification_txt != "" ? 1 : 0

  zone      = data.ovh_domain_zone.main.name
  subdomain = ""
  fieldtype = "TXT"
  ttl       = var.dns_ttl
  target    = var.verification_txt
}

# CAA Records for SSL certificate authorities
resource "ovh_domain_zone_record" "caa_letsencrypt" {
  count = var.create_caa_records ? 1 : 0

  zone      = data.ovh_domain_zone.main.name
  subdomain = ""
  fieldtype = "CAA"
  ttl       = var.dns_ttl
  target    = "0 issue \"letsencrypt.org\""
}

resource "ovh_domain_zone_record" "caa_wildcard" {
  count = var.create_caa_records ? 1 : 0

  zone      = data.ovh_domain_zone.main.name
  subdomain = ""
  fieldtype = "CAA"
  ttl       = var.dns_ttl
  target    = "0 issuewild \"letsencrypt.org\""
}
