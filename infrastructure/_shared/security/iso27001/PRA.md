# Plan de Reprise d'Activite (PRA) - KoproGo

## ISO 27001:2022 - A.17.1 (Information security continuity)

### 1. Objectifs

| Metrique | Objectif |
|----------|----------|
| **RTO** (Recovery Time Objective) | < 1 heure |
| **RPO** (Recovery Point Objective) | < 1 heure (prod), < 24h (staging) |
| **MTTR** (Mean Time To Recover) | < 30 minutes |

### 2. Scenarios de sinistre

| Scenario | Probabilite | Impact | RTO | Runbook |
|----------|-------------|--------|-----|---------|
| S1: Panne VPS unique | Moyen | Haut | 15 min | `restore-namespace.sh` |
| S2: Corruption base de donnees | Faible | Critique | 30 min | `failover-database.sh` |
| S3: Compromission securite | Faible | Critique | 1h | `full-restore.sh` + rotation secrets |
| S4: Panne datacenter OVH | Tres faible | Critique | 2h | Bascule region + restore |
| S5: Ransomware | Faible | Critique | 1h | Restore depuis backup immutable |

### 3. Strategie de backup

| Composant | Methode | Frequence | Retention | Stockage |
|-----------|---------|-----------|-----------|----------|
| Base PostgreSQL | pgbackrest (CrunchyData) | Continue (WAL) + horaire | 30 jours | S3 + local |
| Volumes K8s | Velero snapshots | Horaire (prod), Quotidien (staging) | 7j (horaire), 30j (quotidien) |
| Fichiers uploads | MinIO replication | Continue | 90 jours | S3 cross-region |
| Configuration | Git (infrastructure repo) | Continue | Illimite | GitHub |
| Secrets | Vault snapshots | Quotidien | 30 jours | S3 chiffre |

### 4. Procedures de restauration

#### 4.1 Restauration namespace complet
```
make pra-restore ENV=production
# Ou manuellement:
infrastructure/_shared/scripts/pra/full-restore.sh production
```

#### 4.2 Failover base de donnees
```
infrastructure/_shared/scripts/pra/failover-database.sh koprogo-production
```

#### 4.3 Restauration depuis backup Velero
```
make velero-restore BACKUP=daily-all-20260323020000 ENV=production
```

### 5. Tests PRA

| Test | Frequence | Responsable | Derniere execution |
|------|-----------|-------------|-------------------|
| Restauration namespace staging | Mensuel | Ops | - |
| Failover database staging | Trimestriel | DBA | - |
| PRA complet (simulation) | Semestriel | RSSI | - |
| Restauration fichiers | Mensuel | Ops | - |

### 6. Contacts d'urgence

| Role | Contact | Telephone |
|------|---------|-----------|
| RSSI | - | - |
| DBA | - | - |
| Ops Lead | - | - |
| OVH Support | - | 1007 |

### 7. Post-mortem

Apres chaque incident ou test PRA :
1. Rapport d'incident (cause, timeline, impact)
2. Actions correctives
3. Mise a jour de ce document
4. Communication aux parties prenantes
