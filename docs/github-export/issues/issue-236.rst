==================================================================================================
Issue #236: R&D: Orchestration achats groupés d'énergie - Workflow courtier et intégration CREG
==================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/236>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Le module achats groupés d'énergie est implémenté (#110, backend complet) mais l'orchestration
   complète du workflow de négociation collective n'est pas encore définie. Cette R&D couvre le
   processus de bout en bout : de la collecte des données de consommation jusqu'a la signature
   du contrat collectif, en passant par la négociation avec les fournisseurs.
   
   **Issues liées**: #110 (Energy Buying Groups), #109 (IoT/Linky), #133 (Smart Meters), #227 (R&D IoT)
   **Doc existante**: `docs/ENERGY_BUYING_GROUPS.rst`
   
   ## Objectifs de la R&D
   
   ### 1. Orchestration du workflow complet
   
   Le module actuel gère les entités (EnergyCampaign, ProviderOffer, EnergyBillUpload) mais
   pas l'orchestration de bout en bout. Il faut définir :
   
   ```
   Phase 1: Lancement campagne (syndic)
       |
       v
   Phase 2: Collecte consentements GDPR + factures énergie (copropriétaires)
       |  - Upload factures via PWA (photos OCR ou PDF)
       |  - Consentement explicite GDPR (art. 7)
       |  - K-anonymité >= 5 participants (seuil GDPR)
       |
       v
   Phase 3: Agrégation anonyme des données (système)
       |  - Total kWh par bâtiment (pas par unité)
       |  - Profils de consommation (heures pleines/creuses)
       |  - Puissance de négociation calculée
       |
       v
   Phase 4: Appel d'offres aux fournisseurs (courtier/syndic)
       |  - Génération automatique du cahier des charges
       |  - Envoi aux fournisseurs référencés (Engie, Luminus, TotalEnergies, Mega, etc.)
       |  - Délai de réponse (30 jours)
       |
       v
   Phase 5: Réception et comparaison des offres (système)
       |  - Scoring automatique (prix kWh, frais fixes, % vert, durée contrat)
       |  - Calcul économies estimées vs contrat actuel
       |  - Tableau comparatif pour présentation en AG
       |
       v
   Phase 6: Vote en AG (copropriétaires)
       |  - Présentation des offres (intégration module Resolutions #46)
       |  - Vote a la majorité simple (choix du fournisseur)
       |  - PV de décision (intégration PDF #47)
       |
       v
   Phase 7: Finalisation et switch collectif (courtier/système)
       |  - Signature contrat collectif
       |  - Coordination du switch avec le fournisseur retenu
       |  - Suivi des économies réelles post-switch
       |
       v
   Phase 8: Bilan annuel (système)
          - Comparaison économies prévues vs réelles
          - Rapport pour AG suivante
          - Décision renouvellement ou nouvelle campagne
   ```
   
   ### 2. Rôle du courtier en énergie
   
   - **Statut juridique belge** :
     - Courtier agréé CREG obligatoire pour négociations collectives
     - Charte de bonnes pratiques CREG (2013, actualisée 2018)
     - Label de qualité CREG : vérification automatisée possible ?
   - **Modèle d'intégration** :
     - Option A : KoproGo = outil pour courtier externe (SaaS B2B)
     - Option B : KoproGo = courtier agréé CREG (nécessite agrément)
     - Option C : Partenariat avec courtier existant (ex: Wikipower, Comparateur-Energie.be)
   - **Rémunération courtier** :
     - Commission fixe par switch (ex: 50€/lot)
     - Commission variable (% d'économies réalisées)
     - Transparence obligatoire (CREG impose déclaration des commissions)
   
   ### 3. OCR et extraction de données de factures
   
   - **Problématique** : Les copropriétaires uploadent des factures papier/PDF variées
   - **Solutions a investiguer** :
     - OCR local (Tesseract/PaddleOCR) vs cloud (Google Vision, AWS Textract)
     - Extraction structurée : kWh, prix/kWh, frais fixes, période
     - Validation humaine (syndic vérifie les extractions douteuses)
     - Templates par fournisseur belge (Engie, Luminus, Mega, Eneco, etc.)
   - **GDPR** : Traitement OCR doit rester on-premise ou cloud européen
   
   ### 4. Intégration Linky/smart meters (#109, #133)
   
   - **Alternative a l'upload de factures** :
     - Données de consommation automatiques via compteurs intelligents
     - API Enedis/ORES pour récupération directe (avec autorisation)
     - Granularité horaire vs mensuelle
   - **Avantage** : Pas besoin d'OCR, données structurées, temps réel
   - **Limitation** : Pas tous les bâtiments ont des smart meters
   
   ### 5. Comparateur de fournisseurs belges
   
   - **Sources de données tarifs** :
     - CREG : Tarifs régulés (composante énergie, transport, distribution)
     - VREG (Flandre), CWaPE (Wallonie), BRUGEL (Bruxelles) : Tarifs réseau
     - Monenergie.be, Comparateur-Energie.be : Agrégateurs
   - **Calcul économies** :
     - Simulation basée sur profil de consommation réel du bâtiment
     - Prise en compte heures pleines/creuses (compteur bi-horaire)
     - Tarifs prosumer (si panneaux solaires)
     - Impact TVA (6% rénovation énergétique vs 21% standard)
   - **Actualisation** :
     - Fréquence de mise a jour des tarifs (trimestrielle ?)
     - Web scraping vs API partenaires
     - Cache local pour performance
   
   ### 6. GDPR et k-anonymité
   
   Le module actuel impose k >= 5 participants. Questions ouvertes :
   - Seuil k=5 est-il suffisant pour le RGPD belge (APD) ?
   - Quid des petites copropriétés (< 5 lots) ? Mutualisation inter-bâtiments ?
   - Durée de rétention des données de consommation (90 jours post-campagne suffisant ?)
   - Portabilité des données (art. 20) : export format standard (CSV/JSON) ?
   
   ### 7. Modèle économique pour KoproGo
   
   - **Revenus potentiels** :
     - Commission courtage (si agrément CREG) : 30-80€/lot/switch
     - Abonnement module énergie : 2€/lot/an
     - Commission partenaire courtier : 10-20€/lot/switch
   - **Coûts** :
     - Agrément CREG (si option B) : process administratif + assurance
     - Infrastructure OCR : compute GPU/CPU pour extraction
     - Maintenance tarifs : veille réglementaire continue
   
   ## Points de décision
   
   - [ ] Rôle de KoproGo : outil pour courtier (B2B) ou courtier direct (B2C) ?
   - [ ] Nécessité d'un agrément CREG pour KoproGo ASBL ?
   - [ ] OCR on-premise vs cloud pour extraction factures ?
   - [ ] Intégration smart meters : priorité vs upload factures ?
   - [ ] Mutualisation inter-bâtiments pour petites copropriétés ?
   - [ ] Fournisseurs cibles pour partenariats (Engie, Luminus, Mega, etc.) ?
   - [ ] Modèle de rémunération courtier/plateforme ?
   - [ ] Fréquence des campagnes (annuelle, bi-annuelle) ?
   
   ## Conformité légale belge
   
   - **CREG** : Réglementation des intermédiaires en énergie (Arrêté royal du 29 mars 2012)
   - **GDPR/APD** : Données de consommation = données personnelles (décision APD 2019)
   - **Loi copropriété** : Contrats énergie soumis au vote AG (art. 577-8 §4 CC)
   - **Régulateurs régionaux** : VREG (Flandre), CWaPE (Wallonie), BRUGEL (Bruxelles)
   - **Protection consommateur** : Loi du 29 avril 1999 relative a l'organisation du marché de l'électricité
   
   ## Livrables attendus
   
   - [ ] Étude juridique : agrément CREG nécessaire ou non
   - [ ] Architecture d'orchestration (state machine campagne complète)
   - [ ] POC OCR extraction factures belges (3 fournisseurs minimum)
   - [ ] Maquette comparateur de fournisseurs
   - [ ] Document workflow courtier-copropriété-fournisseur
   - [ ] Analyse économique ROI pour copropriétés type (20, 50, 100 lots)
   
   ## Estimation
   
   - **Complexité** : Très haute (réglementation CREG, OCR, intégrations fournisseurs)
   - **Jalon cible** : Jalon 4 (Automation & Intégrations)
   - **Dépendances** : #110 (module existant), #109/#133 (IoT/Linky), #46 (Votes AG)

.. raw:: html

   </div>

