#!/bin/bash
# Spike test - Sudden traffic spike
# Tests system recovery and resilience

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
echo "⚡ KoproGo Load Test - SPIKE TEST"
echo "========================================="
echo "Base URL: $BASE_URL"
echo "Duration: ~5 minutes total"
echo ""

# Check dependencies
# k6 local, ou Docker pour l'image épinglée
source "${SCRIPT_DIR}/k6.sh"
k6_verifier

if ! curl -f -s "$BASE_URL/api/v1/health" > /dev/null; then
    echo "❌ Error: API is not responding"
    exit 1
fi

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_FILE="$RESULTS_DIR/spike-test_${TIMESTAMP}.txt"

{
    echo "========================================="
    echo "SPIKE TEST - $(date)"
    echo "========================================="
    echo ""

    # Phase 1: Baseline (30s)
    echo "Phase 1/5: Baseline load (30s)"
    echo "- Connections: 10"
    k6_run mixed 10 30s
    echo ""
    sleep 5

    # Phase 2: Ramp up (30s)
    echo "Phase 2/5: Ramp up (30s)"
    echo "- Connections: 50"
    k6_run mixed 50 30s
    echo ""
    sleep 5

    # Phase 3: SPIKE! (1 minute)
    echo "Phase 3/5: SPIKE! (1 minute)"
    echo "- Connections: 200"
    echo "⚡ Simulating sudden traffic surge..."
    k6_run mixed 200 1m
    echo ""
    sleep 5

    # Phase 4: Recovery (30s)
    echo "Phase 4/5: Recovery (30s)"
    echo "- Connections: 50"
    k6_run mixed 50 30s
    echo ""
    sleep 5

    # Phase 5: Back to baseline (30s)
    echo "Phase 5/5: Back to baseline (30s)"
    echo "- Connections: 10"
    k6_run mixed 10 30s
    echo ""

    echo "========================================="
    echo "SPIKE TEST COMPLETE"
    echo "========================================="

} | tee "$RESULT_FILE"

echo ""
echo "✅ Spike test complete!"
echo "Results saved to: $RESULT_FILE"
echo ""
echo "Analysis checklist:"
echo "  1. Did the system handle the spike without crashing?"
echo "  2. What was the error rate during the spike?"
echo "  3. Did performance recover after the spike?"
echo "  4. How long did recovery take?"
echo "  5. Check logs for any errors or warnings"
echo ""
echo "Expected behavior for 1 vCPU / 2GB RAM:"
echo "  ✅ System remains stable (no crashes)"
echo "  ⚠️  High error rate during spike is acceptable (< 10%)"
echo "  ✅ Performance recovers within 30 seconds"
echo "  ✅ Baseline performance restored in Phase 5"
echo ""
