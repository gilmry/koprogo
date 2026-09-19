package iso27001.centralized_logging

import rego.v1

compliant_input := {"monitoring_compose_services": [
	"alertmanager", "cadvisor", "grafana", "loki", "node-exporter",
	"postgres-exporter", "prometheus", "promtail",
]}

# @happy
test_allows_compliant_config if {
	deny == set() with input as compliant_input
}

# @negative — loki dropped from the compose file (e.g. during a stack
# trim) silently breaks log aggregation for every other service.
test_denies_missing_loki if {
	tampered := {"monitoring_compose_services": ["grafana", "promtail", "prometheus"]}
	some msg in deny with input as tampered
	contains(msg, "loki")
}

# @negative — no shipper means logs never leave the host.
test_denies_missing_promtail if {
	tampered := {"monitoring_compose_services": ["grafana", "loki", "prometheus"]}
	some msg in deny with input as tampered
	contains(msg, "promtail")
}

# @edge — empty monitoring stack (compose file missing/unreadable).
test_denies_empty_stack if {
	tampered := {"monitoring_compose_services": []}
	count(deny) == 2 with input as tampered
}
