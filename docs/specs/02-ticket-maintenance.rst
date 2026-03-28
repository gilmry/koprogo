=================================================
Workflow 02 : Cycle de maintenance complet
=================================================

:Issue: #346
:Personas: Voir `00-personas-et-seed.rst <00-personas-et-seed.rst>`_
:Acteurs: Charlie Martin (signale la fuite), Francois Leroy (syndic), Hassan El Amrani (prestataire magic link), Alice Dubois + Diane Peeters (CdC valident)
:Articles CC: Art. 3.89 §5 2° (actes conservatoires), Art. 3.89 §5 1° (execution decisions AG)
:Priorite: Haute

Resume
------

Charlie Martin (lot 3B, 4e etage) signale une fuite au plafond de sa salle de bain
— l'eau vient du lot de Nadia Benali au 4e etage. Francois Leroy, le syndic, qualifie
le ticket, assigne Hassan El Amrani (entrepreneur couvreur) et lui envoie un magic
link pour acceder a la PWA. Hassan remplit un rapport de travaux avec photos
avant/apres. Alice Dubois (presidente CdC) et Diane Peeters (membre CdC, avocate)
valident le rapport, ce qui declenche le paiement automatique.

Pre-conditions legales
-----------------------

- **Art. 3.89 §5 2°** : Le syndic est charge d'accomplir tous actes conservatoires et d'administration provisoire, y compris les travaux urgents sans decision AG prealable.
- **Art. 3.89 §5 1°** : Pour les travaux non urgents, le syndic execute les decisions prises par l'AG.
- **Art. 3.89 §5 12°** : Le syndic soumet un rapport d'evaluation des contrats de fournitures regulieres a l'AG annuelle.

Etapes
------

1. **Charlie Martin** — Signale la fuite en creant un ticket (statut ``Open``).
   - Endpoint : ``POST /tickets``
   - Body : ``building_id``, ``unit_id`` (lot 3B de Charlie), ``title``: "Fuite d'eau au plafond — eau vient du 4e etage", ``description``: "De l'eau coule du plafond de ma salle de bain. L'eau semble provenir de l'appartement 4A (Nadia Benali) au-dessus.", ``category``: Plumbing, ``priority``: High
   - Validation domaine : titre non-vide (max 200 car.), description non-vide (max 5000 car.)
   - Statut initial : ``Open``
   - Audit : ``TicketCreated``

2. **Francois Leroy (syndic)** — Consulte le ticket et assigne Hassan El Amrani.
   - Endpoint : ``PUT /tickets/:id/assign`` (body: ``user_id`` de Hassan)
   - Methode domaine : ``ticket.assign(hassan_id)``
   - Le statut passe automatiquement de ``Open`` a ``InProgress``.
   - Validation : impossible d'assigner un ticket ``Closed`` ou ``Cancelled``.
   - Audit : ``TicketAssigned``

3. **Francois** — Genere un magic link JWT (72h) et l'envoie a Hassan par email.
   - Endpoint : ``POST /contractor-reports`` → cree le rapport en statut ``Draft`` lie au ``ticket_id``
   - Endpoint : ``POST /contractor-reports/:id/generate-magic-link`` → genere un JWT 72h, stocke le hash
   - Methode domaine : ``ticket.send_work_order_to_contractor()`` → renseigne ``work_order_sent_at``
   - Validation : ticket doit etre ``InProgress`` et ``assigned_to`` non-null.
   - Hassan recoit le lien a hassan@toitures-bruxelles.be

4. **Hassan El Amrani (prestataire)** — Accede au rapport via le magic link (sans authentification classique).
   - Endpoint : ``GET /contractor-reports/magic/:token`` → retourne le rapport si le token est valide
   - Validation : ``contractor_report.is_magic_token_valid()`` (token non expire, < 72h)
   - Hassan voit : details du ticket de Charlie, formulaire de rapport

5. **Hassan** — Remplit le rapport de travaux (photos avant/apres, pieces, compte-rendu).
   - Hassan met a jour le rapport :
     - ``work_date`` : date d'intervention
     - ``compte_rendu`` : "Remplacement du joint d'etancheite de la baignoire lot 4A. La fuite provenait d'un joint deteriore. Pas de degats structurels."
     - ``photos_before[]`` : document_ids des photos avant travaux (plafond mouille de Charlie)
     - ``photos_after[]`` : document_ids des photos apres travaux (joint remplace, plafond seche)
     - ``parts_replaced[]`` : [{"name": "Joint etancheite baignoire", "reference": "WEDI-610", "quantity": 1}]
   - Endpoint : ``PUT /contractor-reports/:id`` (via magic link token)

6. **Hassan** — Soumet le rapport (statut ``Draft`` → ``Submitted``).
   - Endpoint : ``POST /contractor-reports/:id/submit``
   - Methode domaine : ``contractor_report.submit()``
   - Validation : ``compte_rendu`` obligatoire et non-vide.
   - ``submitted_at`` est renseigne.

7. **Francois (syndic)** — Commence l'examen du rapport (``Submitted`` → ``UnderReview``).
   - Endpoint : ``PUT /contractor-reports/:id/review``
   - Methode domaine : ``contractor_report.start_review()``

8. **Alice Dubois (presidente CdC) + Diane Peeters (membre CdC)** — Valident OU demandent des corrections OU rejettent.
   - **Cas A — Validation** :
     - Alice et Diane examinent les photos avant/apres et le compte-rendu de Hassan.
     - Diane (avocate) verifie la conformite du rapport avec le devis initial.
     - Endpoint : ``PUT /contractor-reports/:id/validate`` (body: ``validated_by``: alice_id)
     - Methode domaine : ``contractor_report.validate(alice_id)``
     - Statut → ``Validated``, ``validated_at`` et ``validated_by`` renseignes.
     - Declenchement du paiement automatique vers Hassan.
   - **Cas B — Demande de corrections** :
     - Diane remarque que les photos avant manquent.
     - Endpoint : ``PUT /contractor-reports/:id/request-corrections`` (body: ``comments``: "Manque les photos du plafond mouille avant intervention")
     - Methode domaine : ``contractor_report.request_corrections(comments)``
     - Statut → ``RequiresCorrection``, ``review_comments`` renseigne.
     - Hassan peut re-soumettre (retour a l'etape 6) via son magic link (s'il n'est pas expire).
   - **Cas C — Rejet** :
     - Endpoint : ``PUT /contractor-reports/:id/reject`` (body: ``comments``: "Travaux non conformes au devis", ``rejected_by``: diane_id)
     - Methode domaine : ``contractor_report.reject(comments, diane_id)``
     - Statut → ``Rejected`` (terminal).

9. **Francois** — Resout le ticket (``InProgress`` → ``Resolved``).
   - Endpoint : ``PUT /tickets/:id/resolve`` (body: ``resolution_notes``: "Fuite reparee par Hassan El Amrani. Joint baignoire lot 4A remplace. Rapport valide par le CdC.")
   - Methode domaine : ``ticket.resolve(resolution_notes)``
   - ``resolved_at`` est renseigne.

10. **Charlie** — Confirme la resolution et ferme le ticket (``Resolved`` → ``Closed``).
    - Charlie verifie que son plafond ne fuit plus.
    - Endpoint : ``PUT /tickets/:id/close``
    - Methode domaine : ``ticket.close()``
    - ``closed_at`` est renseigne.
    - Audit : ``TicketClosed``

Post-conditions
---------------

- Le ticket de Charlie est en statut ``Closed`` avec ``resolved_at`` et ``closed_at`` renseignes.
- Le rapport de Hassan est en statut ``Validated`` avec ``validated_at`` et ``validated_by`` (Alice) renseignes.
- Les photos avant/apres sont stockees comme documents lies au rapport.
- Le magic link JWT de Hassan est expire ou consomme.
- Le paiement automatique est declenche suite a la validation du rapport par le CdC (Alice + Diane).
- Evenements d'audit emis : ``TicketCreated``, ``TicketAssigned``, ``TicketStatusChanged``, ``TicketResolved``, ``TicketClosed``.

Donnees seed requises
----------------------

.. note::

   Les personas et l'immeuble de reference sont definis dans
   `00-personas-et-seed.rst <00-personas-et-seed.rst>`_.
   Ce workflow utilise les personas suivants du seed partage :

- **Building** : Residence du Parc Royal (42 Avenue Louise, 1050 Ixelles, 182 lots, 10000 tantiemes)
- **Charlie Martin** : Lot 3B, 3e etage, 660/10000 tantiemes — signale la fuite (role ``owner``, charlie@residence-parc.be)
- **Nadia Benali** : Lot 4A, 4e etage, 320/10000 tantiemes — l'eau vient de chez elle (mentionnee dans la description du ticket)
- **Francois Leroy** : Syndic professionnel (role ``syndic``, francois@syndic-leroy.be)
- **Hassan El Amrani** : Entrepreneur couvreur (role ``contractor``, hassan@toitures-bruxelles.be) — accede via magic link
- **Alice Dubois** : Lot 2A, presidente CdC, 450/10000 tantiemes — valide le rapport (alice@residence-parc.be)
- **Diane Peeters** : Lot 3A, membre CdC, avocate, 580/10000 tantiemes — verifie la conformite (diane@residence-parc.be)
- **Ticket** : titre "Fuite d'eau au plafond — eau vient du 4e etage", description avec mention du lot 4A de Nadia, categorie ``Plumbing``, priorite ``High``, statut ``Open``, cree par Charlie

Scenario BDD (Gherkin)
-----------------------

.. code-block:: gherkin

   Feature: Cycle de maintenance complet (Ticket + Rapport prestataire)

     Background:
       Given le building "Residence du Parc Royal" avec 182 lots
       And Charlie Martin possede le lot 3B (660/10000 tantiemes)
       And Francois Leroy est syndic de l'immeuble
       And Hassan El Amrani est prestataire (entrepreneur couvreur)
       And Alice Dubois est presidente du Conseil de Copropriete
       And Diane Peeters est membre du Conseil de Copropriete

     Scenario: Cycle complet de la creation a la cloture
       Given Charlie cree un ticket "Fuite d'eau au plafond" avec priorite "High" et categorie "Plumbing"
       And le ticket est en statut "Open"
       When Francois assigne Hassan au ticket
       Then le ticket passe en statut "InProgress"
       And assigned_to est Hassan

       When Francois genere un magic link pour le rapport de Hassan
       Then work_order_sent_at est renseigne
       And le magic link est valide pendant 72h

       When Hassan accede au rapport via le magic link
       Then le rapport est retourne avec les details du ticket de Charlie

       When Hassan remplit le rapport avec compte_rendu "Remplacement joint etancheite baignoire lot 4A"
       And Hassan ajoute des photos avant (plafond mouille) et apres (joint remplace)
       And Hassan soumet le rapport
       Then le rapport est en statut "Submitted"
       And submitted_at est renseigne

       When Alice et Diane (CdC) valident le rapport
       Then le rapport est en statut "Validated"
       And validated_at est renseigne
       And validated_by est Alice

       When Francois resout le ticket avec notes "Fuite reparee par Hassan, rapport valide par CdC"
       Then le ticket est en statut "Resolved"
       And resolved_at est renseigne

       When Charlie ferme le ticket
       Then le ticket est en statut "Closed"
       And closed_at est renseigne

     Scenario: Diane (CdC) demande des corrections a Hassan
       Given un rapport soumis par Hassan pour le ticket de Charlie
       When Diane demande des corrections avec le commentaire "Manque photos du plafond avant intervention"
       Then le rapport est en statut "RequiresCorrection"
       And review_comments contient "Manque photos du plafond avant intervention"

       When Hassan ajoute les photos manquantes et re-soumet
       Then le rapport est en statut "Submitted"

       When Alice et Diane valident le rapport
       Then le rapport est en statut "Validated"

     Scenario: Rejet du rapport par le CdC
       Given un rapport soumis par Hassan
       When Diane rejette le rapport avec "Travaux non conformes au devis initial"
       Then le rapport est en statut "Rejected"
       And le statut est terminal

     Scenario: Magic link de Hassan expire apres 72h
       Given un magic link genere pour Hassan il y a 73 heures
       When Hassan tente d'acceder au rapport
       Then une erreur "Token expire" est retournee

     Scenario: Impossible d'assigner un ticket ferme
       Given le ticket de Charlie est en statut "Closed"
       When Francois tente d'assigner Hassan au ticket
       Then une erreur "Cannot assign a closed or cancelled ticket" est retournee

     Scenario: Charlie rouvre un ticket mal resolu
       Given le ticket de Charlie est en statut "Closed"
       When Charlie rouvre le ticket avec raison "La fuite persiste, le plafond coule encore"
       Then le ticket est en statut "InProgress"
       And resolution_notes contient "REOPENED"
       And resolved_at est null
       And closed_at est null

     Scenario: Hassan tente de soumettre sans compte-rendu
       Given un rapport de Hassan en statut "Draft" sans compte_rendu
       When Hassan tente de soumettre le rapport
       Then une erreur "compte_rendu est obligatoire" est retournee

     Scenario: Rapport doit etre lie a un ticket ou un devis
       When on tente de creer un rapport sans ticket_id ni quote_id
       Then une erreur "doit etre lie a un ticket ou a un devis" est retournee

Scenario E2E (narratif)
------------------------

**Acteurs** : Charlie Martin (coproprietaire), Francois Leroy (syndic), Hassan El Amrani (prestataire), Alice Dubois (presidente CdC), Diane Peeters (membre CdC)

1. ``humanLogin(charlie)`` → obtient un JWT avec role ``owner`` (charlie@residence-parc.be)
2. ``POST /tickets`` → Charlie cree le ticket "Fuite d'eau au plafond — eau vient du 4e etage", categorie Plumbing, priorite High → 201 Created, recupere ``ticket_id``
3. ``GET /tickets/:ticket_id`` → verifie statut "Open", created_by = charlie_id
4. ``humanLogin(francois)`` → obtient un JWT avec role ``syndic`` (francois@syndic-leroy.be)
5. ``PUT /tickets/:ticket_id/assign`` (body: user_id=hassan_id) → statut passe a "InProgress"
6. ``POST /contractor-reports`` (body: ticket_id, contractor_name: "Hassan El Amrani — Toitures Bruxelles") → 201 Created, recupere ``report_id``
7. ``POST /contractor-reports/:report_id/generate-magic-link`` → recupere ``magic_token``, envoye a hassan@toitures-bruxelles.be
8. ``GET /contractor-reports/magic/:magic_token`` → 200 OK, rapport en statut "Draft" (acces sans auth par Hassan)
9. ``PUT /contractor-reports/:report_id`` (via magic link) → Hassan met a jour : work_date, compte_rendu="Remplacement joint etancheite baignoire lot 4A", photos_before (plafond mouille), photos_after (joint remplace), parts_replaced=[{"name": "Joint etancheite baignoire", "reference": "WEDI-610", "quantity": 1}]
10. ``POST /contractor-reports/:report_id/submit`` → statut passe a "Submitted"
11. ``humanLogin(francois)``
12. ``PUT /contractor-reports/:report_id/review`` → statut passe a "UnderReview"
13. ``humanLogin(alice)`` → obtient un JWT avec role ``owner`` + presidente CdC (alice@residence-parc.be)
14. ``PUT /contractor-reports/:report_id/validate`` (body: validated_by=alice_id) → Alice valide apres accord avec Diane → statut passe a "Validated"
15. ``humanLogin(francois)``
16. ``PUT /tickets/:ticket_id/resolve`` (body: resolution_notes="Fuite reparee par Hassan El Amrani, rapport valide par Alice (CdC)") → statut passe a "Resolved"
17. ``humanLogin(charlie)``
18. ``PUT /tickets/:ticket_id/close`` → Charlie confirme, statut passe a "Closed"
19. ``GET /tickets/:ticket_id`` → verifie : status=Closed, resolved_at non-null, closed_at non-null
20. ``GET /contractor-reports/:report_id`` → verifie : status=Validated, validated_at non-null, validated_by=alice_id
