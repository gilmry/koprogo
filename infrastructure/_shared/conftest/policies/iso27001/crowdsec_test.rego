package iso27001.crowdsec

import rego.v1

compliant_input := {
	"security_tasks": [
		{"name": "Install CrowdSec", "apt": {"name": ["crowdsec", "crowdsec-firewall-bouncer-iptables"], "state": "present"}},
		{"name": "Enable CrowdSec service", "systemd": {"name": "crowdsec", "state": "started", "enabled": true}},
	],
	"hardening_tasks": [],
}

# @happy
test_allows_compliant_config if {
	deny == set() with input as compliant_input
}

# @negative — WAF package never installed.
test_denies_missing_install if {
	tampered := object.union(compliant_input, {"security_tasks": [compliant_input.security_tasks[1]]})
	some msg in deny with input as tampered
	contains(msg, "installs crowdsec")
}

# @negative — installed but the firewall bouncer (the part that actually
# blocks traffic) is missing from the package list.
test_denies_missing_bouncer_package if {
	tampered := object.union(compliant_input, {"security_tasks": [
		{"name": "Install CrowdSec", "apt": {"name": ["crowdsec"], "state": "present"}},
		compliant_input.security_tasks[1],
	]})
	some msg in deny with input as tampered
	contains(msg, "installs crowdsec")
}

# @edge — installed but the service is never enabled/started (dormant WAF).
test_denies_dormant_service if {
	tampered := object.union(compliant_input, {"security_tasks": [compliant_input.security_tasks[0]]})
	some msg in deny with input as tampered
	contains(msg, "enables+starts")
}
