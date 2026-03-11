# VPS Deployment Prerequisites

**Status**: ✅ Ansible playbooks ready, awaiting VPS access

## 🎯 Infrastructure Ready to Deploy

The following infrastructure components are **fully implemented** and ready for deployment:

### ✅ Ansible Playbooks (Complete)

**Security & Monitoring Stack** (`infrastructure/ansible/security-monitoring.yml`):
- Issue #39: LUKS Encryption at Rest
- Issue #40: GPG Encrypted Backups (S3)
- Issue #41: Prometheus + Grafana + Loki Monitoring
- Issue #43: Security Hardening (fail2ban, Suricata, CrowdSec, SSH hardening, kernel hardening)

**Total**: 12+ Ansible roles, 30+ tasks, production-grade security

### 📋 VPS Configuration (Already Set)

**File**: `infrastructure/ansible/inventory.ini`

```ini
[koprogo]
koprogo-vps ansible_host=141.94.213.110 ansible_user=ubuntu ansible_ssh_private_key_file=~/.ssh/id_rsa

[koprogo:vars]
domain=koprogo.com
frontend_domain=koprogo.com
api_domain=api.koprogo.com
acme_email=gilmry+koprogo@gmail.com
```

## ⚙️ Prerequisites for Deployment

### 1. VPS Access

**Required**:
- ✅ VPS IP: `141.94.213.110` (configured)
- ✅ SSH User: `ubuntu` (configured)
- ⚠️ **NEEDED**: SSH private key access (`~/.ssh/id_rsa` or equivalent)
- ⚠️ **NEEDED**: SSH connection tested: `ssh ubuntu@141.94.213.110`

**Verification**:
```bash
# Test SSH access
ssh -i ~/.ssh/id_rsa ubuntu@141.94.213.110 "uptime"

# Should return: VPS uptime without errors
```

### 2. Environment Variables

**Required for Ansible playbook**:

```bash
# Infrastructure/.env (create this file)
export GRAFANA_ADMIN_PASSWORD="<strong-password>"
export ALERT_EMAIL="gilmry+koprogo@gmail.com"

# S3 Backup Credentials (OVH S3 or AWS S3)
export S3_ACCESS_KEY="<ovh-s3-access-key>"
export S3_SECRET_KEY="<ovh-s3-secret-key>"
export S3_BACKUP_BUCKET="koprogo-backups"

# LUKS Encryption Passphrase (store securely!)
export LUKS_PASSPHRASE="<strong-passphrase-32chars+>"

# Optional: Slack/Discord webhook for alerts
export ALERTMANAGER_WEBHOOK_URL="<slack-or-discord-webhook>"
```

### 3. DNS Configuration

**Required DNS A records** (managed via OVH/CloudFlare):

```
koprogo.com           A  141.94.213.110
api.koprogo.com       A  141.94.213.110
*.koprogo.com         A  141.94.213.110  (optional wildcard)
```

**Verification**:
```bash
# Test DNS propagation
dig koprogo.com +short
# Should return: 141.94.213.110

dig api.koprogo.com +short
# Should return: 141.94.213.110
```

### 4. VPS Block Storage (Optional - for LUKS)

**If using LUKS encryption** (Issue #39):

- ⚠️ **NEEDED**: 2 additional block volumes attached to VPS
  - `/dev/vdb` : PostgreSQL data (20-50GB)
  - `/dev/vdc` : Uploaded files (10-20GB)

**OVH Example**:
```bash
# Create block storage volumes via OVH Manager
# - Volume 1: koprogo-postgres (30GB, High Speed)
# - Volume 2: koprogo-uploads (20GB, Classic)

# Attach to VPS koprogo-vps (141.94.213.110)
```

**Verification**:
```bash
ssh ubuntu@141.94.213.110 "lsblk"

# Should show:
# vdb    252:16   0   30G  0 disk  (postgres volume)
# vdc    252:32   0   20G  0 disk  (uploads volume)
```

## 🚀 Deployment Commands

### Option A: Full Deployment (Recommended)

```bash
# 1. Source environment variables
source infrastructure/.env

# 2. Test Ansible connection
cd infrastructure/ansible
ansible -i inventory.ini koprogo -m ping

# 3. Deploy security & monitoring stack
ansible-playbook -i inventory.ini security-monitoring.yml

# Estimated duration: 15-20 minutes
# Components deployed:
# - LUKS encryption setup
# - GPG encrypted backups (daily cron)
# - Prometheus + Grafana + Loki
# - fail2ban, Suricata IDS, CrowdSec WAF
# - SSH hardening, kernel hardening
# - Alertmanager notifications
```

### Option B: Staged Deployment

**Stage 1: Security Hardening** (5 min)
```bash
ansible-playbook -i inventory.ini security-monitoring.yml --tags security
```

**Stage 2: Monitoring Stack** (10 min)
```bash
ansible-playbook -i inventory.ini security-monitoring.yml --tags monitoring
```

**Stage 3: Encrypted Backups** (5 min)
```bash
ansible-playbook -i inventory.ini security-monitoring.yml --tags backups
```

## 📊 Post-Deployment Verification

### 1. Security Components

```bash
# SSH to VPS
ssh ubuntu@141.94.213.110

# Check fail2ban jails
sudo fail2ban-client status

# Check Suricata IDS
sudo systemctl status suricata

# Check CrowdSec
sudo cscli metrics

# Check audit logs
sudo lynis audit system --quick
```

### 2. Monitoring Endpoints

**Access Grafana**:
```
URL: http://141.94.213.110:3001
User: admin
Password: <GRAFANA_ADMIN_PASSWORD>
```

**Access Prometheus**:
```
URL: http://141.94.213.110:9090
```

**Access Alertmanager**:
```
URL: http://141.94.213.110:9093
```

### 3. Backup Verification

```bash
# Check backup cron job
sudo crontab -l | grep koprogo-backup

# Manual backup test
sudo /usr/local/bin/koprogo-backup.sh

# List S3 backups
s3cmd ls s3://koprogo-backups/
```

### 4. Encryption Verification

```bash
# Check LUKS volumes
sudo cryptsetup status koprogo_postgres
sudo cryptsetup status koprogo_uploads

# Check mounts
df -h | grep koprogo
```

## 🔒 Security Notes

### LUKS Passphrase Management

**CRITICAL**: Store LUKS passphrase securely!

- ✅ Use password manager (1Password, Bitwarden)
- ✅ Physical backup (printed, safe location)
- ❌ DO NOT commit to Git
- ❌ DO NOT store in plaintext on VPS

### SSH Key Management

**Best practices**:
```bash
# Generate dedicated SSH key for VPS (if needed)
ssh-keygen -t ed25519 -C "koprogo-vps-deploy" -f ~/.ssh/koprogo_vps

# Add to ssh-agent
ssh-add ~/.ssh/koprogo_vps

# Update inventory.ini
ansible_ssh_private_key_file=~/.ssh/koprogo_vps
```

### GPG Backup Key

**Export GPG private key** (secure backup):
```bash
# After deployment, export GPG private key
ssh ubuntu@141.94.213.110 "sudo gpg --export-secret-keys -a backup@koprogo.local" > koprogo-backup-gpg-private.asc

# Store in password manager or safe location
# Required to restore backups if VPS is lost
```

## 📈 Expected Results

**After successful deployment**:

✅ PostgreSQL data encrypted with LUKS AES-XTS-512
✅ Uploads encrypted with LUKS AES-XTS-512
✅ Daily GPG encrypted backups to S3 (7d local retention)
✅ Prometheus scraping backend metrics every 15s
✅ Grafana dashboards for backend, PostgreSQL, system
✅ Loki aggregating logs (7d retention)
✅ Alertmanager sending alerts to email/Slack
✅ fail2ban protecting SSH, Traefik, API
✅ Suricata IDS monitoring network traffic
✅ CrowdSec WAF blocking malicious IPs
✅ SSH hardened (key-only, modern ciphers)
✅ Kernel hardening (sysctl security parameters)

## 🐛 Troubleshooting

### Common Issues

**1. Ansible SSH Connection Failed**
```bash
# Verify SSH key permissions
chmod 600 ~/.ssh/id_rsa

# Test connection manually
ssh -vvv ubuntu@141.94.213.110
```

**2. LUKS Setup Fails (No Block Devices)**
```bash
# Skip LUKS if no block storage
ansible-playbook -i inventory.ini security-monitoring.yml --skip-tags luks
```

**3. S3 Backup Upload Fails**
```bash
# Test S3 credentials manually
s3cmd --access_key=<key> --secret_key=<secret> ls

# Check s3cfg template
cat infrastructure/ansible/templates/s3cfg.j2
```

**4. Grafana Doesn't Start**
```bash
# Check logs
ssh ubuntu@141.94.213.110 "sudo journalctl -u grafana-server -n 50"

# Verify port not in use
ssh ubuntu@141.94.213.110 "sudo netstat -tlnp | grep 3001"
```

## 🎯 Next Steps After Deployment

**Once infrastructure is deployed**:

1. ✅ Complete Issue #78 (2FA TOTP, JWT refresh tokens)
2. ✅ Deploy KoproGo backend + frontend containers
3. ✅ Configure SSL certificates (Traefik + Let's Encrypt)
4. ✅ Implement Linky API (Issue #133 - IoT Phase 0)
5. ✅ Run production smoke tests

## 📚 References

- Infrastructure Code: `infrastructure/ansible/security-monitoring.yml`
- Inventory: `infrastructure/ansible/inventory.ini`
- Security Documentation: `infrastructure/SECURITY.md`
- LUKS Setup: `infrastructure/ansible/templates/luks-setup.sh.j2`
- Backup Script: `infrastructure/ansible/templates/backup-script.sh.j2`

---

**Status**: Ready for deployment (awaiting SSH access + environment variables)
**Blockers**: None (code complete)
**Owner**: DevOps / Tech Lead
**Estimated Deploy Time**: 15-20 minutes
