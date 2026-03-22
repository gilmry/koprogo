===============================================================================
Issue #222: R&D: Architecture de génération PDF pour documents légaux belges
===============================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement,priority:high legal-compliance,pdf R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/222>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   La génération de documents PDF est requise pour plusieurs modules :
   - PV d'assemblée générale (issue #47)
   - Convocations AG (issue #88 - partiellement implémenté)
   - États des dates (module existant)
   - Relevés de charges
   - Attestations syndic
   
   ## Objectifs de la R&D
   
   1. **Évaluer les bibliothèques** de génération PDF :
      - ``typst`` (Rust natif, templates markdown-like, compilation rapide)
      - ``weasyprint`` (Python, HTML/CSS → PDF, bon rendu)
      - ``wkhtmltopdf`` (CLI, HTML → PDF, legacy)
      - ``printpdf`` (Rust natif, bas niveau, performant)
      - ``tectonic`` (LaTeX → PDF, qualité typographique)
   
   2. **Critères d'évaluation** :
      - Performance (P99 < 500ms pour un PV de 5 pages)
      - Qualité typographique (headers, footers, numérotation)
      - Support multi-langue (FR/NL/DE)
      - Intégration de signatures numériques (eIDAS)
      - Taille du binaire (impact sur l'image Docker)
      - Maintenance (activité projet, communauté)
   
   3. **Templates légaux** :
      - PV AG conforme à la loi belge
      - État des dates (16 sections obligatoires)
      - Appels de fonds avec ventilation par lot
   
   ## Points de décision
   
   - [ ] Choix de la bibliothèque PDF
   - [ ] Architecture : génération synchrone vs. async (queue)
   - [ ] Stockage : filesystem local vs. S3
   - [ ] Caching des templates compilés
   - [ ] Stratégie de versioning des templates
   
   ## Livrables
   
   - Benchmark comparatif (vitesse, qualité, intégration)
   - Template prototype (1 PV AG de test)
   - ADR en RST
   
   ## Estimation
   
   10-15h

.. raw:: html

   </div>

