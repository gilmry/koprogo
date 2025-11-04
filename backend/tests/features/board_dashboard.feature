Feature: Board Dashboard and Alerts
  As a board member
  I want a centralized dashboard
  So that I can monitor all board activities and receive alerts

  Background:
    Given a coproperty management system
    And a building "Résidence Bellevue" with 30 units
    And an owner "Pierre Dupont" owning unit 101
    And "Pierre Dupont" is elected as board president for building "Résidence Bellevue"
    And a meeting "AG Annuelle 2024" for building "Résidence Bellevue"

  Scenario: View board dashboard as board member
    Given 3 decisions with status "pending"
    And 2 decisions with status "in_progress"
    And 1 decision with status "completed"
    And 1 overdue decision
    When "Pierre Dupont" views their board dashboard
    Then the dashboard should show their current mandate
    And the dashboard should show decision statistics
    And the dashboard should show 1 overdue decision
    And the dashboard should show upcoming deadlines

  Scenario: Dashboard shows mandate details
    When "Pierre Dupont" views their board dashboard
    Then the dashboard should show position "president"
    And the dashboard should show mandate start date
    And the dashboard should show mandate end date
    And the dashboard should show building "Résidence Bellevue"

  Scenario: Dashboard alerts for expiring mandate
    Given "Pierre Dupont" mandate expires in 45 days
    When "Pierre Dupont" views their board dashboard
    Then the dashboard should show a mandate expiration alert
    And the alert should indicate 45 days remaining

  Scenario: Dashboard shows upcoming deadlines
    Given a decision "Devis chaudière" with deadline in 7 days
    And a decision "Réparation hall" with deadline in 15 days
    When "Pierre Dupont" views their board dashboard
    Then the dashboard should show upcoming deadlines
    And "Devis chaudière" should be flagged as urgent

  Scenario: Dashboard decision statistics breakdown
    Given 5 decisions with status "pending"
    And 3 decisions with status "in_progress"
    And 8 decisions with status "completed"
    And 2 decisions with status "overdue"
    When "Pierre Dupont" views their board dashboard
    Then the stats should show 5 pending
    And the stats should show 3 in_progress
    And the stats should show 8 completed
    And the stats should show 2 overdue
