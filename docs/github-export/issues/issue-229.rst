===============================================================
Issue #229: R&D: Empreinte carbone et reporting ESG/durabilité
===============================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: priority:low,proptech:energy R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/229>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'issue #96 prévoit le suivi écologique. Cette R&D couvre les modèles
   de calcul et le reporting ESG pour les copropriétés.
   
   **Issue liée**: #96
   
   ## Objectifs de la R&D
   
   1. **Modèles de calcul CO₂** :
      - Facteurs d'émission belges (ADEME, SPF Climat)
      - Énergie : kWh → kg CO₂ (facteur par fournisseur/mix)
      - Eau : m³ → kg CO₂ (traitement, distribution)
      - Déchets : kg → kg CO₂ (par type de tri)
      - Transport : déplacements liés à la copro
   
   2. **Sources de données** :
      - Factures énergie (module existant)
      - Compteurs IoT (issue #109)
      - DPE/PEB (certificat performance énergétique)
      - Données publiques (Eurostat, climat.be)
   
   3. **Reporting** :
      - Bilan carbone annuel par bâtiment
      - Comparaison inter-bâtiments (benchmarking)
      - Objectifs de réduction (trajectoire 2030/2050)
      - Export PDF pour AG (synthèse environnementale)
   
   4. **Certifications** :
      - BREEAM In-Use
      - HQE Exploitation
      - Label éco-quartier
      - Primes régionales (Bruxelles Environnement, Wallonie Énergie)
   
   ## Points de décision
   
   - [ ] Base de facteurs d'émission (ADEME vs. SPF Climat)
   - [ ] Granularité (par lot vs. par bâtiment vs. par organisation)
   - [ ] Automatisation vs. saisie manuelle
   - [ ] Certification tierce-partie nécessaire ?
   
   ## Estimation
   
   10-12h

.. raw:: html

   </div>

