===============================================================================
Issue #224: R&D: Architecture BI Dashboard - Pipeline d'agrégation de données
===============================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement,priority:medium R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/224>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Le dashboard BI (issue #97) nécessite une architecture de données pour l'agrégation
   temps réel des métriques financières, énergétiques et communautaires.
   
   **Issue liée**: #97
   
   ## Objectifs de la R&D
   
   1. **Bibliothèque de visualisation** :
      - Chart.js (léger, bien supporté Svelte)
      - Apache ECharts (puissant, mais plus lourd)
      - D3.js (flexible, mais complexe)
      - Plotly (interactif, export natif)
   
   2. **Pipeline d'agrégation** :
      - Vues matérialisées PostgreSQL (rafraîchissement périodique)
      - Pre-computed aggregates dans tables dédiées
      - Time-series extensions (timescaledb vs. pg_partman)
      - Cache layer (Redis pour métriques hot)
   
   3. **Métriques clés à agréger** :
      - Charges : tendances mensuelles, YoY, par catégorie
      - Paiements : taux de recouvrement, délais moyens
      - Énergie : consommation par bâtiment, anomalies
      - Communauté : participation SEL, taux d'engagement
      - Tickets : SLA respecté, temps moyen résolution
   
   4. **Export** :
      - PDF (rapports formatés pour AG)
      - CSV/Excel (données brutes pour comptables)
      - API JSON (intégration outils tiers)
   
   ## Points de décision
   
   - [ ] Choix bibliothèque charts
   - [ ] Stratégie d'agrégation (vues matérialisées vs. pre-computed)
   - [ ] Fréquence de rafraîchissement (temps réel vs. batch)
   - [ ] Multi-building aggregation pour Organization Admin
   
   ## Estimation
   
   12-16h

.. raw:: html

   </div>

