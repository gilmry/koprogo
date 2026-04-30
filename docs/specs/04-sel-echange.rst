====================================================
Workflow 04 : Echange SEL entre coproprietaires
====================================================

:Issue: #346
:Personas: Voir `00-personas-et-seed.rst <00-personas-et-seed.rst>`_
:Acteurs: Alice Dubois (retraitee, offre cours cuisine), Bob Janssen (offre depannage informatique), Marguerite Lemaire (offre repassage), Nadia Benali (demande babysitting)
:Articles CC: Aucun — les SEL sont legaux en Belgique mais relevent du droit civil general, pas du droit de la copropriete
:Priorite: Moyenne

Resume
------

Le Systeme d'Echange Local (SEL) permet aux coproprietaires de la Residence du
Parc Royal d'echanger des services, des objets et des achats groupes en utilisant
une monnaie temporelle (1 heure = 1 credit). Le syndic n'intervient jamais dans ce
workflow : c'est un echange horizontal entre voisins.

Le SEL cree du lien entre des personnes qui ne se parleraient pas autrement :
Alice (retraitee 67 ans, 2e etage) donne des cours de cuisine a Nadia (infirmiere
32 ans, 4e etage). Bob (comptable 55 ans) depanne l'ordinateur de Marguerite
(veuve 78 ans) qui en retour propose du repassage. Ces echanges brisent l'isolement
et creent une solidarite de voisinage que les assemblees generales seules ne
produisent jamais.

Trois types d'echanges sont possibles :
Service (cours de cuisine, depannage informatique, babysitting, repassage),
ObjectLoan (pret d'outils, livres, appareils) et SharedPurchase (achats groupes
alimentaires, materiel).

Le workflow suit une machine a etats stricte avec 5 statuts :
Offered -> Requested -> InProgress -> Completed -> (Ratings mutuels).
L'annulation est possible a tout moment avant la completion.

Les soldes de credits sont mis a jour atomiquement lors de la completion :
le provider gagne des credits (+N), le requester en depense (-N). Les soldes
negatifs sont autorises (modele de confiance communautaire).

Pre-conditions legales
-----------------------

1. **SEL legal en Belgique** : Les SEL sont reconnus et non imposes si
   l'activite reste non commerciale (troc). Ne doivent pas remplacer des
   services professionnels (questions d'assurance).

2. **Meme immeuble** : Les deux parties doivent etre coproprietaires actifs
   dans le meme immeuble (building_id identique — Residence du Parc Royal).

3. **Pas d'auto-echange** : Le provider ne peut pas demander son propre
   echange (provider_id != requester_id, valide dans le domaine).

4. **Credits dans les limites** : 1 a 100 credits par echange (contrainte
   DB + validation domaine). Titre <= 255 chars, description <= 2000 chars.

5. **Solde de credits initialise** : Chaque partie doit avoir un
   ``OwnerCreditBalance`` pour le building (cree automatiquement si absent
   lors de la premiere completion).

Etapes
------

1. **Alice Dubois (provider)** — Publie une offre de cours de cuisine

   - Alice, retraitee depuis 2 ans et presidente du CdC, veut se rendre utile
     au-dela de la gestion de l'immeuble. Elle adore cuisiner et propose de
     transmettre ses recettes belges (waterzooi, carbonnade flamande).
   - Appel : ``POST /api/v1/exchanges``
   - Payload : ``{ building_id, exchange_type: "Service", title: "Cours de cuisine belge", description: "Je propose un cours de 2h pour apprendre a preparer un waterzooi ou une carbonnade. Ingredients fournis, on cuisine ensemble!", credits: 2 }``
   - Resultat : Exchange cree en statut **Offered** avec ``provider_id = alice``
   - Validation domaine : titre non vide, credits 1-100, description non vide

2. **Nadia Benali (requester)** — Demande le cours de cuisine d'Alice

   - Nadia, infirmiere de 32 ans, vient d'acheter avec son compagnon Karim.
     Elle ne connait personne dans l'immeuble et hesite a demander (syndrome
     imposteur — elle ne sait pas ce qu'elle pourrait offrir en retour). Mais le
     cours de cuisine d'Alice lui fait envie et c'est l'occasion de rencontrer
     une voisine.
   - Appel : ``POST /api/v1/exchanges/{id}/request``
   - Pre-condition : Statut doit etre **Offered**, Nadia != Alice
   - Resultat : Statut passe a **Requested**, ``requester_id = nadia``, ``requested_at`` enregistre
   - Erreur si Nadia = Alice : "Provider cannot request their own exchange"

3. **Alice (provider)** — Accepte et demarre l'echange

   - Alice est ravie de la demande de Nadia — c'est exactement le lien social
     qu'elle recherche.
   - Appel : ``POST /api/v1/exchanges/{id}/start``
   - Pre-condition : Statut doit etre **Requested**, seule Alice (provider) peut demarrer
   - Resultat : Statut passe a **InProgress**, ``started_at`` enregistre
   - Erreur si Nadia tente : "Only the provider can start the exchange"

4. **Alice ou Nadia** — Confirme la completion

   - Le cours de cuisine a eu lieu samedi apres-midi. Nadia a appris a faire
     un waterzooi et Alice a passe un moment agreable.
   - Appel : ``POST /api/v1/exchanges/{id}/complete``
   - Pre-condition : Statut doit etre **InProgress**, seul provider ou requester peut completer
   - Resultat :

     * Statut passe a **Completed**, ``completed_at`` enregistre
     * Mise a jour atomique des soldes de credits :

       - Alice : ``credits_earned += 2``, ``balance += 2``
       - Nadia : ``credits_spent += 2``, ``balance -= 2``

     * ``total_exchanges`` incremente pour les deux parties

5. **Nadia (requester)** — Note Alice

   - Appel : ``PUT /api/v1/exchanges/{id}/rate-provider``
   - Payload : ``{ rating: 5 }``
   - Pre-condition : Statut **Completed**, seule Nadia (requester) peut noter Alice (provider)
   - Validation : Rating entre 1 et 5 (inclus)

6. **Alice (provider)** — Note Nadia

   - Appel : ``PUT /api/v1/exchanges/{id}/rate-requester``
   - Payload : ``{ rating: 5 }``
   - Pre-condition : Statut **Completed**, seule Alice (provider) peut noter Nadia (requester)
   - ``has_mutual_ratings()`` retourne ``true`` apres les deux notations

**Variante : Bob depanne l'ordinateur de Marguerite**

- Bob Janssen (comptable, 55 ans) publie une offre de depannage informatique (1 credit).
  ``POST /exchanges`` avec ``{ exchange_type: "Service", title: "Depannage informatique", description: "Configuration email, mise a jour Windows, aide imprimante. 1h max.", credits: 1 }``
- Marguerite Lemaire (veuve 78 ans) demande l'aide de Bob pour configurer sa tablette.
  Son fils habite loin et elle n'ose pas l'appeler pour ca. Bob passe chez elle un samedi matin.
- Apres la completion, Marguerite a un solde de -1 credit. Elle propose alors du repassage
  (``POST /exchanges`` avec ``exchange_type: "Service", title: "Repassage soigne", credits: 1``).
  Pour Marguerite, ce n'est pas seulement un echange de services : c'est son seul lien social
  dans l'immeuble. Se sentir utile lui redonne de la dignite.

**Variante : Nadia hesite a offrir un service**

- Nadia voudrait demander du babysitting mais n'ose pas offrir de service en retour
  (syndrome imposteur). Son solde est a -2 apres le cours de cuisine d'Alice.
  Le modele de confiance du SEL (soldes negatifs autorises) lui permet de continuer
  a demander des services. Quand elle se sentira prete, elle pourra offrir des
  conseils en sante (mesure de tension, premiers secours) — son expertise d'infirmiere.

**Variante : Annulation**

- A tout moment avant **Completed**, le provider ou le requester peut annuler :
  ``POST /api/v1/exchanges/{id}/cancel`` avec ``{ reason: "Empechement" }``
- Statut passe a **Cancelled**, ``cancelled_at`` et ``cancellation_reason`` enregistres
- Aucun credit n'est transfere
- Un echange deja complete ne peut pas etre annule

Post-conditions
---------------

1. **Exchange en base** : Statut = ``Completed``, ``completed_at`` non null,
   ``provider_rating`` et ``requester_rating`` renseignes.

2. **Soldes de credits mis a jour** :

   - Alice (provider) : ``credits_earned = 2``, ``balance = 2``
   - Nadia (requester) : ``credits_spent = 2``, ``balance = -2``

3. **Compteur d'echanges incremente** : ``total_exchanges += 1`` pour les deux.

4. **Niveaux de participation** recalcules selon le total :

   - 0 = New, 1-5 = Beginner, 6-20 = Active, 21-50 = Veteran, 51+ = Expert

5. **Audit trail** : Evenements ``ExchangeCreated``, ``ExchangeRequested``,
   ``ExchangeStarted``, ``ExchangeCompleted``, ``ExchangeProviderRated``,
   ``ExchangeRequesterRated``, ``CreditBalanceUpdated`` emis (GDPR Art. 30).

6. **Statistiques immeuble** accessibles via
   ``GET /api/v1/buildings/{id}/sel-statistics`` :
   total echanges, echanges actifs/completes, credits echanges, participants
   actifs, note moyenne, type le plus populaire.

7. **Leaderboard** mis a jour : ``GET /api/v1/buildings/{id}/leaderboard``
   (top contributeurs par solde decroissant — Alice en tete avec +2).

Donnees seed requises
----------------------

.. note::

   Les personas et l'immeuble de reference sont definis dans
   `00-personas-et-seed.rst <00-personas-et-seed.rst>`_.
   Ce workflow utilise les personas suivants du seed partage :

- **Building** : Residence du Parc Royal (42 Avenue Louise, 1050 Ixelles, 182 lots, 10000 tantiemes)
- **Alice Dubois** : Lot 2A, 450/10000 tantiemes — retraitee 67 ans, presidente CdC, offre des cours de cuisine (alice@residence-parc.be)
- **Bob Janssen** : Lot 2B, 430/10000 tantiemes — comptable 55 ans, offre du depannage informatique (bob@residence-parc.be)
- **Marguerite Lemaire** : Lot 1A, 380/10000 tantiemes — veuve 78 ans, offre du repassage, seul lien social (marguerite@residence-parc.be)
- **Nadia Benali** : Lot 4A, 320/10000 tantiemes — infirmiere 32 ans, premier achat, hesite a offrir (nadia@residence-parc.be)
- **Credit balances** : Tous a 0 initialement

**Echanges pre-existants dans le seed** :

- Alice offre "Cours de cuisine belge" (Service, 2 credits, statut Offered)
- Bob offre "Depannage informatique" (Service, 1 credit, statut Offered)
- Marguerite offre "Repassage soigne" (Service, 1 credit, statut Offered)

Scenario BDD (Gherkin)
-----------------------

.. code-block:: gherkin

   Feature: SEL - Echange entre coproprietaires de la Residence du Parc Royal

     Background:
       Given le building "Residence du Parc Royal" avec 182 lots
       And Alice Dubois (retraitee 67 ans, lot 2A) avec un solde de credits de 0
       And Nadia Benali (infirmiere 32 ans, lot 4A) avec un solde de credits de 0
       And Bob Janssen (comptable 55 ans, lot 2B) avec un solde de credits de 0
       And Marguerite Lemaire (veuve 78 ans, lot 1A) avec un solde de credits de 0

     Scenario: Workflow complet - Alice donne un cours de cuisine a Nadia
       When Alice publie une offre de type "Service" intitulee "Cours de cuisine belge" pour 2 credits
       Then l'echange est cree en statut "Offered"
       And le provider est Alice

       When Nadia demande l'echange
       Then le statut passe a "Requested"
       And le requester est Nadia

       When Alice demarre l'echange
       Then le statut passe a "InProgress"

       When Alice confirme la completion apres le cours de waterzooi
       Then le statut passe a "Completed"
       And le solde de credits d'Alice est +2 (credits_earned=2, balance=2)
       And le solde de credits de Nadia est -2 (credits_spent=2, balance=-2)
       And le total_exchanges de chaque partie est 1

       When Nadia note Alice avec 5 etoiles
       Then provider_rating = 5

       When Alice note Nadia avec 5 etoiles
       Then requester_rating = 5
       And l'echange a des ratings mutuels complets

     Scenario: Bob depanne l'ordinateur de Marguerite
       When Bob publie une offre "Depannage informatique" pour 1 credit
       And Marguerite demande l'echange
       And Bob demarre l'echange
       And Bob confirme la completion
       Then le solde de Bob est +1 (credits_earned=1, balance=1)
       And le solde de Marguerite est -1 (credits_spent=1, balance=-1)

       When Marguerite publie une offre "Repassage soigne" pour 1 credit
       And Bob demande l'echange de Marguerite
       And Marguerite demarre l'echange
       And Marguerite confirme la completion
       Then le solde de Marguerite est 0 (credits_earned=1, credits_spent=1, balance=0)
       And le solde de Bob est 0 (credits_earned=1, credits_spent=1, balance=0)
       And Marguerite a un total_exchanges de 2

     Scenario: Interdiction d'auto-echange
       When Alice publie une offre "Cours de cuisine" pour 2 credits
       And Alice tente de demander son propre echange
       Then l'erreur "Provider cannot request their own exchange" est retournee

     Scenario: Seul le provider peut demarrer
       When Alice publie une offre et Nadia la demande
       And Nadia tente de demarrer l'echange
       Then l'erreur "Only the provider can start the exchange" est retournee

     Scenario: Annulation avant completion
       When Alice publie une offre et Nadia la demande
       And Nadia annule avec la raison "Garde d'enfant impossible ce samedi"
       Then le statut passe a "Cancelled"
       And la raison d'annulation est "Garde d'enfant impossible ce samedi"
       And aucun credit n'est transfere

     Scenario: Impossible d'annuler un echange complete
       Given un echange complete entre Alice et Nadia
       When Alice tente d'annuler l'echange
       Then l'erreur "Cannot cancel a completed exchange" est retournee

     Scenario: Validation des ratings (limites 1-5)
       Given un echange complete entre Alice et Nadia
       When Nadia tente de noter Alice avec 0 etoiles
       Then l'erreur "Rating must be between 1 and 5" est retournee
       When Nadia tente de noter Alice avec 6 etoiles
       Then l'erreur "Rating must be between 1 and 5" est retournee

     Scenario: Soldes negatifs autorises — Nadia peut continuer a demander
       Given Nadia a un solde de credits de -2 (apres le cours de cuisine d'Alice)
       When un echange de 1 credit est complete (Nadia = requester pour babysitting)
       Then le solde de Nadia est -3 (pas de blocage — modele de confiance)

Scenario E2E (narratif)
------------------------

**Acteurs** : Alice Dubois (retraitee, provider), Nadia Benali (infirmiere, requester), Bob Janssen (comptable), Marguerite Lemaire (veuve)

1. Alice se connecte avec le role ``Owner`` (alice@residence-parc.be) et consulte
   le marketplace SEL de la Residence du Parc Royal
   (``GET /buildings/{id}/exchanges/available``). La liste contient les offres
   existantes de Bob et Marguerite.

2. Alice cree une offre de cours de cuisine de 2 credits via
   ``POST /exchanges``. L'API retourne 201 avec l'echange en statut
   ``Offered``. Le marketplace affiche la nouvelle offre.

3. Nadia se connecte avec son role ``Owner`` (nadia@residence-parc.be) et
   consulte le marketplace. Elle voit l'offre d'Alice "Cours de cuisine belge".
   Elle demande l'echange via ``POST /exchanges/{id}/request``.
   Le statut passe a ``Requested``.

4. Alice recoit une notification (InApp) de la demande de Nadia. Elle est
   ravie — Nadia est la jeune voisine du 4e qu'elle voulait rencontrer. Alice
   accepte et demarre l'echange via ``POST /exchanges/{id}/start``.
   Le statut passe a ``InProgress``.

5. Apres le cours de waterzooi du samedi apres-midi, Alice confirme la
   completion via ``POST /exchanges/{id}/complete``. Le statut passe a
   ``Completed``. Les soldes sont mis a jour atomiquement : Alice +2, Nadia -2.

6. Nadia note Alice 5/5 via ``PUT /exchanges/{id}/rate-provider``.
   Alice note Nadia 5/5 via ``PUT /exchanges/{id}/rate-requester``.

7. Les deux consultent leurs soldes respectifs via
   ``GET /owners/{id}/buildings/{building_id}/credit-balance``.
   Alice : balance=2. Nadia : balance=-2.

8. Le leaderboard de l'immeuble (``GET /buildings/{id}/leaderboard``)
   montre Alice en premiere position, suivie de Bob et Marguerite (si leurs
   echanges ont ete completes).

9. Les statistiques SEL (``GET /buildings/{id}/sel-statistics``)
   montrent 1 echange complete, 2 credits echanges, 2 participants actifs.
   Le type le plus populaire est "Service".

10. Nadia, encouragee par cette premiere experience positive, consulte a
    nouveau le marketplace. Elle voit l'offre de depannage de Bob et l'offre
    de repassage de Marguerite. Le SEL a cree un reseau de solidarite entre
    4 personnes qui se croisaient dans l'ascenseur sans se parler.
