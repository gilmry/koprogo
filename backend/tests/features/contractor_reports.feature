# Feature: Contractor Reports — Backoffice Prestataires PWA (BC16 - Issue #275)
# Magic link workflow: Syndic generates link → Contractor fills report → Board validates
# Workflow: Draft -> Submitted -> UnderReview -> Validated | Rejected | RequiresCorrection

Feature: Contractor Reports
  As a syndic
  I want contractors to submit work reports via a secure magic link
  So that intervention records are maintained and payments can be triggered

  Background:
    Given the system is initialized
    And an organization "BC16 Copro ASBL" exists with id "org-bc16"
    And a building "Residence BC16" exists in organization "org-bc16"
    And a ticket "Leaking roof" exists in the building
    And a contractor "Entreprise Dumont" exists

  # === CREATION ===

  Scenario: Syndic creates a contractor report for a ticket
    When I create a contractor report:
      | building_id     | (from background)        |
      | contractor_name | Entreprise Dumont        |
      | ticket_id       | (from background)        |
    Then the report should be created with status "Draft"
    And the contractor_name should be "Entreprise Dumont"

  Scenario: Create report without linking to a ticket
    When I create a contractor report:
      | contractor_name | Entreprise Lambert |
    Then the report should be created with status "Draft"

  # === MAGIC LINK ===

  Scenario: Syndic generates a magic link for the contractor
    Given a draft contractor report exists
    When I generate a magic link for the contractor report
    Then a magic link should be returned
    And the link should expire in 72 hours

  Scenario: Contractor accesses report via magic link
    Given a contractor report with a valid magic link exists
    When the contractor accesses the report via magic link
    Then the report details should be returned
    And no authentication should be required

  Scenario: Magic link returns 401 after expiry
    Given a contractor report with an expired magic link exists
    When the contractor tries to access via the expired magic link
    Then the access should be denied

  # === CONTRACTOR UPDATE & SUBMIT ===

  Scenario: Contractor updates the report with work details
    Given a draft contractor report with a valid magic link exists
    When the contractor updates the report via magic link:
      | compte_rendu  | Toiture réparée, 3 ardoises remplacées    |
    Then the report should be updated successfully

  Scenario: Contractor submits the report
    Given a draft contractor report with compte_rendu filled in exists
    When I submit the contractor report
    Then the report status should be "Submitted"

  Scenario: Cannot submit report without compte_rendu
    Given a draft contractor report with no compte_rendu
    When I try to submit the report
    Then the submission should fail
    And the error should mention "compte_rendu"

  # === REVIEW WORKFLOW ===

  Scenario: Syndic/Board starts review of submitted report
    Given a submitted contractor report exists
    When I start the review of the contractor report
    Then the report status should be "UnderReview"

  Scenario: Board validates the contractor report
    Given a report under review exists
    When I validate the contractor report
    Then the report status should be "Validated"

  Scenario: Board requests corrections on the report
    Given a report under review exists
    When I request corrections with comments "Missing parts list"
    Then the report status should be "RequiresCorrection"

  Scenario: Board rejects the contractor report
    Given a report under review exists
    When I reject the contractor report with comments "Work not completed as agreed"
    Then the report status should be "Rejected"

  # === RETRIEVAL ===

  Scenario: List reports by building
    Given 2 contractor reports exist in the building
    When I list contractor reports for the building
    Then I should get 2 reports

  Scenario: List reports by ticket
    Given a contractor report linked to a ticket exists
    When I list contractor reports for the ticket
    Then I should get at least 1 report

  # === DELETION ===

  Scenario: Delete a draft report
    Given a draft contractor report exists
    When I delete the contractor report
    Then the report should be deleted successfully

  Scenario: Cannot delete a validated report
    Given a validated contractor report exists
    When I try to delete the validated report
    Then the deletion should fail
