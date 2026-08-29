# Feature: AG Visioconférence Sessions (BC15 - Issue #274)
# Art. 3.87 §1 Code Civil Belge — Remote participation with combined quorum
# Workflow: Scheduled -> Live -> Ended | Cancelled

Feature: AG Visioconférence Sessions
  As a syndic
  I want to create and manage video conference sessions for general assemblies
  So that remote co-owners can participate and contribute to quorum

  Background:
    Given the system is initialized
    And an organization "AG Visio Copro ASBL" exists with id "org-ag-visio"
    And a building "Residence Visio" exists in organization "org-ag-visio"
    And a meeting "AG Ordinaire Visio" scheduled in 10 days exists

  # === CREATION ===

  Scenario: Syndic creates a Zoom video session for an AG
    When I create an AG session:
      | platform         | zoom                          |
      | video_url        | https://zoom.us/j/123456789   |
      | host_url         | https://zoom.us/s/123456789   |
    Then the AG session should be created successfully
    And the session status should be "Scheduled"
    And the session platform should be "zoom"

  Scenario: Syndic creates a Jitsi session without password
    When I create an AG session:
      | platform  | jitsi                                    |
      | video_url | https://meet.jit.si/koprogo-ag-2026-03   |
    Then the AG session should be created successfully
    And the session status should be "Scheduled"

  Scenario: Create session fails with empty video URL
    When I try to create an AG session with empty video_url
    Then the session creation should fail

  Scenario: Cannot create two sessions for the same meeting
    Given an AG session already exists for this meeting
    When I try to create another AG session for the same meeting
    Then the session creation should fail
    And the error should mention "session" or "existe"

  # === LIFECYCLE ===

  Scenario: Syndic starts a scheduled session (Live)
    Given a scheduled AG session exists
    When I start the AG session
    Then the session status should be "Live"
    And the actual_start time should be set

  Scenario: Syndic ends a live session
    Given a live AG session exists
    When I end the AG session
    Then the session status should be "Ended"
    And the actual_end time should be set

  Scenario: Syndic cancels a scheduled session
    Given a scheduled AG session exists
    When I cancel the AG session
    Then the session status should be "Cancelled"

  Scenario: Cannot start a session that is already ended
    Given an ended AG session exists
    When I try to start the AG session again
    Then the operation should fail

  # === REMOTE PARTICIPANT TRACKING ===

  Scenario: Record a remote participant joining
    Given a live AG session exists
    When I record a remote participant with voting power 150 out of 1000 total quotas
    Then the remote_attendees_count should be 1
    And the remote_voting_power should be 150.0

  Scenario: Multiple remote participants accumulate voting power
    Given a live AG session exists
    When I record a remote participant with voting power 100 out of 1000 total quotas
    And I record a remote participant with voting power 200 out of 1000 total quotas
    Then the remote_attendees_count should be 2
    And the remote_voting_power should be 300.0

  # === COMBINED QUORUM (Art. 3.87 §5) ===

  Scenario: Calculate combined quorum with remote participants
    # 550/1000 quotités (≥ 50%) ET 6/10 têtes présentes en comptant le
    # participant distant (> 50%) → quorum double atteint.
    Given a live AG session exists
    And I record a remote participant with voting power 250 out of 1000 total quotas
    When I calculate the combined quorum with 300 physical quotas out of 1000, 5 owners present out of 10
    Then the quorum calculation should succeed
    And the combined_percentage should be approximately 55.0
    And quorum_reached should be true

  @security
  Scenario: Quotas alone never carry the quorum — the head count still applies
    # Issue #661 — avant le correctif, ce cas passait « quorum atteint » : le
    # chemin distanciel ne testait que les quotités. 550/1000 quotités suffisent
    # au volet quotités, mais 2 têtes sur 10 (1 physique + 1 distant) ne font pas
    # « plus de la moitié des copropriétaires » (Art. 3.87 §5 CC).
    Given a live AG session exists
    And I record a remote participant with voting power 250 out of 1000 total quotas
    When I calculate the combined quorum with 300 physical quotas out of 1000, 1 owners present out of 10
    Then the quorum calculation should succeed
    And the combined_percentage should be approximately 55.0
    And quorum_reached should be false

  @edge
  Scenario: Three quarters of quotas carry the quorum without the head count
    # Art. 3.87 §5 CC, alternative : > 3/4 des quotités emportent le quorum
    # quelles que soient les têtes. 800/1000 = 80% > 75%.
    Given a live AG session exists
    And I record a remote participant with voting power 300 out of 1000 total quotas
    When I calculate the combined quorum with 500 physical quotas out of 1000, 1 owners present out of 10
    Then the quorum calculation should succeed
    And quorum_reached should be true

  @edge
  Scenario: Quotas at exactly 50 percent DO satisfy their half of the quorum
    # Issue #661 — Art. 3.87 §5 CC : « pour autant qu'ils possèdent AU MOINS la
    # moitié des quotes-parts » → la borne est INCLUSIVE côté quotités.
    # 500/1000 pile suffit donc, dès lors que les têtes suivent (6/10 > 50%).
    # En f64, cette borne exacte dépendait de l'arrondi binaire ; en Decimal
    # elle est décidable.
    Given a live AG session exists
    And I record a remote participant with voting power 200 out of 1000 total quotas
    When I calculate the combined quorum with 300 physical quotas out of 1000, 5 owners present out of 10
    Then the quorum calculation should succeed
    And the combined_percentage should be approximately 50.0
    And quorum_reached should be true

  @edge
  Scenario: Heads at exactly 50 percent do NOT satisfy their half of the quorum
    # Art. 3.87 §5 CC : « PLUS DE la moitié des copropriétaires » → la borne est
    # STRICTE côté têtes, à l'inverse des quotités. 5 présents sur 10 (4
    # physiques + 1 distant) = 50% pile → quorum non atteint, malgré des
    # quotités largement suffisantes.
    Given a live AG session exists
    And I record a remote participant with voting power 200 out of 1000 total quotas
    When I calculate the combined quorum with 400 physical quotas out of 1000, 4 owners present out of 10
    Then the quorum calculation should succeed
    And quorum_reached should be false

  Scenario: Quorum not reached with insufficient combined attendance
    Given a live AG session exists
    And I record a remote participant with voting power 100 out of 1000 total quotas
    When I calculate the combined quorum with 200 physical quotas out of 1000, 5 owners present out of 10
    Then the combined_percentage should be approximately 30.0
    And quorum_reached should be false

  # === RETRIEVAL ===

  Scenario: Get session by ID
    Given a scheduled AG session exists
    When I retrieve the AG session by ID
    Then the session should be returned
    And the session meeting_id should match

  Scenario: Get session for a specific meeting
    Given a scheduled AG session exists
    When I retrieve the AG session for the meeting
    Then the session should be returned

  Scenario: List all sessions for an organization
    Given 2 AG sessions exist in the organization
    When I list all AG sessions
    Then I should get at least 2 sessions

  # === DELETION ===

  Scenario: Delete a scheduled session
    Given a scheduled AG session exists
    When I delete the AG session
    Then the session should be deleted successfully
