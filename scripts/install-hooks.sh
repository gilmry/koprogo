#!/bin/bash
# Install Git hooks for KoproGo project

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "📦 Installing Git hooks..."

# Check if we're in a git repository
if [ ! -d "$PROJECT_ROOT/.git" ]; then
    echo "❌ Error: Not in a git repository"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Install pre-commit hook
echo "  → Installing pre-commit hook..."
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
# Git pre-commit hook for KoproGo
# Runs formatting and linting before each commit

set -e

echo "🎯 Running pre-commit checks..."

# Change to project root
cd "$(git rev-parse --show-toplevel)"

# Backend formatting
echo "📝 Formatting Rust code..."
cd backend
cargo fmt --check || {
    echo "❌ Rust code not formatted. Running cargo fmt..."
    cargo fmt
    echo "✅ Rust code formatted. Please stage changes and commit again."
    exit 1
}

# Backend linting
echo "🔍 Linting Rust code..."
SQLX_OFFLINE=true cargo clippy -- -D warnings || {
    echo "❌ Clippy found issues. Please fix them before committing."
    exit 1
}

# Frontend formatting (if frontend files changed)
cd ../frontend
if git diff --cached --name-only | grep -q "^frontend/"; then
    echo "📝 Checking frontend formatting..."
    npx prettier --check "src/**/*.{ts,tsx,astro,svelte}" || {
        echo "❌ Frontend code not formatted. Running prettier..."
        npx prettier --write "src/**/*.{ts,tsx,astro,svelte}"
        echo "✅ Frontend code formatted. Please stage changes and commit again."
        exit 1
    }
fi

cd ..
echo "✅ All pre-commit checks passed!"
EOF

chmod +x "$HOOKS_DIR/pre-commit"

# Install pre-push hook
echo "  → Installing pre-push hook..."
cat > "$HOOKS_DIR/pre-push" << 'EOF'
#!/bin/bash
# Git pre-push hook for KoproGo
# Runs tests before pushing to remote

set -e

echo "🚀 Running pre-push checks..."

# Change to project root
cd "$(git rev-parse --show-toplevel)"

# Backend tests
echo "🧪 Running backend tests..."
cd backend

# Unit tests
echo "  → Unit tests..."
SQLX_OFFLINE=true cargo test --lib || {
    echo "❌ Unit tests failed. Please fix them before pushing."
    exit 1
}

# BDD tests (if cucumber is available)
if cargo test --test bdd --no-run &>/dev/null; then
    echo "  → BDD tests..."
    cargo test --test bdd || {
        echo "⚠️  BDD tests failed (non-blocking)"
    }
fi

# SQLx prepare check
echo "📦 Checking SQLx query cache..."
SQLX_OFFLINE=true cargo sqlx prepare --check || {
    echo "❌ SQLx query cache out of date. Run: cargo sqlx prepare"
    exit 1
}

# Frontend build check
cd ../frontend
if [ -d "node_modules" ]; then
    echo "🏗️  Checking frontend build..."
    npm run build || {
        echo "❌ Frontend build failed. Please fix errors before pushing."
        exit 1
    }
fi

cd ..
echo "✅ All pre-push checks passed!"
echo "🎉 Safe to push to remote!"
EOF

chmod +x "$HOOKS_DIR/pre-push"

echo "✅ Git hooks installed successfully!"
echo ""
echo "Hooks installed:"
echo "  • pre-commit: Format + Lint"
echo "  • pre-push: Tests + Build check"
echo ""
echo "To skip hooks temporarily:"
echo "  git commit --no-verify"
echo "  git push --no-verify"
