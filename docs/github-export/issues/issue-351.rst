=============================================================================
Issue #351: docs: harmoniser documentation avec état réel du code (mars 2026)
=============================================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: documentation,priority:high
:Assignees: Unassigned
:Created: 2026-03-28
:Updated: 2026-03-28
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/351>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Après la session du 26-28 mars 2026 (12 commits, ~160 fichiers modifiés), les documents clés du projet sont désynchronisés avec la réalité du code.
   
   ## Chiffres périmés à corriger
   
   | Métrique | Ancien (dans docs) | Réel |
   |----------|-------------------|------|
   | Endpoints API | 511 | **559** |
   | Entités domaine | 57 | **59** |
   | Migrations SQL | 64 | **80** |
   | LOC Rust backend | 110k+ | **137k+** |
   
   ## Métriques manquantes à ajouter
   
   - Frontend : 178 composants Svelte, 22 API clients, 13 utils/validators/services
   - Tests : 819 BDD scenarios, 49 E2E smoke, 12 Documentation Vivante
   - i18n : 4 langues (FR/NL/EN/DE), ~2000 clés, 73% couverture
   - Legal : 29 fichiers, 7 rôles, 65 règles codifiées
   
   ## Documents à mettre à jour
   
   - [ ] CLAUDE.md — Chiffres Jalon 0 + métriques frontend/tests/i18n
   - [ ] README.md — Chiffres + section branches + section infrastructure
   - [ ] docs/WBS_PROJET_COMPLET.rst — Note mise à jour + issues #343-#350
   - [ ] docs/ROADMAP_PAR_CAPACITES.rst — Version 7.0 + note
   - [ ] docs/WBS_RELEASE_0_1_0.md — Matrice Playwright + issues
   
   Generated with [Claude Code](https://claude.com/claude-code)

.. raw:: html

   </div>

