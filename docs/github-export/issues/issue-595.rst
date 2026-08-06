========================================================================
Issue #595: [Story Tx.3] Documentation docs/agent-activity/ (Tier 2 log)
========================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: documentation,track:software maury,track-h-conformite slice-Tx
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/595>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story Tx.3 — Documentation `docs/agent-activity/` (Tier 2 log)
   
   > Maury Phase 6 Exécution · Slice Transversal · Story `story/Tx.3-agent-activity-log` · Refs: #556
   
   ## Goal
   
   Pour chaque slice, créer `docs/agent-activity/YYYY-MM-DD-bob-slice-N.md` log Tier 2 (lecture/diagnostic/proposal). Conforme règle CRITICAL.md §11.
   
   ## Contexte Maury
   
   - **FR/INV** : transverse (gouvernance agent)
   - **Effort** : S (continu)
   - **Deps** : —
   - **ADR refs** : —
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : PR slice N inclut le log activity Tier 2 daté
   - **@edge** : Slice étalée sur plusieurs semaines → 1 fichier log par semaine
   - **@security** : Logs ne contiennent ni token ni secret
   - **@negative** : PR sans log Tier 2 → reviewer demande mise à jour
   
   ## Files
   
   - `docs/agent-activity/2026-MM-DD-bob-slice-N.md` (N=1..5)
   
   ## Definition of Done
   
   - [ ] Template `docs/agent-activity/_template.md` créé
   - [ ] Convention de naming documentée
   - [ ] PR slice N inclut log Tier 2 daté
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §8 Story Tx.3
   - CRITICAL.md §11 (Tier 1/Tier 2) · Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

