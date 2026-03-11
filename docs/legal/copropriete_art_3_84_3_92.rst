Droit Belge de la Copropriete — Art. 3.84 a 3.94 Code Civil
============================================================

.. contents:: Table des matieres
   :local:
   :depth: 2

Introduction
------------

Le droit belge de la copropriete est regi par le **Livre 3, Titre 4** du Code Civil belge
(anciennement Art. 577-2 a 577-14). La reforme de 2018 (Loi du 18 juin 2018) a recodifie
ces dispositions sous les articles 3.84 a 3.100.

KoproGo implemente les dispositions les plus critiques pour un logiciel de gestion
de copropriete. Ce document presente les extraits de loi in extenso et leur alignement
avec le code actuel.

Art. 3.84 — Disposition generale
---------------------------------

**Texte (resume)** : Chaque lot en copropriete comprend une partie privative et une
quote-part dans les parties communes. Les quotes-parts sont determinees par les statuts
(acte de base) en fonction de la valeur respective des parties privatives.

**Alignement KoproGo** :

- Entite ``Unit`` avec ``area``, ``floor``, ``unit_number``
- Relation ``UnitOwner`` avec ``ownership_percentage`` (0.0 a 1.0)
- Trigger PostgreSQL ``validate_unit_ownership_total()`` verifiant que le total = 100% (±0.01%)
- Fichiers : ``backend/src/domain/entities/unit_owner.rs``, migration ``20251120230000``

**Statut** : CONFORME

Art. 3.85 — Statuts et reglement d'ordre interieur
---------------------------------------------------

**Texte (resume)** :

- L'acte de base comprend la description de l'ensemble immobilier et des parties communes.
- Le reglement de copropriete fixe les criteres de repartition des charges.
- La periode de 15 jours pour convocations d'AG est fixee ici initialement
  (confirmee a l'Art. 3.87 §3).

**Alignement KoproGo** :

- Modele ``Building`` avec adresse, nom, total_units
- Repartition des charges via ``ChargeDistribution``
- Fichiers : ``backend/src/domain/entities/building.rs``, ``charge_distribution.rs``

**Statut** : CONFORME (partiel — pas de gestion de l'acte de base numerique)

Art. 3.86 — Personnalite juridique de l'ACP
--------------------------------------------

**Texte (resume)** :

- L'association des coproprietaires (ACP) a la personnalite juridique.
- Elle doit disposer d'un **fonds de roulement** (charges courantes) et d'un
  **fonds de reserve** (gros travaux), chacun sur un compte bancaire separe.
- Le syndic est l'organe executif.

**Alignement KoproGo** :

- Entite ``Organization`` representant l'ACP
- Plan comptable PCMN avec comptes separes fonds de roulement / reserve
- Comptes bancaires : ``Account`` classes 5 (tresorerie)
- Budget annuel : entite ``Budget`` avec workflow d'approbation

**Statut** : CONFORME

Art. 3.87 — Assemblee Generale
-------------------------------

C'est l'article le plus critique pour KoproGo. Il regit les assemblees generales.

§1 — Periodicite
~~~~~~~~~~~~~~~~~

**Texte** : *"Chaque annee, une assemblee generale ordinaire est tenue a la periode
fixee par le reglement de copropriete ou, a defaut, au cours du premier semestre."*

**Alignement** : Entite ``Meeting`` avec date et type. Pas de validation automatique
de la periodicite annuelle.

**Statut** : CONFORME (la validation periodique est hors scope v0.1.0)

§2 — Ordre du jour
~~~~~~~~~~~~~~~~~~~

**Texte** : *"L'ordre du jour de l'assemblee generale est fixe par le syndic.
Toute question inscrite a l'ordre du jour est traitee. Aucune decision ne peut etre
prise sur un point qui ne figure pas a l'ordre du jour."*

**Alignement** : L'entite ``Meeting`` a un champ ``agenda``. Cependant, il n'y a
**pas de lien formel entre les points d'agenda et les resolutions votees**.

**Statut** : LACUNE — Les resolutions ne sont pas liees aux points d'agenda.
Des decisions hors agenda pourraient etre enregistrees, ce qui les rendrait nulles en droit.

- Fichier concerne : ``backend/src/domain/entities/resolution.rs``
- Scenario BDD : ``resolutions.feature``

§3 — Convocations (15 jours minimum)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Texte** : *"Sauf dans les cas d'urgence, la convocation est communiquee quinze jours
au moins avant la date de l'assemblee, sauf si le reglement de copropriete prevoit un
delai plus long."*

.. important::

   Ce delai de **15 jours** s'applique a **TOUS les types d'assemblee** (ordinaire,
   extraordinaire, et deuxieme convocation). La loi ne fait aucune distinction.

**Alignement KoproGo** :

- ``ConvocationType::minimum_notice_days()`` retourne 15 pour tous les types
- Validation dans ``Convocation::new()`` : rejet si meeting_date - 15j < now
- Validation dans ``Convocation::schedule()`` : rejet si send_date > minimum_send_date

**Statut** : CONFORME (corrige dans cette release — precedemment 8 jours pour Extraordinary)

- Fichier : ``backend/src/domain/entities/convocation.rs:23-30``
- Scenario BDD : ``convocations.feature:19-32``
- Test unitaire : ``test_meeting_type_minimum_notice_days``

§4 — Representation
~~~~~~~~~~~~~~~~~~~~

**Texte** : *"Tout coproprietaire peut se faire representer par un mandataire, qui
peut ne pas etre coproprietaire."*

**Alignement** : Champ ``proxy_owner_id`` dans ``ConvocationRecipient`` et ``Vote``.

**Statut** : CONFORME

§5 — Quorum
~~~~~~~~~~~~

**Texte (resume)** :

- Premiere convocation : quorum atteint si les coproprietaires presents ou representes
  detiennent **plus de la moitie des quotes-parts** dans les parties communes.
- Si quorum non atteint, une **deuxieme assemblee** est convoquee dans les **quinze jours**.
  Elle delibere valablement quel que soit le nombre de presents.
- Exception : pour les decisions a **majorite des 3/4**, le quorum de premiere convocation
  exige la presence de coproprietaires detenant **au moins 3/4 des quotes-parts**.

.. warning::

   **LACUNE CRITIQUE** : KoproGo ne valide pas le quorum. Les votes sont possibles
   meme sans quorum de 50%.

**Alignement** :

- Pas de champ ``quorum_met`` dans ``Meeting``
- Pas de verification dans ``resolution_use_cases.rs``
- Pas de workflow automatique de deuxieme convocation

**Statut** : MANQUANT

- Fichiers concernes : ``meeting.rs``, ``resolution_use_cases.rs``
- Plan de remediation : Phase 1 critique

§7 — Procurations (max 3 mandats)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Texte (resume)** :

- Un mandataire ne peut representer **plus de trois coproprietaires**.
- Exception : il peut representer plus de trois si le total des voix dont il dispose
  (les siennes + celles de ses mandants) ne depasse pas **10% des voix totales**.

.. warning::

   **LACUNE CRITIQUE** : KoproGo ne limite pas le nombre de procurations par mandataire.
   Un mandataire peut representer un nombre illimite de coproprietaires.

**Statut** : MANQUANT

- Fichier concerne : ``backend/src/domain/entities/vote.rs``
- Plan de remediation : Phase 1 critique

Art. 3.88 — Majorites qualifiees
---------------------------------

**Texte (resume)** : Decisions requises selon le type :

.. list-table::
   :header-rows: 1
   :widths: 50 30 20

   * - Type de decision
     - Majorite requise
     - Statut KoproGo
   * - Decisions ordinaires
     - Majorite absolue (50%+1 exprimes)
     - CONFORME (``Simple``)
   * - Travaux extraordinaires
     - 2/3 des voix
     - CONFORME (``Qualified(0.667)``)
   * - Modification jouissance parties communes
     - 3/4 des voix
     - CONFORME (``Qualified(0.75)``)
   * - Modification statuts
     - 4/5 des voix
     - CONFORME (``Qualified(0.80)``)
   * - Modification quotes-parts
     - Unanimite
     - CONFORME (``Qualified(1.0)``)

**Alignement** : ``MajorityType`` enum avec ``Simple``, ``Absolute``, ``Qualified(f64)``
dans ``resolution.rs``. Le syndic choisit manuellement la majorite applicable.

**Statut** : CONFORME (via seuil personnalisable). Pas de presets automatiques par type de decision.

Art. 3.89 — Le Syndic
----------------------

**Texte (resume)** :

- Mandat maximum de **3 ans**, renouvelable.
- Obligations : comptabilite transparente, budget previsionnel, conservation documents.
- Responsabilite : le syndic est mandataire de l'ACP (Art. 3.89 §6). Sa responsabilite
  est contractuelle (mandat) et civile (faute de gestion).

**Alignement** :

- Information publique syndic : 7 champs publics, endpoint ``GET /public/buildings/{slug}/syndic``
- Comptabilite PCMN : ~90 comptes, rapports financiers (bilan, compte de resultats)
- Budget : entite ``Budget`` avec workflow d'approbation AG

**Statut** : CONFORME (partiel — pas de validation duree mandat 3 ans)

Art. 3.90 — Conseil de copropriete
-----------------------------------

**Texte (resume)** :

- Un conseil de copropriete est constitue dans tout immeuble ou groupe d'immeubles
  d'au moins **20 lots** (hors caves, garages et parkings).
- Missions : assistance et controle du syndic.

**Alignement** :

- Entite ``BoardMember`` avec positions (President, VicePresident, Treasurer, Secretary, Member)
- Entite ``BoardDecision`` avec workflow de suivi
- Pas de validation automatique du seuil de 20 lots

**Statut** : CONFORME (partiel)

Art. 3.94 — Etat date
-----------------------

**Texte (resume)** :

- En cas de cession d'un lot, le syndic transmet les informations requises dans un delai de :
  - **15 jours** pour une demande simple
  - **30 jours** pour une demande par recommande du notaire
- Le document contient la situation financiere du lot (arrieres, provisions, travaux votes).

**Alignement** :

- Entite ``EtatDate`` avec 16 sections legales
- Delai de detection : ``is_overdue()`` utilise 15 jours (corrige dans cette release — precedemment 10 jours)
- Validite 90 jours : pratique professionnelle, pas une obligation legale

**Statut** : CONFORME (corrige dans cette release)

- Fichier : ``backend/src/domain/entities/etat_date.rs:286-297``
- Repository SQL : ``etat_date_repository_impl.rs`` (INTERVAL '15 days')
