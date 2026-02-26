# Feature: Organization & User Management (SuperAdmin)
# Subscription plans: free, starter, professional, enterprise

Feature: Organization and User Management
  As a SuperAdmin
  I want to manage organizations and users
  So that I can control platform access and subscriptions

  Background:
    Given the system is initialized
    And I am authenticated as SuperAdmin

  # === ORGANIZATIONS ===

  Scenario: Create an organization
    When I create an organization:
      | name            | Copro Bruxelles ASBL     |
      | slug            | copro-bruxelles          |
      | contact_email   | contact@copro-bxl.be     |
      | subscription    | starter                  |
      | max_buildings   | 5                        |
      | max_users       | 20                       |
    Then the organization should be created
    And it should be active

  Scenario: Update an organization
    Given an organization "Copro Test" exists
    When I update the organization name to "Copro Updated"
    Then the organization name should be "Copro Updated"

  Scenario: Suspend an organization
    Given an active organization "Copro Suspend" exists
    When I suspend the organization
    Then the organization should be inactive

  Scenario: Activate a suspended organization
    Given a suspended organization "Copro Reactive" exists
    When I activate the organization
    Then the organization should be active

  Scenario: Delete an organization
    Given an organization "Copro Delete" exists
    When I delete the organization
    Then the organization should be deleted

  Scenario: List all organizations
    Given 3 organizations exist
    When I list all organizations
    Then I should get 3 organizations

  # === USERS ===

  Scenario: Create a user with role assignments
    Given an organization exists
    When I create a user:
      | email      | newuser@test.be          |
      | first_name | Jean                     |
      | last_name  | Nouveau                  |
      | role       | Owner                    |
    Then the user should be created
    And the user should have role "Owner"

  Scenario: Deactivate a user
    Given a user "deactivate@test.be" exists
    When I deactivate the user
    Then the user should be inactive
