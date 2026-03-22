============================================================================
Issue #256: MCP Tool: ag_create + ag_quorum_check + ag_vote + ag_generate_pv
============================================================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:mcp release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/256>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Implémenter les 4 outils MCP de gestion des assemblées générales.
   
   ## Outils
   
   ### ag_create
   Crée une AG (ordinaire/extraordinaire) avec OdJ conforme au Code civil belge. Points obligatoires (art. 3.89 §5 12° et 16°). Séquençage automatique selon dépendances légales. Support mode présentiel/hybride.
   
   ### ag_quorum_check
   Calcule le quorum de présence (quotes-parts présentes/représentées vs 50% requis). Indique la procédure de 2e AG si quorum non atteint (art. 3.87 §5).
   
   ### ag_vote
   Enregistre les votes sur un point de l'OdJ. Calcul automatique de la majorité. Plafonnement à 50% des voix (art. 3.87 §6).
   
   ### ag_generate_pv
   Génère le PV conforme à l'art. 3.87 §10 : majorités obtenues, noms des opposants et abstentionnistes. Délai de transmission : 30 jours. Formats PDF/DOCX.
   
   ## Input Schemas
   
   Voir `backend/koprogo-mcp/README.md` section 3 pour les schemas JSON complets.
   
   ## Tâches
   
   - [ ] Créer `src/mcp/tools/ag.rs`
   - [ ] ag_create : OdJ obligatoire + séquençage légal + points supplémentaires
   - [ ] ag_quorum_check : calcul quotes-parts + procédure 2e AG
   - [ ] ag_vote : majorité automatique + plafonnement 50%
   - [ ] ag_generate_pv : génération PDF/DOCX conforme
   - [ ] Intégrer `docs/legal/assemblee-generale/` pour les règles
   - [ ] Tests avec scénarios (AG ordinaire, extraordinaire, 2e convocation)
   
   ## Dépendances
   
   - Bloqué par #252, #253
   - Réutilise : `MeetingUseCases`, `ResolutionUseCases`, `ConvocationUseCases`
   - Réf légale : `docs/legal/assemblee-generale/sequence_odj.rst`

.. raw:: html

   </div>

