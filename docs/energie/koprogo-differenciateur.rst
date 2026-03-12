=========================================================
KoproGo — Orchestrateur Neutre d'Achat Groupé d'Énergie
=========================================================

:Version: 1.0
:Date: Mars 2026
:Statut: Spécification stratégique

.. contents:: Table des matières
   :depth: 3
   :local:

1. Vision stratégique
----------------------

KoproGo se positionne comme **orchestrateur neutre et transparent** d'achat
groupé d'énergie, sans commission des fournisseurs, pour les copropriétés
ET les maisons individuelles belges.

**Mission** :

  Permettre à chaque consommateur belge d'accéder aux meilleures conditions
  de fourniture d'énergie, avec une transparence totale sur le processus de
  sélection et des économies calculées honnêtement sur la base de sa vraie
  facture — sans jamais recevoir de commission d'un fournisseur.

**Modèle économique** : SaaS uniquement — revenus = abonnements copropriétés
+ abonnements maisons individuelles. Les fournisseurs d'énergie ne paient rien
à KoproGo et n'ont aucun avantage à le faire.

2. Principes non-négociables
------------------------------

2.1 Zéro commission fournisseur
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   DÉCLARATION PUBLIQUE (page "Comment ça marche")
   ─────────────────────────────────────────────────
   KoproGo ne reçoit JAMAIS de commission de la part des fournisseurs
   d'énergie. Aucun fournisseur ne paie pour apparaître dans nos campagnes
   ou être sélectionné. Notre seule source de revenus : l'abonnement mensuel
   des copropriétés et des membres particuliers.

   Ce modèle garantit notre indépendance totale et élimine tout conflit
   d'intérêts dans la sélection du fournisseur.
   ─────────────────────────────────────────────────

**Implications techniques** :

- Pas de système de "sponsor" ou de "mise en avant payante" dans l'interface
- Algorithme de sélection publié (open-source ou publié en clair)
- Aucun contrat commercial avec les fournisseurs (ils répondent aux appels
  d'offres librement)

2.2 Calcul d'économies honnête
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Contrairement aux acteurs actuels qui comparent au "prix moyen du marché",
KoproGo calcule les économies réelles par rapport au **contrat actuel** de
chaque participant :

.. code-block:: text

   Économie réelle = (Prix actuel $/kWh × consommation annuelle kWh)
                   - (Prix offre groupée $/kWh × consommation annuelle kWh)

   Source prix actuel : upload PDF facture → OCR → extraction prix/kWh exact
   Source consommation : LinkyDevice (IoT) OU upload facture

**Conformité CREG** : Principe 3 de la Charte B1614 — exactement ce que
la CREG demande et que personne ne fait actuellement.

2.3 Transparence du processus de sélection
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Cahier des charges de l'appel d'offres publié sur la plateforme
- Critères de sélection pondérés publiés (prix énergie, % renouvelable,
  contrat fixe/variable, service client, licence régionale)
- Toutes les offres reçues affichées (pas seulement la gagnante)
- Procès-verbal de sélection généré et archivé (GDPR Art. 30)

2.4 Tous les fournisseurs agréés bienvenus
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Invitation ouverte à tout fournisseur titulaire d'une licence VREG/CWaPE/BRUGEL
- Vérification automatique de la licence avant invitation
- Aucun accord commercial préalable requis

3. Architecture du module étendu
----------------------------------

3.1 Extension BC8 — Nouveaux sous-modules
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   BC8 Energy & IoT (étendu pour release 0.6.0)
   ─────────────────────────────────────────────
   Existant:
     EnergyCampaign      ← achat groupé copropriété
     EnergyBillUpload    ← upload facture + chiffrement AES-256-GCM
     IoTReading          ← données compteurs connectés
     LinkyDevice         ← intégration Linky/Ores

   Nouveau:
     OpenCampaign        ← campagne ouverte aux maisons individuelles
     IndividualMember    ← membre particulier (hors copropriété)
     EnerCommunity       ← communauté d'énergie (CER/CEC)
     OfferComparison     ← comparaison multi-fournisseurs avec score pondéré
     SavingsCalculator   ← calcul économies honnête (vs contrat réel)
     ProviderRegistry    ← registre fournisseurs agréés (VREG/CWaPE/BRUGEL)

3.2 Workflow étendu
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   CAMPAGNE OUVERTE (copropriétés + maisons individuelles)
   ────────────────────────────────────────────────────────
   1. Création campagne (syndic ou admin KoproGo)
         |
         ├── Zone géographique (code postal, rayon km)
         ├── Énergie ciblée (électricité / gaz / les deux)
         ├── Ouvert à : copropriétés seules OU + maisons individuelles
         └── Dates : début inscription → fin inscription → délibération

   2. Inscription membres
         |
         ├── Copropriétaires (via leur espace KoproGo habituel)
         ├── Maisons individuelles (via landing page publique ou lien)
         └── Consentement RGPD + upload facture OU données Linky

   3. Seuil atteint → Appel d'offres
         |
         ├── Calcul consommation agrégée (k-anonymité ≥ 5)
         ├── Génération cahier des charges PDF
         └── Envoi aux fournisseurs agréés (zone + licence vérifiée)

   4. Réception et affichage des offres
         |
         ├── Toutes les offres affichées publiquement
         ├── Scoring automatique : prix 50% / vert 25% / service 15% / fixe 10%
         └── Aucune offre cachée ou favorisée

   5. Vote (AG copropriété) ou consultation (membres individuels)
         |
         ├── Copropriétés : résolution AG (module Meeting existant)
         └── Membres individuels : vote dans l'application (module Poll)

   6. Sélection et notification
         |
         ├── Email individuel à chaque membre avec sa vraie économie calculée
         ├── Droit de rétractation 14 jours (Code droit économique)
         └── Contact direct fournisseur → contrat individuel

   7. Rapport final
         ├── Procès-verbal de sélection (archivé GDPR Art. 30)
         └── Tableau d'économies agrégées (anonymisé, k-anonymité ≥ 5)

4. Extension maisons individuelles
------------------------------------

4.1 Modèle d'accès
~~~~~~~~~~~~~~~~~~~~

**Option A — Via copropriété cliente (recommandé)**

  La copropriété cliente de KoproGo lance une campagne et "invite ses voisins"
  (maisons individuelles du quartier). KoproGo fournit un lien d'inscription
  public sans authentification complète.

  Inscription maison individuelle :
  - Email + code postal + consentement RGPD
  - Upload facture d'énergie OU saisie manuelle consommation annuelle (kWh)
  - Compte "lite" (pas accès gestion copropriété, seulement module énergie)

**Option B — Campagne publique directe**

  KoproGo publie des campagnes ouvertes par zone géographique sur une page
  publique. Tout ménage belge peut s'inscrire sans lien avec une copropriété.
  Modèle à activer après validation du modèle copropriété.

4.2 Tarification membres individuels
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Gratuit** pour les membres d'une copropriété cliente de KoproGo
- **Abonnement "énergie solo"** pour les maisons individuelles sans lien avec
  une copropriété : 1-2 €/mois (à valider)
- **Aucune commission fournisseur** dans tous les cas

4.3 Impact sur la masse négociante
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Chaque copropriété cliente = ~20 appartements. Si 3-5 maisons individuelles
voisines rejoignent chaque campagne :

::

   100 copropriétés × 20 unités = 2,000 appartements
   + 100 × 4 maisons individuelles = 400 maisons individuelles
   ───────────────────────────────────────────────────────────
   Total masse négociante : 2,400 ménages

   Consommation : 2,400 × 3,500 kWh/an = 8,400,000 kWh/an
   Économie 15% : 8,400,000 × 0.03 €/kWh = 252,000 €/an pour les membres

Plus la masse est grande, plus le pouvoir de négociation est élevé et
plus les économies sont importantes pour chaque membre.

5. Conformité et certifications visées
----------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 45 30

   * - Certification/Conformité
     - Description
     - Calendrier

   * - **Charte CREG B1614**
     - Conformité aux 4 principes (clarté, intuitivité, honnêteté, RGPD)
     - Jalon 3 (implémentation)

   * - **Label CREG (accréditation)**
     - Demande formelle à CREG après implémentation
     - Jalon 3+ (3-6 mois après)

   * - **RGPD complet**
     - AES-256-GCM, k-anonymité ≥ 5, rétention 90j, Art. 15-17-21
     - Déjà implémenté (release 0.5.0)

   * - **Droit de rétractation 14j**
     - Affichage obligatoire, mécanisme d'annulation facile
     - Jalon 3

   * - **Licences fournisseurs vérifiées**
     - API ou contrôle manuel VREG/CWaPE/BRUGEL avant invitation
     - Jalon 3

   * - **Intermédiaire légalement défini**
     - Conformité loi 09/05/2019 (définition intermédiaire achat groupé)
     - Déjà conforme (pas de contrat direct, facilitation seulement)

6. Impact écologique et social
--------------------------------

6.1 Green Score comme critère de sélection pondéré
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Le module actuel (green_score 0/5/10 selon % renouvelable) est intégré
dans l'algorithme de sélection avec une pondération de **25%** (vs prix 50%).

**Principe** : Une offre 5% moins chère mais 0% renouvelable ne doit pas
automatiquement gagner face à une offre légèrement plus chère mais 100% verte.
Les membres votent sur les pondérations en début de campagne.

6.2 Lien avec fonds de solidarité climatique (Jalon 6 MCP)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Économies CO2 calculées (kWh vert × facteur CO2)
- 1% des économies financières → fonds de solidarité climatique (optionnel,
  membre opt-in)
- Crédits carbone → financement panneaux solaires pour copropriétés à faibles
  revenus (lien avec module CER Phase 2)
