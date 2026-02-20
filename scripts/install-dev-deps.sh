#!/usr/bin/env bash
set -euo pipefail

echo "=== KoproGo — Installation des dépendances de développement ==="
echo ""

# 1. Dépendances système (compilation Rust + PostgreSQL client + SSL)
echo "[1/5] Dépendances système (apt)..."
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
  build-essential \
  pkg-config \
  libssl-dev \
  libpq-dev \
  curl \
  ca-certificates

# 2. Rust toolchain
echo ""
echo "[2/5] Rust toolchain..."
if command -v rustup &>/dev/null; then
  echo "  rustup déjà installé ($(rustc --version))"
  rustup update stable --no-self-update
else
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  source "$HOME/.cargo/env"
fi
rustup component add rustfmt clippy

# 3. sqlx-cli
echo ""
echo "[3/5] sqlx-cli..."
if command -v sqlx &>/dev/null; then
  echo "  sqlx-cli déjà installé ($(sqlx --version))"
else
  cargo install sqlx-cli --no-default-features --features postgres
fi

# 4. Node.js + npm
echo ""
echo "[4/5] Node.js..."
if command -v node &>/dev/null && [[ "$(node --version)" == v2* ]]; then
  echo "  Node.js déjà installé ($(node --version))"
else
  curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
  sudo apt-get install -y nodejs
fi

# 5. Dépendances frontend (npm install)
echo ""
echo "[5/5] Dépendances frontend (npm)..."
cd "$(dirname "$0")/../frontend"
npm install

echo ""
echo "=== Installation terminée ==="
echo ""
echo "Vérification:"
echo "  cc:      $(cc --version 2>&1 | head -1)"
echo "  rustc:   $(rustc --version)"
echo "  cargo:   $(cargo --version)"
echo "  clippy:  $(cargo clippy --version)"
echo "  sqlx:    $(sqlx --version 2>/dev/null || echo 'non installé')"
echo "  node:    $(node --version)"
echo "  npm:     $(npm --version)"
echo ""
echo "Prochaines étapes:"
echo "  make docker-up    # Démarrer PostgreSQL"
echo "  make migrate      # Appliquer les migrations"
echo "  make test         # Lancer les tests"
