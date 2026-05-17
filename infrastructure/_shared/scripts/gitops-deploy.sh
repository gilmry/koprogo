#!/bin/bash
################################################################################
# GitOps Deployment Script for KoproGo
# Enhanced for multi-environment support
# Usage: BRANCH=<branch> COMPOSE_DIR=<path> ./gitops-deploy.sh [watch|deploy|status|logs]
################################################################################

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="${REPO_DIR:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
BRANCH="${BRANCH:-main}"
ENV_NAME="${ENV_NAME:-production}"
# TOPOLOGY: vps (default, per-env on a real OVH VPS) | local (supervisor's machine running env-dev)
# Determines which monosite/<topology>/<env>/ layout the override + env-file are read from.
TOPOLOGY="${TOPOLOGY:-vps}"

# Auto-detect compose files based on topology + environment
COMPOSE_BASE="${REPO_DIR}/infrastructure/_shared/docker-compose/docker-compose.base.yml"
COMPOSE_OVERRIDE="${COMPOSE_OVERRIDE:-${REPO_DIR}/infrastructure/monosite/${TOPOLOGY}/${ENV_NAME}/docker-compose.override.yml}"
ENV_FILE="${ENV_FILE:-${REPO_DIR}/infrastructure/monosite/${TOPOLOGY}/${ENV_NAME}/.env}"

CHECK_INTERVAL=${CHECK_INTERVAL:-180}
# Default log path: /var/log on VPS (root), $HOME/.local/state on local (user-mode).
if [ "$TOPOLOGY" = "local" ]; then
    LOG_FILE="${LOG_FILE:-${HOME}/.local/state/koprogo-gitops-${ENV_NAME}.log}"
    mkdir -p "$(dirname "$LOG_FILE")"
else
    LOG_FILE="${LOG_FILE:-/var/log/koprogo-gitops-${ENV_NAME}.log}"
fi

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'

function log()         { echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"; }
function log_error()   { echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" | tee -a "$LOG_FILE"; }
function log_warning() { echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1" | tee -a "$LOG_FILE"; }
function log_info()    { echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO:${NC} $1" | tee -a "$LOG_FILE"; }

# Populate the global COMPOSE_ARGS array with the docker compose invocation.
# Using an array (not a string) keeps paths-with-spaces safe and avoids the
# shellcheck SC2046/SC2086 word-splitting hazard of `$(compose_cmd) ...`.
COMPOSE_ARGS=()
function build_compose_args() {
    COMPOSE_ARGS=(docker compose -f "${COMPOSE_BASE}")
    if [ -f "$COMPOSE_OVERRIDE" ]; then
        COMPOSE_ARGS+=(-f "${COMPOSE_OVERRIDE}")
    fi
    if [ -f "$ENV_FILE" ]; then
        COMPOSE_ARGS+=(--env-file "${ENV_FILE}")
    fi
}

function check_prerequisites() {
    log_info "Checking prerequisites..."
    [ ! -d "$REPO_DIR" ] && log_error "Repository not found: $REPO_DIR" && exit 1
    command -v docker &>/dev/null || { log_error "Docker not installed"; exit 1; }
    command -v git &>/dev/null || { log_error "Git not installed"; exit 1; }
    log "Prerequisites OK (topology=${TOPOLOGY}, env=${ENV_NAME}, branch=${BRANCH})"
}

function pull_latest() {
    cd "$REPO_DIR"
    git fetch origin "$BRANCH" 2>&1 | tee -a "$LOG_FILE"
    local local_sha remote_sha
    local_sha=$(git rev-parse HEAD)
    remote_sha=$(git rev-parse "origin/$BRANCH")
    [ "$local_sha" = "$remote_sha" ] && return 1
    log_info "New commits: $local_sha -> $remote_sha"
    git pull origin "$BRANCH" 2>&1 | tee -a "$LOG_FILE"
    return 0
}

function deploy() {
    log "Starting deployment (env=${ENV_NAME})..."
    cd "$REPO_DIR"

    local current_sha image_tag
    current_sha=$(git rev-parse --short=7 HEAD)
    image_tag="${BRANCH}-${current_sha}"
    export IMAGE_TAG="$image_tag"

    log_info "Commit: $current_sha | Image tag: $image_tag"

    build_compose_args

    # Pull images with retry
    local max_retries=10 retry_delay=90 retry_count=0 pull_output
    while [ "$retry_count" -lt "$max_retries" ]; do
        pull_output=$("${COMPOSE_ARGS[@]}" pull 2>&1 | tee -a "$LOG_FILE")
        if echo "$pull_output" | grep -q "manifest unknown"; then
            retry_count=$((retry_count + 1))
            if [ "$retry_count" -lt "$max_retries" ]; then
                log_warning "Image not ready (attempt $retry_count/$max_retries). Waiting ${retry_delay}s..."
                sleep "$retry_delay"
            else
                log_warning "Falling back to '${BRANCH}-latest'"
                export IMAGE_TAG="${BRANCH}-latest"
                "${COMPOSE_ARGS[@]}" pull 2>&1 | tee -a "$LOG_FILE"
                break
            fi
        else
            log "Images pulled successfully"
            break
        fi
    done

    "${COMPOSE_ARGS[@]}" up -d 2>&1 | tee -a "$LOG_FILE"
    sleep 10
    "${COMPOSE_ARGS[@]}" ps 2>&1 | tee -a "$LOG_FILE"
    log "Deployment complete!"
}

function watch_mode() {
    log "Starting GitOps watch (topology=${TOPOLOGY}, env=${ENV_NAME}, branch=${BRANCH}, interval=${CHECK_INTERVAL}s)..."
    while true; do
        log_info "Checking for updates..."
        if pull_latest; then
            log "New version detected!"
            if deploy; then
                log "Auto-deployment successful"
                docker image prune -f 2>&1 | tee -a "$LOG_FILE"
            else
                log_error "Deployment failed!"
            fi
        else
            log_info "No changes"
        fi
        sleep "$CHECK_INTERVAL"
    done
}

function show_status() {
    cd "$REPO_DIR"
    build_compose_args
    echo "========================================="
    echo "GitOps Status: ${ENV_NAME}"
    echo "========================================="
    echo "Branch: $(git branch --show-current)"
    echo "Commit: $(git rev-parse --short HEAD)"
    echo "Message: $(git log -1 --pretty=%B)"
    echo ""
    "${COMPOSE_ARGS[@]}" ps
}

case "${1:-help}" in
    watch)   check_prerequisites; watch_mode ;;
    deploy)  check_prerequisites; pull_latest || true; deploy ;;
    status)  show_status ;;
    logs)    tail -f "$LOG_FILE" ;;
    help|*)  echo "Usage: $0 [watch|deploy|status|logs]"; echo "Env vars: BRANCH, ENV_NAME, TOPOLOGY (vps|local), REPO_DIR, COMPOSE_OVERRIDE, ENV_FILE" ;;
esac
