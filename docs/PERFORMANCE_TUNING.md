# Performance Tuning Guide

Version: 1.0.0 | Target: P99 < 5ms, Throughput > 100k req/s

## Current Performance (Oct 2025)

- **Latency P99**: 752ms (1 vCPU, sustained load)
- **Throughput**: 287 req/s
- **Memory**: ~80MB
- **CO2/request**: 0.12g

## Backend Optimization

### 1. Database Connection Pool

```rust
// backend/src/main.rs
let pool = PgPoolOptions::new()
    .max_connections(20)        // Increase from 10
    .acquire_timeout(Duration::from_secs(3))
    .connect(&database_url)
    .await?;
```

### 2. Query Optimization

```sql
-- Add indexes for frequent queries
CREATE INDEX idx_expenses_building_status ON expenses(building_id, status);
CREATE INDEX idx_unit_owners_unit_active ON unit_owners(unit_id) WHERE end_date IS NULL;

-- Analyze query plans
EXPLAIN ANALYZE SELECT * FROM expenses WHERE building_id = $1 AND status = 'Approved';
```

### 3. Caching Strategy

```rust
use once_cell::sync::Lazy;
use std::collections::HashMap;

static ACCOUNT_CACHE: Lazy<Mutex<HashMap<String, Account>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});
```

### 4. Async Optimization

```rust
// Use join instead of sequential awaits
let (buildings, units, owners) = tokio::join!(
    fetch_buildings(),
    fetch_units(),
    fetch_owners()
);
```

## Database Tuning

### PostgreSQL Configuration

```ini
# /etc/postgresql/15/main/postgresql.conf

shared_buffers = 256MB              # 25% of RAM
effective_cache_size = 768MB        # 75% of RAM
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1              # SSD
effective_io_concurrency = 200      # SSD
work_mem = 4MB
min_wal_size = 1GB
max_wal_size = 4GB
max_worker_processes = 2
max_parallel_workers_per_gather = 1
max_parallel_workers = 2
```

## Frontend Optimization

### 1. Code Splitting

```astro
---
// Lazy load heavy components
const HeavyChart = () => import('../components/HeavyChart.svelte');
---
```

### 2. Image Optimization

```bash
# Convert to WebP
npm install sharp
npx sharp input.jpg -o output.webp --webp

# Serve with <picture>
<picture>
  <source srcset="image.webp" type="image/webp">
  <img src="image.jpg" alt="...">
</picture>
```

### 3. Bundle Analysis

```bash
npm run build
npx vite-bundle-visualizer
```

## Monitoring & Profiling

### Backend Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile CPU
sudo cargo flamegraph --bin koprogo-api

# Profile memory
valgrind --tool=massif ./target/release/koprogo-api
```

### Database Profiling

```sql
-- Enable pg_stat_statements
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- Find slow queries
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```

## Load Testing

```bash
cd load-tests
k6 run --vus 100 --duration 60s scenarios/api-load.js
```

## Quick Wins

1. ✅ Enable gzip compression (Nginx/Traefik)
2. ✅ Use CDN for static assets
3. ✅ Add database indexes on foreign keys
4. ✅ Cache PCMN accounts (rarely change)
5. ✅ Use connection pooling
6. ✅ Implement HTTP caching headers

---

**Version**: 1.0.0
