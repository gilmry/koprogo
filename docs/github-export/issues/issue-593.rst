==========================================================================
Issue #593: [Story Tx.1] Caractérisation reste VERTE (gate CI inter-slice)
==========================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,testing e2e,playwright maury,track-h-conformite slice-Tx
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/593>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story Tx.1 — Caractérisation reste VERTE (gate CI inter-slice)
   
   > Maury Phase 6 Exécution · Slice Transversal · Story `story/Tx.1-characterization-ci-gate` · Refs: #556
   
   ## Goal
   
   Job CI dédié `test:characterization` qui tourne sur **chaque PR** des slices 1-5. Échec = blocage merge.
   
   ## Contexte Maury
   
   - **FR/INV** : FR43 ; mémoire [[fe-refactor-test-driven]]
   - **Effort** : S
   - **Deps** : Story 0.1
   - **ADR refs** : ADR-0013
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : PR slice 2 → characterization VERT → mergeable
   - **@edge** : Caractérisation partiellement modifiée (commit qui ajuste un test bugué) → review explicite obligatoire
   - **@security** : Bypass via `--no-verify` impossible (CI server-side)
   - **@negative** : Characterization ROUGE → blocage + alerte + investigation
   
   ## Files
   
   - `.github/workflows/ci.yml` (job characterization)
   - `package.json` (script `test:characterization`)
   
   ## Definition of Done
   
   - [ ] Job CI characterization tourne sur chaque PR
   - [ ] Blocage merge si ROUGE
   - [ ] Script `npm run test:characterization` opérationnel
   - [ ] Démarrage **immédiat** dès story 0.1 mergée
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §8 Story Tx.1
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

