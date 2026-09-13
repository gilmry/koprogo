# Feature: Contract Evaluation (Issue #276)
# Art. 3.89 ss5 12 Code Civil Belge: Evaluations contracteurs (L13 annual report)
# Criteria scoring: qualite, delai, prix, communication, proprete, conformite_devis (0-5 each)

Feature: Contract Evaluation Management
  As a syndic or co-owner
  I want to evaluate contractor work with scored criteria
  So that future contractor selection is informed by past performance

  Background:
    Given the system is initialized
    And an organization "Eval Copro ASBL" exists with id "org-eval"
    And a building "Residence Evaluation" exists in organization "org-eval"
    And a service provider "ABC Plomberie" exists in organization "org-eval"
    And the user is authenticated as syndic "Jean Syndic"

  # === CREATION ===

  Scenario: Successfully create evaluation with multiple criteria
    When I create a contract evaluation for provider "ABC Plomberie":
      | criteria_qualite        | 5 |
      | criteria_delai          | 4 |
      | criteria_prix           | 3 |
      | would_recommend         | true |
    Then the evaluation should be created successfully
    And the evaluation should have 3 criteria
    And the global score should be 4.0
    And the would_recommend flag should be true

  Scenario: Successfully create evaluation with all six standard criteria
    When I create a contract evaluation for provider "ABC Plomberie":
      | criteria_qualite          | 5 |
      | criteria_delai            | 4 |
      | criteria_prix             | 3 |
      | criteria_communication    | 5 |
      | criteria_proprete         | 4 |
      | criteria_conformite_devis | 3 |
      | would_recommend           | true |
    Then the evaluation should be created successfully
    And the evaluation should have 6 criteria
    And the global score should be 4.0

  Scenario: Create evaluation with minimum scores (all zeros)
    When I create a contract evaluation for provider "ABC Plomberie":
      | criteria_qualite | 0 |
      | criteria_delai   | 0 |
      | would_recommend  | false |
    Then the evaluation should be created successfully
    And the global score should be 0.0
    And the would_recommend flag should be false

  Scenario: Create evaluation with empty criteria computes zero global score
    When I create a contract evaluation for provider "ABC Plomberie" with no criteria
    Then the evaluation should be created successfully
    And the global score should be 0.0

  # === VALIDATION ERRORS ===

  Scenario: Reject evaluation with criteria score above 5
    When I create a contract evaluation for provider "ABC Plomberie":
      | criteria_qualite | 6 |
      | criteria_delai   | 4 |
      | would_recommend  | true |
    Then the evaluation creation should fail
    And the error should contain "must be 0-5"

  Scenario: Reject evaluation with criteria score of 255 (u8 max edge case)
    When I create a contract evaluation for provider "ABC Plomberie":
      | criteria_qualite | 255 |
      | would_recommend  | true |
    Then the evaluation creation should fail
    And the error should contain "must be 0-5"

  # === LINKING ===

  Scenario: Link evaluation to a quote
    Given a contract evaluation exists for provider "ABC Plomberie"
    And a quote exists for building "Residence Evaluation"
    When I link the evaluation to the quote
    Then the evaluation should have a linked quote_id

  Scenario: Link evaluation to a ticket
    Given a contract evaluation exists for provider "ABC Plomberie"
    And a ticket exists in building "Residence Evaluation"
    When I link the evaluation to the ticket
    Then the evaluation should have a linked ticket_id

  # === LEGAL & ANONYMOUS ===

  Scenario: Mark evaluation as legal L13 report
    Given a contract evaluation exists for provider "ABC Plomberie"
    When I mark the evaluation as a legal evaluation
    Then the evaluation is_legal_evaluation flag should be true

  Scenario: Mark evaluation as anonymous for GDPR compliance
    Given a contract evaluation exists for provider "ABC Plomberie"
    When I mark the evaluation as anonymous
    Then the evaluation is_anonymous flag should be true

  # === COMMENTS ===

  Scenario: Add comments to evaluation
    Given a contract evaluation exists for provider "ABC Plomberie"
    When I set comments "Excellent work, very professional and clean"
    Then the evaluation should have comments set

  Scenario: Reject empty comments
    Given a contract evaluation exists for provider "ABC Plomberie"
    When I set comments ""
    Then setting comments should fail
    And the error should contain "Comments cannot be empty"
