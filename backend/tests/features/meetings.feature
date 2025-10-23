Feature: Meetings Management
  As a property manager
  I want to manage meetings
  So that assemblies are tracked

  Scenario: Create a meeting
    Given a coproperty management system
    When I create a meeting titled "AG Ordinaire"
    Then the meeting should exist

