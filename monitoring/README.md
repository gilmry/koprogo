# KoproGo VPS Monitoring Guide

This monitoring system helps you track VPS resources and estimate capacity for hosting copropriétés.

## Quick Start

```bash
# Run system metrics check
./monitoring/scripts/vps_metrics.sh

# Run PostgreSQL performance analysis
./monitoring/scripts/postgres_metrics.sh

# Calculate capacity
./monitoring/scripts/capacity_calculator.sh
```

## Scripts Overview

### 1. vps_metrics.sh
Monitors system resources (RAM, CPU, disk, load average) and PostgreSQL connections.

**Output:**
- Color-coded status (OK/WARNING/CRITICAL)
- Detailed metrics vs. thresholds
- JSON export for logging/alerting
- Upgrade recommendations

**Exit codes:**
- `0`: All OK
- `1`: Warning threshold exceeded
- `2`: Critical threshold exceeded

**Example usage:**
```bash
# Manual check
./monitoring/scripts/vps_metrics.sh

# Cron job (every 5 minutes)
*/5 * * * * /home/koprogo/monitoring/scripts/vps_metrics.sh >> /var/log/koprogo/metrics.log 2>&1
```

### 2. postgres_metrics.sh
Analyzes PostgreSQL performance and database health.

**Checks:**
- Top slowest queries (requires pg_stat_statements)
- Active connections
- Table and index sizes
- Cache hit ratio (should be > 99%)
- Index usage statistics
- Dead tuples (bloat detection)
- Capacity estimation

**Example usage:**
```bash
# Run full analysis
DATABASE_URL="postgresql://user:pass@localhost/koprogo_db" \
  ./monitoring/scripts/postgres_metrics.sh

# Hourly cron job
0 * * * * /home/koprogo/monitoring/scripts/postgres_metrics.sh >> /var/log/koprogo/postgres.log 2>&1
```

### 3. capacity_calculator.sh
Estimates how many copropriétés can be hosted on current VPS.

**Calculations:**
- Current database statistics
- Average data per copropriété
- Estimated maximum capacity (based on 30GB available disk)
- RAM headroom analysis
- Upgrade recommendations

**Example output:**
```
Current Database Statistics
Copropriétés: 50
Units/Lots: 487
Database size: 2.1 MB

Capacity Estimation
Estimated maximum copropriétés: 15,000
Current capacity usage: 0.33%

Upgrade Recommendations
Current tier: Hetzner CPX11 (4.15€/month) - OPTIMAL
Upgrade to CPX21 at: ~100 copropriétés
```

## Configuration

### Thresholds
Edit `monitoring/config/thresholds.env` to customize alert thresholds:

```bash
RAM_WARNING_THRESHOLD=75      # Warn at 75% RAM usage
RAM_CRITICAL_THRESHOLD=85     # Critical at 85%
CPU_WARNING_THRESHOLD=70
CPU_CRITICAL_THRESHOLD=85
DISK_WARNING_THRESHOLD=70
DISK_CRITICAL_THRESHOLD=80
LOAD_WARNING_THRESHOLD=1.5    # For 2-CPU system
LOAD_CRITICAL_THRESHOLD=2.0
```

### Database Connection
Set the `DATABASE_URL` environment variable:

```bash
export DATABASE_URL="postgresql://koprogo:password@localhost:5432/koprogo_db"
```

Or add to `~/.bashrc` for persistence.

## Monitoring Schedule (Cron)

Recommended cron jobs for production:

```bash
# Edit crontab
crontab -e

# Add these lines:

# System metrics every 5 minutes
*/5 * * * * /home/koprogo/monitoring/scripts/vps_metrics.sh >> /var/log/koprogo/vps.log 2>&1

# PostgreSQL metrics hourly
0 * * * * /home/koprogo/monitoring/scripts/postgres_metrics.sh >> /var/log/koprogo/postgres.log 2>&1

# Daily capacity summary at 9am
0 9 * * * /home/koprogo/monitoring/scripts/capacity_calculator.sh >> /var/log/koprogo/capacity.log 2>&1

# Weekly log rotation (delete logs older than 7 days)
0 2 * * 0 find /home/koprogo/monitoring/logs -name "*.json" -mtime +7 -delete
```

## Interpreting Results

### System Metrics

**RAM Usage:**
- < 75%: OK - Normal operation
- 75-85%: WARNING - Monitor closely, consider optimization
- > 85%: CRITICAL - Upgrade VPS or optimize immediately

**CPU Usage:**
- < 70%: OK
- 70-85%: WARNING - High load, investigate queries/processes
- > 85%: CRITICAL - Performance degradation likely

**Load Average (2-CPU system):**
- < 1.5: OK - 75% utilization
- 1.5-2.0: WARNING - Approaching max
- > 2.0: CRITICAL - CPU bottleneck

**Disk Usage:**
- < 70%: OK
- 70-80%: WARNING - Plan cleanup or upgrade
- > 80%: CRITICAL - Clean up logs/backups immediately

### PostgreSQL Metrics

**Cache Hit Ratio:**
- > 99%: EXCELLENT - Optimal configuration
- 95-99%: GOOD
- 90-95%: FAIR - Consider increasing shared_buffers
- < 90%: POOR - Database undersized for workload

**Query Performance:**
- Mean query time < 10ms: EXCELLENT (target: P99 < 5ms)
- Mean query time 10-50ms: ACCEPTABLE
- Mean query time > 50ms: SLOW - Optimize queries/indexes

**Dead Tuples:**
- < 10%: OK
- 10-20%: MONITOR - Autovacuum working normally
- > 20%: VACUUM RECOMMENDED - Run manual VACUUM

## Capacity Guidelines

Based on Hetzner CPX11 (2GB RAM, 40GB disk, 4.15€/month):

### Small Copropriétés (5-10 lots)
- **500-1,000 copropriétés**: Comfortable
- **Memory**: ~1,500MB used (500MB free)
- **Disk**: ~20GB data (50% usage)
- **Performance**: P99 < 5ms easily achieved

### Medium Copropriétés (20-30 lots)
- **200-500 copropriétés**: Good
- **Memory**: May need upgrade at 500+
- **Disk**: Sufficient for years

### Large Copropriétés (50+ lots)
- **50-100 copropriétés**: OK
- **Consider upgrade**: Hetzner CPX21 (8.25€/month)

## Upgrade Path

### Phase 1: 0-100 copropriétés
- **VPS**: Hetzner CPX11 (4.15€/month)
- **RAM**: 2GB - Sufficient
- **Disk**: 40GB - Sufficient

### Phase 2: 100-500 copropriétés
- **VPS**: Hetzner CPX21 (8.25€/month)
- **RAM**: 4GB - Double capacity
- **Disk**: 80GB

### Phase 3: 500-2,000 copropriétés
- **VPS**: Hetzner CPX31 (16.50€/month)
- **RAM**: 8GB
- **Disk**: 160GB
- OR separate database server

### Phase 4: 2,000+ copropriétés
- **Architecture**: Load balancer + multiple app servers
- **Database**: Managed PostgreSQL (Hetzner/AWS RDS)
- **Cost**: ~50-100€/month

## Alerting Integration

### Email Alerts (simple)
Add to cron jobs:

```bash
*/5 * * * * /home/koprogo/monitoring/scripts/vps_metrics.sh | mail -s "KoproGo Alert" admin@example.com
```

### UptimeRobot (free)
Monitor HTTP endpoint: `https://your-domain.com/api/v1/health`
- Check interval: 5 minutes
- Alert on: Down or response time > 1s

### Advanced Monitoring (optional)
- **Prometheus + Grafana**: Full metrics dashboard
- **PagerDuty**: On-call alerting
- **Sentry**: Error tracking

## Troubleshooting

### High RAM Usage
```bash
# Check what's consuming RAM
free -h
docker stats

# Reduce PostgreSQL shared_buffers if needed
# Edit docker-compose.vps.yml: shared_buffers=256MB
```

### High CPU Load
```bash
# Check PostgreSQL slow queries
./monitoring/scripts/postgres_metrics.sh

# Check running processes
top -o %CPU
```

### Disk Full
```bash
# Clean Docker
docker system prune -a

# Clean old logs
find /var/log -name "*.log" -mtime +30 -delete

# Clean PostgreSQL WAL files (if safe)
# Only if replication not needed
```

### Slow Queries
```bash
# Analyze with PostgreSQL script
./monitoring/scripts/postgres_metrics.sh

# Check for missing indexes
# Look for tables with low index scan counts

# Run VACUUM if high dead tuples
docker exec -it koprogo-postgres psql -U koprogo -d koprogo_db -c "VACUUM ANALYZE;"
```

## Logs

All monitoring outputs are stored in:
```
monitoring/logs/
├── metrics_20241023_120000.json
├── metrics_20241023_120500.json
└── ...
```

View latest metrics:
```bash
cat monitoring/logs/metrics_*.json | tail -1 | jq .
```

## Performance Targets (KoproGo)

From CLAUDE.md specifications:

- **Latency P99**: < 5ms
- **Throughput**: > 100k req/s
- **Memory**: < 128MB per backend instance
- **Connection Pool**: Max 10 PostgreSQL connections
- **CO2/request**: < 0.5g (ecological target)

Monitor these with:
```bash
# Check latency (requires running system)
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8080/api/v1/health

# Check memory
docker stats koprogo-backend --no-stream
```

## Support

For issues or questions:
- Check logs: `monitoring/logs/`
- Review thresholds: `monitoring/config/thresholds.env`
- Consult CLAUDE.md for architecture details
