#!/bin/bash
################################################################################
# Harden the Terraform state bucket (CyFun PR.DS-1, PR.IP-4).
#
# - Versioning: every state write keeps the previous version, so a corrupted
#   or maliciously modified state can be rolled back.
# - Default encryption (SSE-OMK on OVHcloud): objects are encrypted at rest
#   even if a client forgets the encryption header.
#
# Run by a human with credentials for the bucket (Tier 1 - not run by agents
# or CI). Idempotent.
#
# Usage:
#   AWS_ACCESS_KEY_ID=... AWS_SECRET_ACCESS_KEY=... \
#   TFSTATE_ENDPOINT=https://s3.<region>.io.cloud.ovh.net \
#   TFSTATE_REGION=<region> \
#   ./tfstate-bucket-hardening.sh [bucket]
################################################################################

set -euo pipefail

BUCKET="${1:-koprogo-tfstate}"
ENDPOINT="${TFSTATE_ENDPOINT:?Set TFSTATE_ENDPOINT (e.g. https://s3.gra.io.cloud.ovh.net)}"
REGION="${TFSTATE_REGION:?Set TFSTATE_REGION (e.g. gra)}"

s3api() {
    aws s3api --endpoint-url "${ENDPOINT}" --region "${REGION}" "$@"
}

echo ">>> Enabling versioning on ${BUCKET}"
s3api put-bucket-versioning \
    --bucket "${BUCKET}" \
    --versioning-configuration Status=Enabled

echo ">>> Enabling default server-side encryption on ${BUCKET}"
s3api put-bucket-encryption \
    --bucket "${BUCKET}" \
    --server-side-encryption-configuration \
    '{"Rules":[{"ApplyServerSideEncryptionByDefault":{"SSEAlgorithm":"AES256"}}]}'

echo ">>> Current settings"
s3api get-bucket-versioning --bucket "${BUCKET}"
s3api get-bucket-encryption --bucket "${BUCKET}"

echo
echo "Existing state objects are NOT re-encrypted retroactively."
echo "Run a 'terraform apply' (or re-upload the state) per environment to rewrite them."
