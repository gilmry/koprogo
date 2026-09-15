package iso27001.suricata

import rego.v1

compliant_input := {
	"security_tasks": [
		{"name": "Install Suricata", "apt": {"name": "suricata", "state": "present"}},
		{"name": "Enable Suricata service", "systemd": {"name": "suricata", "state": "started", "enabled": true}},
	],
	"hardening_tasks": [],
	"suricata_rules_template": "alert http any any -> any any (msg:\"KoproGo SQL Injection attempt\"; content:\"UNION\"; sid:1000001;)\nalert http any any -> any any (msg:\"KoproGo XSS attempt\"; content:\"<script\"; sid:1000003;)\n",
}

# @happy
test_allows_compliant_config if {
	deny == set() with input as compliant_input
}

# @negative — IDS package never installed.
test_denies_missing_install if {
	tampered := object.union(compliant_input, {"security_tasks": [compliant_input.security_tasks[1]]})
	some msg in deny with input as tampered
	contains(msg, "installs the suricata package")
}

# @negative — installed but never started (blind IDS).
test_denies_dormant_service if {
	tampered := object.union(compliant_input, {"security_tasks": [compliant_input.security_tasks[0]]})
	some msg in deny with input as tampered
	contains(msg, "enables+starts")
}

# @security — an emptied local ruleset (e.g. accidentally overwritten by an
# unrelated template refactor) must fail, not pass because "suricata is
# installed" was mistaken for "suricata detects anything".
test_denies_ruleset_without_sqli_coverage if {
	tampered := object.union(compliant_input, {"suricata_rules_template": "alert http any any -> any any (msg:\"KoproGo XSS attempt\"; content:\"<script\"; sid:1000003;)\n"})
	some msg in deny with input as tampered
	contains(msg, "SQL injection")
}
