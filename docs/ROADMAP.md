# KoproGo - Roadmap 2025-2027 (Révisée)

**Date**: 2025-10-23
**Stratégie**: Bootstrap OpenCore - Rentabilité Immédiate
**Équipe**: 3 personnes (2 FTE) - Side project 20h/semaine
**Timeline**: 18-24 mois jusqu'à scale

---

## 🎯 Vision Stratégique

> **"LA meilleure solution de gestion de copropriété - Gratuite open source + Hosted 1€/mois"**

### Modèle Économique
- **Open Source (MIT)**: Core gratuit, self-hosted
- **Hosted**: 1€/mois flat OU 10€/mois (5 copros) - 35€/mois (20 copros)
- **Différenciation**: Performance (<5ms), Belgian compliance, Open-source trust

### Cibles Prioritaires
1. **Belgique**: Petites copropriétés (2-20 lots) - Auto-syndics
2. **Belgique**: Syndics professionnels (portfolio 10-50 copros)
3. **Tunisie** (Phase 2-3): Test marché low-cost émergent

---

## 📅 Phase 1: MVP Production-Ready (Mois 1-6)

**Objectif**: Version 1.0 self-hosted production-ready avec conformité belge de base

### Scope Fonctionnel

#### Core Métier (Issues Critiques)
- ✅ **#001** - Meeting Management API (6-8h)
  - CRUD assemblées générales
  - Agenda + procès-verbaux
  - Lien avec documents

- ✅ **#002** - Document Upload/Download (8-10h)
  - Upload PDF/JPG/PNG/XLSX
  - Storage local sécurisé
  - Contrôle accès par building

- ✅ **#003** - Financial Reports (10-12h)
  - Appels de fonds
  - Budget prévisionnel
  - Export comptable basique

- ✅ **#004** - Pagination & Filtering (3-4h)
  - Toutes listes paginées (max 100 items)
  - Filtres par date, statut, building
  - Performance scalable >100 copros

- ✅ **#005** - Security Hardening (10-12h)
  - Rate limiting (5 req/15min login, 100 req/min API)
  - JWT refresh tokens (15min access + 7j refresh)
  - CORS restreint (var `ALLOWED_ORIGINS`)
  - Structured logging (tracing)
  - Audit logs actions critiques

#### Belgian Compliance (Nouvelles Issues)
- 🆕 **#016** - PCN Belge Compliance (12-15h)
  - Plan Comptable Normalisé
  - Mapping expenses → comptes PCN
  - Export rapports conformes normes BE

- 🆕 **#019** - i18n FR/NL/EN (8-10h)
  - Backend: Messages trilingues
  - Frontend: `svelte-i18n`
  - Détection auto langue
  - Priorité néerlandais (60% Belgique)

- 🆕 **#020** - Multi-tenancy Parfait (10-12h)
  - Table `organizations`
  - Row-Level Security PostgreSQL
  - JWT avec `organization_id`
  - Tests isolation totale
  - Fondation hosted SaaS

### Livrables Phase 1
- ✅ Core open-source (GitHub public, license MIT)
- ✅ Docker Compose 1-click deploy
- ✅ Documentation exhaustive (setup, API, architecture)
- ✅ Tests E2E coverage >80%
- ✅ CI/CD GitHub Actions (lint, test, build)
- ✅ Benchmark performance (P99 <5ms validé)

### Effort Total Phase 1
**Total**: 67-83 heures
**Timeline**: 8-10 semaines à 20h/sem
**Échéance**: Fin Mois 6 (Juin 2025)

### Métriques de Succès
- 100% endpoints API fonctionnels
- P99 latency <5ms (validé benchmarks)
- 0 vulnérabilités critiques (cargo audit)
- 50 self-hosted instances actives
- 10 copropriétés cloud beta
- 100€ MRR (premiers clients beta)

---

## 📅 Phase 2: Hosted Beta Belgique (Mois 7-12)

**Objectif**: Lancement hosted payant en Belgique avec conformité complète + parité concurrence

### Scope Fonctionnel

#### Belgian Market Fit (Nouvelles Issues)
- 🆕 **#017** - CODA Import Bancaire (15-20h)
  - Parser .cod files (format bancaire BE)
  - Matching auto paiements → expenses
  - Réconciliation bancaire
  - Support banques BE (Fortis, ING, KBC, Belfius)

- 🆕 **#018** - Exact Online Export (10-12h)
  - Format CSV compatible Exact (logiciel compta #1 BE)
  - Mapping PCN → Comptes Exact
  - Export périodique (mois, trim, année)

- 🆕 **#022** - Belgian Council Management (6-8h)
  - Conseil copropriété (obligatoire >20 lots BE)
  - Gestion membres + mandats
  - Décisions conseil vs AG

- 🆕 **#023** - Country Regulations Engine (12-15h)
  - Support BE/FR/ES/IT/TN règles
  - Validations dynamiques selon pays
  - Extensible pour expansion géographique

#### Parité Concurrence (Issues Existantes)
- ✅ **#006** - Online Payments Stripe (15-20h)
  - Paiement CB copropriétaires
  - SEPA Direct Debit
  - Webhooks `payment.succeeded`
  - Reçus PDF auto

- ✅ **#007** - Work Management (12-15h)
  - Gestion travaux (ravalement, toiture)
  - Multi-devis comparaison
  - Workflow: Proposition → Vote AG → Réalisation
  - Galerie photos avant/après

- ✅ **#008** - Ticketing System (8-10h)
  - Déclaration incidents copropriétaires
  - Affectation prestataires
  - Upload photos problème
  - Historique interventions

- ✅ **#009** - Notifications Multi-Canal (8-10h)
  - Email (SendGrid/SMTP)
  - Push web (Service Worker)
  - In-app (cloche)
  - Préférences utilisateur
  - Queue asynchrone (Redis optionnel)

- ✅ **#010** - Progressive Web App (10-12h)
  - Manifest.json installation
  - Service Worker cache
  - Mode offline IndexedDB
  - Background sync
  - Push notifications

#### Hosted Monetization
- 🆕 **#021** - Stripe Billing 1€/Mois (6-8h)
  - Plans: Free (1 copro), Starter (10€/5 copros), Pro (35€/20 copros)
  - Subscriptions Stripe
  - Webhooks auto-activation
  - Self-service upgrade/downgrade

### Infrastructure Phase 2
- K3s deployment (VPS Hetzner)
- Traefik reverse proxy
- PostgreSQL managed (ou self-hosted optimisé)
- Monitoring (Prometheus + Grafana)
- Backups automatiques

### Livrables Phase 2
- ✅ Plateforme hosted live (app.koprogo.com)
- ✅ Signup self-service
- ✅ Billing Stripe opérationnel
- ✅ Conformité belge 100%
- ✅ Documentation utilisateur FR/NL
- ✅ Support email (< 24h response time)

### Effort Total Phase 2
**Total**: 102-127 heures
**Timeline**: 13-16 semaines à 20h/sem
**Échéance**: Fin Mois 12 (Décembre 2025)

### Métriques de Succès
- 50 clients cloud payants (Belgique)
- 300 copropriétés gérées
- 1,200€ MRR
- Churn <5%/mois
- NPS >50
- Support satisfaction >90%
- Lighthouse PWA score >90
- 3,000 GitHub stars

---

## 📅 Phase 3: Scale + Innovation (Mois 13-24)

**Objectif**: Croissance organique, différenciation marché, expansion France + Tunisie

### Scope Fonctionnel

#### Différenciation Compétitive
- ✅ **#013** - Sustainability (12-15h) - **MONTÉ PRIORITÉ**
  - Bilan carbone immeuble
  - Tracking DPE (Performance Énergétique Belgique)
  - Recommandations travaux isolation
  - Calculateur aides MaPrimeRénov' (FR) / Primes Énergie (BE)
  - **Argument marketing unique**: "Green SaaS" + <0.5g CO2/req
  - Certification B Corp potentielle

- ✅ **#014** - Analytics & BI (12-15h) - **MONTÉ PRIORITÉ**
  - Dashboard multi-copropriétés (syndics pro)
  - KPIs temps réel (taux recouvrement, charges/m²)
  - Benchmarking vs moyenne marché
  - Rapports clients auto-générés PDF
  - **Premium feature** payante (plan Pro/Enterprise)

- ✅ **#011** - AI Features (20-30h)
  - OCR factures (extraction auto montant/fournisseur/catégorie)
  - Prédiction charges futures (ML)
  - Détection anomalies dépenses
  - Chatbot assistant copropriétaires
  - Classification auto documents
  - **Tech**: Python microservice (FastAPI) + Azure CV ou Tesseract

#### Expansion Géographique
- 🆕 **#024** - Multi-Currency EUR/TND (6-8h)
  - Support Dinar Tunisien
  - Formatage montants selon locale
  - Conversion taux change (API externe)

#### Scale Infrastructure
- ScyllaDB (si >10k users) ou optimisation PostgreSQL
- Redis cache distributed
- CDN pour assets statiques
- Auto-scaling K3s

### Marketing & Growth Phase 3
- **SEO**: Top 10 Google pour 10 mots-clés BE/FR
- **Communauté**: 10,000 GitHub stars, 100+ contributors
- **Partenariats**: Associations copropriétaires BE (SNPC, APC)
- **Content**: 2 blog posts/semaine, case studies clients
- **Ads micro-budget**: 300€/mois Google Ads BE/FR

### Livrables Phase 3
- ✅ Expansion France (hosted)
- ✅ Beta Tunisie (hosted low-cost 5€/mois)
- ✅ Certification "Green SaaS"
- ✅ Features IA opérationnelles
- ✅ Analytics BI pour syndics pro

### Effort Total Phase 3
**Total**: 50-68 heures (features clés)
**Timeline**: 10-13 semaines à 20h/sem
**Échéance**: Mois 18-20

### Métriques de Succès
- 150 clients cloud (BE + FR + TN)
- 1,000 copropriétés gérées
- 4,000€ MRR
- Churn <5%/mois
- NPS >60
- 10,000 GitHub stars
- 500 self-hosted instances
- OCR accuracy >90%

---

## 📅 Phase 4: Leadership & Long Terme (Mois 24+)

**Objectif**: Domination marché niche (petites copropriétés) + Innovation continue

### Scope Fonctionnel

#### Marketplace & Écosystème
- ✅ **#012** - Marketplace Prestataires (20-25h)
  - Annuaire prestataires vérifiés (plombiers, électriciens, etc.)
  - Notation et avis
  - Demande devis en ligne
  - **Business model**: Commission 5-10% sur contrats signés

#### Mobile Native (Optionnel)
- ✅ **#015** - Mobile Native React Native (30-40h)
  - App iOS/Android
  - Auth biométrique (Face ID, Touch ID)
  - Scanner QR codes factures
  - Push notifications natives
  - Photos haute résolution
  - **Coût**: 150€ (comptes dev Apple + Google)
  - **Alternative**: PWA peut suffire (à évaluer selon feedback users)

#### Innovation Continue
- 🆕 **#025** - TLIS Integration Tunisie (15-20h)
  - Intégration cadastre tunisien
  - Vérification propriété

- Features communauté (crowdsourced)
- Plugins ecosystem (si succès OpenCore)

### Expansion Géographique
- Luxembourg (marché premium)
- Espagne (>4 lots règle)
- Italie (>4 lots règle)
- Algérie/Maroc (depuis Tunisie)

### Livrables Phase 4
- ✅ Marketplace live avec 100+ prestataires
- ✅ Apps stores (si mobile native)
- ✅ Présence 5+ pays

### Métriques de Succès
- 350+ clients cloud
- 2,100+ copropriétés
- 7,000€+ MRR
- Team full-time (3 personnes vivent de KoproGo)
- Marketplace revenue 500€/mois
- Mobile app (si lancée): >1000 downloads, rating >4.5/5

---

## 📊 Synthèse Timeline & Effort

| Phase | Durée | Issues | Effort | MRR Cible | Copros |
|-------|-------|--------|--------|-----------|--------|
| **Phase 1** | Mois 1-6 | #001-#005, #016, #019, #020 | 67-83h | 100€ | 60 |
| **Phase 2** | Mois 7-12 | #006-#010, #017-#018, #021-#023 | 102-127h | 1,200€ | 480 |
| **Phase 3** | Mois 13-20 | #011, #013-#014, #024 | 50-68h | 4,000€ | 1,000 |
| **Phase 4** | Mois 20+ | #012, #015, #025 | 65-85h | 7,000€+ | 2,100+ |
| **TOTAL** | 24 mois | 24 issues | 284-363h | - | - |

**Équivalent**: 36-45 semaines de développement réel (side project 20h/sem)
**Étalement**: 24 mois (parallèle avec marketing, support, ops)

---

## 🎯 Critères de Passage entre Phases

### Phase 1 → Phase 2
- ✅ Tests coverage >80%
- ✅ 0 bugs critiques
- ✅ 50 self-hosted instances actives
- ✅ 10 clients beta satisfaits (NPS >50)
- ✅ Performance P99 <5ms validée
- ✅ Documentation complète FR/NL

### Phase 2 → Phase 3
- ✅ 50 clients cloud payants
- ✅ 1,200€ MRR (rentable)
- ✅ Churn <5%/mois
- ✅ Support <24h response time
- ✅ Infrastructure stable (99.9% uptime)
- ✅ Conformité belge 100% validée par beta users

### Phase 3 → Phase 4
- ✅ 150 clients cloud
- ✅ 4,000€ MRR
- ✅ NPS >60
- ✅ Présence France + Belgique établie
- ✅ Features IA/Analytics adoptées (>30% users)
- ✅ 10,000 GitHub stars (communauté forte)

---

## 🚨 Risques & Mitigations

### Risque: Timeline trop Optimiste (Side Project)
**Mitigation**:
- Buffer 20% sur chaque phase
- Priorisation ruthless (80/20)
- Contributions communauté (issues "good first issue")
- Pas de deadline stricte (bootstrap = pas de pression investisseurs)

### Risque: Concurrence Réagit (Vilogi baisse prix)
**Mitigation**:
- Open-source = moat (communauté fidèle)
- Performance technique (difficile à copier rapidement)
- Crédibilité anti-lock-in (différenciation long-terme)
- First-mover OpenCore copropriété

### Risque: Adoption Belgique Lente
**Mitigation**:
- Partenariats associations copropriétaires
- SEO agressif (top 10 Google "logiciel syndic belgique")
- Freemium généreux (1 copro gratuite = essai sans risque)
- Cas d'usage case studies (syndics satisfaits)

### Risque: Complexité Belgian Compliance Sous-Estimée
**Mitigation**:
- Consultation syndic professionnel belge (validation specs)
- Tests beta avec vrais syndics BE
- Itération rapide sur feedback
- Documentation juridique claire (avec disclaimer)

---

## 💰 Revenus Cumulés (Projections Réalistes)

| Fin Phase | MRR | ARR | Clients | Copros | Cash Cumul |
|-----------|-----|-----|---------|--------|------------|
| **Phase 1** (M6) | 190€ | 2,280€ | 10 | 60 | 855€ |
| **Phase 2** (M12) | 1,600€ | 19,200€ | 80 | 480 | 11,215€ |
| **Phase 3** (M20) | 4,000€+ | 48,000€+ | 150 | 1,000 | 35,000€+ |
| **Phase 4** (M24) | 7,000€+ | 84,000€+ | 350 | 2,100+ | 51,095€+ |

**Break-even**: Mois 2 (rentable dès début)
**Viabilité full-time**: Mois 20-24 (3 personnes peuvent vivre de KoproGo)

---

## 🎉 Vision Long Terme (Année 5+)

- **10,000+ copropriétés gérées** (Belgique, France, Luxembourg, Tunisie, Maghreb)
- **500+ clients payants**
- **20,000€+ MRR** (240k€ ARR)
- **Équipe 5-7 personnes** full-time
- **Leader européen open-source copropriété**
- **Acquisition possible** (si souhaité) ou croissance indépendante
- **Impact écologique**: Millions kg CO2 économisés via tracking PEB

---

**Prochaine étape**: PRIORITIES_TABLE.md (tableau unique consolidé)

**Dernière mise à jour**: 2025-10-23
**Auteur**: KoproGo Strategy Team
