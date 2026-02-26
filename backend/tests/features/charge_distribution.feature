# Feature: Charge Distribution (Issue #205)
# Allocation of invoice charges across unit owners based on ownership percentages

Feature: Charge Distribution
  As a syndic or accountant
  I want to distribute invoice charges across unit owners
  So that each owner pays their fair share based on ownership percentage

  Background:
    Given the system is initialized
    And an organization "Distrib Copro ASBL" exists with id "org-distrib"
    And a building "Residence Distribution" with 3 units exists
    And unit 1 owned by "Alice" at 40%
    And unit 2 owned by "Bob" at 35%
    And unit 3 owned by "Charlie" at 25%
    And an expense of 1000 EUR exists for the building

  Scenario: Calculate distribution for an invoice
    When I calculate charge distribution for the expense
    Then distributions should be created for all 3 owners
    And "Alice" should owe 400 EUR (40%)
    And "Bob" should owe 350 EUR (35%)
    And "Charlie" should owe 250 EUR (25%)

  Scenario: Get distribution by expense
    Given charge distribution has been calculated for the expense
    When I get distribution for the expense
    Then I should get 3 distribution entries
    And the total should equal the expense amount

  Scenario: Get distributions by owner
    Given charge distributions exist for multiple expenses
    When I get distributions for owner "Alice"
    Then I should get all distributions for Alice

  Scenario: Get total amount due by owner
    Given charge distributions exist for 2 expenses (1000 EUR and 500 EUR)
    When I get total due for owner "Alice" (40%)
    Then the total due should be 600 EUR (40% of 1500 EUR)

  Scenario: Distribution respects ownership percentages exactly
    Given an expense of 333 EUR exists
    When I calculate charge distribution
    Then "Alice" should owe 133.20 EUR (40%)
    And "Bob" should owe 116.55 EUR (35%)
    And "Charlie" should owe 83.25 EUR (25%)

  Scenario: Recalculate distribution after ownership change
    Given charge distribution was calculated
    And ownership percentages have changed
    When I recalculate the charge distribution
    Then the new amounts should reflect updated percentages
