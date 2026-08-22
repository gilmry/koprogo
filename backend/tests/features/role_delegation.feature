# Feature: Role Delegation — Temporary role delegation (Story 3.5 — FR8 INV-8)
#
# A syndic delegates their role to an Owner for N days. The assignment
# auto-expires via `valid_until`. Re-delegation of a delegated role is
# forbidden (non-transitive anti-bypass invariant INV-8).
#
# Wiring status: step definitions are pending (`bdd_role_delegation.rs`
# follow-up). Tagged `@wip` so the CI does not fail.

@wip
Feature: Role Delegation — bounded, non-transitive temporary delegation
  As a syndic
  I want to delegate my role to a trusted owner for a bounded duration
  So that they can act on my behalf while I am unavailable

  Background:
    Given the system is initialized
    And an organization "Delegation ASBL" exists
    And a syndic "syndic@delegation.test" exists
    And an owner "owner@delegation.test" exists
    And a second owner "owner-b@delegation.test" exists

  # === @happy ============================================================

  @happy
  Scenario: Syndic delegates their role to an owner for 7 days
    When the syndic delegates role "syndic" to the owner with valid_until "+7d"
    Then the response status is 201
    And the response contains the delegation id
    And the delegation is currently active
    And the delegation delegated_from_user_id is the syndic

  @happy
  Scenario: Subject lists their own active delegations
    Given a delegation of role "syndic" has been issued to the owner
    When the owner requests "GET /role-delegations?subject=<self>"
    Then the response status is 200
    And the response contains exactly 1 delegation

  # === @edge =============================================================

  @edge
  Scenario: Delegation at the maximum 90-day boundary is accepted
    When the syndic delegates role "syndic" with valid_until "+90d"
    Then the response status is 201

  @edge
  Scenario: Revoking a delegation removes it from the active list
    Given a delegation of role "syndic" has been issued to the owner
    When the syndic revokes the delegation
    Then the response status is 204
    And the active delegations list for the owner is empty

  # === @security =========================================================

  @security
  Scenario: An owner without the role cannot delegate it
    Given the owner has no "syndic" role
    When the owner attempts to delegate role "syndic" to owner B
    Then the response status is 403
    And the response kind is "delegation_chain_not_allowed"

  @security
  Scenario: A user that received the role by delegation cannot re-delegate it
    Given a delegation of role "syndic" has been issued to the owner
    When the owner attempts to delegate role "syndic" to owner B
    Then the response status is 403
    And the response kind is "delegation_chain_not_allowed"

  @security
  Scenario: Self-delegation is rejected
    When the syndic delegates role "syndic" to themselves
    Then the response status is 400
    And the response kind is "validation"

  # === @negative =========================================================

  @negative
  Scenario: Delegation with valid_until in the past is rejected
    When the syndic delegates role "syndic" with valid_until "-1d"
    Then the response status is 400
    And the response kind is "validation"

  @negative
  Scenario: Delegation longer than 90 days is rejected
    When the syndic delegates role "syndic" with valid_until "+91d"
    Then the response status is 400
    And the response kind is "validation"

  @negative
  Scenario: Delegation to a user that already holds the role returns 409
    Given owner B already has the "syndic" role natively
    When the syndic delegates role "syndic" to owner B with valid_until "+7d"
    Then the response status is 409
    And the response kind is "role_already_assigned"
