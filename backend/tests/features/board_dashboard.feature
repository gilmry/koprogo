Feature: Board Member Dashboard
  As a board member
  I want to see a dashboard with key metrics and alerts
  So that I can effectively monitor the syndic's execution of board decisions

  Background:
    Given a coproperty management system

  Scenario: Board member views their dashboard with complete data
    Given a building with 25 units exists
    And I am a board member for that building
    And there are 3 pending decisions
    And there is 1 overdue decision
    And my mandate expires in 45 days
    When I request my board dashboard
    Then the dashboard should show 3 pending decisions
    And the dashboard should show 1 overdue decision
    And the dashboard should show my mandate expiring soon

  Scenario: Board member sees deadline approaching alerts
    Given a building with 25 units exists
    And I am a board member for that building
    And a decision has a deadline in 5 days
    When I view my board alerts
    Then I should see 1 approaching deadline alert
    And the alert urgency should be "critical"

  Scenario: Non-board member cannot access board dashboard
    Given a building with 25 units exists
    And I am an owner (not a board member)
    When I try to access the board dashboard
    Then I should receive a 403 Forbidden error

  Scenario: Dashboard filters decisions by building
    Given a building with 25 units exists
    And I am a board member for that building
    And another building exists with its own decisions
    When I request my board dashboard
    Then I should only see decisions from my building
    And I should not see decisions from other buildings

  Scenario: Dashboard shows empty state when no alerts
    Given a building with 25 units exists
    And I am a board member for that building
    And all decisions are completed on time
    When I request my board dashboard
    Then the dashboard should show 0 overdue decisions
    And the dashboard should show 0 pending decisions
