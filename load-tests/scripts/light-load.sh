#!/bin/bash
# Light load test - Simulates normal usage
# Target: 100 req/s, P99 < 50ms

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
echo "🟢 KoproGo Load Test - LIGHT LOAD"
echo "========================================="
echo "Base URL: $BASE_URL"
echo "Duration: 2 minutes"
echo "Target: 100 req/s"
echo ""

# k6 local, ou Docker pour l'image épinglée
source "${SCRIPT_DIR}/k6.sh"
k6_verifier

# Check if API is up
if ! curl -f -s "$BASE_URL/api/v1/health" > /dev/null; then
    echo "❌ Error: API is not responding"
    exit 1
fi

echo "Test parameters:"
echo "- Threads: 2"
echo "- Connections: 10"
echo "- Duration: 2 minutes"
echo ""

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_FILE="$RESULTS_DIR/light-load_${TIMESTAMP}.txt"

echo "Starting test..."
echo ""

k6_run authenticated-mixed 10 2m \
    | tee "$RESULT_FILE"

echo ""
echo "✅ Test complete!"
echo "Results saved to: $RESULT_FILE"
echo ""
echo "Expected results for 1 vCPU / 2GB RAM:"
echo "  ✅ P99 latency: < 50ms"
echo "  ✅ Throughput: > 100 req/s"
echo "  ✅ Error rate: < 0.1%"
echo ""
