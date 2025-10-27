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

# Run make pre-commit (format + lint)
make pre-commit || {
    echo "❌ Pre-commit checks failed. Please fix the issues and try again."
    exit 1
}

echo "✅ All pre-commit checks passed!"
EOF

chmod +x "$HOOKS_DIR/pre-commit"

# Install pre-push hook
echo "  → Installing pre-push hook..."
cat > "$HOOKS_DIR/pre-push" << 'EOF'
#!/bin/bash
# Git pre-push hook for KoproGo
# Runs comprehensive tests before pushing to remote

set -e

echo "🚀 Running pre-push checks..."

# Change to project root
cd "$(git rev-parse --show-toplevel)"

# Run linting
echo "🔍 Running linting..."
make lint || {
    echo "❌ Linting failed. Please fix issues before pushing."
    exit 1
}

# Run all tests (unit + e2e + bdd)
echo "🧪 Running all tests..."
make test || {
    echo "❌ Tests failed. Please fix them before pushing."
    exit 1
}

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
