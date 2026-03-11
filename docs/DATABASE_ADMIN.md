# Database Administration Guide

Version: 1.0.0 | PostgreSQL 15

## Common Operations

### Backups

```bash
# Manual backup
sudo -u postgres pg_dump koprogo_db | gzip > backup_$(date +%Y%m%d).sql.gz

# Restore
gunzip -c backup_20251110.sql.gz | sudo -u postgres psql koprogo_db
```

### Migrations

```bash
# Apply pending migrations
cd backend
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Create new migration
sqlx migrate add add_column_x_to_table_y
```

### Performance

```sql
-- Vacuum and analyze
VACUUM ANALYZE;

-- Reindex
REINDEX DATABASE koprogo_db;

-- Check table sizes
SELECT relname, pg_size_pretty(pg_total_relation_size(relid))
FROM pg_catalog.pg_statio_user_tables
ORDER BY pg_total_relation_size(relid) DESC;

-- Check index usage
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
ORDER BY idx_scan ASC;
```

### Monitoring

```sql
-- Active connections
SELECT count(*) FROM pg_stat_activity WHERE state = 'active';

-- Long-running queries
SELECT pid, now() - query_start as duration, query
FROM pg_stat_activity
WHERE state = 'active' AND now() - query_start > interval '5 minutes';

-- Lock monitoring
SELECT * FROM pg_locks WHERE NOT granted;
```

### Maintenance

```sql
-- Terminate idle connections
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE state = 'idle' AND now() - state_change > interval '10 minutes';

-- Update statistics
ANALYZE;
```

## Troubleshooting

### Connection exhausted

```bash
# Increase max_connections
sudo nano /etc/postgresql/15/main/postgresql.conf
# max_connections = 200

sudo systemctl restart postgresql
```

### Slow queries

```sql
-- Enable slow query logging
ALTER DATABASE koprogo_db SET log_min_duration_statement = 1000; -- 1s

-- Check logs
sudo tail -f /var/log/postgresql/postgresql-15-main.log
```

---

**Version**: 1.0.0
