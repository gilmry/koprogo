#!/bin/bash

# Script to clean up merged branches from remote repository
# Generated: 2025-11-21
# Safe to run: Only deletes branches that are fully merged into main

set -e

echo "=========================================="
echo "  KoproGo Branch Cleanup Script"
echo "=========================================="
echo ""
echo "This script will delete remote branches that are fully merged into main."
echo ""

# Fetch latest to ensure we have up-to-date information
echo "📡 Fetching latest from remote..."
git fetch --all --prune
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counter
total_deleted=0
total_skipped=0

echo "🔍 Analyzing branches..."
echo ""

# List of branches to delete (fully merged into main)
merged_branches=(
    "chore/dependabot-updates-2025-11-03"
    "claude/add-ci-workflows-011CUMhvUnFsKBJoJ9rbWXoN"
    "claude/analyze-koprogov-gaps-011CUh7ioKWdZhjJ1f9FuUsA"
    "claude/board-bdd-tests-011CUoTtHbDHEVcjxuBomuXa"
    "claude/coproperty-management-api-011CUMdbsxYD4gaY6CunHTxJ"
    "claude/implement-feature-011CUu1Thc9FFvC2kV5Qptem"
    "claude/install-make-commands-011CUXqV27esaJtLT3uMgCWw"
    "claude/invoice-system-issue-73-011CUoTtHbDHEVcjxuBomuXa"
    "claude/koprogo-agile-documentation-01CCjEWRbpxLjdDLdteKs58A"
    "claude/koprogo-docs-update-011CV4VuR2WmjfjGkYqnABrp"
    "claude/market-study-priorities-011CUQaaCd44rswjFhufsWVX"
    "claude/restructure-koprogo-docs-015UntXtYRrUE5XMV9CyKRBx"
)

echo "📋 Branches to delete (${#merged_branches[@]} total):"
for branch in "${merged_branches[@]}"; do
    echo "  - origin/$branch"
done
echo ""

# Ask for confirmation
read -p "❓ Do you want to proceed with deletion? (yes/no): " confirm
echo ""

if [ "$confirm" != "yes" ]; then
    echo "${YELLOW}⚠️  Aborted by user${NC}"
    exit 0
fi

echo "🗑️  Starting deletion process..."
echo ""

# Delete each branch
for branch in "${merged_branches[@]}"; do
    echo -n "  Deleting origin/$branch ... "

    # Verify it's actually merged before deleting
    merge_base=$(git merge-base origin/main origin/$branch 2>/dev/null || echo "error")
    branch_head=$(git rev-parse origin/$branch 2>/dev/null || echo "error")

    if [ "$merge_base" = "$branch_head" ] && [ "$merge_base" != "error" ]; then
        # Safe to delete
        if git push origin --delete "$branch" 2>/dev/null; then
            echo -e "${GREEN}✓ deleted${NC}"
            ((total_deleted++))
        else
            echo -e "${RED}✗ failed (may already be deleted)${NC}"
            ((total_skipped++))
        fi
    else
        echo -e "${YELLOW}⚠ skipped (not fully merged or doesn't exist)${NC}"
        ((total_skipped++))
    fi
done

echo ""
echo "=========================================="
echo "  Cleanup Summary"
echo "=========================================="
echo ""
echo -e "${GREEN}✓ Deleted: $total_deleted branches${NC}"
echo -e "${YELLOW}⚠ Skipped: $total_skipped branches${NC}"
echo ""

# Optional: Clean up obsolete Dependabot branches
echo "=========================================="
echo "  Optional: Dependabot Branch Cleanup"
echo "=========================================="
echo ""
echo "There are also ~42 Dependabot branches that may be obsolete."
echo ""
read -p "❓ Do you want to see Dependabot branches? (yes/no): " show_dependabot

if [ "$show_dependabot" = "yes" ]; then
    echo ""
    echo "📦 Dependabot branches:"
    git branch -r | grep "origin/dependabot/" | sort
    echo ""
    echo "To delete obsolete Dependabot branches, you can:"
    echo "  1. Close the corresponding PRs on GitHub"
    echo "  2. Or manually delete with: git push origin --delete dependabot/..."
    echo ""
fi

echo "✅ Cleanup complete!"
echo ""
echo "Next steps:"
echo "  1. Run: git fetch --prune  (to update your local refs)"
echo "  2. Verify on GitHub: https://github.com/gilmry/koprogo/branches"
echo ""
