#!/bin/bash
# Soak test - Long-duration stability test
# Tests for memory leaks and performance degradation over time
# Duration: 30 minutes

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
echo "⏱️  KoproGo Load Test - SOAK TEST"
echo "========================================="
echo "Base URL: $BASE_URL"
echo "Duration: 30 minutes"
echo "Target: Sustained moderate load"
echo ""
echo "⚠️  This test will run for 30 minutes!"
echo "⚠️  Use this to detect memory leaks and degradation."
echo ""

# Check dependencies
# k6 local, ou Docker pour l'image épinglée
source "${SCRIPT_DIR}/k6.sh"
k6_verifier

if ! curl -f -s "$BASE_URL/api/v1/health" > /dev/null; then
    echo "❌ Error: API is not responding"
    exit 1
fi

read -p "Continue with 30-minute soak test? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Test cancelled."
    exit 0
fi

echo ""
echo "Test parameters:"
echo "- Threads: 2"
echo "- Connections: 25"
echo "- Duration: 30 minutes"
echo "- Target: ~250 req/s sustained"
echo ""

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_FILE="$RESULTS_DIR/soak-test_${TIMESTAMP}.txt"
MEMORY_LOG="$RESULTS_DIR/soak-test_memory_${TIMESTAMP}.log"

# Start memory monitoring in background
echo "Starting memory monitoring..."
(
    while true; do
        echo "$(date +%Y-%m-%d\ %H:%M:%S)" >> "$MEMORY_LOG"
        docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}" >> "$MEMORY_LOG"
        echo "" >> "$MEMORY_LOG"
        sleep 60  # Log every minute
    done
) &
MONITOR_PID=$!

# Cleanup function
cleanup() {
    echo ""
    echo "Stopping memory monitoring..."
    kill $MONITOR_PID 2>/dev/null || true
}
trap cleanup EXIT

# Run warmup
echo "Running warmup..."
k6_run health 5 10s > /dev/null 2>&1
echo "✅ Warmup complete"
echo ""

echo "Starting soak test..."
echo "Test will complete at: $(date -d '+30 minutes' '+%H:%M:%S')"
echo ""
echo "Monitor in real-time:"
echo "  Terminal 2: watch -n 1 'docker stats --no-stream'"
echo "  Terminal 3: tail -f $MEMORY_LOG"
echo ""
sleep 3

START_TIME=$(date +%s)

k6_run mixed 25 30m \
    | tee "$RESULT_FILE"

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo "✅ Soak test complete!"
echo "Results saved to: $RESULT_FILE"
echo "Memory log saved to: $MEMORY_LOG"
echo "Actual duration: $((DURATION / 60)) minutes"
echo ""
echo "Analysis checklist:"
echo "  1. Check memory usage trend in: $MEMORY_LOG"
echo "  2. Compare first 5 minutes vs last 5 minutes performance"
echo "  3. Look for memory leaks (steadily increasing RAM)"
echo "  4. Verify error rate remained low throughout"
echo "  5. Check logs for any warnings/errors"
echo ""
echo "Expected results for 1 vCPU / 2GB RAM:"
echo "  ✅ Stable memory usage (no continuous growth)"
echo "  ✅ Consistent latency (P99 < 100ms)"
echo "  ✅ Sustained throughput (~250 req/s)"
echo "  ✅ Error rate < 0.5%"
echo ""
echo "If memory grows continuously:"
echo "  - Check for connection leaks"
echo "  - Review database connection pool settings"
echo "  - Check application logs for errors"
echo ""
