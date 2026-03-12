=========================================================
Extension aux Maisons Individuelles — Module Énergie
=========================================================

:Version: 1.0
:Date: Mars 2026

.. contents:: Table des matières
   :depth: 3
   :local:

1. Contexte légal : pas de différence avec les appartements
------------------------------------------------------------

Il n'existe **aucune distinction légale** entre un habitant d'une maison
individuelle et un copropriétaire d'un appartement dans le cadre des
achats groupés d'énergie belges.

Les lois du 29/04/1999 (électricité) et du 12/04/1965 (gaz) s'adressent
à tous les **utilisateurs finals résidentiels** indistinctement.

La définition d'intermédiaire en achats groupés (loi 09/05/2019) couvre
tout service organisé pour des "utilisateurs finals" — ce qui inclut
tout ménage, copropriété ou maison individuelle.

**Conclusion** : KoproGo peut légalement inclure les maisons individuelles
dans ses campagnes d'achat groupé d'énergie sans modification du cadre légal.

2. Avantages stratégiques
---------------------------

2.1 Canal d'acquisition naturel
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Les habitants du même quartier que les copropriétés clientes KoproGo sont
le canal d'acquisition le plus efficace :

- Confiance par la proximité (voisinage)
- Intérêt partagé (même réseau de distribution, même GRD)
- Coût d'acquisition proche de zéro (bouche-à-oreille)

2.2 Pouvoir négociant augmenté
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Plus la masse de consommation agrégée est élevée, plus les fournisseurs
font des efforts :

.. code-block:: text

   Masse < 500 unités  : économie attendue 5-10%
   Masse 500-2,000     : économie attendue 10-20%
   Masse > 2,000       : économie attendue 20-30%

Inclure les maisons individuelles permet d'atteindre plus rapidement
les seuils de masse critique.

2.3 Montée en charge progressive
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

1. Copropriétés KoproGo (base installée)
2. Voisins directs des copropriétés (invités via lien de campagne)
3. Quartiers entiers (lien avec communes, intercommunales)
4. Communautés d'énergie multi-bâtiments (CER étendue)

3. Modèle opérationnel
------------------------

3.1 Inscription simplifiée
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Pour une maison individuelle, le parcours d'inscription est réduit au minimum :

.. code-block:: text

   PARCOURS INSCRIPTION MAISON INDIVIDUELLE
   ─────────────────────────────────────────
   1. Lien d'invitation (email ou QR code affiché par copropriété)
   2. Page landing : description de la campagne, zone géographique
   3. Formulaire : email + code postal + consentement RGPD
   4. Upload facture PDF OU saisie manuelle (kWh/an + €/kWh actuel)
   5. Confirmation email
   6. Pas de création de compte obligatoire pour la phase de collecte

3.2 Compte "Énergie Solo"
~~~~~~~~~~~~~~~~~~~~~~~~~~

Une fois inscrit à une campagne, l'habitant individuel accède à un
espace minimal :

- Voir les offres de sa campagne
- Voir son économie calculée individuellement
- Voter pour l'offre (si ouvert aux votes individuels)
- Exercer son droit de rétractation (14 jours)
- Accéder à ses données personnelles (RGPD Art. 15)

Pas d'accès aux modules de gestion de copropriété (réservé aux membres
d'une copropriété cliente).

3.3 Isolation des données
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Les données personnelles des membres individuels sont **strictement
séparées** des données des copropriétés :

- Partitionning PostgreSQL : table ``individual_members`` distincte de ``owners``
- Chiffrement AES-256-GCM sur les données de consommation
- k-anonymité ≥ 5 pour les statistiques de campagne
- Rétention limitée : 90 jours après fin de campagne

4. Conformité RGPD spécifique
-------------------------------

Les maisons individuelles ne sont pas liées à une organisation KoproGo
existante. Le traitement de leurs données est basé sur le **consentement
explicite** (RGPD Art. 6(1)(a)) :

- Consentement recueilli lors de l'inscription
- Granularité du consentement :
  - Upload facture : consentement pour extraction des données de consommation
  - Partage avec fournisseurs : consentement séparé (opt-in)
  - Newsletter / futures campagnes : consentement séparé (opt-in)
- Retrait du consentement à tout moment (lien dans chaque email)
- Droit à l'effacement exercé dans les 30 jours (RGPD Art. 17)

5. Impact sur le modèle tarifaire KoproGo
------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 30 35 35

   * - Profil
     - Accès module énergie
     - Tarification

   * - Copropriétaire (membre copro cliente)
     - Complet (toutes campagnes de sa copropriété)
     - Inclus dans l'abonnement copropriété

   * - Syndic / Admin copropriété
     - Création et gestion des campagnes
     - Inclus dans l'abonnement copropriété

   * - Maison individuelle (invité par copro)
     - Inscription à la campagne de la copropriété invitante
     - Gratuit (la copropriété "sponsorise" l'accès)

   * - Maison individuelle (inscription directe)
     - Campagnes publiques ouvertes dans sa zone
     - Abonnement "énergie solo" : 1-2 €/mois (à valider)

   * - PME locale (extension future)
     - Campagnes "professionnel" séparées (tarifs PRO vs résidentiel)
     - Abonnement "énergie pro" : 5-10 €/mois (à valider)

6. Feuille de route
--------------------

**Jalon 3 (implémentation v0.6.0)** :

- Extension entité ``EnergyCampaign`` : champ ``audience_type``
  (CoProprietiesOnly / OpenToIndividuals / Public)
- Nouvelle entité ``IndividualMember`` (email, postal_code, consent, campaign_id)
- Endpoint public : ``POST /energy-campaigns/:id/join-as-individual``
- Calcul économies individuelles vs upload facture (module EnergyBillUpload)

**Jalon 4 (scalabilité)** :

- Page landing publique par campagne (SEO, partage réseaux sociaux)
- Partenariats communes / CPAS / SLSP (Sociétés de Logement de Service Public)
- Campagnes multi-zones (plusieurs codes postaux)

**Jalon 5 (expansion)** :

- Extension PME (tarifs professionnels distincts)
- CER multi-bâtiments (maisons individuelles + copropriétés dans une CER)
- API publique pour intégration par des communes
