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

  # BUG-WF14-2 (Human Review v0.1.0, 2026-04-01) — owner-level building isolation.
  # An owner who owns a unit in only ONE building must not see the OTHER
  # buildings of the same organization. Fixed by commit dddde26; this scenario
  # is the @security regression guard proving the isolation holds.
  @security
  Scenario: Owner sees only buildings where they own a unit (BUG-WF14-2)
    Given an organization with three buildings
    And an owner Alice who owns a unit only in the first building
    When Alice lists buildings scoped to her ownership
    Then Alice sees exactly 1 building
    And Alice does not see the other two buildings

