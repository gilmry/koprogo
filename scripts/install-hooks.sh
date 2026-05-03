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

# Auto-stage formatted files
echo "📝 Staging formatted files..."
git add -u

echo "✅ All pre-commit checks passed!"
EOF

chmod +x "$HOOKS_DIR/pre-commit"

# Install pre-push hook
echo "  → Installing pre-push hook..."
cat > "$HOOKS_DIR/pre-push" << 'EOF'
#!/bin/bash
# Git pre-push hook for KoproGo
# Runs comprehensive CI checks before pushing to remote.
#
# Skips `make ci` when the push contains no new commits relative to origin
# (e.g., creating a snapshot branch like `infra-dev` from `main`, deleting
# a branch, or a no-op push). This avoids the heavy CI suite running
# unchanged code multiple times during admin operations.

set -e

# Detect "no new commits" pushes via the git push hook stdin protocol
# (each line: <local_ref> <local_sha> <remote_ref> <remote_sha>).
ZERO_SHA="0000000000000000000000000000000000000000"
SKIP_CI=true
while read -r local_ref local_sha remote_ref remote_sha; do
    if [ "$local_sha" = "$ZERO_SHA" ]; then
        # Branch deletion — no CI to run
        continue
    fi
    if [ "$remote_sha" = "$ZERO_SHA" ]; then
        # New ref creation — skip CI iff local_sha is already reachable from
        # any origin/* ref (e.g., creating infra-dev that points to main).
        if ! git branch -r --contains "$local_sha" 2>/dev/null | grep -q "origin/"; then
            SKIP_CI=false
            break
        fi
    else
        # Update of an existing ref — skip CI iff no commits between sha pair
        if [ "$(git rev-list --count "${remote_sha}..${local_sha}" 2>/dev/null || echo 1)" != "0" ]; then
            SKIP_CI=false
            break
        fi
    fi
done

if [ "$SKIP_CI" = "true" ]; then
    echo "🟢 No new commits vs origin — skipping make ci (snapshot branch, deletion, or no-op)."
    echo "✅ Pre-push OK (fast path)."
    exit 0
fi

echo "🚀 Running pre-push checks..."

# Change to project root
cd "$(git rev-parse --show-toplevel)"

# Load nvm and use the version declared in .nvmrc (Node >=22 required by Astro)
export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
# shellcheck source=/dev/null
[ -s "$NVM_DIR/nvm.sh" ] && source "$NVM_DIR/nvm.sh"
[ -f ".nvmrc" ] && nvm use --silent 2>/dev/null || true

# Run full CI checks (lint + check-frontend + test + audit)
make ci || {
    echo "❌ CI checks failed. Please fix issues before pushing."
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
echo "  • pre-push: Full CI (Lint + TypeScript Check + Tests + Audit)"
echo ""
echo "To skip hooks temporarily:"
echo "  git commit --no-verify"
echo "  git push --no-verify"
