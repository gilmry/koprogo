#!/bin/bash
# Full PRA (Plan de Reprise d'Activite) runbook
# Usage: ./full-restore.sh <environment>
set -e
ENV="${1:?Usage: $0 <environment>}"
NS="koprogo-${ENV}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "============================================"
echo "KoproGo PRA - Full Restore"
echo "Environment: ${ENV}"
echo "Namespace: ${NS}"
echo "============================================"

# Step 1: Check cluster health
echo ">>> Step 1: Checking cluster health..."
kubectl cluster-info
kubectl get nodes

# Step 2: Find latest backup
echo ">>> Step 2: Finding latest backup for ${NS}..."
LATEST_BACKUP=$(velero backup get -o json | jq -r ".items[] | select(.spec.includedNamespaces[] == \"${NS}\") | .metadata.name" | tail -1)
if [ -z "$LATEST_BACKUP" ]; then
  echo "ERROR: No backup found for namespace ${NS}"
  exit 1
fi
echo "Latest backup: $LATEST_BACKUP"

# Step 3: Apply infrastructure (kustomize)
echo ">>> Step 3: Applying infrastructure (Kustomize)..."
ARCH="k3s"
[ -d "$INFRA_DIR/multisite/k8s/${ENV}" ] && ARCH="k8s" && SITE="multisite" || SITE="monosite"
kustomize build "$INFRA_DIR/${SITE}/${ARCH}/${ENV}/kustomize" | kubectl apply -f -

# Step 4: Restore from backup
echo ">>> Step 4: Restoring from Velero backup..."
bash "$SCRIPT_DIR/restore-namespace.sh" "$LATEST_BACKUP" "$NS"

# Step 5: Wait for pods
echo ">>> Step 5: Waiting for pods to be ready..."
kubectl wait --for=condition=ready pod -l app.kubernetes.io/part-of=koprogo -n "$NS" --timeout=300s

# Step 6: Health check
echo ">>> Step 6: Running health checks..."
kubectl get pods -n "$NS"

echo ""
echo "============================================"
echo "PRA Complete for ${ENV}!"
echo "============================================"
