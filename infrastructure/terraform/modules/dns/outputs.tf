# Outputs for DNS Module

output "api_fqdn" {
  description = "FQDN for API endpoint"
  value       = var.create_api_record ? "api.${var.domain_name}" : null
}

output "app_fqdn" {
  description = "FQDN for frontend app"
  value       = var.create_app_record ? "app.${var.domain_name}" : null
}

output "grafana_fqdn" {
  description = "FQDN for Grafana"
  value       = var.create_monitoring_records ? "grafana.${var.domain_name}" : null
}

output "prometheus_fqdn" {
  description = "FQDN for Prometheus"
  value       = var.create_monitoring_records ? "prometheus.${var.domain_name}" : null
}

output "dns_zone" {
  description = "DNS zone name"
  value       = var.domain_name
}
