# Feature: Syndic = community.moderator (Story 5.3 — #587, INV-4)
#
# A syndic without a co-owner record moderates community content (SEL,
# Notices, SharedObjects) — edit/cancel/delete, motivated — but does not
# participate personally (create/vote/comment) in their own name. A syndic
# who ALSO holds a unit in this ACP acts as a co-owner for participation:
# the right comes from the lot, not from the function (same reasoning as
# ADR-0052 for the accountant).
#
# Poll is deliberately absent from this file: INV-4 blocks personal VOTING,
# not poll creation (a syndic-initiated consultation stays legitimate — see
# `polls.feature` Scenario 8 for the pre-existing anonymous-vote behaviour
# this story must not regress). That refusal lives entirely in the HTTP
# handler (`poll_handlers::cast_poll_vote`, which resolves the caller's
# `Owner` record before ever calling the use case) because `PollVote.owner_id
# = None` already has a legitimate meaning (anonymous vote) that a use-case
# level check cannot tell apart from "no co-owner record" — see the doc
# comment on `PollUseCases::cast_vote`. Covered by unit tests in
# `poll_use_cases.rs`, not by this BDD harness.

Feature: Syndic moderates community content without personal participation
  As a co-owner
  I want the syndic to moderate community content instead of participating in it
  So that the syndic never acts as judge and party in exchanges they arbitrate

  Background:
    Given the system is initialized
    And an organization "Syndic Moderator ASBL" exists with id "org-syndic-mod"
    And a building "Residence Moderee" exists in organization "org-syndic-mod"

  # --- SEL (Local Exchange) ---

  @security
  Scenario: A pure syndic cannot create a SEL offer (INV-4)
    Given a syndic "Paul Syndic" exists without an owner record
    When the syndic "Paul Syndic" tries to create a service exchange offer titled "Coup de main"
    Then the SEL action should be forbidden

  @happy
  Scenario: A syndic moderator cancels a litigious SEL exchange with a reason
    Given an owner "Alice Voisine" exists in building "Residence Moderee"
    And the user is authenticated as owner "Alice Voisine"
    And I create a service exchange offer:
      | title       | Bricolage         |
      | description | Offre litigieuse  |
      | credits     | 2                 |
    And a syndic "Paul Syndic" exists without an owner record
    When the syndic "Paul Syndic" moderates the exchange with reason "Signalement d'un voisin"
    Then the exchange should be cancelled

  @negative
  Scenario: A syndic moderator cannot cancel a SEL exchange without a reason
    Given an owner "Alice Voisine" exists in building "Residence Moderee"
    And the user is authenticated as owner "Alice Voisine"
    And I create a service exchange offer:
      | title       | Bricolage |
      | description | Offre     |
      | credits     | 2         |
    And a syndic "Paul Syndic" exists without an owner record
    When the syndic "Paul Syndic" tries to moderate the exchange without a reason
    Then the SEL action should be refused for a missing reason

  @edge
  Scenario: A syndic who also owns a lot cancels their own exchange without a reason
    Given an owner "Denise Syndic-Copro" exists in building "Residence Moderee"
    And the user is authenticated as owner "Denise Syndic-Copro"
    And I create a service exchange offer:
      | title       | Offre du syndic-copropriétaire |
      | description | Offre personnelle              |
      | credits     | 1                               |
    When "Denise Syndic-Copro" cancels her own exchange as syndic without a reason
    Then the exchange should be cancelled

  # --- Notices ---

  @happy
  Scenario: A syndic moderator archives someone else's notice with a reason
    Given an owner "Bob Voisin" exists in building "Residence Moderee"
    And the user is authenticated as owner "Bob Voisin"
    And I create a notice:
      | title   | Annonce a moderer |
      | content | Contenu           |
    And I publish the notice
    And a syndic "Paul Syndic" exists without an owner record
    When the syndic "Paul Syndic" archives the notice with reason "Contenu inapproprie signale"
    Then the notice should be archived

  @negative
  Scenario: A syndic moderator cannot archive someone else's notice without a reason
    Given an owner "Bob Voisin" exists in building "Residence Moderee"
    And the user is authenticated as owner "Bob Voisin"
    And I create a notice:
      | title   | Annonce a moderer |
      | content | Contenu           |
    And I publish the notice
    And a syndic "Paul Syndic" exists without an owner record
    When the syndic "Paul Syndic" tries to archive the notice without a reason
    Then the notice action should be refused for a missing reason

  # --- SharedObjects ---

  @happy
  Scenario: A syndic moderator deletes a litigious shared object with a reason
    Given an owner "Carl Preteur" exists in building "Residence Moderee"
    And "Carl Preteur" shares an object:
      | name            | Perceuse suspecte |
      | description     | Signalee par un voisin |
      | category        | Tools             |
      | deposit_credits | 0                 |
      | max_loan_days   | 7                 |
    And a syndic "Paul Syndic" exists without an owner record
    When the syndic "Paul Syndic" deletes the shared object with reason "Objet dangereux signale"
    Then the shared object should be deleted

  @negative
  Scenario: A syndic moderator cannot delete a shared object without a reason
    Given an owner "Carl Preteur" exists in building "Residence Moderee"
    And "Carl Preteur" shares an object:
      | name            | Perceuse       |
      | description     | Description    |
      | category        | Tools          |
      | deposit_credits | 0              |
      | max_loan_days   | 7               |
    And a syndic "Paul Syndic" exists without an owner record
    When the syndic "Paul Syndic" tries to delete the shared object without a reason
    Then the shared object action should be refused for a missing reason
