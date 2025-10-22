Feature: Building Management
  As a property manager
  I want to manage buildings in the system
  So that I can track coproperty properties

  Scenario: Create a new building
    Given a coproperty management system
    When I create a building named "Résidence Les Jardins" in "Paris"
    Then the building should be created successfully
    And the building should be in "Paris"

  Scenario: Create multiple buildings
    Given a coproperty management system
    When I create a building named "Building A" in "Lyon"
    And I create a building named "Building B" in "Marseille"
    Then the building should be created successfully
