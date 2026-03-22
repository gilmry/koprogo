=========================================================================================================
Issue #277: feat: Guide légal contextuel UI — LegalHelper + AG Wizard (docs/legal/ source de vérité)
=========================================================================================================

:State: **OPEN**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: documentation,enhancement track:software,legal-compliance governance,release:0.1.0
:Assignees: Unassigned
:Created: 2026-03-11
:Updated: 2026-03-14
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/277>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   Le répertoire `docs/legal/` contient 22+ fichiers RST couvrant toutes les règles légales, déontologiques et best practices pour la copropriété belge. Ces informations doivent guider les utilisateurs en contexte dans l'UI.
   
   **Lien avec MCP** : ce guide alimentera aussi le MCP tool `legal_search` (issue #254).
   
   ## Composants
   
   ### Backend : Legal API
   `backend/src/infrastructure/web/handlers/legal_handlers.rs`
   - `GET /legal/rules?role=syndic&category=mandat` — règles par rôle
   - `GET /legal/rules/:code` — règle par code (L13, T03, AG7, CP01…)
   - `GET /legal/ag-sequence` — séquence OdJ complète avec majorités (backbone : `sequence_odj.rst`)
   - `GET /legal/majority-for/:decision-type` — majorité requise pour un type de décision
   
   Source : `backend/src/infrastructure/legal_index.json` (index statique généré depuis `docs/legal/`)
   
   ### Frontend : LegalHelper.svelte
   - Panneau latéral "?" flottant sur pages clés
   - Contenu contextuel selon page + rôle :
     - Page AG → séquence OdJ (sequence_odj.rst)
     - Page résolution → majorité requise
     - Page travaux → T03 mise en concurrence
   - Différent selon rôle : syndic pro (IPI déontologie) / syndic bénévole (guide simplifié) / copropriétaire (droits CP01-CP15)
   
   ### AG Wizard
   `frontend/src/pages/ag/wizard.astro` + `AgWizard.svelte`
   5 étapes guidées :
   1. Convocation (délais légaux, lien vidéo si visio)
   2. OdJ (template sequence_odj.rst, ajout points copropriétaires)
   3. Quorum (vérification + 2e convocation si KO)
   4. Résolutions (type → majorité suggérée automatiquement)
   5. PV (génération PDF, envoi 30j)
   
   ## Lien
   - `docs/legal/assemblee-generale/sequence_odj.rst` (backbone OdJ)
   - `docs/legal/matrice_conformite.rst` (règles indexées)
   - MCP #254 (`legal_search` tool)
   - BC15 AG Visioconférence #274 (étape 1 wizard)
   
   ## Definition of Done
   - [ ] legal_index.json généré avec tous les codes (L*, T*, G*, CP*, AG*, D*)
   - [ ] LegalHelper.svelte affiché sur page AG et page résolution
   - [ ] AG Wizard opérationnel en 5 étapes
   - [ ] Contenu différencié selon rôle

.. raw:: html

   </div>

