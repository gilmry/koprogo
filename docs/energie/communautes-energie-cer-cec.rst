=========================================================
Communautés d'Énergie — CER & CEC en Belgique
=========================================================

:Sources: RED II (2018/2001), Directive 2019/944, Décret wallon 05/05/2022, CWaPE, VREG, BRUGEL
:Date mise à jour: Mars 2026

.. contents:: Table des matières
   :depth: 3
   :local:

1. Cadre européen
------------------

1.1 Deux types de communautés définis par l'UE
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Communauté d'Énergie Renouvelable (CER)**

- **Base légale** : Directive RED II (2018/2001), Art. 22
- **Définition** : Entité juridique dont les actionnaires/membres sont des
  personnes physiques, des PME, des autorités locales ou des organisations
  à but non lucratif, qui possède ou gère des installations d'énergie
  renouvelable, et dont la participation principale est la fourniture de
  bénéfices environnementaux, économiques ou sociaux aux membres (pas la
  maximisation des profits)
- **Activités** : Production, distribution, fourniture, consommation,
  agrégation, stockage, services d'efficacité énergétique

**Communauté d'Énergie Citoyenne (CEC)**

- **Base légale** : Directive 2019/944 (marché intérieur électricité), Art. 2(11)
- **Définition** : Similaire à la CER mais pas limitée aux énergies renouvelables.
  Peut inclure des installations de cogénération, du stockage, de la flexibilité.
- **Actionnaires/membres** : Personnes physiques, PME, autorités locales, non-profit
- **Condition** : Les membres "professionnels" (grandes entreprises, fournisseurs)
  ne peuvent pas contrôler la communauté (gouvernance démocratique requise)

1.2 Principe de "partage virtuel" (clé pour copropriétés)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Le mécanisme central des CER/CEC : les membres partagent l'énergie produite
localement via le réseau de distribution public (pas via une ligne directe).

::

   [Panneaux PV toiture copropriété]
            ↓ (injection réseau)
   [Réseau de distribution Fluvius/Ores/Sibelga]
            ↓ (déduction sur facture)
   [Appartements membres de la CER]

Le gestionnaire de réseau (GRD) comptabilise l'énergie partagée et la déduit
sur la facture de distribution des membres. Tarifs de partage fixés par les
régulateurs régionaux.

2. Transposition belge par région
-----------------------------------

2.1 Wallonie — Décret du 5 mai 2022
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Décret** modifiant diverses dispositions du décret du 19 janvier 2017 relatif
  à la méthodologie tarifaire et diverses dispositions de la loi de 1999
- **Régulateur** : CWaPE (Commission Wallonne pour l'Énergie)
- **GRD** : Ores (réseau de distribution wallon)
- **Facilitateur** : CWaPE a mis en place un facilitateur pour accompagner les
  porteurs de projets CER : https://www.cwape.be/secteur/communautes-partage-energie
- **Rapport CWaPE** : Rapport d'évaluation publié en février 2025 sur la mise en
  place du partage d'énergie, des communautés d'énergie et de l'autoconsommation
- **Arrêté wallon** (2026) : Nouvelles dispositions sur les autorités locales
  pouvant participer aux CER

2.2 Flandre — Régulation VREG
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Régulateur** : Vlaamse Nutsregulator (ex-VREG)
- **GRD** : Fluvius
- Transposition via décret flamand d'électricité
- Partage d'énergie entre voisins (energie deling) : encadré depuis 2023

2.3 Bruxelles — Ordonnance BRUGEL
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Régulateur** : BRUGEL
- **GRD** : Sibelga
- **Ordonnance électricité Bruxelles** : partage d'énergie dans un même
  bâtiment depuis 2024
- Permet aux copropriétés productrices (PV) de partager le surplus entre
  les occupants du bâtiment via Sibelga et compteurs intelligents

3. Différences fondamentales : achat groupé vs CER
----------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 37 38

   * - Critère
     - Achat groupé classique
     - Communauté d'énergie (CER/CEC)

   * - Objectif
     - Négocier un meilleur prix auprès d'un fournisseur externe
     - Partager l'énergie produite localement entre membres

   * - Source de l'énergie
     - Fournisseur tiers (marché)
     - Production propre des membres (PV, éolien, cogénération)

   * - Infrastructure requise
     - Aucune
     - Installation de production partagée (PV collectif, etc.)

   * - Investissement
     - Nul pour les membres
     - Participation à l'installation collective (ou à une coopérative)

   * - Durée d'engagement
     - Contrat annuel (résiliable à tout moment)
     - Long terme (ROI installation sur 10-20 ans)

   * - Cadre légal
     - Loi 1999/1965 + Charte CREG B1614
     - Directives RED II + 2019/944 + décrets régionaux 2022+

   * - Régulateur
     - CREG (fédéral)
     - Régulateurs régionaux (VREG/CWaPE/BRUGEL)

   * - Économies
     - 10-25% sur la composante énergie seulement
     - 30-60% sur la composante distribution + énergie

   * - Qui peut participer
     - Tout ménage (maison individuelle ou appartement)
     - Membres de la CER (copropriété ou quartier avec production locale)

4. Pertinence pour KoproGo
----------------------------

4.1 Module CER pour copropriétés (extension BC8)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

KoproGo est idéalement positionné pour orchestrer des CER au niveau de la
copropriété :

1. La copropriété **installe des panneaux PV** sur la toiture commune (décision AG)
2. Elle crée une **entité CER** (ASBL ou coopérative) avec les copropriétaires membres
3. KoproGo gère :
   - Les quotes-parts de production de chaque membre
   - La comptabilité du partage virtuel
   - La facturation de l'énergie partagée vs le réseau
   - Le rapport annuel AG sur les économies réalisées
4. Intégration avec **LinkyDevice** (IoT) pour suivi consommation en temps réel

4.2 Progression naturelle
~~~~~~~~~~~~~~~~~~~~~~~~~~~

::

   Phase 1 : Achat groupé (énergie externe négociée, contrat collectif)
        ↓
   Phase 2 : CER légère (PV collectif, partage toiture de la copropriété)
        ↓
   Phase 3 : CER étendue (réseau de quartier, multi-bâtiments)
        ↓
   Phase 4 : Communauté d'énergie citoyenne (CEC, inclut stockage, flexibilité)

4.3 Extension aux maisons individuelles dans une CER
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Une CER n'est pas limitée à un seul bâtiment. Elle peut regrouper :

- Des copropriétaires d'un même immeuble
- Des voisins (maisons individuelles dans un même quartier)
- Des PME locales
- La commune/CPAS

**Critère géographique** : La plupart des régions imposent une proximité
géographique (même zone de distribution, même réseau de distribution basse
tension, ou dans un rayon de quelques km).

**Opportunité** : KoproGo peut devenir le **facilitateur technique** de CER
multi-bâtiments incluant des maisons individuelles autour des copropriétés
déjà clientes — canal d'acquisition naturel.

5. Obligations réglementaires et démarches
-------------------------------------------

Pour créer une CER en Belgique :

**Wallonie** :
1. Créer une entité juridique (ASBL, coopérative, société)
2. Enregistrer la CER auprès de la CWaPE
3. Contrat avec Ores (GRD) pour le mécanisme de partage
4. Règlement intérieur (répartition production, gestion excédents)

**Flandre** :
1. Enregistrement auprès du VREG
2. Contrat avec Fluvius
3. Approbation du projet (installation de production)

**Bruxelles** :
1. Déclaration à BRUGEL
2. Contrat avec Sibelga
3. Conformité ordonnance électricité BXL

6. Sources
-----------

- Directive RED II : https://eur-lex.europa.eu/legal-content/FR/TXT/?uri=CELEX:32018L2001
- Directive 2019/944 : https://eur-lex.europa.eu/legal-content/FR/TXT/?uri=CELEX:32019L0944
- CWaPE — communautés d'énergie : https://www.cwape.be/secteur/communautes-partage-energie
- VREG — energiegemeenschappen : https://www.vreg.be/nl/energiegemeenschappen
- BRUGEL — partage d'énergie : https://www.brugel.brussels
- SPW Énergie (Wallonie) : https://energie.wallonie.be
- Leefmilieu Brussel : https://leefmilieu.brussels/professionelen/themas/energie/energiegemeenschap
