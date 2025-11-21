# Feature: Annual Budget Management
# Belgian Legal Requirement: Budget must be voted in AG before fiscal year starts
# Issue #81

Feature: Budget Management
  As a Syndic
  I want to manage annual budgets with AG approval
  So that I comply with Belgian copropriété law

  Background:
    Given the system is initialized
    And an organization "Budget Copro ASBL" exists with id "org-123"
    And a building "Residence Budget" exists in organization "org-123"
    And a user "Syndic Budget" exists with email "syndic@budget.be" in organization "org-123"
    And the user is authenticated as Syndic

  Scenario: Create draft budget for 2026
    When I create a budget with:
      | fiscal_year              | 2026    |
      | ordinary_budget_cents    | 5000000 |
      | extraordinary_budget_cents | 2000000 |
      | notes                    | Draft budget for AG approval |
    Then the budget should be created successfully
    And the budget status should be "Draft"
    And the ordinary budget should be "50000.00 EUR"
    And the extraordinary budget should be "20000.00 EUR"

  Scenario: Submit draft budget for AG approval
    Given a draft budget exists for fiscal year 2026
    When I submit the budget for approval
    Then the budget status should be "PendingApproval"
    And the submitted_at timestamp should be set
    And the budget should be locked for editing

  Scenario: AG approves budget
    Given a budget in status "PendingApproval" exists
    And an AG meeting exists with id "meeting-456"
    When I approve the budget with meeting id "meeting-456"
    Then the budget status should be "Approved"
    And the approved_by_meeting_id should be "meeting-456"
    And the approved_at timestamp should be set
    And the budget should become the active budget for fiscal year 2026

  Scenario: AG rejects budget with reason
    Given a budget in status "PendingApproval" exists
    When I reject the budget with reason "Ordinary budget too high, needs 10% reduction"
    Then the budget status should be "Rejected"
    And the rejection_reason should be "Ordinary budget too high, needs 10% reduction"
    And the budget should be unlocked for editing

  Scenario: Modify rejected budget and resubmit
    Given a rejected budget exists for fiscal year 2026
    When I update the budget with:
      | ordinary_budget_cents | 4500000 |
    And I submit the budget for approval
    Then the budget status should be "PendingApproval"
    And the ordinary budget should be "45000.00 EUR"

  Scenario: Calculate monthly provisions
    Given an approved budget for fiscal year 2026 with:
      | ordinary_budget_cents    | 6000000 |
      | extraordinary_budget_cents | 2400000 |
    When I calculate monthly provisions
    Then the ordinary monthly provision should be "5000.00 EUR"
    And the extraordinary monthly provision should be "2000.00 EUR"
    And the total monthly provision should be "7000.00 EUR"

  Scenario: Generate budget variance report
    Given an approved budget for fiscal year 2026 with ordinary budget 6000000 cents
    And actual expenses for the year total 6600000 cents
    When I request the budget variance report
    Then the variance should be "10.00%"
    And the variance should be "over budget"
    And I should receive an alert for budget overspending

  Scenario: Prevent duplicate budgets for same fiscal year
    Given an approved budget exists for fiscal year 2026
    When I try to create another budget for fiscal year 2026
    Then the creation should fail
    And I should see error "Budget already exists for fiscal year 2026"

  Scenario: List budgets by fiscal year
    Given budgets exist for fiscal years 2024, 2025, 2026
    When I request budgets for fiscal year 2026
    Then I should receive 1 budget
    And the budget should be for fiscal year 2026

  Scenario: Archive old budget when new budget approved
    Given an approved budget exists for fiscal year 2025
    And a new budget for fiscal year 2026 is approved
    When I request the active budget
    Then I should receive the 2026 budget
    And the 2025 budget status should be "Archived"

  Scenario: Get budget statistics for building
    Given 3 budgets exist for the building
    When I request budget statistics
    Then I should see:
      | total_budgets       | 3      |
      | total_approved      | 2      |
      | average_variance    | 8.5%   |
      | current_fiscal_year | 2026   |
