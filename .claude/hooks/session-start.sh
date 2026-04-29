#!/usr/bin/env bash
# SessionStart hook: verify deps, warn on protected branch, show banner.
# Output goes to stdout (visible at session start).
#
# Exit 0 always.
set -u

printf '\n'
printf '╔══════════════════════════════════════════════════════════════════════╗\n'
printf '║  KoproGo guardrails actifs (cf. issues #425 #426 #427 #428)         ║\n'
printf '╠══════════════════════════════════════════════════════════════════════╣\n'

# Branch check
if git rev-parse --git-dir >/dev/null 2>&1; then
  branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
  case "$branch" in
    main|master|production|staging)
      printf '║  ⚠  Branche actuelle = %-43s  ║\n' "$branch (protégée)"
      printf '║     Crée une branche feature/* ou story/<id> avant de coder.        ║\n' ;;
    *)
      printf '║  Branche : %-57s ║\n' "$branch" ;;
  esac
fi

# Dep checks
deps_ok=1
for dep in git gh; do
  if ! command -v "$dep" >/dev/null 2>&1; then
    printf '║  ✗ Manque : %-56s ║\n' "$dep (obligatoire)"
    deps_ok=0
  fi
done

for dep in gitleaks jq cargo npm rustfmt prettier; do
  if ! command -v "$dep" >/dev/null 2>&1; then
    printf '║  ⚠ Optionnel manquant : %-44s ║\n' "$dep"
  fi
done

if [ "$deps_ok" -eq 1 ]; then
  printf '║  ✓ Outils essentiels présents                                        ║\n'
fi

printf '╠══════════════════════════════════════════════════════════════════════╣\n'
printf '║  Hooks : PreToolUse (deny secret/prod), PostToolUse (fmt + warn),   ║\n'
printf '║          UserPromptSubmit (rules), Stop (gitleaks), SessionStart    ║\n'
printf '║  Doc   : .claude/AGENT_GUARDRAILS.md                                 ║\n'
printf '╚══════════════════════════════════════════════════════════════════════╝\n\n'

exit 0
