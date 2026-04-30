============================================================
Diagnostic : Specifications Multi-Roles & Couverture Tests
============================================================

:Date: 2026-03-26
:Statut: DRAFT — En cours d'elaboration
:Objectif: Comparer les postulats implicites de l'application avec les obligations
           legales belges et identifier les gaps dans la couverture de tests.

1. Roles : Code vs Droit belge
===============================

1.1 Roles implementes dans le code (RBAC)
------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 15 70

   * - Role
     - Type
     - Perimetre
   * - **SuperAdmin**
     - Technique
     - Gestion plateforme SaaS, multi-tenant, seed data
   * - **Syndic**
     - Legal (Art. 3.89)
     - Gestion quotidienne, AG, convocations, devis, tickets, budgets
   * - **Accountant**
     - Legal (implicite)
     - Comptabilite PCMN, journal, rapports financiers, charges
   * - **Owner**
     - Legal (Art. 3.86-3.88)
     - Vote, paiements, tickets, SEL, community, documents
   * - **BoardMember**
     - Legal (Art. 3.90)
     - Entity + dashboard, mais pas role RBAC a part entiere
   * - **Contractor**
     - Operationnel
     - Acces via magic link JWT 72h (pas de role RBAC standard)
   * - **Broker**
     - Operationnel
     - Offres energy campaigns (permission speciale)

1.2 Roles identifies dans la documentation legale (docs/legal/)
----------------------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 15 15 50

   * - Role legal
     - Codes regles
     - Dans le code ?
     - Commentaire
   * - Syndic
     - L01-L18, M01-M11, D01-D14, T01-T07
     - Oui
     - Completement implemente
   * - Coproprietaire
     - CP01-CP15
     - Oui (Owner)
     - Bien couvert. CP02 (assistant), CP03 (procuration max 3), CP05 (AGE 1/5), CP08 (rectification quotes) partiellement implementes
   * - Conseil de copropriete
     - CC01-CC06
     - **Partiel**
     - Entity BoardMember existe, mais CC03 (controle syndic), CC04 (delegations AG) et CC06 (propositions OdJ) manquent comme fonctionnalites
   * - Commissaire aux comptes
     - CO01-CO05
     - **Non**
     - Aucun role dedie. CO03 (acces documents comptables) et CO04 (rapport annuel) non implementes. Le commissaire est critique pour la sequence AG (etape 3 et 6)
   * - Locataire
     - LO01-LO06
     - **Non**
     - Aucune entite ni role. LO01 (information AG), LO03 (observations ecrites) non implementes
   * - ACP (personne morale)
     - ACP01-ACP08
     - **Non**
     - Pas d'entite representant l'ACP elle-meme (patrimoine, BCE, representant legal)
   * - Notaire
     - N01-N05
     - **Partiel**
     - Module Etat Date (N02-N03) implemente, mais N01 (demande initiale) et N05 (transfert syndic) non implementes comme workflow notaire

1.3 Gap : roles legaux non couverts
-------------------------------------

**Priorite haute** :

- **Commissaire aux comptes** : Role central dans la sequence AG (rapport avant vote des comptes,
  decharge distincte). Sans ce role, les etapes 3, 5 et 6 de la sequence AG sont incompletes.

- **Conseil de copropriete** : Obligatoire pour immeubles >= 20 lots (CC01). Les fonctions de
  controle du syndic (CC03) et de proposition OdJ (CC06) sont des droits legaux non implementes.

**Priorite moyenne** :

- **Locataire** : Droit d'information AG (LO01) et d'observations (LO03) sont des obligations
  legales du syndic (L07). L'absence de ce role signifie que le syndic ne peut pas remplir L07
  via la plateforme.

**Priorite basse** :

- **ACP** : Abstraction juridique, pas necessairement un role applicatif.
- **Notaire** : Interaction ponctuelle, le module Etat Date couvre l'essentiel.

2. State Machines & Workflows
==============================

2.1 Workflows single-role (bien couverts)
-------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 20 55

   * - Workflow
     - Acteur principal
     - Etats
   * - Expense lifecycle
     - Syndic/Accountant
     - Draft -> PendingApproval -> Approved/Rejected
   * - Budget lifecycle
     - Syndic
     - Draft -> Submitted -> Approved/Rejected -> Archived
   * - Convocation lifecycle
     - Syndic
     - Draft -> Scheduled -> Sent -> (Cancelled)
   * - Quote lifecycle
     - Syndic (+ Contractor submit)
     - Requested -> Received -> UnderReview -> Accepted/Rejected/Expired
   * - Notification lifecycle
     - System
     - Pending -> Sent/Failed -> Read (InApp)
   * - Payment lifecycle
     - System (Stripe)
     - Pending -> Processing -> Succeeded/Failed/Refunded

2.2 Workflows multi-roles (necessitent tests multi-acteurs)
-------------------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 55 20

   * - Workflow
     - Sequence multi-roles
     - Couverture test
   * - **Ticket maintenance**
     - Owner cree -> Syndic assigne -> Contractor travaille (magic link) -> Syndic valide rapport -> Paiement
     - BDD: 17 scenarios (single-role implicite). E2E: 1 scenario (syndic only). **Gap: pas de test Owner->Syndic->Contractor**
   * - **Vote AG (Resolution)**
     - Syndic cree resolution -> Meeting quorum valide -> Owners votent (tantiemes) -> Syndic cloture -> Resultat (Simple/Absolute/Qualified)
     - BDD: 14+40 scenarios (multi-acteur implicite). E2E: 1 scenario (en debug). **Gap: quorum + multi-owner vote non teste E2E**
   * - **Sondage (Poll)**
     - Syndic cree/publie -> Owners repondent -> Syndic cloture/resultats
     - BDD: 40 scenarios. E2E: 1 scenario (en debug). **Gap: owner vote non teste E2E**
   * - **SEL (echange local)**
     - Owner A offre -> Owner B demande -> Owner A accepte/demarre -> Completion -> Rating mutuel -> Credits
     - BDD: 17 scenarios (Alice/Bob). E2E: 1 scenario (en debug). **Gap: workflow complet non teste E2E**
   * - **Demande AGE**
     - Owner initie -> Owners cosignent (seuil 1/5) -> Syndic repond (15j) -> AG extraordinaire ou auto-convocation
     - BDD: 15 scenarios. E2E: 0. **Gap: aucun test E2E**
   * - **Rapport prestataire (BC16)**
     - Syndic envoie magic link -> Contractor remplit rapport (PWA) -> Board valide -> Paiement auto
     - BDD: 16 scenarios. E2E: 1 smoke. **Gap: workflow complet non teste E2E**
   * - **Notice board**
     - Syndic/Owner cree annonce -> Syndic publie -> Owners lisent
     - BDD: 15 scenarios. E2E: 1 scenario (en debug). **Gap: publication non testee E2E**
   * - **Annonce communautaire**
     - Owners interagissent (Skills, Sharing, Bookings)
     - BDD: 44 scenarios. E2E: smokes. **Gap: aucun scenario multi-owner E2E**

2.3 Pre-conditions legales critiques
---------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 25 50

   * - Pre-condition
     - Article
     - Implementation
   * - Quorum > 50% (strict)
     - Art. 3.87 §5
     - Implemente. Meeting.validate_quorum(). **Pas de endpoint API public** pour valider le quorum.
   * - 2e convocation exempte quorum
     - Art. 3.87 §5
     - Implemente (is_second_convocation flag)
   * - Delai convocation 15 jours
     - Art. 3.87 §3
     - Implemente (minimum_send_date)
   * - Seuil AGE 1/5 quotes-parts
     - Art. 3.87 §2
     - Implemente (AgeRequest)
   * - Delai syndic 15 jours (AGE)
     - Art. 3.87 §2
     - Implemente (syndic_deadline_at)
   * - Max 3 procurations
     - Art. 3.87 §7
     - **Non implemente** — La limite de 3 procurations par mandataire (sauf si total < 10%) n'est pas validee
   * - Plafonnement vote 50%
     - Art. 3.87 §6
     - **Non implemente** — Un coproprietaire > 50% des voix n'est pas automatiquement plafonne
   * - PV dans 30 jours
     - Art. 3.87 §12
     - Implemente (minutes_sent_at, is_minutes_overdue)
   * - Rapport commissaire avant vote comptes
     - Art. 3.91
     - **Non implemente** — Pas de role commissaire, pas de workflow rapport
   * - 3 devis > 5000 EUR
     - Bonne pratique belge
     - Implemente (compare_quotes scoring)
   * - Delai recours 4 mois
     - Art. 3.92 §3
     - Documente (CP07) mais pas d'alerte automatique
   * - Conseil obligatoire >= 20 lots
     - Art. 3.90 §1
     - **Non implemente** — Pas de detection automatique du seuil

3. Inventaire des Tests
========================

3.1 Chiffres globaux
---------------------

- **842** scenarios BDD (68 feature files)
- **240** tests E2E smoke (49 spec files)
- **12** scenarios Documentation Vivante
- **6/12** scenarios passent actuellement

3.2 Couverture par feature
----------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 10 10 10 10

   * - Feature
     - BDD
     - E2E Smoke
     - E2E Scenario
     - Multi-role ?
   * - Authentication
     - 3
     - 1
     - 0
     - Non
   * - Tickets
     - 17
     - 6
     - 1 (passe)
     - Non (syndic seul)
   * - Expenses/Invoices
     - 40
     - 6
     - 1 (passe)
     - Non (syndic seul)
   * - Meetings/Resolutions
     - 39
     - 5
     - 1 (debug)
     - Multi-role ecrit, en debug
   * - Convocations
     - 13
     - 4
     - 1 (passe)
     - Non (syndic seul)
   * - Polls
     - 54
     - 4
     - 1 (debug)
     - Multi-role ecrit, en debug
   * - Payments
     - 28
     - 7
     - 1 (debug)
     - Owner ecrit, en debug
   * - SEL (exchanges)
     - 17
     - 4
     - 1 (debug)
     - Multi-role ecrit, en debug
   * - Quotes
     - 13
     - 5
     - 1 (debug)
     - Syndic seul
   * - Budget
     - 11
     - 4
     - 1 (passe)
     - Non (syndic seul)
   * - Gamification
     - 13
     - 4
     - 0
     - Non
   * - Board Management
     - 50
     - 13
     - 0
     - Non
   * - GDPR
     - 28
     - 5
     - 0
     - Non
   * - Energy Campaigns
     - 14
     - 4
     - 0
     - Non
   * - AGE Requests
     - 15
     - 0
     - 0
     - **Aucun test E2E**
   * - Notices
     - 15
     - 4
     - 1 (debug)
     - Multi-role ecrit, en debug
   * - Contractor Reports
     - 16
     - 4
     - 0
     - Non
   * - IoT/Linky
     - 34
     - 0
     - 0
     - **Aucun test E2E**
   * - Admin/Dashboard
     - 4
     - 10
     - 2 (passent)
     - Non

4. Delta : Specifications vs Implementation
=============================================

4.1 Postulats implicites non conformes au droit belge
------------------------------------------------------

**P1 — Tout utilisateur authentifie est syndic ou owner** :
Le code RBAC part du principe que 4 roles suffisent. Or le droit belge identifie 7 acteurs
distincts dans une copropriete (syndic, coproprietaire, conseil, commissaire, locataire, ACP,
notaire). Le commissaire aux comptes est particulierement critique car il conditionne le vote
des comptes en AG.

**P2 — Les jeux de donnees de test ne couvrent pas le quorum** :
La validation du quorum (Art. 3.87 §5) est correctement implementee comme invariant du domaine
(Meeting.validate_quorum). Cependant, les seeds de test (DatabaseSeeder + global-setup.ts) ne
creent pas de meeting avec quorum valide. Consequence : les tests E2E ne peuvent pas creer de
resolutions car le meeting n'a pas de quorum. Solution : etendre les seeds avec des jeux de
donnees pre-configures (``faker`` est deja utilise dans seed.rs) incluant des meetings avec
units + owners + tantiemes + quorum valide, prets pour les tests de vote.

**P3 — Les procurations sont illimitees** :
Le code gere les proxy_owner_id dans les votes mais ne valide pas la limite de 3 procurations
par mandataire (Art. 3.87 §7). Un mandataire pourrait theoriquement voter pour un nombre
illimite de coproprietaires.

**P4 — Pas de plafonnement a 50% des voix** :
Un coproprietaire majoritaire (> 50% des quotes-parts) devrait voir ses voix plafonnees
(Art. 3.87 §6). Cette regle n'est pas implementee dans Resolution.close_voting().

**P5 — La sequence AG n'est pas imposee par le code** :
La sequence logico-juridique en 12 etapes (docs/legal/assemblee-generale/sequence_odj.rst)
est documentee mais pas enforcee par le code. Rien n'empeche de voter le budget avant
l'approbation des comptes.

4.2 Gaps de tests multi-roles
-------------------------------

**G1 — Aucun test E2E ne fait voter un coproprietaire** :
Les 842 BDD scenarios testent les use cases backend mais aucun test E2E ne valide qu'un
Owner peut se connecter, naviguer vers une AG, et voter sur une resolution. Le scenario
meeting-vote.scenario.ts est ecrit mais echoue (quorum non validable via API).

**G2 — Le workflow complet Ticket -> Rapport -> Paiement n'est pas teste** :
Le ticket-lifecycle scenario ne va que jusqu'a la creation. Il n'y a pas de scenario
qui montre Owner -> Syndic -> Contractor (magic link) -> Board validation -> Paiement.

**G3 — Les echanges SEL ne sont pas testes entre 2 owners** :
Le BDD teste Alice/Bob en isolation. Aucun test E2E ne montre 2 coproprietaires
interagissant sur la marketplace.

**G4 — Les demandes AGE n'ont aucun test E2E** :
15 BDD scenarios mais 0 smoke tests et 0 scenarios. Le workflow Owner -> cosignataires ->
syndic -> AG extraordinaire n'est jamais teste via l'interface.

**G5 — Le commissaire aux comptes est invisible** :
Pas de role, pas de workflow, pas de test. Le rapport annuel (CO04) et la decharge (CO05)
sont des obligations legales non couvertes.

5. Recommandations
===================

5.1 Priorite 1 : Seeds de donnees pour tests multi-roles
-----------------------------------------------------------

Le quorum est un invariant du domaine — il ne doit PAS etre expose comme endpoint API.
La solution est d'etendre les seeds (``backend/src/infrastructure/database/seed.rs``) avec
des jeux de donnees pre-configures pour les scenarios multi-roles :

- **Seed "AG avec quorum"** : Meeting Scheduled + 3 units avec tantiemes (300, 200, 500)
  + 3 owners assignes + quorum valide a 60% (600/1000) → pret pour creer des resolutions
- **Seed "SEL actif"** : 2 owners avec credit balances + 3 echanges (Offered, Requested, Completed)
- **Seed "Devis comparatifs"** : 3 quotes soumises avec montants differents
- **Seed "Notice board"** : 3 annonces publiees + 1 brouillon

Utiliser la crate ``fake`` (deja importee) pour generer les donnees.
Les seeds doivent etre idempotents (verifier si les donnees existent deja).
Chaque seed est un ``POST /seed/{scenario_name}`` (SuperAdmin only, comme l'existant).

5.2 Priorite 2 : Combler les gaps legaux critiques
----------------------------------------------------

- Implementer la validation des procurations (max 3 par mandataire, Art. 3.87 §7)
- Implementer le plafonnement a 50% des voix (Art. 3.87 §6)
- Creer le role Commissaire aux comptes (meme si c'est un sous-role de Owner avec
  permissions supplementaires)

5.3 Priorite 3 : Scenarios multi-roles manquants
--------------------------------------------------

- Ecrire un scenario BDD + E2E aligne pour le workflow AGE complet
- Ecrire un scenario complet Ticket -> Rapport -> Paiement
- Enrichir le scenario meeting-vote avec quorum et multi-owner

5.4 A evaluer (non prioritaire)
---------------------------------

- Role Locataire (LO01-LO06) : utile seulement si des immeubles mixtes sont vises
- Role Notaire (N01-N05) : interaction ponctuelle, le module Etat Date suffit
- Enforcement de la sequence AG : complexe et potentiellement contraignant pour les syndics

5.5 Gaps legaux a combler (identifies par recherche externe)
--------------------------------------------------------------

Les recherches sur les sources officielles belges identifient des obligations non couvertes :

**P6 — Consentement explicite pour convocations electroniques** :
Art. 3.87 §3 exige que chaque destinataire ait « accepte individuellement et expressement »
la reception par voie electronique. Le systeme de convocations envoie des emails sans
tracker ce consentement par coproprietaire. A implementer : champ ``email_consent`` dans
ConvocationRecipient avec date de consentement.

**P7 — Base legale AG numerique non documentee** :
La loi du 22 octobre 2022 amende Art. 3.87 §1 pour autoriser la participation a distance
par « un moyen de communication electronique ». Notre module AgSession implemente cette
fonctionnalite mais la reference legale n'est pas documentee dans docs/legal/.

**P8 — Mandat syndic max 3 ans non enforce** :
Art. 3.89 §1 al.4 : « Le mandat du syndic ne peut exceder trois ans mais est renouvelable ».
Le systeme ne valide pas cette duree maximale.

**P9 — Agenda-resolution non liee** :
Art. 3.87 §2 : Seuls les points inscrits a l'ordre du jour peuvent faire l'objet d'un vote.
Le code a un champ ``agenda_item_index`` dans Resolution mais il est optionnel.
Il devrait etre obligatoire.

5.6 Positionnement concurrentiel
----------------------------------

Les plateformes belges comparables (Syndicus.be, Smovin, Whise, Yardi) couvrent la gestion
financiere et les AG mais ne proposent pas :

- **Open source** (AGPL) — unique sur le marche belge
- **GDPR complet** (Articles 15-21) — la plupart ne font que le minimum
- **Modules communautaires** (SEL, Skills, Sharing, Bookings) — inexistant chez les concurrents
- **Gamification** — approche inedite pour l'engagement des coproprietaires
- **Achats groupes d'energie** — besoin emergent, peu adresse
- **Vote numerique conforme** — la plupart utilisent encore le papier

Ces differenciateurs justifient de prioriser la conformite legale du coeur (AG, quorum,
procurations) avant d'etendre les features communautaires.

6. Sources
===========

Legislation
~~~~~~~~~~~

- Code civil belge, Livre 3, Titre 1, Sous-titre 3 (Art. 3.78 a 3.100)
- Loi du 18 juin 2018 (MB 02/07/2018) — Reforme copropriete
- Loi du 22 octobre 2022 — Participation AG par visioconference (amende Art. 3.87 §1)
- Reglement eIDAS (UE 910/2014) — Signature electronique pour PV
- Reglement RGPD (UE 2016/679)

Sources professionnelles
~~~~~~~~~~~~~~~~~~~~~~~~~

- Code de deontologie IPI (AR 29/06/2018, MB 31/10/2018)
- IPI guides pratiques : https://www.ipi.be/syndic
- SNPC (Syndicat National des Proprietaires) : https://www.snpc-nems.be/copropriete
- Federation Royale du Notariat Belge : https://www.notaire.be/immobilier/la-copropriete

Documentation projet
~~~~~~~~~~~~~~~~~~~~

- docs/legal/ (base legale KoproGo — 7 roles, ~65 regles codifiees)
- docs/ROLE_PERMISSIONS_MATRIX.rst
- Extraction RBAC du code source (mars 2026)
- Extraction state machines du code source (mars 2026)
- Inventaire tests (842 BDD + 240 E2E + 12 scenarios)

Doctrine
~~~~~~~~~

- Pascale Lecocq (ULiege), « La copropriete forcee des immeubles » (Larcier)
- Revue de droit immobilier (JLMB)
