# ISO 27001 A.8.7 — Protection against malware / intrusion (fail2ban).
#
# `security` and `hardening` both touch fail2ban but only `hardening`
# installs the package (`apt: name: fail2ban`) — `security` assumes it's
# already there because in the only playbook that runs both
# (monosite/vps/production/ansible/playbook.yml) hardening always runs
# first. That's an execution-order dependency, not a self-contained
# capability — checking `security_tasks OR hardening_tasks` reflects that
# reality instead of hiding it (see docs/agent-activity/2026-09-14-wp-e2-iac-tests.md).
package iso27001.fail2ban

import rego.v1

has_task(tasks, pred) if {
	some task in tasks
	pred(task)
}

installs_fail2ban if {
	has_task(array.concat(input.security_tasks, input.hardening_tasks), is_install_task)
}

is_install_task(task) if {
	names := task.apt.name
	names == "fail2ban"
}

is_install_task(task) if {
	names := task.apt.name
	is_array(names)
	"fail2ban" in names
}

enables_fail2ban_service if {
	has_task(array.concat(input.security_tasks, input.hardening_tasks), is_enable_task)
}

is_enable_task(task) if {
	task.systemd.name == "fail2ban"
	task.systemd.enabled == true
	task.systemd.state == "started"
}

deny[msg] if {
	not installs_fail2ban
	msg := "A.8.7 fail2ban: no task installs the fail2ban package (security or hardening role)"
}

deny[msg] if {
	not enables_fail2ban_service
	msg := "A.8.7 fail2ban: no task enables+starts the fail2ban systemd service"
}

deny[msg] if {
	not regex.match(`(?s)\[sshd\].*bantime = 3600`, input.fail2ban_jail_template)
	msg := "A.8.7 fail2ban: [sshd] jail bantime is not 3600s (weakened ban duration)"
}

deny[msg] if {
	not regex.match(`(?s)\[sshd\].*maxretry = 3`, input.fail2ban_jail_template)
	msg := "A.8.7 fail2ban: [sshd] jail maxretry is not 3 (weakened threshold)"
}
