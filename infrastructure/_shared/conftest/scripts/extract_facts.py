#!/usr/bin/env python3
"""Extract ISO 27001 compliance facts from the Ansible roles + monitoring
compose file into one JSON document for `conftest test` (issue #354 WP-E2).

Why a facts document instead of feeding raw files to conftest: the
security-critical directives (PermitRootLogin, sysctl keys, fail2ban
bantime...) live in Jinja2 templates that are not valid YAML/INI on their
own, and the same control (fail2ban, CrowdSec, Suricata) is implemented by
two overlapping roles (security, hardening — see docs/agent-activity 2026-09-14
WP-E2 log). A single facts document lets the Rego policies assert on the
real deployed content once, regardless of which role currently owns it.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[4]
ROLES = ROOT / "infrastructure" / "_shared" / "ansible" / "roles"
MONITORING_COMPOSE = (
    ROOT / "infrastructure" / "_shared" / "monitoring" / "docker-compose.monitoring.yml"
)


def load_tasks(role: str) -> list:
    path = ROLES / role / "tasks" / "main.yml"
    with path.open(encoding="utf-8") as f:
        tasks = yaml.safe_load(f) or []
    if not isinstance(tasks, list):
        raise ValueError(f"{path} did not parse to a task list")
    return tasks


def load_text(role: str, template: str) -> str:
    path = ROLES / role / "templates" / template
    return path.read_text(encoding="utf-8")


def load_compose_services() -> list:
    if not MONITORING_COMPOSE.exists():
        return []
    with MONITORING_COMPOSE.open(encoding="utf-8") as f:
        compose = yaml.safe_load(f) or {}
    return sorted((compose.get("services") or {}).keys())


def main() -> None:
    facts = {
        "security_tasks": load_tasks("security"),
        "hardening_tasks": load_tasks("hardening"),
        "backup_tasks": load_tasks("backup"),
        "luks_script": load_text("security", "luks-setup.sh.j2"),
        "sshd_template": load_text("security", "sshd_config_hardening.j2"),
        "sysctl_template": load_text("security", "sysctl-hardening.conf.j2"),
        "fail2ban_jail_template": load_text("security", "fail2ban-koprogo.conf.j2"),
        "suricata_rules_template": load_text("security", "suricata-local.rules.j2"),
        "monitoring_compose_services": load_compose_services(),
    }
    out = sys.argv[1] if len(sys.argv) > 1 else None
    text = json.dumps(facts, indent=2, sort_keys=True)
    if out:
        Path(out).write_text(text + "\n", encoding="utf-8")
    else:
        print(text)


if __name__ == "__main__":
    main()
