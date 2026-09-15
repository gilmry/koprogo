package iso27001.ssh_hardening

import rego.v1

good_sshd_template := "Port 22\nPermitRootLogin no\nPubkeyAuthentication yes\nPasswordAuthentication no\nPermitEmptyPasswords no\nMaxAuthTries 3\n"

compliant_input := {
	"security_tasks": [
		{"name": "Deploy hardened SSH configuration", "template": {
			"src": "sshd_config_hardening.j2",
			"dest": "/etc/ssh/sshd_config",
			"validate": "/usr/sbin/sshd -t -f %s",
		}},
	],
	"sshd_template": good_sshd_template,
}

# @happy
test_allows_compliant_config if {
	deny == set() with input as compliant_input
}

# @negative — config deployed without the `sshd -t` validate guard: a typo
# could lock every operator out of the box with no rollback.
test_denies_missing_validate_guard if {
	tampered := object.union(compliant_input, {"security_tasks": [{"name": "x", "template": {
		"src": "sshd_config_hardening.j2",
		"dest": "/etc/ssh/sshd_config",
	}}]})
	some msg in deny with input as tampered
	contains(msg, "validate")
}

# @security — root login re-enabled (e.g. a well-meaning "temporary" debug
# change) must be caught, since this directly grants root over SSH.
test_denies_root_login_enabled if {
	tampered := object.union(compliant_input, {"sshd_template": replace(good_sshd_template, "PermitRootLogin no", "PermitRootLogin yes")})
	some msg in deny with input as tampered
	contains(msg, "PermitRootLogin")
}

# @security — password auth re-enabled defeats key-only access control.
test_denies_password_auth_enabled if {
	tampered := object.union(compliant_input, {"sshd_template": replace(good_sshd_template, "PasswordAuthentication no", "PasswordAuthentication yes")})
	some msg in deny with input as tampered
	contains(msg, "PasswordAuthentication")
}

# @edge — MaxAuthTries loosened past the 1-3 bound (e.g. set to 20).
test_denies_loosened_max_auth_tries if {
	tampered := object.union(compliant_input, {"sshd_template": replace(good_sshd_template, "MaxAuthTries 3", "MaxAuthTries 20")})
	some msg in deny with input as tampered
	contains(msg, "MaxAuthTries")
}
