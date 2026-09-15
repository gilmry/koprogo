# ISO 27001 A.8.15 — Logging (centralized log shipping/aggregation).
package iso27001.centralized_logging

import rego.v1

deny[msg] if {
	not "loki" in input.monitoring_compose_services
	msg := "A.8.15 logging: monitoring stack has no log aggregation service (loki)"
}

deny[msg] if {
	not "promtail" in input.monitoring_compose_services
	msg := "A.8.15 logging: monitoring stack has no log shipping service (promtail)"
}
