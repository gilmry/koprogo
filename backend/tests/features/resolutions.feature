# Feature: Meeting Resolutions & Voting System (Issue #46)
# Belgian copropriete law compliance
# Majority types: Simple (50%+1 expressed), Absolute (50%+1 all), Qualified (custom threshold)

Feature: Meeting Resolutions and Voting System
  As a syndic
  I want to manage resolutions and voting at general assemblies
  So that decisions comply with Belgian copropriete law

  Background:
    Given the system is initialized
    And an organization "Vote Copro ASBL" exists with id "org-vote"
    And a building "Residence Democratique" exists in organization "org-vote"
    And a meeting "AG Ordinaire 2026" exists for the building
    And an owner "Alice" with 300 voting power (tantiemes) exists
    And an owner "Bob" with 200 voting power (tantiemes) exists
    And an owner "Charlie" with 500 voting power (tantiemes) exists
    # Art. 3.87 §5 CC: a resolution (agenda item put to the vote) can only be
    # created on a validly-constituted AG. 1000/1000 tantiemes present = 100%.
    And the AG is validly constituted (quorum reached)

  # === RESOLUTION CREATION ===

  # Belgian named majority model (Art. 3.88 CC). Legacy "Simple"/"Qualified"
  # labels were superseded by the named MajorityType enum (#310/#323): a simple
  # copro decision = Absolute majority (Art. 3.88 §1), a 2/3 decision = TwoThirds.
  Scenario: Create a resolution with absolute majority
    When I create a resolution for the meeting:
      | title              | Repaint lobby walls              |
      | description        | Repaint lobby in blue as discussed |
      | majority_required  | Absolute                         |
    Then the resolution should be created
    And the resolution status should be "Pending"
    And the majority type should be "Absolute"

  Scenario: Create a resolution with two-thirds qualified majority
    When I create a resolution for the meeting:
      | title              | Modify building rules            |
      | description        | Amend rules for pet ownership    |
      | majority_required  | TwoThirds                        |
    Then the resolution should be created
    And the majority type should be "TwoThirds"

  # === VOTING ===

  Scenario: Cast a vote on a resolution
    Given a pending resolution "Repaint lobby" exists
    When "Alice" votes "Pour" on the resolution
    Then the vote should be recorded
    And the vote choice should be "Pour"
    And the voting power should be 300

  Scenario: Vote against a resolution
    Given a pending resolution "New rules" exists
    When "Bob" votes "Contre" on the resolution
    Then the vote should be recorded
    And the vote choice should be "Contre"

  Scenario: Abstain from a resolution
    Given a pending resolution "Budget 2026" exists
    When "Charlie" votes "Abstention" on the resolution
    Then the vote should be recorded
    And the vote choice should be "Abstention"

  Scenario: Change vote before closing
    Given a pending resolution "Paint color" exists
    And "Alice" has voted "Pour" on the resolution
    When "Alice" changes her vote to "Contre"
    Then the updated vote should be "Contre"

  Scenario: Vote by proxy delegation
    Given a pending resolution "Elevator repair" exists
    When "Alice" votes "Pour" as proxy for "Bob"
    Then the vote should be recorded
    And the proxy owner should be "Alice"
    And the voting power should be "Bob"'s tantiemes

  # === VOTING CLOSURE ===

  Scenario: Close voting - Simple majority adopted
    Given a pending resolution "Simple vote" with simple majority
    And "Alice" (300) voted "Pour"
    And "Bob" (200) voted "Pour"
    And "Charlie" (500) voted "Contre"
    When I close voting on the resolution
    Then the resolution should be "Rejected"
    # 500 Pour vs 500 Contre - tie is rejected (strict > required)

  Scenario: Close voting - Absolute majority adopted
    Given a pending resolution "Absolute vote" with absolute majority
    And "Alice" (300) voted "Pour"
    And "Bob" (200) voted "Pour"
    And "Charlie" (500) voted "Pour"
    When I close voting on the resolution
    Then the resolution should be "Adopted"
    # 1000/1000 = 100% > 50%+1

  Scenario: Close voting - Qualified majority rejected
    Given a pending resolution "Qualified vote" with qualified majority of 0.6667
    And "Alice" (300) voted "Pour"
    And "Bob" (200) voted "Pour"
    And "Charlie" (500) voted "Contre"
    When I close voting on the resolution
    Then the resolution should be "Rejected"
    # 500/1000 = 50% < 66.67%

  # === INVALID OPERATIONS ===

  Scenario: Cannot vote after voting is closed
    Given a closed resolution "Old vote" exists
    When "Alice" tries to vote "Pour" on the closed resolution
    Then the vote should be rejected

  Scenario: Cannot delete resolution with existing votes
    Given a pending resolution "Active vote" with votes
    When I try to delete the resolution
    Then the deletion should fail

  # === SECURITY — VOTE INTEGRITY ===
  # Art. 3.87 CC: one lot = one voice. A unit may not vote twice on the same
  # resolution (ballot-stuffing / double-vote prevention). The legitimate way
  # to revise a choice is "change vote" (covered above), not a second ballot.

  @security
  Scenario: A unit cannot vote twice on the same resolution
    Given a pending resolution "Roof works budget" exists
    And "Alice" has voted "Pour" on the resolution
    When "Alice" votes "Contre" on the resolution
    Then the vote should be rejected

  # === LISTING & SUMMARY ===

  Scenario: List resolutions for a meeting
    Given 3 resolutions exist for the meeting
    When I list resolutions for the meeting
    Then I should get 3 resolutions

  Scenario: Get vote summary for meeting
    Given resolutions with votes exist for the meeting
    When I get the vote summary for the meeting
    Then the summary should include vote counts per resolution
