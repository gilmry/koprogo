# Feature: Automatic AG Convocations (Issue #88)
# Belgian legal deadline: 15 days for ALL types per Art. 3.87 §3 Code Civil
# Workflow: Draft -> Scheduled -> Sent -> Cancelled

Feature: Automatic AG Convocations
  As a syndic
  I want to send legal AG convocations automatically
  So that general assembly invitations comply with Belgian law

  Background:
    Given the system is initialized
    And an organization "Convoc Copro ASBL" exists with id "org-convoc"
    And a building "Residence Legale" exists in organization "org-convoc"
    And a meeting "AG Ordinaire Mars 2026" scheduled in 20 days exists
    And 3 owners exist in the building with email addresses

  # === CREATION ===

  Scenario: Create convocation for ordinary AG (15-day minimum)
    When I create a convocation:
      | meeting_type | Ordinary  |
      | language     | FR        |
    Then the convocation should be created with status "Draft"
    And the minimum send date should be at least 15 days before the meeting

  Scenario: Create convocation for extraordinary AG (15-day minimum per Art. 3.87 §3)
    Given a meeting "AG Extraordinaire" scheduled in 20 days exists
    When I create a convocation:
      | meeting_type | Extraordinary |
      | language     | FR            |
    Then the convocation should be created
    And the minimum send date should be at least 15 days before the meeting

  Scenario: Reject convocation violating legal deadline
    Given a meeting "AG Last Minute" scheduled in 3 days exists
    When I try to create a convocation for an ordinary AG
    Then the creation should fail
    And the error should mention "legal deadline" or "minimum"

  # === SCHEDULING ===

  Scenario: Schedule convocation send date
    Given a draft convocation exists
    When I schedule the convocation for 18 days before the meeting
    Then the convocation status should be "Scheduled"
    And the scheduled date should respect the legal deadline

  Scenario: Cannot schedule past the legal deadline
    Given a draft convocation for an ordinary AG exists
    When I try to schedule the convocation for 10 days before the meeting
    Then the scheduling should fail

  # === SENDING ===

  Scenario: Send convocation to all owners
    Given a scheduled convocation exists
    When I send the convocation
    Then the convocation status should be "Sent"
    And recipients should be created for all building owners
    And total_recipients should match the number of owners

  # === TRACKING ===

  Scenario: Track email opened
    Given a sent convocation with recipients exists
    When recipient "owner1@test.be" opens the email
    Then the email_opened_at should be set for that recipient

  Scenario: Update attendance status
    Given a sent convocation with recipients exists
    When recipient "owner1@test.be" confirms attendance
    Then the attendance status should be "WillAttend"

  Scenario: Set proxy delegation (procuration)
    Given a sent convocation with recipients exists
    When recipient "owner2@test.be" delegates proxy to "owner1@test.be"
    Then the proxy should be recorded

  # === REMINDERS ===

  Scenario: Send J-3 reminders to unopened emails
    Given a sent convocation with 2 unopened recipients
    And the meeting is in 3 days
    When I send reminders
    Then reminders should be sent to the 2 unopened recipients

  # === TRACKING SUMMARY ===

  Scenario: Get tracking summary statistics
    Given a sent convocation with tracked recipients
    When I get the tracking summary
    Then the summary should include opening rate
    And the summary should include attendance counts

  # === CANCELLATION ===

  Scenario: Cancel a convocation
    Given a draft convocation exists
    When I cancel the convocation
    Then the convocation status should be "Cancelled"

  # === LISTING ===

  Scenario: List convocations for building
    Given 2 convocations exist for the building
    When I list convocations for the building
    Then I should get 2 convocations

  # =================================================================
  # WORKFLOW COMPLET — Convocation AG avec suivi de presence
  # Personas Residence du Parc Royal
  # Spec: docs/specs/07-convocation-attendance.rst
  # Art. 3.87 §3 (delai 15j), Art. 3.87 §5 6° (notification locataires),
  # Art. 3.87 §7 (procurations max 3), Art. 3.89 §9 (syndic non mandataire)
  # =================================================================

  Scenario: Workflow complet — Francois envoie la convocation a tous les coproprietaires
    Given the building "Residence du Parc Royal" with 182 lots and 10000 tantiemes
    And Francois Leroy is syndic of the building
    And the AG Ordinaire 2026 is scheduled in 20 days
    And the following co-owners exist:
      | name                    | tantiemes | email                         | has_email |
      | Alice Dubois            | 450       | alice@residence-parc.be       | true      |
      | Bob Janssen             | 430       | bob@residence-parc.be         | true      |
      | Charlie Martin          | 660       | charlie@residence-parc.be     | true      |
      | Diane Peeters           | 580       | diane@residence-parc.be       | true      |
      | Marcel Dupont           | 450       | marcel@residence-parc.be      | true      |
      | Nadia Benali            | 320       | nadia@residence-parc.be       | true      |
      | Marguerite Lemaire      | 380       | marguerite@residence-parc.be  | true      |
      | Jeanne Devos            | 290       | null                          | false     |
      | Emmanuel Claes          | 1280      | emmanuel@residence-parc.be    | true      |
      | Philippe Vandermeulen   | 1800      | philippe@residence-parc.be    | true      |
    And Ahmed Mansouri is a tenant of lot 6A (Philippe's apartment)

    # Etape 1: Francois cree la convocation (Draft)
    When Francois creates a convocation for the AG in language "FR"
    Then the convocation status should be "Draft"
    And minimum_send_date should be meeting_date minus 15 days

    # Etape 2: Francois programme l'envoi 20 jours avant (optionnel)
    When Francois schedules the convocation for today
    Then the convocation status should be "Scheduled"

    # Etape 3: Francois envoie la convocation
    When Francois sends the convocation
    Then the convocation status should be "Sent"
    And total_recipients should be 10
    And a PDF convocation should be generated
    # 9 coproprietaires recoivent un email
    # Jeanne Devos recoit un courrier recommande (pas d'email)

  Scenario: Jeanne Devos recoit un courrier recommande (pas d'email — 82 ans)
    # Art. 3.87 §3: la convocation doit etre envoyee a TOUS les coproprietaires
    Given a sent convocation for the Residence du Parc Royal
    And Jeanne Devos has no email address (courrier recommande)
    When the convocation is sent
    Then Jeanne's recipient should be marked as postal delivery
    And email_sent_at should be null for Jeanne
    # Jeanne (82 ans, pension minimum 1050 EUR/mois) confirme par telephone a Francois

    When Francois manually records Jeanne's attendance as "WillAttend"
    Then will_attend_count should be incremented

  Scenario: Emmanuel donne procuration a Philippe — Art. 3.87 §7
    # Emmanuel (1280 tantiemes) donne procuration a Philippe (1800 tantiemes)
    # Philippe represente desormais 1800 + 1280 = 3080 tantiemes (30.8%)
    Given a sent convocation for the Residence du Parc Royal
    When Emmanuel delegates his proxy to Philippe
    Then proxy_owner_id should be set to Philippe's ID
    # Pouvoir de blocage sur les majorites qualifiees (2/3, 4/5)

  Scenario: Procuration a soi-meme echoue
    Given a sent convocation for the Residence du Parc Royal
    When Emmanuel tries to delegate his proxy to himself
    Then an error "Cannot delegate to self" is returned

  Scenario: Francois (syndic) ne peut pas etre mandataire — Art. 3.89 §9
    Given a sent convocation for the Residence du Parc Royal
    When Emmanuel tries to delegate his proxy to Francois (syndic)
    Then an error "Syndic cannot be proxy" is returned

  Scenario: Ahmed (locataire) notifie mais sans droit de vote — Art. 3.87 §5 6°
    Given a sent convocation for the Residence du Parc Royal
    And Ahmed Mansouri is a tenant in the building
    When the convocation is sent
    Then Ahmed should receive an information notification about the AG
    And Ahmed can submit written observations to the syndic
    But Ahmed has no voting rights
    # Art. 3.87 §5 6°: le locataire doit etre informe de l'AG

  Scenario: Alice ouvre l'email en premiere et confirme sa presence
    Given a sent convocation with 10 recipients
    When Alice opens the convocation email
    Then email_opened_at should be recorded for Alice
    And opened_count should be 1

    When Alice confirms attendance as "WillAttend"
    Then will_attend_count should be incremented
    # Alice (presidente CdC) prepare l'AG depuis des semaines

  Scenario: Rappel J-3 envoye a Emmanuel (email non ouvert)
    Given a sent convocation with meeting in 2 days
    And Emmanuel has not opened the email
    And Alice, Bob, Charlie, Diane, Marcel, Nadia, Marguerite have opened the email
    When the J-3 reminder job runs
    Then a reminder should be sent to Emmanuel
    And no reminder should be sent to Alice (already opened)
    And reminder_sent_at should be set for Emmanuel

  Scenario: 2e convocation apres quorum non atteint en 1ere AG
    # Sans Philippe (1800) et Emmanuel (1280) presents:
    # 6640 - 3080 = 3560 tantiemes presents = 35.6% < 50% quorum
    Given the first AG failed due to quorum not reached (Philippe and Emmanuel absent)
    When Francois creates a 2nd convocation 15 days after the first AG
    Then the convocation type should be "SecondConvocation"
    And no_quorum_required should be true
    And first_meeting_id should be set
    # Art. 3.87 §5: aucun quorum requis pour la 2e convocation

  Scenario: 2e convocation trop tot echoue (< 15 jours)
    Given a first meeting on a specific date
    When Francois tries to create a 2nd convocation only 10 days after
    Then an error containing "15 days after" is returned

  Scenario: Meeting dans 5 jours echoue (< 15 jours legaux)
    When Francois tries to create a convocation for an AG in 5 days
    Then an error "Meeting date too soon" is returned
    # Art. 3.87 §3: delai minimum de 15 jours

  Scenario: Annulation d'une convocation Sent
    Given a sent convocation for the Residence du Parc Royal
    When Francois cancels the convocation
    Then the convocation status should be "Cancelled"
