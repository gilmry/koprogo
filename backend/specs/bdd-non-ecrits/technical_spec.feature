# Feature: TechnicalSpec — versionable cahier des charges + signatures multi-parties
# (Story 3.8 — FR33)
#
# Story 3.8 introduces a versionable TechnicalSpec entity (SemVer strict)
# signed off by multiple parties (Syndic, AMO, Lawyer, Architect, ACP
# representative). Signatures are append-only (DB trigger guards UPDATE /
# DELETE) and a major version bump invalidates the previous signatures.
#
# Wiring status: step definitions for this feature are pending
# (`bdd_technical_spec.rs` follow-up). Tagged `@wip` so CI does not fail —
# entity / use-cases / handlers / migration ship in Phase A; the Cucumber
# glue lands in Phase B alongside the FE work.

@wip
Feature: TechnicalSpec — versionable + signatures multi-parties
  As a syndic
  I want to draft a cahier des charges, submit it for signatures,
   and collect approvals from each required party
  So that the ACP has an audit-grade, versioned specification before works begin

  Background:
    Given the system is initialized
    And an organization "Specs ASBL" exists
    And a syndic "syndic@specs.test" exists
    And an ACP "ACP Specs" exists in the organization
    And a building "Residence Mimosas" exists in the ACP

  # === @happy ============================================================

  @happy
  Scenario: Syndic creates a Draft TechnicalSpec for the ACP
    When the syndic POSTs a TechnicalSpec with title "Toiture - etancheite",
      description "Renovation toiture batiment A : etancheite + isolation 18 cm.",
      version "1.0.0",
      deliverables ["Plan", "Cahier des charges"],
      required_signatures ["syndic", "architect"]
    Then the TechnicalSpec is created with status 201
    And the response body status is "draft"
    And the response body version is "1.0.0"

  @happy
  Scenario: Syndic submits a Draft and collects both required signatures
    Given a Draft TechnicalSpec exists requiring ["syndic", "acp_representative"]
    When the syndic submits the spec for signatures
    Then the spec status becomes "pending_signatures"
    When the syndic signs the spec under role "syndic"
    And an ACP representative signs the spec under role "acp_representative"
    Then the spec status becomes "approved"

  # === @edge =============================================================

  @edge
  Scenario: Minor bump v1.0.0 -> v1.1.0 keeps signatures conceptually valid
    Given an Approved TechnicalSpec v1.0.0 exists
    When the syndic bumps the version to "1.1.0"
    Then the new spec is created in status "draft"
    And the previous spec status becomes "superseded"
    And requires_resignature is false for this bump

  @edge
  Scenario: Major bump v1.5.7 -> v2.0.0 requires a fresh signature round
    Given an Approved TechnicalSpec v1.5.7 exists
    When the syndic bumps the version to "2.0.0"
    Then the new spec is created in status "draft"
    And requires_resignature is true for this bump

  # === @security =========================================================

  @security
  Scenario: An owner without role cannot create a TechnicalSpec (RBAC)
    Given an owner "owner@specs.test" is logged in (role "owner")
    When the owner POSTs a TechnicalSpec on the ACP
    Then the API returns status 403 with kind "forbidden"

  @security
  Scenario: A signatory whose role is not in required_signatures gets 403
    Given a TechnicalSpec is pending signatures requiring ["syndic", "architect"]
    When a user with role "acp_representative" attempts to sign
    Then the API returns status 403 with kind "signatory_not_authorized"

  @security
  Scenario: A signature cannot be edited (append-only, DB trigger)
    Given a TechnicalSpecSignature row exists
    When a direct SQL UPDATE on the row is attempted
    Then the database raises an error matching "append-only"

  @security
  Scenario: A mandataire role without an active Mandate cannot sign
    Given a TechnicalSpec is pending signatures requiring ["syndic", "lawyer"]
    When a user with role "lawyer" attempts to sign without a mandate_id
    Then the API returns status 403 with kind "signatory_not_authorized"

  # === @negative =========================================================

  @negative
  Scenario: Title shorter than 5 chars is rejected
    When the syndic POSTs a TechnicalSpec with title "ABC"
    Then the API returns status 400 with kind "validation"

  @negative
  Scenario: Description shorter than 50 chars is rejected
    When the syndic POSTs a TechnicalSpec with description "trop court"
    Then the API returns status 400 with kind "validation"

  @negative
  Scenario: Submitting a TechnicalSpec that is already approved returns 409
    Given an Approved TechnicalSpec exists
    When the syndic attempts to submit the spec for signatures again
    Then the API returns status 409 with kind "tech_spec_approved"

  @negative
  Scenario: Same (signatory, role) cannot sign twice on the same spec
    Given a TechnicalSpec is pending signatures requiring ["syndic", "architect"]
    And the syndic has signed the spec under role "syndic"
    When the same syndic attempts to sign again under role "syndic"
    Then the API returns status 409 with kind "signature_already_exists"

  @negative
  Scenario: SemVer with v-prefix is rejected
    When the syndic POSTs a TechnicalSpec with version "v1.2.3"
    Then the API returns status 400 with kind "validation"

  @negative
  Scenario: SemVer with pre-release suffix is rejected
    When the syndic POSTs a TechnicalSpec with version "1.2.3-rc1"
    Then the API returns status 400 with kind "validation"
