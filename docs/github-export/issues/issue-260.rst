==================================
Issue #260: MCP Tool: alertes_list
==================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:mcp release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/260>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Implémenter l'outil MCP de listing des alertes et rappels de conformité.
   
   ## Outil
   
   ### alertes_list
   Liste les alertes et rappels actifs selon le rôle de l'utilisateur :
   - Expiration mandat syndic
   - Renouvellement RC (responsabilité civile)
   - Formation continue IPI
   - Délai de convocation AG
   - Fonds de réserve insuffisant (minimum 5% budget annuel)
   - Inscription BCE manquante
   - PV non envoyé (délai 30 jours)
   - Contrats fournisseurs à évaluer
   
   ## Input Schema
   
   Voir `backend/koprogo-mcp/README.md` section 7 pour le schema JSON complet.
   
   ## Tâches
   
   - [ ] Créer `src/mcp/tools/alertes.rs`
   - [ ] Vérifier expiration mandat syndic (3 ans max, art. 3.89 §1)
   - [ ] Vérifier délais convocations (15j ordinaire, 8j extraordinaire)
   - [ ] Vérifier fonds de réserve (art. 3.86 §3)
   - [ ] Vérifier PV non transmis dans les 30 jours
   - [ ] Filtrer alertes par rôle (syndic, copropriétaire, commissaire, CdC)
   - [ ] Tests unitaires
   
   ## Dépendances
   
   - Bloqué par #252, #253
   - Réutilise : `BuildingUseCases`, `MeetingUseCases`, `ConvocationUseCases`, `BudgetUseCases`

.. raw:: html

   </div>

