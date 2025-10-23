Feature: Meetings update, completion, cancellation and listing
  As a property manager
  I want to manage meeting lifecycle and list meetings by building
  So that assemblies are kept up-to-date

  Background:
    Given a coproperty management system
    And I create a meeting titled "AG Lifecycle"

  Scenario: Update a meeting
    When I update the last meeting title to "AG Updated" and location to "Salle B"
    Then the meeting update should succeed

  Scenario: Complete a meeting
    When I complete the last meeting with 42 attendees
    Then the meeting completion should succeed

  Scenario: Cancel a meeting
    When I cancel the last meeting
    Then the meeting cancellation should succeed

  Scenario: List meetings by building
    When I list meetings for the building
    Then I should get at least 1 meeting

