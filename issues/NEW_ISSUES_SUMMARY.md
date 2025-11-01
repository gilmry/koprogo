# Nouvelles Issues Créées - Gap Analysis KoproGo

**Date de création**: 2025-11-01
**Basé sur**: GAP_ANALYSIS_KoproGov.md
**Issues créées**: 6
**Effort total estimé**: 52-64 heures (6.5-8 jours)

---

## 📊 Vue d'ensemble

Suite à l'analyse des gaps entre KoproGo et les fonctionnalités requises pour une plateforme de gestion de copropriété conforme à la législation belge, **6 nouvelles issues** ont été créées pour combler les lacunes critiques et importantes identifiées.

### Répartition par Priorité

| Priorité | Nombre | Effort Total | Emplacement |
|----------|--------|--------------|-------------|
| 🔴 **CRITIQUE** | 3 | 22-28 heures | `issues/critical/` |
| 🟡 **HIGH** | 3 | 30-36 heures | `issues/important/` |
| **TOTAL** | **6** | **52-64 heures** | - |

---

## 🔴 Issues Critiques (Conformité Légale Belgique)

Ces issues sont **bloquantes pour la mise en production** car elles concernent des obligations légales belges pour les copropriétés.

### Issue #016 - Plan Comptable Normalisé Belge

**Fichier**: `issues/critical/016-plan-comptable-belge.md`
**Estimation**: 8-10 heures
**Labels**: `enhancement`, `backend`, `critical`, `finance`, `legal-compliance`

**Description**:
Implémenter un plan comptable conforme à l'arrêté royal du 12 juillet 2012 (plan comptable normalisé belge pour copropriétés).

**Problème actuel**:
- Catégories d'expenses basiques (`maintenance`, `repairs`, `insurance`)
- Pas de structure comptable classe 4, 5, 6, 7
- Impossible de générer états financiers conformes (bilan, compte de résultat)

**Livrables**:
- Enum `AccountCode` avec 24+ codes comptables belges
- Migration SQL pour ajouter `account_code` aux expenses
- `FinancialReportingUseCases` (génération bilan + compte résultat)
- Vue SQL `financial_statements` pour faciliter requêtes
- Mapping ancien/nouveau système
- Frontend: affichage codes comptables

**Dépendances**:
- Bloque: Issue #003 (Rapports Financiers), Production deployment

**Impact Business**: CRITIQUE - Sans plan comptable conforme, comptes présentés en AG contestables légalement.

---

### Issue #017 - Génération État Daté (Mutations Immobilières)

**Fichier**: `issues/critical/017-etat-date-generation.md`
**Estimation**: 6-8 heures
**Labels**: `enhancement`, `backend`, `frontend`, `critical`, `legal-compliance`, `pdf-generation`

**Description**:
Générer automatiquement des **états datés** conformes pour mutations immobilières (ventes, donations, successions). L'état daté est un document obligatoire fourni par le syndic au notaire lors de toute transaction immobilière.

**Contexte légal**:
- Article 577-2 Code Civil belge
- Délai légal: 15 jours maximum après demande notaire
- Contenu obligatoire: situation financière du lot, décisions AG, garanties, etc.

**Problème actuel**:
- Aucune fonctionnalité de génération d'états datés
- **Bloque toutes les ventes de lots** en copropriété

**Livrables**:
- Entity `EtatDate` avec toutes les sections légales
- Table `etat_dates` avec tracking demandes
- `EtatDateUseCases` (calcul situation financière, extraction décisions AG)
- Génération PDF conforme format légal belge
- Endpoints API: request, generate, download, send to notary
- Frontend: interface demande état daté

**Dépendances**:
- Recommandées: Issue #016 (calcul financier précis), Issue #047 (PDF templates)

**Impact Business**: CRITIQUE - Sans état daté, mutations immobilières impossibles.

---

### Issue #018 - Budget Prévisionnel Annuel

**Fichier**: `issues/critical/018-budget-previsionnel.md`
**Estimation**: 8-10 heures
**Labels**: `enhancement`, `backend`, `frontend`, `critical`, `finance`, `legal-compliance`

**Description**:
Implémenter un système complet de budget prévisionnel annuel, base de toute gestion financière de copropriété. Le budget doit être voté en AG et sert au calcul des provisions/appels de fonds.

**Contexte légal**:
- Obligation légale belge: budget prévisionnel présenté chaque année en AG
- Budget = base calcul provisions trimestrielles/mensuelles

**Problème actuel**:
- Expense tracking existe, mais pas de budget planifié annuel
- Impossible de comparer budget vs réel (variance analysis)
- Pas de calcul automatique provisions

**Livrables**:
- Entity `Budget` (statuts: Draft, Proposed, Approved, Active, Closed)
- `BudgetSection` (ordinaire/extraordinaire) + `BudgetLineItem`
- Table `budgets` avec contrainte unicité (building_id, fiscal_year)
- `BudgetUseCases` (CRUD, variance analysis, copy from previous year)
- Calcul automatique provisions (trimestrielles, mensuelles)
- Mise à jour automatique dépenses réelles vs budget
- Frontend: création/édition budgets, dashboard budget vs réel

**Dépendances**:
- Recommandées: Issue #016 (account_code), Issue #046 (vote AG budget)

**Impact Business**: CRITIQUE - Base de toute gestion financière, impossible de gérer appels de fonds sans ça.

---

## 🟡 Issues Importantes (Automation & Compliance)

Ces issues améliorent significativement l'expérience utilisateur et la conformité, mais ne bloquent pas immédiatement la production.

### Issue #019 - Convocations AG Automatiques

**Fichier**: `issues/important/019-convocations-ag-automatiques.md`
**Estimation**: 5-7 heures
**Labels**: `enhancement`, `backend`, `frontend`, `notifications`, `legal-compliance`

**Description**:
Automatiser la génération et l'envoi des convocations aux assemblées générales (PDF + email).

**Contexte légal**:
- Délai minimum: 15 jours avant AG ordinaire, 8 jours avant AG extraordinaire
- Contenu obligatoire: ordre du jour complet, date/heure/lieu, formulaire procuration

**Problème actuel**:
- Meeting API existe, mais convocations manuelles
- Pas de vérification délais légaux
- Pas de génération PDF automatique

**Livrables**:
- `ConvocationUseCases` (génération données, envoi emails, tracking)
- Extension `Meeting` entity: `check_legal_delay()`, `prepare_convocation_data()`
- `SmtpEmailService` (envoi email avec PDF attaché)
- `ConvocationPdfGenerator` (template HTML → PDF)
- Endpoints: send_convocations, resend_to_owner
- Frontend: bouton "Envoyer convocations" avec prévisualisation

**Dépendances**:
- Recommandées: Issue #009 (Notifications System), Issue #047 (PDF templates)

**Impact Business**: HIGH - Fait gagner énormément de temps syndic, garantit conformité délais.

---

### Issue #020 - Carnet d'Entretien et Suivi Travaux

**Fichier**: `issues/important/020-carnet-entretien.md`
**Estimation**: 10-12 heures
**Labels**: `enhancement`, `backend`, `frontend`, `maintenance`, `legal-compliance`

**Description**:
Implémenter un carnet d'entretien numérique pour tracer tous travaux, interventions, maintenances et contrôles techniques obligatoires.

**Contexte légal**:
- Fortement recommandé (pas strictement obligatoire)
- Indispensable pour: garanties décennales, contrôles techniques, valorisation immeuble

**Problème actuel**:
- Aucune fonctionnalité de suivi travaux historique
- Pas de gestion garanties
- Pas d'alertes contrôles techniques

**Livrables**:
- Entity `WorkReport` (types: Maintenance, Repair, Installation, Inspection, Improvement)
- Entity `TechnicalInspection` (types: Elevator, Boiler, Fire, Electrical, Energy, Gas, Water)
- Tables `work_reports` + `technical_inspections`
- `MaintenanceUseCases` (CRUD reports, warranties, inspections, alerts)
- Gestion garanties (contractuelle 2 ans, biennale, décennale 10 ans)
- Alertes automatiques 30 jours avant contrôles obligatoires
- Upload photos avant/après interventions
- Export PDF carnet d'entretien complet
- Frontend: timeline chronologique travaux, dashboard garanties/inspections

**Dépendances**:
- Recommandées: Issue #002 (Document upload), Issue #047 (PDF), Issue #009 (Alertes)

**Impact Business**: HIGH - Réduit coûts maintenance, facilite audits, améliore gestion préventive.

---

### Issue #021 - GDPR Articles Complémentaires (16, 18, 21)

**Fichier**: `issues/important/021-gdpr-articles-complementaires.md`
**Estimation**: 5-7 heures
**Labels**: `enhancement`, `backend`, `frontend`, `gdpr`, `legal-compliance`, `privacy`

**Description**:
Compléter implémentation GDPR en ajoutant Articles 16 (Rectification), 18 (Limitation du traitement), 21 (Opposition).

**État actuel**:
- ✅ Article 15 (Right to Access) - Export données
- ✅ Article 17 (Right to Erasure) - Anonymisation
- ❌ Article 16 (Rectification) - Correction données
- ❌ Article 18 (Restriction) - Gel temporaire traitement
- ❌ Article 21 (Objection) - Opposition marketing/profilage

**Livrables**:

**Article 16 - Rectification**:
- `request_rectification(changes: Vec<FieldChange>)` use case
- Workflow validation + approval (admin pour champs sensibles comme email)
- Application automatique changements approuvés

**Article 18 - Restriction**:
- `request_restriction(reason, effective_from, effective_until)` use case
- Flag `processing_restricted = true` sur User
- Gel traitement marketing/analytique, conservation données
- Levée restriction sur demande

**Article 21 - Objection**:
- `request_objection(objection_type, processing_purposes)` use case
- Types: Marketing, Profiling, LegitimateInterest, Research
- Acceptation automatique objections marketing
- Flags `marketing_consent`, `profiling_consent`

**Frontend**:
- Extension `GdprDataPanel.svelte` avec 3 nouvelles sections
- Modals pour rectification/restriction
- Checkboxes consentements marketing/profilage

**Dépendances**:
- Bloquantes: Articles 15 & 17 déjà implémentés ✅
- Optionnelles: Issue #009 (Notifications confirmation)

**Impact Business**: HIGH - Conformité RGPD complète, évite sanctions.

---

## 📋 Roadmap Recommandée

### Phase 1 : Conformité Légale Critique (22-28 heures = 3-4 jours)

**Objectif**: Prêt pour production légale en Belgique

```
Semaine 1-2 : Issues Critiques Parallèles
  ├─ #016 Plan Comptable Belge (8-10h)
  ├─ #017 État Daté Génération (6-8h)
  └─ #018 Budget Prévisionnel (8-10h)
```

**Ordre suggéré**:
1. **#016 Plan Comptable** (en premier, car dépendance pour #017 et #018)
2. **#018 Budget Prévisionnel** (utilise #016 pour account_code)
3. **#017 État Daté** (utilise #016 pour situation financière précise)

**Livrables Phase 1**:
- Comptabilité conforme arrêté royal 12/07/2012
- États datés générables pour notaires
- Budgets annuels votables en AG
- Appels de fonds calculables automatiquement

---

### Phase 2 : Automation & Features Avancées (30-36 heures = 4-5 jours)

**Objectif**: Améliorer productivité syndic + conformité complète

```
Semaine 3-4 : Issues Importantes
  ├─ #019 Convocations AG Auto (5-7h)
  ├─ #020 Carnet Entretien (10-12h)
  └─ #021 GDPR Compléments (5-7h)
```

**Ordre suggéré**:
1. **#019 Convocations AG** (quick win, automation visible)
2. **#021 GDPR Compléments** (finaliser conformité)
3. **#020 Carnet Entretien** (feature complète, plus complexe)

**Livrables Phase 2**:
- Convocations AG automatiques avec PDF + emails
- RGPD 100% conforme (Articles 15-18, 21)
- Carnet entretien numérique complet
- Alertes contrôles techniques

---

## 🔗 Dépendances Inter-Issues

### Issues Existantes à Compléter d'abord

Avant de commencer les nouvelles issues, il est recommandé de compléter :

- **Issue #001** : Meeting Management API (bloque #019 Convocations)
- **Issue #002** : Document Upload (bloque #017 États datés, #020 Carnet)
- **Issue #047** : PDF Generation Extended (bloque #017, #019, #020)
- **Issue #042** : GDPR Data Export & Deletion (complété par #021)

### Chaîne de Dépendances Nouvelles Issues

```
#016 Plan Comptable
  └─▶ #018 Budget Prévisionnel (utilise account_code)
  └─▶ #017 État Daté (situation financière précise)

#001 Meeting API ─▶ #019 Convocations AG

#002 Document Upload ─▶ #017 État Daté
                      └─▶ #020 Carnet Entretien (photos travaux)

#047 PDF Generation ─▶ #017 État Daté
                      └─▶ #019 Convocations
                      └─▶ #020 Carnet (export PDF)

GDPR Articles 15+17 ─▶ #021 GDPR Compléments
```

---

## 📊 Métriques de Succès

### Conformité Légale (Phase 1)
- [ ] 100% codes comptables conformes arrêté royal 12/07/2012
- [ ] États datés générables en < 3 secondes
- [ ] Budgets annuels votables avec variance analysis
- [ ] 0% erreurs migration plan comptable

### Automation (Phase 2)
- [ ] Convocations envoyées en < 5 minutes pour 100 copropriétaires
- [ ] 100% contrôles techniques trackés avec alertes
- [ ] RGPD compliance score = 100% (Articles 15-18, 21)

### Performance
- [ ] Génération PDF < 3 secondes (états datés, convocations)
- [ ] Calcul budget variance < 500ms pour 100 lignes
- [ ] P99 latency < 5ms maintenue

---

## 🚀 Instructions de Démarrage

### Pour chaque issue

1. **Lire le fichier Markdown complet** dans `issues/critical/` ou `issues/important/`
2. **Vérifier les dépendances bloquantes** (issues à terminer d'abord)
3. **Suivre la checklist de développement** (étapes numérotées à la fin de chaque issue)
4. **Exécuter les tests** (tests unitaires + E2E fournis dans chaque issue)
5. **Mettre à jour CHANGELOG.md** avec le message de commit suggéré

### Ordre de Développement Optimal

```
1. #016 Plan Comptable Belge (8-10h)
   ├─ Impacte: #017, #018
   └─ Commit: "feat: implement Belgian accounting plan compliance"

2. #018 Budget Prévisionnel (8-10h)
   ├─ Dépend: #016
   └─ Commit: "feat: implement annual budgeting system"

3. #017 État Daté (6-8h)
   ├─ Dépend: #016, #047 (optionnel)
   └─ Commit: "feat: implement état daté generation for real estate transactions"

4. #019 Convocations AG (5-7h)
   ├─ Dépend: #001 (Meeting API), #047 (optionnel)
   └─ Commit: "feat: implement automatic AG convocations with email/PDF"

5. #021 GDPR Compléments (5-7h)
   ├─ Dépend: GDPR #042 existant
   └─ Commit: "feat: implement GDPR Articles 16, 18, 21"

6. #020 Carnet Entretien (10-12h)
   ├─ Dépend: #002 (Documents), #009 (Notifications optionnel)
   └─ Commit: "feat: implement digital maintenance logbook"
```

**Durée totale**: 52-64 heures = **6.5-8 jours** (développeur senior)

---

## 💰 Estimation Budgétaire

### Développement (tarif indépendant 500€/jour)

| Phase | Heures | Jours | Coût HT |
|-------|--------|-------|---------|
| Phase 1 (Conformité) | 22-28h | 3-4j | 1 500 - 2 000€ |
| Phase 2 (Automation) | 30-36h | 4-5j | 2 000 - 2 500€ |
| **TOTAL** | **52-64h** | **7-9j** | **3 500 - 4 500€** |

### Comparaison avec Roadmap Existante

Issues roadmap ROADMAP.rst :
- Phase 1 VPS MVP: 42-59 jours (issues #39-51)

Nouvelles issues :
- 7-9 jours supplémentaires (intégrable Phase 1)

**Total Phase 1 ajusté** : **49-68 jours** (10-14 semaines)

---

## 📞 Contact & Support

Pour questions sur ces issues :
- Référencer `GAP_ANALYSIS_KoproGov.md` (analyse complète)
- Consulter `ROADMAP.rst` (planification globale)
- Ouvrir discussion GitHub si besoin clarifications

---

## 📜 Résumé Fichiers Créés

```
issues/
├── critical/
│   ├── 016-plan-comptable-belge.md                  (8-10h)
│   ├── 017-etat-date-generation.md                  (6-8h)
│   └── 018-budget-previsionnel.md                   (8-10h)
├── important/
│   ├── 019-convocations-ag-automatiques.md          (5-7h)
│   ├── 020-carnet-entretien.md                      (10-12h)
│   └── 021-gdpr-articles-complementaires.md         (5-7h)
└── NEW_ISSUES_SUMMARY.md                             (ce fichier)
```

---

**Créé le** : 2025-11-01
**Auteur** : Claude Code Analysis
**Version** : 1.0
**Prochaine étape** : Commencer Issue #016 (Plan Comptable Belge) 🚀
