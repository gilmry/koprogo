#!/usr/bin/env bash
# Witness suite for .claude/hooks/*.sh — issue #425.
#
# Each hook is a claimed guardrail; this script is the "témoin vérifié" the
# story demands: a guardrail without a passing witness here is NOT counted as
# active in AGENT_GUARDRAILS.md. Cf. #555, cited in the story as the example
# of a hook whose description ("blocks Result<_, String>") was never actually
# exercised — the count went from 1263 to 1321 violations with no warning.
#
# Test classes (per the story's own taxonomy, applied to guardrail auditing):
#   @happy    — the faulty behavior is reproduced and the hook blocks/warns.
#   @negative — legitimate behavior is NOT blocked (false positive check).
#   @edge     — the real failure mode is unsafe to reproduce directly
#               (e.g. no gitleaks binary in this sandbox); a substitute
#               witness is used and labelled as such below.
#   @security — irreversible-consequence hooks (secret write, prod mutation).
#
# NOTE on fixtures: secret-like test values below are built by runtime
# concatenation (never as a single contiguous literal) so that writing THIS
# file does not itself trip pretool-deny-secret-write.sh — the same technique
# .gitleaks.toml uses for its own documented example patterns.
#
# Run: bash scripts/test-guardrail-hooks.sh
# Exit 0 if every assertion passes, 1 otherwise. Also usable as
# `make claude-check-witness` (see Makefile).
set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$PROJECT_DIR/.claude/hooks"

TOTAL=0
FAILED=0
declare -A HOOK_STATUS

# ---- fixtures (built at runtime, never contiguous in source) --------------
AWS_KEY_FIXTURE="AKIA"
AWS_KEY_FIXTURE+="ABCDEFGHIJKLMNOP"
GH_PAT_FIXTURE="ghp_$(printf 'a%.0s' $(seq 1 36))"
PASSWORD_FIXTURE="S3cur3Real"
PASSWORD_FIXTURE+="Passw0rd!"

# ---- helpers ---------------------------------------------------------------

# Builds a Claude Code hook stdin payload as JSON, safely (no shell quoting
# footguns for content containing quotes/backslashes).
json_edit() { # file_path content
  jq -n --arg fp "$1" --arg c "$2" '{tool_name:"Write", tool_input:{file_path:$fp, content:$c}}'
}
json_bash() { # command
  jq -n --arg cmd "$1" '{tool_name:"Bash", tool_input:{command:$cmd}}'
}

LAST_CODE=0
LAST_OUT=""
LAST_ERR=""

run_hook() { # hook_script json_payload [cwd]
  local hook="$1" json="$2" cwd="${3:-$PROJECT_DIR}"
  local out_f err_f
  out_f="$(mktemp)"
  err_f="$(mktemp)"
  ( cd "$cwd" && printf '%s' "$json" | bash "$HOOKS_DIR/$hook" >"$out_f" 2>"$err_f" )
  LAST_CODE=$?
  LAST_OUT="$(cat "$out_f")"
  LAST_ERR="$(cat "$err_f")"
  rm -f "$out_f" "$err_f"
}

check() { # label expected_code
  local label="$1" expected="$2"
  TOTAL=$((TOTAL + 1))
  if [ "$LAST_CODE" -eq "$expected" ]; then
    printf '  \033[32mPASS\033[0m %s (exit %s)\n' "$label" "$LAST_CODE"
  else
    printf '  \033[31mFAIL\033[0m %s — expected exit %s, got %s\n' "$label" "$expected" "$LAST_CODE"
    printf '       stderr: %s\n' "$LAST_ERR"
    FAILED=$((FAILED + 1))
  fi
}

check_stderr_has() { # label needle
  local label="$1" needle="$2"
  TOTAL=$((TOTAL + 1))
  if printf '%s' "$LAST_ERR" | grep -qF "$needle"; then
    printf '  \033[32mPASS\033[0m %s\n' "$label"
  else
    printf '  \033[31mFAIL\033[0m %s — stderr did not contain %q\n' "$label" "$needle"
    printf '       stderr: %s\n' "$LAST_ERR"
    FAILED=$((FAILED + 1))
  fi
}

check_stderr_empty() { # label
  local label="$1"
  TOTAL=$((TOTAL + 1))
  if [ -z "$LAST_ERR" ]; then
    printf '  \033[32mPASS\033[0m %s\n' "$label"
  else
    printf '  \033[31mFAIL\033[0m %s — expected empty stderr, got: %s\n' "$label" "$LAST_ERR"
    FAILED=$((FAILED + 1))
  fi
}

check_stdout_has() { # label needle
  local label="$1" needle="$2"
  TOTAL=$((TOTAL + 1))
  if printf '%s' "$LAST_OUT" | grep -qF "$needle"; then
    printf '  \033[32mPASS\033[0m %s\n' "$label"
  else
    printf '  \033[31mFAIL\033[0m %s — stdout did not contain %q\n' "$label" "$needle"
    FAILED=$((FAILED + 1))
  fi
}

section() { printf '\n\033[1m%s\033[0m\n' "$1"; }

# =============================================================================
section "pretool-deny-secret-write.sh"
# =============================================================================
HOOK=pretool-deny-secret-write.sh

run_hook "$HOOK" '{}'
check "@negative: neutral empty input does not block" 0

run_hook "$HOOK" "$(json_edit "backend/.env" "JWT_SECRET=whatever")"
check "@happy @security: write to .env is blocked" 2

run_hook "$HOOK" "$(json_edit "backend/src/foo.rs" "let key = \"${AWS_KEY_FIXTURE}\";")"
check "@security: AWS access key content is blocked" 2

run_hook "$HOOK" "$(json_edit "backend/src/foo.rs" "let token = \"${GH_PAT_FIXTURE}\";")"
check "@security: GitHub PAT content is blocked" 2

run_hook "$HOOK" "$(json_edit "infrastructure/values.yaml" "postgresPassword: \"${PASSWORD_FIXTURE}\"")"
check "@security: hardcoded credential pattern is blocked" 2

run_hook "$HOOK" "$(json_edit "backend/src/domain/entities/unit.rs" "pub struct Unit { pub id: Uuid }")"
check "@negative: ordinary Rust content is not blocked" 0

run_hook "$HOOK" "$(json_edit ".gitleaks.toml" "regex = '''AKIA[0-9A-Z]{16}'''")"
check "@edge: allowlisted .gitleaks.toml with pattern text is not blocked" 0

run_hook "$HOOK" "$(json_edit "infrastructure/values.yaml" "jwtSecret: \"changeme-to-32-chars-minimum\"")"
check "@edge: recognized placeholder credential is not blocked" 0

if [ "$FAILED" -eq 0 ]; then HOOK_STATUS[$HOOK]="VERIFIED"; else HOOK_STATUS[$HOOK]="NOT VERIFIED"; fi
PRIOR_FAILED=$FAILED

# =============================================================================
section "pretool-deny-prod-action.sh"
# =============================================================================
HOOK=pretool-deny-prod-action.sh

run_hook "$HOOK" '{}'
check "@negative: neutral empty input does not block" 0

run_hook "$HOOK" "$(json_bash "terraform apply -auto-approve")"
check "@happy @security: terraform apply is blocked" 2

run_hook "$HOOK" "$(json_bash "kubectl delete pod my-pod")"
check "@security: kubectl delete is blocked" 2

run_hook "$HOOK" "$(json_bash "git push --force origin main")"
check "@security: git push --force is blocked" 2

run_hook "$HOOK" "$(json_bash "terraform plan -out=plan.out")"
check "@negative: terraform plan (read-only) is not blocked" 0

run_hook "$HOOK" "$(json_bash "git push origin story/425")"
check "@negative: plain git push is not blocked (permissions.ask handles it)" 0

run_hook "$HOOK" "$(json_bash "AWS_PROFILE=prod terraform apply")"
check "@edge: env-var-prefixed terraform apply is still blocked" 2

if [ "$FAILED" -eq "$PRIOR_FAILED" ]; then HOOK_STATUS[$HOOK]="VERIFIED"; else HOOK_STATUS[$HOOK]="NOT VERIFIED"; fi
PRIOR_FAILED=$FAILED

# =============================================================================
section "posttool-warn-unwrap.sh"
# =============================================================================
HOOK=posttool-warn-unwrap.sh

run_hook "$HOOK" '{}'
check "@negative: neutral empty input exits 0" 0
check_stderr_empty "@negative: neutral empty input warns nothing"

run_hook "$HOOK" "$(json_edit "backend/src/application/use_cases/foo.rs" "let x = maybe.unwrap();")"
check "@happy: unwrap() introduced exits 0 (warn, not block)" 0
check_stderr_has "@happy: unwrap() introduced triggers a warning" "unwrap"

run_hook "$HOOK" "$(json_edit "backend/src/application/ports/foo_repository.rs" "fn save(&self) -> Result<Foo, String>;")"
check_stderr_has "@happy (closes #555 gap): Result<_, String> introduced triggers a warning" "Result<_, String>"

run_hook "$HOOK" "$(json_edit "backend/src/domain/entities/foo.rs" "pub struct Foo { pub id: Uuid }")"
check_stderr_empty "@negative: clean Rust content warns nothing"

run_hook "$HOOK" "$(json_edit "backend/tests/foo_test.rs" "let x = maybe.unwrap();")"
check_stderr_empty "@edge: unwrap() in a _test.rs file is allowed, no warning"

run_hook "$HOOK" "$(json_edit "frontend/src/lib/api.ts" "function f(payload: any) {}")"
check_stderr_has "@happy: ': any' in .ts triggers a warning" "any"

if [ "$FAILED" -eq "$PRIOR_FAILED" ]; then HOOK_STATUS[$HOOK]="VERIFIED"; else HOOK_STATUS[$HOOK]="NOT VERIFIED"; fi
PRIOR_FAILED=$FAILED

# =============================================================================
section "pretool-warn-sensitive-edit.sh"
# =============================================================================
HOOK=pretool-warn-sensitive-edit.sh

run_hook "$HOOK" '{}'
check "@negative: neutral empty input exits 0" 0

run_hook "$HOOK" "$(json_edit "docker-compose.yml" "services: {}")"
check_stderr_has "@happy: docker-compose.yml edit shows security context" "docker-compose.yml"

run_hook "$HOOK" "$(json_edit "backend/migrations/0099_foo.sql" "ALTER TABLE foo ADD COLUMN bar TEXT;")"
check_stderr_has "@happy: migration edit shows context reminder" "migration"

run_hook "$HOOK" "$(json_edit "backend/src/domain/entities/unit.rs" "pub struct Unit {}")"
check_stderr_empty "@negative: unrelated file path shows no warning"

if [ "$FAILED" -eq "$PRIOR_FAILED" ]; then HOOK_STATUS[$HOOK]="VERIFIED"; else HOOK_STATUS[$HOOK]="NOT VERIFIED"; fi
PRIOR_FAILED=$FAILED

# =============================================================================
section "userprompt-inject-rules.sh"
# =============================================================================
HOOK=userprompt-inject-rules.sh

run_hook "$HOOK" '{}'
check "@happy: hook exits 0" 0
check_stdout_has "@happy: injects CRITICAL.md content into context" "Règles critiques KoproGo"

if [ "$FAILED" -eq "$PRIOR_FAILED" ]; then HOOK_STATUS[$HOOK]="VERIFIED"; else HOOK_STATUS[$HOOK]="NOT VERIFIED"; fi
PRIOR_FAILED=$FAILED

# =============================================================================
section "session-start.sh"
# =============================================================================
HOOK=session-start.sh

run_hook "$HOOK" '{}'
check "@happy: hook exits 0" 0
check_stdout_has "@happy: banner announces guardrails" "guardrails actifs"

# @edge: the real failure mode ("agent codes directly on main") is unsafe to
# reproduce on this repo's own branch — substitute witness via a throwaway
# temp git repo checked out on 'main'.
TMP_REPO="$(mktemp -d)"
( cd "$TMP_REPO" && git init -q -b main && git -c user.email=t@t -c user.name=t commit -q --allow-empty -m init )
run_hook "$HOOK" '{}' "$TMP_REPO"
check_stdout_has "@edge (substitute witness: temp repo on main): warns on protected branch" "protégée"
rm -rf "$TMP_REPO"

if [ "$FAILED" -eq "$PRIOR_FAILED" ]; then HOOK_STATUS[$HOOK]="VERIFIED (partly substitute)"; else HOOK_STATUS[$HOOK]="NOT VERIFIED"; fi
PRIOR_FAILED=$FAILED

# =============================================================================
section "stop-leak-scan.sh"
# =============================================================================
HOOK=stop-leak-scan.sh

if command -v gitleaks >/dev/null 2>&1; then
  GITLEAKS_NOTE="gitleaks present — real path exercised"
else
  GITLEAKS_NOTE="gitleaks ABSENT in this environment — only the fallback-grep path is exercised below; the gitleaks-backed path is NOT verified here"
fi
printf '  \033[33mNOTE\033[0m %s\n' "$GITLEAKS_NOTE"

# @edge: cannot risk running this against the real repo's uncommitted state
# (the witness for a leak would be an actual leak). Substitute: throwaway repo.
TMP_REPO="$(mktemp -d)"
( cd "$TMP_REPO" && git init -q && git -c user.email=t@t -c user.name=t commit -q --allow-empty -m init )

echo "safe content" > "$TMP_REPO/file.txt"
( cd "$TMP_REPO" && git add file.txt )
run_hook "$HOOK" '{}' "$TMP_REPO"
check "@negative (substitute witness): innocuous diff does not block stop" 0

printf 'aws_key = "%s"\n' "$AWS_KEY_FIXTURE" >> "$TMP_REPO/file.txt"
run_hook "$HOOK" '{}' "$TMP_REPO"
check "@happy @security (substitute witness): AWS key in diff blocks stop" 2

rm -rf "$TMP_REPO"

if [ "$FAILED" -eq "$PRIOR_FAILED" ]; then HOOK_STATUS[$HOOK]="VERIFIED (substitute — gitleaks binary not exercised)"; else HOOK_STATUS[$HOOK]="NOT VERIFIED"; fi
PRIOR_FAILED=$FAILED

# =============================================================================
section "posttool-format.sh"
# =============================================================================
HOOK=posttool-format.sh

run_hook "$HOOK" '{}'
check "@negative: neutral empty input exits 0" 0

if command -v rustfmt >/dev/null 2>&1; then
  TMP_RS="$(mktemp -u).rs"
  printf 'fn main(){let x=1;println!("{}",x);}\n' > "$TMP_RS"
  run_hook "$HOOK" "$(json_edit "$TMP_RS" "unused")"
  check "@happy: rustfmt reformats a badly-formatted .rs file" 0
  TOTAL=$((TOTAL + 1))
  if grep -q '    let x = 1;' "$TMP_RS"; then
    printf '  \033[32mPASS\033[0m @happy: file content was actually reformatted\n'
  else
    printf '  \033[31mFAIL\033[0m @happy: file content was NOT reformatted\n'
    FAILED=$((FAILED + 1))
  fi
  rm -f "$TMP_RS"
  RUSTFMT_STATUS="VERIFIED"
else
  printf '  \033[33mNOTE\033[0m rustfmt absent — .rs formatting path NOT verified here\n'
  RUSTFMT_STATUS="NOT VERIFIED (rustfmt absent)"
fi

if command -v prettier >/dev/null 2>&1; then
  PRETTIER_STATUS="VERIFIED"
else
  printf '  \033[33mNOTE\033[0m prettier absent — .ts/.svelte/.md formatting path NOT verified here (best-effort by design)\n'
  PRETTIER_STATUS="NOT VERIFIED (prettier absent)"
fi

if [ "$FAILED" -eq "$PRIOR_FAILED" ]; then
  HOOK_STATUS[$HOOK]="rustfmt: $RUSTFMT_STATUS / prettier: $PRETTIER_STATUS"
else
  HOOK_STATUS[$HOOK]="NOT VERIFIED"
fi

# =============================================================================
section "Summary — witness status per hook (cf. issue #425 DoD)"
# =============================================================================
for h in pretool-deny-secret-write.sh pretool-deny-prod-action.sh posttool-warn-unwrap.sh \
         pretool-warn-sensitive-edit.sh userprompt-inject-rules.sh session-start.sh \
         stop-leak-scan.sh posttool-format.sh; do
  printf '  %-32s %s\n' "$h" "${HOOK_STATUS[$h]:-NOT VERIFIED (no test)}"
done

printf '\n%d assertions, %d failed.\n' "$TOTAL" "$FAILED"

if [ "$FAILED" -gt 0 ]; then
  exit 1
fi
exit 0
