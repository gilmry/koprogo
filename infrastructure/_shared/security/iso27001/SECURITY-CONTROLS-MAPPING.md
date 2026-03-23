# Security Controls Mapping - KoproGo

## ISO 27001:2022 Annex A Controls -> Technical Implementation

### A.5 Organizational Controls
| Control | Description | Implementation |
|---------|-------------|----------------|
| A.5.1 | Policies for information security | This document + CLAUDE.md |
| A.5.2 | Information security roles | RBAC (owner/syndic/admin), K8s RBAC |

### A.6 People Controls
| Control | Description | Implementation |
|---------|-------------|----------------|
| A.6.1 | Screening | GitHub branch protection, code review |
| A.6.8 | Information security event reporting | Alertmanager + ElastAlert2 notifications |

### A.7 Physical Controls
| Control | Description | Implementation |
|---------|-------------|----------------|
| A.7.1 | Physical security perimeters | OVH datacenter (ISO 27001 certified) |

### A.8 Technological Controls
| Control | Description | Implementation |
|---------|-------------|----------------|
| A.8.1 | User endpoint devices | N/A (SaaS - browser-based) |
| A.8.2 | Privileged access rights | SSH key-only, sudo audit, K8s RBAC |
| A.8.3 | Information access restriction | JWT + role-based permissions |
| A.8.5 | Secure authentication | JWT + 2FA (TOTP), bcrypt password hashing |
| A.8.6 | Capacity management | K8s HPA, resource limits, monitoring |
| A.8.7 | Protection against malware | rkhunter (daily), CrowdSec, Suricata IDS |
| A.8.8 | Management of technical vulnerabilities | Lynis (weekly), cargo audit, npm audit |
| A.8.9 | Configuration management | Terraform + Ansible + Helm (IaC, GitOps) |
| A.8.10 | Information deletion | GDPR erasure (DELETE /gdpr/erase) |
| A.8.11 | Data masking | Anonymization in GDPR export |
| A.8.12 | Data leakage prevention | Network policies, CORS, CSP headers |
| A.8.15 | Logging | Prometheus metrics, ELK logs, auditd |
| A.8.16 | Monitoring activities | Grafana dashboards, Kibana, Alertmanager |
| A.8.20 | Networks security | NetworkPolicy, Suricata IDS, CrowdSec WAF |
| A.8.21 | Security of network services | TLS (Let's Encrypt), HSTS, mTLS (Vault) |
| A.8.24 | Use of cryptography | LUKS (AES-XTS-512), GPG backups, bcrypt |
| A.8.25 | Secure development lifecycle | TDD, BDD, CI/CD, branch protection |
| A.8.26 | Application security requirements | OWASP Top 10, parameterized queries |
| A.8.28 | Secure coding | Rust (memory safety), Clippy, cargo audit |
| A.8.31 | Separation of environments | dev/integration/staging/production namespaces |
| A.8.32 | Change management | Git branching, PR reviews, ArgoCD GitOps |
| A.8.33 | Test information | Testcontainers, isolated test DBs |
| A.8.34 | Audit information systems | AIDE (daily), Lynis (weekly), auditd |

### A.9 Access Control (legacy mapping)
| Control | Implementation |
|---------|----------------|
| A.9.1 Access control policy | JWT + RBAC (6 roles) |
| A.9.2 User management | /users endpoints, /auth/login |
| A.9.4 System access control | SSH key-only, 2FA, fail2ban |

### A.12 Operations Security (legacy mapping)
| Control | Implementation |
|---------|----------------|
| A.12.1 Operational procedures | Ansible playbooks, Makefile targets |
| A.12.2 Malware protection | rkhunter, CrowdSec, Suricata |
| A.12.3 Backup | Velero (K8s), pgbackrest, GPG backups (VPS) |
| A.12.4 Logging & monitoring | ELK + Prometheus + Grafana + auditd |
| A.12.6 Vulnerability management | Lynis, cargo audit, npm audit |

### A.17 Business Continuity
| Control | Implementation |
|---------|----------------|
| A.17.1 Information security continuity | PRA runbooks (full-restore.sh) |
| A.17.2 Redundancies | CrunchyData HA, K8s multi-replica, Velero |

### A.18 Compliance
| Control | Implementation |
|---------|----------------|
| A.18.1 Legal requirements | GDPR (Articles 15-18, 21), Belgian law |
| A.18.2 Security reviews | Lynis weekly audits, annual pentest (planned) |
