# Feature: Board Decision Poll System
# Belgian Context: Quick consultations between general assemblies
# Issue #51 - Phase 2

Feature: Board Decision Poll System
  As a property syndic
  I want to create polls to consult owners on quick decisions
  So that I can get owner input between general assemblies

  Background:
    Given the system is initialized
    And an organization "Test Copro ASBL" exists with id "org-poll"
    And a building "Residence Democratique" exists in organization "org-poll"
    And a syndic user "Jean Syndic" exists for building "Residence Democratique"
    And an owner "Marie Proprietaire" exists in building "Residence Democratique"
    And an owner "Pierre Coproprietaire" exists in building "Residence Democratique"
    And the user is authenticated as syndic "Jean Syndic"

  # Scenario 1: Create Yes/No Poll
  Scenario: Syndic creates a yes/no poll for simple decision
    When I create a yes/no poll:
      | question     | Should we repaint the lobby in blue? |
      | description  | Current lobby is white. Proposed color: ocean blue |
      | starts_at    | 2025-11-20T09:00:00Z |
      | ends_at      | 2025-11-25T18:00:00Z |
      | is_anonymous | false |
    Then the poll should be created successfully
    And the poll status should be "Draft"
    And the poll type should be "YesNo"
    And the poll should have 2 options: "Yes" and "No"
    And total_eligible_voters should be calculated from building owners

  # Scenario 2: Create Multiple Choice Poll
  Scenario: Syndic creates multiple choice poll for contractor selection
    When I create a multiple choice poll:
      | question     | Which contractor should we hire for roof repairs? |
      | description  | We received 3 quotes. Choose the best option. |
      | starts_at    | 2025-11-20T09:00:00Z |
      | ends_at      | 2025-11-27T18:00:00Z |
      | is_anonymous | false |
      | allow_multiple_votes | false |
    And I add the following options:
      | option_text                | display_order |
      | Contractor A - €15,000     | 1             |
      | Contractor B - €18,500     | 2             |
      | Contractor C - €14,200     | 3             |
    Then the poll should be created with 3 options
    And the poll type should be "MultipleChoice"
    And allow_multiple_votes should be false

  # Scenario 3: Create Rating Poll
  Scenario: Syndic creates rating poll for satisfaction survey
    When I create a rating poll:
      | question     | How satisfied are you with our new cleaning service? |
      | description  | Rate from 1 (very unsatisfied) to 5 (very satisfied) |
      | starts_at    | 2025-11-20T09:00:00Z |
      | ends_at      | 2025-11-30T18:00:00Z |
      | is_anonymous | true |
      | min_rating   | 1 |
      | max_rating   | 5 |
    Then the poll should be created successfully
    And the poll type should be "Rating"
    And is_anonymous should be true
    And the poll should have 5 rating options (1-5 stars)

  # Scenario 4: Create Open-Ended Poll
  Scenario: Syndic creates open-ended poll for feedback collection
    When I create an open-ended poll:
      | question     | What improvements would you suggest for our building? |
      | description  | Share your ideas for making our building better |
      | starts_at    | 2025-11-20T09:00:00Z |
      | ends_at      | 2025-12-01T18:00:00Z |
      | is_anonymous | false |
    Then the poll should be created successfully
    And the poll type should be "OpenEnded"
    And the poll should allow free text responses

  # Scenario 5: Publish Poll
  Scenario: Syndic publishes draft poll to make it active
    Given a draft poll "Should we install bike racks?" exists
    When I publish the poll
    Then the poll status should change to "Active"
    And owners should receive email notifications about the poll
    And the poll should appear in active polls list

  # Scenario 6: Owner Votes on Yes/No Poll
  Scenario: Owner casts vote on active yes/no poll
    Given an active poll "Should we repaint the lobby in blue?"
    And I am authenticated as owner "Marie Proprietaire"
    When I vote "Yes" on the poll
    Then my vote should be recorded
    And the vote_count for "Yes" option should increase by 1
    And total_votes_cast should increase by 1
    And I should not be able to vote again on this poll

  # Scenario 7: Owner Votes on Multiple Choice Poll
  Scenario: Owner selects contractor in multiple choice poll
    Given an active poll "Which contractor should we hire for roof repairs?"
    And I am authenticated as owner "Pierre Coproprietaire"
    When I vote for option "Contractor C - €14,200"
    Then my vote should be recorded
    And the vote_count for "Contractor C" should increase by 1
    And I should not be able to vote again on this poll

  # Scenario 8: Anonymous Voting
  Scenario: Owner votes anonymously on satisfaction poll
    Given an active anonymous poll "How satisfied are you with cleaning service?"
    And I am authenticated as owner "Marie Proprietaire"
    When I cast an anonymous vote with rating 4
    Then the vote should be recorded without my identity
    And my name should NOT appear in vote records
    And only my IP address should be logged for audit

  # Scenario 9: Duplicate Vote Prevention
  Scenario: Owner cannot vote twice on same poll
    Given an active poll "Should we repaint the lobby?"
    And I am authenticated as owner "Marie Proprietaire"
    And I have already voted "Yes" on this poll
    When I try to vote "No" on the same poll
    Then the system should reject my vote
    And I should see error "You have already voted on this poll"
    And my original "Yes" vote should remain unchanged

  # Scenario 10: Close Poll and Calculate Results
  Scenario: Syndic closes poll and views results
    Given an active poll "Should we repaint the lobby in blue?"
    And 8 owners have voted: 5 Yes, 3 No
    And I am authenticated as syndic "Jean Syndic"
    When I close the poll
    Then the poll status should change to "Closed"
    And the winning option should be "Yes" with 5 votes (62.5%)
    And the participation rate should be 80% (8 out of 10 owners)
    And I should see detailed vote breakdown

  # Scenario 11: Poll Automatic Expiration
  Scenario: Poll automatically expires after end date
    Given an active poll "Color choice poll" with end date "2025-11-25T18:00:00Z"
    When the current time reaches "2025-11-25T18:01:00Z"
    And the system runs auto-close job
    Then the poll status should automatically change to "Closed"
    And no more votes should be accepted

  # Scenario 12: List Active Polls
  Scenario: Owner views all active polls for their building
    Given the following polls exist for building "Residence Democratique":
      | Question              | Status | Ends At            |
      | Repaint lobby?        | Active | 2025-11-25 18:00   |
      | Hire contractor?      | Active | 2025-11-27 18:00   |
      | Satisfaction survey   | Active | 2025-11-30 18:00   |
      | Last year poll        | Closed | 2024-12-01 18:00   |
    And I am authenticated as owner "Marie Proprietaire"
    When I request active polls list
    Then I should see 3 active polls
    And I should NOT see closed polls
    And polls should be ordered by end date (soonest first)

  # Scenario 13: Get Poll Results
  Scenario: Syndic retrieves poll results with statistics
    Given a closed poll "Contractor selection" with:
      | Option         | Votes | Percentage |
      | Contractor A   | 2     | 20%        |
      | Contractor B   | 3     | 30%        |
      | Contractor C   | 5     | 50%        |
    And 10 owners were eligible to vote
    When I request poll results
    Then I should see winning option "Contractor C" with 5 votes
    And I should see participation rate 100% (10 votes / 10 eligible)
    And I should see vote percentages for all options

  # Scenario 14: Poll Statistics for Building
  Scenario: Syndic views poll statistics for building
    Given the building has conducted the following polls:
      | Question        | Status | Total Votes | Eligible Voters |
      | Poll A          | Closed | 8           | 10             |
      | Poll B          | Closed | 7           | 10             |
      | Poll C          | Active | 5           | 10             |
    When I request building poll statistics
    Then I should see total polls: 3
    And I should see active polls: 1
    And I should see closed polls: 2
    And I should see average participation rate: 66.67% ((8+7+5)/(3*10))

  # Scenario 15: Cancel Poll (Not Started)
  Scenario: Syndic cancels poll before it starts
    Given a draft poll "Future decision poll"
    And the poll has not been published
    When I cancel the poll
    Then the poll status should change to "Cancelled"
    And the poll should not appear in active polls
    And no notifications should be sent

  # Scenario 16: Multi-Select Multiple Choice Poll
  Scenario: Owner votes for multiple options when allowed
    Given an active poll "Which amenities should we add?" with:
      | question     | Which amenities should we prioritize? (select up to 3) |
      | allow_multiple_votes | true |
    And the poll has options:
      | Bike racks        |
      | Package room      |
      | Gym equipment     |
      | Rooftop garden    |
      | EV charging       |
    And I am authenticated as owner "Marie Proprietaire"
    When I vote for options:
      | Bike racks    |
      | Package room  |
      | EV charging   |
    Then all 3 votes should be recorded
    And each selected option vote_count should increase by 1
    And I should not be able to vote again

  # Scenario 17: Belgian Legal Compliance - Quick Consultation
  Scenario: Syndic uses poll for urgent decision between GAs
    Given the next general assembly is scheduled for 2026-03-15
    And an urgent decision is needed about heating repair contractor
    And I am authenticated as syndic "Jean Syndic"
    When I create a poll "Emergency heating repair contractor selection"
    And I set the poll description to "Article 577-8/4 §4 consultation between assemblies"
    And I publish the poll immediately
    Then owners can vote within 5 days
    And the poll results can be used for board decision
    And the poll should be documented in meeting minutes of next GA

  # Scenario 18: Poll with Attachment
  Scenario: Syndic creates poll with contractor quote attachments
    Given I am creating a poll "Select best quote for elevator maintenance"
    When I add the following options with attachments:
      | option_text       | attachment_url                          |
      | Contractor A      | https://storage/quotes/contractor_a.pdf |
      | Contractor B      | https://storage/quotes/contractor_b.pdf |
      | Contractor C      | https://storage/quotes/contractor_c.pdf |
    Then owners should see PDF links in poll options
    And owners can download quotes before voting

  # Scenario 19: Open-Ended Poll Response Collection
  Scenario: Owner submits free text response in open-ended poll
    Given an active open-ended poll "What improvements would you suggest?"
    And I am authenticated as owner "Pierre Coproprietaire"
    When I submit the response:
      """
      I suggest installing solar panels on the roof to reduce energy costs.
      Also, we could create a community garden on the unused ground floor space.
      """
    Then my response should be recorded
    And the syndic should be able to read all responses
    And responses should be exportable to PDF for meeting documentation

  # Scenario 20: Poll Access Control
  Scenario: Only building owners can vote on polls
    Given an active poll for building "Residence Democratique"
    And I am authenticated as owner "External User" from different building
    When I try to vote on the poll
    Then the system should reject my vote
    And I should see error "You are not authorized to vote on this poll"

  # Belgian Legal Context Notes:
  # - Article 577-8/4 §4 Code Civil Belge allows syndic consultations between assemblies
  # - Poll results are advisory and should be ratified in next general assembly
  # - Polls complement (not replace) formal resolutions in general assemblies
  # - Useful for urgent decisions: contractor selection, color choices, scheduling
  # - Participation rate and vote distribution should be documented in AG minutes
