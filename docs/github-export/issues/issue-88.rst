============================================================================
Issue #88: feat: Automatic AG Convocations with Legal Deadline Verification
============================================================================

:State: **CLOSED**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,phase:k3s track:software,priority:high legal-compliance,automation pdf
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2026-03-21
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/88>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #019 - Convocations AG Automatiques
   
   **Priorité**: 🟡 IMPORTANT  
   **Estimation**: 5-7 heures  
   **Phase**: Phase 2 K3s + Automation  
   
   ## 📋 Description
   
   Génération automatique convocations AG avec PDF + email + vérification délais légaux.
   
   ## 🎯 Objectifs
   
   - [ ] Templates PDF convocations (FR/NL/DE/EN)
   - [ ] Vérification délais: 15j (AG ordinaire), 8j (extraordinaire)
   - [ ] Génération auto: ordre du jour + annexes
   - [ ] Envoi email auto avec PDF attaché
   - [ ] Accusés réception + relance J-3 si non ouvert
   - [ ] Tracking présences prévues vs effectives
   
   ## 📐 Délais Légaux Belges
   
   - AG Ordinaire: 15 jours minimum
   - AG Extraordinaire: 8 jours minimum
   - Première convocation manquée → deuxième convocation (8j)
   
   ## ✅ Critères d'Acceptation
   
   - Templates multi-langue conformes
   - Workflow automatique complet
   - Tests E2E convocation → réception
   - Dashboard syndic statut convocations
   
   ---
   
   **Dépend de**: #001 (Meeting API)  
   **Voir**: \`issues/important/019-convocations-ag-automatiques.md\`

.. raw:: html

   </div>

