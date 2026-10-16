#!/bin/bash
################################################################################
# Make off-site backups immutable with S3 Object Lock (WORM)
# CyFun PR.IP-4 / RC.RP-1: a compromised server (or stolen S3 key) must not be
# able to delete or overwrite existing backups (ransomware scenario).
#
# Object Lock can only be enabled when the bucket is CREATED. If the current
# bucket was created without it, this script creates nothing and tells you so:
# create a new bucket, point s3_backup_bucket at it, then migrate.
#
# Retention mode:
#   COMPLIANCE (default) - nobody, including the account owner, can shorten
#                          retention or delete a locked version before expiry.
#   GOVERNANCE           - users with s3:BypassGovernanceRetention can.
#
# Run by a human with admin credentials on the bucket (Tier 1). Idempotent.
#
# Usage:
#   AWS_ACCESS_KEY_ID=... AWS_SECRET_ACCESS_KEY=... \
#   BACKUP_ENDPOINT=https://s3.<region>.io.cloud.ovh.net \
#   BACKUP_REGION=<region> \
#   [RETENTION_DAYS=30] [RETENTION_MODE=COMPLIANCE] \
#   ./backup-bucket-object-lock.sh [bucket] [--create]
################################################################################

set -euo pipefail

BUCKET="${1:-koprogo-backups}"
CREATE="${2:-}"
ENDPOINT="${BACKUP_ENDPOINT:?Set BACKUP_ENDPOINT (e.g. https://s3.gra.io.cloud.ovh.net)}"
REGION="${BACKUP_REGION:?Set BACKUP_REGION (e.g. gra)}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"
RETENTION_MODE="${RETENTION_MODE:-COMPLIANCE}"

case "$RETENTION_MODE" in
    COMPLIANCE|GOVERNANCE) ;;
    *) echo "RETENTION_MODE must be COMPLIANCE or GOVERNANCE" >&2; exit 1 ;;
esac

s3api() {
    aws s3api --endpoint-url "${ENDPOINT}" --region "${REGION}" "$@"
}

if ! s3api head-bucket --bucket "${BUCKET}" 2>/dev/null; then
    if [ "${CREATE}" != "--create" ]; then
        echo "Bucket ${BUCKET} does not exist. Re-run with --create to create it with Object Lock." >&2
        exit 1
    fi
    echo ">>> Creating ${BUCKET} with Object Lock enabled"
    s3api create-bucket --bucket "${BUCKET}" --object-lock-enabled-for-bucket
fi

if ! s3api get-object-lock-configuration --bucket "${BUCKET}" >/dev/null 2>&1; then
    echo "Bucket ${BUCKET} exists WITHOUT Object Lock, which cannot be enabled afterwards." >&2
    echo "Create a new bucket with:  $0 <new-bucket> --create" >&2
    echo "then set s3_backup_bucket to it in the production group_vars." >&2
    exit 1
fi

echo ">>> Setting default retention: ${RETENTION_MODE} ${RETENTION_DAYS} days"
s3api put-object-lock-configuration \
    --bucket "${BUCKET}" \
    --object-lock-configuration \
    "{\"ObjectLockEnabled\":\"Enabled\",\"Rule\":{\"DefaultRetention\":{\"Mode\":\"${RETENTION_MODE}\",\"Days\":${RETENTION_DAYS}}}}"

echo ">>> Enabling default server-side encryption"
s3api put-bucket-encryption \
    --bucket "${BUCKET}" \
    --server-side-encryption-configuration \
    '{"Rules":[{"ApplyServerSideEncryptionByDefault":{"SSEAlgorithm":"AES256"}}]}'

echo ">>> Current settings"
s3api get-object-lock-configuration --bucket "${BUCKET}"
s3api get-bucket-versioning --bucket "${BUCKET}"

echo
echo "Use a dedicated S3 user for the server with PutObject only (no Delete,"
echo "no PutBucket*/PutObjectRetention) so a stolen key cannot weaken the lock."
