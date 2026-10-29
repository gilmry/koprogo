#!/bin/bash
# Heavy load test - Finds the breaking point
# Target: 1000 req/s or until system saturates

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"

# Default to api.koprogo.com for remote testing
if [ -z "$BASE_URL" ]; then
    BASE_URL="https://api.koprogo.com"
    echo "ℹ️  Using default URL: $BASE_URL"
    echo ""
    echo "💡 For local development: export BASE_URL=http://localhost:8080"
    echo "💡 For other domain: export BASE_URL=https://your-domain.com"
    echo ""
fi

mkdir -p "$RESULTS_DIR"

echo "========================================="
echo "🔴 KoproGo Load Test - HEAVY LOAD"
echo "========================================="
echo "Base URL: $BASE_URL"
echo "Duration: 3 minutes"
echo "Target: Push to limit"
echo ""
echo "⚠️  WARNING: This test will push the system to its limits!"
echo "⚠️  Expect high CPU usage and potential errors."
echo ""

# Check dependencies
# k6 local, ou Docker pour l'image épinglée
source "${SCRIPT_DIR}/k6.sh"
k6_verifier

if ! curl -f -s "$BASE_URL/api/v1/health" > /dev/null; then
    echo "❌ Error: API is not responding"
    exit 1
fi

read -p "Continue with heavy load test? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Test cancelled."
    exit 0
fi

echo ""
echo "Test parameters:"
echo "- Threads: 4"
echo "- Connections: 100"
echo "- Duration: 3 minutes"
echo ""

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_FILE="$RESULTS_DIR/heavy-load_${TIMESTAMP}.txt"

# Run warmup
echo "Running warmup..."
k6_run health 5 10s > /dev/null 2>&1
echo "✅ Warmup complete"
echo ""

echo "Starting heavy load test..."
echo "⚠️  MONITOR RESOURCES NOW:"
echo "    Terminal 2: watch -n 1 'docker stats --no-stream'"
echo "    Terminal 3: docker compose -f docker-compose.vps.yml logs -f backend"
echo ""
sleep 3

k6_run authenticated-mixed 100 3m \
    | tee "$RESULT_FILE"

echo ""
echo "✅ Test complete!"
echo "Results saved to: $RESULT_FILE"
echo ""
echo "Analysis for 1 vCPU / 2GB RAM:"
echo "  📊 Check error rate - should be < 5%"
echo "  📊 Check if throughput plateaued (saturation point)"
echo "  📊 Check CPU usage - likely at 95-100%"
echo "  📊 P99 latency - acceptable if < 200ms"
echo ""
echo "If errors > 5%, consider:"
echo "  - Reducing concurrent connections"
echo "  - Optimizing database queries"
echo "  - Adding horizontal scaling"
echo ""
