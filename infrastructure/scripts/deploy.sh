#!/bin/bash
# KoproGo Infrastructure Deployment Script
# Usage: ./deploy.sh [dev|staging|prod]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TERRAFORM_DIR="$PROJECT_ROOT/terraform"
ANSIBLE_DIR="$PROJECT_ROOT/ansible"
HELM_DIR="$PROJECT_ROOT/helm"

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    local missing_tools=()

    command -v terraform >/dev/null 2>&1 || missing_tools+=("terraform")
    command -v ansible >/dev/null 2>&1 || missing_tools+=("ansible")
    command -v helm >/dev/null 2>&1 || missing_tools+=("helm")
    command -v kubectl >/dev/null 2>&1 || missing_tools+=("kubectl")

    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_error "Please install missing tools and try again"
        exit 1
    fi

    log_info "All prerequisites met ✓"
}

check_environment() {
    local env=$1

    if [[ ! "$env" =~ ^(dev|staging|prod)$ ]]; then
        log_error "Invalid environment: $env"
        log_error "Usage: $0 [dev|staging|prod]"
        exit 1
    fi

    log_info "Deploying to environment: $env"
}

deploy_terraform() {
    local env=$1
    local tf_dir="$TERRAFORM_DIR/environments/$env"

    log_info "Step 1/5: Deploying infrastructure with Terraform..."

    if [ ! -d "$tf_dir" ]; then
        log_error "Terraform directory not found: $tf_dir"
        exit 1
    fi

    cd "$tf_dir"

    # Initialize Terraform
    log_info "Initializing Terraform..."
    if [ -f "backend.hcl" ]; then
        terraform init -backend-config=backend.hcl
    else
        log_warn "backend.hcl not found, using default backend configuration"
        terraform init
    fi

    # Validate configuration
    log_info "Validating Terraform configuration..."
    terraform validate

    # Plan
    log_info "Creating Terraform plan..."
    terraform plan -out=tfplan

    # Apply
    read -p "Apply Terraform plan? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        log_warn "Terraform apply cancelled"
        rm -f tfplan
        exit 0
    fi

    log_info "Applying Terraform..."
    terraform apply tfplan
    rm -f tfplan

    # Generate inventory
    log_info "Generating Ansible inventory..."
    terraform output -raw ansible_inventory > "$ANSIBLE_DIR/inventory/$env.yml"

    log_info "Terraform deployment complete ✓"
}

deploy_ansible() {
    local env=$1

    log_info "Step 2/5: Configuring infrastructure with Ansible..."

    cd "$ANSIBLE_DIR"

    # Wait for instances to be reachable
    log_info "Waiting for instances to be reachable..."
    sleep 30

    # Test connectivity
    log_info "Testing connectivity to all nodes..."
    ansible all -i "inventory/$env.yml" -m ping

    # Run bootstrap playbook
    log_info "Running bootstrap playbook..."
    ansible-playbook -i "inventory/$env.yml" playbooks/01-bootstrap.yml

    # Run security playbook
    log_info "Running security hardening..."
    ansible-playbook -i "inventory/$env.yml" playbooks/02-security.yml

    # Deploy K3s
    log_info "Deploying K3s cluster..."
    ansible-playbook -i "inventory/$env.yml" playbooks/site.yml

    log_info "Ansible configuration complete ✓"
}

configure_kubectl() {
    local env=$1

    log_info "Step 3/5: Configuring kubectl..."

    export KUBECONFIG="$HOME/.kube/koprogo-$env"

    if [ ! -f "$KUBECONFIG" ]; then
        log_error "Kubeconfig not found: $KUBECONFIG"
        log_error "Ansible may not have completed successfully"
        exit 1
    fi

    log_info "Testing cluster connectivity..."
    kubectl cluster-info
    kubectl get nodes

    log_info "Kubectl configured ✓"
}

deploy_helm() {
    local env=$1

    log_info "Step 4/5: Deploying applications with Helm..."

    export KUBECONFIG="$HOME/.kube/koprogo-$env"

    cd "$HELM_DIR"

    # Add Helm repositories
    log_info "Adding Helm repositories..."
    helm repo add bitnami https://charts.bitnami.com/bitnami
    helm repo update

    # Deploy koprogo-api
    log_info "Deploying koprogo-api..."
    helm upgrade --install koprogo-api ./koprogo-api \
        --namespace koprogo \
        --create-namespace \
        --values "./koprogo-api/values-$env.yaml" \
        --wait \
        --timeout 10m

    # Deploy koprogo-frontend (if exists)
    if [ -d "./koprogo-frontend" ]; then
        log_info "Deploying koprogo-frontend..."
        helm upgrade --install koprogo-frontend ./koprogo-frontend \
            --namespace koprogo \
            --values "./koprogo-frontend/values-$env.yaml" \
            --wait \
            --timeout 10m
    fi

    log_info "Helm deployments complete ✓"
}

run_smoke_tests() {
    local env=$1

    log_info "Step 5/5: Running smoke tests..."

    export KUBECONFIG="$HOME/.kube/koprogo-$env"

    # Check all pods are running
    log_info "Checking pod status..."
    kubectl get pods -n koprogo

    # Test API health endpoint
    log_info "Testing API health endpoint..."
    local api_pod=$(kubectl get pods -n koprogo -l app.kubernetes.io/name=koprogo-api -o jsonpath='{.items[0].metadata.name}')

    if [ -n "$api_pod" ]; then
        kubectl exec -n koprogo "$api_pod" -- curl -f http://localhost:8080/api/v1/health || {
            log_warn "API health check failed"
        }
    fi

    log_info "Smoke tests complete ✓"
}

print_summary() {
    local env=$1

    echo ""
    echo "======================================"
    echo "  Deployment Complete! 🚀"
    echo "======================================"
    echo ""
    echo "Environment: $env"
    echo "Kubeconfig: $HOME/.kube/koprogo-$env"
    echo ""
    echo "To access the cluster:"
    echo "  export KUBECONFIG=$HOME/.kube/koprogo-$env"
    echo "  kubectl get nodes"
    echo "  kubectl get pods -A"
    echo ""
    echo "To access the API:"
    echo "  kubectl port-forward -n koprogo svc/koprogo-api 8080:8080"
    echo "  curl http://localhost:8080/api/v1/health"
    echo ""
}

# Main execution
main() {
    local env="${1:-}"

    if [ -z "$env" ]; then
        log_error "Environment not specified"
        echo "Usage: $0 [dev|staging|prod]"
        exit 1
    fi

    check_prerequisites
    check_environment "$env"

    echo ""
    echo "======================================"
    echo "  KoproGo Infrastructure Deployment"
    echo "  Environment: $env"
    echo "======================================"
    echo ""

    deploy_terraform "$env"
    deploy_ansible "$env"
    configure_kubectl "$env"
    deploy_helm "$env"
    run_smoke_tests "$env"

    print_summary "$env"
}

# Execute main function
main "$@"
