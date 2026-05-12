#!/usr/bin/env bash
# UserPromptSubmit hook: inject the CRITICAL rules into the agent's context
# at the start of every user turn. stdout is prepended to context.
#
# Exit 0.
set -u

rules_file="${CLAUDE_PROJECT_DIR:-.}/.claude/rules/CRITICAL.md"

if [ -f "$rules_file" ]; then
  printf '\n<system-injected from=".claude/rules/CRITICAL.md">\n'
  cat "$rules_file"
  printf '\n</system-injected>\n\n'
fi

exit 0
