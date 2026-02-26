# Feature: Owner Contributions (Issue #202)
# Individual owner payment contributions
# Payment statuses: Pending -> Paid

Feature: Owner Contributions
  As a syndic
  I want to track individual owner contributions
  So that I can monitor who has paid and who hasn't

  Background:
    Given the system is initialized
    And an organization "Contrib Copro ASBL" exists with id "org-contrib"
    And a building "Residence Contribution" exists in organization "org-contrib"
    And an owner "Marie Payeuse" exists with a unit in the building

  Scenario: Create an owner contribution
    When I create a contribution for "Marie Payeuse":
      | description        | Charges trimestrielles Q1 |
      | amount             | 250.00                    |
      | contribution_type  | QuarterlyCharge           |
      | account_code       | 701000                    |
    Then the contribution should be created
    And the payment status should be "Pending"

  Scenario: Get contribution by ID
    Given a contribution exists for "Marie Payeuse"
    When I get the contribution by ID
    Then I should receive the contribution details
    And the amount should be correct

  Scenario: List contributions by owner
    Given 3 contributions exist for "Marie Payeuse"
    When I list contributions for "Marie Payeuse"
    Then I should get 3 contributions

  Scenario: Get outstanding (unpaid) contributions
    Given 2 unpaid and 1 paid contributions exist for "Marie Payeuse"
    When I list outstanding contributions for "Marie Payeuse"
    Then I should get 2 contributions
    And all should have status "Pending"

  Scenario: Record payment for a contribution
    Given an unpaid contribution of 250 EUR exists
    When I mark the contribution as paid
    Then the payment status should be "Paid"
    And the payment date should be set

  Scenario: Cannot pay an already-paid contribution
    Given a paid contribution exists
    When I try to mark it as paid again
    Then the operation should fail

  Scenario: List all contributions for organization
    Given contributions exist for multiple owners
    When I list all contributions for the organization
    Then I should get all contributions

  Scenario: Contribution has correct account code
    Given a contribution with account code "701000" exists
    When I get the contribution by ID
    Then the account code should be "701000"
