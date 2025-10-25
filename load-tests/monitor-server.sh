#!/bin/bash
# Script de monitoring côté serveur pendant les tests de charge
# À lancer sur le VPS pendant que les tests tournent depuis la machine cliente

set -e

DURATION=${1:-300}  # Durée en secondes (défaut: 5 minutes)
INTERVAL=${2:-5}    # Intervalle entre les mesures (défaut: 5s)

RESULTS_DIR="./monitoring-results"
mkdir -p "$RESULTS_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="$RESULTS_DIR/server-monitoring_${TIMESTAMP}.log"

echo "========================================="
echo "📊 KoproGo Server Monitoring"
echo "========================================="
echo "Duration: ${DURATION}s ($(($DURATION / 60)) minutes)"
echo "Interval: ${INTERVAL}s"
echo "Output: $OUTPUT_FILE"
echo ""
echo "Press Ctrl+C to stop monitoring"
echo ""

# Start time
START_TIME=$(date +%s)
END_TIME=$((START_TIME + DURATION))

# Header
{
    echo "========================================="
    echo "Server Monitoring Started"
    echo "========================================="
    echo "Start time: $(date)"
    echo "Duration: ${DURATION}s"
    echo ""
    echo "System Info:"
    echo "  OS: $(uname -s)"
    echo "  Kernel: $(uname -r)"
    echo "  CPU cores: $(nproc)"
    echo "  Total RAM: $(free -h | awk '/^Mem:/ {print $2}')"
    echo ""
} | tee "$OUTPUT_FILE"

# Cleanup function
cleanup() {
    echo ""
    echo "=========================================" | tee -a "$OUTPUT_FILE"
    echo "Monitoring Stopped" | tee -a "$OUTPUT_FILE"
    echo "=========================================" | tee -a "$OUTPUT_FILE"
    echo "End time: $(date)" | tee -a "$OUTPUT_FILE"
    ACTUAL_DURATION=$(($(date +%s) - START_TIME))
    echo "Actual duration: ${ACTUAL_DURATION}s" | tee -a "$OUTPUT_FILE"
    echo "" | tee -a "$OUTPUT_FILE"
    echo "Results saved to: $OUTPUT_FILE" | tee -a "$OUTPUT_FILE"
    echo ""
    echo "📊 Quick Summary:"
    echo ""
    echo "Docker Stats Summary:"
    grep "koprogo-backend" "$OUTPUT_FILE" | tail -5
    echo ""
    echo "Full report: cat $OUTPUT_FILE"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Main monitoring loop
ITERATION=0

while [ $(date +%s) -lt $END_TIME ]; do
    ITERATION=$((ITERATION + 1))
    CURRENT_TIME=$(date +%H:%M:%S)
    ELAPSED=$(($(date +%s) - START_TIME))

    {
        echo "================================================"
        echo "[$CURRENT_TIME] Iteration $ITERATION (Elapsed: ${ELAPSED}s)"
        echo "================================================"
        echo ""

        # Docker stats
        echo "Docker Containers:"
        echo "-------------------"
        docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.NetIO}}"
        echo ""

        # System resources
        echo "System Resources:"
        echo "-------------------"
        echo "RAM: $(free -h | awk '/^Mem:/ {print "Used: "$3" / "$2" ("$3/$2*100"%)"}')"
        echo "Swap: $(free -h | awk '/^Swap:/ {print "Used: "$3" / "$2}')"
        echo "Load Average: $(uptime | awk -F'load average:' '{print $2}')"
        echo ""

        # Disk I/O
        echo "Disk I/O:"
        echo "-------------------"
        iostat -x 1 1 | tail -n +4 | head -5 || echo "iostat not available"
        echo ""

        # Database connections
        echo "PostgreSQL Connections:"
        echo "-------------------"
        # Try both locations for docker-compose file
        COMPOSE_FILE=""
        if [ -f "docker-compose.vps.yml" ]; then
            COMPOSE_FILE="docker-compose.vps.yml"
        elif [ -f "../deploy/production/docker-compose.yml" ]; then
            COMPOSE_FILE="../deploy/production/docker-compose.yml"
        elif [ -f "/root/koprogo/deploy/production/docker-compose.yml" ]; then
            COMPOSE_FILE="/root/koprogo/deploy/production/docker-compose.yml"
        fi

        if [ -n "$COMPOSE_FILE" ]; then
            docker compose -f "$COMPOSE_FILE" exec -T postgres \
                psql -U koprogo -d koprogo_db -t -c "
                SELECT
                    'Total: ' || count(*) ||
                    ' | Active: ' || count(*) FILTER (WHERE state = 'active') ||
                    ' | Idle: ' || count(*) FILTER (WHERE state = 'idle')
                FROM pg_stat_activity
                WHERE datname = 'koprogo_db';
                " 2>/dev/null || echo "Unable to connect to PostgreSQL"
        else
            echo "Docker compose file not found"
        fi
        echo ""

        # Recent errors in logs (last 10 seconds)
        echo "Recent Errors (last ${INTERVAL}s):"
        echo "-------------------"
        ERROR_COUNT=$(docker compose -f docker-compose.vps.yml logs --since=${INTERVAL}s backend 2>/dev/null | grep -i -E "error|panic|fatal" | wc -l || echo "0")
        if [ "$ERROR_COUNT" -gt 0 ]; then
            echo "⚠️  Found $ERROR_COUNT errors:"
            docker compose -f docker-compose.vps.yml logs --since=${INTERVAL}s backend 2>/dev/null | grep -i -E "error|panic|fatal" | tail -5
        else
            echo "✅ No errors detected"
        fi
        echo ""

        # Network connections
        echo "Network Connections:"
        echo "-------------------"
        # Check both Traefik (443) and Backend (8080) ports
        TRAEFIK_ESTAB=$(ss -tan | grep ESTAB | grep :443 | wc -l)
        BACKEND_ESTAB=$(ss -tan | grep ESTAB | grep :8080 | wc -l)
        TRAEFIK_TW=$(ss -tan | grep TIME-WAIT | grep :443 | wc -l)
        BACKEND_TW=$(ss -tan | grep TIME-WAIT | grep :8080 | wc -l)
        echo "Traefik :443  - Established: $TRAEFIK_ESTAB | Time-Wait: $TRAEFIK_TW"
        echo "Backend :8080 - Established: $BACKEND_ESTAB | Time-Wait: $BACKEND_TW"
        echo ""

    } | tee -a "$OUTPUT_FILE"

    # Sleep until next iteration
    sleep "$INTERVAL"
done

# If we reach here (duration expired), call cleanup
cleanup
