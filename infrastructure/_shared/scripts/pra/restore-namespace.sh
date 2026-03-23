#!/bin/bash
# Restore a Kubernetes namespace from Velero backup
# Usage: ./restore-namespace.sh <backup-name> <namespace>
set -e
BACKUP="${1:?Usage: $0 <backup-name> <namespace>}"
NS="${2:-koprogo-production}"
echo "Restoring namespace $NS from backup $BACKUP..."
velero restore create "restore-${NS}-$(date +%Y%m%d%H%M)" \
  --from-backup "$BACKUP" \
  --include-namespaces "$NS" \
  --wait
echo "Restore complete. Check: kubectl get pods -n $NS"
