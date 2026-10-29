#!/bin/bash
# Warmup script - Prépare le système avant les tests de charge
# Run this before any load test to warm up caches, JIT, etc.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Default to api.koprogo.com for remote testing
# Use http://localhost:8080 for local development
if [ -z "$BASE_URL" ]; then
    BASE_URL="https://api.koprogo.com"
    echo "ℹ️  Using default URL: $BASE_URL"
    echo ""
    echo "💡 For local development: export BASE_URL=http://localhost:8080"
    echo "💡 For other domain: export BASE_URL=https://your-domain.com"
    echo ""
fi

echo "========================================="
echo "🔥 KoproGo Load Test - WARMUP"
echo "========================================="
echo "Base URL: $BASE_URL"
echo "Duration: 30 seconds"
echo ""

# k6 local, ou Docker pour l'image épinglée
source "${SCRIPT_DIR}/k6.sh"
k6_verifier

# Check if API is up
echo "Checking API health..."
if ! curl -f -s "$BASE_URL/api/v1/health" > /dev/null; then
    echo "❌ Error: API is not responding at $BASE_URL"
    echo "Make sure the API is running: docker compose -f docker-compose.vps.yml ps"
    exit 1
fi
echo "✅ API is healthy"
echo ""

# Warmup phase: Low load for 30 seconds
echo "Starting warmup (30 seconds)..."
echo "- 2 threads"
echo "- 5 connections"
echo "- Target: ~50 req/s"
echo ""

k6_run health 5 30s

echo ""
echo "✅ Warmup complete! System is ready for load testing."
echo ""
