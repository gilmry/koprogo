# Plan de Continuite d'Activite (PCA) - KoproGo

## ISO 27001:2022 - A.17 (Information security aspects of business continuity)

### 1. Services critiques

| Service | RTO | RPO | Criticite |
|---------|-----|-----|-----------|
| API Backend | 15 min | 0 (stateless) | Critique |
| Frontend | 15 min | 0 (stateless) | Haute |
| PostgreSQL | 30 min | < 1h | Critique |
| MinIO (documents) | 1h | < 24h | Moyenne |
| Monitoring | 2h | N/A | Basse |

### 2. Architecture de resilience

#### VPS (monosite)
- Backup quotidien GPG + S3
- GitOps auto-redeploy (3 min)
- Script rollback

#### K3s/K8s
- CrunchyData PostgreSQL HA (1 primary + 1 replica)
- Velero backups horaires (production)
- Pod anti-affinity (spread across nodes)
- PodDisruptionBudget (minAvailable: 1)
- HPA (autoscaling 2-5 pods, multisite)

### 3. Niveaux de service

| Environnement | SLA | Maintenance window |
|---------------|-----|--------------------|
| Production | 99.9% (8.7h/an downtime) | Dimanche 2h-4h CET |
| Staging | 99% | Libre |
| Integration | Best effort | Libre |
| Dev | Best effort | Libre |
