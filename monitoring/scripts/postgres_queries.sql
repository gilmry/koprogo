-- ============================================================================
-- KoproGo PostgreSQL Performance Monitoring Queries
-- ============================================================================

-- Query 1: Top 10 slowest queries (requires pg_stat_statements extension)
-- ----------------------------------------------------------------------------
SELECT
    query,
    calls,
    ROUND(total_exec_time::numeric, 2) as total_time_ms,
    ROUND(mean_exec_time::numeric, 2) as mean_time_ms,
    ROUND(max_exec_time::numeric, 2) as max_time_ms,
    ROUND(stddev_exec_time::numeric, 2) as stddev_time_ms,
    ROUND((100 * total_exec_time / SUM(total_exec_time) OVER ())::numeric, 2) as percentage
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Query 2: Current active connections
-- ----------------------------------------------------------------------------
SELECT
    pid,
    usename,
    application_name,
    client_addr,
    state,
    query_start,
    state_change,
    wait_event_type,
    wait_event,
    LEFT(query, 100) as query_preview
FROM pg_stat_activity
WHERE datname = 'koprogo_db'
ORDER BY query_start DESC;

-- Query 3: Database size and table sizes
-- ----------------------------------------------------------------------------
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS total_size,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) AS table_size,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename) - pg_relation_size(schemaname||'.'||tablename)) AS index_size
FROM pg_tables
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Query 4: Index usage statistics
-- ----------------------------------------------------------------------------
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan as index_scans,
    idx_tup_read as tuples_read,
    idx_tup_fetch as tuples_fetched,
    pg_size_pretty(pg_relation_size(indexrelid)) as index_size
FROM pg_stat_user_indexes
ORDER BY idx_scan ASC;

-- Query 5: Cache hit ratio (should be > 99%)
-- ----------------------------------------------------------------------------
SELECT
    'cache hit rate' AS metric,
    ROUND(
        sum(blks_hit) * 100.0 / NULLIF(sum(blks_hit) + sum(blks_read), 0),
        2
    ) AS percentage
FROM pg_stat_database
WHERE datname = 'koprogo_db';

-- Query 6: Table bloat estimation
-- ----------------------------------------------------------------------------
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as total_size,
    n_live_tup as live_tuples,
    n_dead_tup as dead_tuples,
    ROUND((n_dead_tup * 100.0 / NULLIF(n_live_tup + n_dead_tup, 0))::numeric, 2) as dead_tuple_percent,
    last_autovacuum,
    last_autoanalyze
FROM pg_stat_user_tables
WHERE n_dead_tup > 0
ORDER BY n_dead_tup DESC;

-- Query 7: Row count estimates for main tables
-- ----------------------------------------------------------------------------
SELECT
    schemaname,
    relname as table_name,
    n_live_tup as estimated_rows,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||relname)) as total_size,
    ROUND((pg_total_relation_size(schemaname||'.'||relname) / NULLIF(n_live_tup, 0))::numeric, 0) as bytes_per_row
FROM pg_stat_user_tables
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY n_live_tup DESC;

-- Query 8: Locks currently held
-- ----------------------------------------------------------------------------
SELECT
    pl.pid,
    pa.usename,
    pa.application_name,
    pl.locktype,
    pl.mode,
    pl.granted,
    LEFT(pa.query, 80) as query
FROM pg_locks pl
LEFT JOIN pg_stat_activity pa ON pl.pid = pa.pid
WHERE pa.datname = 'koprogo_db'
ORDER BY pl.granted, pl.pid;

-- Query 9: Transaction age (long-running transactions)
-- ----------------------------------------------------------------------------
SELECT
    pid,
    usename,
    application_name,
    state,
    now() - xact_start AS transaction_duration,
    LEFT(query, 100) as query
FROM pg_stat_activity
WHERE datname = 'koprogo_db'
    AND state != 'idle'
    AND xact_start IS NOT NULL
ORDER BY xact_start;

-- Query 10: Capacity estimation for KoproGo
-- ----------------------------------------------------------------------------
WITH table_stats AS (
    SELECT
        'buildings' as entity,
        COUNT(*) as current_count,
        pg_total_relation_size('public.buildings') as total_bytes,
        CASE WHEN COUNT(*) > 0
            THEN pg_total_relation_size('public.buildings') / COUNT(*)
            ELSE 0
        END as bytes_per_row
    FROM buildings
    UNION ALL
    SELECT
        'units',
        COUNT(*),
        pg_total_relation_size('public.units'),
        CASE WHEN COUNT(*) > 0
            THEN pg_total_relation_size('public.units') / COUNT(*)
            ELSE 0
        END
    FROM units
    UNION ALL
    SELECT
        'owners',
        COUNT(*),
        pg_total_relation_size('public.owners'),
        CASE WHEN COUNT(*) > 0
            THEN pg_total_relation_size('public.owners') / COUNT(*)
            ELSE 0
        END
    FROM owners
    UNION ALL
    SELECT
        'expenses',
        COUNT(*),
        pg_total_relation_size('public.expenses'),
        CASE WHEN COUNT(*) > 0
            THEN pg_total_relation_size('public.expenses') / COUNT(*)
            ELSE 0
        END
    FROM expenses
)
SELECT
    entity,
    current_count,
    pg_size_pretty(total_bytes) as current_size,
    ROUND(bytes_per_row::numeric, 0) as bytes_per_row,
    -- Estimate: how many could fit in 30GB
    CASE WHEN bytes_per_row > 0
        THEN FLOOR(30 * 1024 * 1024 * 1024 / bytes_per_row)
        ELSE 0
    END as estimated_max_rows_30gb
FROM table_stats;
