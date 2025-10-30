#!/bin/bash
# Medium load test - For REMOTE execution from client machine
# Run this from your local machine or a separate VPS client
# Target: 500 req/s, P99 < 100ms

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"

mkdir -p "$RESULTS_DIR"

# Default to api.koprogo.com if BASE_URL not set
if [ -z "$BASE_URL" ]; then
    BASE_URL="https://api.koprogo.com"
    echo "ℹ️  Using default URL: $BASE_URL"
    echo ""
    echo "💡 To use a different URL:"
    echo "   export BASE_URL=https://your-domain.com"
    echo "   $0"
    echo ""
fi

# Ensure BASE_URL has protocol (http:// or https://)
if [[ ! "$BASE_URL" =~ ^https?:// ]]; then
    echo "⚠️  BASE_URL missing protocol, adding https://"
    BASE_URL="https://$BASE_URL"
fi

echo "========================================="
echo "🟡 KoproGo REMOTE Load Test - MEDIUM"
echo "========================================="
echo "Target: $BASE_URL"
echo "Duration: 5 minutes"
echo "Expected: 500 req/s"
echo ""

# Verify we're not running on localhost
if [[ "$BASE_URL" == *"localhost"* ]] || [[ "$BASE_URL" == *"127.0.0.1"* ]]; then
    echo "⚠️  WARNING: You're targeting localhost!"
    echo "⚠️  This should be run from a REMOTE machine."
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check dependencies
if ! command -v wrk &> /dev/null; then
    echo "❌ Error: wrk is not installed"
    exit 1
fi

# Check API connectivity
echo "Checking API connectivity..."
if ! curl -f -s -m 10 "$BASE_URL/api/v1/health" > /dev/null; then
    echo "❌ Error: Cannot reach API at $BASE_URL"
    exit 1
fi
echo "✅ API is reachable"
echo ""

# Measure baseline latency
echo "Measuring network latency..."
LATENCY=$(curl -o /dev/null -s -w '%{time_total}\n' "$BASE_URL/api/v1/health" | awk '{print $1 * 1000}')
echo "📡 Baseline latency: ${LATENCY}ms"
echo ""

echo "Test parameters:"
echo "- Threads: 4"
echo "- Connections: 50"
echo "- Duration: 5 minutes"
echo "- Scenario: Mixed workload"
echo ""

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_FILE="$RESULTS_DIR/remote-medium-load_${TIMESTAMP}.txt"

# Run warmup first
echo "Running warmup (10s)..."
wrk -t2 -c5 -d10s --latency "$BASE_URL/api/v1/health" > /dev/null 2>&1
echo "✅ Warmup complete"
echo ""

echo "Starting medium load test..."
echo "💡 IMPORTANT: Monitor your VPS server now!"
echo ""
echo "On the VPS, run:"
echo "  ssh user@vps-ip"
echo "  cd /opt/koprogo/load-tests"
echo "  ./monitor-server.sh 300"
echo ""
echo "Test starting in 5 seconds..."
sleep 5

wrk -t4 -c50 -d5m \
    --latency \
    -s "${SCRIPT_DIR}/../lua/mixed.lua" \
    "$BASE_URL" \
    | tee "$RESULT_FILE"

echo ""
echo "✅ Remote test complete!"
echo "Results saved to: $RESULT_FILE"
echo ""
echo "Expected results (1 vCPU / 2GB RAM + network latency):"
echo "  ✅ P99 latency: < 100ms + ${LATENCY}ms network = < $((100 + ${LATENCY%.*}))ms"
echo "  ✅ Throughput: > 500 req/s"
echo "  ✅ Error rate: < 0.5%"
echo "  ⚠️  CPU usage on VPS: 70-80%"
echo ""
echo "Next steps:"
echo "  1. SSH to VPS and check: docker stats"
echo "  2. Review monitoring logs on VPS"
echo "  3. Check for errors: docker compose -f docker-compose.vps.yml logs backend | grep ERROR"
echo ""
