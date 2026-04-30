#!/usr/bin/env bash
################################################################################
# gitops-bootstrap.sh — install ArgoCD + cluster prereqs + KoproGo ApplicationSets.
#
# Usage:
#   ./gitops-bootstrap.sh                          # auto-detect cluster type
#   ./gitops-bootstrap.sh docker-desktop           # explicit
#   ./gitops-bootstrap.sh k3s-self-hosted
#   ./gitops-bootstrap.sh k8s-managed
#
#   KOPROGO_DOMAIN=ma-copro.be ./gitops-bootstrap.sh k3s-self-hosted
#
# Tier 1 per CRITICAL.md: applies kubectl/helm against a real cluster.
# Run EXPLICITLY BY A HUMAN. Agent IA must NOT invoke this autonomously.
#
# Idempotent: safe to re-run — components already-installed are skipped.
################################################################################
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
log()   { echo -e "${GREEN}[gitops-bootstrap]${NC} $1"; }
warn()  { echo -e "${YELLOW}[gitops-bootstrap]${NC} $1"; }
fatal() { echo -e "${RED}[gitops-bootstrap]${NC} $1" >&2; exit 1; }

# ---------- Prerequisites ----------
for cmd in kubectl helm envsubst; do
    command -v "$cmd" >/dev/null 2>&1 || fatal "$cmd not found in PATH"
done

# ---------- Determine cluster type ----------
CLUSTER_TYPE="${1:-auto}"
KOPROGO_DOMAIN="${KOPROGO_DOMAIN:-}"

if [ "$CLUSTER_TYPE" = "auto" ]; then
    CONTEXT=$(kubectl config current-context 2>/dev/null || echo "unknown")
    case "$CONTEXT" in
        docker-desktop)            CLUSTER_TYPE="docker-desktop" ;;
        *k3s*|k3d-*|*k3d*)         CLUSTER_TYPE="k3s-self-hosted" ;;
        *)                         CLUSTER_TYPE="k8s-managed" ;;
    esac
    log "Auto-detected cluster type: $CLUSTER_TYPE (context=$CONTEXT)"
fi

# Verify profile exists
PROFILE_FILE="$REPO_DIR/infrastructure/_shared/cluster-profiles/${CLUSTER_TYPE}.yaml"
[ -f "$PROFILE_FILE" ] || fatal "cluster profile not found: $PROFILE_FILE
Supported: docker-desktop, k3s-self-hosted, k8s-managed"

log "Using cluster profile: $PROFILE_FILE"

# ---------- Confirm with user (Tier 1 safeguard) ----------
echo ""
echo "============================================================"
echo "  KoproGo GitOps Bootstrap"
echo "============================================================"
echo "  Cluster context : $(kubectl config current-context 2>/dev/null || echo unknown)"
echo "  Cluster type    : $CLUSTER_TYPE"
echo "  KOPROGO_DOMAIN  : ${KOPROGO_DOMAIN:-<not set, profile default>}"
echo "  Repo            : $REPO_DIR"
echo "============================================================"
echo ""
echo "About to apply kubectl + helm changes to this cluster."
read -r -p "Type 'yes' to proceed: " confirm
[ "$confirm" = "yes" ] || fatal "Aborted by user"

# ---------- Run prereqs installer ----------
log "Running prereqs installer..."
"$SCRIPT_DIR/install-prereqs.sh" "$CLUSTER_TYPE"

# ---------- ConfigMap argocd-cluster-config ----------
log "Creating/updating argocd-cluster-config ConfigMap..."
kubectl create configmap argocd-cluster-config \
    --from-literal=clusterType="$CLUSTER_TYPE" \
    --from-literal=domain="$KOPROGO_DOMAIN" \
    -n argocd --dry-run=client -o yaml | kubectl apply -f -

# ---------- Apply AppProject ----------
log "Applying AppProject..."
kubectl apply -f "$REPO_DIR/infrastructure/_shared/argocd/appproject.yaml"

# ---------- Render + apply ApplicationSet (templatized) ----------
APPSET_TPL="$REPO_DIR/infrastructure/_shared/argocd/applicationset.yaml.tpl"
APPSET_LEGACY="$REPO_DIR/infrastructure/_shared/argocd/applicationset.yaml"

if [ -f "$APPSET_TPL" ]; then
    log "Rendering ApplicationSet template (CLUSTER_TYPE=$CLUSTER_TYPE)..."
    export CLUSTER_TYPE
    # Only substitute ${CLUSTER_TYPE} — preserve $values (Helm multi-source ref)
    envsubst '${CLUSTER_TYPE}' < "$APPSET_TPL" | kubectl apply -f -
    # Remove legacy AppSet (without cluster-profile stacking) to avoid double sync
    if kubectl -n argocd get applicationset koprogo-app >/dev/null 2>&1; then
        # Check if our templated one is named differently or same — if same, our apply overrides
        warn "Legacy ApplicationSet still present. The new template overrides it on apply."
    fi
elif [ -f "$APPSET_LEGACY" ]; then
    warn "applicationset.yaml.tpl not found; falling back to legacy applicationset.yaml (no cluster-profile stacking)"
    kubectl apply -f "$APPSET_LEGACY"
else
    fatal "Neither applicationset.yaml.tpl nor applicationset.yaml found in _shared/argocd/"
fi

# ---------- Output ----------
ADMIN_PASS=$(kubectl -n argocd get secret argocd-initial-admin-secret \
    -o jsonpath='{.data.password}' 2>/dev/null | base64 -d 2>/dev/null || echo "(already rotated)")

echo ""
echo "============================================================"
echo "  ✅ Bootstrap complete"
echo "============================================================"
echo "  ArgoCD UI       : kubectl port-forward svc/argocd-server -n argocd 8080:443"
echo "                    → https://localhost:8080"
echo "  ArgoCD admin    : admin / $ADMIN_PASS"
echo "  Cluster type    : $CLUSTER_TYPE"
echo "  Applications    : kubectl get applications -n argocd"
echo ""

if [ "$CLUSTER_TYPE" = "docker-desktop" ]; then
    echo "  ⚠️  Add hosts file entries (sandbox uses 127.0.0.1 for all envs):"
    echo "     cat $REPO_DIR/infrastructure/_shared/scripts/hosts-local-gitops.txt"
    echo "     # then append to /etc/hosts (Linux/Mac) or hosts (Windows admin)"
fi

echo "============================================================"
echo "  Next: monitor sync via 'make gitops-status'"
echo "  Docs: $REPO_DIR/infrastructure/GITOPS.md"
echo "============================================================"
