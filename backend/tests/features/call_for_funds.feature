# Feature: Call for Funds (Issue #201)
# Collective payment requests to all unit owners
# Statuses: Draft -> Sent -> Cancelled

Feature: Call for Funds
  As a syndic
  I want to create calls for funds
  So that owners can contribute proportionally to building expenses

  Background:
    Given the system is initialized
    And an organization "Appel Copro ASBL" exists with id "org-appel"
    And a building "Residence Appel" with 4 units exists in organization "org-appel"
    And 4 owners with ownership percentages exist

  Scenario: Create a call for funds
    When I create a call for funds:
      | title              | Charges Q1 2026           |
      | total_amount       | 10000.00                  |
      | contribution_type  | QuarterlyCharge           |
      | due_date           | 2026-03-31                |
      | account_code       | 701000                    |
    Then the call for funds should be created with status "Draft"

  Scenario: Send call for funds generates individual contributions
    Given a draft call for funds of 10000 EUR exists
    When I send the call for funds
    Then the status should be "Sent"
    And individual contributions should be generated for each owner
    And each contribution amount should be proportional to ownership percentage

  Scenario: List calls for funds by building
    Given 3 calls for funds exist for the building
    When I list calls for funds for the building
    Then I should get 3 calls

  Scenario: Get overdue calls for funds
    Given a sent call for funds with past due date exists
    When I list overdue calls for funds
    Then the overdue call should appear in the list

  Scenario: Cancel a call for funds
    Given a draft call for funds exists
    When I cancel the call for funds
    Then the status should be "Cancelled"

  Scenario: Delete a draft call for funds
    Given a draft call for funds exists
    When I delete the call for funds
    Then the call should be deleted

  Scenario: Cannot delete a sent call for funds
    Given a sent call for funds exists
    When I try to delete the sent call for funds
    Then the deletion should fail

  Scenario: Get call for funds by ID
    Given a call for funds exists
    When I get the call for funds by ID
    Then I should receive the full details
    And the total amount should be correct

  Scenario: Contribution amounts are proportional to ownership
    Given 2 owners with 60% and 40% ownership exist
    And a call for funds of 1000 EUR is sent
    Then owner with 60% should have contribution of 600 EUR
    And owner with 40% should have contribution of 400 EUR

  Scenario: Record owner payment against contribution
    Given a sent call for funds with contributions exists
    When owner pays their contribution
    Then the contribution should be marked as paid
    And the payment date should be recorded
