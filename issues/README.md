# KoproGo - Roadmap des Fonctionnalités Futures

Ce dossier contient **15 issues détaillées** avec cahiers des charges complets pour l'évolution de la plateforme KoproGo.

---

## 📊 Vue d'Ensemble

| Catégorie | Nombre | Estimation Totale | Priorité |
|-----------|--------|-------------------|----------|
| **Critique** | 5 | 37-46 heures | 🔴 À faire en priorité |
| **Important** | 5 | 58-72 heures | 🟡 Phase 2 |
| **Nice-to-Have** | 5 | 94-125 heures | 🟢 Long terme |
| **TOTAL** | **15** | **189-243 heures** | ~5-6 semaines |

---

## 🔴 Issues Critiques (MVP Complet)

Ces fonctionnalités sont **essentielles** pour une version 1.0 production-ready.

### [#001 - Gestion des Assemblées Générales](./critical/001-meeting-management-api.md)
**Estimation** : 6-8h | **Labels** : `backend`, `domain-logic`

Implémenter les use cases et API pour les assemblées générales (AG). Les entités et migrations existent déjà.

**Livrables** :
- Use Cases : `MeetingUseCases` avec CRUD complet
- Endpoints : 10 routes (création, modification, agenda, PV, présences)
- Tests : E2E + BDD Gherkin
- Intégration avec `Document` pour PV

**Valeur métier** : Obligation légale pour syndics de copropriété.

---

### [#002 - Gestion Documentaire Complète](./critical/002-document-upload-download.md)
**Estimation** : 8-10h | **Labels** : `backend`, `frontend`, `file-storage`

Système complet upload/download de fichiers avec stockage sécurisé.

**Livrables** :
- Trait `FileStorageService` + implémentation locale
- Upload multipart/form-data (10MB max)
- Validation types MIME (PDF, JPG, PNG, XLSX)
- Contrôle d'accès par `building_id`
- Composant Svelte `FileUpload`

**Valeur métier** : Archivage légal de documents (PV, factures, règlements).

---

### [#003 - Génération de Rapports Financiers](./critical/003-financial-reports-generation.md)
**Estimation** : 10-12h | **Labels** : `backend`, `frontend`, `finance`

6 types de rapports : appels de fonds, budget, impayés, FEC (export comptable).

**Livrables** :
- `ReportUseCases` intégrant `ExpenseCalculator` existant
- Génération PDF (printpdf) et Excel (rust_xlsxwriter)
- Export FEC conforme DGFiP
- Composant `ReportDashboard.svelte`

**Valeur métier** : Transparence financière obligatoire en AG.

---

### [#004 - Pagination et Filtrage](./critical/004-pagination-filtering.md)
**Estimation** : 3-4h | **Labels** : `backend`, `performance`

Ajouter pagination/filtrage à tous les endpoints de liste.

**Livrables** :
- Structs génériques `PageRequest`, `PageResponse`, `PaginationMeta`
- Filtres spécifiques : `BuildingFilters`, `ExpenseFilters`, etc.
- Query parameters : `?page=1&per_page=20&sort_by=created_at&order=desc`
- Migration tous repositories vers `find_all_paginated()`

**Valeur métier** : Performance scalable (100+ copropriétés).

---

### [#005 - Renforcement Sécurité](./critical/005-security-hardening.md)
**Estimation** : 10-12h | **Labels** : `security`, `backend`, `production-ready`

Sécurisation avant production.

**Livrables** :
- Rate limiting (5 req/15min login, 100 req/min API)
- JWT refresh tokens (expiry 15min + refresh 7j)
- CORS restreint (variable `ALLOWED_ORIGINS`)
- Structured logging (migration `tracing`)
- Audit logs pour actions critiques
- Validation JWT_SECRET (pas de fallback)
- 2FA TOTP (optionnel)

**Valeur métier** : Conformité sécurité, prêt production.

---

## 🟡 Issues Importantes (Phase 2)

Fonctionnalités **fortement recommandées** pour compétitivité marché.

### [#006 - Système de Paiement en Ligne](./important/006-online-payments.md)
**Estimation** : 15-20h | **Labels** : `backend`, `frontend`, `payments`

Intégration Stripe + SEPA Direct Debit.

**Fonctionnalités** :
- Paiement CB via Stripe Elements
- Prélèvement automatique SEPA
- Webhook `payment.succeeded`
- Génération reçus PDF automatiques
- Dashboard encaissements syndic

**ROI** : Réduction impayés, automatisation encaissements.

---

### [#007 - Gestion des Travaux](./important/007-work-management.md)
**Estimation** : 12-15h | **Labels** : `backend`, `frontend`, `feature`

Module complet gestion travaux copropriété.

**Fonctionnalités** :
- Entités `Work` + `Quote` (devis)
- Workflow : Proposition → Vote AG → En cours → Terminé
- Galerie photos avant/après
- Appels de fonds exceptionnels
- Garanties décennales

**ROI** : Simplification gestion gros chantiers.

---

### [#008 - Système de Ticketing](./important/008-ticketing-system.md)
**Estimation** : 8-10h | **Labels** : `backend`, `frontend`, `support`

Tickets de maintenance et interventions.

**Fonctionnalités** :
- Déclaration incidents par copropriétaires
- Affectation prestataires
- Statuts : Open → InProgress → Resolved
- Upload photos problème
- Historique interventions

**ROI** : Amélioration satisfaction résidents.

---

### [#009 - Notifications Multi-Canal](./important/009-notifications-system.md)
**Estimation** : 8-10h | **Labels** : `backend`, `frontend`, `notifications`

Notifications email + push web + in-app.

**Fonctionnalités** :
- Email SMTP/SendGrid
- Push web (Service Worker)
- Cloche notifications in-app
- Préférences utilisateur
- Queue asynchrone (Redis)

**ROI** : Engagement utilisateurs, réduction no-shows AG.

---

### [#010 - Progressive Web App](./important/010-progressive-web-app.md)
**Estimation** : 10-12h | **Labels** : `frontend`, `pwa`, `offline`

Transformer en PWA installable.

**Fonctionnalités** :
- `manifest.json` + Service Worker
- Mode hors-ligne avec IndexedDB
- Background sync
- Installation mobile/desktop
- Push notifications natives

**ROI** : Expérience mobile premium sans app native.

---

## 🟢 Issues Nice-to-Have (Long Terme)

Fonctionnalités **innovantes** pour différenciation marché.

### [#011 - Intelligence Artificielle](./nice-to-have/011-ai-features.md)
**Estimation** : 20-30h | **Labels** : `ai`, `ml`, `innovation`

5 fonctionnalités IA :
- OCR factures (extraction auto montant/fournisseur)
- Prédiction charges futures (ML)
- Détection anomalies dépenses
- Chatbot assistant copropriétaires
- Classification auto documents

**Tech** : Python microservice (FastAPI) + Azure Computer Vision ou Tesseract.

---

### [#012 - Marketplace Prestataires](./nice-to-have/012-marketplace.md)
**Estimation** : 20-25h | **Labels** : `marketplace`, `backend`, `frontend`

Annuaire prestataires vérifiés + notation + demandes de devis.

**Business Model** : Commission sur contrats signés.

---

### [#013 - Écologie et Durabilité](./nice-to-have/013-sustainability.md)
**Estimation** : 12-15h | **Labels** : `sustainability`, `green`

Aligné avec objectif **< 0.5g CO2/req**.

**Fonctionnalités** :
- Bilan carbone immeuble
- Suivi consommations énergétiques
- Tracking DPE (évolution avant/après travaux)
- Calculateur aides MaPrimeRénov'
- Recommandations IA travaux isolation

**Marketing** : Certification "Green SaaS".

---

### [#014 - Analytics & Business Intelligence](./nice-to-have/014-analytics-bi.md)
**Estimation** : 12-15h | **Labels** : `analytics`, `bi`

Tableaux de bord multi-copropriétés pour cabinets syndics.

**Fonctionnalités** :
- KPIs temps réel (taux recouvrement, charges/m²)
- Benchmarking vs marché
- Prédictions financières ML
- Rapports clients auto-générés PDF

**Business** : Premium feature payante.

---

### [#015 - Application Mobile Native](./nice-to-have/015-mobile-app.md)
**Estimation** : 30-40h | **Labels** : `mobile`, `react-native`

App iOS/Android avec fonctionnalités natives.

**Fonctionnalités** :
- Auth biométrique (Face ID, Touch ID)
- Scanner QR codes factures
- Photos haute résolution
- Push notifications natives
- Partage localisation technicien

**Tech** : React Native (code partagé 90%).

---

## 📅 Roadmap Suggérée

### Phase 1 : MVP Production-Ready (4-6 semaines)
**Objectif** : Version 1.0 déployable en production

```
Semaine 1-2 : Issues Critiques #001-#003
  ├─ Assemblées Générales API
  ├─ Documents upload/download
  └─ Rapports financiers

Semaine 3 : Issues Critiques #004-#005
  ├─ Pagination/Filtres
  └─ Sécurité (rate limiting, JWT refresh, CORS)

Semaine 4 : Tests & Déploiement
  ├─ Tests E2E complets
  ├─ Load testing
  ├─ Setup CI/CD Kubernetes
  └─ Documentation utilisateur
```

**Livrables** :
- API complète fonctionnelle
- Frontend intégré
- Tests coverage > 80%
- Déploiement production

---

### Phase 2 : Différenciation Marché (6-8 semaines)
**Objectif** : Fonctionnalités compétitives

```
Semaine 5-7 : Paiements & Travaux
  ├─ #006 Stripe integration
  └─ #007 Gestion travaux

Semaine 8-9 : Communication
  ├─ #008 Ticketing
  └─ #009 Notifications

Semaine 10-12 : PWA
  └─ #010 Mode offline complet
```

**Livrables** :
- Paiements en ligne opérationnels
- Module travaux testé en AG
- PWA installable

---

### Phase 3 : Innovation & Scalabilité (3-6 mois)
**Objectif** : Leadership marché

```
Mois 4-5 : IA & Analytics
  ├─ #011 OCR + Prédictions
  └─ #014 BI Dashboard

Mois 5-6 : Marketplace & Écologie
  ├─ #012 Prestataires
  └─ #013 Bilan carbone

Mois 6+ : Mobile Native
  └─ #015 React Native app
```

**Livrables** :
- Différenciation IA
- Certification Green SaaS
- Apps stores iOS/Android

---

## 💰 Estimations Budgétaires

### Développement (tarif indépendant 500€/j)

| Phase | Heures | Jours | Coût HT |
|-------|--------|-------|---------|
| Phase 1 (MVP) | 37-46h | 5-6j | 2 500 - 3 000€ |
| Phase 2 (Différenciation) | 58-72h | 7-9j | 3 500 - 4 500€ |
| Phase 3 (Innovation) | 94-125h | 12-16j | 6 000 - 8 000€ |
| **TOTAL** | **189-243h** | **24-31j** | **12 000 - 15 500€** |

### Infrastructure (coûts mensuels)

| Service | Coût/mois |
|---------|-----------|
| PostgreSQL RDS (production) | 50€ |
| Redis (sessions/cache) | 30€ |
| Kubernetes cluster | 100€ |
| Stripe fees (2.9% + 0.25€) | Variable |
| SendGrid (emails) | 15€ |
| Azure Computer Vision (OCR) | 50€ |
| **TOTAL** | **~245€/mois** |

### One-time costs
- Apple Developer Account : 99$/an
- Google Play Developer : 25$ one-time
- SSL certificats : Gratuit (Let's Encrypt)

---

## 🎯 Recommandations Stratégiques

### Démarrage Immédiat
1. **Issue #001** (Assemblées Générales) - Obligation légale
2. **Issue #005** (Sécurité) - Bloque production
3. **Issue #004** (Pagination) - Performance essentielle

### Quick Wins
- **Issue #009** (Notifications) - Impact utilisateur élevé, implémentation simple
- **Issue #008** (Ticketing) - Amélioration satisfaction immédiate

### Différenciation Concurrentielle
- **Issue #011** (IA/OCR) - Innovation forte
- **Issue #013** (Écologie) - Argument marketing unique
- **Issue #006** (Paiements) - Réduction impayés = ROI immédiat

---

## 📈 Métriques de Succès

### Phase 1 (MVP)
- [ ] 100% endpoints API fonctionnels
- [ ] P99 latency < 5ms
- [ ] Test coverage > 80%
- [ ] 0 vulnérabilités critiques (cargo audit)
- [ ] Lighthouse score > 90

### Phase 2 (Différenciation)
- [ ] Taux adoption paiements en ligne > 30%
- [ ] Temps résolution tickets < 48h
- [ ] PWA installs > 20% utilisateurs
- [ ] NPS (Net Promoter Score) > 50

### Phase 3 (Innovation)
- [ ] OCR accuracy > 90%
- [ ] Réduction CO2/req < 0.5g
- [ ] Mobile app rating stores > 4.5/5
- [ ] Churn rate < 5%/mois

---

## 🛠️ Comment Utiliser ces Issues

### 1. Copier sur GitHub
Chaque fichier `.md` peut être copié-collé directement dans une GitHub issue.

**Template issue** :
```
Titre : [Copier depuis H1 du fichier]
Labels : [Copier depuis Labels du fichier]
Description : [Copier tout le contenu markdown]
```

### 2. Priorisation
- **Critiques** : À faire en premier (MVP)
- **Importantes** : Phase 2 (après production)
- **Nice-to-Have** : Backlog long terme

### 3. Estimation
Toutes les estimations sont en **heures développeur senior**.
Multiplier par 1.5-2x pour développeur junior.

### 4. Dépendances
Vérifier section "Dépendances" de chaque issue avant de commencer.

Exemple : Issue #007 (Travaux) dépend de #001 (Meetings) et #002 (Documents).

---

## 📞 Contact & Support

Pour questions sur ces issues :
- Ouvrir discussion GitHub
- Consulter `CLAUDE.md` pour architecture
- Référencer `CHANGELOG.md` pour historique

---

## 📜 Licence

Ces cahiers des charges sont propriété du projet KoproGo.
Utilisation interne uniquement.

---

**Généré le** : 2025-10-23
**Version** : 1.0
**Auteur** : Claude Code Analysis

---

## 🔗 Navigation Rapide

### Par Priorité
- [Critiques (1-5)](./critical/)
- [Importantes (6-10)](./important/)
- [Nice-to-Have (11-15)](./nice-to-have/)

### Par Thématique
- **Backend** : #001, #002, #003, #004, #005, #006, #007, #008
- **Frontend** : #002, #003, #006, #007, #008, #010, #015
- **Sécurité** : #005
- **Paiements** : #006
- **IA/ML** : #011, #014
- **Mobile** : #010, #015
- **Écologie** : #013

---

**Prochaine étape** : Choisir une issue et démarrer l'implémentation ! 🚀
