# ISO 27001 A.8.9 — Configuration management (kernel/network hardening).
package iso27001.kernel_hardening

import rego.v1

has_task(tasks, pred) if {
	some task in tasks
	pred(task)
}

applies_sysctl if {
	has_task(input.security_tasks, is_apply_task)
}

is_apply_task(task) if {
	startswith(task.command, "sysctl -p /etc/sysctl.d/99-koprogo-hardening.conf")
}

required_keys := {
	"net.ipv4.tcp_syncookies = 1",
	"net.ipv4.conf.all.rp_filter = 1",
	"net.ipv4.conf.all.accept_source_route = 0",
	"net.ipv4.conf.all.accept_redirects = 0",
	"kernel.dmesg_restrict = 1",
	"kernel.kptr_restrict = 2",
}

deny[msg] if {
	not applies_sysctl
	msg := "A.8.9 kernel: no task actually applies the sysctl file (`sysctl -p`) — deploying the file alone leaves the running kernel unhardened until reboot"
}

deny[msg] if {
	some key in required_keys
	not contains(input.sysctl_template, key)
	msg := sprintf("A.8.9 kernel: sysctl template is missing or weakens '%s'", [key])
}
