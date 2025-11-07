Feature: Board Decisions Tracking
  As a board member
  I want to track and monitor AG decisions
  So that I can ensure the syndic executes them on time

  Background:
    Given a coproperty management system
    And a building "Résidence Bellevue" with 30 units
    And an owner "Pierre Dupont" owning unit 101
    And "Pierre Dupont" is elected as board president for building "Résidence Bellevue"
    And a meeting "AG Annuelle 2024" for building "Résidence Bellevue"

  Scenario: Create a decision to track
    When I create a decision "Réparer ascenseur" for meeting "AG Annuelle 2024"
    Then the decision should have status "pending"
    And the decision should be assigned to meeting "AG Annuelle 2024"

  Scenario: Create decision with deadline
    When I create a decision "Obtenir 3 devis toiture" with deadline in 30 days
    Then the decision should have a deadline
    And the decision should not be overdue

  Scenario: Update decision status from pending to in progress
    Given a decision "Réparer ascenseur" with status "pending"
    When I update the decision status to "in_progress"
    Then the decision should have status "in_progress"

  Scenario: Mark decision as completed
    Given a decision "Réparer ascenseur" with status "in_progress"
    When I complete the decision
    Then the decision should have status "completed"
    And the decision should have a completion date

  Scenario: Cannot modify completed decision
    Given a decision "Réparer ascenseur" with status "completed"
    When I try to update the decision status to "pending"
    Then the status update should fail with error "Cannot change status of a completed decision"

  Scenario: Add notes to decision
    Given a decision "Réparer ascenseur"
    When I add notes "Le syndic a confirmé le début des travaux pour le 15 mars"
    Then the decision should have the notes

  Scenario: Detect overdue decision
    Given a decision "Obtenir devis" with deadline 5 days ago
    When I check if the decision is overdue
    Then the decision should be overdue
    And the decision status should be "overdue"

  Scenario: Decision without deadline is never overdue
    Given a decision "Étude faisabilité" with no deadline
    When I check if the decision is overdue
    Then the decision should not be overdue

  Scenario: Completed decision is never overdue
    Given a decision "Travaux terminés" that was completed yesterday
    And the original deadline was 5 days ago
    When I check if the decision is overdue
    Then the decision should not be overdue

  Scenario: Cancel a decision
    Given a decision "Installer panneaux solaires" with status "pending"
    When I update the decision status to "cancelled"
    Then the decision should have status "cancelled"

  Scenario: Cannot modify cancelled decision
    Given a decision "Projet annulé" with status "cancelled"
    When I try to update the decision status to "in_progress"
    Then the status update should fail with error "Cannot change status of a cancelled decision"

  Scenario: List decisions by status
    Given a decision "Décision 1" with status "pending"
    And a decision "Décision 2" with status "in_progress"
    And a decision "Décision 3" with status "completed"
    When I list decisions with status "pending"
    Then I should see 1 decision
    And "Décision 1" should be in the list

  Scenario: List all decisions for a building
    Given a decision "Décision A" for building "Résidence Bellevue"
    And a decision "Décision B" for building "Résidence Bellevue"
    When I list all decisions for building "Résidence Bellevue"
    Then I should see 2 decisions

  Scenario: List overdue decisions only
    Given a decision "En retard 1" with deadline 10 days ago and status "pending"
    And a decision "En retard 2" with deadline 5 days ago and status "in_progress"
    And a decision "À temps" with deadline in 30 days and status "pending"
    When I list overdue decisions for building "Résidence Bellevue"
    Then I should see 2 decisions
    And "En retard 1" should be in the list
    And "En retard 2" should be in the list

  Scenario: View decision statistics
    Given 3 decisions with status "pending"
    And 2 decisions with status "in_progress"
    And 5 decisions with status "completed"
    And 1 decision with status "overdue"
    When I request decision statistics for building "Résidence Bellevue"
    Then the stats should show 3 pending decisions
    And the stats should show 2 in_progress decisions
    And the stats should show 5 completed decisions
    And the stats should show 1 overdue decision
    And the total should be 11 decisions

  Scenario: Invalid status transition
    Given a decision "Test" with status "pending"
    When I try to update the decision status to "completed"
    Then the status update should fail with error "Invalid status transition"

  Scenario: Valid status transitions
    Given a decision "Test" with status "pending"
    When I update the decision status to "in_progress"
    And I update the decision status to "completed"
    Then the decision should have status "completed"

  Scenario: Create decision with empty subject fails
    When I try to create a decision with empty subject
    Then the decision creation should fail with error "Decision subject cannot be empty"

  Scenario: Create decision with empty text fails
    When I try to create a decision with subject "Test" but empty text
    Then the decision creation should fail with error "Decision text cannot be empty"

  Scenario: Create decision with past deadline fails
    When I try to create a decision with deadline yesterday
    Then the decision creation should fail with error "Deadline must be in the future"

  Scenario: Board member monitors multiple decisions
    Given "Pierre Dupont" is board president
    And 5 decisions with various statuses
    When "Pierre Dupont" views the decisions dashboard
    Then they should see all 5 decisions
    And they should see statistics for each status
    And they should see alerts for overdue decisions
