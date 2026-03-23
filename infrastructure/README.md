# KoproGo Infrastructure

## Architecture Overview

```
                    ┌──────────────────────────────────────────┐
                    │              GitHub (Source)              │
                    │   main -> integration -> staging -> prod │
                    └────────────┬─────────────────────────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              │                  │                  │
         ┌────▼─────┐    ┌──────▼──────┐    ┌─────▼──────┐
         │  VPS      │    │   K3s       │    │    K8s     │
         │ (Docker   │    │ (single     │    │ (multi-    │
         │  Compose) │    │  node)      │    │  node)     │
         └────┬──────┘    └──────┬──────┘    └─────┬──────┘
              │                  │                  │
         GitOps Script     ArgoCD + Helm      ArgoCD + Helm
         (systemd)         (Kustomize infra)  (Kustomize infra)
```

## Directory Structure

```
infrastructure/
  _shared/                    # Shared configs (DRY)
    terraform/modules/        # OVH VPS, K3s, K8s Terraform modules
    ansible/roles/            # 12 Ansible roles (common, docker, security, etc.)
    docker-compose/           # Base Docker Compose (parameterized)
    helm/
      koprogo/                # App Helm chart (backend, frontend, postgres, minio)
      monitoring/             # Monitoring Helm chart (Prometheus + ELK)
      vault/                  # HashiCorp Vault values
      velero/                 # Velero backup values
    kustomize/base/           # K8s infra (namespace, SA, ingress)
    argocd/                   # ApplicationSets (branch -> env mapping)
    monitoring/               # Prometheus, Grafana, ELK configs (VPS)
    security/                 # ISO 27001 docs, hardening configs
    secrets/                  # SOPS configuration
    scripts/                  # GitOps, setup, PRA runbooks

  monosite/
    vps/{dev,integration,staging,production}/
    k3s/{dev,integration,staging,production,local}/

  multisite/
    k8s/{dev,integration,staging,production}/
```

## Quick Start

### Local Development (Docker Compose)
```bash
make dev                    # Start with hot reload (root docker-compose.yml)
```

### Local Kubernetes (Docker Desktop / minikube / k3d)
```bash
make local-k8s-up           # Deploy on local K8s cluster
make local-k8s-status       # Check status
make local-k8s-forward      # Port-forward services
make local-k8s-down         # Teardown
```

### VPS Deployment (Docker Compose)
```bash
make deploy-dev-vps         # Deploy dev environment
make deploy-prod-vps        # Deploy production
make vps-logs ENV=production
```

### K3s/K8s Deployment (Helm + ArgoCD)
```bash
make helm-install ENV=dev ARCH=k3s
make helm-upgrade ENV=staging ARCH=k3s
make helm-rollback ENV=production REVISION=3
make helm-status ENV=production
```

### Full Infrastructure Provisioning
```bash
make provision ENV=production ARCH=vps  # Terraform + Ansible + Deploy
make provision ENV=production ARCH=k3s  # Terraform + Ansible (ArgoCD handles deploy)
```

## Deployment Methods

| Method | Architecture | GitOps | Rollback |
|--------|-------------|--------|----------|
| Docker Compose | VPS (monosite) | systemd script | `git checkout` |
| Helm + ArgoCD | K3s (monosite) | ArgoCD auto-sync | `helm rollback` |
| Helm + ArgoCD | K8s (multisite) | ArgoCD auto-sync | `helm rollback` |
| Helm (local) | Docker Desktop/minikube | Manual | `helm rollback` |

## Branching Strategy

```
feature/* --> main --> integration --> staging --> production
                        (auto)       (PR manual)  (PR + approval)
```

| Branch | Environment | Auto-deploy |
|--------|-------------|-------------|
| main | - (dev images built) | CI only |
| dev | dev | Manual |
| integration | integration | Auto |
| staging | staging | Auto |
| production | production | Auto |

## Security (ISO 27001)

All environments include:
- **fail2ban**: Brute-force protection (SSH, API, DB)
- **CrowdSec**: Community threat intelligence WAF
- **Suricata**: Intrusion Detection System (IDS)
- **rkhunter**: Daily rootkit scanning
- **Lynis**: Weekly security auditing
- **AIDE**: Daily file integrity monitoring
- **auditd**: System call auditing
- **SSH hardening**: Key-only, modern ciphers, no root
- **Kernel hardening**: sysctl (SYN cookies, ASLR, no redirects)

See `_shared/security/iso27001/` for full ISO 27001 documentation.

## Secrets Management

| Environment | Method |
|-------------|--------|
| Local dev | Plain text (.env) |
| Dev/Integration | SOPS + age |
| Staging/Production | HashiCorp Vault + ESO |
| VPS (Ansible) | Ansible Vault |

```bash
make secrets-keygen          # Generate age key (once)
make secrets-encrypt ENV=production ARCH=vps
make secrets-decrypt ENV=production ARCH=vps
make secrets-edit ENV=production ARCH=vps
```

## Monitoring (Double Pipeline)

- **Metrics**: Prometheus -> Grafana + Alertmanager
- **Logs**: Filebeat -> Elasticsearch -> Kibana + ElastAlert2

```bash
make monitoring-up ENV=production ARCH=vps
make monitoring-status ENV=production ARCH=k3s
```

## Backup & PRA

- **PostgreSQL**: CrunchyData PGO (HA) + pgbackrest
- **Kubernetes**: Velero (hourly prod, daily all)
- **PRA Scripts**: `_shared/scripts/pra/`

```bash
make velero-backup ENV=production
make velero-status
make pra-restore ENV=production
```

## Environment Replication

```bash
make replicate-env TARGET=staging ARCH=vps
```

## Makefile Reference

```bash
make infra-help             # Show all infrastructure targets
```
