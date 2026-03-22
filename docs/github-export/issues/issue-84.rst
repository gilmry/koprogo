====================================================================
Issue #84: feat: Online Payment System (Stripe + SEPA Direct Debit)
====================================================================

:State: **CLOSED**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,phase:vps track:software,priority:high finance
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2025-11-17
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/84>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #006 - Système de Paiement en Ligne
   
   **Priorité**: 🟡 IMPORTANT  
   **Estimation**: 15-20 heures  
   **Phase**: VPS MVP - Phase 2 Automation  
   
   ## 📋 Description
   
   Intégration Stripe + SEPA Direct Debit pour paiements en ligne des charges de copropriété.
   
   ## 🎯 Objectifs
   
   - [ ] Intégration Stripe Payment Intent API
   - [ ] SEPA Direct Debit (prélèvements automatiques)
   - [ ] Webhook handlers pour confirmations paiement
   - [ ] Réconciliation automatique paiements/expenses
   - [ ] Dashboard paiements pour propriétaires
   - [ ] Historique transactions avec statuts
   
   ## 📐 Méthodes de Paiement
   
   1. **Carte bancaire** (Stripe)
   2. **SEPA Direct Debit** (prélèvement automatique)
   3. **Virement bancaire** (manuel, avec référence structurée)
   
   ## ✅ Critères d'Acceptation
   
   - Paiements Stripe fonctionnels
   - SEPA mandats gérés
   - Webhooks sécurisés
   - Réconciliation automatique
   - Tests E2E paiement complet
   
   ---
   
   **Voir**: \`issues/important/006-online-payments.md\`

.. raw:: html

   </div>

