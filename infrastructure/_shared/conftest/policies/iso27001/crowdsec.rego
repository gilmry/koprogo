# ISO 27001 A.8.7 — Protection against malware (CrowdSec WAF / bouncer).
package iso27001.crowdsec

import rego.v1

has_task(tasks, pred) if {
	some task in tasks
	pred(task)
}

installs_crowdsec if {
	has_task(array.concat(input.security_tasks, input.hardening_tasks), is_install_task)
}

is_install_task(task) if {
	names := task.apt.name
	is_array(names)
	"crowdsec" in names
	"crowdsec-firewall-bouncer-iptables" in names
}

enables_crowdsec_service if {
	has_task(array.concat(input.security_tasks, input.hardening_tasks), is_enable_task)
}

is_enable_task(task) if {
	task.systemd.name == "crowdsec"
	task.systemd.enabled == true
	task.systemd.state == "started"
}

deny[msg] if {
	not installs_crowdsec
	msg := "A.8.7 CrowdSec: no task installs crowdsec + crowdsec-firewall-bouncer-iptables"
}

deny[msg] if {
	not enables_crowdsec_service
	msg := "A.8.7 CrowdSec: no task enables+starts the crowdsec systemd service"
}
