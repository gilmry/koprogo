#!/bin/bash
################################################################################
# GitOps Deployment Script for KoproGo
# Automatically pulls changes from Git and redeploys with Docker Compose
################################################################################

set -e

# Configuration
REPO_DIR="/home/debian/koprogo"
COMPOSE_FILE="deploy/production/docker-compose.yml"
ENV_FILE="deploy/production/.env"
BRANCH="main"
CHECK_INTERVAL=180  # 3 minutes (like Argo CD)
LOG_FILE="${HOME}/koprogo-gitops.log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

function log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

function log_error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" | tee -a "$LOG_FILE"
}

function log_warning() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1" | tee -a "$LOG_FILE"
}

function log_info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO:${NC} $1" | tee -a "$LOG_FILE"
}

function check_prerequisites() {
    log_info "Checking prerequisites..."

    if [ ! -d "$REPO_DIR" ]; then
        log_error "Repository directory not found: $REPO_DIR"
        exit 1
    fi

    if ! command -v docker &> /dev/null; then
        log_error "Docker not installed"
        exit 1
    fi

    if ! command -v git &> /dev/null; then
        log_error "Git not installed"
        exit 1
    fi

    log "Prerequisites OK"
}

function pull_latest() {
    cd "$REPO_DIR"

    # Fetch latest changes
    git fetch origin "$BRANCH" 2>&1 | tee -a "$LOG_FILE"

    # Check if there are new commits
    LOCAL=$(git rev-parse HEAD)
    REMOTE=$(git rev-parse origin/$BRANCH)

    if [ "$LOCAL" = "$REMOTE" ]; then
        return 1  # No changes
    fi

    log_info "New commits detected: $LOCAL -> $REMOTE"

    # Pull changes
    git pull origin "$BRANCH" 2>&1 | tee -a "$LOG_FILE"

    return 0  # Changes pulled
}

function deploy() {
    log "🚀 Starting deployment..."

    cd "$REPO_DIR"

    # Check if .env exists
    if [ ! -f "$ENV_FILE" ]; then
        log_error ".env file not found at $ENV_FILE"
        return 1
    fi

    # Get current commit SHA (short version - 7 chars like GitHub Actions)
    local current_sha=$(git rev-parse --short=7 HEAD)
    local image_tag="main-${current_sha}"

    log_info "Current commit: $current_sha"
    log_info "Target image tag: $image_tag"

    # Update IMAGE_TAG in .env temporarily for this deployment
    export IMAGE_TAG="$image_tag"

    # Pull latest images with retry logic (wait for GitHub Actions)
    log_info "Pulling Docker images with tag $image_tag..."

    local max_retries=10
    local retry_delay=90  # 90 seconds between retries (total: 15 minutes)
    local retry_count=0

    while [ $retry_count -lt $max_retries ]; do
        # Capture output to check for errors
        pull_output=$(docker compose -f "$COMPOSE_FILE" pull 2>&1 | tee -a "$LOG_FILE")

        # Check if pull was successful (no "manifest unknown" error)
        if echo "$pull_output" | grep -q "manifest unknown"; then
            retry_count=$((retry_count + 1))
            if [ $retry_count -lt $max_retries ]; then
                log_warning "Image $image_tag not yet available (attempt $retry_count/$max_retries). Waiting ${retry_delay}s for GitHub Actions to finish building..."
                sleep $retry_delay
            else
                log_error "Image $image_tag still not available after $max_retries attempts. GitHub Actions may have failed."
                log_warning "Falling back to 'latest' tag"
                export IMAGE_TAG="latest"
                docker compose -f "$COMPOSE_FILE" pull 2>&1 | tee -a "$LOG_FILE"
                break
            fi
        else
            log "✅ Images pulled successfully"
            break
        fi
    done

    # Deploy with Docker Compose
    log_info "Deploying services..."
    docker compose -f "$COMPOSE_FILE" up -d 2>&1 | tee -a "$LOG_FILE"

    # Wait for health checks
    log_info "Waiting for services to be healthy..."
    sleep 10

    # Check service status
    docker compose -f "$COMPOSE_FILE" ps 2>&1 | tee -a "$LOG_FILE"

    log "✅ Deployment complete!"

    # Send notification (optional)
    # curl -X POST https://hooks.slack.com/... -d "Deployment successful"

    return 0
}

function rollback() {
    log_warning "🔄 Starting rollback process..."

    cd "$REPO_DIR"

    # Show recent commits
    echo ""
    echo "Recent deployments (last 10 commits):"
    echo "========================================"
    git log --oneline --decorate -10
    echo ""

    # Ask for target commit
    read -p "Enter commit SHA to rollback to (or press Enter for previous commit): " TARGET_COMMIT

    if [ -z "$TARGET_COMMIT" ]; then
        # Default to previous commit
        TARGET_COMMIT=$(git rev-parse HEAD~1)
        log_info "No commit specified, using previous commit: $TARGET_COMMIT"
    fi

    # Verify commit exists
    if ! git rev-parse --verify "$TARGET_COMMIT" >/dev/null 2>&1; then
        log_error "Invalid commit: $TARGET_COMMIT"
        return 1
    fi

    # Get short SHA for display
    TARGET_SHA=$(git rev-parse --short "$TARGET_COMMIT")
    CURRENT_SHA=$(git rev-parse --short HEAD)

    log_warning "Rolling back from $CURRENT_SHA to $TARGET_SHA"

    # Confirm
    read -p "Are you sure? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log "Rollback cancelled"
        return 0
    fi

    # Checkout target commit
    git checkout "$TARGET_COMMIT" 2>&1 | tee -a "$LOG_FILE"

    # Redeploy with the target commit's image
    deploy

    log "✅ Rollback complete to commit $TARGET_SHA"
    log_warning "Note: You are now in detached HEAD state"
    log_info "To make this permanent, create a new branch or reset main:"
    log_info "  git checkout -b rollback-to-$TARGET_SHA"
    log_info "  OR: git checkout main && git reset --hard $TARGET_COMMIT && git push --force"
}

function cleanup_old_images() {
    log_info "Cleaning up old Docker images..."
    docker image prune -f 2>&1 | tee -a "$LOG_FILE"
}

function watch_mode() {
    log "👀 Starting GitOps watch mode (checking every ${CHECK_INTERVAL}s)..."
    log "Branch: $BRANCH"
    log "Repository: $REPO_DIR"
    log "Compose file: $COMPOSE_FILE"
    log ""

    while true; do
        log_info "Checking for updates..."

        if pull_latest; then
            log "📦 New version detected!"

            if deploy; then
                log "✅ Auto-deployment successful"
                cleanup_old_images
            else
                log_error "❌ Deployment failed!"
                # Optionally rollback on failure
                # rollback
            fi
        else
            log_info "No changes detected"
        fi

        log_info "Next check in ${CHECK_INTERVAL}s..."
        sleep "$CHECK_INTERVAL"
    done
}

function manual_deploy() {
    log "🚀 Manual deployment triggered"

    if pull_latest; then
        log "📦 Pulled latest changes"
    else
        log_info "Already up to date"
    fi

    deploy
    cleanup_old_images
}

function show_status() {
    cd "$REPO_DIR"

    echo "========================================="
    echo "GitOps Deployment Status"
    echo "========================================="
    echo "Current branch: $(git branch --show-current)"
    echo "Current commit: $(git rev-parse --short HEAD)"
    echo "Latest commit message: $(git log -1 --pretty=%B)"
    echo ""
    echo "Docker Compose Services:"
    docker compose -f "$COMPOSE_FILE" ps
    echo ""
    echo "Recent logs (last 20 lines):"
    tail -n 20 "$LOG_FILE"
}

# Main
case "${1:-help}" in
    watch)
        check_prerequisites
        watch_mode
        ;;
    deploy)
        check_prerequisites
        manual_deploy
        ;;
    rollback)
        check_prerequisites
        rollback
        ;;
    status)
        show_status
        ;;
    logs)
        tail -f "$LOG_FILE"
        ;;
    help|--help|-h)
        cat << EOF
GitOps Deployment Script for KoproGo

Usage: $0 [COMMAND]

Commands:
    watch       Start continuous deployment (checks Git every 3 min)
    deploy      Manually deploy latest version from Git
    rollback    Rollback to previous commit
    status      Show current deployment status
    logs        Show deployment logs (live)
    help        Show this help message

Examples:
    $0 watch        # Start GitOps auto-deployment
    $0 deploy       # Manually deploy now
    $0 status       # Check current status
    $0 rollback     # Rollback to previous version

EOF
        ;;
    *)
        log_error "Unknown command: $1"
        echo ""
        $0 help
        exit 1
        ;;
esac
