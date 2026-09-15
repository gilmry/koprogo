package iso27001.luks

import rego.v1

compliant_input := {
	"security_tasks": [
		{
			"name": "Install LUKS encryption tools",
			"apt": {"name": ["cryptsetup", "cryptsetup-bin"], "state": "present"},
		},
		{
			"name": "Deploy LUKS setup script",
			"template": {
				"src": "luks-setup.sh.j2",
				"dest": "/usr/local/bin/koprogo-luks-setup.sh",
				"mode": "0700",
			},
		},
	],
	"luks_script": "cryptsetup luksFormat \"$DEVICE\" --cipher aes-xts-plain64 --key-size 512 --hash sha512 --use-random",
}

# @happy — a role that installs cryptsetup, deploys the script with the
# right permissions, and uses strong crypto parameters passes clean.
test_allows_compliant_config if {
	deny == set() with input as compliant_input
}

# @negative — no cryptsetup package install task at all.
test_denies_missing_cryptsetup_install if {
	tampered := object.union(compliant_input, {"security_tasks": [compliant_input.security_tasks[1]]})
	some msg in deny with input as tampered
	contains(msg, "cryptsetup")
}

# @negative — script deployed to the wrong path/mode (e.g. world-readable).
test_denies_missing_script_deploy if {
	tampered := object.union(compliant_input, {"security_tasks": [compliant_input.security_tasks[0]]})
	some msg in deny with input as tampered
	contains(msg, "luks-setup.sh.j2")
}

# @security — a weakened cipher (e.g. aes-cbc instead of aes-xts-plain64)
# must fail the gate, not pass silently.
test_denies_weak_cipher if {
	tampered := object.union(compliant_input, {"luks_script": "cryptsetup luksFormat \"$DEVICE\" --cipher aes-cbc-essiv:sha256 --key-size 256"})
	some msg in deny with input as tampered
	contains(msg, "AES-XTS")
}

# @edge — script present but key size below the 512-bit baseline.
test_denies_weak_key_size if {
	tampered := object.union(compliant_input, {"luks_script": "cryptsetup luksFormat \"$DEVICE\" --cipher aes-xts-plain64 --key-size 256 --hash sha512"})
	some msg in deny with input as tampered
	contains(msg, "512-bit")
}
