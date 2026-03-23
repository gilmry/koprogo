#!/bin/bash
################################################################################
# GitOps Deployment Script for KoproGo
# Enhanced for multi-environment support
# Usage: BRANCH=<branch> COMPOSE_DIR=<path> ./gitops-deploy.sh [watch|deploy|status|logs]
################################################################################

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="${REPO_DIR:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
BRANCH="${BRANCH:-main}"
ENV_NAME="${ENV_NAME:-production}"

# Auto-detect compose files based on environment
COMPOSE_BASE="${REPO_DIR}/infrastructure/_shared/docker-compose/docker-compose.base.yml"
COMPOSE_OVERRIDE="${COMPOSE_OVERRIDE:-${REPO_DIR}/infrastructure/monosite/vps/${ENV_NAME}/docker-compose.override.yml}"
ENV_FILE="${ENV_FILE:-${REPO_DIR}/infrastructure/monosite/vps/${ENV_NAME}/.env}"

CHECK_INTERVAL=${CHECK_INTERVAL:-180}
LOG_FILE="${LOG_FILE:-/var/log/koprogo-gitops-${ENV_NAME}.log}"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'

function log()         { echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"; }
function log_error()   { echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" | tee -a "$LOG_FILE"; }
function log_warning() { echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1" | tee -a "$LOG_FILE"; }
function log_info()    { echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO:${NC} $1" | tee -a "$LOG_FILE"; }

function compose_cmd() {
    local cmd="docker compose -f ${COMPOSE_BASE}"
    if [ -f "$COMPOSE_OVERRIDE" ]; then
        cmd="$cmd -f ${COMPOSE_OVERRIDE}"
    fi
    if [ -f "$ENV_FILE" ]; then
        cmd="$cmd --env-file ${ENV_FILE}"
    fi
    echo "$cmd"
}

function check_prerequisites() {
    log_info "Checking prerequisites..."
    [ ! -d "$REPO_DIR" ] && log_error "Repository not found: $REPO_DIR" && exit 1
    command -v docker &>/dev/null || { log_error "Docker not installed"; exit 1; }
    command -v git &>/dev/null || { log_error "Git not installed"; exit 1; }
    log "Prerequisites OK (env=${ENV_NAME}, branch=${BRANCH})"
}

function pull_latest() {
    cd "$REPO_DIR"
    git fetch origin "$BRANCH" 2>&1 | tee -a "$LOG_FILE"
    LOCAL=$(git rev-parse HEAD)
    REMOTE=$(git rev-parse origin/$BRANCH)
    [ "$LOCAL" = "$REMOTE" ] && return 1
    log_info "New commits: $LOCAL -> $REMOTE"
    git pull origin "$BRANCH" 2>&1 | tee -a "$LOG_FILE"
    return 0
}

function deploy() {
    log "Starting deployment (env=${ENV_NAME})..."
    cd "$REPO_DIR"

    local current_sha=$(git rev-parse --short=7 HEAD)
    local image_tag="${BRANCH}-${current_sha}"
    export IMAGE_TAG="$image_tag"

    log_info "Commit: $current_sha | Image tag: $image_tag"

    # Pull images with retry
    local max_retries=10 retry_delay=90 retry_count=0
    while [ $retry_count -lt $max_retries ]; do
        pull_output=$($(compose_cmd) pull 2>&1 | tee -a "$LOG_FILE")
        if echo "$pull_output" | grep -q "manifest unknown"; then
            retry_count=$((retry_count + 1))
            [ $retry_count -lt $max_retries ] && { log_warning "Image not ready (attempt $retry_count/$max_retries). Waiting ${retry_delay}s..."; sleep $retry_delay; } || { log_warning "Falling back to '${BRANCH}-latest'"; export IMAGE_TAG="${BRANCH}-latest"; $(compose_cmd) pull 2>&1 | tee -a "$LOG_FILE"; break; }
        else
            log "Images pulled successfully"; break
        fi
    done

    $(compose_cmd) up -d 2>&1 | tee -a "$LOG_FILE"
    sleep 10
    $(compose_cmd) ps 2>&1 | tee -a "$LOG_FILE"
    log "Deployment complete!"
}

function watch_mode() {
    log "Starting GitOps watch (env=${ENV_NAME}, branch=${BRANCH}, interval=${CHECK_INTERVAL}s)..."
    while true; do
        log_info "Checking for updates..."
        if pull_latest; then
            log "New version detected!"
            deploy && { log "Auto-deployment successful"; docker image prune -f 2>&1 | tee -a "$LOG_FILE"; } || log_error "Deployment failed!"
        else
            log_info "No changes"
        fi
        sleep "$CHECK_INTERVAL"
    done
}

function show_status() {
    cd "$REPO_DIR"
    echo "========================================="
    echo "GitOps Status: ${ENV_NAME}"
    echo "========================================="
    echo "Branch: $(git branch --show-current)"
    echo "Commit: $(git rev-parse --short HEAD)"
    echo "Message: $(git log -1 --pretty=%B)"
    echo ""
    $(compose_cmd) ps
}

case "${1:-help}" in
    watch)   check_prerequisites; watch_mode ;;
    deploy)  check_prerequisites; pull_latest || true; deploy ;;
    status)  show_status ;;
    logs)    tail -f "$LOG_FILE" ;;
    help|*)  echo "Usage: $0 [watch|deploy|status|logs]"; echo "Env vars: BRANCH, ENV_NAME, REPO_DIR, COMPOSE_OVERRIDE, ENV_FILE" ;;
esac
