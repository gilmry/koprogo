# Feature: Second Convocation & Quorum (Issues #311, #313)
# Art. 3.87 ss3 CC: 15 days minimum notice for ALL assembly types
# Art. 3.87 ss5 CC: 2nd convocation deliberates validly regardless of attendance (no quorum)
# Minutes distribution tracking: document_id + sent_at timestamp

Feature: Second Convocation and Quorum Management
  As a syndic managing general assemblies
  I want to handle second convocations and meeting minutes
  So that Belgian copropriete law is respected (Art. 3.87 CC)

  Background:
    Given the system is initialized
    And an organization "Quorum Copro ASBL" exists with id "org-quorum"
    And a building "Residence Quorum" exists in organization "org-quorum"
    And the user is authenticated as syndic "Jean Syndic"

  # === SECOND CONVOCATION CREATION ===

  Scenario: Create second convocation without quorum requirement
    Given a first meeting "AGO 2025" was held on "2026-04-15" for building "Residence Quorum"
    And the first meeting quorum was not reached (40% present, 50% required)
    When I create a second convocation:
      | first_meeting_date | 2026-04-15  |
      | new_meeting_date   | 2026-05-05  |
      | language           | FR          |
    Then the second convocation should be created successfully
    And the convocation type should be "SecondConvocation"
    And the convocation no_quorum_required flag should be true
    And the convocation should reference the first meeting
    And the convocation status should be "Draft"

  Scenario: Create second convocation exactly 15 days after first meeting
    Given a first meeting "AGO 2025" was held for building "Residence Quorum"
    When I create a second convocation 15 days after the first meeting
    Then the second convocation should be created successfully
    And the convocation no_quorum_required flag should be true

  Scenario: Create second convocation in Dutch language
    Given a first meeting was held for building "Residence Quorum"
    When I create a second convocation:
      | new_meeting_date   | 50 days from now |
      | language           | NL               |
    Then the second convocation should be created successfully
    And the convocation language should be "NL"

  # === SECOND CONVOCATION VALIDATION ===

  Scenario: Reject second convocation less than 15 days after first meeting
    Given a first meeting "AGO 2025" was held on "2026-04-15" for building "Residence Quorum"
    When I create a second convocation:
      | first_meeting_date | 2026-04-15  |
      | new_meeting_date   | 2026-04-24  |
      | language           | FR          |
    Then the second convocation creation should fail
    And the error should contain "15 days after"
    And the error should contain "Art. 3.87"

  Scenario: Reject second convocation only 10 days after first meeting
    Given a first meeting was held on "2026-04-15"
    When I create a second convocation:
      | first_meeting_date | 2026-04-15  |
      | new_meeting_date   | 2026-04-25  |
      | language           | FR          |
    Then the second convocation creation should fail
    And the error should contain "15 days after"

  # === QUORUM VALIDATION FOR FIRST CONVOCATION ===

  Scenario: Quorum reached with 60% of quotas present
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    When I validate quorum with 600 present quotas out of 1000 total
    Then the quorum should be reached
    And the quorum percentage should be 60.0
    And the quorum_validated flag should be true

  Scenario: Quorum not reached at exactly 50% (strict majority required)
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    When I validate quorum with 500 present quotas out of 1000 total
    Then the quorum should not be reached
    And the quorum_validated flag should be false

  Scenario: Quorum not reached with 40% of quotas present
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    When I validate quorum with 400 present quotas out of 1000 total
    Then the quorum should not be reached
    And a second convocation is required

  Scenario: Voting allowed on second convocation without quorum check
    Given a meeting "2e Convocation AGO" exists for building "Residence Quorum"
    And the meeting is marked as second convocation
    When I check quorum for voting
    Then voting should be allowed
    And no quorum validation should be required

  Scenario: Voting blocked when quorum not validated on first convocation
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    And the meeting is a first convocation
    And the quorum has not been validated
    When I check quorum for voting
    Then voting should be blocked
    And the error should contain "not been validated yet"

  # === MINUTES DOCUMENT TRACKING ===

  Scenario: Attach minutes document to completed meeting
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    And the meeting is completed with 45 attendees
    And a document "PV AGO 2025" exists with id "doc-pv-2025"
    When I attach minutes document "doc-pv-2025" to the meeting
    Then the meeting should have minutes_document_id set
    And the meeting should have minutes_sent_at timestamp

  Scenario: Mark minutes as sent updates timestamp
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    And the meeting is completed with 45 attendees
    When I set the meeting minutes as sent with document "doc-pv"
    Then the meeting minutes_sent_at should not be null
    And the meeting minutes_document_id should be "doc-pv"

  Scenario: Reject attaching minutes to non-completed meeting
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    And the meeting status is "Scheduled"
    When I attach minutes document "doc-pv-2025" to the meeting
    Then attaching minutes should fail
    And the error should contain "Minutes can only be sent after meeting is completed"

  Scenario: Reject attaching minutes to cancelled meeting
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    And the meeting is cancelled
    When I attach minutes document "doc-pv-2025" to the meeting
    Then attaching minutes should fail
    And the error should contain "Minutes can only be sent after meeting is completed"

  # === MINUTES OVERDUE TRACKING ===

  Scenario: Minutes are not overdue within 30 days of completion
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    And the meeting was completed recently
    And no minutes have been sent
    Then the meeting minutes should not be overdue

  Scenario: Minutes are overdue after 30 days without sending
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    And the meeting was completed more than 30 days ago
    And no minutes have been sent
    Then the meeting minutes should be overdue

  Scenario: Minutes are not overdue when already sent
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    And the meeting was completed more than 30 days ago
    And the minutes have been sent
    Then the meeting minutes should not be overdue

  Scenario: Scheduled meeting is never overdue for minutes
    Given a meeting "AGO 2025" exists for building "Residence Quorum"
    And the meeting status is "Scheduled"
    Then the meeting minutes should not be overdue
