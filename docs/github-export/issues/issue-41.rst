==============================================================================
Issue #41: feat(infra): Deploy monitoring stack (Prometheus + Grafana + Loki)
==============================================================================

:State: **OPEN**
:Milestone: Phase 1: VPS MVP + Legal Compliance
:Labels: phase:vps,track:infrastructure priority:critical
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-01
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/41>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Current monitoring relies on **bash scripts + cron jobs** (`monitoring/scripts/*.sh`) which provide basic metrics but lack:
   - Centralized metrics storage
   - Real-time alerting
   - Historical trend analysis
   - Visual dashboards
   - Log aggregation
   
   Production deployment requires a proper observability stack for incident response and performance monitoring.
   
   ## Current Implementation (30% Complete)
   
   **Existing monitoring scripts:**
   - `monitoring/scripts/vps_metrics.sh` - RAM, CPU, disk, load average
   - `monitoring/scripts/postgres_metrics.sh` - Slow queries, connections, cache hit ratio
   - `monitoring/scripts/capacity_calculator.sh` - Database capacity estimation
   - Cron jobs: health checks every 5 minutes
   
   **Limitations:**
   - No centralized storage (logs scattered)
   - No alerting mechanism
   - No dashboards
   - Manual metric collection
   - No log aggregation
   
   ## Objective
   
   Deploy production-grade monitoring stack with:
   1. **Prometheus** - Metrics collection & storage
   2. **Grafana** - Dashboards & visualization
   3. **Loki** - Log aggregation
   4. **Alertmanager** - Alert routing & notification
   5. **Node Exporter** - System metrics
   6. **PostgreSQL Exporter** - Database metrics
   7. **cAdvisor** - Container metrics
   
   ## Architecture
   
   ```
   ┌─────────────────┐
   │   Grafana       │ ← Dashboard UI (port 3001)
   │   (Dashboards)  │
   └────────┬────────┘
            │
       ┌────┴────┬─────────┐
       │         │         │
   ┌───▼───┐ ┌──▼───┐ ┌───▼────┐
   │Prom-  │ │ Loki │ │Alert-  │
   │etheus │ │      │ │manager │
   └───┬───┘ └──┬───┘ └────────┘
       │        │
   ┌───┴────────┴───────────┐
   │  Exporters (scraping)   │
   ├─────────────────────────┤
   │ - Node Exporter (VPS)   │
   │ - PostgreSQL Exporter   │
   │ - cAdvisor (containers) │
   │ - Traefik metrics       │
   │ - Application /metrics  │
   └─────────────────────────┘
   ```
   
   ## Implementation Plan
   
   ### 1. Docker Compose Stack
   
   **Create:** `monitoring/docker-compose.monitoring.yml`
   
   ```yaml
   version: '3.8'
   
   services:
     prometheus:
       image: prom/prometheus:v2.45.0
       volumes:
         - ./prometheus.yml:/etc/prometheus/prometheus.yml
         - prometheus_data:/prometheus
       command:
         - '--config.file=/etc/prometheus/prometheus.yml'
         - '--storage.tsdb.retention.time=30d'
       ports:
         - "9090:9090"
       restart: unless-stopped
   
     grafana:
       image: grafana/grafana:10.0.3
       volumes:
         - grafana_data:/var/lib/grafana
         - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
         - ./grafana/datasources:/etc/grafana/provisioning/datasources
       environment:
         - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD}
         - GF_SERVER_ROOT_URL=https://${MONITORING_DOMAIN}
       ports:
         - "3001:3000"
       restart: unless-stopped
   
     loki:
       image: grafana/loki:2.9.0
       volumes:
         - loki_data:/loki
         - ./loki-config.yml:/etc/loki/local-config.yaml
       ports:
         - "3100:3100"
       restart: unless-stopped
   
     promtail:
       image: grafana/promtail:2.9.0
       volumes:
         - /var/log:/var/log:ro
         - /var/lib/docker/containers:/var/lib/docker/containers:ro
         - ./promtail-config.yml:/etc/promtail/config.yml
       command: -config.file=/etc/promtail/config.yml
       restart: unless-stopped
   
     alertmanager:
       image: prom/alertmanager:v0.26.0
       volumes:
         - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml
       ports:
         - "9093:9093"
       restart: unless-stopped
   
     node-exporter:
       image: prom/node-exporter:v1.6.1
       volumes:
         - /proc:/host/proc:ro
         - /sys:/host/sys:ro
         - /:/rootfs:ro
       command:
         - '--path.procfs=/host/proc'
         - '--path.sysfs=/host/sys'
         - '--collector.filesystem.mount-points-exclude=^/(sys|proc|dev|host|etc)($$|/)'
       ports:
         - "9100:9100"
       restart: unless-stopped
   
     postgres-exporter:
       image: prometheuscommunity/postgres-exporter:v0.13.2
       environment:
         DATA_SOURCE_NAME: "postgresql://koprogo:${POSTGRES_PASSWORD}@postgres:5432/koprogo_db?sslmode=disable"
       ports:
         - "9187:9187"
       restart: unless-stopped
   
     cadvisor:
       image: gcr.io/cadvisor/cadvisor:v0.47.0
       volumes:
         - /:/rootfs:ro
         - /var/run:/var/run:ro
         - /sys:/sys:ro
         - /var/lib/docker:/var/lib/docker:ro
       ports:
         - "8082:8080"
       restart: unless-stopped
   
   volumes:
     prometheus_data:
     grafana_data:
     loki_data:
   ```
   
   ### 2. Prometheus Configuration
   
   **Create:** `monitoring/prometheus.yml`
   
   ```yaml
   global:
     scrape_interval: 15s
     evaluation_interval: 15s
   
   alerting:
     alertmanagers:
       - static_configs:
           - targets: ['alertmanager:9093']
   
   rule_files:
     - '/etc/prometheus/alerts/*.yml'
   
   scrape_configs:
     - job_name: 'prometheus'
       static_configs:
         - targets: ['localhost:9090']
   
     - job_name: 'node-exporter'
       static_configs:
         - targets: ['node-exporter:9100']
   
     - job_name: 'postgres-exporter'
       static_configs:
         - targets: ['postgres-exporter:9187']
   
     - job_name: 'cadvisor'
       static_configs:
         - targets: ['cadvisor:8080']
   
     - job_name: 'traefik'
       static_configs:
         - targets: ['traefik:8080']
   
     - job_name: 'koprogo-backend'
       static_configs:
         - targets: ['backend:8080']
       metrics_path: '/metrics'
   ```
   
   ### 3. Alert Rules
   
   **Create:** `monitoring/alerts/koprogo.yml`
   
   ```yaml
   groups:
     - name: koprogo_alerts
       rules:
         # High CPU
         - alert: HighCPUUsage
           expr: 100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 85
           for: 5m
           labels:
             severity: critical
           annotations:
             summary: "High CPU usage on {{ $labels.instance }}"
             description: "CPU usage is {{ $value }}%"
   
         # High Memory
         - alert: HighMemoryUsage
           expr: (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100 > 85
           for: 5m
           labels:
             severity: critical
           annotations:
             summary: "High memory usage on {{ $labels.instance }}"
             description: "Memory usage is {{ $value }}%"
   
         # Disk Space
         - alert: DiskSpaceLow
           expr: (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"}) * 100 < 20
           for: 5m
           labels:
             severity: warning
           annotations:
             summary: "Low disk space on {{ $labels.instance }}"
             description: "{{ $value }}% remaining"
   
         # PostgreSQL
         - alert: PostgreSQLDown
           expr: pg_up == 0
           for: 1m
           labels:
             severity: critical
           annotations:
             summary: "PostgreSQL is down"
   
         - alert: PostgreSQLSlowQueries
           expr: rate(pg_stat_statements_mean_time_seconds[5m]) > 0.005
           for: 5m
           labels:
             severity: warning
           annotations:
             summary: "PostgreSQL P99 latency > 5ms target"
             description: "Average query time: {{ $value }}s"
   
         # Container
         - alert: ContainerDown
           expr: up{job="cadvisor"} == 0
           for: 2m
           labels:
             severity: critical
           annotations:
             summary: "Container {{ $labels.name }} is down"
   
         # Backup
         - alert: BackupFailed
           expr: time() - koprogo_last_backup_timestamp_seconds > 86400
           for: 1h
           labels:
             severity: critical
           annotations:
             summary: "Backup has not run in 24h"
   ```
   
   ### 4. Grafana Dashboards
   
   **Create pre-configured dashboards:**
   - `monitoring/grafana/dashboards/koprogo-overview.json` - System overview
   - `monitoring/grafana/dashboards/postgres.json` - PostgreSQL metrics
   - `monitoring/grafana/dashboards/docker.json` - Container metrics
   - `monitoring/grafana/dashboards/traefik.json` - HTTP traffic
   
   **Import community dashboards:**
   - Node Exporter Full (ID: 1860)
   - PostgreSQL Database (ID: 9628)
   - Docker and System Monitoring (ID: 179)
   - Traefik 2 (ID: 11462)
   
   ### 5. Alertmanager Configuration
   
   **Create:** `monitoring/alertmanager.yml`
   
   ```yaml
   global:
     resolve_timeout: 5m
   
   route:
     group_by: ['alertname', 'severity']
     group_wait: 10s
     group_interval: 10s
     repeat_interval: 12h
     receiver: 'email'
   
   receivers:
     - name: 'email'
       email_configs:
         - to: '${ALERT_EMAIL}'
           from: 'alertmanager@koprogo.com'
           smarthost: 'smtp.gmail.com:587'
           auth_username: '${SMTP_USERNAME}'
           auth_password: '${SMTP_PASSWORD}'
           headers:
             Subject: '[KoproGo] {{ .GroupLabels.alertname }}'
   ```
   
   ### 6. Backend Metrics Endpoint
   
   **Add to backend:**
   
   `backend/src/infrastructure/web/metrics.rs` (new):
   ```rust
   use actix_web::{get, HttpResponse};
   use prometheus::{Encoder, TextEncoder, Registry};
   
   #[get("/metrics")]
   async fn metrics() -> HttpResponse {
       let encoder = TextEncoder::new();
       let metric_families = prometheus::gather();
       let mut buffer = vec![];
       encoder.encode(&metric_families, &mut buffer).unwrap();
       
       HttpResponse::Ok()
           .content_type("text/plain; version=0.0.4")
           .body(buffer)
   }
   ```
   
   Add `prometheus` crate to `Cargo.toml`:
   ```toml
   prometheus = "0.13"
   ```
   
   ### 7. Ansible Deployment
   
   **Update:** `infrastructure/ansible/playbook.yml`
   
   ```yaml
   - name: Create monitoring directory
     file:
       path: /opt/koprogo/monitoring
       state: directory
       owner: koprogo
       mode: '0755'
   
   - name: Copy monitoring configs
     template:
       src: "{{ item }}"
       dest: /opt/koprogo/monitoring/
     with_fileglob:
       - "monitoring/*.yml"
   
   - name: Start monitoring stack
     command: docker-compose -f /opt/koprogo/monitoring/docker-compose.monitoring.yml up -d
   ```
   
   ## Testing & Validation
   
   - [ ] All exporters scraping successfully (Prometheus targets page)
   - [ ] Grafana dashboards loading with data
   - [ ] Alerts firing correctly (test by triggering conditions)
   - [ ] Logs visible in Loki
   - [ ] Email alerts received
   - [ ] Performance impact acceptable (<5% CPU overhead)
   - [ ] Retention policy working (30d metrics, 7d logs)
   
   ## Security
   
   - [ ] Grafana admin password strong (min 20 chars)
   - [ ] Prometheus/Grafana not exposed publicly (localhost or VPN only)
   - [ ] Traefik reverse proxy with authentication if exposed
   - [ ] SMTP credentials in Ansible vault
   
   ## Documentation
   
   - [ ] Update `monitoring/README.md` with access URLs
   - [ ] Document alert thresholds and tuning
   - [ ] Create runbook for common alerts
   - [ ] Update CLAUDE.md with monitoring architecture
   
   ## Acceptance Criteria
   
   - [ ] Prometheus scraping all metrics (VPS, PostgreSQL, containers, Traefik, backend)
   - [ ] Grafana dashboards operational (4 pre-configured dashboards)
   - [ ] Loki aggregating logs from containers and VPS
   - [ ] Alertmanager sending email notifications
   - [ ] Critical alerts configured (CPU, memory, disk, PostgreSQL, backups)
   - [ ] Documentation complete
   - [ ] Monitoring stack integrated with Ansible deployment
   
   ## Resource Requirements
   
   **Estimated overhead:**
   - Prometheus: ~200MB RAM, 10GB disk (30d retention)
   - Grafana: ~100MB RAM
   - Loki: ~150MB RAM, 5GB disk (7d retention)
   - Exporters: ~50MB RAM total
   - **Total: ~500MB RAM, 15GB disk**
   
   **VPS impact:** Acceptable on 2GB VPS (25% overhead)
   
   ## Effort Estimate
   
   **Large** (3-5 days)
   - Day 1: Docker Compose stack + Prometheus
   - Day 2: Grafana dashboards
   - Day 3: Loki + log aggregation
   - Day 4: Alert rules + testing
   - Day 5: Documentation + refinement
   
   ## Related
   
   - Supports: Issue #40 (backup monitoring)
   - Supports: Performance optimization (P99 latency tracking)
   - Enables: Production incident response
   
   ## References
   
   - Prometheus: https://prometheus.io/docs/
   - Grafana: https://grafana.com/docs/
   - Loki: https://grafana.com/docs/loki/
   - Node Exporter: https://github.com/prometheus/node_exporter
   - PostgreSQL Exporter: https://github.com/prometheus-community/postgres_exporter

.. raw:: html

   </div>

