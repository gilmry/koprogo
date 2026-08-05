#!/usr/bin/env bash
# Stop hook: scan for secrets in staged + unstaged changes before agent stops.
# Uses gitleaks if available; falls back to a minimal grep.
#
# Exit 2 -> block stop (forces agent to address the leak).
# Exit 0 -> allow stop.
set -u

# Skip if not in a git repo (e.g., running outside the project)
if ! git rev-parse --git-dir >/dev/null 2>&1; then
  exit 0
fi

leak_found=0
leak_msg=""

if command -v gitleaks >/dev/null 2>&1; then
  config_arg=""
  if [ -f "${CLAUDE_PROJECT_DIR:-.}/.gitleaks.toml" ]; then
    config_arg="--config=${CLAUDE_PROJECT_DIR:-.}/.gitleaks.toml"
  fi

  # Scan staged + working tree (uncommitted changes)
  if ! gitleaks protect $config_arg --no-banner --redact --exit-code 1 --staged 2>/tmp/gitleaks-staged.log; then
    leak_found=1
    leak_msg+="$(cat /tmp/gitleaks-staged.log 2>/dev/null || true)"$'\n'
  fi
  if ! gitleaks protect $config_arg --no-banner --redact --exit-code 1 2>/tmp/gitleaks-wt.log; then
    leak_found=1
    leak_msg+="$(cat /tmp/gitleaks-wt.log 2>/dev/null || true)"$'\n'
  fi
else
  # Minimal fallback: scan diff for AWS keys, GitHub PATs, PEM blocks.
  # Only scan ADDED lines (^+, excluding the +++ file header) — grepping the
  # full diff also matches unchanged context lines pulled into a hunk by an
  # unrelated nearby edit, which false-positives on any file that documents
  # these patterns as literal text (e.g. AGENT_GUARDRAILS.md describing the
  # pretool-deny-secret-write.sh regexes, or AWS's canonical docs example
  # access key, deliberately not spelled out here to avoid self-matching
  # this very scanner). Confirmed false positive + fix authorized by
  # @gilmry 2026-07-31 (AskUserQuestion), cf. docs/agent-activity/2026-07-31-fix-stop-leak-scan.md.
  diff_content="$(git diff HEAD 2>/dev/null; git diff --cached 2>/dev/null)"
  added_lines="$(printf '%s' "$diff_content" | grep -E '^\+' | grep -vE '^\+\+\+')"
  if printf '%s' "$added_lines" | grep -qE 'AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{36}|BEGIN (RSA |OPENSSH |EC )?PRIVATE KEY'; then
    leak_found=1
    leak_msg="Pattern secret detected in added lines (gitleaks not installed; fallback grep)."
  fi
fi

# Also check git diff --check for whitespace and merge conflict markers
if ! git diff --check 2>/dev/null >/dev/null; then
  printf 'GUARDRAIL WARN: git diff --check reports whitespace/conflict issues.\n' >&2
fi

if [ "$leak_found" -eq 1 ]; then
  printf 'GUARDRAIL BLOCK: secret leak detected in your changes.\n' >&2
  printf '%s\n' "$leak_msg" >&2
  printf '\nRemove the secret from the diff. If false positive, add an allowlist rule to .gitleaks.toml.\n' >&2
  exit 2
fi

exit 0
