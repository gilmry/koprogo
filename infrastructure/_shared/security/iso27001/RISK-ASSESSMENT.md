# Risk Assessment - KoproGo

## ISO 27001:2022 - Clause 6.1.2

### Risk Assessment Methodology

| Impact | Low (1) | Medium (2) | High (3) | Critical (4) |
|--------|---------|------------|----------|---------------|
| **Likelihood** | | | | |
| Rare (1) | 1 | 2 | 3 | 4 |
| Unlikely (2) | 2 | 4 | 6 | 8 |
| Possible (3) | 3 | 6 | 9 | 12 |
| Likely (4) | 4 | 8 | 12 | 16 |

Risk appetite: Score >= 9 requires treatment.

### Risk Register

| ID | Risk | Impact | Likelihood | Score | Treatment | Control |
|----|------|--------|------------|-------|-----------|---------|
| R01 | SQL Injection | Critical (4) | Unlikely (2) | 8 | Mitigate | Parameterized queries (sqlx), Suricata IDS |
| R02 | Data breach (GDPR) | Critical (4) | Possible (3) | 12 | Mitigate | LUKS encryption, Vault secrets, RBAC |
| R03 | DDoS attack | High (3) | Possible (3) | 9 | Mitigate | CrowdSec WAF, fail2ban, rate limiting |
| R04 | Unauthorized access | Critical (4) | Unlikely (2) | 8 | Mitigate | JWT + 2FA, SSH key-only, RBAC |
| R05 | Data loss | Critical (4) | Rare (1) | 4 | Mitigate | Velero backups, pgbackrest, PRA |
| R06 | Supply chain attack | High (3) | Unlikely (2) | 6 | Mitigate | cargo audit, npm audit, SBOM |
| R07 | Insider threat | High (3) | Rare (1) | 3 | Accept | Audit logging, RBAC, code review |
| R08 | Infrastructure failure | High (3) | Possible (3) | 9 | Mitigate | K8s HA, CrunchyData replicas, Velero |
| R09 | Ransomware | Critical (4) | Unlikely (2) | 8 | Mitigate | AIDE, rkhunter, immutable backups |
| R10 | API abuse | Medium (2) | Likely (4) | 8 | Mitigate | Rate limiting, fail2ban, monitoring |
