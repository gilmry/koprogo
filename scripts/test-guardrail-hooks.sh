#!/usr/bin/env bash
#
# Témoin exécutable du modèle Tier 1 / Tier 2 (Issue #429, règle CRITICAL.md #11).
#
# La story #429 pose quatre classes de preuve pour le modèle d'autorisation :
#   @happy    — un agent propose sans exécuter, rien ne le bloque en lecture.
#   @negative — une mutation de production part et EST refusée (règle 2).
#   @edge     — une forme ambiguë/obfusquée choisit Tier 1 (refus), jamais Tier 2.
#   @security — un secret en clair est refusé (règle 1) et les journaux
#               d'activité déjà commités n'en contiennent aucun.
#
# Avant ce script, `make claude-check` vérifiait que les hooks existent et sont
# exécutables, jamais qu'ils bloquent effectivement ce qu'ils prétendent
# bloquer. Un hook qui ne matche plus rien (regex cassée par un refactor)
# passerait `claude-check` haut la main. Ce script est le "témoin" que la DoD
# #429 réclame : "mutations de production refusées avec témoin".
#
# Note : les fixtures @security qui doivent ressembler à un secret (clé AWS,
# PAT GitHub, bloc PEM) sont assemblées à l'exécution à partir de deux
# fragments courts, jamais écrites en clair et contiguës dans ce fichier —
# sinon pretool-deny-secret-write.sh (à l'édition) et stop-leak-scan.sh (à la
# fin du tour) bloqueraient l'écriture de CE script, pour la même raison qui
# fait qu'ils doivent bloquer le cas réel qu'il teste.
#
# Usage : scripts/test-guardrail-hooks.sh
# Exit 0 si toutes les assertions passent, 1 sinon (liste des échecs sur stderr).

set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DENY_PROD="$ROOT/.claude/hooks/pretool-deny-prod-action.sh"
DENY_SECRET="$ROOT/.claude/hooks/pretool-deny-secret-write.sh"

failures=0
total=0

# json_escape: échappe guillemets et antislashs pour un payload JSON minimal.
# Les fixtures sont des littéraux contrôlés par ce script (pas d'entrée
# externe) : un échappement simple suffit, pas besoin de jq -Rs.
json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

# assert_bash_command <class> <description> <command> <expected_exit>
# Simule un appel PreToolUse Bash vers pretool-deny-prod-action.sh.
assert_bash_command() {
  local class="$1" desc="$2" command="$3" expected="$4"
  total=$((total + 1))
  local payload actual
  payload="{\"tool_name\":\"Bash\",\"tool_input\":{\"command\":\"$(json_escape "$command")\"}}"
  actual="$(printf '%s' "$payload" | "$DENY_PROD" >/tmp/guardrail-test-out.$$ 2>&1; echo $?)"
  if [ "$actual" != "$expected" ]; then
    failures=$((failures + 1))
    echo "❌ [$class] $desc" >&2
    echo "   command : $command" >&2
    echo "   attendu : exit $expected — obtenu : exit $actual" >&2
  fi
  rm -f /tmp/guardrail-test-out.$$
}

# assert_write <class> <description> <file_path> <content> <expected_exit>
# Simule un appel PreToolUse Write vers pretool-deny-secret-write.sh.
assert_write() {
  local class="$1" desc="$2" file_path="$3" content="$4" expected="$5"
  total=$((total + 1))
  local payload actual
  payload="{\"tool_name\":\"Write\",\"tool_input\":{\"file_path\":\"$(json_escape "$file_path")\",\"content\":\"$(json_escape "$content")\"}}"
  actual="$(printf '%s' "$payload" | "$DENY_SECRET" >/tmp/guardrail-test-out.$$ 2>&1; echo $?)"
  if [ "$actual" != "$expected" ]; then
    failures=$((failures + 1))
    echo "❌ [$class] $desc" >&2
    echo "   file    : $file_path" >&2
    echo "   attendu : exit $expected — obtenu : exit $actual" >&2
  fi
  rm -f /tmp/guardrail-test-out.$$
}

echo "🧪 Témoin Tier 1/Tier 2 — pretool-deny-prod-action.sh + pretool-deny-secret-write.sh"

# ── @happy — lecture/diagnostic/proposal jamais bloqué ──────────────────────
assert_bash_command "@happy" "terraform plan est un diagnostic, pas une mutation" \
  "terraform plan -out=tfplan" 0
assert_bash_command "@happy" "kubectl get est une lecture" \
  "kubectl get pods -n koprogo-prod" 0
assert_bash_command "@happy" "helm template est un rendu local, pas un déploiement" \
  "helm template ./chart" 0
assert_bash_command "@happy" "argocd app get est une lecture" \
  "argocd app get koprogo-prod" 0
assert_bash_command "@happy" "git push normal (non force) reste autorisé au niveau hook" \
  "git push origin story/429" 0
assert_write "@happy" "écrire du code de domaine ordinaire n'est pas un secret" \
  "backend/src/domain/entities/example.rs" "pub struct Example { pub id: i64 }" 0

# ── @negative — une mutation de production part, et EST refusée ─────────────
assert_bash_command "@negative" "terraform apply refusé" \
  "terraform apply -auto-approve" 2
assert_bash_command "@negative" "helm upgrade refusé" \
  "helm upgrade koprogo ./chart --namespace koprogo-prod" 2
assert_bash_command "@negative" "kubectl apply refusé" \
  "kubectl apply -f deployment.yaml" 2
assert_bash_command "@negative" "kubectl exec refusé" \
  "kubectl exec -it backend-0 -- sh" 2
assert_bash_command "@negative" "argocd app sync refusé" \
  "argocd app sync koprogo-prod" 2
assert_bash_command "@negative" "velero restore refusé" \
  "velero restore create --from-backup nightly" 2
assert_bash_command "@negative" "git push --force refusé" \
  "git push --force origin main" 2
assert_bash_command "@negative" "git commit --no-verify refusé" \
  "git commit --no-verify -m x" 2
assert_bash_command "@negative" "gh release create refusé" \
  "gh release create v1.0.0" 2
assert_bash_command "@negative" "npm publish refusé" \
  "npm publish" 2

# Fragments assemblés à l'exécution — voir note d'en-tête.
_aws_prefix="AKIA"
_aws_suffix="ABCDEFGHIJKLMNOP"
_gh_prefix="ghp_"
_gh_suffix="0123456789abcdefghijklmnopqrstuvwxyz01"
assert_write "@negative" "clé AWS en clair refusée" \
  "backend/src/config.rs" "${_aws_prefix}${_aws_suffix}" 2
assert_write "@negative" "PAT GitHub en clair refusé" \
  "docs/notes.md" "token: ${_gh_prefix}${_gh_suffix}" 2
assert_write "@negative" "écriture directe d'un .env refusée" \
  "infrastructure/.env.production" "DATABASE_URL=postgres://real:credential@host/db" 2

# ── @edge — forme ambiguë/obfusquée : le défaut reste Tier 1 (refus) ────────
assert_bash_command "@edge" "préfixe de variable d'env avant terraform apply reste bloqué" \
  "TF_VAR_env=prod terraform apply -auto-approve" 2
assert_bash_command "@edge" "espaces multiples n'échappent pas le pattern kubectl apply" \
  "kubectl   apply   -f deployment.yaml" 2
assert_bash_command "@edge" "commande chaînée après && reste détectée" \
  "echo start && kubectl delete pod backend-0" 2
assert_bash_command "@edge" "git push -f (forme courte) reste bloqué comme --force" \
  "git push -f origin main" 2

# ── @security — secret refusé, ET journaux déjà commités sans fuite ────────
assert_write "@security" "kubeconfig refusé (identifiants admin cluster)" \
  "infrastructure/kubeconfig" "apiVersion: v1" 2
assert_write "@security" "tfstate refusé (peut contenir des secrets en clair)" \
  "infrastructure/terraform.tfstate" "{}" 2
assert_write "@security" "placeholder non-secret n'est PAS un faux positif" \
  "infrastructure/_shared/helm/values.yaml.example" "password: \"changeme\"" 0

echo
echo "🔎 [@security] docs/agent-activity/*.md commités ne contiennent aucun secret"
total=$((total + 1))
activity_dir="$ROOT/docs/agent-activity"
if [ -d "$activity_dir" ]; then
  if command -v gitleaks >/dev/null 2>&1; then
    if ! gitleaks detect --no-banner --redact --no-git \
      --config "$ROOT/.gitleaks.toml" --source "$activity_dir" >/tmp/guardrail-activity-scan.$$ 2>&1; then
      failures=$((failures + 1))
      echo "❌ [@security] gitleaks a détecté un secret dans docs/agent-activity/" >&2
      cat /tmp/guardrail-activity-scan.$$ >&2
    fi
    rm -f /tmp/guardrail-activity-scan.$$
  else
    if grep -rlE 'AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{36}|BEGIN (RSA |OPENSSH |EC )?PRIVATE KEY' "$activity_dir" 2>/dev/null; then
      failures=$((failures + 1))
      echo "❌ [@security] pattern secret détecté dans docs/agent-activity/ (fallback grep, gitleaks absent)" >&2
    fi
  fi
else
  echo "  (docs/agent-activity/ absent — rien à scanner)"
fi

echo
echo "── Résumé : $((total - failures))/$total assertions passées ──"
if [ "$failures" -gt 0 ]; then
  echo "$failures échec(s). Le modèle Tier 1/Tier 2 n'est pas vérifié — voir #429." >&2
  exit 1
fi
echo "✅ Modèle Tier 1/Tier 2 vérifié : mutations refusées, secrets refusés, journaux propres."
