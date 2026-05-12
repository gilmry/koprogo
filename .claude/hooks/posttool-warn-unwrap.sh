#!/usr/bin/env bash
# PostToolUse hook: warn (not block) when an Edit/Write introduces:
#   - .unwrap() / .expect( in Rust files (outside tests)
#   - : any / as any in TS/Svelte files
# Helps surface Anti-pattern discipline (#427 TDD/BDD + #425 quality).
#
# Exit 0 always; just stderr warnings.
set -u

input="$(cat)"

if command -v jq >/dev/null 2>&1; then
  file_path="$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty')"
  new_content="$(printf '%s' "$input" | jq -r '.tool_input.new_string // .tool_input.content // empty')"
else
  file_path="$(printf '%s' "$input" | grep -oE '"file_path":"[^"]*"' | head -1 | sed 's/.*:"\([^"]*\)"/\1/')"
  new_content=""
fi

[ -z "$file_path" ] && exit 0
[ -z "$new_content" ] || [ "$new_content" = "null" ] && exit 0

warn() {
  printf 'GUARDRAIL WARN: %s\n' "$1" >&2
  printf 'File: %s\n' "$file_path" >&2
}

case "$file_path" in
  *_test.rs|*/tests/*.rs|*tests/features/*) : ;; # tests can use unwrap freely
  *.rs)
    unwrap_count=$(printf '%s' "$new_content" | grep -cE '\.unwrap\(\)|\.expect\(' 2>/dev/null || true)
    if [ "${unwrap_count:-0}" -gt 0 ]; then
      warn "$unwrap_count occurrence(s) of .unwrap() / .expect( introduced. Prefer typed AppError + ? operator. Cf. #425 + #427 (4-cat negative tests)."
    fi
    ;;
  *.ts|*.tsx|*.svelte)
    any_count=$(printf '%s' "$new_content" | grep -cE ':\s*any[^a-zA-Z]|as\s+any[^a-zA-Z]' 2>/dev/null || true)
    if [ "${any_count:-0}" -gt 0 ]; then
      warn "$any_count occurrence(s) of \`: any\` / \`as any\` introduced. TypeScript strict expects typed payloads. Cf. #427 (frontend TS discipline)."
    fi
    ;;
esac

exit 0
