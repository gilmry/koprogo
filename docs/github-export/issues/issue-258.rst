=======================================
Issue #258: MCP Tool: travaux_qualifier
=======================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:mcp release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/258>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Implémenter l'outil MCP de qualification des travaux.
   
   ## Outil
   
   ### travaux_qualifier
   Aide à qualifier un travail : urgent/conservatoire (syndic peut agir seul, art. 3.89 §5 2°) vs non-urgent (nécessite décision AG, art. 3.88 §1 1°b). Détermine la majorité requise en fonction du montant et de la nature des travaux.
   
   ## Input Schema
   
   Voir `backend/koprogo-mcp/README.md` section 5 pour le schema JSON complet.
   
   ## Tâches
   
   - [ ] Créer `src/mcp/tools/travaux.rs`
   - [ ] Logique de qualification : urgent vs non-urgent (critères légaux)
   - [ ] Calcul de la majorité requise selon type de travaux
   - [ ] Intégrer les seuils belges (montant, nature)
   - [ ] Référencer `docs/legal/syndic/travaux.rst` (T01-T05)
   - [ ] Tests avec scénarios (fuite urgente, ravalement façade, ascenseur)
   
   ## Dépendances
   
   - Bloqué par #252, #253
   - Réf légale : `docs/legal/syndic/travaux.rst`

.. raw:: html

   </div>

