Feature: Meeting completion gate (Track H Story H3 — Art. 3.87 §3-5 CC)
  As a syndic
  I want the system to reject closing an AG unless all legal invariants are met
  So that I cannot accidentally close an AG without convocations, votes, attendance, quorum, and minutes draft

  Background:
    Given a coproperty management system
    And a scheduled meeting "AG H3 Demo"

  # ----------------------------------------------------------------------
  # @happy
  # ----------------------------------------------------------------------

  @happy
  Scenario: All invariants present — closing succeeds
    Given the completion checklist has convocations sent, no open resolutions, attendance recorded, quorum 600 of 1000, minutes draft present
    When the syndic asserts completion
    Then the assertion is Ok

  # ----------------------------------------------------------------------
  # @edge
  # ----------------------------------------------------------------------

  @edge
  Scenario: Quorum exactly at 50% is rejected
    Given the completion checklist has convocations sent, no open resolutions, attendance recorded, quorum 500 of 1000, minutes draft present
    When the syndic asserts completion
    Then the assertion fails with 1 missing invariant
    And the missing invariant contains "QuorumNotReached"

  @edge
  Scenario: Quorum just above 50% passes
    Given the completion checklist has convocations sent, no open resolutions, attendance recorded, quorum 501 of 1000, minutes draft present
    When the syndic asserts completion
    Then the assertion is Ok

  @edge
  Scenario: 1 open resolution blocks closing (Art. 3.87 §4)
    Given the completion checklist has convocations sent, 1 open resolution, attendance recorded, quorum 600 of 1000, minutes draft present
    When the syndic asserts completion
    Then the assertion fails with 1 missing invariant
    And the missing invariant contains "VotesNotClosed"

  @edge
  Scenario: total_quotas zero is rejected as QuorumNotReached, not panic
    Given the completion checklist has convocations sent, no open resolutions, attendance recorded, quorum 0 of 0, minutes draft present
    When the syndic asserts completion
    Then the assertion fails with 1 missing invariant
    And the missing invariant contains "QuorumNotReached"

  # ----------------------------------------------------------------------
  # @security
  # ----------------------------------------------------------------------

  @security
  Scenario: Bypass attempt — convocations not sent but everything else OK
    Given the completion checklist has convocations NOT sent, no open resolutions, attendance recorded, quorum 600 of 1000, minutes draft present
    When the syndic asserts completion
    Then the assertion fails with 1 missing invariant
    And the missing invariant contains "ConvocationsNotSent"

  @security
  Scenario: Bypass attempt — attendance not recorded
    Given the completion checklist has convocations sent, no open resolutions, attendance NOT recorded, quorum 600 of 1000, minutes draft present
    When the syndic asserts completion
    Then the assertion fails with at least 1 missing invariant
    And the missing invariant contains "AttendanceNotRecorded"

  # ----------------------------------------------------------------------
  # @negative
  # ----------------------------------------------------------------------

  @negative
  Scenario: All invariants missing — exhaustive list of 5
    Given the completion checklist has convocations NOT sent, 3 open resolutions, attendance NOT recorded, quorum 0 of 1000, minutes draft NOT present
    When the syndic asserts completion
    Then the assertion fails with 5 missing invariants

  @negative
  Scenario: Only minutes draft missing
    Given the completion checklist has convocations sent, no open resolutions, attendance recorded, quorum 600 of 1000, minutes draft NOT present
    When the syndic asserts completion
    Then the assertion fails with 1 missing invariant
    And the missing invariant contains "MinutesDraftMissing"
