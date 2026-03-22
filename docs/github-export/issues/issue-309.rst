============================================================================================================================
Issue #309: [Architecture] Connecter la chaîne d'approbation des dépenses (Ticket → Rapport → Validation → Dépense)
============================================================================================================================

:State: **OPEN**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: conformité,architecture
:Assignees: Unassigned
:Created: 2026-03-22
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/309>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   Les modules Ticket, ContractorReport, BoardDecision et Expense existent individuellement mais ne sont PAS connectés dans un workflow métier.
   
   ## Workflow attendu
   1. **Ticket créé** → Ordre de service généré
   2. **Ordre de service accepté** → Magic link PWA envoyé au prestataire
   3. **Prestataire soumet rapport** via PWA (photos, dictée vocale FR-BE, pièces)
   4. **Rapport validé** par CdC (si >20 lots) ou Syndic (si ≤20 lots)
   5. **Dépense approuvée** → Écriture comptable automatique + paiement déclenché
   
   ## Actions requises
   - [ ] Créer l'entité WorkOrder (lien Ticket → ContractorReport)
   - [ ] Ajouter FK contractor_report_id à Expense
   - [ ] Rendre ContractorReport obligatoire avant Expense.approve()
   - [ ] Ajouter FK board_decision_id à Expense si immeuble >20 lots
   - [ ] Auto-trigger magic link PWA à l'acceptation de l'ordre de service
   - [ ] Créer BuildingContractor (registre fournisseurs officiels ACP)
   
   ## Réf. rapport
   rapport-tests-e2e-koprogo.docx — Section 3.3

.. raw:: html

   </div>

