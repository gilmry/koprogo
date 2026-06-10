# Feature: Role Assignment CRUD endpoints — Story B0bis (gap Story 3.1)
#
# Story 3.1 a livré l'entité `UserRoleAssignment` + repo + helpers RBAC mais
# n'a JAMAIS exposé un endpoint REST CRUD. Story B0bis comble ce gap pour
# débloquer Phase B FE Story B1 (`RoleAssignmentForm` / `RoleAssignmentList`).
#
# Routes:
#   - POST   /users/{user_id}/role-assignments
#   - GET    /users/{user_id}/role-assignments
#   - DELETE /users/{user_id}/role-assignments/{assignment_id}
#   - GET    /role-assignments?organization_id=&role=  (superadmin only)
#
# Wiring status: step definitions are pending (`bdd_role_assignment.rs`
# follow-up). Tagged `@wip` so the CI does not fail.

@wip
Feature: Role Assignment — CRUD REST endpoints
  As a superadmin or syndic
  I want to assign / list / revoke sub-roles for users
  So that I can administer the role separation invariants (INV-10 INV-8)

  Background:
    Given the system is initialized
    And an organization "Assignment ASBL" exists
    And a superadmin "admin@assignment.test" exists
    And a syndic "syndic@assignment.test" exists in organization "Assignment ASBL"
    And an owner "owner@assignment.test" exists in organization "Assignment ASBL"
    And an owner "stranger@other-org.test" exists in a different organization

  # === @happy ============================================================

  @happy
  Scenario: Superadmin assigns "accountant.encodeur" to an owner
    Given the superadmin is authenticated
    When the superadmin POSTs to "/users/{owner.id}/role-assignments" with role "accountant.encodeur"
    Then the response status is 201
    And the response body has field "role" equal to "accountant.encodeur"
    And the response body has field "user_id" equal to the owner's id
    And the response body has field "valid_until" equal to null

  @happy
  Scenario: Syndic assigns "community.moderator" to an owner of their organization
    Given the syndic is authenticated
    When the syndic POSTs to "/users/{owner.id}/role-assignments" with role "community.moderator"
    Then the response status is 201
    And the response body has field "role" equal to "community.moderator"

  @happy
  Scenario: User lists their own role assignments
    Given the owner is authenticated
    And the owner has a native assignment for role "owner"
    When the owner requests "GET /users/{self.id}/role-assignments"
    Then the response status is 200
    And the response is a JSON array with at least 1 element
    And every element has field "role"

  @happy
  Scenario: Syndic revokes an assignment they granted
    Given the syndic is authenticated
    And the syndic has previously assigned "accountant.encodeur" to the owner
    When the syndic DELETEs "/users/{owner.id}/role-assignments/{assignment.id}"
    Then the response status is 204
    And the owner no longer holds role "accountant.encodeur"

  # === @edge =============================================================

  @edge
  Scenario: Assigning a role with a future valid_until creates a delegated assignment
    Given the superadmin is authenticated
    When the superadmin POSTs to "/users/{owner.id}/role-assignments" with role "lawyer" and valid_until "+30d"
    Then the response status is 201
    And the response body has field "delegated_from_user_id" equal to the superadmin's id
    And the response body has field "valid_until" not equal to null

  @edge
  Scenario: Assigning a role with a past valid_until is rejected as validation error
    Given the superadmin is authenticated
    When the superadmin POSTs to "/users/{owner.id}/role-assignments" with role "lawyer" and valid_until "-1s"
    Then the response status is 400
    And the response body kind is "validation"

  @edge
  Scenario: Assigning a role with organization_id = null is accepted as a global assignment
    Given the superadmin is authenticated
    When the superadmin POSTs to "/users/{owner.id}/role-assignments" with role "community.moderator" and organization_id null
    Then the response status is 201
    And the response body has field "organization_id" equal to null

  # === @security =========================================================

  @security
  Scenario: An owner cannot assign roles to themselves nor other users
    Given the owner is authenticated
    When the owner POSTs to "/users/{owner.id}/role-assignments" with role "syndic"
    Then the response status is 403
    And the response body kind is "forbidden"

  @security
  Scenario: A user cannot self-grant the syndic role even via superadmin endpoint when they themselves call it
    Given the syndic is authenticated
    When the syndic POSTs to "/users/{syndic.id}/role-assignments" with role "syndic"
    Then the response status is 400
    And the response body kind is "validation"
    And the response body error message contains "self-grant"

  @security
  Scenario: A syndic from organization A cannot assign roles to a user in organization B
    Given the syndic is authenticated
    When the syndic POSTs to "/users/{stranger.id}/role-assignments" with role "accountant.encodeur"
    Then the response status is 403
    And the response body kind is "forbidden"

  @security
  Scenario: The admin-only list endpoint refuses non-superadmin callers
    Given the syndic is authenticated
    When the syndic requests "GET /role-assignments?role=syndic"
    Then the response status is 403
    And the response body kind is "forbidden"

  # === @negative =========================================================

  @negative
  Scenario: Assigning a role that already exists actively for the same (user, role, org) returns 409
    Given the superadmin is authenticated
    And the owner already has a native assignment for role "owner" in organization "Assignment ASBL"
    When the superadmin POSTs to "/users/{owner.id}/role-assignments" with role "owner" and organization_id "{Assignment ASBL.id}"
    Then the response status is 409
    And the response body kind is "role_already_assigned"

  @negative
  Scenario: Assigning an unknown role string is rejected with 400 (whitelist enforcement)
    Given the superadmin is authenticated
    When the superadmin POSTs to "/users/{owner.id}/role-assignments" with role "hackerman"
    Then the response status is 400
    And the response body kind is "validation"

  @negative
  Scenario: Revoking a non-existent assignment returns 404
    Given the superadmin is authenticated
    When the superadmin DELETEs "/users/{owner.id}/role-assignments/00000000-0000-0000-0000-000000000000"
    Then the response status is 404
    And the response body kind is "not_found"

  @negative
  Scenario: Assigning a role to an unknown user returns 404
    Given the superadmin is authenticated
    When the superadmin POSTs to "/users/00000000-0000-0000-0000-000000000000/role-assignments" with role "owner"
    Then the response status is 404
    And the response body kind is "not_found"
