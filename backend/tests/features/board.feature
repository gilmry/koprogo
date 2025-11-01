Feature: Board of Directors Management (Conseil de Copropriété)
  As a property manager or board member
  I want to manage the board of directors and their decisions
  So that the condominium governance is properly tracked according to Belgian law

  Background:
    Given a coproperty management system
    And a building with more than 20 units exists

  # Board Member Election and Management
  Scenario: Elect a board member
    When I elect a user as board president with a 1-year mandate
    Then the board member should be created
    And the mandate should be active
    And the mandate duration should be approximately 1 year

  Scenario: Elect multiple board members with different positions
    When I elect a user as board president with a 1-year mandate
    And I elect another user as board treasurer with a 1-year mandate
    And I elect a third user as board member with a 1-year mandate
    Then the building should have 3 active board members
    And the board should have a president
    And the board should have a treasurer

  Scenario: Cannot elect board member for building with 20 or fewer units
    Given a building with 20 units exists
    When I attempt to elect a user as board president
    Then the election should fail with "only required for buildings with more than 20 units"

  Scenario: Board mandate expiration detection
    Given a board member with mandate ending in 45 days
    When I check the board member status
    Then the mandate should be flagged as expiring soon

  Scenario: Renew an existing mandate
    Given an active board member
    And a new general assembly meeting
    When I renew the board member's mandate at the new meeting
    Then the mandate end date should be extended by approximately 1 year
    And the mandate should still be active

  Scenario: Remove a board member before mandate end
    Given an active board member
    When I remove the board member
    Then the board member should no longer exist
    And the building should have one less active board member

  Scenario: Get board statistics
    Given 3 active board members
    And 2 expired board members
    When I request board statistics for the building
    Then the statistics should show 3 active members
    And the statistics should indicate presence of president
    And the statistics should indicate presence of treasurer

  # Board Decisions Management
  Scenario: Create a decision to follow after general assembly
    Given a general assembly meeting has occurred
    When I create a decision "Roof repairs" with deadline in 60 days
    Then the decision should be created with status "pending"
    And the decision should have a deadline
    And the decision should not be overdue

  Scenario: Create a decision without deadline
    Given a general assembly meeting has occurred
    When I create a decision "Prepare annual budget" without deadline
    Then the decision should be created with status "pending"
    And the decision should not have a deadline

  Scenario: Update decision status from pending to in progress
    Given a pending decision exists
    When I update the decision status to "in_progress"
    Then the decision status should be "in_progress"

  Scenario: Complete a decision
    Given a decision in progress
    When I mark the decision as completed
    Then the decision status should be "completed"
    And the decision should have a completion timestamp

  Scenario: Cannot change completed decision to pending
    Given a completed decision
    When I attempt to update the status to "pending"
    Then the status update should fail with "Invalid status transition"

  Scenario: Add notes to a decision
    Given a pending decision exists
    When I add notes "Contractor contacted, waiting for quote"
    Then the decision should have the notes

  Scenario: Decision becomes overdue when deadline passes
    Given a decision with deadline yesterday
    When I check the decision status
    Then the decision should be flagged as overdue
    And the decision status should be "overdue"

  Scenario: List overdue decisions for a building
    Given 2 overdue decisions
    And 1 pending decision with future deadline
    When I request all overdue decisions for the building
    Then I should get 2 overdue decisions

  Scenario: List decisions by status
    Given 3 pending decisions
    And 2 in-progress decisions
    And 1 completed decision
    When I request all decisions with status "pending"
    Then I should get 3 decisions

  Scenario: Get decision statistics
    Given 3 pending decisions
    And 2 in-progress decisions
    And 1 completed decision
    And 1 overdue decision
    When I request decision statistics for the building
    Then the statistics should show 7 total decisions
    And the statistics should show 3 pending decisions
    And the statistics should show 2 in-progress decisions
    And the statistics should show 1 completed decision
    And the statistics should show 1 overdue decision

  # Legal Compliance Scenarios
  Scenario: Syndic cannot be elected as board member (legal incompatibility)
    Given a user with role "syndic"
    When I attempt to elect this user as board president
    Then the election should fail due to legal incompatibility

  Scenario: Board member mandate must be approximately 1 year
    When I attempt to create a board member with 6-month mandate
    Then the creation should fail with "Mandate duration must be approximately 1 year"

  Scenario: Board member mandate must be approximately 1 year (too long)
    When I attempt to create a board member with 18-month mandate
    Then the creation should fail with "Mandate duration must be approximately 1 year"

  # Edge Cases
  Scenario: Cannot create decision for non-existent building
    When I attempt to create a decision for a non-existent building
    Then the creation should fail with "Building not found"

  Scenario: Cannot create decision for non-existent meeting
    When I attempt to create a decision for a non-existent meeting
    Then the creation should fail with "Meeting not found"

  Scenario: List active board members only excludes expired mandates
    Given 2 active board members
    And 1 board member with expired mandate
    When I request all active board members
    Then I should get 2 board members
