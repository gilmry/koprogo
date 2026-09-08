# Feature: Magic Link — Public-access tokens (Story 3.2 — FR6 INV-13 INV-17)
#
# A syndic / superadmin issues a single-use, time-bounded token that grants
# a recipient (typically an external contractor) read access to a single
# ticket / quote / invoice / contractor-evaluation WITHOUT requiring the
# recipient to create an account.
#
# Wiring status: step definitions for this feature are pending
# (`bdd_magic_link.rs` follow-up). Tagged `@wip` so the CI does not fail.

@wip
Feature: Magic Link — secure public access via short-lived tokens
  As a syndic
  I want to share a single-use link to a ticket / quote / invoice / evaluation
  So that a contractor or third party can consult it without an account

  Background:
    Given the system is initialized
    And an organization "Magic ASBL" exists
    And a syndic "syndic@magic.test" exists
    And a contractor user "plombier@external.test" exists
    And a ticket "Fuite chaudière" exists in the building

  # === @happy ============================================================

  @happy
  Scenario: Syndic issues a link for a contractor and they open it once
    When the syndic issues a magic link for the ticket valid for 7 days
    Then a clear token is returned in the response
    And the response status is 201
    When the contractor opens "/c/<token>"
    Then the response status is 200
    And the payload scope_kind is "ticket"
    And the payload scope_id matches the ticket

  # === @edge =============================================================

  @edge
  Scenario: Link used exactly at TTL boundary still resolves once
    Given a magic link exists with TTL 60 seconds
    When the contractor opens "/c/<token>" 59 seconds after issue
    Then the response status is 200

  @edge
  Scenario: Second attempt on the same link is rejected
    Given a magic link has already been consumed
    When the contractor opens "/c/<token>" again
    Then the response status is 403
    And the response kind is "magic_link_consumed"

  # === @security =========================================================

  @security
  Scenario: Forged token returns 403 magic_link_invalid (no enumeration)
    When the contractor opens "/c/forged-token-not-in-db"
    Then the response status is 403
    And the response kind is "magic_link_invalid"

  @security
  Scenario: Magic link cannot be issued for a self-recipient
    When the syndic issues a magic link where subject_user_id equals the syndic id
    Then the response status is 400
    And the response kind is "validation"

  @security
  Scenario: Only syndic / superadmin can issue magic links
    Given an owner "owner@magic.test" is logged in
    When the owner issues a magic link for the ticket
    Then the response status is 403
    And the response kind is "forbidden"

  # === @negative =========================================================

  @negative
  Scenario: Expired link returns 403 with French message
    Given a magic link exists with expires_at one minute in the past
    When the contractor opens "/c/<token>"
    Then the response status is 403
    And the response kind is "magic_link_expired"
    And the response message contains "demandez-en un nouveau au syndic"

  @negative
  Scenario: TTL below minimum is rejected
    When the syndic issues a magic link with expires_in_seconds 30
    Then the response status is 400
    And the response kind is "validation"
