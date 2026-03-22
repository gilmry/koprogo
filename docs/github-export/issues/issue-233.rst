=======================================================================================
Issue #233: R&D: Stratégies de test avancées (property-based, contract, load testing)
=======================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: priority:low,testing R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-16
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/233>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   La suite de tests actuelle (unit + BDD + E2E) couvre les scénarios connus.
   Cette R&D explore les techniques complémentaires pour la robustesse.
   
   ## Objectifs de la R&D
   
   1. **Property-based testing** (Rust) :
      - ``proptest`` crate pour domain entities
      - Génération aléatoire de données (emails, montants, dates)
      - Invariants à vérifier (quotes-parts ≤ 100%, montants ≥ 0)
      - Shrinking automatique pour trouver les cas minimaux
   
   2. **Contract testing** :
      - Pact.io pour API contracts frontend ↔ backend
      - Vérification automatique que le frontend n'appelle pas d'endpoints cassés
      - Integration dans CI/CD
   
   3. **Load testing** :
      - k6 (JavaScript, léger, CI-friendly)
      - Locust (Python, distribué)
      - Scenarios : 100 utilisateurs simultanés, burst 1000
      - Métriques : P99 < 5ms maintenu sous charge
   
   4. **Chaos engineering** (Jalon K8s) :
      - Litmus (Kubernetes-native)
      - Scenarios : perte de noeud, partition réseau, disk full
      - Game days trimestriels
   
   ## Points de décision
   
   - [ ] Proptest : quels domain entities en premier
   - [ ] Load testing tool (k6 vs. Locust)
   - [ ] Objectifs de performance formalisés (SLA)
   - [ ] Budget CI pour tests de charge (runners dédiés)
   
   ## Estimation
   
   6-8h

.. raw:: html

   </div>

