#!/bin/bash
################################################################################
# Replicate production config to another environment
# Usage: ./replicate-env.sh <target_env> <arch>
# Example: ./replicate-env.sh staging vps
################################################################################

set -e

TARGET_ENV="${1:?Usage: $0 <target_env> <arch>}"
ARCH="${2:-vps}"
SITE="${3:-monosite}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SOURCE="${INFRA_DIR}/${SITE}/${ARCH}/production"
DEST="${INFRA_DIR}/${SITE}/${ARCH}/${TARGET_ENV}"

echo "Replicating production -> ${TARGET_ENV} (${SITE}/${ARCH})"
echo "Source: ${SOURCE}"
echo "Dest:   ${DEST}"

# Terraform
if [ -d "${SOURCE}/terraform" ]; then
    cp "${SOURCE}/terraform/terraform.tfvars" "${DEST}/terraform/terraform.tfvars" 2>/dev/null || true
    echo "  Copied terraform.tfvars - UPDATE instance_name and environment!"
fi

# Ansible
if [ -d "${SOURCE}/ansible/group_vars" ]; then
    cp -r "${SOURCE}/ansible/group_vars/"* "${DEST}/ansible/group_vars/" 2>/dev/null || true
    echo "  Copied ansible group_vars - UPDATE koprogo_branch, domains, secrets!"
fi

# Docker Compose override
if [ -f "${SOURCE}/docker-compose.override.yml" ]; then
    cp "${SOURCE}/docker-compose.override.yml" "${DEST}/docker-compose.override.yml" 2>/dev/null || true
    echo "  Copied docker-compose.override.yml - ADJUST resource limits!"
fi

# .env.example
if [ -f "${SOURCE}/.env.example" ]; then
    cp "${SOURCE}/.env.example" "${DEST}/.env.example" 2>/dev/null || true
    echo "  Copied .env.example - UPDATE all secrets and domains!"
fi

# Kustomize
if [ -d "${SOURCE}/kustomize/patches" ]; then
    cp -r "${SOURCE}/kustomize/patches/"* "${DEST}/kustomize/patches/" 2>/dev/null || true
    echo "  Copied kustomize patches - ADJUST replicas and resources!"
fi

echo ""
echo "Done! Remember to update environment-specific values in all copied files."
