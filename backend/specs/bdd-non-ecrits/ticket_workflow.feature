# Feature: Cycle de maintenance complet (Ticket + Rapport prestataire)
# Workflow multi-roles avec personas du seed Residence du Parc Royal
# Issue #346 — Spec: docs/specs/02-ticket-maintenance.rst
# Articles CC: Art. 3.89 §5 2° (actes conservatoires), Art. 3.89 §5 1° (execution decisions AG)

Feature: Cycle de maintenance complet — Ticket + Rapport prestataire (Art. 3.89 §5 CC)
  As Charlie Martin (co-owner reporting a leak)
  I want the syndic to assign a contractor who fills a work report via magic link
  So that the CdC can validate the work and trigger payment

  Background:
    Given the system is initialized
    And the building "Residence du Parc Royal" with 182 lots and 10000 tantiemes
    And Charlie Martin owns lot 3B (660/10000 tantiemes, 3rd floor)
    And Nadia Benali owns lot 4A (320/10000 tantiemes, 4th floor)
    And Francois Leroy is syndic of the building
    And Hassan El Amrani is a contractor (entrepreneur couvreur, hassan@toitures-bruxelles.be)
    And Alice Dubois is presidente of the Conseil de Copropriete (lot 2A, 450/10000)
    And Diane Peeters is membre of the Conseil de Copropriete (lot 3A, 580/10000, avocate)

  # ===============================================================
  # CYCLE COMPLET : Creation → Assignation → Rapport → Validation → Cloture
  # ===============================================================

  Scenario: Cycle complet — de la fuite de Charlie a la cloture
    # Etape 1: Charlie signale la fuite
    When Charlie creates a ticket:
      | title       | Fuite d'eau au plafond — eau vient du 4e etage                              |
      | description | De l'eau coule du plafond de ma salle de bain. L'eau semble provenir du lot 4A (Nadia Benali) au-dessus. |
      | category    | Plumbing                                                                     |
      | priority    | High                                                                         |
    Then the ticket should be created successfully
    And the ticket status should be "Open"
    And the ticket category should be "Plumbing"
    And the ticket priority should be "High"

    # Etape 2: Francois assigne Hassan
    When Francois assigns Hassan to the ticket
    Then the ticket status should be "InProgress"
    And the ticket should be assigned to Hassan

    # Etape 3: Francois genere un magic link pour Hassan (72h)
    When Francois creates a contractor report for the ticket linked to Hassan
    And Francois generates a magic link for Hassan's report
    Then work_order_sent_at should be set
    And the magic link should be valid for 72 hours

    # Etape 4: Hassan accede au rapport via magic link (sans authentification)
    When Hassan accesses the report via the magic link
    Then the report is returned with ticket details
    And the report status should be "Draft"

    # Etape 5: Hassan remplit le rapport (photos avant/apres, compte-rendu)
    When Hassan updates the report:
      | work_date      | 2026-03-20                                                       |
      | compte_rendu   | Remplacement du joint d'etancheite de la baignoire lot 4A. La fuite provenait d'un joint deteriore. Pas de degats structurels. |
    And Hassan adds before photos (plafond mouille de Charlie)
    And Hassan adds after photos (joint remplace, plafond seche)
    And Hassan adds parts replaced:
      | name                          | reference | quantity |
      | Joint etancheite baignoire    | WEDI-610  | 1        |

    # Etape 6: Hassan soumet le rapport (Draft -> Submitted)
    When Hassan submits the report
    Then the report status should be "Submitted"
    And submitted_at should be set

    # Etape 7: Francois commence l'examen (Submitted -> UnderReview)
    When Francois starts the review of Hassan's report
    Then the report status should be "UnderReview"

    # Etape 8: Alice (presidente CdC) et Diane (membre CdC) valident
    When Alice validates the report on behalf of the CdC
    Then the report status should be "Validated"
    And validated_at should be set
    And validated_by should be Alice
    # Validation triggers automatic payment to Hassan

    # Etape 9: Francois resout le ticket
    When Francois resolves the ticket with notes "Fuite reparee par Hassan El Amrani. Joint baignoire lot 4A remplace. Rapport valide par le CdC."
    Then the ticket status should be "Resolved"
    And resolved_at should be set

    # Etape 10: Charlie confirme et ferme le ticket
    When Charlie closes the ticket
    Then the ticket status should be "Closed"
    And closed_at should be set

  # ===============================================================
  # VARIANTE : Demande de corrections par le CdC
  # ===============================================================

  Scenario: Diane (CdC) demande des corrections a Hassan
    Given a submitted report by Hassan for Charlie's plumbing ticket
    And the report is in status "UnderReview"
    When Diane requests corrections with comment "Manque les photos du plafond mouille avant intervention"
    Then the report status should be "RequiresCorrection"
    And review_comments should contain "Manque les photos du plafond"

    When Hassan adds the missing before photos and resubmits
    Then the report status should be "Submitted"

    When Alice validates the report on behalf of the CdC
    Then the report status should be "Validated"

  # ===============================================================
  # VARIANTE : Rejet du rapport
  # ===============================================================

  Scenario: Diane rejette le rapport — travaux non conformes au devis
    Given a submitted report by Hassan in status "UnderReview"
    When Diane rejects the report with comment "Travaux non conformes au devis initial"
    Then the report status should be "Rejected"
    # Rejected is a terminal state — no further action possible on this report

  # ===============================================================
  # CAS LIMITES
  # ===============================================================

  Scenario: Magic link de Hassan expire apres 72h
    Given a magic link was generated for Hassan 73 hours ago
    When Hassan tries to access the report via the expired magic link
    Then an error "Token expire" is returned

  Scenario: Impossible d'assigner un ticket ferme
    Given Charlie's ticket is in status "Closed"
    When Francois tries to assign Hassan to the closed ticket
    Then an error "Cannot assign a closed or cancelled ticket" is returned

  Scenario: Charlie rouvre un ticket mal resolu
    Given Charlie's ticket is in status "Closed"
    When Charlie reopens the ticket with reason "La fuite persiste, le plafond coule encore"
    Then the ticket status should be "InProgress"
    And resolved_at should be null
    And closed_at should be null

  Scenario: Hassan tente de soumettre un rapport sans compte-rendu
    Given a draft report by Hassan without compte_rendu
    When Hassan tries to submit the report
    Then an error "compte_rendu est obligatoire" is returned

  Scenario: Rapport doit etre lie a un ticket ou un devis
    When one tries to create a contractor report without ticket_id nor quote_id
    Then an error "doit etre lie a un ticket ou a un devis" is returned

  Scenario: Seul le CdC peut valider un rapport — Francois (syndic) ne peut pas valider seul
    Given a report by Hassan in status "UnderReview"
    When Francois tries to validate the report without CdC approval
    # Le syndic lance la review mais le CdC (Alice + Diane) valide
    Then the validation requires a CdC member (Alice or Diane)

  Scenario: Priorite High genere une due_date a J+1
    When Charlie creates a ticket with priority "High"
    Then the due_date should be within 24 hours
    # Priorite High = delai 24h selon la configuration systeme
