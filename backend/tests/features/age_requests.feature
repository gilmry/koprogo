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
