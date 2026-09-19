package iso27001.fail2ban

import rego.v1

compliant_input := {
	"security_tasks": [
		{"name": "Ensure fail2ban is running", "systemd": {"name": "fail2ban", "state": "started", "enabled": true}},
	],
	"hardening_tasks": [
		{"name": "Install fail2ban", "apt": {"name": "fail2ban", "state": "present"}},
	],
	"fail2ban_jail_template": "[sshd]\nenabled = true\nmaxretry = 3\nbantime = 3600\nfindtime = 600\n",
}

# @happy
test_allows_compliant_config if {
	deny == set() with input as compliant_input
}

# @negative — package never installed anywhere in the two roles.
test_denies_missing_install if {
	tampered := object.union(compliant_input, {"hardening_tasks": []})
	some msg in deny with input as tampered
	contains(msg, "installs the fail2ban package")
}

# @negative — service deployed but never enabled/started.
test_denies_missing_service_enable if {
	tampered := object.union(compliant_input, {"security_tasks": []})
	some msg in deny with input as tampered
	contains(msg, "enables+starts")
}

# @security — a weakened bantime (e.g. dropped to 60s) must fail, since a
# policy that always passes proves nothing (per story's @security class).
test_denies_weakened_bantime if {
	tampered := object.union(compliant_input, {"fail2ban_jail_template": "[sshd]\nmaxretry = 3\nbantime = 60\n"})
	some msg in deny with input as tampered
	contains(msg, "bantime")
}

# @edge — maxretry loosened (e.g. raised to 30 attempts before a ban).
test_denies_loosened_maxretry if {
	tampered := object.union(compliant_input, {"fail2ban_jail_template": "[sshd]\nmaxretry = 30\nbantime = 3600\n"})
	some msg in deny with input as tampered
	contains(msg, "maxretry")
}
