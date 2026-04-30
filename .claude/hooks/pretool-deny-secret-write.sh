#!/usr/bin/env bash
# PreToolUse hook: block writes to secret-bearing files OR content matching
# secret patterns. Defense in depth on top of permissions.deny.
#
# Exit 2 -> block tool use, stderr shown to agent.
# Exit 0 -> allow.
set -u

input="$(cat)"

if command -v jq >/dev/null 2>&1; then
  tool_name="$(printf '%s' "$input" | jq -r '.tool_name // empty')"
  file_path="$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty')"
  content="$(printf '%s' "$input" | jq -r '.tool_input.content // .tool_input.new_string // empty')"
else
  tool_name="$(printf '%s' "$input" | grep -oE '"tool_name":"[^"]*"' | head -1 | sed 's/.*:"\([^"]*\)"/\1/')"
  file_path="$(printf '%s' "$input" | grep -oE '"file_path":"[^"]*"' | head -1 | sed 's/.*:"\([^"]*\)"/\1/')"
  content=""
fi

[ -z "$file_path" ] && exit 0

norm="$(printf '%s' "$file_path" | tr '\\' '/' | tr '[:upper:]' '[:lower:]')"

block() {
  printf 'GUARDRAIL BLOCK: %s\n' "$1" >&2
  printf 'File: %s\n' "$file_path" >&2
  printf 'Tool: %s\n' "$tool_name" >&2
  printf '\nIf you genuinely need this edit, ask the human to do it manually.\n' >&2
  exit 2
}

# Allowlist: security-tool config files are expected to reference secret patterns.
# Same for issue body drafts (they may quote audit findings verbatim).
# Use trailing-match wildcards to catch both bare filenames and prefixed paths.
case "$norm" in
  *gitleaks.toml|*trivyignore|*trufflehogignore|*detect-secrets-baseline|*semgrep.yml|*semgrep.yaml)
    exit 0 ;;
  */.claude/_drafts/*|*/.claude/rules/*|*/.claude/hooks/*|*/.claude/agents/*|.claude/_drafts/*|.claude/rules/*|.claude/hooks/*|.claude/agents/*)
    exit 0 ;;
  *agent_guardrails*.md|*agent-guardrails*.md)
    exit 0 ;;
esac

# Path-based deny
case "$norm" in
  */.env|*/.env.local|*/.env.production|*/.env.staging|*/.env.loadtest|*/.env.integration)
    block "Refusing env file with real credentials. Use .env.example with placeholders." ;;
  */age.key|*/age.txt|*/.sops.yaml|*/.sops/*|*infrastructure/_shared/secrets/*)
    block "Refusing SOPS/age/secrets material. Rotate via human-only path." ;;
  *id_rsa|*id_ed25519|*id_ecdsa|*.pem|*.key|*.p12|*.pfx|*.gpg|*.asc)
    block "Refusing key/cert material. Generate locally, reference path only." ;;
  */.vault_pass|*/.vault_pass.txt|*/.ansible-vault-password)
    block "Refusing Ansible Vault password files." ;;
  */kubeconfig|*/kubeconfig.yaml|*/.kube/config)
    block "Refusing kubeconfig (admin credentials)." ;;
  *.tfstate|*.tfstate.backup|*.tfstate.lock.info)
    block "Refusing Terraform state files (managed by remote backend)." ;;
esac

# Content-based detection
if [ -n "$content" ] && [ "$content" != "null" ]; then
  if printf '%s' "$content" | grep -qE 'AKIA[0-9A-Z]{16}'; then
    block "AWS Access Key ID pattern in content."
  fi
  if printf '%s' "$content" | grep -qE 'ghp_[A-Za-z0-9]{36}|gho_[A-Za-z0-9]{36}|ghs_[A-Za-z0-9]{36}'; then
    block "GitHub token pattern in content."
  fi
  if printf '%s' "$content" | grep -qE 'xox[baprs]-[0-9A-Za-z-]{10,}'; then
    block "Slack token pattern in content."
  fi
  if printf '%s' "$content" | grep -qE 'BEGIN (RSA |OPENSSH |EC |DSA |PGP )?PRIVATE KEY'; then
    block "PEM private key block in content."
  fi

  # Hardcoded credentials (skip example/template/test files and known placeholders)
  case "$norm" in
    *.example|*example.*|*template.*|*.tpl|*.sample|*test*|*mock*|*fixture*) : ;;
    *)
      # Match both quoted ("value") and unquoted YAML/INI (key: value) credentials, 12+ chars
      # -i for case-insensitive (postgresPassword, JwtSecret, API_KEY all match)
      if printf '%s' "$content" | grep -qiE "(password|passwd|secret|api[_-]?key|access[_-]?token|jwt[_-]?secret)[[:space:]]*[:=][[:space:]]*[\"'\`]?[A-Za-z0-9!@#%^&*+=._/+-]{12,}"; then
        if ! printf '%s' "$content" | grep -qiE '(changeme|placeholder|example|fixme|dummy|<your|todo|xxxx|<replace|<insert|test123|mockvalue|\$\{|\{\{)'; then
          block "Hardcoded credential pattern. Use a secret reference (Vault, ExternalSecret, env var) instead."
        fi
      fi ;;
  esac
fi

exit 0
