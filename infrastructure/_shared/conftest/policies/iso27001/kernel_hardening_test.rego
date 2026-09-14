package iso27001.kernel_hardening

import rego.v1

good_sysctl := `net.ipv4.tcp_syncookies = 1
net.ipv4.conf.all.rp_filter = 1
net.ipv4.conf.default.rp_filter = 1
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.all.accept_redirects = 0
kernel.dmesg_restrict = 1
kernel.kptr_restrict = 2
`

compliant_input := {
	"security_tasks": [
		{"name": "Apply sysctl hardening", "command": "sysctl -p /etc/sysctl.d/99-koprogo-hardening.conf"},
	],
	"sysctl_template": good_sysctl,
}

# @happy
test_allows_compliant_config if {
	deny == set() with input as compliant_input
}

# @negative — the hardening file is templated but never applied at runtime,
# so it silently does nothing until the next reboot.
test_denies_missing_apply_task if {
	tampered := object.union(compliant_input, {"security_tasks": []})
	some msg in deny with input as tampered
	contains(msg, "sysctl -p")
}

# @security — IP spoofing protection (rp_filter) removed from the template.
test_denies_missing_rp_filter if {
	tampered := object.union(compliant_input, {"sysctl_template": replace(good_sysctl, "net.ipv4.conf.all.rp_filter = 1\n", "")})
	some msg in deny with input as tampered
	contains(msg, "rp_filter")
}

# @edge — SYN-flood protection disabled (tcp_syncookies flipped to 0).
test_denies_syncookies_disabled if {
	tampered := object.union(compliant_input, {"sysctl_template": replace(good_sysctl, "net.ipv4.tcp_syncookies = 1", "net.ipv4.tcp_syncookies = 0")})
	some msg in deny with input as tampered
	contains(msg, "tcp_syncookies")
}
