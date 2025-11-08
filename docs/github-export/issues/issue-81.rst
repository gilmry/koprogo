=============================================================
Issue #81: feat: Annual Budget System with Variance Analysis
=============================================================

:State: **OPEN**
:Milestone: Phase 1: VPS MVP + Legal Compliance
:Labels: enhancement,phase:vps track:software,priority:critical finance,legal-compliance
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2025-11-08
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/81>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #018 - Budget Prévisionnel Annuel
   
   **Priorité**: 🔴 CRITIQUE  
   **Estimation**: 8-10 heures  
   **Phase**: VPS MVP (Nov 2025 - Mar 2026)  
   
   ## 📋 Description
   
   Système de budget annuel (ordinaire + extraordinaire) avec variance analysis mensuelle. Obligation légale: vote du budget en AG avant début exercice fiscal.
   
   ## 🎯 Objectifs
   
   - [ ] Entity Budget (fiscal_year, ordinary_budget, extraordinary_budget, status)
   - [ ] Calcul automatique provisions mensuelles
   - [ ] Variance analysis (budget vs actual) mensuelle
   - [ ] Vote AG obligatoire avant exercice fiscal
   - [ ] Endpoints: POST /buildings/:id/budget, GET /budget/:year/variance
   - [ ] Dashboard syndic: alertes dépassements budgétaires
   
   ## 📐 Structure Budget
   
   ```rust
   pub struct Budget {
       pub id: Uuid,
       pub building_id: Uuid,
       pub fiscal_year: i32,
       pub ordinary_budget: Decimal,  // Charges courantes
       pub extraordinary_budget: Decimal,  // Travaux
       pub status: BudgetStatus,  // Draft, Voted, Active
       pub approved_at: Option<DateTime>,
       pub approved_by_meeting_id: Option<Uuid>,
   }
   ```
   
   ## 🔗 Dépendances
   
   **Dépend de**: #016 (Plan Comptable pour catégorisation)
   
   ## ✅ Critères d'Acceptation
   
   - Génération PDF budget pour vote AG
   - Calcul provisions mensuelles automatique
   - Alertes dépassements > 10%
   - Rapports variance trimestriels
   - Tests E2E workflow complet
   
   ---
   
   **Voir**: \`issues/critical/018-budget-previsionnel.md\`

.. raw:: html

   </div>

