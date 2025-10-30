#!/bin/bash
# Light load test - For REMOTE execution from client machine
# Run this from your local machine or a separate VPS client
# Target: 100 req/s, P99 < 50ms

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
echo "🟢 KoproGo REMOTE Load Test - LIGHT"
echo "========================================="
echo "Target: $BASE_URL"
echo "Duration: 2 minutes"
echo "Expected: 100 req/s"
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

# Check if wrk is installed
if ! command -v wrk &> /dev/null; then
    echo "❌ Error: wrk is not installed"
    echo ""
    echo "Install wrk:"
    echo "  Ubuntu/Debian: sudo apt-get install wrk"
    echo "  macOS: brew install wrk"
    exit 1
fi

# Check if API is accessible
echo "Checking API connectivity..."
if ! curl -f -s -m 10 "$BASE_URL/api/v1/health" > /dev/null; then
    echo "❌ Error: Cannot reach API at $BASE_URL"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Check DNS: nslookup $(echo $BASE_URL | sed 's|https\?://||' | cut -d'/' -f1)"
    echo "  2. Check firewall on VPS"
    echo "  3. Check Traefik is running: ssh user@vps 'docker ps | grep traefik'"
    exit 1
fi
echo "✅ API is reachable"
echo ""

# Measure baseline latency
echo "Measuring network latency..."
LATENCY=$(curl -o /dev/null -s -w '%{time_total}\n' "$BASE_URL/api/v1/health" | awk '{print $1 * 1000}')
echo "📡 Baseline latency: ${LATENCY}ms"
echo ""

if (( $(echo "$LATENCY > 100" | bc -l) )); then
    echo "⚠️  WARNING: High network latency detected (${LATENCY}ms)"
    echo "⚠️  Test results will include this latency overhead"
    echo ""
fi

echo "Test parameters:"
echo "- Threads: 2"
echo "- Connections: 10"
echo "- Duration: 2 minutes"
echo "- Scenario: Mixed workload"
echo ""

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_FILE="$RESULTS_DIR/remote-light-load_${TIMESTAMP}.txt"

echo "Starting remote test..."
echo "💡 TIP: Monitor server with: ssh user@vps 'cd /opt/koprogo/load-tests && ./monitor-server.sh 120'"
echo ""
sleep 2

wrk -t2 -c10 -d2m \
    --latency \
    -s "${SCRIPT_DIR}/../lua/mixed.lua" \
    "$BASE_URL" \
    | tee "$RESULT_FILE"

echo ""
echo "✅ Remote test complete!"
echo "Results saved to: $RESULT_FILE"
echo ""
echo "Expected results (1 vCPU / 2GB RAM + network latency):"
echo "  ✅ P99 latency: < 50ms + ${LATENCY}ms network = < $((50 + ${LATENCY%.*}))ms"
echo "  ✅ Throughput: > 100 req/s"
echo "  ✅ Error rate: < 0.1%"
echo ""
echo "Next steps:"
echo "  1. Check server monitoring logs on the VPS"
echo "  2. Review detailed results: cat $RESULT_FILE"
echo "  3. Compare with previous runs"
echo ""
