===================================================================
Issue #203: feat: Internationalization (i18n) - 4 Belgian Languages
===================================================================

:State: **CLOSED**
:Milestone: Jalon 5: Mobile & API Publique 📱
:Labels: enhancement,phase:vps track:software,priority:high
:Assignees: Unassigned
:Created: 2026-02-18
:Updated: 2026-02-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/203>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## ✅ DÉJÀ IMPLÉMENTÉ — Issue créée rétroactivement (audit 2026-02-18)
   
   Support multilingue pour les 4 langues pertinentes en Belgique.
   
   ### Frontend
   - **Module** : `frontend/src/lib/i18n.ts` (svelte-i18n)
   - **Composant** : `LanguageSelector.svelte`
   - **Dépendance** : `svelte-i18n` 4.0.1
   
   ### Langues supportées
   | Langue | Code | Fichier | Status |
   |--------|------|---------|--------|
   | Nederlands (Néerlandais) | `nl` | Translation file | ✅ Fallback default |
   | Français | `fr` | Translation file | ✅ |
   | Deutsch (Allemand) | `de` | Translation file | ✅ |
   | English | `en` | Translation file | ✅ |
   
   ### Intégration
   - Header `Accept-Language` envoyé au backend via `apiFetch()`
   - Sélecteur de langue dans la navigation
   - BDD feature file : `backend/tests/features/i18n.feature`
   - Backend : support multilingue convocations (FR/NL/DE/EN pour PDF)
   
   ### Contexte belge
   - NL par défaut (60% population)
   - FR, DE : langues officielles belges
   - EN : support international
   
   *Issue créée par audit de synchronisation code↔issues — feature déjà implémentée*

.. raw:: html

   </div>

