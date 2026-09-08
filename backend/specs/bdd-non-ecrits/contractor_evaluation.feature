# Feature: ContractorEvaluation — append-only contractor rating gated by an
# approved TechnicalSpec (Story 3.9 — FR34 FR35 INV-21 INV-24).
#
# Story 3.9 introduces an audit-grade evaluation flow distinct from the
# legacy marketplace rating (ContractEvaluation, Issue #276). Three hard
# invariants:
#
#   - the referenced TechnicalSpec MUST be in status `approved` (FR34)
#     — otherwise the API replies 422 with kind `technical_spec_required`;
#   - the evaluator MUST NOT be the contractor being evaluated (INV-21)
#     — otherwise 422 with kind `evaluator_is_contractor`;
#   - rows are append-only — a DB trigger blocks UPDATE / DELETE (INV-24).
#
# Wiring status: step definitions for this feature are pending
# (`bdd_contractor_evaluation.rs` follow-up). Tagged `@wip` so CI does not
# fail — entity / use-cases / handlers / migration ship in Phase A; the
# Cucumber glue lands in Phase B alongside the FE work.

@wip
Feature: ContractorEvaluation — append-only, gated by an approved TechnicalSpec
  As a syndic
  I want to record an audit-grade evaluation of a contractor's prestation
   once the cahier des charges they worked under has been signed off
  So that the ACP has a defensible reputation trail for future works

  Background:
    Given the system is initialized
    And an organization "Evals ASBL" exists
    And a syndic "syndic@evals.test" exists
    And an ACP "ACP Evals" exists in the organization
    And a building "Residence Tilleuls" exists in the ACP
    And a contractor "contractor@evals.test" exists with role "contractor"

  # === @happy ============================================================

  @happy
  Scenario: Syndic records an evaluation against an approved spec
    Given an Approved TechnicalSpec exists for the ACP
    When the syndic POSTs a ContractorEvaluation with
      contractor "contractor@evals.test",
      scores quality=5 timeliness=5 communication=4 cost_compliance=4 overall=5,
      comment "Travail soigné, livré dans les délais, communication impeccable."
    Then the evaluation is created with status 201
    And the response body average_score is 4.6

  @happy
  Scenario: Two evaluations against the same contractor list newest first
    Given an Approved TechnicalSpec exists for the ACP
    And the syndic has recorded an evaluation for "contractor@evals.test"
    When the syndic records a second evaluation for "contractor@evals.test"
    Then GET /contractors/{contractor_user_id}/evaluations returns 2 entries
    And the first entry was created more recently than the second

  # === @edge =============================================================

  @edge
  Scenario: Comment exactly at the lower bound (10 chars) is accepted
    Given an Approved TechnicalSpec exists for the ACP
    When the syndic POSTs an evaluation with comment "0123456789"
    Then the evaluation is created with status 201

  @edge
  Scenario: 20 linked_ticket_ids (the upper bound) is accepted
    Given an Approved TechnicalSpec exists for the ACP
    When the syndic POSTs an evaluation with 20 linked ticket ids
    Then the evaluation is created with status 201

  @edge
  Scenario: Empty linked_ticket_ids is accepted
    Given an Approved TechnicalSpec exists for the ACP
    When the syndic POSTs an evaluation with no linked ticket ids
    Then the evaluation is created with status 201

  # === @security =========================================================

  @security
  Scenario: An owner without role cannot record an evaluation (RBAC)
    Given an owner "owner@evals.test" is logged in (role "owner")
    And an Approved TechnicalSpec exists for the ACP
    When the owner POSTs an evaluation
    Then the API returns status 403 with kind "forbidden"

  @security
  Scenario: A contractor cannot evaluate themselves (INV-21)
    Given an Approved TechnicalSpec exists for the ACP
    And the contractor "contractor@evals.test" is logged in as both evaluator and contractor
    When the syndic POSTs an evaluation where evaluator_user_id == contractor_user_id
    Then the API returns status 422 with kind "evaluator_is_contractor"

  @security
  Scenario: An evaluation cannot be edited (append-only, DB trigger)
    Given a ContractorEvaluation row exists
    When a direct SQL UPDATE on the row is attempted
    Then the database raises an error matching "append-only"

  @security
  Scenario: An evaluation cannot be deleted (append-only, DB trigger)
    Given a ContractorEvaluation row exists
    When a direct SQL DELETE on the row is attempted
    Then the database raises an error matching "append-only"

  # === @negative =========================================================

  @negative
  Scenario: Drafting an evaluation against a Draft spec returns 422
    Given a Draft TechnicalSpec exists for the ACP
    When the syndic POSTs an evaluation against the Draft spec
    Then the API returns status 422 with kind "technical_spec_required"

  @negative
  Scenario: Drafting an evaluation against a PendingSignatures spec returns 422
    Given a PendingSignatures TechnicalSpec exists for the ACP
    When the syndic POSTs an evaluation against the PendingSignatures spec
    Then the API returns status 422 with kind "technical_spec_required"

  @negative
  Scenario: Drafting an evaluation against a Superseded spec returns 422
    Given a Superseded TechnicalSpec exists for the ACP
    When the syndic POSTs an evaluation against the Superseded spec
    Then the API returns status 422 with kind "technical_spec_required"

  @negative
  Scenario: Score outside [1, 5] is rejected
    Given an Approved TechnicalSpec exists for the ACP
    When the syndic POSTs an evaluation with score quality=6
    Then the API returns status 400 with kind "validation"

  @negative
  Scenario: Comment shorter than 10 chars is rejected
    Given an Approved TechnicalSpec exists for the ACP
    When the syndic POSTs an evaluation with comment "short"
    Then the API returns status 400 with kind "validation"

  @negative
  Scenario: Unknown technical_spec_id returns 404
    When the syndic POSTs an evaluation against a random unknown technical_spec_id
    Then the API returns status 404 with kind "not_found"

  @negative
  Scenario: Duplicate linked_ticket_ids are rejected
    Given an Approved TechnicalSpec exists for the ACP
    When the syndic POSTs an evaluation with linked_ticket_ids containing duplicates
    Then the API returns status 400 with kind "validation"
