# KoproGo Documentation

Documentation complète du projet KoproGo ASBL.

---

## 🗺️ Roadmap

**📅 [ROADMAP.md](ROADMAP.md)** - Plan de développement Nov 2025 - Août 2026

Phases:
- **Phase 1 (VPS MVP)**: Sécurité, GDPR, Backups, Board Tools (Nov 2025 - Fév 2026)
- **Phase 2 (K3s)**: Voting, Community Features, Contractor Backoffice (Mar - Mai 2026)
- **Phase 3 (K8s)**: Performance, Real-time, Mobile App (Jun - Août 2026)

Voir aussi:
- [Software Roadmap (GitHub Project #2)](https://github.com/users/gilmry/projects/2)
- [Infrastructure Roadmap (GitHub Project #3)](https://github.com/users/gilmry/projects/3)

---

## 📚 Structure Documentation

### 🎯 Guides Principaux

#### Déploiement
- **[deployment/](deployment/)** ⭐ **NOUVEAU**
  - [Vue d'ensemble](deployment/index.md) - TL;DR: `make setup-infra`
  - [Configuration OVH](deployment/ovh-setup.md) - Compte, OpenStack, credentials
  - [Terraform + Ansible](deployment/terraform-ansible.md) - Détails techniques
  - [GitOps Auto-Update](deployment/gitops.md) - Service systemd (3 minutes)
  - [Troubleshooting](deployment/troubleshooting.md) - Résolution de problèmes

#### Business & Stratégie
- **[ECONOMIC_MODEL.md](ECONOMIC_MODEL.md)** ⭐
  - Modèle économique complet ASBL 2025-2030
  - Structure juridique ASBL belge et gouvernance
  - Modèle hybride OpenCore (Cloud 1€/copro + Self-hosted gratuit)
  - Projections, viabilité financière, transparence comptable
  - Exemples open source réussis et opportunités de soutien

#### Performance & Tests
- **[PERFORMANCE_REPORT.md](PERFORMANCE_REPORT.md)** ⭐
  - Tests de charge officiels (Grafana k6)
  - 99.74% uptime, 287 req/s sur 1 vCPU
  - Calculs CO₂ (0.12g/req OVH France)
  - Capacité: 1,000-1,500 copros par vCPU

- **[E2E_TESTING_GUIDE.md](E2E_TESTING_GUIDE.md)**
  - Tests E2E avec Playwright
  - Génération automatique de vidéos
  - Tests d'accessibilité et performance

- **[PERFORMANCE_TESTING.md](PERFORMANCE_TESTING.md)**
  - Guide des tests de charge
  - Configuration k6 et scenarios

#### Développement
- **[MAKEFILE_GUIDE.md](MAKEFILE_GUIDE.md)** ⭐
  - Toutes les commandes make disponibles
  - `make setup` - Setup complet développement
  - `make setup-infra` - Déploiement VPS
  - `make test` - Tous les tests
  - `make ci` - Pipeline CI

- **[PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md)**
  - Architecture hexagonale (Ports & Adapters)
  - Organisation du code backend Rust
  - Organisation du code frontend Astro/Svelte

---

## 📖 Guide de Lecture Recommandé

### Pour Nouveaux Contributeurs
1. **README.md** (racine) : Vue d'ensemble projet
2. **ECONOMIC_MODEL.md** : Comprendre le modèle ASBL
3. **deployment/** : Déployer sa propre instance
4. **CLAUDE.md** (racine) : Architecture technique hexagonale

### Pour ASBL / Admins
1. **ECONOMIC_MODEL.md** : Stratégie, finances et gouvernance
2. **deployment/** : Maintenir infra cloud
3. **PERFORMANCE_REPORT.md** : Benchmarks et capacités

### Pour Utilisateurs Self-Hosted
1. **deployment/** : Installation automatique avec `make setup-infra`
2. **PERFORMANCE_REPORT.md** : Dimensionner son VPS

---

## 🔄 Maintenance Documentation

**Fréquence updates** :
- **ECONOMIC_MODEL.md** : Annuel (ou si changement majeur)
- **PERFORMANCE_REPORT.md** : Après chaque test de charge
- **deployment/** : Quand nouvelle version infrastructure
- **MAKEFILE_GUIDE.md** : Quand nouvelles commandes make
- **changelog.md** : À chaque release

**Responsable** : ASBL KoproGo (Conseil d'Administration)

---

## 🗂️ Fichiers Documentation

| Fichier | Description | Status |
|---------|-------------|--------|
| **deployment/** | Guide déploiement VPS complet | ✅ À jour |
| **ECONOMIC_MODEL.md** | Modèle économique ASBL 2025-2030 | ✅ À jour |
| **PERFORMANCE_REPORT.md** | Tests de charge officiels | ✅ À jour |
| **MAKEFILE_GUIDE.md** | Commandes make | ✅ À jour |
| **E2E_TESTING_GUIDE.md** | Tests E2E Playwright | ✅ À jour |
| **PERFORMANCE_TESTING.md** | Tests de charge k6 | ✅ À jour |
| **PROJECT_STRUCTURE.md** | Architecture codebase | ✅ À jour |
| **changelog.md** | Historique versions | ✅ À jour |

---

## 📚 Documentation Technique (RST)

Documentation générée automatiquement pour Sphinx :

- **backend/** : Documentation Rust (entités, services, use cases)
- **frontend/** : Documentation Frontend (à générer)

---

**Dernière mise à jour** : Octobre 2025

**Contact** : contact@koprogo.com (à créer)
