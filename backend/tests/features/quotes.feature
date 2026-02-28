# Feature: Contractor Quotes Management (Issue #91)
# Belgian professional best practice: 3 quotes recommended for works > 5000 EUR
# Scoring: price 40%, delay 30%, warranty 20%, reputation 10%

Feature: Contractor Quotes Management
  As a syndic
  I want to manage contractor quotes with legal compliance
  So that construction works follow Belgian procurement rules

  Background:
    Given the system is initialized
    And an organization "Devis Copro ASBL" exists with id "org-devis"
    And a building "Residence Travaux" exists in organization "org-devis"
    And a contractor "Plomberie Dupont" exists
    And a contractor "Electricite Martin" exists
    And a contractor "Renovation Lambert" exists

  # === CREATION ===

  Scenario: Syndic creates a quote request
    When I create a quote request:
      | project_title      | Roof waterproofing               |
      | contractor_id      | Plomberie Dupont                 |
      | description        | Complete roof waterproofing work |
      | estimated_budget   | 15000                            |
    Then the quote should be created with status "Requested"

  # === SUBMISSION ===

  Scenario: Contractor submits quote with pricing
    Given a requested quote exists for "Plomberie Dupont"
    When the contractor submits the quote:
      | amount_excl_vat        | 12000  |
      | vat_rate               | 21     |
      | estimated_duration_days | 14    |
      | warranty_years         | 2      |
      | validity_days          | 30     |
    Then the quote status should be "Received"
    And the amount_incl_vat should be calculated with 21% VAT

  Scenario: Submit quote with renovation VAT (6%)
    Given a requested quote exists for "Renovation Lambert"
    When the contractor submits the quote:
      | amount_excl_vat        | 8000   |
      | vat_rate               | 6      |
      | estimated_duration_days | 7     |
      | warranty_years         | 10     |
      | validity_days          | 30     |
    Then the amount_incl_vat should include 6% VAT

  # === REVIEW & DECISION ===

  Scenario: Start review of submitted quote
    Given a received quote exists
    When I start reviewing the quote
    Then the quote status should be "UnderReview"

  Scenario: Accept quote with decision audit trail
    Given a quote under review exists
    When I accept the quote with notes "Best price-quality ratio"
    Then the quote status should be "Accepted"
    And the decision notes should be recorded
    And the decision_at timestamp should be set

  Scenario: Reject quote with decision notes
    Given a quote under review exists
    When I reject the quote with notes "Too expensive for the scope"
    Then the quote status should be "Rejected"
    And the decision notes should be recorded

  Scenario: Contractor withdraws quote
    Given a received quote exists for "Electricite Martin"
    When the contractor withdraws the quote
    Then the quote status should be "Withdrawn"

  # === QUOTE COMPARISON (Best practice: 3 quotes for >5000 EUR) ===

  Scenario: Compare 3 quotes with automatic scoring
    Given 3 submitted quotes exist for the same project:
      | contractor           | amount_excl | duration_days | warranty_years | rating |
      | Plomberie Dupont     | 12000       | 14            | 2              | 80     |
      | Electricite Martin   | 15000       | 10            | 5              | 90     |
      | Renovation Lambert   | 10000       | 21            | 2              | 70     |
    When I compare the 3 quotes
    Then the comparison should include scores for each
    And the scoring should weight price at 40%
    And the scoring should weight delay at 30%
    And the scoring should weight warranty at 20%
    And the scoring should weight reputation at 10%

  # === CONTRACTOR RATING ===

  Scenario: Update contractor rating
    Given a quote exists for "Plomberie Dupont"
    When I update the contractor rating to 85
    Then the contractor rating should be 85

  # === LISTING ===

  Scenario: List quotes by building
    Given 3 quotes exist for the building
    When I list quotes for the building
    Then I should get 3 quotes

  Scenario: List quotes by status
    Given quotes in various statuses exist
    When I list quotes with status "Received"
    Then all returned quotes should have status "Received"

  # === EXPIRATION ===

  Scenario: Detect expired quotes
    Given a quote with validity_date in the past exists
    When I check for expired quotes
    Then the expired quote should be detected

  # === DELETION ===

  Scenario: Delete a quote
    Given a requested quote exists
    When I delete the quote
    Then the quote should be deleted
