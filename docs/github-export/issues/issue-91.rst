================================================================
Issue #91: feat: Contractor Quotes Module with Multi-Comparison
================================================================

:State: **CLOSED**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,phase:k3s track:software,priority:high finance
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2026-03-21
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/91>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #024 - Module Devis Travaux
   
   **Priorité**: 🟡 IMPORTANT  
   **Estimation**: 8-10 heures  
   **Phase**: Phase 2 K3s + Automation  
   
   ## 📋 Description
   
   Gestion devis avec comparaison multi-entrepreneurs + scoring automatique.
   
   **Obligation légale**: 3 devis obligatoires pour travaux >5000€
   
   ## 🎯 Objectifs
   
   - [ ] Entity Quote (contractor, description, amount, validity, status)
   - [ ] Comparaison multi-devis: tableau prix + délais
   - [ ] Scoring auto: prix (40%), délai (30%), garanties (20%), réputation (10%)
   - [ ] Workflow: demande → réception → comparaison → vote AG → attribution
   - [ ] Tracking: devis acceptés → WorkReport (carnet #020)
   - [ ] Historique contractors: notes, délais, qualité
   
   ## 📐 Scoring Automatique
   
   ```rust
   score = (prix * 0.4) + (delai * 0.3) + (garanties * 0.2) + (reputation * 0.1)
   ```
   
   ## ✅ Critères d'Acceptation
   
   - Système devis complet
   - Algorithme scoring automatique
   - Dashboard comparaison visuelle
   - Tests E2E workflow
   
   ---
   
   **Voir**: \`issues/important/024-module-devis-travaux.md\`

.. raw:: html

   </div>

