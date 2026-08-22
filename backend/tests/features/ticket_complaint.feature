# Feature: Ticket — Complaint extension + INV-24 (Story 3.6 — FR31, brief C17)
#
# Story 3.6 extends Ticket with a `kind` (Request | Complaint), a `severity`
# tier, an optional `incident_date`, `evidence_attachments` (≤ 10) and
# `witnesses` (≤ 10, no duplicates). The Ticket becomes immutable to direct
# field edits 5 minutes after `created_at` (INV-24); subsequent changes go
# through the dedicated workflow endpoints (assign / resolve / cancel / …).
#
# Wiring status: step definitions for this feature are pending
# (`bdd_ticket_complaint.rs` follow-up). Tagged `@wip` so the CI does not fail.

@wip
Feature: Ticket — Complaint with evidence and witnesses + 5-min immutability
  As an owner of an ACP
  I want to file a formal complaint with evidence and witnesses
  So that the syndic can triage and act with full context

  Background:
    Given the system is initialized
    And an organization "Plainte ASBL" exists
    And a syndic "syndic@plainte.test" exists
    And an owner "owner@plainte.test" is logged in
    And a building "Résidence Acacias" exists in the ACP
    And two other owners "temoin1@plainte.test" and "temoin2@plainte.test" exist

  # === @happy ============================================================

  @happy
  Scenario: Owner files a critical complaint with 3 evidence URLs and 2 witnesses
    When the owner creates a ticket with payload
      | title       | Bruit nocturne récurrent          |
      | description | Tapage du 3e étage, plaintes 3sem |
      | category    | Other                             |
      | priority    | High                              |
      | kind        | complaint                         |
      | severity    | critical                          |
      | evidence    | 3                                 |
      | witnesses   | 2                                 |
    Then the response status is 201
    And the response payload kind is "complaint"
    And the response payload severity is "critical"
    And the response payload evidence_attachments has 3 items
    And the response payload witnesses has 2 items

  @happy
  Scenario: Owner edits their ticket title within the 5-min window
    Given the owner created a ticket 2 minutes ago
    When the owner sends PATCH "/tickets/<id>" with a new title
    Then the response status is 200
    And the response payload title matches the new title

  # === @edge =============================================================

  @edge
  Scenario: Pre-3.6 client without kind defaults to Request and stays valid
    When the owner creates a ticket without "kind"
    Then the response status is 201
    And the response payload kind is "request"
    And the response payload evidence_attachments is an empty array

  @edge
  Scenario: Exactly 10 evidence attachments is accepted
    When the owner creates a complaint with 10 evidence URLs
    Then the response status is 201

  @edge
  Scenario: Editing exactly at 4m59s after creation still succeeds
    Given the owner created a ticket 4m59s ago
    When the owner sends PATCH "/tickets/<id>" with a new description
    Then the response status is 200

  # === @security =========================================================

  @security
  Scenario: PATCH after 5 minutes returns 403 ticket_immutable
    Given the owner created a ticket 6 minutes ago
    When the owner sends PATCH "/tickets/<id>" with a new title
    Then the response status is 403
    And the response kind is "ticket_immutable"
    And the response message contains "verrouillé"

  @security
  Scenario: Witnesses cannot include the ticket creator (self-witnessing)
    When the owner creates a complaint listing themselves as a witness
    Then the response status is 400
    And the response kind is "validation"

  @security
  Scenario: Duplicate witnesses are rejected
    When the owner creates a complaint with the same witness id twice
    Then the response status is 400
    And the response kind is "validation"

  # === @negative =========================================================

  @negative
  Scenario: Complaint without severity is rejected
    When the owner creates a ticket with kind "complaint" and no severity
    Then the response status is 400
    And the response kind is "validation"
    And the response message contains "severity"

  @negative
  Scenario: 11 evidence attachments are rejected
    When the owner creates a complaint with 11 evidence URLs
    Then the response status is 400
    And the response kind is "validation"

  @negative
  Scenario: 11 witnesses are rejected
    When the owner creates a complaint with 11 distinct witness user_ids
    Then the response status is 400
    And the response kind is "validation"
