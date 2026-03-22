Feature: Multi-tenancy isolation
  As a platform
  I want to isolate tenant data
  So that organizations cannot access each other's data

  Scenario: List buildings filtered by organization
    Given a coproperty management system with two organizations
    When I list buildings for the first organization
    Then I should not see buildings from the second organization

  # Bug #B2 — Tests E2E 22/03/2026

  Scenario: Get unit by ID must verify organization ownership
    Given a coproperty management system with two organizations
    And organization A has a building with a unit
    When a user from organization B requests that unit by ID
    Then the response should be 403 Forbidden

  Scenario: List units by building must verify organization ownership
    Given a coproperty management system with two organizations
    And organization A has a building with units
    When a user from organization B lists units for that building
    Then the response should be 403 Forbidden

  Scenario: List paginated units must filter by organization
    Given a coproperty management system with two organizations
    And each organization has units in their buildings
    When a user from organization A lists all units
    Then only units from organization A are returned
    And no units from organization B are visible

  Scenario: SuperAdmin can see all units across organizations
    Given a coproperty management system with two organizations
    And each organization has units in their buildings
    When a superadmin lists all units without organization filter
    Then units from both organizations are returned

