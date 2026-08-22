#!/usr/bin/env bash
# check-no-local-helpers.sh
#
# Story Tx.2 / ADR-0013 lint gate.
#
# Enforces that no test under `frontend/tests/e2e/refonte-ux/` re-implements
# a local UI-login flow. Every authenticated scenario MUST go through the
# shared helpers in `frontend/tests/e2e/helpers/{auth,building,magic-link}.ts`.
#
# Detected smells (any of these in a refonte-ux/ spec file = fail):
#   - `page.goto('/login')` (or "/login")
#   - `page.fill(...password...)` / `page.fill(...email...)` followed shortly
#     by `page.click(...login|submit|sign-in...)`
#   - direct `localStorage.setItem('koprogo_token'|'koprogo_user', ...)`
#     outside `helpers/`
#
# Exit codes:
#   0 — clean, no local helper detected
#   1 — at least one violation (printed with file + line + matched snippet)
#   2 — usage error (e.g. ripgrep / grep missing)
#
# Usage:
#   ./scripts/check-no-local-helpers.sh
#   ./scripts/check-no-local-helpers.sh --path frontend/tests/e2e/refonte-ux

set -euo pipefail

ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
TARGET="${ROOT}/frontend/tests/e2e/refonte-ux"

# Parse --path override (used by CI to point at a custom directory).
while [[ $# -gt 0 ]]; do
  case "$1" in
    --path)
      TARGET="$2"
      shift 2
      ;;
    -h|--help)
      sed -n '2,25p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

# The directory is allowed to not exist yet (slices 1-5 not started).
if [[ ! -d "${TARGET}" ]]; then
  echo "check-no-local-helpers: ${TARGET} does not exist — nothing to scan (OK)."
  exit 0
fi

# Pick the search tool — prefer ripgrep if available (faster, respects
# .gitignore), fall back to GNU grep with extended regex (-E) since our
# patterns use alternation `(a|b|c)`.
if command -v rg >/dev/null 2>&1; then
  SEARCH() {
    local pattern="$1"
    rg --no-heading --line-number --color=never \
       --glob "*.ts" --glob "*.tsx" \
       -e "${pattern}" "${TARGET}" || true
  }
elif command -v grep >/dev/null 2>&1; then
  SEARCH() {
    local pattern="$1"
    grep -REHn --color=never \
         --include="*.ts" --include="*.tsx" \
         -e "${pattern}" "${TARGET}" || true
  }
else
  echo "check-no-local-helpers: neither ripgrep (rg) nor grep is available." >&2
  exit 2
fi

violations=0
tmp_report="$(mktemp)"
trap 'rm -f "${tmp_report}"' EXIT

scan() {
  local label="$1"
  local pattern="$2"
  local matches
  matches="$(SEARCH "${pattern}" 2>/dev/null || true)"
  if [[ -n "${matches}" ]]; then
    echo "" >>"${tmp_report}"
    echo "FAIL — ${label}" >>"${tmp_report}"
    echo "${matches}" >>"${tmp_report}"
    violations=$((violations + 1))
  fi
}

# 1. Direct navigation to /login from a refonte-ux spec.
scan "page.goto('/login') — use loginAs* helper instead" \
     "page\.goto\\(\\s*['\"]\\/login['\"]"

# 2. UI password entry (any spec that types a password in the browser).
scan "page.fill on a password-ish field — use loginAs* helper instead" \
     "page\.fill\\([^)]*(password|mot.?de.?passe|wachtwoord|passwort)"

# 3. UI email entry followed (within 5 lines) by a click on a login/submit
#    button. Multi-line detection: search for the click pattern; manual
#    review will pair it with the nearby email fill.
scan "page.click on a login/submit/sign-in button — verify it's not a UI login" \
     "page\.click\\([^)]*(login|signin|sign-in|connexion|aanmelden|anmelden|submit-login|btn-login)"

# 4. Direct localStorage tampering with auth keys (must live in helpers/).
scan "direct localStorage write on auth keys — move to helpers/" \
     "localStorage\\.setItem\\(\\s*['\"](koprogo_token|koprogo_user|koprogo_refresh)"

if [[ ${violations} -gt 0 ]]; then
  echo "============================================================"
  echo "check-no-local-helpers: ${violations} violation pattern(s) found"
  echo "============================================================"
  cat "${tmp_report}"
  echo ""
  echo "Refonte-UX tests MUST import from frontend/tests/e2e/helpers/."
  echo "See: docs/maury/refonte-ux-multi-role-acp/stories.md §8 Story Tx.2"
  exit 1
fi

echo "check-no-local-helpers: OK — no local UI-login helper detected under ${TARGET}"
exit 0
