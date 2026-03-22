========================================================
Issue #261: MCP Tool: documents_list + document_generate
========================================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:mcp release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/261>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Implémenter les outils MCP de gestion documentaire.
   
   ## Outils
   
   ### documents_list
   Liste les documents d'une copropriété par catégorie : statuts, ROI, PV d'AG, contrats, polices d'assurance, devis, factures.
   
   ### document_generate
   Génère un document type conforme à la législation belge :
   - Convocation AG
   - Procuration
   - Contrat syndic
   - Rapport d'évaluation des contrats
   - Affichage entrée immeuble (coordonnées syndic)
   - Appel de fonds
   - PV d'AG
   - Dossier transmission lot
   - Inventaire transition syndic
   
   ## Input Schemas
   
   Voir `backend/koprogo-mcp/README.md` section 8 pour les schemas JSON complets.
   
   ## Tâches
   
   - [ ] Créer `src/mcp/tools/documents.rs`
   - [ ] documents_list : brancher sur `DocumentUseCases` avec filtrage catégorie
   - [ ] document_generate : templates PDF pour chaque type de document
   - [ ] Respecter filtrage par rôle (locataire = ROI uniquement, copropriétaire = non privé)
   - [ ] Tests unitaires
   
   ## Dépendances
   
   - Bloqué par #252, #253
   - Réutilise : `DocumentUseCases`
   - Templates PDF à créer (nouvelle dépendance : bibliothèque PDF Rust)

.. raw:: html

   </div>

