#!/bin/bash
################################################################################
# KoproGo VPS Monitoring Script
# Tracks system resources and alerts when thresholds are exceeded
################################################################################

set -euo pipefail

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Load configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_FILE="${SCRIPT_DIR}/../config/thresholds.env"

# Default thresholds (can be overridden by config file)
RAM_WARNING_THRESHOLD=75
RAM_CRITICAL_THRESHOLD=85
CPU_WARNING_THRESHOLD=70
CPU_CRITICAL_THRESHOLD=85
DISK_WARNING_THRESHOLD=70
DISK_CRITICAL_THRESHOLD=80
LOAD_WARNING_THRESHOLD=1.5
LOAD_CRITICAL_THRESHOLD=2.0

# Load custom thresholds if config exists
if [[ -f "$CONFIG_FILE" ]]; then
    source "$CONFIG_FILE"
fi

# Output file for JSON metrics
OUTPUT_JSON="${SCRIPT_DIR}/../logs/metrics_$(date +%Y%m%d_%H%M%S).json"
mkdir -p "$(dirname "$OUTPUT_JSON")"

################################################################################
# Functions
################################################################################

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_critical() {
    echo -e "${RED}[CRITICAL]${NC} $1"
}

check_threshold() {
    local value=$1
    local warning=$2
    local critical=$3
    local metric_name=$4

    if (( $(echo "$value >= $critical" | bc -l) )); then
        log_critical "$metric_name: ${value}% (threshold: ${critical}%)"
        return 2
    elif (( $(echo "$value >= $warning" | bc -l) )); then
        log_warning "$metric_name: ${value}% (threshold: ${warning}%)"
        return 1
    else
        log_info "$metric_name: ${value}% - OK"
        return 0
    fi
}

get_ram_usage() {
    free | grep Mem | awk '{printf "%.2f", ($3/$2) * 100.0}'
}

get_cpu_usage() {
    top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1}'
}

get_disk_usage() {
    df -h / | awk 'NR==2 {print $5}' | sed 's/%//'
}

get_load_average() {
    uptime | awk -F'load average:' '{print $2}' | awk '{print $1}' | sed 's/,//'
}

get_postgres_connections() {
    if command -v psql &> /dev/null; then
        local db_url="${DATABASE_URL:-postgresql://koprogo:koprogo123@localhost:5432/koprogo_db}"
        psql "$db_url" -t -c "SELECT count(*) FROM pg_stat_activity WHERE datname = 'koprogo_db';" 2>/dev/null || echo "0"
    else
        echo "N/A"
    fi
}

get_postgres_db_size() {
    if command -v psql &> /dev/null; then
        local db_url="${DATABASE_URL:-postgresql://koprogo:koprogo123@localhost:5432/koprogo_db}"
        psql "$db_url" -t -c "SELECT pg_size_pretty(pg_database_size('koprogo_db'));" 2>/dev/null | xargs || echo "N/A"
    else
        echo "N/A"
    fi
}

################################################################################
# Main Monitoring
################################################################################

echo "========================================"
echo "KoproGo VPS Monitoring Report"
echo "Timestamp: $(date '+%Y-%m-%d %H:%M:%S')"
echo "========================================"
echo ""

# Collect metrics
RAM_USAGE=$(get_ram_usage)
CPU_USAGE=$(get_cpu_usage)
DISK_USAGE=$(get_disk_usage)
LOAD_AVG=$(get_load_average)
PG_CONNECTIONS=$(get_postgres_connections)
DB_SIZE=$(get_postgres_db_size)

# System Resources
echo "=== System Resources ==="
check_threshold "$RAM_USAGE" "$RAM_WARNING_THRESHOLD" "$RAM_CRITICAL_THRESHOLD" "RAM Usage"
RAM_STATUS=$?

check_threshold "$CPU_USAGE" "$CPU_WARNING_THRESHOLD" "$CPU_CRITICAL_THRESHOLD" "CPU Usage"
CPU_STATUS=$?

check_threshold "$DISK_USAGE" "$DISK_WARNING_THRESHOLD" "$DISK_CRITICAL_THRESHOLD" "Disk Usage"
DISK_STATUS=$?

# Load Average
echo ""
echo "=== Load Average ==="
LOAD_STATUS=0
if (( $(echo "$LOAD_AVG >= $LOAD_CRITICAL_THRESHOLD" | bc -l) )); then
    log_critical "Load Average: $LOAD_AVG (threshold: $LOAD_CRITICAL_THRESHOLD)"
    LOAD_STATUS=2
elif (( $(echo "$LOAD_AVG >= $LOAD_WARNING_THRESHOLD" | bc -l) )); then
    log_warning "Load Average: $LOAD_AVG (threshold: $LOAD_WARNING_THRESHOLD)"
    LOAD_STATUS=1
else
    log_info "Load Average: $LOAD_AVG - OK"
fi

# PostgreSQL
echo ""
echo "=== PostgreSQL ==="
log_info "Active connections: $PG_CONNECTIONS / 10 (pool max)"
log_info "Database size: $DB_SIZE"

# Memory breakdown
echo ""
echo "=== Memory Breakdown ==="
echo "$(free -h)"

# Disk space
echo ""
echo "=== Disk Space ==="
df -h /

# Capacity estimation
echo ""
echo "=== Capacity Estimation ==="
if [[ "$DB_SIZE" != "N/A" ]]; then
    # Extract size in bytes (rough estimation)
    log_info "Current database size: $DB_SIZE"
    log_info "Estimated capacity at current growth: See capacity calculator"
else
    log_info "Database metrics not available (PostgreSQL not accessible)"
fi

# Overall status
echo ""
echo "=== Overall Status ==="
OVERALL_STATUS="OK"
if [[ $RAM_STATUS -eq 2 ]] || [[ $CPU_STATUS -eq 2 ]] || [[ $DISK_STATUS -eq 2 ]] || [[ $LOAD_STATUS -eq 2 ]]; then
    OVERALL_STATUS="CRITICAL"
    log_critical "System requires immediate attention!"
    echo ""
    echo "Recommended actions:"
    [[ $RAM_STATUS -eq 2 ]] && echo "  - Upgrade VPS RAM or optimize application memory usage"
    [[ $CPU_STATUS -eq 2 ]] && echo "  - Upgrade VPS CPU or optimize queries/code"
    [[ $DISK_STATUS -eq 2 ]] && echo "  - Clean up logs/backups or upgrade disk space"
    [[ $LOAD_STATUS -eq 2 ]] && echo "  - Investigate high load (check running processes)"
elif [[ $RAM_STATUS -eq 1 ]] || [[ $CPU_STATUS -eq 1 ]] || [[ $DISK_STATUS -eq 1 ]] || [[ $LOAD_STATUS -eq 1 ]]; then
    OVERALL_STATUS="WARNING"
    log_warning "System approaching capacity limits"
else
    log_info "All systems operating normally"
fi

# Export to JSON
cat > "$OUTPUT_JSON" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "status": "$OVERALL_STATUS",
  "metrics": {
    "ram": {
      "usage_percent": $RAM_USAGE,
      "status": $([ $RAM_STATUS -eq 0 ] && echo '"ok"' || [ $RAM_STATUS -eq 1 ] && echo '"warning"' || echo '"critical"')
    },
    "cpu": {
      "usage_percent": $CPU_USAGE,
      "status": $([ $CPU_STATUS -eq 0 ] && echo '"ok"' || [ $CPU_STATUS -eq 1 ] && echo '"warning"' || echo '"critical"')
    },
    "disk": {
      "usage_percent": $DISK_USAGE,
      "status": $([ $DISK_STATUS -eq 0 ] && echo '"ok"' || [ $DISK_STATUS -eq 1 ] && echo '"warning"' || echo '"critical"')
    },
    "load_average": {
      "value": $LOAD_AVG,
      "status": $([ $LOAD_STATUS -eq 0 ] && echo '"ok"' || [ $LOAD_STATUS -eq 1 ] && echo '"warning"' || echo '"critical"')
    },
    "postgresql": {
      "connections": "$PG_CONNECTIONS",
      "database_size": "$DB_SIZE"
    }
  },
  "thresholds": {
    "ram_warning": $RAM_WARNING_THRESHOLD,
    "ram_critical": $RAM_CRITICAL_THRESHOLD,
    "cpu_warning": $CPU_WARNING_THRESHOLD,
    "cpu_critical": $CPU_CRITICAL_THRESHOLD,
    "disk_warning": $DISK_WARNING_THRESHOLD,
    "disk_critical": $DISK_CRITICAL_THRESHOLD
  }
}
EOF

echo ""
log_info "Metrics exported to: $OUTPUT_JSON"
echo "========================================"

# Exit with appropriate code
if [[ "$OVERALL_STATUS" == "CRITICAL" ]]; then
    exit 2
elif [[ "$OVERALL_STATUS" == "WARNING" ]]; then
    exit 1
else
    exit 0
fi
