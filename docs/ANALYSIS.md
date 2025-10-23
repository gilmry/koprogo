# Analyse des Issues Existantes (#001-#015)

**Date**: 2025-10-23
**Auteur**: Révision Priorités KoproGo
**Version**: 1.0

---

## 🎯 Métrique de Décision

> "Un syndic choisit-il KoproGo plutôt que Sogis/Vilogi (15-30€/mois) ?"

**Critères d'Analyse**:
1. **Bloquant Production**: Empêche lancement production-ready
2. **Concurrence A**: Présent chez Vilogi/Copriciel/Septeo
3. **Étude Mentionne**: Cité dans l'analyse de marché
4. **Hosted**: Critique pour modèle hosted 1€/mois

---

## 🔴 Issues Critiques (#001-#005)

### #001 - Meeting Management API
- **Bloquant Production**: ✅ OUI - Obligation légale syndic
- **Concurrence A**: ✅ OUI - 100% concurrents ont
- **Étude Mentionne**: ✅ OUI - "AG obligation légale pour toute copropriété"
- **Priorité Révisée**: 🔴 **CRITIQUE**
- **Action**: **GARDER** - Phase 1
- **Justification**: Sans gestion AG, pas de syndic. Bloque production.

---

### #002 - Document Upload/Download
- **Bloquant Production**: ✅ OUI - Archivage légal (PV, factures, règlements)
- **Concurrence A**: ✅ OUI
- **Étude Mentionne**: ✅ OUI - "Archivage légal de documents"
- **Priorité Révisée**: 🔴 **CRITIQUE**
- **Action**: **GARDER** - Phase 1
- **Justification**: Stockage PV AG = obligation légale.

---

### #003 - Financial Reports Generation
- **Bloquant Production**: ✅ OUI - Transparence financière obligatoire en AG
- **Concurrence A**: ✅ OUI
- **Étude Mentionne**: ✅ OUI - "Export FEC (export comptable)", "Rapports financiers"
- **Priorité Révisée**: 🔴 **CRITIQUE**
- **Action**: **GARDER** - Phase 1
- **Justification**: Appels de fonds + budget prévisionnel = cœur métier syndic.

---

### #004 - Pagination & Filtering
- **Bloquant Production**: ✅ OUI - Performance scalability (>100 copros)
- **Concurrence A**: ✅ OUI - Standard technique
- **Étude Mentionne**: ❌ NON - Requirement technique
- **Priorité Révisée**: 🔴 **CRITIQUE**
- **Action**: **GARDER** - Phase 1
- **Justification**: Sans pagination, UX catastrophique avec >50 buildings. Bloque scale.

---

### #005 - Security Hardening
- **Bloquant Production**: ✅ OUI - Sécurité = production-blocker
- **Concurrence A**: ✅ OUI - Standard sécurité
- **Étude Mentionne**: ✅ OUI - "GDPR compliant"
- **Hosted**: ✅ CRITIQUE - Hosted impossible sans sécurité
- **Priorité Révisée**: 🔴 **CRITIQUE**
- **Action**: **GARDER** - Phase 1
- **Justification**: Rate limiting, JWT refresh, CORS = minimum sécurité production.

---

## 🟡 Issues Importantes (#006-#010)

### #006 - Online Payments (Stripe)
- **Bloquant Production**: ❌ NON - Paiements manuels possibles (MVP)
- **Concurrence A**: ✅ OUI - Vilogi, Copriciel ont
- **Étude Mentionne**: ✅ OUI - "Paiements en ligne (Stripe/PayPal)"
- **Priorité Révisée**: 🟡 **IMPORTANT** (Phase 2)
- **Action**: **DESCENDRE** - Pas critique MVP mais essentiel croissance
- **Justification**: Améliore recouvrement mais pas bloquant. Phase 2 acceptable.

---

### #007 - Work Management
- **Bloquant Production**: ❌ NON
- **Concurrence A**: ✅ OUI
- **Étude Mentionne**: ✅ OUI - "Gestion travaux importants (ravalement, toiture)"
- **Priorité Révisée**: 🟡 **IMPORTANT** (Phase 2)
- **Action**: **GARDER**
- **Justification**: Feature importante mais MVP fonctionne sans.

---

### #008 - Ticketing System
- **Bloquant Production**: ❌ NON
- **Concurrence A**: ✅ OUI
- **Étude Mentionne**: ⚠️ IMPLICITE - Maintenance mentionnée
- **Priorité Révisée**: 🟡 **IMPORTANT** (Phase 2)
- **Action**: **GARDER**
- **Justification**: Améliore satisfaction résidents, pas bloquant MVP.

---

### #009 - Notifications System
- **Bloquant Production**: ❌ NON - Email basique suffit MVP
- **Concurrence A**: ✅ OUI
- **Étude Mentionne**: ✅ OUI - "Communication essentielle"
- **Priorité Révisée**: 🟡 **IMPORTANT** (Phase 2)
- **Action**: **MONTER** - Plus important qu'on pensait pour rétention
- **Justification**: Engagement utilisateurs critique. Notifications push = standard moderne.

---

### #010 - Progressive Web App (PWA)
- **Bloquant Production**: ❌ NON
- **Concurrence A**: ✅ OUI - Mobile experience attendue
- **Étude Mentionne**: ⚠️ IMPLICITE - "Mobile (PWA min)"
- **Priorité Révisée**: 🟡 **IMPORTANT** (Phase 2)
- **Action**: **MONTER** - Critique pour UX mobile
- **Justification**: 60%+ traffic mobile. PWA > App native court terme.

---

## 🟢 Issues Nice-to-Have (#011-#015)

### #011 - AI Features (OCR, Predictions)
- **Bloquant Production**: ❌ NON
- **Concurrence A**: ❌ NON - **DIFFÉRENCIATION**
- **Étude Mentionne**: ❌ NON
- **Priorité Révisée**: 🟢 **NICE** (Phase 3-4)
- **Action**: **GARDER** - Backlog différenciation
- **Justification**: Innovation forte mais pas MVP. Phase 3 pour différenciation marché.

---

### #012 - Marketplace Prestataires
- **Bloquant Production**: ❌ NON
- **Concurrence A**: ❌ NON - **DIFFÉRENCIATION**
- **Étude Mentionne**: ❌ NON
- **Priorité Révisée**: 🟢 **NICE** (Phase 4)
- **Action**: **GARDER** - Backlog long terme
- **Justification**: Business model intéressant (commission) mais pas prioritaire.

---

### #013 - Sustainability (PEB, Carbon Footprint)
- **Bloquant Production**: ❌ NON
- **Concurrence A**: ❌ NON - **DIFFÉRENCIATION**
- **Étude Mentionne**: ⚠️ PARTIEL - "Tracking PEB" mentionné pour Belgique/Tunisie
- **Priorité Révisée**: 🟡 **IMPORTANT** (Phase 3) - **MONTER**
- **Action**: **MONTER** - Argument marketing unique + aligné <0.5g CO2/req
- **Justification**: PEB obligatoire Belgique. Argument marketing "Green SaaS" fort.

---

### #014 - Analytics & BI
- **Bloquant Production**: ❌ NON
- **Concurrence A**: ✅ OUI - Syndics pro ont besoin
- **Étude Mentionne**: ❌ NON explicitement
- **Priorité Révisée**: 🟡 **IMPORTANT** (Phase 2-3)
- **Action**: **MONTER** - Important pour syndics gérant >10 immeubles
- **Justification**: Cible "Syndics professionnels - Portfolio" = segment important.

---

### #015 - Mobile Native (React Native)
- **Bloquant Production**: ❌ NON - PWA suffit Phase 1-2
- **Concurrence A**: ✅ OUI
- **Étude Mentionne**: ✅ OUI - "Mobile app"
- **Priorité Révisée**: ⚫ **BACKLOG** (Phase 4)
- **Action**: **DESCENDRE** - PWA d'abord (#010), app native après
- **Justification**: 30-40h développement + 150€ coûts. PWA couvre 80% besoins.

---

## 📊 Résumé Actions

| Action | Issues | Justification |
|--------|--------|---------------|
| **GARDER** | #001-#005, #007-#008, #011-#012, #015 | Scope validé |
| **MONTER** | #009, #010, #013, #014 | Plus importants qu'anticipé |
| **DESCENDRE** | #006 | Important mais pas bloquant MVP |

---

## 🚨 Issues MANQUANTES Identifiées

L'étude de marché révèle des **gaps critiques** non couverts par #001-#015:

### 🔴 Critiques pour Belgique (Bloquent lancement BE)
- **PCN Belge** (Plan Comptable Normalisé) - Conformité comptable obligatoire
- **CODA Import** (format bancaire belge) - Standard bancaire BE
- **i18n FR/NL/EN** - Bilinguisme obligatoire Belgique
- **Multi-tenancy Parfait** - Crucial modèle hosted 1€/mois
- **Belgian Council Management** - Conseil copropriété >20 lots (obligation légale)

### 🟡 Importantes
- **Exact Online Export** - Logiciel compta #1 Belgique
- **Country Regulations Engine** - Support BE/FR/ES/IT/TN règles
- **Stripe Billing 1€** - Monétisation hosted
- **Multi-currency (EUR/TND)** - Expansion Tunisie

### Total Gaps: ~9 issues critiques/importantes non planifiées

---

## 💡 Recommandations Stratégiques

### Phase 1 (Mois 1-6) - MVP Production-Ready
**Garder**: #001-#005 (37-46h)
**Ajouter**:
- PCN Belge (#016) - 12-15h
- i18n FR/NL (#019) - 8-10h
- Multi-tenancy (#020) - 10-12h
- Belgian Council (#022) - 6-8h

**Total Phase 1**: ~73-91h (9-11 semaines à 20h/semaine)

### Phase 2 (Mois 7-12) - Hosted Beta Belgique
**Garder**: #006-#010 (53-64h)
**Ajouter**:
- CODA Import (#017) - 15-20h
- Exact Export (#018) - 10-12h
- Stripe Billing (#021) - 6-8h
- Regulations Engine (#023) - 12-15h

**Total Phase 2**: ~96-119h (12-15 semaines)

### Phase 3 (Mois 13-24) - Scale + Innovation
- #013 Sustainability (12-15h) - Argument marketing
- #014 Analytics BI (12-15h) - Syndics pro
- #011 AI Features (20-30h) - Différenciation

### Phase 4 (Mois 24+) - Long Terme
- #012 Marketplace (20-25h)
- #015 Mobile Native (30-40h)
- Tunisia expansion (#024, #025)

---

**Dernière mise à jour**: 2025-10-23
**Prochaine étape**: Créer NEW_ISSUES.md avec specs des 9 issues manquantes
