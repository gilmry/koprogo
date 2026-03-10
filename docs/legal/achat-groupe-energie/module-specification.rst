=========================================================
KoproGo — Module d'Achat Groupé d'Énergie
=========================================================

Vision
======

KoproGo propose un module d'\ **orchestration neutre** d'achats groupés d'énergie
(électricité + gaz) qui permet à ses membres (copropriétés ET maisons individuelles)
de se regrouper pour négocier collectivement un meilleur tarif auprès des fournisseurs
d'énergie belges.

**Différenciateurs clés :**

- **Neutre** : KoproGo ne perçoit aucune commission du fournisseur
- **Multi-type** : copropriétés + maisons individuelles dans le même groupe
- **Intégré** : le syndic gère l'énergie des PC ET les copropriétaires individuels participent
- **Transparent** : processus d'enchère vérifiable, cahier des charges public
- **Open-source** : le code d'orchestration est auditable


1. Cadre légal
==============

1.1 Marché libéralisé de l'énergie en Belgique
-----------------------------------------------

============================================  ==========================================================
Source                                        Contenu
============================================  ==========================================================
Loi du 29/04/1999 (électricité)               Organisation du marché libéralisé, rôle de la CREG
Loi du 29/04/1999 (gaz)                       Idem pour le gaz naturel
Suppression indemnité de rupture (13/09/2012)  Changement de fournisseur à tout moment sans frais
Délai de préavis                               Généralement 1 mois pour changer de fournisseur
Droit de rétractation                          14 jours pour annuler un contrat conclu à distance
============================================  ==========================================================

1.2 CREG — Charte de bonnes pratiques
--------------------------------------

La CREG a établi une « Charte pour une fourniture efficace d'informations par les
comparateurs en ligne » (version 2018), étendue aux intermédiaires en achats groupés.

**Principes de la charte :**

- Objectivité et indépendance vis-à-vis des fournisseurs
- Qualité et exhaustivité des informations transmises au consommateur
- Transparence sur le processus de sélection
- Protection des données personnelles des participants

**Label de qualité CREG :**

- Accréditation volontaire, valable 2 ans renouvelables
- Seul Wikipower l'a obtenu pour les achats groupés (oct. 2021, décision B2282)
- KoproGo pourrait demander ce label → crédibilité et différenciation

1.3 Régulateurs régionaux
--------------------------

==========  ===========  ======================================================
Région      Régulateur   Compétence
==========  ===========  ======================================================
Bruxelles   BRUGEL       Autorisation des fournisseurs, tarifs de distribution
Wallonie    CWaPE        Idem pour la Wallonie
Flandre     VREG         Idem pour la Flandre
==========  ===========  ======================================================

.. important::

   Les tarifs de transport (CREG) et de distribution (régionaux) ne sont **PAS** négociables.
   Seule la composante **prix de l'énergie** est négociable dans un achat groupé.

1.4 RGPD
---------

- Les données de consommation sont des données personnelles
- Base légale : consentement explicite du participant
- Minimisation : seules les données nécessaires à l'établissement de l'offre
- Droit de retrait à tout moment

1.5 Partage d'énergie en immeuble (Bruxelles)
----------------------------------------------

Depuis l'ordonnance électricité de Bruxelles, le « partage d'énergie dans un même
bâtiment » permet aux copropriétés productrices (panneaux PV) de partager le surplus
entre occupants via Sibelga et des compteurs intelligents.
KoproGo pourrait intégrer ce use case à terme.


2. État de l'art — Comment fonctionne un achat groupé
======================================================

2.1 Workflow classique (Wikipower, Mr. Énergie, etc.)
------------------------------------------------------

**Phase 1 — Inscription** (2-4 semaines)

Le participant s'inscrit gratuitement, sans engagement. Données collectées : type
d'énergie, consommation annuelle, région, fournisseur actuel, code EAN.

**Phase 2 — Enchère** (1-2 semaines)

L'organisateur rédige un cahier des charges (critères : vert, fixe/variable, service
client), lance un appel d'offres, et sélectionne l'offre gagnante. Chez Wikipower,
le processus est supervisé par un huissier de justice.

**Phase 3 — Proposition** (2-4 semaines)

Chaque participant reçoit une offre personnalisée basée sur sa consommation déclarée
vs son tarif actuel. Il accepte ou refuse librement.

**Phase 4 — Switching** (automatique, ~30 jours)

Si acceptation, le contrat se signe directement avec le fournisseur gagnant.
Le nouveau fournisseur prend en charge la résiliation de l'ancien contrat.
Droit de rétractation de 14 jours.

2.2 Modèle économique des organisateurs existants
--------------------------------------------------

================  ===========================================================
Organisateur      Modèle
================  ===========================================================
Wikipower         Commission payée par le fournisseur (par contrat souscrit)
Mr. Énergie       Idem — tarifs négociés exclusifs
Communes          Partenariat avec un organisateur (souvent Wikipower)
Test-Achats       Commission fournisseur — réservé aux membres
================  ===========================================================

.. warning::

   Le participant ne paie rien directement, mais la commission de l'organisateur est
   intégrée au prix du kWh proposé par le fournisseur. C'est un coût caché.

2.3 Limites identifiées
-------------------------

1. **Pas tous les fournisseurs participent** — les grands acteurs sont souvent réticents
2. **Commission cachée** — intégrée au prix du kWh
3. **Délai entre enchère et souscription** — les prix peuvent évoluer
4. **Calcul basé sur un profil type** — pas sur la consommation réelle
5. **Pas de suivi post-souscription** — le participant est livré à lui-même


3. Module KoproGo — Architecture
=================================

3.1 Positionnement
-------------------

::

   Modèle classique              Modèle KoproGo
   ────────────────              ───────────────
   Organisateur = intermédiaire   KoproGo = plateforme neutre
   Commission du fournisseur      Zéro commission (modèle SaaS)
   Processus opaque               Processus auditable (open-source)
   Participants isolés            Communauté de copropriétés
   One-shot                       Récurrent (renégociation annuelle)

3.2 Rôles dans le module
--------------------------

==========================  ====================================================  ====================
Rôle                        Description                                           Interface
==========================  ====================================================  ====================
**Membre KoproGo**          Copropriétaire individuel, propriétaire maison,       App membre
                            ou syndic (pour les PC)
**Syndic**                  Gère la participation de l'ACP (énergie PC)           Backoffice syndic
**Admin KoproGo**           Orchestre la campagne d'achat groupé                  Backoffice admin
**Courtier énergie**        Rédige le cahier des charges, supervise l'enchère     Backoffice courtier
**Fournisseur d'énergie**   Soumet ses offres en réponse à l'appel d'offres      Portail fournisseur
==========================  ====================================================  ====================

3.3 Workflow détaillé
----------------------

Phase 1 — Campagne & inscription (4-6 semaines)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Admin KoproGo :**

- Crée une campagne (nom, région, dates, critères)
- Définit le cahier des charges (avec courtier optionnel)
- Publie la campagne sur la plateforme

**Syndic :**

- Inscrit l'ACP pour les compteurs parties communes
- Optionnel : décision AG si engagement collectif (PC)
- Communique la campagne aux copropriétaires

**Membre (copropriétaire / maison) :**

- S'inscrit via l'app (gratuit, sans engagement)
- Renseigne : EAN, type compteur, consommation annuelle,
  fournisseur actuel, type souhaité (fixe/variable/vert)
- Upload optionnel : dernier décompte annuel (OCR)

**KoproGo :**

- Agrège les volumes (MWh total, nombre de points)
- Dashboard temps réel : participants, volume, régions
- Validation des données (EAN, cohérence consommation)

Phase 2 — Appel d'offres & enchère (2-3 semaines)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Admin / Courtier :**

- Rédige le cahier des charges :

  - Volume agrégé (MWh élec + gaz)
  - Nombre de points de fourniture
  - Répartition géographique (BXL / WAL / VL)
  - Critères obligatoires (énergie verte BE, tarif fixe ou variable, service client FR/NL)
  - Critères de scoring pondérés

- Invite les fournisseurs agréés via le portail
- Fixe une date limite de remise des offres

**Fournisseur (via portail) :**

- Reçoit le cahier des charges anonymisé (pas de noms de participants)
- Soumet son offre (prix/kWh, conditions, durée)
- Peut poser des questions (Q&A public, anonyme)

**KoproGo :**

- Horodatage des offres (blockchain-ready ou huissier)
- Scoring automatique selon les critères pondérés
- Comparaison avec le CREG Scan (vérification marché)

Phase 3 — Sélection & offre personnalisée (1-2 semaines)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Admin / Courtier :**

- Valide le classement des offres
- Publie le résultat (fournisseur gagnant + conditions)
- Optionnel : huissier certifie le processus

**KoproGo :**

- Génère une offre personnalisée par membre :

  - Simulation basée sur **sa** consommation réelle
  - Comparaison avec son contrat actuel
  - Économie estimée (annuelle, en €)
  - Détail : composante énergie vs transport vs taxes

- Envoie l'offre par email + notification in-app

**Membre :**

- Consulte son offre personnalisée
- **ACCEPTE** ou **REFUSE** (bouton dans l'app)
- Si accepte : signature électronique du contrat directement avec le fournisseur
- Délai de rétractation : 14 jours

**Syndic (pour les PC) :**

- Accepte/refuse pour les compteurs parties communes
- Si montant > seuil de mise en concurrence : nécessite décision AG préalable

Phase 4 — Switching & suivi (30 jours)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Fournisseur gagnant :**

- Prend en charge le switch (résiliation ancien fournisseur)
- Envoie la confirmation de contrat au membre

**KoproGo :**

- Suivi du statut de chaque switch (en cours / actif / erreur)
- Tableau de bord campagne : taux d'acceptation, volume
- Rappel du droit de rétractation (J+14)

Phase 5 — Post-campagne & suivi annuel (continu)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**KoproGo :**

- Monitoring : alerte si le tarif du marché devient significativement meilleur
- Rappel annuel : « votre contrat arrive à échéance, nouveau groupe ? »
- Statistiques communautaires : économies réalisées
- Feedback des membres (satisfaction fournisseur)


4. Spécificités copropriété
============================

4.1 Deux types de participation
--------------------------------

====================  ==========================  ====================================  ====================
Type                  Compteur                    Qui décide                            Contrat avec
====================  ==========================  ====================================  ====================
Parties communes      Compteur commun de l'ACP    Syndic (+ AG si > seuil concurrence)  ACP (personne morale)
Lots privatifs        Compteur individuel          Chaque copropriétaire                 Le copropriétaire
====================  ==========================  ====================================  ====================

4.2 Lien avec l'AG
--------------------

Si la copropriété veut changer de fournisseur pour les parties communes :

- **Art. 3.89 §5 12°** : le syndic soumet un rapport d'évaluation des contrats de fournitures
  régulières à l'AG annuelle. L'énergie en fait partie.
- **Art. 3.88 §1 1°c** : l'AG fixe le montant de mise en concurrence. Si le contrat énergie
  dépasse ce seuil, il faut passer par une mise en concurrence formelle.
- **Majorité requise** : majorité absolue pour un simple changement de fournisseur.

→ KoproGo inscrit automatiquement le point « changement de fournisseur énergie PC »
à l'OdJ de l'AG, avec le comparatif de l'offre groupée vs le contrat actuel.

4.3 Locataires
---------------

Les locataires avec compteur individuel peuvent aussi participer en tant que membres
KoproGo (rôle « occupant énergie »). Ils s'inscrivent directement, sans passer par le syndic.


5. Interfaces
=============

5.1 App Membre
---------------

::

   [Mes campagnes]
     └─ Campagne "Énergie Verte BXL 2026"
         ├─ Statut : En attente d'offre
         ├─ Mon inscription : ✅ confirmée
         ├─ Ma consommation : 3 500 kWh élec / 12 000 kWh gaz
         ├─ Mon fournisseur actuel : ENGIE — Easy Fix
         └─ [Voir l'offre quand disponible]

   [Mon offre personnalisée]
     ├─ Fournisseur gagnant : Mega — Super Green
     ├─ Prix/kWh élec : 0,0823 € (vs 0,1105 € actuel)
     ├─ Économie estimée : 287 €/an
     ├─ Type : fixe 1 an, 100% vert Belgique
     ├─ [ACCEPTER] [REFUSER]
     └─ Détail : énergie | transport | taxes | total

5.2 Backoffice Syndic
-----------------------

::

   [Achat groupé énergie]
     ├─ Compteurs parties communes inscrits : 2 (élec + gaz)
     ├─ Copropriétaires informés : 24/30
     ├─ Copropriétaires inscrits individuellement : 18
     ├─ [Inscrire à l'OdJ de la prochaine AG]
     └─ [Voir l'offre pour les PC]

5.3 Portail Fournisseur
-------------------------

::

   [Appel d'offres KoproGo #2026-Q3-BXL]
     ├─ Volume : 850 MWh élec / 2 100 MWh gaz
     ├─ Points de fourniture : 342
     ├─ Régions : BXL (78%), WAL (22%)
     ├─ Cahier des charges : [PDF]
     ├─ Date limite : 15/09/2026
     ├─ Q&A : [3 questions posées]
     └─ [SOUMETTRE MON OFFRE]

5.4 Backoffice Courtier
-------------------------

::

   [Campagne "Énergie Verte BXL 2026"]
     ├─ Phase actuelle : ENCHÈRE
     ├─ Offres reçues : 4/8 fournisseurs invités
     ├─ Classement provisoire (scoring pondéré) :
     │   1. Mega — 87/100
     │   2. Octa+ — 82/100
     │   3. Eneco — 79/100
     │   4. Bolt — 74/100
     ├─ Vérification CREG Scan : ✅ offre 1 < marché
     └─ [VALIDER LE CLASSEMENT] [DEMANDER BEST & FINAL]


6. Outils MCP (extension du serveur existant)
==============================================

``energie_campagne_list``
   Liste des campagnes d'achat groupé actives.

``energie_inscrire``
   Inscrire un membre ou une ACP à une campagne.

``energie_offre_personnalisee``
   Récupérer l'offre personnalisée d'un membre pour une campagne donnée.

``energie_comparer_tarif``
   Comparer le tarif actuel d'un membre avec l'offre groupée et le marché (CREG Scan).

``energie_ag_point``
   Générer le point OdJ pour le changement de fournisseur énergie PC, avec le comparatif.


7. Règles métier — Énergie
===========================

EN01 — Inscription sans engagement
   **Source** : Pratique du marché + droit de la consommation.
   Le participant s'inscrit gratuitement et sans engagement. Aucun contrat n'est conclu
   avec KoproGo au titre de la fourniture d'énergie.

EN02 — Données minimales collectées
   **Source** : RGPD art. 5 (minimisation).
   Seules les données nécessaires à l'établissement de l'offre sont collectées : EAN,
   type compteur, consommation annuelle, fournisseur actuel, région.

EN03 — Consentement explicite
   **Source** : RGPD art. 6 et 7.
   Le participant donne son consentement explicite pour le traitement de ses données
   de consommation dans le cadre de l'achat groupé.

EN04 — Indépendance vis-à-vis des fournisseurs
   **Source** : Charte CREG 2018.
   KoproGo ne perçoit aucune commission du fournisseur gagnant. L'offre présentée
   est le prix réel, sans surcoût caché.

EN05 — Transparence du processus d'enchère
   **Source** : Charte CREG 2018.
   Le cahier des charges est public, les critères de scoring sont publiés, le classement
   est vérifiable (huissier ou horodatage certifié).

EN06 — Contrat direct fournisseur-membre
   **Source** : Pratique du marché.
   Aucun contrat de fourniture d'énergie n'est signé entre le participant et KoproGo.
   Le contrat se conclut directement entre le membre et le fournisseur gagnant.

EN07 — Droit de rétractation 14 jours
   **Source** : Code de droit économique, Livre VI (vente à distance).
   Le membre dispose de 14 jours pour annuler le contrat conclu à distance sans frais.

EN08 — Pas d'indemnité de rupture
   **Source** : Loi du 13/09/2012.
   Le changement de fournisseur se fait sans indemnité de rupture, quel que soit
   le type de contrat en cours (fixe ou variable, déterminé ou indéterminé).

EN09 — Lien avec l'AG pour les parties communes
   **Source** : Art. 3.89 §5 12° + Art. 3.88 §1 1°c.
   Le syndic soumet l'évaluation des contrats de fournitures à l'AG annuelle.
   Si le montant dépasse le seuil de mise en concurrence fixé par l'AG, le changement
   de fournisseur nécessite une décision AG (majorité absolue).

EN10 — Composantes du prix
   **Source** : Structure tarifaire réglementée.
   Seule la composante « prix de l'énergie » est négociable. Les tarifs de transport
   (approuvés par la CREG) et de distribution (régionaux) ne sont pas négociables.
   Les taxes et surcharges sont fixées par les autorités.

EN11 — Fournisseurs agréés uniquement
   **Source** : Régulateurs régionaux (BRUGEL, CWaPE, VREG).
   Seuls les fournisseurs disposant d'une licence de fourniture délivrée par le
   régulateur régional compétent peuvent participer à l'appel d'offres.

EN12 — Label CREG (objectif)
   **Source** : Charte CREG pour les intermédiaires en achats groupés.
   KoproGo vise l'obtention du label de qualité CREG pour garantir au consommateur
   l'objectivité et la qualité des informations transmises.


8. Roadmap
==========

Phase 1 — MVP (3-6 mois)
--------------------------

- Inscription des membres + collecte des données de consommation
- Cahier des charges standard (template)
- Portail fournisseur minimal (soumission d'offre par formulaire)
- Génération d'offre personnalisée (simulation)
- Accept/Refuse dans l'app

Phase 2 — Automatisation (6-12 mois)
--------------------------------------

- OCR du décompte annuel pour extraction automatique des données
- Intégration API CREG Scan pour comparaison marché
- Scoring automatique des offres
- Intégration OdJ de l'AG pour les parties communes
- Dashboard communautaire (économies réalisées)

Phase 3 — Scale (12-18 mois)
------------------------------

- Demande du label de qualité CREG
- Enchère en ligne avec horodatage certifié
- Multi-énergie : mazout, pellets, panneaux PV
- Partage d'énergie en immeuble (Bruxelles, via Sibelga)
- API fournisseur pour switch automatisé


9. Modèle économique
=====================

KoproGo ne perçoit **aucune commission** du fournisseur.

1. **Inclus dans l'abonnement** KoproGo pour les copropriétés (parties communes)
2. **Gratuit** pour les copropriétaires individuels et les maisons → canal d'acquisition

Le vrai ROI :

- **Acquisition** : propriétaires de maisons s'inscrivent pour l'achat groupé → découvrent KoproGo
- **Rétention** : les copropriétés restent car le module énergie génère de la valeur récurrente
- **Volume** : plus de membres = meilleur pouvoir de négociation = meilleures offres (cercle vertueux)
- **Données** : la consommation agrégée ouvre la voie à des services (audit énergétique, rénovation groupée)


10. Références
==============

=========================================  ============================================================
Source                                     Contenu
=========================================  ============================================================
CREG — Label de qualité                    Charte et conditions d'accréditation
CREG — Décision B2282 (Wikipower)          Dossier d'accréditation type
Wikipower — FAQ                            Workflow détaillé d'un achat groupé
Sibelga — Partage d'énergie                Partage en immeuble (BXL)
Loi 29/04/1999                             Organisation du marché électricité + gaz
Loi 13/09/2012                             Suppression indemnité de rupture
Eneco — Analyse critique                   Points d'attention pour le consommateur
ENGIE — Guide                              Avantages et inconvénients achats groupés
Mega — Fonctionnement                      Explication du workflow côté fournisseur
=========================================  ============================================================
