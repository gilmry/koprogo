# Feature: Accountant Dashboard
# Quick overview of financial status

Feature: Accountant Dashboard
  As an accountant
  I want a dashboard with key financial metrics
  So that I can quickly assess the financial health of my buildings

  Background:
    Given the system is initialized
    And an organization "Dashboard Copro ASBL" exists with id "org-dashboard"
    And a building with expenses and payments exists

  Scenario: Get accountant dashboard statistics
    When I request the accountant dashboard stats
    Then I should receive dashboard statistics
    And the stats should include expense totals
    And the stats should include payment totals
    And the stats should include outstanding amounts

  Scenario: Get recent transactions
    Given 15 transactions exist
    When I request recent transactions with limit 10
    Then I should get 10 transactions
    And they should be ordered by date descending

  Scenario: Dashboard statistics include contribution totals
    Given owner contributions exist
    When I request the accountant dashboard stats
    Then the stats should include contribution summaries

  Scenario: Empty dashboard for new organization
    Given no financial data exists
    When I request the accountant dashboard stats
    Then all totals should be zero
