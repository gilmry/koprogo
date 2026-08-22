=====================================================================================
Issue #341: fix: Paiement auto contractor post-validation non implémenté (TODO B16-6)
=====================================================================================

:State: **CLOSED**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,track:software priority:medium
:Assignees: Unassigned
:Created: 2026-03-25
:Updated: 2026-03-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/341>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Problème
   
   Dans `backend/src/application/use_cases/contractor_report_use_cases.rs:211` :
   
   ```rust
   // TODO (B16-6) : déclencher paiement automatique si quote_id présent
   ```
   
   Quand un rapport de prestataire est validé (`Validated`), le paiement automatique lié au devis n'est pas déclenché.
   
   ## Comportement attendu
   
   Après validation du rapport (BC16), si un `quote_id` est associé :
   1. Créer un `Payment` avec le montant du devis
   2. Déclencher le workflow de paiement (Stripe/SEPA)
   3. Notifier le prestataire
   
   ## Sévérité
   
   MEDIUM — Le workflow fonctionne manuellement mais le chaînage automatique ticket→rapport→validation→paiement est incomplet.

.. raw:: html

   </div>

