=============================================================
Issue #257: MCP Tool: comptabilite_situation + appel_de_fonds
=============================================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:mcp release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/257>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Implémenter les outils MCP de comptabilité et gestion des charges.
   
   ## Outils
   
   ### comptabilite_situation
   Situation financière de l'ACP : soldes fonds de roulement et fonds de réserve, charges en cours, arriérés par copropriétaire. Filtrage par exercice comptable.
   
   ### appel_de_fonds
   Génère un appel de fonds (ordinaire/spécial) réparti selon les quotes-parts. Conforme à l'art. 3.86 §3 : communication de la part affectée au fonds de réserve.
   
   ## Input Schemas
   
   Voir `backend/koprogo-mcp/README.md` section 4 pour les schemas JSON complets.
   
   ## Tâches
   
   - [ ] Créer `src/mcp/tools/comptabilite.rs`
   - [ ] comptabilite_situation : agrégation comptes PCMN + arriérés par owner
   - [ ] appel_de_fonds : calcul répartition quotes-parts + génération
   - [ ] Respecter filtrage par rôle (copropriétaire = son lot uniquement)
   - [ ] Tests unitaires
   
   ## Dépendances
   
   - Bloqué par #252, #253
   - Réutilise : `AccountUseCases`, `FinancialReportUseCases`, `CallForFundsUseCases`, `OwnerContributionUseCases`

.. raw:: html

   </div>

