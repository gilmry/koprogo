====================================================================
Issue #77: feat: Financial Reports Generation (Rapports financiers)
====================================================================

:State: **OPEN**
:Milestone: Phase 1: VPS MVP + Legal Compliance
:Labels: enhancement,phase:vps track:software,priority:critical
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2025-11-01
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/77>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #003 - Génération de Rapports Financiers
   
   **Priorité**: 🔴 CRITIQUE  
   **Estimation**: 10-12 heures  
   **Phase**: VPS MVP (Nov 2025 - Mar 2026)  
   
   ## 📋 Description
   
   Système de génération automatique de rapports financiers pour les copropriétés (appels de fonds, états des impayés, budgets, rapports comptables FEC).
   
   ## 🎯 Types de Rapports
   
   1. **Appels de Fonds** (Call for Funds)
   2. **Budget Prévisionnel/Réel**
   3. **États des Impayés** (Outstanding Payments)
   4. **Situation Financière** (Financial Statement)
   5. **Charges par Propriétaire**
   6. **FEC Comptable** (Fichier Écritures Comptables)
   
   ## 📐 Endpoints
   
   | Méthode | Endpoint | Description |
   |---------|----------|-------------|
   | `GET` | `/api/v1/reports/call-for-funds/:building_id` | Appel de fonds |
   | `GET` | `/api/v1/reports/outstanding-payments/:building_id` | Impayés |
   | `GET` | `/api/v1/reports/budget/:building_id/:year` | Budget annuel |
   | `GET` | `/api/v1/reports/financial-statement/:building_id` | Situation financière |
   | `GET` | `/api/v1/reports/owner-charges/:owner_id` | Charges par proprio |
   | `GET` | `/api/v1/reports/fec/:building_id/:year` | FEC comptable |
   
   ## 🔗 Dépendances
   
   **Dépend de**: #016 (Plan Comptable Belge)  
   **Format**: PDF + Excel export
   
   ## ✅ Critères d'Acceptation
   
   - Génération PDF conformes législation belge
   - Export Excel (.xlsx) pour tous rapports
   - Génération FEC conforme norme française (optionnel export)
   - Tests E2E génération rapports
   - Multi-langue (FR/NL/DE/EN)
   
   ---
   
   **Voir**: `issues/critical/003-financial-reports-generation.md`

.. raw:: html

   </div>

