# KoproGo Security & Monitoring Infrastructure

Comprehensive security and monitoring setup for production KoproGo deployment.

**Issues implemented:**
- [#39](https://github.com/gilmry/koprogo/issues/39) - LUKS Encryption at Rest
- [#40](https://github.com/gilmry/koprogo/issues/40) - Encrypted Backups (GPG + S3)
- [#41](https://github.com/gilmry/koprogo/issues/41) - Monitoring Stack (Prometheus + Grafana + Loki)
- [#43](https://github.com/gilmry/koprogo/issues/43) - Advanced Security Hardening

---

## 🚀 Quick Start

Deploy all security and monitoring features:

```bash
cd infrastructure/ansible
ansible-playbook -i inventory.ini security-monitoring.yml
```

Or run specific components:

```bash
# Security only
ansible-playbook -i inventory.ini security-monitoring.yml --tags security

# Monitoring only
ansible-playbook -i inventory.ini security-monitoring.yml --tags monitoring
```

---

## 🔐 Security Features

### 1. Encryption at Rest (LUKS) - Issue #39

**Status:** ✅ Implemented

Encrypts PostgreSQL data and document uploads volumes using LUKS full-disk encryption.

**Features:**
- AES-XTS-PLAIN64 cipher with 512-bit keys
- SHA-512 hashing
- Automatic unlock on boot via `/root/.koprogo-luks-key`
- Configured in `/etc/crypttab` and `/etc/fstab`

**Setup:**

```bash
# Configure volume devices (edit inventory.ini or pass as variables)
ansible-playbook -i inventory.ini security-monitoring.yml \
  -e "postgres_volume_device=vdb" \
  -e "uploads_volume_device=vdc"

# Verify encryption
cryptsetup status postgres_encrypted
cryptsetup status uploads_encrypted
```

**Backup encryption key:**

```bash
# Export and encrypt the LUKS key (CRITICAL!)
sudo gpg --encrypt --recipient admin@koprogo.com /root/.koprogo-luks-key

# Store encrypted key in secure location (password manager, offline storage)
```

**Performance impact:** < 5% (AES-NI hardware acceleration)

**Files:**
- Template: `ansible/templates/luks-setup.sh.j2`
- Script: `/usr/local/bin/koprogo-luks-setup.sh` (deployed on VPS)
- Config: `/etc/crypttab`, `/etc/fstab.d/koprogo-luks`

---

### 2. Encrypted Backups (GPG + S3) - Issue #40

**Status:** ✅ Implemented

Daily encrypted backups of PostgreSQL database, MinIO documents, and configuration files.

**Features:**
- GPG encryption (RSA 4096-bit)
- Compression (gzip level 9)
- S3 sync to off-site storage (OVH, Scaleway, Backblaze B2)
- Server-side encryption (SSE)
- Retention policy: 7 days local, configurable S3 lifecycle
- Automated restore testing

**Backup schedule:** Daily at 2:00 AM UTC (cron)

**What's backed up:**
- PostgreSQL database (full dump)
- MinIO metadata (file listings)
- `.env` configuration (encrypted)
- Backup metrics for Prometheus monitoring

**S3 Configuration:**

Edit `inventory.ini` or `.env`:

```ini
# S3 provider: ovh, scaleway, backblaze
s3_provider=ovh
s3_access_key=YOUR_ACCESS_KEY
s3_secret_key=YOUR_SECRET_KEY
s3_bucket=koprogo-backups
s3_region=GRA  # For OVH: GRA, SBG, BHS

# Alternative: Scaleway
# s3_provider=scaleway
# s3_region=fr-par
```

**Manual backup:**

```bash
sudo /home/koprogo/koprogo/scripts/backup-encrypted.sh
```

**Restore from backup:**

```bash
# List available backups
ls -lh /home/koprogo/koprogo/backups/

# Decrypt and restore
gpg --decrypt koprogo_20251104_020000.sql.gz.gpg | \
  gunzip | \
  docker exec -i koprogo-postgres psql -U koprogo koprogo_db
```

**Verify GPG key:**

```bash
gpg --list-keys backup@koprogo.local
```

**Cost estimate (100GB storage):**
- OVH: €1.10/month
- Scaleway: €1.10/month
- Backblaze B2: €0.50/month (cheapest)

**Files:**
- Template: `ansible/templates/backup-encrypted.sh.j2`
- Script: `/home/koprogo/koprogo/scripts/backup-encrypted.sh`
- Config: `/root/.s3cfg`
- GPG key: `/root/.gnupg/`

---

### 3. fail2ban Custom Jails

**Status:** ✅ Implemented

Protects against brute-force attacks, bot abuse, and API abuse.

**Active jails:**
- **sshd:** SSH brute-force protection (3 attempts, 1h ban)
- **traefik-auth:** HTTP 401/403 detection (5 attempts, 30min ban)
- **traefik-badbots:** Malicious bot blocking (2 attempts, 24h ban)
- **koprogo-api-abuse:** API rate limiting violations (20 attempts, 1h ban)
- **postgres-bruteforce:** PostgreSQL auth failures (3 attempts, 2h ban)

**Check status:**

```bash
sudo fail2ban-client status
sudo fail2ban-client status sshd
sudo fail2ban-client status traefik-auth

# List banned IPs
sudo fail2ban-client status koprogo-api-abuse | grep "Banned IP"
```

**Unban IP:**

```bash
sudo fail2ban-client set sshd unbanip 1.2.3.4
```

**Files:**
- Config: `/etc/fail2ban/jail.d/koprogo.conf`
- Filters: `/etc/fail2ban/filter.d/traefik-*.conf`, `koprogo-*.conf`, `postgres-*.conf`
- Logs: `/var/log/fail2ban.log`

---

### 4. CrowdSec WAF (Web Application Firewall)

**Status:** ✅ Implemented

Community-powered threat intelligence and IP reputation blocking.

**Features:**
- Shared threat intelligence from CrowdSec community
- Automatic blocking of known malicious IPs
- Behavioral analysis (detects patterns like credential stuffing, scanning)
- Integration with Traefik reverse proxy

**Check status:**

```bash
sudo cscli metrics
sudo cscli decisions list  # Active bans
sudo cscli alerts list     # Recent alerts
```

**Dashboard:**

```bash
sudo cscli dashboard setup
# Access: http://localhost:3000 (Metabase)
```

**Files:**
- Config: `/etc/crowdsec/config.yaml`
- Bouncer: `/etc/crowdsec/bouncers/`

---

### 5. Suricata IDS (Intrusion Detection System)

**Status:** ✅ Implemented

Network-based intrusion detection with custom rules for KoproGo.

**Custom rules:**
- SQL injection detection
- XSS (Cross-Site Scripting) attempts
- Path traversal attacks
- Command injection
- LDAP/XXE injection
- PostgreSQL brute-force
- MinIO unauthorized access
- DDoS/flood detection
- GDPR data exfiltration attempts

**Check alerts:**

```bash
sudo tail -f /var/log/suricata/fast.log
sudo tail -f /var/log/suricata/eve.json | jq .
```

**Rules:**

```bash
# View custom rules
sudo cat /etc/suricata/rules/local.rules

# Update rules
sudo suricata-update
sudo systemctl reload suricata
```

**Files:**
- Custom rules: `/etc/suricata/rules/local.rules`
- Config: `/etc/suricata/suricata.yaml`
- Logs: `/var/log/suricata/`

---

### 6. SSH Hardening

**Status:** ✅ Implemented

Secure SSH configuration following industry best practices.

**Configuration:**
- ✅ Password authentication disabled (key-only)
- ✅ Root login prohibited (or key-only)
- ✅ Max 3 authentication attempts
- ✅ 30-second login grace time
- ✅ Modern ciphers only (ChaCha20, AES-256-GCM)
- ✅ Key exchange: Curve25519, DH-GEX-SHA256
- ✅ Strong MACs: HMAC-SHA2-512/256
- ✅ X11 forwarding disabled
- ✅ Verbose logging

**Add SSH key:**

```bash
# On your local machine
ssh-copy-id -i ~/.ssh/id_ed25519.pub koprogo@your-vps-ip

# Test connection
ssh koprogo@your-vps-ip
```

**Emergency access (if locked out):**

Use OVH console to restore `/etc/ssh/sshd_config.backup`.

**Files:**
- Config: `/etc/ssh/sshd_config`
- Backup: `/etc/ssh/sshd_config.backup`
- Keys: `/home/koprogo/.ssh/authorized_keys`

---

### 7. Kernel Hardening (sysctl)

**Status:** ✅ Implemented

System-level security hardening via kernel parameters.

**Features:**
- SYN cookies (SYN flood protection)
- IP forwarding disabled
- Source routing disabled
- ICMP redirect blocking
- Martian packet logging
- Reverse path filtering (IP spoofing protection)
- ASLR (Address Space Layout Randomization)
- Core dumps disabled
- Kernel pointer hiding

**Apply changes:**

```bash
sudo sysctl -p /etc/sysctl.d/99-koprogo-hardening.conf
```

**Files:**
- Config: `/etc/sysctl.d/99-koprogo-hardening.conf`

---

### 8. Security Auditing

**Status:** ✅ Implemented

Automated security audits and vulnerability scanning.

**Tools:**
- **Lynis:** Security auditing (weekly, Sunday 3am)
- **rkhunter:** Rootkit detection (daily, 4am)
- **AIDE:** File integrity monitoring (installed)
- **unattended-upgrades:** Automatic security updates

**Run audit manually:**

```bash
sudo /usr/local/bin/koprogo-security-audit.sh
```

**View audit logs:**

```bash
sudo tail -f /var/log/koprogo/security-audits/audit_*.log
```

**Lynis score target:** > 75/100

**Files:**
- Script: `/usr/local/bin/koprogo-security-audit.sh`
- Logs: `/var/log/koprogo/security-audits/`
- Config: `/etc/lynis/custom.prf`

---

### 9. Rust Dependency Security (cargo audit)

**Status:** ✅ Implemented — CI enforced via `.cargo/audit.toml`

Automated vulnerability scanning of all Rust crate dependencies using `cargo audit` on every CI run.

**Run locally:**

```bash
cd backend
cargo audit
```

**Configuration:** `backend/.cargo/audit.toml`

#### Accepted Risks (documented suppressions)

The following advisories are suppressed with documented justification. They must be reviewed on every new `aws-smithy-http-client` or `aws-sdk-s3` release.

---

##### RUSTSEC-2026-0049 — `rustls-webpki 0.101.7` — CRL Distribution Point matching

| Field | Value |
|-------|-------|
| **Advisory** | [RUSTSEC-2026-0049](https://rustsec.org/advisories/RUSTSEC-2026-0049) |
| **Crate** | `rustls-webpki 0.101.7` |
| **Title** | CRLs not considered authoritative by Distribution Point due to faulty matching logic |
| **Fix available** | ≥ 0.103.10 (incompatible semver with 0.101.x) |
| **Status** | **Accepted risk — upstream blocker** |

**Root cause:** `aws-smithy-http-client 1.1.12` (latest as of 2026-03-21) unconditionally bundles `hyper-rustls 0.24.2` / `rustls 0.21.x` as a legacy transport layer regardless of feature flags. The 0.101.x series has no patch — the only fix is migrating to ≥ 0.103.10 which requires `rustls 0.23.x`.

**Why low impact:** AWS SDK TLS does not use CRL Distribution Points for certificate revocation. AWS infrastructure relies on OCSP/stapling rather than CRL-based revocation, so the faulty matching logic is not exercised in practice. The 0.103.x instance was fixed (`0.103.9 → 0.103.10`, commit updating Cargo.lock).

**Mitigation:** Monitor `aws-smithy-http-client` releases. Remove suppression once a release drops the `hyper-rustls 0.24.x` / `rustls 0.21.x` legacy dependency.

---

##### RUSTSEC-2026-0002 — `lru 0.12.5` — IterMut violates Stacked Borrows (unsound)

| Field | Value |
|-------|-------|
| **Advisory** | [RUSTSEC-2026-0002](https://rustsec.org/advisories/RUSTSEC-2026-0002) |
| **Crate** | `lru 0.12.5` |
| **Title** | `IterMut` violates Stacked Borrows by invalidating internal pointer |
| **Fix available** | Upgrade to `lru ≥ 0.13` (no 0.12.x patch) |
| **Status** | **Accepted risk — transitive / no direct usage** |

**Root cause:** `lru 0.12.5` is a transitive dependency of `aws-sdk-s3 1.119.0` (latest). No 0.12.x patch exists; `cargo update` confirms 0.12.5 is the latest compatible version. koprogo never calls `lru` directly.

**Why low impact:** Theoretical soundness violation under the Stacked Borrows memory model. No known practical exploit path through `aws-sdk-s3` usage patterns (connection pooling). The issue requires exercising `IterMut` in a specific aliasing pattern that is not present in aws-sdk-s3 usage.

**Mitigation:** Monitor `aws-sdk-s3` for a release that upgrades to `lru ≥ 0.13`.

---

##### RUSTSEC-2023-0071 — `rsa 0.9.8` — Marvin Attack

| Field | Value |
|-------|-------|
| **Advisory** | [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071) |
| **Crate** | `rsa 0.9.8` |
| **Title** | Marvin Attack: timing side-channel in RSA decryption |
| **Status** | **Accepted risk — unreachable code path** |

**Root cause:** Pulled by `sqlx-mysql` (transitive). koprogo uses PostgreSQL only — the `sqlx-mysql` feature is never enabled, making this code path dead in production.

**Mitigation:** If MySQL support is ever added, re-evaluate.

---

#### Advisory History

| Date | Advisory | Action |
|------|----------|--------|
| 2026-03-21 | RUSTSEC-2026-0049 (`rustls-webpki 0.103.9`) | **Fixed** — updated to 0.103.10 via `cargo update` |
| 2026-03-21 | RUSTSEC-2026-0049 (`rustls-webpki 0.101.7`) | Accepted risk — AWS SDK upstream blocker |
| 2026-03-21 | RUSTSEC-2026-0002 (`lru 0.12.5`) | Accepted risk — transitive dep, no direct usage |
| 2026-03-23 | RUSTSEC-2026-0066 (`astral-tokio-tar 0.5.6`) | **Fixed** — updated `testcontainers` 0.27.1 → 0.27.2 (now uses 0.6.0) |
| 2025-11-xx | RUSTSEC-2025-0111 (`tokio-tar 0.3.1`) | Accepted risk — test environment only |
| 2023-xx-xx | RUSTSEC-2023-0071 (`rsa 0.9.8`) | Accepted risk — sqlx-mysql not enabled |

---

## 📊 Monitoring Stack - Issue #41

**Status:** ✅ Implemented

Full observability stack with metrics, logs, and alerting.

### Architecture

```
┌─────────────────┐
│   Grafana       │ ← Dashboards (port 3001)
│   (Visualization)│
└────────┬────────┘
         │
    ┌────┴────┬─────────┐
    │         │         │
┌───▼───┐ ┌──▼───┐ ┌───▼────┐
│Prome- │ │ Loki │ │Alert-  │
│theus  │ │      │ │manager │
└───┬───┘ └──┬───┘ └────────┘
    │        │
┌───┴────────┴───────────┐
│  Exporters (scraping)   │
├─────────────────────────┤
│ - Node Exporter (VPS)   │
│ - PostgreSQL Exporter   │
│ - cAdvisor (containers) │
│ - Traefik metrics       │
│ - Backend /metrics      │
│ - MinIO metrics         │
└─────────────────────────┘
```

### Components

**1. Prometheus (Metrics)**
- **URL:** http://vps-ip:9090
- **Scrape interval:** 15s
- **Retention:** 30 days
- **Storage:** ~10GB for 30d retention

**2. Grafana (Dashboards)**
- **URL:** http://vps-ip:3001
- **Default credentials:** admin / (set via GRAFANA_ADMIN_PASSWORD)
- **Pre-configured datasources:** Prometheus, Loki
- **Dashboards:** Auto-provisioned

**3. Loki (Log Aggregation)**
- **URL:** http://vps-ip:3100
- **Retention:** 7 days
- **Sources:** Docker containers, system logs, Traefik, PostgreSQL

**4. Alertmanager (Alerts)**
- **URL:** http://vps-ip:9093
- **Email notifications:** Configured via SMTP
- **Alert rules:** CPU, memory, disk, PostgreSQL, backups

### Metrics Exporters

| Exporter | Port | Metrics |
|----------|------|---------|
| Node Exporter | 9100 | CPU, RAM, disk, network, load |
| PostgreSQL Exporter | 9187 | Connections, queries, cache hit ratio, dead tuples |
| cAdvisor | 8082 | Container resources, restarts |
| Traefik | 8080 | HTTP requests, latency, errors |
| Backend | 8080 | Application metrics (via `/metrics`) |
| MinIO | 9000 | Storage, bucket stats, API calls |

### Starting Monitoring Stack

```bash
cd /home/koprogo/koprogo/monitoring
docker compose up -d

# Check status
docker compose ps

# View logs
docker compose logs -f grafana
docker compose logs -f prometheus
```

### Pre-configured Alerts

**Critical (immediate notification):**
- CPU usage > 85% for 5 minutes
- Memory usage > 85% for 5 minutes
- Disk space < 10% remaining
- PostgreSQL down
- Container down
- Backup not run in 24 hours

**Warning (12h repeat):**
- Disk space < 20%
- PostgreSQL slow queries (P99 > 5ms)
- High HTTP 5xx error rate
- PostgreSQL cache hit ratio < 95%

**Email configuration:**

Edit `monitoring/.env`:

```bash
ALERT_EMAIL=admin@example.com
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
```

### Resource Usage

**Total monitoring overhead:**
- **RAM:** ~500MB
- **Disk:** ~15GB (30d metrics + 7d logs)
- **CPU:** < 5% average
- **Network:** Negligible (local scraping)

**Acceptable on 2GB VPS (25% overhead)**

### Accessing Grafana

1. Open: http://your-vps-ip:3001
2. Login: admin / (your password)
3. Browse pre-configured dashboards:
   - KoproGo Overview
   - PostgreSQL Metrics
   - Docker Containers
   - Traefik HTTP Traffic

**Import community dashboards:**
- Node Exporter Full (ID: 1860)
- PostgreSQL Database (ID: 9628)
- Docker Monitoring (ID: 179)
- Traefik 2 (ID: 11462)

### Files

- **Docker Compose:** `monitoring/docker-compose.monitoring.yml`
- **Prometheus config:** `monitoring/prometheus/prometheus.yml`
- **Alert rules:** `monitoring/prometheus/alerts/koprogo.yml`
- **Loki config:** `monitoring/loki/loki-config.yml`
- **Promtail config:** `monitoring/promtail/promtail-config.yml`
- **Alertmanager config:** `monitoring/alertmanager/alertmanager.yml`
- **Grafana datasources:** `monitoring/grafana/provisioning/datasources/`
- **Grafana dashboards:** `monitoring/grafana/provisioning/dashboards/`

---

## 📝 Security Checklist

Production deployment security checklist:

### Pre-deployment

- [ ] Generate strong passwords (min 20 chars)
- [ ] Create SSH key pair (Ed25519 recommended)
- [ ] Setup S3 bucket for backups
- [ ] Configure SMTP for alerts
- [ ] Review `inventory.ini` variables

### Post-deployment

- [ ] Change Grafana admin password
- [ ] Verify LUKS encryption active (`cryptsetup status`)
- [ ] Test backup restore procedure
- [ ] Confirm fail2ban jails active (`fail2ban-client status`)
- [ ] Check Suricata running (`systemctl status suricata`)
- [ ] Verify SSH key-only access (disable password auth)
- [ ] Run security audit (`koprogo-security-audit.sh`)
- [ ] Test email alerts (trigger test alert)
- [ ] Backup LUKS encryption key (GPG encrypted, offline storage)
- [ ] Document admin procedures

### Regular maintenance

- [ ] Weekly: Review security audit logs
- [ ] Weekly: Check Grafana dashboards for anomalies
- [ ] Monthly: Test backup restore
- [ ] Monthly: Review fail2ban banned IPs
- [ ] Quarterly: Rotate GPG backup key
- [ ] Quarterly: Update Suricata rules
- [ ] Annually: Penetration testing (optional)

---

## 🔧 Troubleshooting

### fail2ban not banning

```bash
# Check jail status
sudo fail2ban-client status traefik-auth

# Test filter
sudo fail2ban-regex /var/log/traefik/access.log /etc/fail2ban/filter.d/traefik-auth.conf

# Restart fail2ban
sudo systemctl restart fail2ban
```

### Prometheus not scraping

```bash
# Check targets
curl http://localhost:9090/api/v1/targets

# Verify exporter is running
curl http://localhost:9100/metrics  # Node Exporter
curl http://localhost:9187/metrics  # PostgreSQL Exporter

# Check Prometheus logs
docker compose -f monitoring/docker-compose.yml logs prometheus
```

### Backup failed

```bash
# Check GPG key
gpg --list-keys backup@koprogo.local

# Test S3 connection
s3cmd ls s3://koprogo-backups/

# Run backup manually with verbose output
sudo /home/koprogo/koprogo/scripts/backup-encrypted.sh
```

### Grafana dashboard not loading

```bash
# Check Grafana logs
docker compose -f monitoring/docker-compose.yml logs grafana

# Verify datasource connection
curl -u admin:password http://localhost:3001/api/datasources

# Restart Grafana
docker compose -f monitoring/docker-compose.yml restart grafana
```

### LUKS volume won't mount

```bash
# Check crypttab
sudo cat /etc/crypttab

# Manual unlock
sudo cryptsetup luksOpen /dev/vdb postgres_encrypted --key-file=/root/.koprogo-luks-key

# Check mount
sudo mount | grep encrypted

# Verify key file exists
ls -lah /root/.koprogo-luks-key
```

---

## 📚 References

**Official Documentation:**
- [Prometheus](https://prometheus.io/docs/)
- [Grafana](https://grafana.com/docs/)
- [Loki](https://grafana.com/docs/loki/)
- [fail2ban](https://www.fail2ban.org/wiki/index.php/Main_Page)
- [CrowdSec](https://docs.crowdsec.net/)
- [Suricata](https://suricata.readthedocs.io/)
- [LUKS/cryptsetup](https://gitlab.com/cryptsetup/cryptsetup)

**Best Practices:**
- [CIS Docker Benchmark](https://www.cisecurity.org/benchmark/docker)
- [OWASP Security Headers](https://owasp.org/www-project-secure-headers/)
- [ANSSI Security Recommendations](https://www.ssi.gouv.fr/en/)

---

## 🆘 Support

**Issues:** https://github.com/gilmry/koprogo/issues

**Security issues:** Email security@koprogo.com (do not create public issues)

---

**KoproGo ASBL** - Secure by Default 🔒
