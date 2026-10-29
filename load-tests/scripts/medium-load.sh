#!/bin/bash
# Medium load test - Simulates peak usage
# Target: 500 req/s, P99 < 100ms

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
echo "🟡 KoproGo Load Test - MEDIUM LOAD"
echo "========================================="
echo "Base URL: $BASE_URL"
echo "Duration: 5 minutes"
echo "Target: 500 req/s"
echo ""

# Check dependencies
# k6 local, ou Docker pour l'image épinglée
source "${SCRIPT_DIR}/k6.sh"
k6_verifier

if ! curl -f -s "$BASE_URL/api/v1/health" > /dev/null; then
    echo "❌ Error: API is not responding"
    exit 1
fi

echo "Test parameters:"
echo "- Threads: 4"
echo "- Connections: 50"
echo "- Duration: 5 minutes"
echo ""

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_FILE="$RESULTS_DIR/medium-load_${TIMESTAMP}.txt"

# Run warmup first
echo "Running warmup..."
k6_run health 5 10s > /dev/null 2>&1
echo "✅ Warmup complete"
echo ""

echo "Starting medium load test..."
echo "⚠️  Monitor resources: watch -n 1 'docker stats --no-stream'"
echo ""

k6_run authenticated-mixed 50 5m \
    | tee "$RESULT_FILE"

echo ""
echo "✅ Test complete!"
echo "Results saved to: $RESULT_FILE"
echo ""
echo "Expected results for 1 vCPU / 2GB RAM:"
echo "  ✅ P99 latency: < 100ms"
echo "  ✅ Throughput: > 500 req/s"
echo "  ✅ Error rate: < 0.5%"
echo "  ⚠️  CPU usage: 60-80%"
echo ""
