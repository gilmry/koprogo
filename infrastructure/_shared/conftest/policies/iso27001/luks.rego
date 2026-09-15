# ISO 27001 A.8.24 — Use of cryptography (LUKS full-disk encryption).
#
# Real-world caveat (documented, not silently ignored — see
# docs/agent-activity/2026-09-14-wp-e2-iac-tests.md): `cryptsetup luksFormat`
# is destructive (it can wipe a device), so the deployed
# koprogo-luks-setup.sh script is intentionally never auto-invoked by a task
# — a human runs it once against the correct device. This policy therefore
# only verifies the *capability* is deployed correctly with strong crypto
# parameters, not that encryption is active on a given host (that can only
# be observed at runtime, e.g. `cryptsetup status`).
package iso27001.luks

import rego.v1

has_task(tasks, mod, pred) if {
	some task in tasks
	task[mod]
	pred(task)
}

installs_cryptsetup if {
	has_task(input.security_tasks, "apt", pkg_has_cryptsetup)
}

pkg_has_cryptsetup(task) if {
	names := task.apt.name
	is_array(names)
	"cryptsetup" in names
}

deploys_setup_script if {
	has_task(input.security_tasks, "template", is_luks_script_task)
}

is_luks_script_task(task) if {
	task.template.src == "luks-setup.sh.j2"
	task.template.dest == "/usr/local/bin/koprogo-luks-setup.sh"
	task.template.mode == "0700"
}

deny[msg] if {
	not installs_cryptsetup
	msg := "A.8.24 LUKS: no task in security role installs cryptsetup/cryptsetup-bin"
}

deny[msg] if {
	not deploys_setup_script
	msg := "A.8.24 LUKS: no task deploys luks-setup.sh.j2 to /usr/local/bin/koprogo-luks-setup.sh mode 0700"
}

deny[msg] if {
	not contains(input.luks_script, "--cipher aes-xts-plain64")
	msg := "A.8.24 LUKS: setup script does not use AES-XTS cipher (aes-xts-plain64)"
}

deny[msg] if {
	not contains(input.luks_script, "--key-size 512")
	msg := "A.8.24 LUKS: setup script does not use a 512-bit key"
}

deny[msg] if {
	not contains(input.luks_script, "--hash sha512")
	msg := "A.8.24 LUKS: setup script does not use SHA-512 hashing"
}
