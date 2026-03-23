#!/bin/bash
# Trigger PostgreSQL failover using CrunchyData PGO
# Usage: ./failover-database.sh <namespace>
set -e
NS="${1:-koprogo-production}"
CLUSTER="koprogo-pg"
echo "Triggering PostgreSQL failover in namespace $NS..."
echo "Current primary:"
kubectl get pods -n "$NS" -l postgres-operator.crunchydata.com/role=master -o name
echo ""
echo "Annotating for failover..."
kubectl annotate -n "$NS" postgrescluster "$CLUSTER" \
  postgres-operator.crunchydata.com/trigger-failover="$(date +%s)" --overwrite
echo "Failover triggered. Monitor: kubectl get pods -n $NS -w"
