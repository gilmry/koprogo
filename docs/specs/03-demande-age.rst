=========================================================
Workflow 03 : Demande d'Assemblee Generale Extraordinaire
=========================================================

:Issue: #346
:Personas: Voir `00-personas-et-seed.rst <00-personas-et-seed.rst>`_
:Acteurs: Marcel Dupont (initiateur), Alice Dubois (cosignataire), Charlie Martin (cosignataire), Bob Janssen (cosignataire), Diane Peeters (cosignataire), Francois Leroy (syndic)
:Articles CC: Art. 3.87 §2
:Priorite: Haute

Resume
------

Marcel Dupont (67 ans, retraite qui decouvre 20 ans de retard d'entretien) initie une
demande d'AG extraordinaire pour des travaux urgents de renovation estimes a 200.000 EUR.
D'autres coproprietaires cosignent la demande jusqu'a atteindre le seuil legal de 1/5
des quotes-parts (2000/10000 tantiemes). La demande est soumise a Francois Leroy (syndic)
qui dispose de 15 jours pour repondre. Si le syndic accepte, une AG est convoquee.
S'il ne repond pas dans le delai, les demandeurs peuvent convoquer eux-memes l'AG
(auto-convocation).

**Progression des cosignatures** :

- Marcel seul : 450/10000 = 4.5% < 20% ❌
- + Alice (presidente CdC, soutient Marcel) : 450 + 450 = 900/10000 = 9.0% < 20% ❌
- + Charlie (inquiet mais convaincu par Marcel) : 900 + 660 = 1560/10000 = 15.6% < 20% ❌
- + Bob (commissaire aux comptes) : 1560 + 430 = 1990/10000 = 19.9% < 20% ❌ (si proche!)
- + Diane (avocate, membre CdC) : 1990 + 580 = 2570/10000 = 25.7% >= 20% ✅ Seuil atteint!

Pre-conditions legales
-----------------------

- **Art. 3.87 §2 al. 1** : Tout coproprietaire peut demander au syndic de convoquer une AG.
- **Art. 3.87 §2 al. 2** : Si la demande emane de coproprietaires representant au moins 1/5 (20%) des quotes-parts, le syndic est TENU de convoquer l'AG.
- **Art. 3.87 §2 al. 2 (suite)** : Si le syndic ne donne pas suite dans les 15 jours, les demandeurs peuvent convoquer eux-memes l'AG.
- **Art. 3.87 §3** : La convocation de l'AGE doit respecter le delai de 15 jours minimum.

Etapes
------

1. **Marcel Dupont (initiateur)** — Cree une demande d'AGE (statut ``Draft``).
   - Endpoint : ``POST /buildings/:building_id/age-requests``
   - Body : ``title``: "Travaux urgents de renovation — 200.000 EUR", ``description``: "Apres 20 ans sans entretien majeur, la toiture fuit, la facade se degrade, et l'installation electrique des communs n'est plus aux normes. Il faut agir maintenant avant que les degats ne s'aggravent."
   - Methode domaine : ``AgeRequest::new(organization_id, building_id, title, description, marcel_id)``
   - Statut initial : ``Draft``
   - Le seuil legal est configure a ``threshold_pct = 0.20`` (1/5) par defaut (constante ``DEFAULT_THRESHOLD_PCT``).
   - Validation : titre non-vide (max 255 car.)

2. **Marcel** — Ouvre la demande pour signatures publiques (``Draft`` → ``Open``).
   - Endpoint : ``PUT /age-requests/:id/open``
   - Methode domaine : ``age_request.open()``
   - La demande devient visible aux coproprietaires de la Residence du Parc Royal.

3. **Marcel** — Cosigne en premier (ajoute sa propre signature avec ses 450/10000 tantiemes).
   - Endpoint : ``POST /age-requests/:id/cosign`` (body: ``owner_id``: marcel_id, ``shares_pct``: 0.045)
   - Methode domaine : ``age_request.add_cosignatory(marcel_id, 0.045)``
   - Le total passe a ``total_shares_pct = 0.045`` (4.5%), loin du seuil de 20%.

4. **Alice Dubois (presidente CdC)** — Cosigne la demande. Alice soutient Marcel car elle aussi constate le retard d'entretien.
   - Endpoint : ``POST /age-requests/:id/cosign`` (body: ``owner_id``: alice_id, ``shares_pct``: 0.045)
   - Methode domaine : ``age_request.add_cosignatory(alice_id, 0.045)``
   - ``total_shares_pct = 0.090`` (9.0%) — seuil non atteint.

5. **Charlie Martin** — Cosigne la demande. Charlie est inquiet du cout (200k EUR = charges supplementaires enormes pour son budget deja serre) mais Marcel l'a convaincu que reporter les travaux coutera encore plus cher.
   - Endpoint : ``POST /age-requests/:id/cosign`` (body: ``owner_id``: charlie_id, ``shares_pct``: 0.066)
   - Methode domaine : ``age_request.add_cosignatory(charlie_id, 0.066)``
   - ``total_shares_pct = 0.156`` (15.6%) — seuil non atteint.

6. **Bob Janssen (commissaire aux comptes)** — Cosigne la demande. Bob a verifie les chiffres et confirme que les travaux sont financierement justifies.
   - Endpoint : ``POST /age-requests/:id/cosign`` (body: ``owner_id``: bob_id, ``shares_pct``: 0.043)
   - Methode domaine : ``age_request.add_cosignatory(bob_id, 0.043)``
   - ``total_shares_pct = 0.199`` (19.9%) — **si proche mais toujours en-dessous du seuil!**

7. **Diane Peeters (avocate, membre CdC)** — Cosigne la demande. Diane, en tant qu'avocate en droit immobilier, sait que reporter ces travaux engage la responsabilite du syndic.
   - Endpoint : ``POST /age-requests/:id/cosign`` (body: ``owner_id``: diane_id, ``shares_pct``: 0.058)
   - Methode domaine : ``age_request.add_cosignatory(diane_id, 0.058)``
   - ``total_shares_pct = 0.257`` (25.7%) → ``total_shares_pct >= threshold_pct`` → **seuil atteint!**
   - Transition automatique : ``Open`` → ``Reached``
   - ``threshold_reached = true``, ``threshold_reached_at`` renseigne.

8. **Marcel (initiateur)** — Soumet formellement la demande a Francois Leroy (``Reached`` → ``Submitted``).
   - Endpoint : ``POST /age-requests/:id/submit``
   - Methode domaine : ``age_request.submit_to_syndic()``
   - ``submitted_to_syndic_at`` est renseigne.
   - ``syndic_deadline_at`` = ``submitted_to_syndic_at + 15 jours`` (constante ``SYNDIC_DEADLINE_DAYS = 15``).
   - Le delai de 15 jours pour Francois demarre.

9. **Francois Leroy (syndic)** — Repond a la demande dans les 15 jours.
   - **Cas A — Acceptation** :
     - Endpoint : ``POST /age-requests/:id/accept`` (body: ``notes``: "AGE convoquee pour le 15 mai 2026. Trois devis de couvreurs seront presentes.")
     - Methode domaine : ``age_request.accept_by_syndic(Some("AGE convoquee pour le 15 mai 2026"))``
     - Statut → ``Accepted`` (terminal).
     - ``syndic_response_at`` renseigne, ``syndic_notes`` renseigne.
     - Francois cree ensuite la reunion AGE (workflow meeting).
   - **Cas B — Rejet** :
     - Endpoint : ``POST /age-requests/:id/reject`` (body: ``reason``: "Les travaux ne sont pas urgents au sens de l'Art. 3.89 §5 2°")
     - Methode domaine : ``age_request.reject_by_syndic(reason)``
     - Validation : motif obligatoire (non-vide).
     - Statut → ``Rejected`` (terminal).
     - Diane (avocate) pourra contester ce rejet car 5 coproprietaires representant 25.7% ont signe.

10. **Systeme (cron/background)** — Si Francois ne repond pas sous 15 jours (``Submitted`` → ``Expired``).
    - Verification : ``age_request.is_deadline_expired() == true``
    - Methode domaine : ``age_request.trigger_auto_convocation()``
    - Validation : le delai doit effectivement etre depasse (``Utc::now() > syndic_deadline_at``).
    - Statut → ``Expired`` (terminal).
    - ``auto_convocation_triggered = true``
    - Marcel et les cosignataires peuvent desormais convoquer eux-memes l'AG.

Post-conditions
---------------

- **Cas Acceptation** : La demande est en statut ``Accepted``, ``syndic_response_at`` renseigne, une reunion AGE est creee et liee via ``meeting_id``.
- **Cas Rejet** : La demande est en statut ``Rejected``, ``syndic_notes`` contient le motif de Francois.
- **Cas Expiration** : La demande est en statut ``Expired``, ``auto_convocation_triggered = true``, Marcel et les cosignataires creent eux-memes l'AGE.
- Les 5 cosignataires sont traces avec leur ``shares_pct`` et ``signed_at`` : Marcel (4.5%), Alice (4.5%), Charlie (6.6%), Bob (4.3%), Diane (5.8%).
- ``total_shares_pct = 0.257`` (25.7% >= seuil de 20%).
- Evenements d'audit emis selon le cas.

Donnees seed requises
----------------------

.. note::

   Les personas et l'immeuble de reference sont definis dans
   `00-personas-et-seed.rst <00-personas-et-seed.rst>`_.
   Ce workflow utilise les personas suivants du seed partage :

- **Building** : Residence du Parc Royal (42 Avenue Louise, 1050 Ixelles, 182 lots, 10000 tantiemes)
- **Marcel Dupont** : Lot 4B, 450/10000 tantiemes — initiateur de la demande d'AGE (marcel@residence-parc.be)
- **Alice Dubois** : Lot 2A, 450/10000 tantiemes — presidente CdC, cosignataire (alice@residence-parc.be)
- **Charlie Martin** : Lot 3B, 660/10000 tantiemes — jeune couple sous pression, cosignataire (charlie@residence-parc.be)
- **Bob Janssen** : Lot 2B, 430/10000 tantiemes — commissaire aux comptes, cosignataire (bob@residence-parc.be)
- **Diane Peeters** : Lot 3A, 580/10000 tantiemes — avocate, membre CdC, cosignataire decisif (diane@residence-parc.be)
- **Francois Leroy** : Syndic professionnel (francois@syndic-leroy.be) — a 15 jours pour repondre
- **AGE Request** : titre "Travaux urgents de renovation — 200.000 EUR", statut ``Open``, creee par Marcel, 0 cosignataires (signatures a collecter dans le test)

**Tantiemes cumules pour le test du seuil** :

.. list-table::
   :header-rows: 1
   :widths: 20 15 15 15 35

   * - Cosignataire
     - Tantiemes
     - shares_pct
     - Cumul
     - Seuil 1/5 (2000/10000)
   * - Marcel Dupont
     - 450
     - 0.045
     - 450 (4.5%)
     - ❌ Non atteint
   * - + Alice Dubois
     - 450
     - 0.045
     - 900 (9.0%)
     - ❌ Non atteint
   * - + Charlie Martin
     - 660
     - 0.066
     - 1560 (15.6%)
     - ❌ Non atteint
   * - + Bob Janssen
     - 430
     - 0.043
     - 1990 (19.9%)
     - ❌ Si proche!
   * - + Diane Peeters
     - 580
     - 0.058
     - 2570 (25.7%)
     - ✅ **Seuil atteint**

Scenario BDD (Gherkin)
-----------------------

.. code-block:: gherkin

   Feature: Demande d'AG Extraordinaire (Art. 3.87 §2 CC)

     Background:
       Given le building "Residence du Parc Royal" avec 182 lots et 10000 tantiemes
       And Marcel Dupont possede le lot 4B avec 450/10000 tantiemes (4.5%)
       And Alice Dubois possede le lot 2A avec 450/10000 tantiemes (4.5%)
       And Charlie Martin possede le lot 3B avec 660/10000 tantiemes (6.6%)
       And Bob Janssen possede le lot 2B avec 430/10000 tantiemes (4.3%)
       And Diane Peeters possede le lot 3A avec 580/10000 tantiemes (5.8%)
       And Francois Leroy est syndic de l'immeuble

     Scenario: Parcours complet — collecte progressive des signatures puis acceptation par Francois
       Given Marcel cree une demande d'AGE "Travaux urgents de renovation — 200.000 EUR"
       And la demande est en statut "Draft"
       When Marcel ouvre la demande pour signatures
       Then la demande est en statut "Open"

       When Marcel cosigne avec 4.5% des quotes-parts (450/10000)
       Then total_shares_pct est 0.045
       And le seuil 1/5 n'est pas atteint
       And la demande est en statut "Open"

       When Alice cosigne avec 4.5% des quotes-parts (450/10000)
       Then total_shares_pct est 0.090
       And le seuil 1/5 n'est pas atteint

       When Charlie cosigne avec 6.6% des quotes-parts (660/10000)
       Then total_shares_pct est 0.156
       And le seuil 1/5 n'est pas atteint

       When Bob cosigne avec 4.3% des quotes-parts (430/10000)
       Then total_shares_pct est 0.199
       And le seuil 1/5 n'est pas atteint (si proche — 19.9%!)

       When Diane cosigne avec 5.8% des quotes-parts (580/10000)
       Then total_shares_pct est 0.257
       And le seuil 1/5 est atteint (25.7% >= 20%)
       And la demande est en statut "Reached"
       And threshold_reached_at est renseigne

       When Marcel soumet la demande a Francois
       Then la demande est en statut "Submitted"
       And submitted_to_syndic_at est renseigne
       And syndic_deadline_at est dans 15 jours

       When Francois accepte la demande avec notes "AGE convoquee pour le 15 mai 2026"
       Then la demande est en statut "Accepted"
       And syndic_response_at est renseigne
       And syndic_notes est "AGE convoquee pour le 15 mai 2026"

     Scenario: Rejet par Francois avec motif obligatoire
       Given une demande d'AGE de Marcel soumise a Francois (statut "Submitted")
       When Francois rejette sans motif
       Then une erreur "motif de refus est obligatoire" est retournee

       When Francois rejette avec motif "Les travaux ne sont pas urgents"
       Then la demande est en statut "Rejected"
       And syndic_notes contient "pas urgents"

     Scenario: Auto-convocation apres expiration du delai de Francois
       Given une demande d'AGE de Marcel soumise a Francois il y a 16 jours
       When le systeme verifie les demandes expirees
       Then is_deadline_expired retourne true

       When le systeme declenche l'auto-convocation
       Then la demande est en statut "Expired"
       And auto_convocation_triggered est true

     Scenario: Auto-convocation impossible avant expiration
       Given une demande d'AGE de Marcel soumise a Francois il y a 10 jours
       When le systeme tente de declencher l'auto-convocation
       Then une erreur "delai syndic n'est pas encore depasse" est retournee

     Scenario: Retrait de Bob fait perdre le seuil
       Given une demande d'AGE avec Marcel (4.5%), Alice (4.5%), Charlie (6.6%), Bob (4.3%) et Diane (5.8%) comme cosignataires
       And la demande est en statut "Reached" (total 25.7%)
       When Bob retire sa signature
       Then total_shares_pct est 0.214
       And le seuil 1/5 est toujours atteint (21.4% >= 20%)

       When Charlie retire aussi sa signature
       Then total_shares_pct est 0.148
       And le seuil 1/5 n'est plus atteint (14.8% < 20%)
       And la demande retombe en statut "Open"

     Scenario: Doublon de signature refuse
       Given une demande d'AGE de Marcel en statut "Open"
       And Marcel a deja cosigne
       When Marcel tente de cosigner a nouveau
       Then une erreur "a deja signe cette demande" est retournee

     Scenario: Retrait par l'initiateur uniquement
       Given une demande d'AGE en statut "Open" creee par Marcel
       When Alice tente de retirer la demande
       Then une erreur "Seul l'initiateur peut retirer" est retournee

       When Marcel retire la demande
       Then la demande est en statut "Withdrawn"

     Scenario: Impossible de retirer une demande terminale
       Given une demande d'AGE en statut "Accepted" (acceptee par Francois)
       When Marcel tente de retirer la demande
       Then une erreur "Impossible de retirer" est retournee

     Scenario: Soumission impossible sans atteindre le seuil
       Given une demande d'AGE en statut "Open" avec Marcel (4.5%) et Alice (4.5%) = 9.0%
       When Marcel tente de soumettre a Francois
       Then une erreur "doit etre en statut Reached" est retournee

     Scenario: Progression vers le seuil
       Given une demande d'AGE de Marcel en statut "Open" sans cosignataires
       Then calculate_progress_percentage retourne 0.0
       And shares_pct_missing retourne 0.20

       When Marcel cosigne avec 4.5% des quotes-parts
       Then calculate_progress_percentage retourne 22.5
       And shares_pct_missing retourne 0.155

       When Alice cosigne avec 4.5% des quotes-parts
       Then calculate_progress_percentage retourne 45.0
       And shares_pct_missing retourne 0.110

Scenario E2E (narratif)
------------------------

**Acteurs** : Marcel Dupont (initiateur), Alice Dubois, Charlie Martin, Bob Janssen, Diane Peeters (cosignataires), Francois Leroy (syndic)

1. ``humanLogin(marcel)`` → obtient un JWT avec role ``owner`` (marcel@residence-parc.be)
2. ``POST /buildings/:building_id/age-requests`` (body: title="Travaux urgents de renovation — 200.000 EUR", description="Apres 20 ans sans entretien majeur...") → 201 Created, recupere ``age_request_id``
3. ``GET /age-requests/:age_request_id`` → verifie statut "Draft", total_shares_pct=0.0
4. ``PUT /age-requests/:age_request_id/open`` → statut passe a "Open"
5. ``POST /age-requests/:age_request_id/cosign`` (body: owner_id=marcel_id, shares_pct=0.045) → 200 OK, total_shares_pct=0.045, seuil non atteint
6. ``humanLogin(alice)`` → obtient un JWT (alice@residence-parc.be)
7. ``POST /age-requests/:age_request_id/cosign`` (body: owner_id=alice_id, shares_pct=0.045) → 200 OK, total_shares_pct=0.090, seuil non atteint
8. ``humanLogin(charlie)`` → obtient un JWT (charlie@residence-parc.be)
9. ``POST /age-requests/:age_request_id/cosign`` (body: owner_id=charlie_id, shares_pct=0.066) → 200 OK, total_shares_pct=0.156, seuil non atteint
10. ``humanLogin(bob)`` → obtient un JWT (bob@residence-parc.be)
11. ``POST /age-requests/:age_request_id/cosign`` (body: owner_id=bob_id, shares_pct=0.043) → 200 OK, total_shares_pct=0.199, seuil non atteint (19.9%!)
12. ``humanLogin(diane)`` → obtient un JWT (diane@residence-parc.be)
13. ``POST /age-requests/:age_request_id/cosign`` (body: owner_id=diane_id, shares_pct=0.058) → 200 OK, total_shares_pct=0.257, **seuil atteint**, statut="Reached"
14. ``GET /age-requests/:age_request_id`` → verifie : status=Reached, threshold_reached=true, cosignatories.length=5
15. ``humanLogin(marcel)``
16. ``POST /age-requests/:age_request_id/submit`` → statut passe a "Submitted", syndic_deadline_at dans 15 jours
17. ``humanLogin(francois)`` → obtient un JWT avec role ``syndic`` (francois@syndic-leroy.be)
18. ``GET /buildings/:building_id/age-requests`` → la demande de Marcel apparait dans la liste, statut "Submitted"
19. ``POST /age-requests/:age_request_id/accept`` (body: notes="AGE convoquee pour le 15 mai 2026. Trois devis seront presentes.") → statut passe a "Accepted"
20. ``GET /age-requests/:age_request_id`` → verifie : status=Accepted, syndic_response_at non-null, syndic_notes contient "15 mai 2026"

**Variante E2E : Expiration du delai de Francois**

1. Reprendre les etapes 1 a 16 ci-dessus (demande de Marcel soumise a Francois)
2. [Simuler passage de 16 jours — en test, manipuler syndic_deadline_at dans la base]
3. ``GET /age-requests/:age_request_id`` → verifie is_deadline_expired=true
4. ``humanLogin(marcel)``
5. ``POST /age-requests/:age_request_id`` (trigger auto-convocation via cron/endpoint admin) → statut passe a "Expired", auto_convocation_triggered=true
6. Marcel et les cosignataires peuvent creer une meeting AGE eux-memes via ``POST /meetings``
