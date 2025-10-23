#!/bin/bash
# KoproGo Infrastructure Destruction Script
# Usage: ./destroy.sh [dev|staging|prod]

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TERRAFORM_DIR="$PROJECT_ROOT/terraform"

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

destroy_infrastructure() {
    local env=$1
    local tf_dir="$TERRAFORM_DIR/environments/$env"

    log_warn "⚠️  WARNING: This will DESTROY all infrastructure for environment: $env"
    log_warn "This action cannot be undone!"
    echo ""

    read -p "Type the environment name to confirm ($env): " confirm

    if [ "$confirm" != "$env" ]; then
        log_error "Confirmation failed. Aborting."
        exit 1
    fi

    cd "$tf_dir"

    log_info "Initializing Terraform..."
    terraform init

    log_info "Planning destruction..."
    terraform plan -destroy

    echo ""
    read -p "Proceed with destruction? (yes/no): " final_confirm

    if [ "$final_confirm" != "yes" ]; then
        log_warn "Destruction cancelled"
        exit 0
    fi

    log_info "Destroying infrastructure..."
    terraform destroy -auto-approve

    log_info "Destruction complete ✓"
}

main() {
    local env="${1:-}"

    if [ -z "$env" ]; then
        log_error "Environment not specified"
        echo "Usage: $0 [dev|staging|prod]"
        exit 1
    fi

    if [[ ! "$env" =~ ^(dev|staging|prod)$ ]]; then
        log_error "Invalid environment: $env"
        exit 1
    fi

    destroy_infrastructure "$env"
}

main "$@"
