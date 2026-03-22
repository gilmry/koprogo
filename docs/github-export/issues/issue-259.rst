==============================================
Issue #259: MCP Tool: transmission_lot_dossier
==============================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:mcp release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/259>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Implémenter l'outil MCP de génération du dossier de transmission de lot (vente d'appartement).
   
   ## Outil
   
   ### transmission_lot_dossier
   Génère le dossier complet d'information art. 3.94 pour le notaire :
   - Montants fonds de roulement et fonds de réserve
   - Arriérés du vendeur
   - Appels de fonds en cours
   - PV des 3 dernières AG
   - Dernier bilan approuvé
   - Procédures judiciaires en cours
   
   Délai légal : 15 jours. Formats PDF/ZIP.
   
   ## Input Schema
   
   Voir `backend/koprogo-mcp/README.md` section 6 pour le schema JSON complet.
   
   ## Tâches
   
   - [ ] Créer `src/mcp/tools/transmission.rs`
   - [ ] Agréger les données financières du lot (FR, FdR, arriérés)
   - [ ] Collecter les PV des 3 dernières AG
   - [ ] Générer le dossier PDF/ZIP conforme art. 3.94
   - [ ] Respecter filtrage par rôle (copropriétaire = son lot)
   - [ ] Tests unitaires
   
   ## Dépendances
   
   - Bloqué par #252, #253
   - Réutilise : `EtatDateUseCases`, `AccountUseCases`, `MeetingUseCases`, `DocumentUseCases`
   - Réf légale : `docs/legal/notaire/transmission_lot.rst` (N01-N05)

.. raw:: html

   </div>

