=========================================================
Marché Belge — Acteurs et Analyse Concurrentielle
=========================================================

:Sources: Étude CREG F1827 (2018), sites acteurs, observations marché 2025
:Date mise à jour: Mars 2026

.. contents:: Table des matières
   :depth: 3
   :local:

1. Acteurs principaux
----------------------

1.1 iChoosr (dominant)
~~~~~~~~~~~~~~~~~~~~~~~

- **Forme** : Filiale belge d'une entreprise néerlandaise
- **Part de marché** : ~50% des achats groupés belges (avec Pricewise : 96% total)
- **Partenaires** : Nombreuses communes belges, Test-Achats, syndicats, mutualités
- **Site** : https://www.ichoosr.com/fr-be
- **Tagline 2025** : "Empowering people, energising society"
- **Modèle économique** : Commission fournisseur (15-30 €/contrat acquisition,
  9-20 €/contrat rétention)
- **Public cible** : Grand public résidentiel (maisons ET appartements)

**Faiblesses iChoosr** :
- Commission opaque payée par les fournisseurs
- Comparaison au prix moyen du marché, pas au contrat réel
- Entreprise néerlandaise (pas de sensibilité locale belge)
- Pas de lien avec la gestion de copropriété

1.2 Pricewise BV (co-dominant)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Forme** : Entreprise néerlandaise
- **Part de marché** : ~46% avec iChoosr = 96% du total
- **Modèle** : Identique à iChoosr
- **Site** : https://www.pricewise.be

**Faiblesses Pricewise** : mêmes que iChoosr + très peu de présence
institutionnelle belge.

1.3 Wikipower SPRL (acteur belge)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Forme** : SPRL belge (Liège, BE 0840.194.796)
- **Zone** : Surtout Bruxelles et Wallonie
- **Particularité** : Seul acteur belge, seul accrédité CREG (B2282, 2021,
  validité expirée 2023)
- **Services élargis** : Au-delà de l'énergie — panneaux PV, isolation, LED
- **Faiblesses** : Accréditation expirée (2023), petite taille, peu transparent
  sur ses commissions (a refusé de communiquer ses contrats à la CREG en 2018)

1.4 Energie.be / Comparateurs en ligne
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Comparateurs de prix (VREG-tool, Mijnenergie.be, Energyscanner.be)
- Rémunérés par clic/lead ou commission fournisseur
- Comparaison en temps réel mais pas d'achat groupé à proprement parler
- Pas de label CREG pour la plupart

2. Analyse des faiblesses systémiques (source CREG F1827)
----------------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 30 45 25

   * - Problème
     - Description
     - Impact consommateur

   * - Commissions opaques
     - 15-30 €/contrat payés par fournisseur, non publiés
     - Biais de sélection : fournisseurs qui paient plus sont avantagés

   * - Économies trompeuses
     - Comparaison au prix moyen marché, pas au contrat réel
     - Économies surestimées de 30-50% vs réalité

   * - Pas de garantie prix bas
     - Fournisseurs discount ne participent pas toujours
     - Le "meilleur prix du marché" peut être ailleurs

   * - Switchers récurrents ciblés
     - Mêmes consommateurs actifs recontactés ad nauseam
     - Pas d'impact sur les "dormants" (cibles initiales)

   * - Concentration 3 acteurs
     - iChoosr + Pricewise + Wikipower depuis 2012
     - Pas de concurrence réelle, pas d'innovation

   * - Label CREG quasi-absent
     - 1 seul accrédité (Wikipower, expiré 2023)
     - Pas de garantie qualité officielle sur le marché

   * - Charte volontaire
     - Aucune obligation légale de respecter la Charte CREG
     - Pas de contrôle des pratiques

3. Opportunité KoproGo
-----------------------

3.1 Positionnement différenciant
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

KoproGo peut se positionner sur des axes NON couverts par les acteurs actuels :

**Axe 1 — Zéro commission = confiance**

  Aucun acteur actuel n'est à zéro commission. iChoosr et Pricewise vivent
  de leurs commissions fournisseur. Wikipower aussi. KoproGo, avec un modèle
  SaaS, peut légitimement afficher "0 commission de la part des fournisseurs"
  et publier ce fait en preuve de transparence (CREG Principe 1).

**Axe 2 — Calcul d'économies honnête**

  Basé sur l'upload de la vraie facture du consommateur (module EnergyBillUpload
  déjà implémenté), KoproGo peut calculer l'économie réelle par rapport au
  contrat actuel du consommateur — exactement ce que la CREG recommande et
  que personne ne fait.

**Axe 3 — Copropriétés = groupe préexistant**

  Les membres d'une copropriété sont un groupe naturellement constitué, avec
  des intérêts communs. La confiance est plus grande (voisins). Le taux de
  participation attendu est plus élevé (60-80% vs 5-15% dans les achats
  groupés publics traditionnels).

**Axe 4 — Extension maisons individuelles**

  Les voisins des copropriétés clientes = canal d'acquisition naturel.
  Un habitant d'une maison individuelle dans le même quartier peut rejoindre
  la campagne énergétique de la copropriété voisine.

**Axe 5 — Label CREG atteignable**

  Avec zéro commission, calcul honnête, RGPD parfait (AES-256-GCM, k-anonymité),
  KoproGo est parfaitement positionné pour obtenir le label CREG. Premiers
  acteurs accrédités depuis la fin du label Wikipower (2023).

3.2 Risques à anticiper
~~~~~~~~~~~~~~~~~~~~~~~~~

- **iChoosr** peut s'attaquer au marché des copropriétés s'il perçoit la menace
- **Réglementation** : La charte volontaire pourrait devenir obligatoire (pression CREG)
- **Fournisseurs** : Certains pourraient refuser de participer sans commission
- **Fraude** : Faux uploads de factures pour manipuler les statistiques de campagne
- **Volatilité prix** : En période de crise énergétique (2022-2023), les économies
  sur les achats groupés peuvent être nulles ou négatives (fournisseurs trop
  prudents sur leurs offres)

4. Benchmarks prix (indicatifs 2025)
-------------------------------------

.. code-block:: text

   PRIX ÉLECTRICITÉ RÉSIDENTIEL BELGE (indicatif, composante fourniture seule)
   ──────────────────────────────────────────────────────────────────────────
   Tarif variable marché             : 0.10 - 0.25 €/kWh (fluctuant)
   Tarif fixe 1 an (marché)          : 0.12 - 0.18 €/kWh (selon conjoncture)
   Achat groupé historique (économie): -10% à -25% vs tarif variable courant
   Tarif social (revenus modestes)   : ~0.09 €/kWh (non négociable)
   ──────────────────────────────────────────────────────────────────────────
   IMPORTANT: Ces prix fluctuent énormément. Référence VREG-tool pour temps réel.

.. code-block:: text

   PRIX GAZ NATUREL RÉSIDENTIEL BELGE (indicatif, composante fourniture seule)
   ──────────────────────────────────────────────────────────────────────────
   Tarif variable marché             : 0.06 - 0.15 €/kWh (fluctuant)
   Tarif fixe 1 an (marché)          : 0.08 - 0.12 €/kWh
   Achat groupé historique (économie): -10% à -20% vs tarif courant
   ──────────────────────────────────────────────────────────────────────────

5. Sources
-----------

- Étude CREG F1827 : https://www.creg.be/sites/default/files/assets/Publications/Studies/F1827FR.pdf
- iChoosr Belgique : https://www.ichoosr.com/fr-be
- VREG-tool (comparateur officiel Flandre) : https://www.vreg.be/nl/wat-kost-energie
- Mijnenergie.be (comparateur privé) : https://www.mijnenergie.be
- Test-Achats énergie : https://www.test-achats.be/energie
- CREG Charte accréditée : https://www.creg.be/fr/particuliers/electricite/comparateurs-de-prix
