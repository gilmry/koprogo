# Feature: SyndicResponse + SLA tracking (Story 3.7 — FR32, INV-23)
#
# Story 3.7 introduces an append-only `SyndicResponse` relation (the syndic
# may answer a ticket; replies cannot be edited nor deleted — INV-23) and a
# severity-based SLA window for the syndic's first response. A cron job
# (`SlaEscalationJob`) flags tickets that exceeded the deadline without a
# response.
#
# Wiring status: step definitions for this feature are pending
# (`bdd_syndic_response_sla.rs` follow-up). Tagged `@wip` so CI does not
# fail — the use-cases / entity / migration ship in Phase A but the
# Cucumber glue lands in Phase B.

@wip
Feature: SyndicResponse — append-only reply + SLA escalation
  As a syndic
  I want to reply to a ticket with a structured, immutable response
  So that copropriétaires have an audit-grade record of my actions
  And the system flags tickets I did not address in time

  Background:
    Given the system is initialized
    And an organization "Réponses ASBL" exists
    And a syndic "syndic@reponses.test" exists
    And an owner "owner@reponses.test" is logged in
    And a building "Résidence Mimosas" exists in the ACP

  # === @happy ============================================================

  @happy
  Scenario: Syndic replies to a critical complaint within 24h SLA window
    Given the owner filed a complaint with severity "critical" 6 hours ago
    When the syndic posts a response with body "Inspection planifiée pour demain matin."
      And action "schedule_inspection"
    Then the response is saved with status 201
    And the ticket's sla_escalated_at is set (SLA consumed in time)

  @happy
  Scenario: Anyone in the ACP can list the responses of a ticket
    Given the syndic posted a response on the owner's complaint
    When the owner lists responses for the ticket
    Then the response is visible in the list, oldest first

  # === @edge =============================================================

  @edge
  Scenario: Response with exactly 10 chars body is accepted
    Given the owner filed a complaint with severity "normal"
    When the syndic posts a response with body "0123456789"
    Then the response is saved with status 201

  @edge
  Scenario: Response posted after sla_due_at does not mark ticket as escalated by the use-case
    Given the owner filed a complaint with severity "critical" 30 hours ago (SLA already over)
    When the syndic posts a response with body "Retour tardif, désolé du délai."
    Then the response is saved with status 201
    But the ticket's sla_escalated_at remains untouched by the response use-case
    # (the cron job is responsible for setting it for late tickets)

  # === @security =========================================================

  @security
  Scenario: An owner cannot post a SyndicResponse (RBAC)
    Given the owner is logged in (role "owner")
    When the owner POSTs a syndic-response on the ticket
    Then the API returns status 403 with kind "forbidden"

  @security
  Scenario: A response cannot be edited (append-only, INV-23)
    Given the syndic posted a response on the owner's complaint
    When the syndic attempts to update the response body via direct SQL
    Then the database raises an error matching "append-only"
    # In-app there is no PATCH/PUT endpoint on SyndicResponse — same INV-23.

  @security
  Scenario: SLA escalation cron is idempotent across passes
    Given 3 tickets exceeded their sla_due_at without a syndic response
    When the SLA escalation job runs once
    Then 3 tickets are marked as sla_escalated
    When the SLA escalation job runs again at the same instant
    Then 0 additional tickets are marked

  # === @negative =========================================================

  @negative
  Scenario: Body shorter than 10 chars is rejected with validation error
    Given the owner filed a complaint with severity "normal"
    When the syndic posts a response with body "trop"
    Then the API returns status 400 with kind "validation"

  @negative
  Scenario: Action proposed outside the whitelist is rejected
    Given the owner filed a complaint with severity "normal"
    When the syndic posts a response with action "rm_minus_rf"
    Then the API returns status 400 with kind "validation"

  @negative
  Scenario: POST a response on an unknown ticket id returns 404
    When the syndic posts a response on ticket id "00000000-0000-0000-0000-000000000000"
    Then the API returns status 404 with kind "not_found"
