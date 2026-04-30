#!/usr/bin/env bash
################################################################################
# install-prereqs.sh — install K8s prerequisites for KoproGo GitOps deployment.
#
# Usage:
#   ./install-prereqs.sh <cluster-type>
#
#   cluster-type: docker-desktop | k3s-self-hosted | k8s-managed
#
# Idempotent: re-running detects already-installed components and skips them.
# Called by gitops-bootstrap.sh — not usually invoked directly.
#
# Tier 1 per CRITICAL.md: this script applies kubectl/helm to a cluster.
# It must be run **explicitly by a human** — gitops-bootstrap.sh delegates
# to it but the bootstrap itself is also human-triggered.
################################################################################
set -euo pipefail

CLUSTER_TYPE="${1:-}"

if [ -z "$CLUSTER_TYPE" ]; then
    echo "ERROR: cluster-type argument required" >&2
    echo "Usage: $0 <docker-desktop|k3s-self-hosted|k8s-managed>" >&2
    exit 1
fi

# Verify required CLIs
for cmd in kubectl helm; do
    command -v "$cmd" >/dev/null 2>&1 || {
        echo "ERROR: $cmd not found in PATH" >&2
        exit 1
    }
done

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
log()  { echo -e "${GREEN}[prereqs]${NC} $1"; }
warn() { echo -e "${YELLOW}[prereqs]${NC} $1"; }

# ----------------------------------------------------------------------
# 1. Ingress controller (per cluster profile)
# ----------------------------------------------------------------------
case "$CLUSTER_TYPE" in
    docker-desktop|k8s-managed)
        if kubectl get ns ingress-nginx >/dev/null 2>&1; then
            log "ingress-nginx already installed"
        else
            log "Installing ingress-nginx..."
            helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx 2>/dev/null || true
            helm repo update
            helm upgrade --install ingress-nginx ingress-nginx/ingress-nginx \
                --namespace ingress-nginx --create-namespace \
                --set controller.service.type=LoadBalancer \
                --wait --timeout 5m
        fi
        ;;
    k3s-self-hosted)
        log "K3s ships with traefik ingress by default — skipping ingress controller install"
        ;;
esac

# ----------------------------------------------------------------------
# 2. cert-manager (only when TLS enabled in profile)
# ----------------------------------------------------------------------
case "$CLUSTER_TYPE" in
    k3s-self-hosted|k8s-managed)
        if kubectl get ns cert-manager >/dev/null 2>&1; then
            log "cert-manager already installed"
        else
            log "Installing cert-manager..."
            helm repo add jetstack https://charts.jetstack.io 2>/dev/null || true
            helm repo update
            helm upgrade --install cert-manager jetstack/cert-manager \
                --namespace cert-manager --create-namespace \
                --set installCRDs=true \
                --wait --timeout 5m
            warn "Reminder: create a ClusterIssuer 'letsencrypt-prod' before pushing app — see docs/letsencrypt-issuer.yaml"
        fi
        ;;
    docker-desktop)
        log "TLS off in docker-desktop profile — skipping cert-manager"
        ;;
esac

# ----------------------------------------------------------------------
# 3. Secrets backend (per cluster profile)
# ----------------------------------------------------------------------
case "$CLUSTER_TYPE" in
    k3s-self-hosted)
        if kubectl get ns sealed-secrets >/dev/null 2>&1; then
            log "sealed-secrets already installed"
        else
            log "Installing sealed-secrets-controller..."
            helm repo add sealed-secrets https://bitnami-labs.github.io/sealed-secrets 2>/dev/null || true
            helm repo update
            helm upgrade --install sealed-secrets sealed-secrets/sealed-secrets \
                --namespace sealed-secrets --create-namespace \
                --wait --timeout 5m
        fi
        ;;
    k8s-managed)
        if kubectl get ns external-secrets >/dev/null 2>&1; then
            log "external-secrets already installed"
        else
            log "Installing external-secrets-operator..."
            helm repo add external-secrets https://charts.external-secrets.io 2>/dev/null || true
            helm repo update
            helm upgrade --install external-secrets external-secrets/external-secrets \
                --namespace external-secrets --create-namespace \
                --wait --timeout 5m
            warn "Reminder: configure SecretStore (Vault auth) before pushing app — see docs/vault-secretstore.yaml"
        fi
        ;;
    docker-desktop)
        log "secretsBackend=raw in docker-desktop profile — skipping secrets controller (Helm values cleartext)"
        ;;
esac

# ----------------------------------------------------------------------
# 4. ArgoCD itself
# ----------------------------------------------------------------------
# Always apply (server-side apply is idempotent). Previous "skip if ns exists"
# logic was brittle: a partial first install (e.g. a single CRD failure) would
# leave the namespace present but the install incomplete — and every subsequent
# run would skip the repair, requiring manual intervention.
log "Ensuring ArgoCD is installed (idempotent server-side apply)..."
kubectl create namespace argocd --dry-run=client -o yaml | kubectl apply -f -
# --server-side required: ArgoCD CRDs (notably applicationsets.argoproj.io) exceed
# the 262144-byte limit of the client-side last-applied-configuration annotation.
kubectl apply -n argocd --server-side --force-conflicts \
    -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml
kubectl wait --for=condition=available --timeout=300s deployment/argocd-server -n argocd

log "Prerequisites OK for cluster-type=$CLUSTER_TYPE"
