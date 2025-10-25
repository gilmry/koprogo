# KoproGo Documentation

Documentation complète du projet KoproGo ASBL.

## 📚 Structure Documentation

### 🎯 Documents Principaux (À Jour 2025)

#### Business & Stratégie
- **[BUSINESS_PLAN_BOOTSTRAP.md](BUSINESS_PLAN_BOOTSTRAP.md)** ⭐
  - **LE** business plan officiel ASBL 2025-2028
  - Modèle hybride OpenCore (Cloud 1€/copro + Self-hosted gratuit)
  - Projections, team, roadmap, unit economics
  - **Utiliser celui-ci** (le plus récent et complet)

- **[BUSINESS_PLAN.md](BUSINESS_PLAN.md)** 📦
  - Version précédente (modèle startup avec levée de fonds)
  - **ARCHIVÉ** : Ne plus utiliser, remplacé par BUSINESS_PLAN_BOOTSTRAP.md
  - Gardé pour historique seulement

#### Infrastructure & Déploiement
- **[VPS_DEPLOYMENT.md](VPS_DEPLOYMENT.md)** ⭐
  - Guide complet déploiement VPS OVH France
  - Pour ASBL cloud multi-tenant OU self-hosting manuel
  - Architecture, setup, maintenance, troubleshooting

- **[DEPLOY_GITOPS.md](DEPLOY_GITOPS.md)** ⭐ **NOUVEAU**
  - Déploiement self-hosted avec auto-update GitHub
  - Installation 1-click, GitOps, rollback, backups
  - **Recommandé pour utilisateurs self-hosted**

- **[INFRASTRUCTURE_ROADMAP.md](INFRASTRUCTURE_ROADMAP.md)** ⭐
  - Évolution infrastructure selon croissance
  - Phase 1 (VPS) → Phase 4 (Kubernetes HA)
  - Capacités validées par tests de charge

#### Performance & Technique
- **[PERFORMANCE_REPORT.md](PERFORMANCE_REPORT.md)** ⭐
  - Tests de charge officiels (Grafana k6)
  - 99.74% uptime, 287 req/s sur 1 vCPU
  - Calculs CO₂ (0.12g/req OVH France)
  - Capacité: 1,000-1,500 copros par vCPU

### 📦 Documents Archivés (docs/archive/)

**✅ Nettoyage effectué le 25/10/2025**

Tous les documents obsolètes ont été déplacés vers `docs/archive/` :

**archive/** (anciens business plans):
- BUSINESS_PLAN.md (modèle startup avec levée de fonds)
- ROADMAP.md (roadmap obsolète)
- MARKET_ANALYSIS.md
- ANALYSIS.md
- NEW_ISSUES.md
- ISSUE_004_COMPLETION_GUIDE.md
- PRIORITIES_TABLE.md

**archive/root-md/** (fichiers root obsolètes):
- DEPLOYMENT_VPS.md (doublon de VPS_DEPLOYMENT.md)
- infrastructure.md (stub)

**archive/load-tests-troubleshooting/** (problèmes résolus):
- PANIC_FIXES.md (fixes Oct 24-25)
- TROUBLESHOOTING_401.md (fixes JWT)
- IMPLEMENTATION_SUMMARY.md (rate limiting)
- CHANGELOG_RATE_LIMITING.md
- SESSION_SUMMARY.md (backend session)


## 📖 Guide de Lecture Recommandé

### Pour Nouveaux Contributeurs
1. **README.md** (racine) : Vue d'ensemble projet
2. **BUSINESS_PLAN_BOOTSTRAP.md** : Comprendre le modèle ASBL
3. **DEPLOY_GITOPS.md** : Installer sa propre instance
4. **CLAUDE.md** (racine) : Architecture technique hexagonale

### Pour ASBL / Admins
1. **BUSINESS_PLAN_BOOTSTRAP.md** : Stratégie et finances
2. **VPS_DEPLOYMENT.md** : Maintenir infra cloud
3. **INFRASTRUCTURE_ROADMAP.md** : Planifier scaling
4. **PERFORMANCE_REPORT.md** : Benchmarks et capacités

### Pour Utilisateurs Self-Hosted
1. **DEPLOY_GITOPS.md** : Installation automatique
2. **VPS_DEPLOYMENT.md** : Setup manuel détaillé
3. **PERFORMANCE_REPORT.md** : Dimensionner son VPS

## 🔄 Maintenance Documentation

**Fréquence updates** :
- **BUSINESS_PLAN_BOOTSTRAP.md** : Annuel (ou si changement majeur)
- **PERFORMANCE_REPORT.md** : Après chaque test de charge
- **VPS_DEPLOYMENT.md** : Quand nouvelle version infrastructure
- **DEPLOY_GITOPS.md** : Quand workflow GitOps change
- **changelog.md** : À chaque release

**Responsable** : ASBL KoproGo (Conseil d'Administration)

---

**Dernière mise à jour** : Janvier 2025

**Contact** : contact@koprogo.com (à créer)
