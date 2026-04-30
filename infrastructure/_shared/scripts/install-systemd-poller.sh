#!/usr/bin/env bash
################################################################################
# install-systemd-poller.sh — install the KoproGo GitOps systemd user unit.
#
# Usage:
#   ./infrastructure/_shared/scripts/install-systemd-poller.sh
#
# Idempotent: re-running re-renders the unit with the current REPO_DIR.
#
# Linux/Mac (with systemd) only. For Windows, use windows-task-poller.ps1.
################################################################################
set -euo pipefail

REPO_DIR="$(git rev-parse --show-toplevel)"
UNIT_NAME="koprogo-gitops-env-dev.service"
TEMPLATE="$REPO_DIR/infrastructure/_shared/systemd/$UNIT_NAME"
USER_UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
TARGET="$USER_UNIT_DIR/$UNIT_NAME"

if ! command -v systemctl >/dev/null 2>&1; then
    echo "ERROR: systemctl not found. systemd-based Linux required." >&2
    echo "       For Windows, use: .\\infrastructure\\_shared\\scripts\\windows-task-poller.ps1" >&2
    exit 1
fi

if [ ! -f "$TEMPLATE" ]; then
    echo "ERROR: unit template not found at $TEMPLATE" >&2
    exit 1
fi

if [ ! -f "$REPO_DIR/infrastructure/monosite/local/env-dev/.env" ]; then
    echo "WARNING: $REPO_DIR/infrastructure/monosite/local/env-dev/.env is missing." >&2
    echo "         Copy from .env.example before starting the unit:" >&2
    echo "           cp infrastructure/monosite/local/env-dev/.env.example \\" >&2
    echo "              infrastructure/monosite/local/env-dev/.env" >&2
fi

mkdir -p "$USER_UNIT_DIR"

# Render placeholder
sed "s|__KOPROGO_REPO_DIR__|$REPO_DIR|g" "$TEMPLATE" > "$TARGET"
chmod 0644 "$TARGET"
chmod +x "$REPO_DIR/infrastructure/_shared/scripts/gitops-deploy.sh"

systemctl --user daemon-reload

echo "✅ Installed: $TARGET"
echo "   REPO_DIR: $REPO_DIR"
echo ""
echo "Next steps (you must do them manually — Tier 1 per CRITICAL.md):"
echo "  1. cp infrastructure/monosite/local/env-dev/.env.example infrastructure/monosite/local/env-dev/.env"
echo "     # then edit .env (rotate JWT_SECRET to random)"
echo "  2. Add to /etc/hosts:  127.0.0.1 envdev.koprogo.local api-envdev.koprogo.local"
echo "  3. systemctl --user enable --now $UNIT_NAME"
echo "  4. journalctl --user -u $UNIT_NAME -f"
echo ""
echo "To uninstall: systemctl --user disable --now $UNIT_NAME && rm $TARGET"
