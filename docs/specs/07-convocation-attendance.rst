=================================================
Workflow 07 : Convocation d'AG et suivi de presence
=================================================

:Issue: #346
:Personas: Voir 00-personas-et-seed.rst
:Acteurs: Francois Leroy (syndic envoie), Alice/Bob/Charlie/Diane/Marcel/Nadia/Marguerite (email), Jeanne Devos (courrier recommande), Emmanuel/Philippe (procuration), Ahmed Mansouri (locataire notifie)
:Articles CC: Art. 3.87 S3 (delai 15j), Art. 3.87 S5 (2e convocation sans quorum), Art. 3.87 S5 6° (notification locataires), Art. 3.87 S7 (procurations max 3)
:Priorite: Haute

Resume
------

Francois Leroy (syndic) cree une convocation pour l'assemblee generale
ordinaire annuelle, programme son envoi dans le respect du delai legal
de 15 jours (Art. 3.87 S3 CC), et l'envoie aux coproprietaires.

La realite humaine derriere cette convocation :

- **Alice, Bob, Charlie, Diane, Marcel, Nadia, Marguerite** recoivent
  l'email — ils ont tous donne leur consentement electronique prealable.
- **Jeanne Devos** (82 ans, pension minimum) n'a pas d'email. Elle recoit
  un **courrier recommande** — cout supplementaire pour la copropriete mais
  obligation legale.
- **Emmanuel et Philippe** (investisseurs absents, 30.8% des tantiemes a
  eux deux) recoivent l'email mais **ne le liront probablement pas**. Ils
  donneront une procuration a la derniere minute — souvent "non a tout"
  par defaut.
- **Ahmed Mansouri** (locataire de Philippe, lot 6A) devrait etre notifie
  de l'AG et peut soumettre des observations (Art. 3.87 S5 6°), mais n'a
  pas de droit de vote.

Des rappels automatiques J-3 sont envoyes aux destinataires n'ayant pas
ouvert l'email. Le systeme suit en temps reel les taux d'ouverture et de
presence annoncee.

Dimension humaine — Qui repond, qui ignore
--------------------------------------------

Le taux d'ouverture de la convocation est un miroir de l'engagement :

.. list-table:: Comportement attendu par persona
   :header-rows: 1
   :widths: 20 12 68

   * - Persona
     - Tantiemes
     - Comportement
   * - **Alice Dubois**
     - 450
     - Ouvre immediatement, confirme WillAttend. Presidente CdC, elle prepare
       l'AG depuis des semaines.
   * - **Bob Janssen**
     - 430
     - Ouvre rapidement, confirme WillAttend. En tant que commissaire aux
       comptes, il doit presenter le rapport financier.
   * - **Charlie Martin**
     - 660
     - Ouvre dans les 24h, confirme WillAttend. Stresse par l'OdJ (travaux
       toiture = appel de fonds) mais sait qu'il doit etre present.
   * - **Diane Peeters**
     - 580
     - Ouvre immediatement, confirme WillAttend. Verifie que la convocation
       respecte le delai legal de 15 jours.
   * - **Marcel Dupont**
     - 450
     - Ouvre dans les 48h, confirme WillAttend. Impatient de discuter les
       travaux qu'il pousse depuis sa retraite.
   * - **Nadia Benali**
     - 320
     - Ouvre dans les 24h, confirme WillAttend. Angoissee par le point
       "appel de fonds" a l'OdJ.
   * - **Marguerite Lemaire**
     - 380
     - Ouvre avec l'aide de sa petite-fille, confirme WillAttend. Ne comprend
       pas bien les termes techniques de l'OdJ.
   * - **Jeanne Devos**
     - 290
     - Recoit par courrier recommande (pas d'email). Confirme via telephone
       a Francois. Sera presente mais ne comprendra pas tout.
   * - **Emmanuel Claes**
     - 1280
     - Email non ouvert pendant 5 jours. Donne procuration "non a tout"
       a Philippe la veille de l'AG.
   * - **Philippe Vandermeulen**
     - 1800
     - Email ouvert mais ne confirme pas. Donnera procuration a son avocat
       avec instructions de voter non aux travaux.
   * - **Ahmed Mansouri**
     - 0 (locataire)
     - Devrait etre notifie (Art. 3.87 S5 6°) mais n'a pas de vote.
       Peut soumettre des observations ecrites.

Pre-conditions
--------------

1. Le building "Residence du Parc Royal" existe avec au moins les 10
   coproprietaires nommes + 172 lots non attribues.
2. Un meeting existe (type Ordinary) avec une date >= J+16 (pour permettre
   le delai legal de 15 jours).
3. Francois Leroy est authentifie avec le role ``syndic``.
4. Les owners ont une adresse email valide (contient ``@``) sauf Jeanne
   Devos qui recevra un courrier recommande.
5. Pour une 2e convocation (Art. 3.87 S5 CC) : la 1ere AG existe et la 2e
   AG est programmee >= 15 jours apres la 1ere.
6. Ahmed Mansouri est enregistre comme locataire du lot 6A de Philippe.

Etapes
------

**Etape 1 — Francois cree la convocation (Draft)**

- Francois appelle ``POST /convocations`` avec :
  ``building_id``, ``meeting_id``, ``meeting_type = ordinary``,
  ``meeting_date``, ``language = FR``.
- Le domain calcule ``minimum_send_date = meeting_date - 15 jours`` (Art. 3.87 S3 CC).
- Si ``minimum_send_date < now``, la creation echoue (``Meeting date too soon``).
- La convocation est creee en statut **Draft**.
- Audit event : ``ConvocationCreated``.

**Etape 2 — Francois programme l'envoi (Draft -> Scheduled)**

- Francois appelle ``PUT /convocations/:id/schedule`` avec une ``send_date``
  fixee a J-20 (5 jours avant le minimum legal, par prudence).
- La ``send_date`` doit etre <= ``minimum_send_date``.
- Le statut passe a **Scheduled**.
- Audit event : ``ConvocationScheduled``.

.. note::
   L'etape 2 est optionnelle. Francois peut passer directement du Draft
   a l'envoi (etape 3).

**Etape 3 — Envoi de la convocation (Draft/Scheduled -> Sent)**

- Francois appelle ``POST /convocations/:id/send``.
- Le systeme genere un PDF de convocation et cree un ``ConvocationRecipient``
  pour chaque coproprietaire actif du building :

  - **Email** : Alice, Bob, Charlie, Diane, Marcel, Nadia, Marguerite,
    Emmanuel, Philippe (+ 172 autres coproprietaires)
  - **Courrier recommande** : Jeanne Devos (pas d'email)
  - **Notification locataire** : Ahmed Mansouri (Art. 3.87 S5 6°, pas de
    vote mais droit d'information)

- ``total_recipients`` est mis a jour, ``actual_send_date`` est enregistre.
- Le statut passe a **Sent**.
- Audit event : ``ConvocationSent``.

**Etape 4 — Les coproprietaires recoivent et ouvrent l'email**

- Alice et Diane ouvrent immediatement (< 1h). Bob ouvre dans les 2h.
- Charlie et Nadia ouvrent dans les 24h.
- Marcel ouvre dans les 48h.
- Marguerite ouvre avec l'aide de sa petite-fille (J+3).
- Emmanuel n'ouvre pas (J+5 sans ouverture → candidat au rappel).
- Philippe ouvre mais ne confirme pas sa presence.

- Le tracking pixel declenche ``PUT /convocation-recipients/:id/email-opened``.
- ``email_opened_at`` est enregistre (idempotent si deja ouvert).
- Le compteur ``opened_count`` de la convocation est incremente.
- Le taux d'ouverture est calcule : ``(opened_count / total_recipients) * 100``.

**Etape 5 — Les coproprietaires confirment leur presence**

- Alice confirme **WillAttend** — elle presidera l'AG.
- Bob confirme **WillAttend** — il presentera le rapport financier.
- Charlie confirme **WillAttend** — inquiet mais present.
- Diane confirme **WillAttend** — verifiera les majorites.
- Marcel confirme **WillAttend** — impatient de voter les travaux.
- Nadia confirme **WillAttend** — angoissee par l'appel de fonds.
- Marguerite confirme **WillAttend** — sa petite-fille l'accompagnera.
- Jeanne confirme par telephone aupres de Francois (pas d'email) — sera presente.
- Emmanuel et Philippe ne confirment pas.

- Chaque confirmation via ``PUT /convocation-recipients/:id/attendance``
  avec ``WillAttend`` ou ``WillNotAttend``.
- Workflow de presence :

.. code-block:: text

   Pending --> WillAttend --> Attended (apres AG)
     |
     '--> WillNotAttend --> DidNotAttend (apres AG)

- Les statuts ``Attended`` et ``DidNotAttend`` sont finaux (post-AG).
- Audit event : ``ConvocationAttendanceUpdated``.

**Etape 6 — Procurations de derniere minute (Art. 3.87 S7 CC)**

- **Emmanuel** (1280 tantiemes) donne procuration a **Philippe** la veille
  de l'AG, avec instruction "voter non a tout".
  ``PUT /convocation-recipients/:id/proxy`` avec ``proxy_owner_id = philippe_id``

- **Philippe** (1800 tantiemes) donne procuration a son avocat (non
  coproprietaire — scenario a verifier legalement).

- Regles metier :

  - Impossible de se deleguer a soi-meme.
  - Maximum 3 procurations par mandataire (Art. 3.87 S7 CC),
    sauf si le total des voix representees < 10% du total.
  - Le syndic (Francois) ne peut pas etre mandataire (Art. 3.89 S9).
  - Philippe avec la procuration d'Emmanuel represente 1800 + 1280 =
    **3080 tantiemes (30.8%)** — pouvoir de blocage sur toute majorite qualifiee.

- Audit event : ``ConvocationProxySet``.

**Etape 7 — Rappels automatiques J-3**

- Le systeme verifie ``should_send_reminder()`` : convocation Sent +
  meeting dans <= 3 jours.
- ``POST /convocations/:id/reminders`` envoie un email de rappel a
  Emmanuel (email non ouvert).
- ``reminder_sent_at`` est enregistre.
- Audit event : ``ConvocationReminderSent``.

**Etape 8 — Notification locataire Ahmed (Art. 3.87 S5 6°)**

- Ahmed Mansouri (locataire lot 6A de Philippe) recoit une notification
  l'informant de la tenue de l'AG.
- Ahmed peut soumettre des observations ecrites au syndic avant l'AG.
- Ahmed n'a **aucun droit de vote** mais doit etre informe des decisions
  qui impactent sa jouissance du bien (travaux parties communes, etc.).

**Etape 9 (optionnelle) — Annulation**

- Francois appelle ``PUT /convocations/:id/cancel``.
- Le statut passe a **Cancelled**.
- Audit event : ``ConvocationCancelled``.

**Etape 10 (2e convocation) — Quorum non atteint**

- Si le quorum n'est pas atteint lors de la 1ere AG (les 10 personas
  nommes representent seulement 66.4% — suffisant en theorie, mais si
  Philippe et Emmanuel ne viennent pas : 66.4% - 30.8% = 35.6% < 50%),
  Francois cree une 2e convocation via ``new_second_convocation()`` :

  - La date de la 2e AG doit etre >= 15 jours apres la 1ere.
  - ``no_quorum_required = true`` (Art. 3.87 S5 CC).
  - ``first_meeting_id`` relie a la 1ere AG.

Post-conditions
---------------

1. La convocation est en statut **Sent** (ou **Cancelled** si annulee).
2. Chaque recipient a un ``email_sent_at`` non nul (sauf Jeanne → courrier).
3. Le ``respects_legal_deadline()`` retourne ``true`` (envoi <= J-15).
4. Les tracking counts sont a jour (``opened_count``, ``will_attend_count``,
   ``will_not_attend_count``).
5. Les recipients avec procuration ont un ``proxy_owner_id`` non nul
   (Emmanuel → Philippe).
6. Un rappel J-3 a ete envoye a Emmanuel (email non ouvert).
7. Ahmed (locataire) a ete notifie de l'AG (Art. 3.87 S5 6°).

Donnees seed requises
----------------------

.. note::

   Ce workflow utilise le seed partage defini dans ``00-personas-et-seed.rst``.
   Building : **Residence du Parc Royal** (182 lots, 10000 tantiemes).

Donnees specifiques au workflow :

.. code-block:: sql

   -- Meeting (AG ordinaire dans 20 jours)
   INSERT INTO meetings (id, organization_id, building_id, title, meeting_date, meeting_type)
   VALUES ('m0700000-0000-0000-0000-000000000001', 'org00000-0000-0000-0000-000000000001',
           '<residence-du-parc-royal-uuid>',
           'AG Ordinaire 2026', NOW() + INTERVAL '20 days', 'ordinary');

   -- Convocation Sent avec recipients
   INSERT INTO convocations (id, organization_id, building_id, meeting_id, meeting_type,
       meeting_date, status, minimum_send_date, actual_send_date,
       pdf_file_path, language, total_recipients, opened_count,
       will_attend_count, will_not_attend_count, created_by)
   VALUES ('c0700000-0000-0000-0000-000000000001', 'org00000-0000-0000-0000-000000000001',
           '<residence-du-parc-royal-uuid>', 'm0700000-0000-0000-0000-000000000001',
           'ordinary', NOW() + INTERVAL '20 days', 'sent',
           NOW() + INTERVAL '5 days', NOW(),
           '/uploads/convocations/ag-ordinaire-2026.pdf', 'FR', 10, 8, 8, 0,
           '<francois-user-id>');

   -- Recipients (10 coproprietaires nommes)
   INSERT INTO convocation_recipients (id, convocation_id, owner_id, email,
       email_sent_at, email_opened_at, attendance_status) VALUES
   -- Alice : email ouvert, WillAttend (presidente CdC)
   ('cr070000-0000-0000-0000-000000000001', 'c0700000-0000-0000-0000-000000000001',
    '<alice-owner-id>', 'alice@residence-parc.be', NOW(), NOW(), 'will_attend'),
   -- Bob : email ouvert, WillAttend (commissaire aux comptes)
   ('cr070000-0000-0000-0000-000000000002', 'c0700000-0000-0000-0000-000000000001',
    '<bob-owner-id>', 'bob@residence-parc.be', NOW(), NOW(), 'will_attend'),
   -- Charlie : email ouvert, WillAttend (stresse par l'OdJ)
   ('cr070000-0000-0000-0000-000000000003', 'c0700000-0000-0000-0000-000000000001',
    '<charlie-owner-id>', 'charlie@residence-parc.be', NOW(), NOW(), 'will_attend'),
   -- Diane : email ouvert, WillAttend (verifie les majorites)
   ('cr070000-0000-0000-0000-000000000004', 'c0700000-0000-0000-0000-000000000001',
    '<diane-owner-id>', 'diane@residence-parc.be', NOW(), NOW(), 'will_attend'),
   -- Marcel : email ouvert, WillAttend (impatient)
   ('cr070000-0000-0000-0000-000000000005', 'c0700000-0000-0000-0000-000000000001',
    '<marcel-owner-id>', 'marcel@residence-parc.be', NOW(), NOW(), 'will_attend'),
   -- Nadia : email ouvert, WillAttend (angoissee)
   ('cr070000-0000-0000-0000-000000000006', 'c0700000-0000-0000-0000-000000000001',
    '<nadia-owner-id>', 'nadia@residence-parc.be', NOW(), NOW(), 'will_attend'),
   -- Marguerite : email ouvert (aide petite-fille), WillAttend
   ('cr070000-0000-0000-0000-000000000007', 'c0700000-0000-0000-0000-000000000001',
    '<marguerite-owner-id>', 'marguerite@residence-parc.be', NOW(), NOW(), 'will_attend'),
   -- Jeanne : PAS D'EMAIL — courrier recommande, confirme par telephone
   ('cr070000-0000-0000-0000-000000000008', 'c0700000-0000-0000-0000-000000000001',
    '<jeanne-owner-id>', NULL, NOW(), NULL, 'will_attend'),
   -- Emmanuel : email NON ouvert — procuration a Philippe
   ('cr070000-0000-0000-0000-000000000009', 'c0700000-0000-0000-0000-000000000001',
    '<emmanuel-owner-id>', 'emmanuel@residence-parc.be', NOW(), NULL, 'pending'),
   -- Philippe : email ouvert mais PAS de confirmation
   ('cr070000-0000-0000-0000-000000000010', 'c0700000-0000-0000-0000-000000000001',
    '<philippe-owner-id>', 'philippe@residence-parc.be', NOW(), NOW(), 'pending');

   -- Procuration d'Emmanuel vers Philippe
   UPDATE convocation_recipients
   SET proxy_owner_id = '<philippe-owner-id>'
   WHERE id = 'cr070000-0000-0000-0000-000000000009';

BDD
---

.. code-block:: gherkin

   Feature: Convocation d'AG et suivi de presence

     Background:
       Given l'immeuble "Residence du Parc Royal" avec 182 lots et 10000 tantiemes
       And le syndic "Francois Leroy"
       And l'AG Ordinaire 2026 programmee dans 20 jours
       And les coproprietaires : Alice (450‱), Bob (430‱), Charlie (660‱), Diane (580‱), Marcel (450‱), Nadia (320‱), Marguerite (380‱), Jeanne (290‱), Emmanuel (1280‱), Philippe (1800‱)
       And le locataire Ahmed Mansouri (lot 6A de Philippe)

     Scenario: Francois cree une convocation
       When Francois cree une convocation pour l'AG en langue "FR"
       Then the convocation status is "Draft"
       And the minimum_send_date is meeting_date minus 15 days

     Scenario: Francois programme l'envoi 20 jours avant
       Given a convocation in status "Draft"
       When Francois programme l'envoi pour aujourd'hui
       Then the convocation status is "Scheduled"

     Scenario: Programmation trop tardive echoue (10 jours au lieu de 15)
       Given a convocation in status "Draft"
       When Francois programme l'envoi a meeting_date minus 10 days
       Then I receive an error "after minimum send date"

     Scenario: Francois envoie la convocation — canaux differencies
       Given a convocation in status "Draft"
       When Francois envoie la convocation
       Then the convocation status is "Sent"
       And total_recipients is 10
       And a PDF file is generated
       And Alice, Bob, Charlie, Diane, Marcel, Nadia, Marguerite, Emmanuel, Philippe recoivent un email
       And Jeanne recoit un courrier recommande (pas d'email)
       And Ahmed (locataire) recoit une notification d'information

     Scenario: Meeting trop proche echoue (5 jours < 15 jours legaux)
       When Francois tente de creer une convocation pour une AG dans 5 jours
       Then I receive an error "Meeting date too soon"

     Scenario: Alice ouvre l'email en premiere (presidente CdC)
       Given a sent convocation with 10 recipients
       When Alice ouvre l'email de convocation
       Then email_opened_at is recorded for Alice
       And opened_count is 1

     Scenario: Alice confirme sa presence (presidera l'AG)
       Given a sent convocation with 10 recipients
       When Alice confirme sa presence "WillAttend"
       Then will_attend_count is incremented

     Scenario: Charlie confirme malgre son stress financier
       Given a sent convocation
       When Charlie confirme "WillAttend"
       Then will_attend_count is incremented
       # Charlie est stresse par le point "appel de fonds" a l'OdJ

     Scenario: Jeanne confirme par telephone (pas d'email)
       Given a sent convocation
       And Jeanne n'a pas d'email (courrier recommande)
       When Francois enregistre la confirmation de Jeanne "WillAttend"
       Then will_attend_count is incremented

     Scenario: Emmanuel donne procuration a Philippe (vote "non a tout")
       Given a sent convocation
       When Emmanuel delegue sa procuration a Philippe
       Then proxy_owner_id is set to Philippe's ID
       # Philippe represente desormais 1800 + 1280 = 3080 tantiemes (30.8%)
       # Pouvoir de blocage sur les majorites qualifiees

     Scenario: Procuration a soi-meme echoue
       Given a sent convocation
       When Emmanuel tente de deleguer sa procuration a lui-meme
       Then I receive an error "Cannot delegate to self"

     Scenario: Francois ne peut pas etre mandataire (Art. 3.89 S9)
       Given a sent convocation
       When Emmanuel tente de deleguer sa procuration a Francois (syndic)
       Then I receive an error "Syndic cannot be proxy"

     Scenario: Rappel J-3 envoye a Emmanuel (email non ouvert)
       Given a sent convocation with meeting in 2 days
       And Emmanuel n'a pas ouvert l'email
       And Alice, Bob, Charlie, Diane ont ouvert l'email
       When the reminder job runs
       Then a reminder is sent to Emmanuel
       And no reminder is sent to Alice who already opened

     Scenario: Ahmed (locataire) notifie mais sans droit de vote
       Given a sent convocation
       When Ahmed consulte la notification de l'AG
       Then Ahmed peut soumettre des observations ecrites
       But Ahmed n'a pas de droit de vote

     Scenario: Francois annule la convocation
       Given a convocation in status "Sent"
       When Francois annule la convocation
       Then the convocation status is "Cancelled"

     Scenario: 2e convocation apres quorum non atteint
       Given l'AG a echoue (quorum non atteint sans Philippe et Emmanuel)
       # Sans Philippe (1800) et Emmanuel (1280) : 35.6% < 50%
       When Francois cree une 2e convocation 15 jours apres la 1ere AG
       Then the convocation type is "SecondConvocation"
       And no_quorum_required is true
       And first_meeting_id is set

     Scenario: 2e convocation trop tot echoue
       Given a first meeting on a specific date
       When Francois cree une 2e convocation seulement 10 jours apres
       Then I receive an error "15 days after"

E2E
---

.. code-block:: text

   Test 1 : Cycle complet convocation — personas differencies
     1. POST /convocations (Francois cree, Draft)
     2. PUT /convocations/:id/schedule (Scheduled, J-20)
     3. POST /convocations/:id/send (Sent, 10 recipients)
     4. PUT /convocation-recipients/:id/email-opened (Alice ouvre en 1ere)
     5. PUT /convocation-recipients/:id/email-opened (Bob, Charlie, Diane, Marcel, Nadia, Marguerite ouvrent)
     6. PUT /convocation-recipients/:id/attendance (Alice WillAttend)
     7. PUT /convocation-recipients/:id/attendance (8 confirmations WillAttend)
     8. Jeanne : confirmation par telephone (Francois enregistre WillAttend)
     9. Emmanuel : email NON ouvert → candidat rappel J-3
     10. PUT /convocation-recipients/:id/proxy (Emmanuel → Philippe, 3080 tantiemes)
     11. GET /convocations/:id/tracking-summary (taux ouverture ~90%, presence 80%)
     12. POST /convocations/:id/reminders (rappel J-3 a Emmanuel)
     13. GET /convocations/:id/recipients (verification finale)

   Test 2 : Notification locataire Ahmed (Art. 3.87 S5 6°)
     1. POST /convocations/:id/send (Sent)
     2. Verifier qu'Ahmed recoit une notification d'information
     3. Ahmed peut soumettre des observations mais pas voter

   Test 3 : 2e convocation apres echec quorum
     1. POST /convocations (1ere AG)
     2. POST /convocations/:id/send (Sent)
     3. L'AG echoue : Philippe + Emmanuel absents = 35.6% < 50%
     4. POST /convocations (2e convocation, type SecondConvocation)
     5. Verifier first_meeting_id et no_quorum_required

   Test 4 : Annulation
     1. POST /convocations (Francois cree, Draft)
     2. PUT /convocations/:id/cancel (Cancelled)
     3. Verifier que l'envoi est impossible apres annulation

   Test 5 : Validation delai legal
     1. Tenter POST /convocations avec meeting dans 5 jours -> 400
     2. POST /convocations avec meeting dans 20 jours -> 201
     3. Tenter schedule avec send_date trop tardive -> 400

   Test 6 : Jeanne — canal courrier recommande
     1. POST /convocations/:id/send
     2. Verifier que Jeanne (pas d'email) est marquee courrier recommande
     3. Francois enregistre sa confirmation manuellement

Diagramme de workflow
---------------------

.. code-block:: text

   Convocation:
     Draft ---schedule---> Scheduled ---send---> Sent ---[J-3]---> Reminders
       |                                          |
       '---send--->  Sent                         '---cancel---> Cancelled
       |
       '---cancel---> Cancelled

   Recipient (differencies par persona):
     [Email envoye]  --> [Email ouvert] --> [Presence confirmee] --> [Post-AG]
       |  (Alice,Bob,       (Alice 1h,        (WillAttend)           (Attended)
       |   Charlie,Diane,    Bob 2h,
       |   Marcel,Nadia,     Charlie 24h...)
       |   Marguerite,
       |   Emmanuel,
       |   Philippe)
       |
     [Courrier recommande]  -->  [Confirmation telephone]  --> [Post-AG]
       (Jeanne Devos)            (Francois enregistre)        (Attended)
       |
     [Email non ouvert]  --> [Rappel J-3]  --> [Procuration]
       (Emmanuel)             (automatique)    (→ Philippe)

   Attendance:
     Pending --> WillAttend --> Attended (final)
       |
       '--> WillNotAttend --> DidNotAttend (final)

   Locataire (Ahmed):
     [Notification info] --> [Observations ecrites] (pas de vote)
