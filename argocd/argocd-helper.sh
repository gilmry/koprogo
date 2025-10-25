#!/bin/bash
# Argo CD Helper Script for KoproGo
# Simplifies common Argo CD operations

set -e

ARGOCD_SERVER="${ARGOCD_SERVER:-localhost:8080}"
ARGOCD_NAMESPACE="${ARGOCD_NAMESPACE:-argocd}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

function print_header() {
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}$1${NC}"
    echo -e "${GREEN}========================================${NC}"
}

function print_error() {
    echo -e "${RED}❌ $1${NC}"
}

function print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

function print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

function install_argocd() {
    print_header "Installing Argo CD"

    # Start Argo CD stack
    docker compose -f docker-compose.argocd.yml up -d

    print_success "Argo CD is starting..."
    echo "Waiting for Argo CD to be ready (this may take 1-2 minutes)..."
    sleep 30

    # Get initial admin password
    print_header "Argo CD Initial Password"
    INITIAL_PASSWORD=$(docker logs argocd-server 2>&1 | grep "admin password" | tail -1 | awk '{print $NF}')

    if [ -z "$INITIAL_PASSWORD" ]; then
        print_warning "Could not extract initial password. Check logs manually:"
        echo "docker logs argocd-server 2>&1 | grep 'admin password'"
    else
        echo "Username: admin"
        echo "Password: $INITIAL_PASSWORD"
        echo ""
        print_warning "IMPORTANT: Change this password immediately!"
    fi

    echo ""
    echo "Access Argo CD UI at: http://localhost:8080"
    echo ""
    print_success "Installation complete!"
}

function uninstall_argocd() {
    print_header "Uninstalling Argo CD"

    read -p "This will remove all Argo CD data. Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Cancelled."
        exit 0
    fi

    docker compose -f docker-compose.argocd.yml down -v
    print_success "Argo CD uninstalled"
}

function create_app() {
    print_header "Creating Argo CD Application"

    # Check if argocd CLI is installed
    if ! command -v argocd &> /dev/null; then
        print_error "argocd CLI not found. Installing..."

        # Install argocd CLI
        VERSION=$(curl -s https://api.github.com/repos/argoproj/argo-cd/releases/latest | grep tag_name | cut -d '"' -f 4)
        curl -sSL -o /tmp/argocd-linux-amd64 https://github.com/argoproj/argo-cd/releases/download/$VERSION/argocd-linux-amd64
        sudo install -m 555 /tmp/argocd-linux-amd64 /usr/local/bin/argocd
        rm /tmp/argocd-linux-amd64
        print_success "argocd CLI installed"
    fi

    # Login to Argo CD
    echo "Login to Argo CD (use the password from installation)"
    argocd login $ARGOCD_SERVER --insecure

    # Apply application
    kubectl apply -f application.yaml

    print_success "Application created!"
    echo "View in UI: http://localhost:8080/applications/koprogo-production"
}

function sync_app() {
    print_header "Syncing Application"

    argocd app sync koprogo-production --insecure
    argocd app wait koprogo-production --insecure

    print_success "Application synced!"
}

function get_status() {
    print_header "Application Status"

    argocd app get koprogo-production --insecure
}

function get_logs() {
    print_header "Application Logs"

    argocd app logs koprogo-production --insecure --follow
}

function rollback() {
    print_header "Rollback Application"

    # List history
    argocd app history koprogo-production --insecure

    echo ""
    read -p "Enter revision ID to rollback to: " REVISION

    if [ -z "$REVISION" ]; then
        print_error "No revision provided"
        exit 1
    fi

    argocd app rollback koprogo-production $REVISION --insecure

    print_success "Rollback initiated!"
}

function update_image_tag() {
    print_header "Update Image Tag"

    read -p "Enter new image tag (e.g., v1.2.3, main-abc1234): " NEW_TAG

    if [ -z "$NEW_TAG" ]; then
        print_error "No tag provided"
        exit 1
    fi

    # Update .env file
    cd ../deploy/production

    if [ ! -f .env ]; then
        cp .env.example .env
    fi

    # Update IMAGE_TAG in .env
    sed -i "s/^IMAGE_TAG=.*/IMAGE_TAG=$NEW_TAG/" .env

    print_success "Updated IMAGE_TAG to $NEW_TAG in .env"
    print_warning "Don't forget to commit and push this change to trigger Argo CD sync!"

    echo ""
    echo "Next steps:"
    echo "1. git add deploy/production/.env"
    echo "2. git commit -m \"chore: update image tag to $NEW_TAG\""
    echo "3. git push"
    echo "4. Argo CD will auto-sync within 3 minutes"
}

function show_ui() {
    print_header "Argo CD UI"

    echo "Argo CD UI: http://localhost:8080"
    echo ""
    echo "To access from a remote machine, use SSH port forwarding:"
    echo "ssh -L 8080:localhost:8080 user@your-vps-ip"
}

function show_help() {
    cat << EOF
Argo CD Helper Script for KoproGo

Usage: $0 [COMMAND]

Commands:
    install         Install Argo CD on this VPS
    uninstall       Uninstall Argo CD and remove all data
    create-app      Create the KoproGo application in Argo CD
    sync            Manually sync the application
    status          Get application status
    logs            Stream application logs
    rollback        Rollback to a previous revision
    update-tag      Update the Docker image tag
    ui              Show Argo CD UI URL
    help            Show this help message

Examples:
    $0 install              # Install Argo CD
    $0 create-app           # Create the application
    $0 sync                 # Force sync
    $0 update-tag           # Update image tag
    $0 status               # Check status

EOF
}

# Main
case "${1:-help}" in
    install)
        install_argocd
        ;;
    uninstall)
        uninstall_argocd
        ;;
    create-app)
        create_app
        ;;
    sync)
        sync_app
        ;;
    status)
        get_status
        ;;
    logs)
        get_logs
        ;;
    rollback)
        rollback
        ;;
    update-tag)
        update_image_tag
        ;;
    ui)
        show_ui
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
