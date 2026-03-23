#!/bin/bash
################################################################################
# Infrastructure Setup Orchestrator
# Usage: ./setup-infra.sh <env> <arch> [site]
# Example: ./setup-infra.sh production vps monosite
################################################################################

set -e

ENV="${1:?Usage: $0 <env> <arch> [site]}"
ARCH="${2:?Usage: $0 <env> <arch> [site]}"
SITE="${3:-monosite}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET="${INFRA_DIR}/${SITE}/${ARCH}/${ENV}"

echo "========================================="
echo "KoproGo Infrastructure Setup"
echo "Environment: ${ENV}"
echo "Architecture: ${ARCH}"
echo "Site: ${SITE}"
echo "Target: ${TARGET}"
echo "========================================="

# Step 1: Terraform
if [ -d "${TARGET}/terraform" ]; then
    echo ""
    echo ">>> Step 1: Terraform (provisioning infrastructure)..."
    cd "${TARGET}/terraform"
    terraform init
    terraform plan
    read -p "Apply Terraform changes? (y/N) " -n 1 -r
    echo
    [[ $REPLY =~ ^[Yy]$ ]] && terraform apply -auto-approve
fi

# Step 2: Ansible
if [ -d "${TARGET}/ansible" ] && [ -f "${TARGET}/ansible/inventory.ini" ]; then
    echo ""
    echo ">>> Step 2: Ansible (configuring servers)..."
    cd "${TARGET}/ansible"
    ansible-playbook -i inventory.ini playbook.yml
fi

# Step 3: Deploy
if [ "${ARCH}" = "vps" ]; then
    echo ""
    echo ">>> Step 3: Deploying via Docker Compose..."
    cd "${INFRA_DIR}/.."
    docker compose \
        -f "${INFRA_DIR}/_shared/docker-compose/docker-compose.base.yml" \
        -f "${TARGET}/docker-compose.override.yml" \
        --env-file "${TARGET}/.env" \
        up -d
elif [ "${ARCH}" = "k3s" ] || [ "${ARCH}" = "k8s" ]; then
    echo ""
    echo ">>> Step 3: ArgoCD will handle deployment via GitOps"
    echo "    Push to branch '${ENV}' to trigger deployment"
fi

echo ""
echo "========================================="
echo "Setup complete for ${ENV}/${ARCH}!"
echo "========================================="
