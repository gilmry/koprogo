#!/usr/bin/env bash
# PreToolUse hook: print contextual warnings for sensitive edits without blocking.
# (settings.json `ask` already gates user approval; this just adds context.)
#
# Exit 0 always.
set -u

input="$(cat)"

if command -v jq >/dev/null 2>&1; then
  file_path="$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty')"
else
  file_path="$(printf '%s' "$input" | grep -oE '"file_path":"[^"]*"' | head -1 | sed 's/.*:"\([^"]*\)"/\1/')"
fi

[ -z "$file_path" ] && exit 0

norm="$(printf '%s' "$file_path" | tr '\\' '/' | tr '[:upper:]' '[:lower:]')"

warn() {
  printf 'GUARDRAIL CONTEXT: %s\n' "$1" >&2
}

case "$norm" in
  */backend/migrations/*.sql)
    warn "Editing a SQL migration. If this migration was already applied (dev/staging), DO NOT modify it; create a new migration with a corrective change instead. Otherwise document the modification reason." ;;
  */claude.md|*/claude_md)
    warn "Editing CLAUDE.md (auto-loaded by every agent session). Keep it lean (<5k tokens cible). No \"NOUVEAU\" log, no roadmap, no endpoint listings (those go in docs/)." ;;
  */docker-compose.yml|*/docker-compose.*.yml)
    warn "Editing docker-compose.yml. Verify no hardcoded secrets (use .env + env_file:). Verify ports binding (prefer 127.0.0.1:port for dev only)." ;;
  */dockerfile|*/dockerfile.*)
    warn "Editing a Dockerfile. Verify USER non-root, multi-stage build, no secrets in layers, base image pinned by digest." ;;
  */.github/workflows/*)
    warn "Editing a GitHub Actions workflow. Verify: permissions: minimal, secrets via secrets.* not env, third-party actions pinned by SHA, no continue-on-error masking real failures." ;;
  */infrastructure/*/*.tf|*/infrastructure/*.tf)
    warn "Editing Terraform code. Verify: outputs sensitive=true for credentials, no hardcoded passwords, providers version pinned, prevent_destroy on critical resources." ;;
  */infrastructure/*/values*.yaml|*/infrastructure/*/helm-values.yaml)
    warn "Editing Helm values. Verify: no plaintext secrets (use SealedSecrets/ExternalSecrets), images pinned by digest (no :latest), securityContext set, network policies respected." ;;
  */backend/src/infrastructure/database/seed.rs)
    warn "Editing seed.rs. Avoid PII in seeds. Avoid letting seed file grow > 800 LOC; split by domain if needed (current state is already a problem to revisit)." ;;
  */cargo.toml|*/package.json)
    warn "Editing dependency manifest. Verify: pinned versions for direct deps, security audit (cargo audit / npm audit) before committing, no dev-deps in main." ;;
esac

exit 0
