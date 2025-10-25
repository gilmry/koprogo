#!/bin/bash
# Master script - Run all load tests in sequence
# Generates comprehensive performance report

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/results"

# Default to api.koprogo.com for remote testing
if [ -z "$BASE_URL" ]; then
    BASE_URL="https://api.koprogo.com"
    echo "ℹ️  Using default URL: $BASE_URL"
    echo ""
    echo "💡 For api2: export BASE_URL=https://api2.koprogo.com"
    echo "💡 For local: export BASE_URL=http://localhost:8080"
    echo ""
fi

# Export BASE_URL so child scripts inherit it
export BASE_URL

mkdir -p "$RESULTS_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$RESULTS_DIR/full-report_${TIMESTAMP}.txt"

echo "=========================================" | tee "$REPORT_FILE"
echo "🚀 KoproGo Complete Load Test Suite" | tee -a "$REPORT_FILE"
echo "=========================================" | tee -a "$REPORT_FILE"
echo "Started at: $(date)" | tee -a "$REPORT_FILE"
echo "Base URL: $BASE_URL" | tee -a "$REPORT_FILE"
echo "" | tee -a "$REPORT_FILE"

# Check prerequisites
echo "Checking prerequisites..." | tee -a "$REPORT_FILE"

if ! command -v wrk &> /dev/null; then
    echo "❌ Error: wrk is not installed" | tee -a "$REPORT_FILE"
    echo "Install with: sudo apt-get install wrk" | tee -a "$REPORT_FILE"
    exit 1
fi

if ! curl -f -s "$BASE_URL/api/v1/health" > /dev/null; then
    echo "❌ Error: API is not responding at $BASE_URL" | tee -a "$REPORT_FILE"
    exit 1
fi

echo "✅ All prerequisites met" | tee -a "$REPORT_FILE"
echo "" | tee -a "$REPORT_FILE"

# Record system info
echo "System Information:" | tee -a "$REPORT_FILE"
echo "-------------------" | tee -a "$REPORT_FILE"
echo "OS: $(uname -a)" | tee -a "$REPORT_FILE"
echo "CPU cores: $(nproc)" | tee -a "$REPORT_FILE"
echo "Total RAM: $(free -h | awk '/^Mem:/ {print $2}')" | tee -a "$REPORT_FILE"
echo "" | tee -a "$REPORT_FILE"

# Record Docker info
echo "Docker Container Status:" | tee -a "$REPORT_FILE"
echo "------------------------" | tee -a "$REPORT_FILE"
docker ps --format "table {{.Names}}\t{{.Status}}" | tee -a "$REPORT_FILE"
echo "" | tee -a "$REPORT_FILE"

# Test sequence
TESTS=(
    "warmup:Warmup"
    "light-load:Light Load (2 min)"
    "medium-load:Medium Load (5 min)"
    "heavy-load:Heavy Load (3 min)"
)

TOTAL_TESTS=${#TESTS[@]}
CURRENT_TEST=0

echo "Test Suite:" | tee -a "$REPORT_FILE"
echo "-----------" | tee -a "$REPORT_FILE"
for test in "${TESTS[@]}"; do
    test_name="${test%%:*}"
    test_desc="${test#*:}"
    echo "  - $test_desc" | tee -a "$REPORT_FILE"
done
echo "" | tee -a "$REPORT_FILE"

read -p "Run all tests? This will take ~15 minutes. (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Tests cancelled." | tee -a "$REPORT_FILE"
    exit 0
fi

echo "" | tee -a "$REPORT_FILE"

# Run tests
for test in "${TESTS[@]}"; do
    CURRENT_TEST=$((CURRENT_TEST + 1))
    test_name="${test%%:*}"
    test_desc="${test#*:}"

    echo "=========================================" | tee -a "$REPORT_FILE"
    echo "Test $CURRENT_TEST/$TOTAL_TESTS: $test_desc" | tee -a "$REPORT_FILE"
    echo "=========================================" | tee -a "$REPORT_FILE"
    echo "Started at: $(date)" | tee -a "$REPORT_FILE"
    echo "" | tee -a "$REPORT_FILE"

    # Run test and capture output
    TEST_START=$(date +%s)
    bash "${SCRIPT_DIR}/scripts/${test_name}.sh" 2>&1 | tee -a "$REPORT_FILE"
    TEST_END=$(date +%s)
    TEST_DURATION=$((TEST_END - TEST_START))

    echo "" | tee -a "$REPORT_FILE"
    echo "Completed in ${TEST_DURATION}s" | tee -a "$REPORT_FILE"
    echo "" | tee -a "$REPORT_FILE"

    # Wait between tests (except after last test)
    if [ $CURRENT_TEST -lt $TOTAL_TESTS ]; then
        echo "Waiting 30s before next test..." | tee -a "$REPORT_FILE"
        sleep 30
    fi
done

# Generate summary
echo "=========================================" | tee -a "$REPORT_FILE"
echo "📊 Test Suite Complete" | tee -a "$REPORT_FILE"
echo "=========================================" | tee -a "$REPORT_FILE"
echo "Finished at: $(date)" | tee -a "$REPORT_FILE"
echo "" | tee -a "$REPORT_FILE"

echo "Summary:" | tee -a "$REPORT_FILE"
echo "--------" | tee -a "$REPORT_FILE"
echo "  Total tests: $TOTAL_TESTS" | tee -a "$REPORT_FILE"
echo "  Results directory: $RESULTS_DIR" | tee -a "$REPORT_FILE"
echo "  Full report: $REPORT_FILE" | tee -a "$REPORT_FILE"
echo "" | tee -a "$REPORT_FILE"

echo "Next steps:" | tee -a "$REPORT_FILE"
echo "1. Review full report: cat $REPORT_FILE" | tee -a "$REPORT_FILE"
echo "2. Check individual test results in: $RESULTS_DIR" | tee -a "$REPORT_FILE"
echo "3. Analyze container stats during tests" | tee -a "$REPORT_FILE"
echo "4. Review application logs for errors" | tee -a "$REPORT_FILE"
echo "" | tee -a "$REPORT_FILE"

echo "Performance targets for 1 vCPU / 2GB RAM:" | tee -a "$REPORT_FILE"
echo "  ✅ Light load: P99 < 50ms, > 100 req/s" | tee -a "$REPORT_FILE"
echo "  ✅ Medium load: P99 < 100ms, > 500 req/s" | tee -a "$REPORT_FILE"
echo "  ⚠️  Heavy load: P99 < 200ms, errors < 5%" | tee -a "$REPORT_FILE"
echo "" | tee -a "$REPORT_FILE"

echo "✅ All tests complete! Report saved to: $REPORT_FILE"
