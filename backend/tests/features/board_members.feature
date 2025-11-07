Feature: Board of Directors (Conseil de Copropriété)
  As a property owner
  I want to participate in the board of directors
  So that I can monitor the syndic and help manage the building

  Background:
    Given a coproperty management system
    And a building "Résidence Bellevue" with 30 units
    And an owner "Pierre Dupont" owning unit 101
    And an owner "Marie Martin" owning unit 102
    And an owner "Jacques Durand" owning unit 103
    And a meeting "AG Annuelle 2024" for building "Résidence Bellevue"

  Scenario: Elect a board president
    When I elect "Pierre Dupont" as board president for building "Résidence Bellevue" at meeting "AG Annuelle 2024"
    Then the board member should have position "president"
    And the mandate should be active
    And the mandate duration should be approximately 1 year

  Scenario: Elect multiple board members
    When I elect "Pierre Dupont" as board president for building "Résidence Bellevue" at meeting "AG Annuelle 2024"
    And I elect "Marie Martin" as board treasurer for building "Résidence Bellevue" at meeting "AG Annuelle 2024"
    And I elect "Jacques Durand" as board member for building "Résidence Bellevue" at meeting "AG Annuelle 2024"
    Then the building should have 3 active board members
    And "Pierre Dupont" should be board president
    And "Marie Martin" should be board treasurer
    And "Jacques Durand" should be board member

  Scenario: Cannot elect same owner twice for same position
    Given "Pierre Dupont" is elected as board president for building "Résidence Bellevue"
    When I try to elect "Pierre Dupont" as board president again
    Then the election should fail with error "already has this position"

  Scenario: Renew board member mandate
    Given "Pierre Dupont" is elected as board president for building "Résidence Bellevue"
    And the mandate expires in 50 days
    And a meeting "AG 2025" for building "Résidence Bellevue"
    When I renew the mandate of "Pierre Dupont" at meeting "AG 2025"
    Then the new mandate should start after the current one
    And the mandate duration should be approximately 1 year

  Scenario: Cannot renew mandate too early
    Given "Pierre Dupont" is elected as board president for building "Résidence Bellevue"
    And the mandate expires in 200 days
    When I try to renew the mandate
    Then the renewal should fail with error "Cannot extend mandate more than 60 days before expiration"

  Scenario: Remove board member
    Given "Pierre Dupont" is elected as board president for building "Résidence Bellevue"
    When I remove "Pierre Dupont" from the board
    Then "Pierre Dupont" should no longer be an active board member

  Scenario: View active board members
    Given "Pierre Dupont" is elected as board president for building "Résidence Bellevue"
    And "Marie Martin" is elected as board treasurer for building "Résidence Bellevue"
    When I list active board members for building "Résidence Bellevue"
    Then I should see 2 active board members
    And "Pierre Dupont" should be in the list
    And "Marie Martin" should be in the list

  Scenario: Board member can view their mandates
    Given a user "pierre@example.com" linked to owner "Pierre Dupont"
    And "Pierre Dupont" is elected as board president for building "Résidence Bellevue"
    When user "pierre@example.com" requests their mandates
    Then they should see 1 active mandate
    And the mandate should be for building "Résidence Bellevue"

  Scenario: Mandate with invalid duration fails
    When I try to elect "Pierre Dupont" with a mandate of 200 days
    Then the election should fail with error "Mandate duration must be approximately 1 year"

  Scenario: Mandate start date after end date fails
    When I try to elect "Pierre Dupont" with start date after end date
    Then the election should fail with error "Mandate start date must be before end date"

  Scenario: View board statistics
    Given "Pierre Dupont" is elected as board president for building "Résidence Bellevue"
    And "Marie Martin" is elected as board treasurer for building "Résidence Bellevue"
    And "Jacques Durand" is elected as board member for building "Résidence Bellevue"
    When I request board statistics for building "Résidence Bellevue"
    Then the stats should show 3 active members
    And the stats should show 1 president
    And the stats should show 1 treasurer
    And the stats should show 1 regular member

  Scenario: LEGAL - Syndic cannot be board member (incompatibility)
    Given a user "syndic@example.com" with role "syndic" for organization
    And an owner "Jean Syndic" linked to user "syndic@example.com"
    When I try to elect "Jean Syndic" as board member
    Then the election should fail with error "syndic cannot be a board member"

  Scenario: LEGAL - Board member cannot become syndic (reciprocal incompatibility)
    Given an owner "Pierre Dupont" owning unit 101
    And "Pierre Dupont" is elected as board president for building "Résidence Bellevue"
    And a user "pierre@example.com" linked to owner "Pierre Dupont"
    When I try to assign role "syndic" to user "pierre@example.com"
    Then the role assignment should fail with error "board member cannot be a syndic"
