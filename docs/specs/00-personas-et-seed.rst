=================================================
Personas & Seed de reference — Immeuble type
=================================================

:Issue: #346, #347
:Date: 28 mars 2026

Ce document definit l'immeuble de reference et les personas utilises dans
**toutes** les specs multi-roles. C'est le jeu de donnees unique qui sera
implemente comme seed backend (issue #347).

**Prealable obligatoire** : Lire `00-sociologie-copropriete.rst <00-sociologie-copropriete.rst>`_
qui explique la complexite humaine derriere ces personas. Chaque persona represente
un archetype sociologique reel, pas un role technique abstrait.

**Principe du seed** : Chaque persona a une raison d'etre dans le jeu de donnees.
Le setup cree le persona avec un etat initial precis, le test valide un comportement
metier, et le teardown nettoie. Le "pourquoi" explique quelle realite humaine ce test
adresse. KoproGo ne jette la pierre a personne — il met en place des processus et de
la transparence qui aident tout le monde dans cette mission impossible.

Immeuble de reference : Residence du Parc Royal
=================================================

:Nom: Residence du Parc Royal
:Adresse: 42 Avenue Louise, 1050 Ixelles
:Ville: Bruxelles
:Total lots: **182** (appartements, parkings, caves)
:Total tantiemes: **10 000** (dix-milliemes)
:Construction: 1965
:Conseil de copropriete: Obligatoire (>= 20 lots, Art. 3.90 §1)

Repartition des lots (seed de test)
-------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 8 15 8 10 12 20 27

   * - Lot
     - Type
     - Etage
     - Surface
     - Tantiemes
     - Proprietaire
     - Archetype
   * - 1A
     - Appartement
     - 1
     - 75 m²
     - 380/10000
     - Marguerite Lemaire
     - Personne agee, tresorerie limitee
   * - 1B
     - Appartement
     - 1
     - 55 m²
     - 290/10000
     - Jeanne Devos
     - Personne agee en precarite
   * - 2A
     - Appartement
     - 2
     - 90 m²
     - 450/10000
     - Alice Dubois
     - Retraitee impliquee, presidente CdC
   * - 2B
     - Appartement
     - 2
     - 85 m²
     - 430/10000
     - Bob Janssen
     - Comptable, commissaire aux comptes
   * - 3A
     - Appartement
     - 3
     - 110 m²
     - 580/10000
     - Diane Peeters
     - Avocate, membre CdC
   * - 3B
     - Appartement
     - 3
     - 105 m²
     - 660/10000
     - Charlie Martin
     - Jeune couple sous pression
   * - 4A
     - Appartement
     - 4
     - 60 m²
     - 320/10000
     - Nadia Benali
     - Jeune acquereur surendettee
   * - 4B
     - Appartement
     - 4
     - 90 m²
     - 450/10000
     - Marcel Dupont
     - Ancien qui veut tout changer
   * - 5A
     - Penthouse
     - 5
     - 180 m²
     - 1280/10000
     - Emmanuel Claes
     - Investisseur absent
   * - 6A-6C
     - Appartements
     - 6
     - 3x100 m²
     - 3x600=1800/10000
     - Philippe Vandermeulen (x3)
     - Investisseur multi-lots
   * - Reste
     - 172 lots
     - 1-12
     - Variable
     - 3360/10000
     - Non attribues (autres coproprietaires)
     - Simulent la majorite silencieuse

**Total seed** : 10 coproprietaires = 6640/10000 tantiemes (66.4%)

Personas : Coproprietaires
============================

Alice Dubois — La retraitee qui tient l'immeuble a bout de bras
-----------------------------------------------------------------

:Email: alice@residence-parc.be
:Tantiemes: 450 (4.5%)
:Roles: Coproprietaire + **Presidente du CdC**
:Lot: 2A, appartement 90 m², 2e etage

**Qui** : 67 ans, retraitee, veuve. Vit dans l'immeuble depuis 25 ans. Pension correcte
+ petit livret epargne (15.000 EUR). Tres impliquee : presente a chaque AG, organise
le SEL (cours de cuisine), connait tout le monde dans l'immeuble.

**Pourquoi dans le seed** : Teste le role CdC (presidente), le droit d'initiative AGE
(Art. 3.87 §2), la participation active au SEL, et le cas du coproprietaire benevole
qui compense a lui seul le desengagement des autres.

**Ce que ca teste (metier)** :

- Le dashboard CdC fonctionne-t-il pour un role non-syndic ?
- Les missions du syndic (L01-L18) sont-elles visibles pour le CdC ?
- Une presidente peut-elle initier une demande AGE ?
- Le SEL fonctionne-t-il pour les retraites (pas de competence technique) ?

**Ce que ca resout (humain)** : Alice represente les benevoles qui font tourner la
copropriete. Sans elle, personne ne controle le syndic, personne n'anime la communaute.
KoproGo doit rendre son engagement **tenable** — pas l'alourdir de bureaucratie.

Bob Janssen — L'expert comptable qui verifie les comptes
----------------------------------------------------------

:Email: bob@residence-parc.be
:Tantiemes: 430 (4.3%)
:Roles: Coproprietaire + **Commissaire aux comptes**
:Lot: 2B, appartement 85 m², 2e etage

**Qui** : 55 ans, comptable independant. Verifie les comptes annuels benevolement.
Connait le PCMN par coeur. Participe au SEL (depannage informatique).

**Pourquoi dans le seed** : Teste le role commissaire aux comptes (CO01-CO05), l'acces
aux documents comptables (Art. 3.91), et le rapport annuel avant AG.

**Ce que ca teste (metier)** :

- Le commissaire a-t-il acces en lecture aux rapports financiers ?
- Le rapport annuel peut-il etre soumis avant l'AG (etape 3 de la sequence OdJ) ?
- La decharge du commissaire est-elle un point separe (etape 6) ?

**Ce que ca resout (humain)** : Bob donne de son temps gratuitement. KoproGo doit lui
fournir les donnees **sans qu'il ait a les demander** au syndic. Acces direct = moins
de friction = Bob continue a s'impliquer.

Charlie Martin — Le jeune couple sous pression financiere
-----------------------------------------------------------

:Email: charlie@residence-parc.be
:Tantiemes: 660 (6.6%)
:Roles: Coproprietaire
:Lot: 3B, appartement 105 m², 3e etage

**Qui** : 38 ans, couple avec 2 enfants (7 et 4 ans). Achete en 2021 avec credit a taux
variable remonte a 4.2%. Charges communes = 48% du revenu. Utilise le partage d'objets
et les reservations.

**Pourquoi dans le seed** : Teste l'impact financier des decisions d'AG sur un budget
serre, les modules communautaires (partage, reservations) comme levier d'economie.

**Ce que ca teste (metier)** :

- La simulation d'impact financier avant vote affiche-t-elle le bon montant ?
- Un coproprietaire sous pression peut-il demander un echelonnement ?
- Le partage d'objets et les reservations fonctionnent-ils ?

**Ce que ca resout (humain)** : Charlie ne vote pas "non" par mauvaise volonte — il
vote "non" parce que 3.000 EUR d'appel de fonds le mettraient dans le rouge. KoproGo
doit lui montrer l'impact AVANT le vote et proposer des solutions AVANT la crise.

Diane Peeters — L'avocate qui verifie chaque virgule
-------------------------------------------------------

:Email: diane@residence-parc.be
:Tantiemes: 580 (5.8%)
:Roles: Coproprietaire + **Membre du CdC**
:Lot: 3A, appartement 110 m², 3e etage

**Qui** : 45 ans, avocate en droit immobilier. Connait le Code Civil mieux que le syndic.
Verifie la conformite des convocations et des majorites.

**Pourquoi dans le seed** : Teste la conformite legale de bout en bout. Si Diane ne
trouve pas d'erreur dans KoproGo, le systeme est conforme.

**Ce que ca teste (metier)** :

- Les convocations respectent-elles le delai de 15 jours ?
- La decharge du syndic est-elle un point separe des comptes ?
- Les majorites sont-elles correctement calculees (Art. 3.88) ?

**Ce que ca resout (humain)** : KoproGo doit rendre les erreurs de procedure
**impossibles** — pas compter sur la vigilance de Diane pour les detecter.

Emmanuel Claes — L'investisseur absent qui bloque
----------------------------------------------------

:Email: emmanuel@residence-parc.be
:Tantiemes: 1280 (12.8%)
:Roles: Coproprietaire
:Lot: 5A, penthouse 180 m², 5e etage

**Qui** : 52 ans, cadre bancaire. Penthouse loue a des expats. Ne vit pas sur place.
Jamais aux AG, donne procuration sans lire l'OdJ.

**Pourquoi dans le seed** : Teste le vote par procuration, le plafonnement 50%
(Art. 3.87 §6), et la dynamique de blocage par les investisseurs absents.

**Ce que ca teste (metier)** :

- Le systeme de procuration fonctionne-t-il (max 3 mandats, exception 10%) ?
- Le plafonnement a 50% s'applique-t-il correctement ?
- Le vote a distance (visio AG) encourage-t-il la participation ?

**Ce que ca resout (humain)** : S'il peut voter en 2 clics depuis son bureau,
Emmanuel votera peut-etre "oui" au lieu de donner une procuration "non" par defaut.

Nadia Benali — La jeune acquereur surendettee
------------------------------------------------

:Email: nadia@residence-parc.be
:Tantiemes: 320 (3.2%)
:Roles: Coproprietaire
:Lot: 4A, appartement 60 m², 4e etage

**Qui** : 32 ans, infirmiere. Premier achat en 2024 avec Karim. Credit 25 ans, taux
variable 4.2%. Charges = dernier poste avant le decouvert.

**Pourquoi dans le seed** : Teste les cas limites financiers — simulation d'impact,
echelonnement, relances de paiement, et le Fonds de Solidarite.

**Ce que ca teste (metier)** :

- L'impact financier par lot s'affiche-t-il avant chaque vote ?
- Les relances respectent-elles l'escalade (Gentle → Formal → FinalNotice → Legal) ?
- Le Fonds de Solidarite est-il accessible en cas de defaut ?

**Ce que ca resout (humain)** : Nadia ne doit pas apprendre par surprise qu'elle doit
3.000 EUR. KoproGo doit la prevenir EN AMONT. L'objectif : zero expulsion pour impaye.

Marguerite Lemaire — La veuve a pension reduite
--------------------------------------------------

:Email: marguerite@residence-parc.be
:Tantiemes: 380 (3.8%)
:Roles: Coproprietaire
:Lot: 1A, appartement 75 m², 1er etage

**Qui** : 78 ans, veuve, pension de survie 1.200 EUR/mois. Vit dans l'immeuble depuis
1985. Appartement rembourse mais charges = 15% de sa pension. Petit livret (12.000 EUR)
qui fond lentement. Ne comprend pas les termes techniques en AG.

**Pourquoi dans le seed** : Teste l'accessibilite (langage simple, multilingue),
l'impact financier sur les petites pensions, et le SEL comme lien social.

**Ce que ca teste (metier)** :

- L'interface est-elle comprehensible pour une personne de 78 ans ?
- L'impact financier est-il affiche en langage simple ?
- Le SEL permet-il a Marguerite de se sentir utile (offre repassage) ?

**Ce que ca resout (humain)** : Marguerite ne doit pas se sentir exclue des decisions
qui la concernent. Les annonces de l'immeuble sont son seul lien social numerique.

Jeanne Devos — La personne agee en precarite
-----------------------------------------------

:Email: jeanne@residence-parc.be
:Tantiemes: 290 (2.9%)
:Roles: Coproprietaire
:Lot: 1B, appartement 55 m², 1er etage

**Qui** : 82 ans, veuve, pension minimum 1.050 EUR/mois. AUCUNE tresorerie. Son fils
l'aide parfois. Quand les charges augmentent, elle reduit ses repas. Un appel de fonds
de 2.000 EUR la mettrait en defaut de paiement. Pas d'email — tout par courrier.

**Pourquoi dans le seed** : Teste le cas extreme de precarite, la convocation par
courrier recommande (pas d'email), et le Fonds de Solidarite.

**Ce que ca teste (metier)** :

- Le systeme gere-t-il les destinataires sans email (courrier recommande) ?
- Le Fonds de Solidarite est-il accessible pour les cas extremes ?
- L'escalade de relances s'arrete-t-elle avant l'action legale quand le CPAS intervient ?

**Ce que ca resout (humain)** : Jeanne est la personne la plus vulnerable de l'immeuble.
Chaque decision de l'AG peut menacer son logement. KoproGo doit detecter sa fragilite
et orienter vers les aides AVANT qu'il soit trop tard.

Philippe Vandermeulen — L'investisseur multi-lots qui bloque
--------------------------------------------------------------

:Email: philippe@residence-parc.be
:Tantiemes: 1800 (18.0%)
:Roles: Coproprietaire (3 lots)
:Lots: 6A, 6B, 6C, trois appartements de 100 m², 6e etage

**Qui** : 52 ans, directeur financier. 3 appartements loues. Ne vit pas dans l'immeuble.
Analyse chaque decision en termes de rendement locatif. Ses procurations "non"
systematiques bloquent les majorites qualifiees.

**Pourquoi dans le seed** : Teste le multi-lots (1 owner = 3 units), le pouvoir de
blocage (18% + Emmanuel 12.8% = 30.8%), et les limites de procuration.

**Ce que ca teste (metier)** :

- Le systeme gere-t-il correctement un proprietaire de plusieurs lots ?
- Les tantiemes sont-ils agreges correctement pour le vote ?
- La coalition Philippe+Emmanuel bloque-t-elle les 2/3 ? Les 4/5 ?
- Le reporting d'impact montre-t-il l'effet du report de travaux sur la valeur du bien ?

**Ce que ca resout (humain)** : Philippe n'est pas "mechant" — il est rationnel a court
terme. KoproGo doit lui montrer que le report de travaux degrade son investissement.

Marcel Dupont — L'ancien qui veut tout changer d'un coup
----------------------------------------------------------

:Email: marcel@residence-parc.be
:Tantiemes: 450 (4.5%)
:Roles: Coproprietaire (+ syndic benevole dans le scenario alternatif)
:Lot: 4B, appartement 90 m², 4e etage

**Qui** : 67 ans, retraite depuis 2 ans. 30 ans dans l'immeuble. A toujours vote
"oui vite fait" aux AG pendant sa carriere. Decouvre maintenant le retard d'entretien.

**Pourquoi dans le seed** : Teste la demande d'AGE (initiateur), le plan de travaux
pluriannuel, et le scenario syndic benevole.

**Ce que ca teste (metier)** :

- La demande AGE fonctionne-t-elle (seuil 1/5, cosignatures, delai 15j) ?
- Le plan de travaux echelonne est-il comprehensible ?
- En mode syndic benevole : les templates d'OdJ et alertes de delai l'aident-ils ?

**Ce que ca resout (humain)** : Marcel est le futur syndic benevole. **C'est pour lui
que KoproGo existe en priorite** — memes 18 obligations legales qu'un pro, sans
formation, sans logiciel. L'application doit rendre le role tenable.

Personas : Professionnels
===========================

Francois Leroy — Le syndic professionnel entre marteau et enclume
-------------------------------------------------------------------

:Email: francois@syndic-leroy.be
:Roles: **Syndic professionnel** (IPI n° 500.xxx)

**Qui** : 48 ans, gere 15 immeubles soit ~800 lots. Lie par 18 missions legales
(Art. 3.89 §5) et le code de deontologie IPI. Mandat max 3 ans, revocable a tout moment.
Ne peut etre mandataire en AG ni membre du CdC (Art. 3.89 §9).

**Pourquoi dans le seed** : Teste toutes les fonctions syndic — convocations, AG,
resolutions, devis, tickets, rapports, comptabilite. C'est le role principal du systeme.

**Ce que ca teste (metier)** :

- Les 18 missions legales sont-elles toutes couvertes par le systeme ?
- Les alertes de delai fonctionnent-elles (convocation 15j, PV 30j, mandat 3 ans) ?
- Les templates d'OdJ pre-remplissent-ils les 12 points obligatoires d'une AGO ?

**Ce que ca resout (humain)** : Le metier de syndic est en penurie en Belgique —
ingrat, sous-paye, responsabilite personnelle. KoproGo automatise l'administratif
pour que Francois passe moins de temps sur la conformite et plus sur la mediation.

Gisele Vandenberghe — La comptable externe
---------------------------------------------

:Email: gisele@cabinet-vdb.be
:Roles: **Comptable**

**Qui** : Comptable externe specialisee en copropriete. Saisit les ecritures PCMN,
prepare les rapports financiers, gere les appels de fonds et les relances.

**Pourquoi dans le seed** : Teste le role comptable — acces PCMN, journal entries,
rapports financiers, distribution des charges.

**Ce que ca teste (metier)** :

- Le module PCMN est-il utilisable par un comptable professionnel ?
- Les rapports financiers (bilan, compte de resultat) sont-ils conformes ?
- La distribution des charges par tantiemes est-elle correcte ?

**Ce que ca resout (humain)** : Gisele gagne du temps si le systeme calcule
automatiquement les repartitions. Moins de saisie manuelle = moins d'erreurs.

Hassan El Amrani — Le prestataire via magic link
---------------------------------------------------

:Email: hassan@toitures-bruxelles.be
:Roles: **Prestataire** (acces via magic link JWT 72h)

**Qui** : Entrepreneur couvreur. Recoit les demandes de devis par email. Accede aux
rapports de travaux via magic link (pas de compte permanent).

**Pourquoi dans le seed** : Teste le workflow magic link (BC16), le rapport avec
photos avant/apres, et la validation par le CdC.

**Ce que ca teste (metier)** :

- Le magic link est-il genere et envoye correctement (72h expiry) ?
- Le prestataire peut-il remplir le rapport via la PWA sans compte ?
- Le CdC peut-il valider/rejeter le rapport ?

**Ce que ca resout (humain)** : Hassan n'a pas besoin d'un compte KoproGo — un lien
dans un email suffit. Ca reduit la friction pour les artisans.

Personas : Communaute (pas de droit de vote en AG)
=====================================================

Le SEL, le partage d'objets, les competences, les reservations et les annonces sont
ouverts a **tous les habitants et voisins**, pas seulement aux coproprietaires.

Ahmed Mansouri — Le locataire invisible
-----------------------------------------

:Email: ahmed@gmail.com
:Type: **Locataire** (lot 6A de Philippe)

**Qui** : 28 ans, developpeur web freelance. Subit les consequences des decisions
de l'AG sans pouvoir voter. Droit d'etre informe des AG (Art. 3.87 §5 6°).

**Pourquoi dans le seed** : Teste le role locataire — notification AG, SEL ouvert
aux non-proprietaires, signalement de problemes sans vote.

**Ce que ca teste (metier)** :

- Un locataire est-il notifie avant les AG (obligation legale L07) ?
- Peut-il participer au SEL et au partage d'objets ?
- Peut-il creer un ticket de maintenance ?

**Ce que ca resout (humain)** : Ahmed vit dans l'immeuble, il y contribue (SEL,
aide aux aines). L'exclure du systeme serait absurde.

Sophie Martin — La conjointe qui gere le quotidien
-----------------------------------------------------

:Email: sophie@gmail.com
:Type: **Conjointe** (de Charlie, pas sur l'acte de propriete)

**Qui** : 36 ans, compagne de Charlie. Gere les reservations, echanges SEL,
partage d'objets. A son propre compte KoproGo "membre communautaire".

**Pourquoi dans le seed** : Teste le role communautaire non-proprietaire.

**Ce que ca teste (metier)** :

- Un membre communautaire peut-il reserver, echanger, partager ?
- Les credits SEL sont-ils independants du lot ?

**Ce que ca resout (humain)** : Sophie fait vivre la communaute au quotidien.
L'exclure parce qu'elle n'est pas sur l'acte serait contreproductif.

Lucas Martin — L'adolescent qui connecte les generations
-----------------------------------------------------------

:Email: lucas.m@school.be
:Type: **Adolescent** (16 ans, fils de Charlie)

**Qui** : Offre du babysitting et de l'aide informatique pour les personnes agees
(installe WhatsApp pour Marguerite). Gagne des credits SEL.

**Pourquoi dans le seed** : Teste le SEL intergenerationnel et les credits
gagnes par un mineur.

**Ce que ca teste (metier)** :

- Un mineur peut-il avoir un compte SEL ?
- Les credits sont-ils utilisables pour du soutien scolaire ?

**Ce que ca resout (humain)** : Lucas est le lien entre les generations.
Il aide Marguerite, elle se sent moins seule. C'est le coeur du SEL.

Fatima El Amrani — La voisine invitee
----------------------------------------

:Email: fatima@gmail.com
:Type: **Voisine** (immeuble d'a cote, invitee par Alice)

**Qui** : 45 ans, habite l'immeuble voisin. Offre des cours de couture au SEL.

**Pourquoi dans le seed** : Teste le SEL ouvert au quartier (pas seulement a
l'immeuble).

**Ce que ca teste (metier)** :

- Un non-resident peut-il etre invite au SEL ?
- Les credits SEL inter-immeubles fonctionnent-ils ?

**Ce que ca resout (humain)** : Le graph social de proximite depasse les murs
de l'immeuble. C'est un reseau de quartier, pas un reseau de copropriete.

Admin plateforme
~~~~~~~~~~~~~~~~~~

:Persona: **Admin KoproGo**
:Email: admin@koprogo.com

Administrateur technique. Cree les organisations et buildings. N'intervient pas
dans la gestion de la copropriete. Teste le multi-tenant et le seed/cleanup.

Tantiemes : calculs de majorite
=================================

Avec les 10 coproprietaires presents (total present = 6640/10000 = 66.4%) :

.. list-table::
   :header-rows: 1
   :widths: 25 15 15 45

   * - Majorite
     - Seuil
     - En tantiemes
     - Exemples concrets
   * - **Absolue** (>50%)
     - >3320 pour
     - (presents, hors abstentions)
     - Alice+Bob+Charlie+Diane = 2120 ❌ / +Nadia+Marguerite+Marcel = 3270 ❌ / +Jeanne = 3560 ✅
   * - **2/3** (>=66.67%)
     - >=4427 pour
     - (presents, hors abstentions)
     - Tous sauf Philippe+Emmanuel = 3560 ❌ / +Emmanuel = 4840 ✅
   * - **4/5** (>=80%)
     - >=5312 pour
     - (presents, hors abstentions)
     - Tous sauf Philippe = 4840 ❌ / Tous = 6640 ✅
   * - **Unanimite** (100% TOTAL)
     - =10000 pour
     - **TOUS les tantiemes**
     - Impossible meme avec 10 personas (6640 < 10000) ❌

Coalitions de blocage et de vulnerabilite
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Bloc investisseur** : Philippe (1800) + Emmanuel (1280) = 3080 (46.4%)
→ Bloquent la majorite absolue a eux deux. Ne vivent pas dans l'immeuble.

**Bloc renovateur** : Marcel (450) + Alice (450) + Diane (580) = 1480 (22.3%)
→ Veulent moderniser. Insuffisants seuls.

**Bloc vulnerable** : Nadia (320) + Marguerite (380) + Jeanne (290) = 990 (14.9%)
→ Subissent sans pouvoir influencer.

**Electrons libres** : Charlie (660) + Bob (430) = 1090 (16.4%)
→ Font basculer les majorites.

Tensions recurrentes en AG
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

1. **Travaux vs charges** : Marcel veut renover, Nadia ne peut pas payer
2. **Absence vs blocage** : Philippe donne procuration "non" sans lire l'OdJ
3. **CdC non representatif** : Alice + Diane (10.3%) supervisent pour 182 lots
4. **Syndic sous pression** : Francois deborde, pas de mauvaise volonte
5. **Locataire invisible** : Ahmed subit tout mais n'a aucun pouvoir
6. **Thesaurisation manquee** : 20 ans sans fonds de reserve → 755.000 EUR de travaux

Ces tensions sont les **cas de test reels** que les scenarios BDD et E2E doivent valider.

Mapping personas ↔ workflows
================================

.. list-table::
   :header-rows: 1
   :widths: 15 85

   * - Workflow
     - Personas et pourquoi
   * - **01 Vote AG**
     - Francois (cree AG), Bob (rapport commissaire), Alice (preside). 4 majorites testees :
       absolue (budget), 2/3 (travaux facade), 4/5 (vente parking), unanimite (bloquee).
       Philippe+Emmanuel bloquent par procuration. Nadia voit l'impact financier AVANT.
   * - **02 Ticket**
     - Charlie (signale fuite), Francois (assigne), Hassan (magic link), Alice+Diane (valident)
   * - **03 AGE**
     - Marcel (initie), Alice+Charlie+Bob+Diane cosignent → seuil 1/5. Francois a 15 jours.
   * - **04 SEL**
     - Alice↔Bob (echange entre CP). Ahmed+Lucas+Sophie+Fatima (communaute ouverte).
       Marguerite offre repassage (lien social).
   * - **05 Sondage**
     - Francois (cree). Residents votent. Philippe/Emmanuel absents → mesure du desengagement.
   * - **06 Facture**
     - Gisele (saisit), Francois (soumet). Impact : Nadia 1.440 EUR, Jeanne 1.305 EUR.
       Alice+Diane (CdC) approuvent avec echelonnement.
   * - **07 Convocation**
     - Francois (envoie). Jeanne (courrier). Philippe/Emmanuel (procuration tardive).
       Ahmed (notification locataire, Art. L07).
   * - **08 Annonces**
     - Francois (travaux). Alice (SEL). Charlie (vend poussette). Marguerite (lit — lien social).
