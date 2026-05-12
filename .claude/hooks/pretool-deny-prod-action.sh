#!/usr/bin/env bash
# PreToolUse hook: block dangerous Bash commands as defense in depth on top
# of permissions.deny. Permissions use glob patterns; this hook does flexible
# regex matching catching obfuscations (env var prefixes, variable expansions).
#
# Exit 2 -> block. Exit 0 -> allow.
set -u

input="$(cat)"

if command -v jq >/dev/null 2>&1; then
  command="$(printf '%s' "$input" | jq -r '.tool_input.command // empty')"
else
  command="$(printf '%s' "$input" | grep -oE '"command":"[^"]*"' | head -1 | sed 's/.*:"\([^"]*\)"/\1/')"
fi

[ -z "$command" ] && exit 0

block() {
  printf 'GUARDRAIL BLOCK: command refused.\n' >&2
  printf 'Reason: %s\n' "$1" >&2
  printf 'Command: %s\n' "$command" >&2
  printf '\nProd actions require human invocation. The agent cannot run this autonomously.\n' >&2
  exit 2
}

# Strip env var prefixes (FOO=bar BAZ=qux command...)
stripped="$(printf '%s' "$command" | sed -E 's/^([A-Z_][A-Z0-9_]*=[^ ]+ +)+//')"

# Block patterns
case "$stripped" in
  *"terraform apply"*|*"terraform destroy"*|*"terraform import"*|*"terraform state rm"*|*"terraform state push"*)
    block "Terraform mutation outside read-only plan/validate." ;;
  *"helm install"*|*"helm upgrade"*|*"helm uninstall"*|*"helm delete"*|*"helm rollback"*)
    block "Helm cluster mutation." ;;
  *"kubectl apply"*|*"kubectl delete"*|*"kubectl create"*|*"kubectl edit"*|*"kubectl patch"*|*"kubectl replace"*|*"kubectl scale"*|*"kubectl drain"*|*"kubectl exec"*|*"kubectl cp"*)
    block "kubectl mutation/exec." ;;
  *"argocd app sync"*|*"argocd app delete"*|*"argocd app create"*|*"argocd app set"*)
    block "ArgoCD mutation." ;;
  *"velero backup delete"*|*"velero restore"*)
    block "Velero backup/restore mutation." ;;
  *"git push --force"*|*"git push -f "*|*"git push -f$"*|*"git push --force-with-lease"*)
    block "Force push." ;;
  *"git reset --hard"*|*"git clean -fd"*|*"git clean -fdx"*)
    block "Destructive git operation." ;;
  *"git commit --no-verify"*|*"git commit -n "*|*"git push --no-verify"*)
    block "Bypass of git hooks (--no-verify) refused." ;;
  *"git filter-branch"*|*"git filter-repo"*)
    block "History rewrite refused." ;;
  *"npm publish"*|*"cargo publish"*|*"docker push"*)
    block "Publishing to a public registry." ;;
  *"gh release create"*|*"gh release delete"*|*"gh repo delete"*|*"gh secret set"*|*"gh secret delete"*)
    block "GitHub release/repo/secret mutation." ;;
  *"rm -rf /"*|*"rm -rf ~"*|*"rm -rf \$HOME"*|*"rm -rf \${HOME}"*)
    block "Recursive delete on root/home." ;;
  *"curl"*"|"*"sh"*|*"curl"*"|"*"bash"*|*"wget"*"|"*"sh"*|*"wget"*"|"*"bash"*)
    block "curl/wget piped to shell (supply-chain risk)." ;;
  *":(){ :|"*)
    block "Fork bomb pattern." ;;
esac

exit 0
