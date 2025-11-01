# KoproGo - Issues Roadmap

Ce dossier contient **25 issues détaillées** avec cahiers des charges complets pour l'évolution de la plateforme KoproGo vers une conformité légale belge complète et des fonctionnalités avancées.

**Dernière mise à jour** : 2025-11-01 (Gap Analysis complète)

---

## 📊 Vue d'Ensemble

| Catégorie | Nombre | Estimation Totale | Priorité |
|-----------|--------|-------------------|----------|
| **🔴 Critique** | 9 | 86-106 heures | À faire AVANT production |
| **🟡 Important** | 10 | 107-135 heures | Phase 2 automation/compliance |
| **🟢 Nice-to-Have** | 6 | 127-167 heures | Long terme différenciation |
| **TOTAL** | **25** | **320-408 heures** | ~40-51 jours |

**Note** : Issues #004 (Pagination) et #007 (Work Management) supprimées (déjà implémentées ou fusionnées).

---

## 🔴 Issues Critiques (Production-Ready Belgique)

Ces fonctionnalités sont **bloquantes pour la mise en production légale** en Belgique.

### [#001 - Gestion des Assemblées Générales](./critical/001-meeting-management-api.md)
**Estimation** : 6-8h | **Labels** : `backend`, `domain-logic`, `legal-compliance`

API complète pour assemblées générales (AG ordinaires/extraordinaires). Entités existent, manque use cases et endpoints.

**Livrables** : Use Cases, 10 endpoints REST, tests E2E + BDD Gherkin

---

### [#002 - Gestion Documentaire Complète](./critical/002-document-upload-download.md)
**Estimation** : 8-10h | **Labels** : `backend`, `frontend`, `file-storage`

Upload/download documents avec stockage sécurisé (PV, factures, règlements).

**Livrables** : Trait `FileStorageService`, upload multipart, composant Svelte

---

### [#003 - Génération de Rapports Financiers](./critical/003-financial-reports-generation.md)
**Estimation** : 10-12h | **Labels** : `backend`, `frontend`, `finance`

6 types de rapports : appels de fonds, budget, impayés, FEC comptable.

**Livrables** : `ReportUseCases`, génération PDF + Excel, dashboard frontend

---

### [#005 - Renforcement Sécurité](./critical/005-security-hardening.md)
**Estimation** : 10-12h | **Labels** : `security`, `backend`, `production-ready`

Rate limiting, JWT refresh tokens, CORS, structured logging, 2FA optionnel.

**Valeur** : Conformité sécurité production

---

### [#016 - Plan Comptable Normalisé Belge](./critical/016-plan-comptable-belge.md) 🆕
**Estimation** : 8-10h | **Labels** : `backend`, `critical`, `finance`, `legal-compliance`

**NOUVEAU** - Implémenter plan comptable conforme arrêté royal 12/07/2012 (classes 4, 5, 6, 7).

**Livrables** : Enum `AccountCode` (24+ codes), génération bilan + compte de résultat

**Bloque** : #017, #018, #003

---

### [#017 - État Daté Génération](./critical/017-etat-date-generation.md) 🆕
**Estimation** : 6-8h | **Labels** : `backend`, `frontend`, `critical`, `legal-compliance`

**NOUVEAU** - Génération états datés pour mutations immobilières (obligation Article 577-2 Code Civil).

**Impact** : BLOQUE toutes les ventes de lots sans ça

**Livrables** : Entity `EtatDate`, génération PDF, délai 15 jours

---

### [#018 - Budget Prévisionnel Annuel](./critical/018-budget-previsionnel.md) 🆕
**Estimation** : 8-10h | **Labels** : `backend`, `frontend`, `critical`, `finance`

**NOUVEAU** - Système budget annuel (ordinaire + extraordinaire) avec variance analysis.

**Livrables** : Entity `Budget`, calcul provisions automatiques, vote AG

---

### [#022 - Conseil de Copropriété](./critical/022-conseil-copropriete.md) 🆕
**Estimation** : 12-15h | **Labels** : `backend`, `frontend`, `critical`, `legal-compliance`, `governance`

**NOUVEAU** - **OBLIGATION LÉGALE** pour immeubles >20 lots (Article 577-8/4 Code Civil).

**0% implémenté actuellement** - Gap critique identifié.

**Livrables** :
- Rôle `BoardMember` avec permissions spéciales
- Élections et mandats annuels
- Dashboard suivi décisions AG
- Tracking délais (devis, travaux)
- Alertes retards syndic
- Rapports semestriels/annuels automatiques
- Vérification incompatibilité syndic ≠ conseil

**Bloque** : Production pour copropriétés >20 lots

---

### [#023 - Workflow Recouvrement Impayés](./critical/023-workflow-recouvrement.md) 🆕
**Estimation** : 6-8h | **Labels** : `backend`, `frontend`, `critical`, `finance`, `automation`

**NOUVEAU** - Workflow automatisé relances 3 niveaux (J+15, J+30, J+60 mise en demeure).

**Livrables** : Entity `PaymentReminder`, génération PDF lettres, cron job quotidien

**Impact** : Réduction impayés 30-50%

---

## 🟡 Issues Importantes (Automation & Features)

Ces fonctionnalités améliorent significativement l'expérience et la conformité.

### [#006 - Système de Paiement en Ligne](./important/006-online-payments.md)
**Estimation** : 15-20h | **Labels** : `backend`, `frontend`, `payments`

Intégration Stripe + SEPA Direct Debit pour paiements en ligne.

---

### [#008 - Système de Ticketing](./important/008-ticketing-system.md)
**Estimation** : 8-10h | **Labels** : `backend`, `frontend`, `support`

Tickets maintenance et interventions (déclaration incidents copropriétaires).

---

### [#009 - Notifications Multi-Canal](./important/009-notifications-system.md)
**Estimation** : 8-10h | **Labels** : `backend`, `frontend`, `notifications`

Notifications email + push web + in-app avec queue asynchrone.

---

### [#010 - Progressive Web App](./important/010-progressive-web-app.md)
**Estimation** : 10-12h | **Labels** : `frontend`, `pwa`, `offline`

Transformer en PWA installable avec mode hors-ligne (IndexedDB).

---

### [#019 - Convocations AG Automatiques](./important/019-convocations-ag-automatiques.md) 🆕
**Estimation** : 5-7h | **Labels** : `backend`, `frontend`, `notifications`, `legal-compliance`

**NOUVEAU** - Génération automatique convocations AG avec PDF + email, vérification délais légaux (15j/8j).

---

### [#020 - Carnet d'Entretien](./important/020-carnet-entretien.md) 🆕
**Estimation** : 10-12h | **Labels** : `backend`, `frontend`, `maintenance`, `legal-compliance`

**NOUVEAU** - Carnet d'entretien numérique avec suivi travaux, garanties (décennales), alertes contrôles techniques.

---

### [#021 - GDPR Articles Complémentaires](./important/021-gdpr-articles-complementaires.md) 🆕
**Estimation** : 5-7h | **Labels** : `backend`, `frontend`, `gdpr`, `legal-compliance`

**NOUVEAU** - Compléter GDPR avec Articles 16 (Rectification), 18 (Restriction), 21 (Opposition).

**État actuel** : Articles 15 & 17 ✅ | Manque : 16, 18, 21

---

### [#024 - Module Devis Travaux](./important/024-module-devis-travaux.md) 🆕
**Estimation** : 8-10h | **Labels** : `backend`, `frontend`, `finance`, `procurement`

**NOUVEAU** - Gestion devis multi-prestataires avec tableau comparatif automatique (obligation 3 devis pour travaux >5000€).

---

### [#025 - Affichage Public Syndic](./important/025-affichage-public-syndic.md) 🆕
**Estimation** : 3-4h | **Labels** : `frontend`, `legal-compliance`

**NOUVEAU** - Page publique (non authentifiée) affichage coordonnées syndic (obligation légale).

---

### [#027 - Accessibilité WCAG 2.1 AA](./important/027-accessibilite-wcag.md) 🆕
**Estimation** : 8-10h | **Labels** : `frontend`, `accessibility`, `a11y`, `compliance`

**NOUVEAU** - Conformité WCAG 2.1 niveau AA (contraste, clavier, ARIA, lecteurs d'écran).

---

## 🟢 Issues Nice-to-Have (Innovation & Différenciation)

Fonctionnalités **innovantes** pour différenciation marché et mission sociale ASBL.

### [#011 - Intelligence Artificielle](./nice-to-have/011-ai-features.md)
**Estimation** : 20-30h | **Labels** : `ai`, `ml`, `innovation`

5 fonctionnalités IA : OCR factures, prédiction charges, détection anomalies, chatbot, classification documents.

---

### [#012 - Marketplace Prestataires](./nice-to-have/012-marketplace.md)
**Estimation** : 20-25h | **Labels** : `marketplace`, `backend`, `frontend`

Annuaire prestataires vérifiés + notation + demandes de devis.

---

### [#013 - Écologie et Durabilité](./nice-to-have/013-sustainability.md)
**Estimation** : 12-15h | **Labels** : `sustainability`, `green`

Bilan carbone immeuble, suivi consommations énergétiques, DPE, aides rénovation.

---

### [#014 - Analytics & Business Intelligence](./nice-to-have/014-analytics-bi.md)
**Estimation** : 12-15h | **Labels** : `analytics`, `bi`

Tableaux de bord multi-copropriétés pour cabinets syndics (KPIs, benchmarking, ML).

---

### [#015 - Application Mobile Native](./nice-to-have/015-mobile-app.md)
**Estimation** : 30-40h | **Labels** : `mobile`, `react-native`

App iOS/Android avec auth biométrique, scanner QR, push notifications natives.

---

### [#026 - Modules Communautaires](./nice-to-have/026-modules-communautaires.md) 🆕
**Estimation** : 15-20h | **Labels** : `backend`, `frontend`, `community`, `social-impact`

**NOUVEAU** - 5 modules pour mission sociale ASBL :
1. **SEL** (Système d'Échange Local) - Troc compétences
2. **Bazar de Troc** - Échange/don objets
3. **Prêt d'Objets** - Bibliothèque outils
4. **Annuaire Compétences** - Listing habitants
5. **Tableau Affichage** - Petites annonces

**Livrables** : 5 entities, tracking impact social, gamification (badges), rapport annuel

**Impact** : Différenciateur fort vs concurrents classiques

---

## 📋 Roadmap Suggérée

### Phase 1 : MVP Production-Ready Belgique (86-106 heures = 11-14 jours)

**Objectif** : Conformité légale complète pour production en Belgique

```
Semaine 1-2 : Conformité Comptable & Financière
  ├─ #016 Plan Comptable Belge (8-10h) ← PRIORITÉ 1
  ├─ #018 Budget Prévisionnel (8-10h)
  └─ #017 État Daté (6-8h)

Semaine 2-3 : Governance & Recouvrement
  ├─ #022 Conseil de Copropriété (12-15h) ← BLOQUANT >20 lots
  └─ #023 Workflow Recouvrement (6-8h)

Semaine 3-4 : Finalisation MVP
  ├─ #001 Meeting Management API (6-8h)
  ├─ #002 Document Upload (8-10h)
  ├─ #003 Financial Reports (10-12h)
  └─ #005 Security Hardening (10-12h)
```

**Livrables Phase 1** :
- ✅ Comptabilité conforme AR 12/07/2012
- ✅ États datés générables (mutations)
- ✅ Budgets annuels votables
- ✅ Conseil de copropriété opérationnel
- ✅ Recouvrement automatisé
- ✅ Sécurité production-ready

---

### Phase 2 : Automation & Compliance (107-135 heures = 13-17 jours)

**Objectif** : Amélioration productivité + conformité GDPR complète

```
Semaine 5-6 : Automation Juridique
  ├─ #019 Convocations AG Auto (5-7h)
  ├─ #024 Module Devis (8-10h)
  ├─ #025 Affichage Public (3-4h)
  └─ #021 GDPR Compléments (5-7h)

Semaine 7-8 : Features Métier
  ├─ #020 Carnet Entretien (10-12h)
  ├─ #008 Ticketing (8-10h)
  ├─ #009 Notifications (8-10h)
  └─ #027 Accessibilité WCAG (8-10h)

Semaine 9-11 : Advanced Features
  ├─ #010 PWA (10-12h)
  └─ #006 Paiements en Ligne (15-20h)
```

**Livrables Phase 2** :
- ✅ Convocations automatiques
- ✅ GDPR 100% conforme (Articles 15-18, 21)
- ✅ Carnet entretien numérique
- ✅ Accessibilité WCAG AA
- ✅ PWA installable
- ✅ Paiements en ligne Stripe + SEPA

---

### Phase 3 : Innovation & Différenciation (127-167 heures = 16-21 jours)

**Objectif** : Leadership marché + mission sociale

```
Mois 4-5 : Community & Sustainability
  ├─ #026 Modules Communautaires (15-20h)
  ├─ #013 Écologie/Durabilité (12-15h)
  └─ #014 Analytics BI (12-15h)

Mois 5-6 : IA & Marketplace
  ├─ #011 AI Features (20-30h)
  └─ #012 Marketplace (20-25h)

Mois 6+ : Mobile
  └─ #015 Mobile App (30-40h)
```

**Livrables Phase 3** :
- ✅ SEL, Troc, Partage opérationnels
- ✅ Bilan carbone, DPE tracking
- ✅ IA OCR factures + prédictions
- ✅ Mobile app iOS/Android

---

## 🔗 Dépendances Critiques

### Chaînes de Dépendances Principales

```
#016 Plan Comptable Belge
  ├─▶ #018 Budget Prévisionnel (utilise account_code)
  ├─▶ #017 État Daté (situation financière)
  └─▶ #003 Financial Reports (bilan, compte résultat)

#001 Meeting Management API
  ├─▶ #019 Convocations AG
  └─▶ #022 Conseil Copropriété (vote élections)

#002 Document Upload
  ├─▶ #017 État Daté (PDF génération)
  ├─▶ #020 Carnet Entretien (photos travaux)
  └─▶ #024 Module Devis (upload devis PDF)

GDPR #042 (Articles 15 & 17 existants)
  └─▶ #021 GDPR Compléments (Articles 16, 18, 21)

#009 Notifications
  ├─▶ #019 Convocations (emails)
  ├─▶ #020 Carnet Entretien (alertes)
  └─▶ #023 Recouvrement (relances)
```

---

## 💰 Estimation Budgétaire

### Développement (tarif senior 500€/jour)

| Phase | Heures | Jours | Coût HT |
|-------|--------|-------|---------|
| Phase 1 (MVP Belgique) | 86-106h | 11-14j | 5 500 - 7 000€ |
| Phase 2 (Automation) | 107-135h | 13-17j | 6 500 - 8 500€ |
| Phase 3 (Innovation) | 127-167h | 16-21j | 8 000 - 10 500€ |
| **TOTAL** | **320-408h** | **40-52j** | **20 000 - 26 000€** |

**Note** : Estimation développeur senior Rust/Svelte. Multiplier par 1.5-2x pour junior.

---

## 📚 Documents Complémentaires

- **[GAP_ANALYSIS_KoproGov.md](../docs/GAP_ANALYSIS_KoproGov.md)** : Analyse complète gaps projet vs fonctionnalités requises (93 fonctionnalités, 29% complétion)
- **[NEW_ISSUES_SUMMARY.md](./NEW_ISSUES_SUMMARY.md)** : Résumé des 12 nouvelles issues créées depuis gap analysis
- **[ROADMAP.rst](../docs/ROADMAP.rst)** : Roadmap infrastructure Phase 1-3 (VPS → K3s → K8s)

---

## 🎯 Métriques de Succès

### Phase 1 (MVP)
- [ ] 100% conformité légale Belgique (plan comptable, états datés, conseil copropriété)
- [ ] 0 vulnérabilités critiques (cargo audit)
- [ ] P99 latency < 5ms
- [ ] Test coverage > 80%

### Phase 2 (Automation)
- [ ] Taux adoption paiements en ligne > 30%
- [ ] Réduction impayés > 30%
- [ ] GDPR compliance score = 100%
- [ ] NPS (Net Promoter Score) > 50

### Phase 3 (Innovation)
- [ ] Taux participation SEL > 20% habitants
- [ ] OCR accuracy > 90%
- [ ] CO2/req < 0.5g
- [ ] Mobile app rating > 4.5/5

---

## 📞 Contact & Support

Pour questions sur ces issues :
- Référencer **GAP_ANALYSIS_KoproGov.md** pour contexte
- Consulter **ROADMAP.rst** pour planification globale
- Ouvrir discussion GitHub

---

**Dernière mise à jour** : 2025-11-01
**Version** : 2.0 (Post Gap Analysis)
**Auteur** : KoproGo Development Team
**Prochaine étape** : Commencer #016 (Plan Comptable Belge) 🚀
