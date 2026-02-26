# Feature: Automatic AG Convocations (Issue #88)
# Belgian legal deadlines: Ordinary (15 days), Extraordinary (8 days)
# Workflow: Draft -> Scheduled -> Sent -> Cancelled

Feature: Automatic AG Convocations
  As a syndic
  I want to send legal AG convocations automatically
  So that general assembly invitations comply with Belgian law

  Background:
    Given the system is initialized
    And an organization "Convoc Copro ASBL" exists with id "org-convoc"
    And a building "Residence Legale" exists in organization "org-convoc"
    And a meeting "AG Ordinaire Mars 2026" scheduled in 20 days exists
    And 3 owners exist in the building with email addresses

  # === CREATION ===

  Scenario: Create convocation for ordinary AG (15-day minimum)
    When I create a convocation:
      | meeting_type | Ordinary  |
      | language     | FR        |
    Then the convocation should be created with status "Draft"
    And the minimum send date should be at least 15 days before the meeting

  Scenario: Create convocation for extraordinary AG (8-day minimum)
    Given a meeting "AG Extraordinaire" scheduled in 12 days exists
    When I create a convocation:
      | meeting_type | Extraordinary |
      | language     | FR            |
    Then the convocation should be created
    And the minimum send date should be at least 8 days before the meeting

  Scenario: Reject convocation violating legal deadline
    Given a meeting "AG Last Minute" scheduled in 3 days exists
    When I try to create a convocation for an ordinary AG
    Then the creation should fail
    And the error should mention "legal deadline" or "minimum"

  # === SCHEDULING ===

  Scenario: Schedule convocation send date
    Given a draft convocation exists
    When I schedule the convocation for 18 days before the meeting
    Then the convocation status should be "Scheduled"
    And the scheduled date should respect the legal deadline

  Scenario: Cannot schedule past the legal deadline
    Given a draft convocation for an ordinary AG exists
    When I try to schedule the convocation for 10 days before the meeting
    Then the scheduling should fail

  # === SENDING ===

  Scenario: Send convocation to all owners
    Given a scheduled convocation exists
    When I send the convocation
    Then the convocation status should be "Sent"
    And recipients should be created for all building owners
    And total_recipients should match the number of owners

  # === TRACKING ===

  Scenario: Track email opened
    Given a sent convocation with recipients exists
    When recipient "owner1@test.be" opens the email
    Then the email_opened_at should be set for that recipient

  Scenario: Update attendance status
    Given a sent convocation with recipients exists
    When recipient "owner1@test.be" confirms attendance
    Then the attendance status should be "WillAttend"

  Scenario: Set proxy delegation (procuration)
    Given a sent convocation with recipients exists
    When recipient "owner2@test.be" delegates proxy to "owner1@test.be"
    Then the proxy should be recorded

  # === REMINDERS ===

  Scenario: Send J-3 reminders to unopened emails
    Given a sent convocation with 2 unopened recipients
    And the meeting is in 3 days
    When I send reminders
    Then reminders should be sent to the 2 unopened recipients

  # === TRACKING SUMMARY ===

  Scenario: Get tracking summary statistics
    Given a sent convocation with tracked recipients
    When I get the tracking summary
    Then the summary should include opening rate
    And the summary should include attendance counts

  # === CANCELLATION ===

  Scenario: Cancel a convocation
    Given a draft convocation exists
    When I cancel the convocation
    Then the convocation status should be "Cancelled"

  # === LISTING ===

  Scenario: List convocations for building
    Given 2 convocations exist for the building
    When I list convocations for the building
    Then I should get 2 convocations
