Feature: Multi-tenancy isolation
  As a platform
  I want to isolate tenant data
  So that organizations cannot access each other's data

  Scenario: List buildings filtered by organization
    Given a coproperty management system with two organizations
    When I list buildings for the first organization
    Then I should not see buildings from the second organization

