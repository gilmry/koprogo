#!/bin/bash
################################################################################
# KoproGo PostgreSQL Performance Monitoring
# Executes diagnostic queries and generates reports
################################################################################

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SQL_FILE="${SCRIPT_DIR}/postgres_queries.sql"
OUTPUT_DIR="${SCRIPT_DIR}/../logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Database connection
DB_URL="${DATABASE_URL:-postgresql://koprogo:koprogo123@localhost:5432/koprogo_db}"

mkdir -p "$OUTPUT_DIR"

################################################################################
# Functions
################################################################################

print_header() {
    echo -e "${BLUE}========================================"
    echo -e "$1"
    echo -e "========================================${NC}"
}

run_query() {
    local query_name=$1
    local query=$2

    echo -e "${GREEN}Running: $query_name${NC}"
    psql "$DB_URL" -c "$query" 2>&1 || echo -e "${YELLOW}Query failed or returned no results${NC}"
    echo ""
}

################################################################################
# Main
################################################################################

print_header "KoproGo PostgreSQL Performance Report\nTimestamp: $(date '+%Y-%m-%d %H:%M:%S')"

# Check if PostgreSQL is accessible
if ! psql "$DB_URL" -c "SELECT version();" &>/dev/null; then
    echo -e "${YELLOW}WARNING: Cannot connect to PostgreSQL${NC}"
    echo "Database URL: $DB_URL"
    echo "Please check your DATABASE_URL environment variable and database status"
    exit 1
fi

# Enable pg_stat_statements if not already enabled
echo -e "${YELLOW}Note: Some queries require pg_stat_statements extension${NC}"
psql "$DB_URL" -c "CREATE EXTENSION IF NOT EXISTS pg_stat_statements;" 2>/dev/null || true
echo ""

# Query 1: Top 10 slowest queries
print_header "Top 10 Slowest Queries (by mean execution time)"
run_query "Slowest queries" "
SELECT
    LEFT(query, 80) as query_preview,
    calls,
    ROUND(total_exec_time::numeric, 2) as total_time_ms,
    ROUND(mean_exec_time::numeric, 2) as mean_time_ms,
    ROUND(max_exec_time::numeric, 2) as max_time_ms
FROM pg_stat_statements
WHERE query NOT LIKE '%pg_stat_statements%'
ORDER BY mean_exec_time DESC
LIMIT 10;"

# Query 2: Current connections
print_header "Current Active Connections"
run_query "Active connections" "
SELECT
    COUNT(*) FILTER (WHERE state = 'active') as active,
    COUNT(*) FILTER (WHERE state = 'idle') as idle,
    COUNT(*) as total
FROM pg_stat_activity
WHERE datname = 'koprogo_db';"

# Query 3: Database and table sizes
print_header "Database and Table Sizes"
run_query "Table sizes" "
SELECT
    tablename,
    pg_size_pretty(pg_total_relation_size('public.'||tablename)) AS total_size,
    pg_size_pretty(pg_relation_size('public.'||tablename)) AS table_size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size('public.'||tablename) DESC;"

# Query 4: Cache hit ratio
print_header "Cache Hit Ratio (should be > 99%)"
run_query "Cache hit ratio" "
SELECT
    'cache hit rate' AS metric,
    ROUND(
        sum(blks_hit) * 100.0 / NULLIF(sum(blks_hit) + sum(blks_read), 0),
        2
    ) AS percentage,
    CASE
        WHEN sum(blks_hit) * 100.0 / NULLIF(sum(blks_hit) + sum(blks_read), 0) > 99 THEN 'EXCELLENT'
        WHEN sum(blks_hit) * 100.0 / NULLIF(sum(blks_hit) + sum(blks_read), 0) > 95 THEN 'GOOD'
        WHEN sum(blks_hit) * 100.0 / NULLIF(sum(blks_hit) + sum(blks_read), 0) > 90 THEN 'FAIR'
        ELSE 'POOR - Consider increasing shared_buffers'
    END as status
FROM pg_stat_database
WHERE datname = 'koprogo_db';"

# Query 5: Row counts and capacity estimation
print_header "Row Counts and Capacity Estimation"
run_query "Capacity analysis" "
WITH table_stats AS (
    SELECT 'buildings' as entity, COUNT(*) as count FROM buildings
    UNION ALL SELECT 'units', COUNT(*) FROM units
    UNION ALL SELECT 'owners', COUNT(*) FROM owners
    UNION ALL SELECT 'expenses', COUNT(*) FROM expenses
    UNION ALL SELECT 'meetings', COUNT(*) FROM meetings
    UNION ALL SELECT 'documents', COUNT(*) FROM documents
)
SELECT
    entity,
    count as current_rows,
    CASE entity
        WHEN 'buildings' THEN ROUND(count * 10.0) -- avg 10 units per building
        WHEN 'units' THEN count
        ELSE NULL
    END as estimated_units,
    CASE entity
        WHEN 'buildings' THEN count
        ELSE NULL
    END as coproprietés
FROM table_stats
ORDER BY
    CASE entity
        WHEN 'buildings' THEN 1
        WHEN 'units' THEN 2
        WHEN 'owners' THEN 3
        WHEN 'expenses' THEN 4
        WHEN 'meetings' THEN 5
        WHEN 'documents' THEN 6
    END;"

# Query 6: Index usage
print_header "Index Usage Statistics (unused indexes shown first)"
run_query "Index usage" "
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan as scans,
    pg_size_pretty(pg_relation_size(indexrelid)) as size,
    CASE
        WHEN idx_scan = 0 THEN 'UNUSED - Consider dropping'
        WHEN idx_scan < 100 THEN 'RARELY USED'
        ELSE 'OK'
    END as status
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY idx_scan ASC, pg_relation_size(indexrelid) DESC
LIMIT 15;"

# Query 7: Dead tuples (bloat)
print_header "Table Bloat (Dead Tuples)"
run_query "Dead tuples" "
SELECT
    schemaname,
    tablename,
    n_live_tup as live,
    n_dead_tup as dead,
    ROUND((n_dead_tup * 100.0 / NULLIF(n_live_tup + n_dead_tup, 0))::numeric, 2) as dead_pct,
    last_autovacuum,
    CASE
        WHEN n_dead_tup * 100.0 / NULLIF(n_live_tup + n_dead_tup, 0) > 20 THEN 'VACUUM RECOMMENDED'
        WHEN n_dead_tup * 100.0 / NULLIF(n_live_tup + n_dead_tup, 0) > 10 THEN 'MONITOR'
        ELSE 'OK'
    END as status
FROM pg_stat_user_tables
WHERE n_dead_tup > 0
ORDER BY n_dead_tup DESC;"

# Summary
print_header "Capacity Summary"
echo "Based on current data:"
psql "$DB_URL" -t -c "
SELECT
    'Total copropriétés: ' || COUNT(*) as summary
FROM buildings
UNION ALL
SELECT
    'Total lots/units: ' || COUNT(*)
FROM units
UNION ALL
SELECT
    'Total copropriétaires: ' || COUNT(*)
FROM owners
UNION ALL
SELECT
    'Database size: ' || pg_size_pretty(pg_database_size('koprogo_db'))
UNION ALL
SELECT
    'Estimated max copropriétés (30GB disk): ' ||
    CASE
        WHEN pg_database_size('koprogo_db') > 0 AND (SELECT COUNT(*) FROM buildings) > 0
        THEN FLOOR((30.0 * 1024 * 1024 * 1024) / (pg_database_size('koprogo_db') / (SELECT COUNT(*) FROM buildings)))
        ELSE 0
    END::text
;" | grep -v "^$"

echo ""
print_header "Report Complete"
echo "For detailed analysis, see: $SQL_FILE"
echo "You can run specific queries with:"
echo "  psql \"\$DATABASE_URL\" -f $SQL_FILE"
