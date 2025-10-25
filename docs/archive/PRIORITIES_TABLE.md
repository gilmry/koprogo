# KoproGo - Tableau Unique de Priorisation

**Date**: 2025-10-23
**Total Issues**: 24 (15 existantes + 9 nouvelles)
**Timeline**: 24 mois (4 phases)

---

## 📋 Tableau Consolidé (Toutes Issues)

| Issue | Nom | Bloquant | Concurrence | Étude | Hosted | Effort | Priorité | Phase |
|-------|-----|----------|-------------|-------|--------|--------|----------|-------|
| **#001** | Meeting Management API | ✅ | ✅ | ✅ | - | 6-8h | 🔴 Critique | 1 |
| **#002** | Document Upload/Download | ✅ | ✅ | ✅ | - | 8-10h | 🔴 Critique | 1 |
| **#003** | Financial Reports | ✅ | ✅ | ✅ | - | 10-12h | 🔴 Critique | 1 |
| **#004** | Pagination & Filtering | ✅ | ✅ | ❌ | - | 3-4h | 🔴 Critique | 1 |
| **#005** | Security Hardening | ✅ | ✅ | ✅ | ✅ | 10-12h | 🔴 Critique | 1 |
| **#016** | PCN Belge Compliance | ✅ | ✅ | ✅ | - | 12-15h | 🔴 Critique | 1 |
| **#019** | i18n FR/NL/EN | ✅ | ✅ | ✅ | - | 8-10h | 🔴 Critique | 1 |
| **#020** | Multi-tenancy Parfait | ✅ | - | ❌ | ✅ | 10-12h | 🔴 Critique | 1 |
| **#006** | Online Payments | ❌ | ✅ | ✅ | - | 15-20h | 🟡 Important | 2 |
| **#007** | Work Management | ❌ | ✅ | ✅ | - | 12-15h | 🟡 Important | 2 |
| **#008** | Ticketing System | ❌ | ✅ | ⚠️ | - | 8-10h | 🟡 Important | 2 |
| **#009** | Notifications | ❌ | ✅ | ✅ | - | 8-10h | 🟡 Important | 2 |
| **#010** | PWA Offline | ❌ | ✅ | ⚠️ | - | 10-12h | 🟡 Important | 2 |
| **#017** | CODA Import Bancaire | ✅ | ✅ | ✅ | - | 15-20h | 🔴 Critique | 2 |
| **#018** | Exact Online Export | ❌ | ✅ | ✅ | - | 10-12h | 🟡 Important | 2 |
| **#021** | Stripe Billing 1€ | ❌ | - | ❌ | ✅ | 6-8h | 🟡 Important | 2 |
| **#022** | Belgian Council >20 lots | ❌ | ✅ | ✅ | - | 6-8h | 🟡 Important | 2 |
| **#023** | Country Regulations Engine | ❌ | ❌ | ✅ | - | 12-15h | 🟡 Important | 2-3 |
| **#011** | AI Features (OCR, ML) | ❌ | ❌ | ❌ | - | 20-30h | 🟢 Nice | 3 |
| **#013** | Sustainability (PEB, CO2) | ❌ | ❌ | ⚠️ | - | 12-15h | 🟡 Important | 3 |
| **#014** | Analytics & BI | ❌ | ✅ | ❌ | - | 12-15h | 🟡 Important | 3 |
| **#024** | Multi-currency EUR/TND | ❌ | ❌ | ✅ | - | 6-8h | 🟡 Important | 3 |
| **#012** | Marketplace Prestataires | ❌ | ❌ | ❌ | - | 20-25h | 🟢 Nice | 4 |
| **#015** | Mobile Native App | ❌ | ✅ | ✅ | - | 30-40h | ⚫ Backlog | 4 |
| **#025** | TLIS Integration Tunisie | ❌ | ❌ | ⚠️ | - | 15-20h | 🟢 Nice | 4 |

---

## 📊 Statistiques Globales

### Par Priorité
- 🔴 **Critiques**: 8 issues (33%) - 67-83h
- 🟡 **Importantes**: 12 issues (50%) - 145-203h
- 🟢 **Nice**: 3 issues (12.5%) - 47-70h
- ⚫ **Backlog**: 1 issue (4.5%) - 30-40h

**Total Effort**: 289-396 heures (36-50 semaines à 20h/sem)

### Par Phase
- **Phase 1** (Mois 1-6): 8 issues - 67-83h
- **Phase 2** (Mois 7-12): 9 issues - 102-135h
- **Phase 3** (Mois 13-20): 4 issues - 50-68h
- **Phase 4** (Mois 20+): 3 issues - 65-85h

### Par Origine
- **Existantes** (#001-#015): 15 issues (62.5%)
- **Nouvelles** (#016-#025): 9 issues (37.5%)

---

## 🔴 Phase 1 - Détail (Mois 1-6)

**Objectif**: MVP self-hosted production-ready avec conformité belge

| Issue | Nom | Effort | Cumul | Bloque | Justification |
|-------|-----|--------|-------|--------|---------------|
| #004 | Pagination | 3-4h | 3-4h | Scale | Performance >100 copros |
| #001 | Meetings | 6-8h | 9-12h | Legal | Obligation légale syndic |
| #002 | Documents | 8-10h | 17-22h | Legal | Archivage PV obligatoire |
| #019 | i18n FR/NL | 8-10h | 25-32h | Belgian | Bilinguisme BE obligatoire |
| #003 | Reports | 10-12h | 35-44h | Legal | Transparence financière AG |
| #005 | Security | 10-12h | 45-56h | Prod | Sécurité = production-blocker |
| #020 | Multi-tenancy | 10-12h | 55-68h | Hosted | Fondation SaaS |
| #016 | PCN Belge | 12-15h | 67-83h | Belgian | Comptabilité normalisée BE |

**Total Phase 1**: 67-83h (8-10 semaines)
**Livrables**:
- ✅ Version 1.0 open-source
- ✅ Docker Compose ready
- ✅ Tests >80% coverage
- ✅ Documentation FR/NL
- ✅ 50 self-hosted instances
- ✅ 10 beta clients (100€ MRR)

---

## 🟡 Phase 2 - Détail (Mois 7-12)

**Objectif**: Hosted beta Belgique avec parité concurrence

| Issue | Nom | Effort | Cumul | Catégorie | Justification |
|-------|-----|--------|-------|-----------|---------------|
| #021 | Stripe Billing | 6-8h | 6-8h | Hosted | Monétisation 1€/mois |
| #022 | Belgian Council | 6-8h | 12-16h | Belgian | >20 lots règle BE |
| #008 | Ticketing | 8-10h | 20-26h | UX | Satisfaction résidents |
| #009 | Notifications | 8-10h | 28-36h | UX | Engagement users |
| #010 | PWA Offline | 10-12h | 38-48h | Mobile | 60% traffic mobile |
| #018 | Exact Export | 10-12h | 48-60h | Belgian | Logiciel #1 compta BE |
| #007 | Work Mgmt | 12-15h | 60-75h | Métier | Gestion travaux importants |
| #023 | Regulations | 12-15h | 72-90h | Scale | Multi-pays extensible |
| #006 | Payments | 15-20h | 87-110h | Métier | Réduction impayés |
| #017 | CODA Import | 15-20h | 102-130h | Belgian | Format bancaire BE standard |

**Total Phase 2**: 102-130h (13-16 semaines)
**Livrables**:
- ✅ Hosted live (app.koprogo.com)
- ✅ Signup self-service
- ✅ 80 clients payants
- ✅ 480 copropriétés gérées
- ✅ 1,600€ MRR
- ✅ Conformité BE 100%
- ✅ PWA score >90

---

## 🟢 Phase 3 - Détail (Mois 13-20)

**Objectif**: Scale + innovation (IA, Analytics, Sustainability)

| Issue | Nom | Effort | Cumul | Type | Justification |
|-------|-----|--------|-------|------|---------------|
| #024 | Multi-currency | 6-8h | 6-8h | Expansion | Tunisie (TND) |
| #013 | Sustainability | 12-15h | 18-23h | Différenciation | PEB BE + Green marketing |
| #014 | Analytics BI | 12-15h | 30-38h | Premium | Syndics pro >10 copros |
| #011 | AI Features | 20-30h | 50-68h | Innovation | OCR, ML, Chatbot |

**Total Phase 3**: 50-68h (10-13 semaines)
**Livrables**:
- ✅ 150 clients cloud
- ✅ 1,000 copropriétés
- ✅ 4,000€ MRR
- ✅ Expansion France + Tunisie beta
- ✅ Features IA opérationnelles
- ✅ Certification "Green SaaS"

---

## ⚫ Phase 4 - Détail (Mois 20+)

**Objectif**: Leadership marché niche + long terme

| Issue | Nom | Effort | Cumul | Type | Justification |
|-------|-----|--------|-------|------|---------------|
| #025 | TLIS Tunisie | 15-20h | 15-20h | Expansion | Cadastre TN |
| #012 | Marketplace | 20-25h | 35-45h | Business | Revenue commissions |
| #015 | Mobile Native | 30-40h | 65-85h | UX | iOS/Android (si ROI validé) |

**Total Phase 4**: 65-85h (13-17 semaines)
**Livrables**:
- ✅ 350+ clients cloud
- ✅ 2,100+ copropriétés
- ✅ 7,000€+ MRR
- ✅ Marketplace live
- ✅ Apps stores (optionnel)
- ✅ Présence 5+ pays

---

## 🎯 Critères de Priorisation Appliqués

### 🔴 Critique = ANY of:
1. **Bloquant production**: Empêche lancement production-ready
2. **Legal blocker**: Non-conformité légale (AG, archivage, PCN)
3. **Hosted blocker**: Empêche modèle hosted (multi-tenancy, sécurité)
4. **Belgian market fit**: Obligatoire pour Belgique (CODA, i18n FR/NL, PCN)

### 🟡 Important = ANY of:
1. **Parité concurrence**: Feature standard chez Vilogi/Copriciel
2. **Étude mentionne**: Cité dans market analysis
3. **Scale enabler**: Permet croissance >100 copros (pagination, analytics)
4. **Différenciation forte**: Argument marketing unique (sustainability, IA)

### 🟢 Nice = ALL of:
1. **Pas bloquant MVP**
2. **Innovation long-terme**
3. **ROI incertain court terme**

### ⚫ Backlog = ALL of:
1. **Alternative existe** (PWA vs Mobile Native)
2. **Coût/effort très élevé**
3. **Validation marché requise d'abord**

---

## 📈 Évolution Effort Cumulé par Phase

```
Phase 1: ████████░░░░░░░░░░░░░░░░░░░░  67-83h   (23% total)
Phase 2: ████████████████████░░░░░░░░  169-213h (60% total)
Phase 3: ████████████████████████░░░░  219-281h (78% total)
Phase 4: ████████████████████████████  284-366h (100% total)
```

**Effort moyen par issue**: 12-15h
**Issues critiques intensité**: 8.4-10.4h (plus rapides, prioritaires)
**Issues nice/backlog intensité**: 22-28h (plus complexes, moins urgentes)

---

## 💰 Retour sur Investissement (ROI)

| Phase | Effort | MRR Cible | €/heure dev | CAC |
|-------|--------|-----------|-------------|-----|
| **Phase 1** | 67-83h | 190€ | 2.3-2.8€/h | ~20€/client |
| **Phase 2** | 102-130h | 1,600€ | 12-16€/h | ~15€/client |
| **Phase 3** | 50-68h | 4,000€ | 59-80€/h | ~12€/client |
| **Phase 4** | 65-85h | 7,000€ | 82-108€/h | ~10€/client |

**Rentabilité dès Phase 1** (mois 2)
**ROI positif** même avec timeline x2 (imprévus)

---

## 🚨 Top 5 Issues Critiques (À Faire ASAP)

| Rang | Issue | Nom | Effort | Bloque | Impact |
|------|-------|-----|--------|--------|--------|
| 🥇 1 | #004 | Pagination | 3-4h | Scale | 100% endpoints |
| 🥈 2 | #001 | Meetings | 6-8h | Legal | Cœur métier syndic |
| 🥉 3 | #002 | Documents | 8-10h | Legal | Archivage obligatoire |
| 4 | #019 | i18n FR/NL | 8-10h | Belgian | 60% Belgique néerlandophone |
| 5 | #005 | Security | 10-12h | Prod | Sécurité = dealbreaker |

**Total Top 5**: 35-44h (Sprint initial de 4-5 semaines)

---

## ✅ Checklist Validation Issues

Chaque issue doit satisfaire AVANT démarrage dev:

- [ ] **Specs techniques** détaillées (architecture, endpoints, entities)
- [ ] **User stories** BDD (Gherkin scenarios)
- [ ] **Critères acceptation** fonctionnels + performance
- [ ] **Tests plan** (unit, integration, E2E)
- [ ] **Documentation** utilisateur (FR + NL pour BE)
- [ ] **Dependencies** claires (bloque/bloquée par)
- [ ] **Estimation** validée (challenge 80/20)
- [ ] **Migration SQL** (si modifs schema)

---

## 🎉 Prochaines Étapes

### Immédiat (Semaine 1-2)
1. ✅ Créer GitHub Projects avec 4 milestones (Phase 1-4)
2. ✅ Convertir PRIORITIES_TABLE → GitHub issues (#001-#025)
3. ✅ Assigner issues Phase 1 → Sprint 1
4. ✅ Setup CI/CD (lint, test, build)
5. ✅ Démarrer #004 Pagination (issue la plus rapide)

### Court Terme (Mois 1-2)
6. ✅ Compléter Top 5 issues critiques (35-44h)
7. ✅ Premier beta test (5 syndics belges)
8. ✅ Feedback loop itération

### Moyen Terme (Mois 3-6)
9. ✅ Achever Phase 1 complète
10. ✅ Lancer hosted beta

---

**Tableau Vivant**: Mis à jour hebdomadaire selon avancement réel
**Dernière mise à jour**: 2025-10-23
**Prochaine révision**: Fin Phase 1 (Juin 2025)
