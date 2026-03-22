========================================================
Issue #254: MCP Tool: legal_search + majority_calculator
========================================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:mcp release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/254>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Implémenter les outils MCP de référence légale belge, s'appuyant sur la base documentaire `docs/legal/`.
   
   ## Outils
   
   ### legal_search
   Recherche dans la base légale par mot-clé, code de règle (AG09, M02, CP07...), rôle, ou catégorie.
   
   ### majority_calculator
   Calcule la majorité requise (absolue, 2/3, 4/5, unanimité) selon le type de décision AG. Cite l'article du Code civil applicable. Peut calculer concrètement avec les quotes-parts d'une copropriété.
   
   ## Input Schemas
   
   Voir `backend/koprogo-mcp/README.md` section 1 pour les schemas JSON complets.
   
   ## Tâches
   
   - [ ] Créer `src/mcp/tools/legal.rs`
   - [ ] Parser et indexer les fichiers `docs/legal/**/*.rst` (codes de règles, articles, rôles)
   - [ ] Implémenter la recherche full-text sur la base légale
   - [ ] Implémenter le lookup par code de règle (ex: AG09 → art. 3.88 §1)
   - [ ] Implémenter le calculateur de majorités (absolue, 2/3, 4/5, unanimité)
   - [ ] Mapper les types de décision aux majorités requises (Code civil art. 3.88)
   - [ ] Tests avec scénarios réels (travaux parties communes, modification statuts, etc.)
   
   ## Sources légales
   
   - Code civil belge, Livre 3, Titre 1, Sous-titre 3 (Art. 3.78 à 3.100)
   - Code de déontologie IPI (AR 29/06/2018)
   - Convention de nommage : voir `docs/legal/README.rst`
   
   ## Dépendances
   
   - Bloqué par #252 (serveur SSE), #262 (indexation base légale)
   - Fichiers `docs/legal/` ✅ créés (commit `64c35f2`)

.. raw:: html

   </div>

