====================================================
Workflow 05 : Sondage de consultation entre AG
====================================================

:Issue: #346
:Personas: Voir 00-personas-et-seed.rst
:Acteurs: Francois Leroy (syndic), Alice Dubois (presidente CdC), Charlie Martin, Nadia Benali, Marguerite Lemaire, Marcel Dupont, Philippe Vandermeulen, Emmanuel Claes
:Articles CC: Art. 577-8/4 par.4 Code Civil belge (consultation entre assemblees)
:Priorite: Haute

Resume
------

Le syndic peut consulter les coproprietaires entre deux assemblees generales
via des sondages non contraignants. Contrairement aux votes en AG (resolutions
avec majorite legale), les resultats d'un sondage sont **purement
consultatifs** : ils n'ont pas force de loi et ne lient ni le syndic ni l'AG.
Les resultats doivent etre documentes dans le PV de la prochaine AG.

Quatre types de sondages sont disponibles :

- **YesNo** : Question binaire (ex: "Repeindre le hall en vert ?") — 2 options
- **MultipleChoice** : Choix entre options (ex: choix entrepreneur) — 2+ options
- **Rating** : Satisfaction 1-5 etoiles — 5 options pre-definies
- **OpenEnded** : Texte libre (ex: suggestions d'amelioration) — 0 options

Le lifecycle suit une machine a etats : Draft -> Active -> Closed/Cancelled.
Un sondage ne peut accepter des votes que lorsqu'il est **Active** et que
``starts_at <= NOW() <= ends_at``. La cloture peut etre manuelle (syndic) ou
automatique (background job quand ``ends_at`` est depasse).

Le vote anonyme est supporte : si ``is_anonymous = true``, le ``owner_id``
est stocke comme NULL et seule l'adresse IP est conservee pour audit.

Dimension humaine
------------------

Ce sondage illustre le fosse entre les **residents impliques** et les
**investisseurs absents** :

- **Alice, Charlie, Nadia, Marguerite, Marcel** vivent dans l'immeuble.
  Ils traversent le hall degrade chaque jour. Pour eux, la question n'est
  pas abstraite : c'est leur cadre de vie.
- **Philippe et Emmanuel** ne vivent pas sur place. Leurs appartements sont
  loues. Un hall repeint ne change rien a leur rendement locatif. Ils ne
  repondront meme pas au sondage.
- **Marcel** vote Oui avec enthousiasme — il veut rattraper 20 ans de
  negligence et repeindre n'est qu'un debut.
- **Marguerite** vote Oui mais s'inquiete du cout. Son livret de 12.000 EUR
  fond lentement.
- **Nadia** vote Oui malgre sa situation financiere tendue — un hall propre
  ameliore la valeur de son appartement, sa seule richesse.

Le sondage revele une realite : **le taux de participation est un indicateur
social**. 70% de reponse chez les residents, 0% chez les investisseurs.
Francois utilisera ces resultats pour argumenter en AG que la demande est
reelle et portee par ceux qui vivent dans l'immeuble.

Pre-conditions legales
-----------------------

1. **Pas de force de loi** : Art. 577-8/4 par.4 CC autorise les consultations
   entre AG, mais les resultats sont consultatifs. Seule l'AG peut prendre
   des decisions engageant la copropriete.

2. **Role syndic ou board member** : Seul le syndic ou un membre du conseil
   de copropriete peut creer et publier un sondage (``created_by`` = user
   avec role Syndic ou BoardMember).

3. **Immeuble valide** : Le sondage doit etre lie a un building existant
   dans l'organisation du createur.

4. **Date de fin future** : ``ends_at`` doit etre dans le futur au moment
   de la creation.

5. **Electeurs eligibles** : ``total_eligible_voters > 0`` — au moins un
   coproprietaire actif dans l'immeuble.

6. **Coherence options/type** :

   - YesNo : exactement 2 options
   - MultipleChoice : au moins 2 options
   - Rating : exactement 5 options (1 a 5 etoiles)
   - OpenEnded : 0 options (texte libre)

Etapes
------

1. **Francois (syndic)** — Cree un sondage (brouillon)

   - Appel : ``POST /api/v1/polls``
   - Payload :

     .. code-block:: json

        {
          "building_id": "<residence-du-parc-royal-uuid>",
          "title": "Faut-il repeindre le hall d'entree ?",
          "description": "Le hall d'entree montre des signes de degradation. Plusieurs coproprietaires ont signale des traces d'humidite et de la peinture ecaillee. Ce sondage vise a mesurer l'interet avant de mettre le sujet a l'ordre du jour de la prochaine AG.",
          "poll_type": "YesNo",
          "options": [
            { "option_text": "Oui", "display_order": 1 },
            { "option_text": "Non", "display_order": 2 }
          ],
          "is_anonymous": false,
          "ends_at": "2026-04-15T23:59:59Z"
        }

   - Resultat : Poll cree en statut **Draft**, ``total_votes_cast = 0``
   - Le sondage n'est pas visible par les coproprietaires tant qu'il n'est pas publie

2. **Francois** — Publie le sondage

   - Appel : ``PUT /api/v1/polls/{id}/publish``
   - Pre-condition : Statut doit etre **Draft**
   - Resultat : Statut passe a **Active**, ``starts_at = NOW()``
   - Le sondage est desormais visible et ouvert aux votes
   - Une notification est envoyee aux 10 coproprietaires nommes + 172 autres

3. **Alice (presidente CdC, 450 tantiemes)** — Vote en premiere

   - Appel : ``POST /api/v1/polls/{id}/vote``
   - Payload : ``{ "option_id": "uuid-de-oui" }``
   - Alice vote Oui — elle traverse le hall chaque jour et constate la degradation
   - Resultat : ``total_votes_cast = 1``, option Oui ``vote_count = 1``

4. **Charlie (660 tantiemes)** — Vote Oui

   - Charlie vote Oui — le hall degrade etait une mauvaise surprise a l'achat
   - ``total_votes_cast = 2``

5. **Nadia (320 tantiemes)** — Vote Oui malgre sa situation

   - Nadia vote Oui — un hall propre preserve la valeur de son appartement,
     sa seule richesse. Elle espere que le cout sera echelonne.
   - ``total_votes_cast = 3``

6. **Marguerite (380 tantiemes)** — Vote Oui avec inquietude

   - Marguerite vote Oui mais note mentalement qu'il faudra verifier le cout.
     Son livret de 12.000 EUR est sa seule securite.
   - ``total_votes_cast = 4``

7. **Marcel (450 tantiemes)** — Vote Oui avec enthousiasme

   - Marcel vote Oui — c'est exactement le type de travaux qu'il pousse
     depuis sa retraite. "Il etait temps !"
   - ``total_votes_cast = 5``

8. **Philippe (1800 tantiemes) et Emmanuel (1280 tantiemes)** — Ne repondent pas

   - Philippe ne lit pas l'email — ses 3 appartements sont geres par une agence.
   - Emmanuel est en deplacement — la copropriete est un investissement passif.
   - Le sondage n'a pas de force legale : leur absence ne bloque rien.
     Mais elle revele leur desengagement.

9. **Francois** — Cloture le sondage et consulte les resultats

   - Appel : ``PUT /api/v1/polls/{id}/close``
   - Pre-condition : Statut doit etre **Active**
   - Resultat : Statut passe a **Closed**

   - Consultation des resultats : ``GET /api/v1/polls/{id}/results``
   - Retourne :

     * Option gagnante : **Oui** (5 voix sur 5 votants = 100%)
     * Taux de participation : 50% (5 votants sur 10 coproprietaires nommes)
     * **Analyse sociologique** : 100% des residents ayant repondu sont favorables.
       Les 2 investisseurs absents (Philippe + Emmanuel = 30.8% des tantiemes)
       n'ont pas participe.

   - Francois documentera ces resultats dans le PV de la prochaine AG en
     soulignant que la demande est unanime parmi les residents.

**Variante : Annulation**

- Francois peut annuler un sondage Draft ou Active via
  ``PUT /api/v1/polls/{id}/cancel``
- Un sondage Closed ne peut pas etre annule

**Variante : Cloture automatique**

- Un background job verifie periodiquement les sondages Active dont
  ``ends_at <= NOW()`` et les passe automatiquement en Closed
  (``poll.auto_close_if_ended()``)

Post-conditions
---------------

1. **Poll en base** : Statut = ``Closed``, ``total_votes_cast = 5``,
   option Oui avec ``vote_count = 5``, option Non avec ``vote_count = 0``.

2. **Resultats calcules** : Option gagnante = Oui (100%), taux de
   participation = 50% (5/10 coproprietaires nommes).

3. **Votes enregistres** : 5 ``PollVote`` (Alice, Charlie, Nadia,
   Marguerite, Marcel). Si anonyme, ``owner_id = NULL`` et seule
   ``ip_address`` est conservee.

4. **Prevention doublons** : Contrainte UNIQUE ``(poll_id, owner_id)``
   empeche les votes multiples pour les sondages non anonymes.

5. **Pas de decision legale** : Les resultats n'ont aucune force executoire.
   Francois doit presenter les resultats a la prochaine AG pour ratification
   si une decision formelle est necessaire.

6. **Audit trail** : Evenements ``PollCreated``, ``PollPublished``,
   ``PollVoteCast`` (x5), ``PollClosed``, ``PollResultsCalculated`` emis
   (GDPR Art. 30).

7. **Notification implicite** : Les coproprietaires devraient recevoir une
   notification ``PollPublished`` (a implementer via le systeme de
   notifications multi-canal existant).

8. **Indicateur social** : Le taux de non-reponse des investisseurs absents
   (Philippe, Emmanuel) est un signal que Francois peut mentionner en AG
   pour illustrer le desengagement.

Donnees seed requises
----------------------

.. note::

   Ce workflow utilise le seed partage defini dans ``00-personas-et-seed.rst``.
   Building : **Residence du Parc Royal** (182 lots, 10000 tantiemes).
   Pas de seed SQL specifique a ce workflow — toutes les donnees proviennent
   du seed de reference.

Personas impliques dans le seed :

- **Francois Leroy** (syndic) — cree et publie le sondage
- **Alice Dubois** (450 tantiemes, presidente CdC) — vote Oui
- **Charlie Martin** (660 tantiemes) — vote Oui
- **Nadia Benali** (320 tantiemes) — vote Oui
- **Marguerite Lemaire** (380 tantiemes) — vote Oui
- **Marcel Dupont** (450 tantiemes) — vote Oui
- **Philippe Vandermeulen** (1800 tantiemes) — ne repond pas
- **Emmanuel Claes** (1280 tantiemes) — ne repond pas

Scenario BDD (Gherkin)
-----------------------

.. code-block:: gherkin

   Feature: Sondage de consultation entre AG

     Background:
       Given l'immeuble "Residence du Parc Royal" avec 182 lots et 10000 tantiemes
       And le syndic "Francois Leroy" responsable de l'immeuble
       And les coproprietaires Alice Dubois (450‱), Charlie Martin (660‱), Nadia Benali (320‱), Marguerite Lemaire (380‱), Marcel Dupont (450‱), Philippe Vandermeulen (1800‱), Emmanuel Claes (1280‱)

     Scenario: Workflow complet - Sondage YesNo "Repeindre le hall"
       When Francois cree un sondage YesNo "Faut-il repeindre le hall d'entree ?"
       Then le sondage est en statut "Draft"
       And le sondage n'est pas visible par les coproprietaires

       When Francois publie le sondage
       Then le statut passe a "Active"
       And le sondage est ouvert aux votes

       When Alice vote "Oui"
       Then total_votes_cast = 1
       And l'option "Oui" a vote_count = 1

       When Charlie vote "Oui"
       Then total_votes_cast = 2

       When Nadia vote "Oui"
       Then total_votes_cast = 3

       When Marguerite vote "Oui"
       Then total_votes_cast = 4

       When Marcel vote "Oui"
       Then total_votes_cast = 5

       # Philippe (1800‱) et Emmanuel (1280‱) ne repondent pas — investisseurs absents

       When Francois cloture le sondage
       Then le statut passe a "Closed"
       And les resultats montrent "Oui" = 100%, "Non" = 0%
       And le taux de participation est 50% (5/10 coproprietaires nommes)

     Scenario: Les investisseurs absents ne bloquent pas la consultation
       Given un sondage Active "Faut-il repeindre le hall d'entree ?"
       And Philippe et Emmanuel n'ont pas vote
       When Francois cloture le sondage
       Then les resultats sont valides malgre l'absence des investisseurs
       And le taux de participation reflette le desengagement (50%)

     Scenario: Vote duplique interdit
       Given un sondage Active "Faut-il repeindre le hall d'entree ?"
       And Alice a deja vote "Oui"
       When Alice tente de voter "Non"
       Then l'erreur "You have already voted on this poll" est retournee

     Scenario: Vote impossible sur sondage Draft
       Given un sondage Draft "Faut-il repeindre le hall d'entree ?"
       When Nadia tente de voter
       Then l'erreur "Poll is not currently accepting votes" est retournee

     Scenario: Seul un sondage Draft peut etre publie
       Given un sondage Active "Faut-il repeindre le hall d'entree ?"
       When Francois tente de publier le sondage
       Then l'erreur "Only draft polls can be published" est retournee

     Scenario: Annulation d'un sondage actif
       Given un sondage Active "Faut-il repeindre le hall d'entree ?"
       When Francois annule le sondage
       Then le statut passe a "Cancelled"

     Scenario: Impossible d'annuler un sondage cloture
       Given un sondage Closed "Faut-il repeindre le hall d'entree ?"
       When Francois tente d'annuler le sondage
       Then l'erreur "Cannot cancel a closed poll" est retournee

     Scenario: Cloture automatique apres date de fin
       Given un sondage Active avec ends_at dans le passe
       When le job de cloture automatique s'execute
       Then le statut passe a "Closed" automatiquement

     Scenario: Sondage MultipleChoice - Choix entrepreneur facade
       When Francois cree un sondage MultipleChoice "Quel entrepreneur pour la facade ?"
       With les options "Toitures Bruxelles", "Renov'Art SPRL", "Lejeune & Fils"
       And Francois publie le sondage
       And Alice, Diane, Marcel, Bob votent "Toitures Bruxelles"
       And Charlie, Nadia, Marguerite votent "Renov'Art SPRL"
       And Jeanne vote "Lejeune & Fils"
       When Francois cloture le sondage
       Then l'option gagnante est "Toitures Bruxelles" avec 50%
       And le taux de participation est 80% (8/10)

     Scenario: Sondage anonyme - owner_id non stocke
       When Francois cree un sondage YesNo anonyme "Etes-vous satisfait du nettoyage ?"
       And Francois publie le sondage
       And Alice vote "Oui"
       Then le vote est enregistre avec owner_id = NULL
       And l'adresse IP est conservee pour audit

Scenario E2E (narratif)
------------------------

**Acteurs** : Francois Leroy (Syndic), Alice Dubois, Charlie Martin, Nadia Benali, Marguerite Lemaire, Marcel Dupont (Coproprietaires residents), Philippe Vandermeulen, Emmanuel Claes (Investisseurs absents)

1. Francois se connecte avec le role ``Syndic`` et accede au module sondages
   de l'immeuble "Residence du Parc Royal".

2. Francois cree un sondage YesNo "Faut-il repeindre le hall d'entree ?"
   avec une description detaillant les degradations constatees et une date
   de fin a J+7 via ``POST /polls``. L'API retourne 201 avec le sondage en
   statut ``Draft``.

3. Francois publie le sondage via ``PUT /polls/{id}/publish``. Le statut
   passe a ``Active``. Une notification est envoyee aux coproprietaires.

4. Alice se connecte avec le role ``Owner`` et consulte les sondages
   actifs de son immeuble (``GET /buildings/{id}/polls/active``). En tant
   que presidente du CdC, elle vote en premiere pour donner l'exemple.

5. Alice vote "Oui" via ``POST /polls/{id}/vote``. L'API retourne 200.
   Le ``total_votes_cast`` passe a 1.

6. Alice tente de voter une seconde fois. L'API retourne 400 avec le
   message "You have already voted on this poll".

7. Charlie se connecte et vote "Oui" — le hall degrade etait une mauvaise
   surprise lors de son achat en 2021. ``total_votes_cast`` passe a 2.

8. Nadia vote "Oui" malgre son angoisse financiere — elle espere un
   echelonnement. ``total_votes_cast`` passe a 3.

9. Marguerite vote "Oui" — elle y passe chaque jour et le hall degrade
   la rend triste. ``total_votes_cast`` passe a 4.

10. Marcel vote "Oui" avec enthousiasme — "Il etait temps de s'en
    occuper !" ``total_votes_cast`` passe a 5.

11. Philippe et Emmanuel ne se connectent pas. Leurs emails restent non
    lus. Le sondage ne les concerne pas dans leur logique d'investisseur.

12. Apres 7 jours, Francois cloture le sondage via
    ``PUT /polls/{id}/close``. Le statut passe a ``Closed``.

13. Francois consulte les resultats via ``GET /polls/{id}/results``. L'API
    retourne : Oui = 100% (5/5), participation = 50% (5/10). Francois
    note que tous les residents ayant repondu sont favorables et que les
    investisseurs absents n'ont pas participe. Il documentera ces resultats
    dans le PV de la prochaine AG pour appuyer la mise a l'ordre du jour
    des travaux de peinture.

14. Francois tente d'annuler le sondage cloture via
    ``PUT /polls/{id}/cancel``. L'API retourne 400 avec
    "Cannot cancel a closed poll".
