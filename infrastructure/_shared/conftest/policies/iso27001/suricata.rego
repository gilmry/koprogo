# ISO 27001 A.8.16 — Monitoring activities (Suricata IDS).
package iso27001.suricata

import rego.v1

has_task(tasks, pred) if {
	some task in tasks
	pred(task)
}

installs_suricata if {
	has_task(array.concat(input.security_tasks, input.hardening_tasks), is_install_task)
}

is_install_task(task) if {
	task.apt.name == "suricata"
}

enables_suricata_service if {
	has_task(array.concat(input.security_tasks, input.hardening_tasks), is_enable_task)
}

is_enable_task(task) if {
	task.systemd.name == "suricata"
	task.systemd.enabled == true
	task.systemd.state == "started"
}

deny[msg] if {
	not installs_suricata
	msg := "A.8.16 Suricata: no task installs the suricata package"
}

deny[msg] if {
	not enables_suricata_service
	msg := "A.8.16 Suricata: no task enables+starts the suricata systemd service"
}

deny[msg] if {
	not contains(lower(input.suricata_rules_template), "sql injection")
	msg := "A.8.16 Suricata: local ruleset has no SQL injection detection rule"
}

deny[msg] if {
	not contains(lower(input.suricata_rules_template), "xss")
	msg := "A.8.16 Suricata: local ruleset has no XSS detection rule"
}
