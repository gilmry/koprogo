# Feature: AGE Requests — Demandes d'AGE par copropriétaires (BC17 - Issue #279)
# Art. 3.87 §2 Code Civil Belge — Threshold: 1/5 of total shares
# Workflow: Draft -> Open -> Reached -> Submitted -> Accepted | Rejected | Expired | Withdrawn

Feature: AGE Requests
  As a co-owner
  I want to request an extraordinary general assembly
  So that urgent matters can be addressed when the syndic hasn't called one

  Background:
    Given the system is initialized
    And an organization "AGE Copro ASBL" exists with id "org-age"
    And a building "Residence AGE" exists in organization "org-age"
    And owners exist with shares:
      | name           | shares_pct |
      | Alice Dupont   | 0.25       |
      | Bob Martin     | 0.25       |
      | Charlie Leroy  | 0.20       |
      | Diana Bernard  | 0.30       |

  # === CREATION ===

  Scenario: Owner creates an AGE request
    When owner "Alice Dupont" creates an AGE request:
      | title       | Urgent roof repairs needed         |
      | description | Significant water infiltration detected |
    Then the AGE request should be created with status "Draft"
    And the threshold_pct should be 0.20
    And the total_shares_pct should be 0.0
    And threshold_reached should be false

  Scenario: AGE request creation fails with empty title
    When owner "Alice Dupont" tries to create an AGE request with empty title
    Then the creation should fail

  # === OPENING FOR SIGNATURES ===

  Scenario: Initiator opens the request for signatures
    Given owner "Alice Dupont" has a draft AGE request
    When owner "Alice Dupont" opens the request for signatures
    Then the AGE request status should be "Open"

  # === COSIGNING ===

  Scenario: Another owner cosigns the request
    Given an open AGE request exists created by "Alice Dupont"
    When owner "Bob Martin" cosigns with shares 0.10
    Then the AGE request total_shares_pct should be 0.10
    And threshold_reached should be false

  Scenario: Threshold reached when enough shares signed (1/5 rule)
    Given an open AGE request exists created by "Alice Dupont"
    When owner "Bob Martin" cosigns with shares 0.25
    And owner "Charlie Leroy" cosigns with shares 0.20
    Then the total_shares_pct should be at least 0.20
    And threshold_reached should be true
    And the AGE request status should be "Reached"

  Scenario: Owner cannot cosign twice
    Given an open AGE request exists created by "Alice Dupont"
    And owner "Bob Martin" has already cosigned
    When owner "Bob Martin" tries to cosign again
    Then the cosigning should fail

  Scenario: Initiator can remove a cosignatory
    Given an open AGE request exists created by "Alice Dupont"
    And owner "Bob Martin" has cosigned
    When owner "Alice Dupont" removes "Bob Martin" from cosignatories
    Then the cosignatory should be removed
    And the total_shares_pct should decrease

  # === SUBMISSION TO SYNDIC ===

  Scenario: Submit request to syndic when threshold is reached
    Given an AGE request with status "Reached" exists
    When the initiator submits the request to the syndic
    Then the AGE request status should be "Submitted"
    And submitted_to_syndic_at should be set
    And syndic_deadline_at should be 15 days after submission

  Scenario: Cannot submit when threshold is not reached
    Given an open AGE request without enough shares
    When the initiator tries to submit to syndic
    Then the submission should fail

  # === SYNDIC RESPONSE ===

  Scenario: Syndic accepts the AGE request
    Given a submitted AGE request exists
    When the syndic accepts the request
    Then the AGE request status should be "Accepted"
    And syndic_response_at should be set

  Scenario: Syndic rejects the AGE request with reason
    Given a submitted AGE request exists
    When the syndic rejects the request with notes "Not urgent, will be handled at next AG"
    Then the AGE request status should be "Rejected"
    And syndic_notes should contain "Not urgent"

  Scenario: Rejection requires a reason
    Given a submitted AGE request exists
    When the syndic tries to reject without providing notes
    Then the rejection should fail

  # === WITHDRAWAL ===

  Scenario: Initiator withdraws an open request
    Given an open AGE request exists created by "Alice Dupont"
    When owner "Alice Dupont" withdraws the request
    Then the AGE request status should be "Withdrawn"

  # === RETRIEVAL ===

  Scenario: List AGE requests for a building
    Given 2 AGE requests exist in the building
    When I list AGE requests for the building
    Then I should get 2 AGE requests

  Scenario: Get AGE request details
    Given a submitted AGE request exists
    When I retrieve the AGE request by ID
    Then the request details should include cosignatories
    And shares_pct_missing should be 0.0

  # =================================================================
  # WORKFLOW COMPLET — Personas Residence du Parc Royal (Art. 3.87 §2)
  # Marcel initie, Alice/Charlie/Bob/Diane cosignent -> seuil 1/5
  # Spec: docs/specs/03-demande-age.rst
  # =================================================================

  Scenario: Parcours complet — Marcel initie une demande d'AGE pour travaux urgents
    # Marcel Dupont (lot 4B, 450/10000 = 4.5%) initie la demande
    # Apres 20 ans sans entretien majeur, la toiture fuit et la facade se degrade
    Given the building "Residence du Parc Royal" with 182 lots and 10000 tantiemes
    And Francois Leroy is syndic of the building
    And the following owners exist with their tantiemes:
      | name              | lot | tantiemes | shares_pct |
      | Marcel Dupont     | 4B  | 450       | 0.045      |
      | Alice Dubois      | 2A  | 450       | 0.045      |
      | Charlie Martin    | 3B  | 660       | 0.066      |
      | Bob Janssen       | 2B  | 430       | 0.043      |
      | Diane Peeters     | 3A  | 580       | 0.058      |

    # Etape 1: Marcel cree la demande (Draft)
    When Marcel creates an AGE request:
      | title       | Travaux urgents de renovation — 200.000 EUR                                      |
      | description | Apres 20 ans sans entretien majeur, la toiture fuit, la facade se degrade, et l'installation electrique des communs n'est plus aux normes. |
    Then the AGE request should be created with status "Draft"

    # Etape 2: Marcel ouvre pour signatures (Draft -> Open)
    When Marcel opens the request for signatures
    Then the AGE request status should be "Open"

    # Etape 3: Marcel cosigne en premier (4.5%)
    When Marcel cosigns with shares 0.045
    Then total_shares_pct should be 0.045
    And threshold_reached should be false
    # 450/10000 = 4.5% << 20%

    # Etape 4: Alice cosigne (presidente CdC, soutient Marcel) -> 9.0%
    When Alice cosigns with shares 0.045
    Then total_shares_pct should be 0.090
    And threshold_reached should be false

    # Etape 5: Charlie cosigne (inquiet du cout mais convaincu) -> 15.6%
    When Charlie cosigns with shares 0.066
    Then total_shares_pct should be 0.156
    And threshold_reached should be false

    # Etape 6: Bob cosigne (commissaire aux comptes, confirme les chiffres) -> 19.9%
    When Bob cosigns with shares 0.043
    Then total_shares_pct should be 0.199
    And threshold_reached should be false
    # Si proche! 19.9% < 20%

    # Etape 7: Diane cosigne (avocate, sait que le report engage la responsabilite du syndic) -> 25.7%
    When Diane cosigns with shares 0.058
    Then total_shares_pct should be 0.257
    And threshold_reached should be true
    And the AGE request status should be "Reached"
    # 2570/10000 = 25.7% >= 20% — SEUIL ATTEINT!

    # Etape 8: Marcel soumet a Francois (15 jours pour repondre)
    When Marcel submits the request to Francois
    Then the AGE request status should be "Submitted"
    And submitted_to_syndic_at should be set
    And syndic_deadline_at should be 15 days after submission

    # Etape 9: Francois accepte la demande
    When Francois accepts the request with notes "AGE convoquee pour le 15 mai 2026. Trois devis de couvreurs seront presentes."
    Then the AGE request status should be "Accepted"
    And syndic_response_at should be set
    And syndic_notes should contain "15 mai 2026"

  Scenario: Rejet par Francois — motif obligatoire (Art. 3.87 §2)
    Given a submitted AGE request by Marcel to Francois (status "Submitted")
    When Francois tries to reject without providing a reason
    Then an error "motif de refus est obligatoire" is returned

    When Francois rejects with reason "Les travaux ne sont pas urgents au sens de l'Art. 3.89 §5 2°"
    Then the AGE request status should be "Rejected"
    And syndic_notes should contain "pas urgents"
    # Diane (avocate) pourra contester — 5 coproprietaires representant 25.7% ont signe

  Scenario: Auto-convocation apres expiration du delai de Francois (15 jours)
    Given a submitted AGE request by Marcel submitted 16 days ago
    When the system checks for expired requests
    Then is_deadline_expired should return true

    When the system triggers auto-convocation
    Then the AGE request status should be "Expired"
    And auto_convocation_triggered should be true
    # Marcel et les cosignataires peuvent desormais convoquer eux-memes l'AG

  Scenario: Retrait de Bob fait perdre le seuil — retombe en Open
    Given an AGE request with Marcel (4.5%), Alice (4.5%), Charlie (6.6%), Bob (4.3%), Diane (5.8%) as cosignatories
    And the AGE request is in status "Reached" (total 25.7%)
    When Bob removes his signature
    Then total_shares_pct should be 0.214
    And threshold_reached should still be true
    # 21.4% >= 20% — still above threshold

    When Charlie also removes his signature
    Then total_shares_pct should be 0.148
    And threshold_reached should be false
    # 14.8% < 20% — below threshold
    And the AGE request status should be "Open"

  Scenario: Soumission impossible sans atteindre le seuil
    Given an open AGE request with Marcel (4.5%) and Alice (4.5%) = 9.0%
    When Marcel tries to submit to Francois
    Then an error "doit etre en statut Reached" is returned

  Scenario: Seul Marcel (initiateur) peut retirer la demande
    Given an open AGE request created by Marcel
    When Alice tries to withdraw the request
    Then an error "Seul l'initiateur peut retirer" is returned

    When Marcel withdraws the request
    Then the AGE request status should be "Withdrawn"

  Scenario: Progression vers le seuil — pourcentages affiches
    Given an open AGE request by Marcel without cosignatories
    Then calculate_progress_percentage should return 0.0
    And shares_pct_missing should return 0.20

    When Marcel cosigns with shares 0.045
    Then calculate_progress_percentage should return 22.5
    And shares_pct_missing should return 0.155

    When Alice cosigns with shares 0.045
    Then calculate_progress_percentage should return 45.0
    And shares_pct_missing should return 0.110
