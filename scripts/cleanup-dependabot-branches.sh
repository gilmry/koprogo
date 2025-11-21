#!/bin/bash

# Script to clean up obsolete Dependabot branches
# Generated: 2025-11-21
# Deletes old Dependabot branches for superseded dependency versions

set -e

echo "=========================================="
echo "  Dependabot Branch Cleanup Script"
echo "=========================================="
echo ""
echo "This script will delete obsolete Dependabot branches."
echo ""

# Fetch latest
echo "📡 Fetching latest from remote..."
git fetch --all --prune
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Counter
total_deleted=0
total_skipped=0

echo "🔍 Analyzing Dependabot branches..."
echo ""

# List all Dependabot branches grouped by package
echo "📦 Current Dependabot branches by package:"
echo ""

echo "Backend (Cargo):"
git branch -r | grep "origin/dependabot/cargo" | sed 's/origin\/dependabot\/cargo\/backend\//  - /' | sort
echo ""

echo "Frontend (NPM):"
git branch -r | grep "origin/dependabot/npm_and_yarn/frontend" | sed 's/origin\/dependabot\/npm_and_yarn\/frontend\//  - /' | sort
echo ""

echo "GitHub Actions:"
git branch -r | grep "origin/dependabot/github_actions" | sed 's/origin\/dependabot\/github_actions\//  - /' | sort
echo ""

# Strategy: Keep only the LATEST version of each package, delete older ones
echo "=========================================="
echo "  Recommended Cleanup Strategy"
echo "=========================================="
echo ""
echo "We should keep ONLY the latest version of each dependency and delete older ones."
echo ""

# Branches to DELETE (obsolete versions)
obsolete_dependabot=(
    # Astro: Keep 5.15.4, delete 5.15.1, 5.15.3
    "dependabot/npm_and_yarn/frontend/astro-5.15.1"
    "dependabot/npm_and_yarn/frontend/astro-5.15.3"

    # Astro/Svelte: Keep 7.2.2, delete 7.2.1
    "dependabot/npm_and_yarn/frontend/astrojs/svelte-7.2.1"

    # Svelte: Keep 5.43.5, delete older versions
    "dependabot/npm_and_yarn/frontend/svelte-5.41.3"
    "dependabot/npm_and_yarn/frontend/svelte-5.42.2"
    "dependabot/npm_and_yarn/frontend/svelte-5.43.2"

    # Tailwind: Keep 4.1.17, delete 4.1.16
    "dependabot/npm_and_yarn/frontend/tailwindcss-4.1.16"

    # Tailwind/Vite: Keep 4.1.17, delete 4.1.16
    "dependabot/npm_and_yarn/frontend/tailwindcss/vite-4.1.16"

    # AWS SDK: Keep 1.111.0, delete 1.110.0
    "dependabot/cargo/backend/aws-sdk-s3-1.110.0"

    # jsonwebtoken: Keep 10.2.0, delete 10.1.0
    "dependabot/cargo/backend/jsonwebtoken-10.1.0"

    # rust_xlsxwriter: Keep 0.91.0, delete 0.90.2
    "dependabot/cargo/backend/rust_xlsxwriter-0.90.2"

    # Duplicate npm_and_yarn (cryptic hashes - likely obsolete)
    "dependabot/npm_and_yarn/frontend/npm_and_yarn-0084abb8db"
    "dependabot/npm_and_yarn/frontend/npm_and_yarn-a1461c8ffc"
)

echo "📋 Obsolete Dependabot branches to delete (${#obsolete_dependabot[@]} total):"
for branch in "${obsolete_dependabot[@]}"; do
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
for branch in "${obsolete_dependabot[@]}"; do
    echo -n "  Deleting origin/$branch ... "

    if git push origin --delete "$branch" 2>/dev/null; then
        echo -e "${GREEN}✓ deleted${NC}"
        ((total_deleted++))
    else
        echo -e "${RED}✗ failed (may already be deleted)${NC}"
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

echo "📊 Remaining Dependabot branches:"
echo "  Cargo: $(git branch -r | grep 'origin/dependabot/cargo' | wc -l)"
echo "  NPM: $(git branch -r | grep 'origin/dependabot/npm_and_yarn' | wc -l)"
echo "  GitHub Actions: $(git branch -r | grep 'origin/dependabot/github_actions' | wc -l)"
echo ""

echo "✅ Dependabot cleanup complete!"
echo ""
echo "Next steps:"
echo "  1. Run: git fetch --prune"
echo "  2. Merge remaining Dependabot PRs on GitHub"
echo "  3. Re-run this script after merging to clean up merged branches"
echo ""
