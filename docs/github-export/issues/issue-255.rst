=============================================================
Issue #255: MCP Tool: copropriete_info + list_coproprietaires
=============================================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:mcp release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/255>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Implémenter les outils MCP de consultation des données de copropriété.
   
   ## Outils
   
   ### copropriete_info
   Récupère les informations d'une copropriété : adresse, n° BCE, nombre de lots, syndic en fonction, date d'expiration du mandat.
   
   ### list_coproprietaires
   Liste des copropriétaires avec leurs lots, quotes-parts, et coordonnées. Option pour inclure les locataires.
   
   ## Input Schemas
   
   Voir `backend/koprogo-mcp/README.md` section 2 pour les schemas JSON complets.
   
   ## Tâches
   
   - [ ] Créer `src/mcp/tools/copropriete.rs`
   - [ ] Brancher sur les use cases existants : `BuildingUseCases`, `UnitOwnerUseCases`
   - [ ] Implémenter copropriete_info (building + syndic info + units count)
   - [ ] Implémenter list_coproprietaires (owners + units + quotes-parts)
   - [ ] Respecter le filtrage par rôle (copropriétaire = vue limitée à son lot)
   - [ ] Tests unitaires
   
   ## Dépendances
   
   - Bloqué par #252 (serveur SSE), #253 (auth)
   - Réutilise : `BuildingUseCases`, `UnitOwnerUseCases`, `OwnerUseCases`

.. raw:: html

   </div>

