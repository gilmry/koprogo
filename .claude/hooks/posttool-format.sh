#!/usr/bin/env bash
# PostToolUse hook: auto-format the file just edited based on its extension.
# Best-effort; never block.
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
[ ! -f "$file_path" ] && exit 0

# Best-effort format dispatch by extension
case "$file_path" in
  *.rs)
    if command -v rustfmt >/dev/null 2>&1; then
      rustfmt --edition 2021 "$file_path" 2>/dev/null || true
    fi ;;
  *.ts|*.tsx|*.js|*.jsx|*.svelte|*.json|*.md|*.yaml|*.yml|*.css|*.html)
    if command -v prettier >/dev/null 2>&1; then
      prettier --write --log-level silent "$file_path" 2>/dev/null || true
    elif command -v npx >/dev/null 2>&1; then
      (cd "$(dirname "$file_path")" 2>/dev/null && npx --no-install prettier --write --log-level silent "$file_path" 2>/dev/null) || true
    fi ;;
  *.tf|*.tfvars)
    if command -v terraform >/dev/null 2>&1; then
      terraform fmt "$file_path" >/dev/null 2>&1 || true
    fi ;;
  *.sh)
    if command -v shfmt >/dev/null 2>&1; then
      shfmt -w "$file_path" 2>/dev/null || true
    fi ;;
  *.py)
    if command -v black >/dev/null 2>&1; then
      black --quiet "$file_path" 2>/dev/null || true
    fi ;;
esac

exit 0
