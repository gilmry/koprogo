# Feature: Mandate — Juridical delegation tracker (Story 3.4 — FR7 INV-14)
#
# A syndic issues a `Mandate` to materialise a juridical/technical
# delegation to an external professional (notaire, avocat, AMO, architecte,
# BET, gardien). The mandate carries mandatory temporal validity
# (`valid_until`) and is bound to a scope (Building or whole ACP).
#
# Wiring status: step definitions are pending (`bdd_mandate.rs` follow-up).
# Tagged `@wip` so the CI does not fail.

@wip
Feature: Mandate — secure delegation with temporal validity
  As a syndic
  I want to issue a time-bounded mandate to an external professional
  So that the platform enforces scope + expiry on every action they take

  Background:
    Given the system is initialized
    And an organization "Mandate ASBL" exists
    And a syndic "syndic@mandate.test" exists
    And a notary "notaire@etude.test" with role "notary" exists
    And a building "Résidence Lilas" exists in the ACP

  # === @happy ============================================================

  @happy
  Scenario: Syndic issues a notarial mandate scoped to one building
    When the syndic issues a mandate for the notary
      | kind       | notary                                |
      | scope_kind | building                              |
      | reason     | Cession unité C12 — acte authentique  |
      | valid_until| +60d                                  |
    Then the response status is 201
    And the response contains the mandate id
    And the mandate is currently active

  @happy
  Scenario: Subject lists their own active mandates
    Given a notarial mandate has been issued for the notary
    When the notary requests "GET /mandates?subject=<self>"
    Then the response status is 200
    And the response contains exactly 1 mandate

  # === @edge =============================================================

  @edge
  Scenario: Issuing a mandate at the maximum 5-year boundary is accepted
    When the syndic issues a mandate with valid_until "+5y"
    Then the response status is 201

  @edge
  Scenario: Revoking an already-revoked mandate is idempotent
    Given a notarial mandate has been revoked
    When the syndic revokes the same mandate again
    Then the response status is 204

  # === @security =========================================================

  @security
  Scenario: An owner cannot issue a mandate
    Given an owner "owner@mandate.test" is logged in
    When the owner attempts to issue a mandate for the notary
    Then the response status is 403
    And the response kind is "forbidden"

  @security
  Scenario: A mandataire acting on the wrong building is refused
    Given a notarial mandate scoped to building A has been issued for the notary
    When the notary acts on building B as a mandataire
    Then the response status is 403
    And the response kind is "mandate_invalid_scope"

  # === @negative =========================================================

  @negative
  Scenario: Mandate without valid_until is rejected
    When the syndic issues a mandate without "valid_until"
    Then the response status is 400
    And the response kind is "validation"

  @negative
  Scenario: Expired mandate triggers a 403 mandate_expired
    Given a notarial mandate has expired one minute ago
    When the notary acts as a mandataire
    Then the response status is 403
    And the response kind is "mandate_expired"

  @negative
  Scenario: Mandate duration > 5 years is rejected at issue time
    When the syndic issues a mandate with valid_until "+5y1d"
    Then the response status is 400
    And the response kind is "validation"
