#!/usr/bin/env bash
################################################################################
# gitops-teardown.sh — remove KoproGo ApplicationSets + ArgoCD from cluster.
#
# Does NOT delete app workloads (koprogo-{env} namespaces) — only the GitOps
# control plane. Re-running gitops-bootstrap.sh restores it without losing data
# (ArgoCD will re-sync the existing namespaces).
#
# Usage: ./gitops-teardown.sh
#
# Tier 1 per CRITICAL.md: applies kubectl mutations. Human-triggered only.
################################################################################
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
log()  { echo -e "${GREEN}[teardown]${NC} $1"; }
warn() { echo -e "${YELLOW}[teardown]${NC} $1"; }

echo ""
echo "============================================================"
echo "  KoproGo GitOps Teardown"
echo "============================================================"
echo "  Context : $(kubectl config current-context 2>/dev/null || echo unknown)"
echo "  Removes : ApplicationSets + AppProject + argocd namespace"
echo "  Keeps   : koprogo-{env} namespaces and their workloads"
echo "============================================================"
echo ""
read -r -p "Type 'yes' to proceed: " confirm
[ "$confirm" = "yes" ] || { warn "Aborted"; exit 1; }

# Remove templated AppSet (if present)
log "Removing ApplicationSets..."
kubectl -n argocd delete applicationset koprogo-infra koprogo-app --ignore-not-found

log "Removing AppProject..."
kubectl -n argocd delete appproject koprogo --ignore-not-found

log "Removing ArgoCD namespace (this deletes ArgoCD itself)..."
kubectl delete namespace argocd --ignore-not-found

log "Removing argocd-cluster-config ConfigMap (if leftover)..."
kubectl delete configmap argocd-cluster-config -n argocd --ignore-not-found 2>/dev/null || true

echo ""
echo "✅ Teardown complete. Workloads in koprogo-* namespaces are preserved."
echo "   Re-run ./gitops-bootstrap.sh to restore the GitOps control plane."
