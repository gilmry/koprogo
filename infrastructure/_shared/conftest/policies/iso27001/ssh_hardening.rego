# ISO 27001 A.8.9 — Configuration management (SSH hardening).
package iso27001.ssh_hardening

import rego.v1

has_task(tasks, pred) if {
	some task in tasks
	pred(task)
}

deploys_validated_config if {
	has_task(input.security_tasks, is_validated_deploy_task)
}

is_validated_deploy_task(task) if {
	task.template.src == "sshd_config_hardening.j2"
	task.template.dest == "/etc/ssh/sshd_config"
	task.template.validate == "/usr/sbin/sshd -t -f %s"
}

deny[msg] if {
	not deploys_validated_config
	msg := "A.8.9 SSH: hardened sshd_config is not deployed with `validate: sshd -t` (a syntax error could lock out SSH)"
}

deny[msg] if {
	not regex.match(`(?m)^PermitRootLogin no$`, input.sshd_template)
	msg := "A.8.9 SSH: PermitRootLogin is not set to 'no'"
}

deny[msg] if {
	not regex.match(`(?m)^PasswordAuthentication no$`, input.sshd_template)
	msg := "A.8.9 SSH: PasswordAuthentication is not set to 'no' (key-only auth required)"
}

deny[msg] if {
	not regex.match(`(?m)^PermitEmptyPasswords no$`, input.sshd_template)
	msg := "A.8.9 SSH: PermitEmptyPasswords is not set to 'no'"
}

deny[msg] if {
	not regex.match(`(?m)^MaxAuthTries [1-3]$`, input.sshd_template)
	msg := "A.8.9 SSH: MaxAuthTries is not bounded to 1-3 (brute-force exposure)"
}
