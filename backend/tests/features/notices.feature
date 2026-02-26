# Feature: Community Notice Board (Issue #49 Phase 2)
# Types: Announcement, Event, Alert, Discussion
# Statuses: Draft -> Published -> Archived

Feature: Community Notice Board
  As a co-owner
  I want to post and browse notices on the community board
  So that I can share information with my building neighbors

  Background:
    Given the system is initialized
    And an organization "Notice Copro ASBL" exists with id "org-notice"
    And a building "Residence Communaute" exists in organization "org-notice"
    And an owner "Marie Auteure" exists in building "Residence Communaute"

  # === CREATION ===

  Scenario: Create an announcement notice
    When I create a notice:
      | title       | Building maintenance schedule    |
      | content     | Elevator maintenance on March 15th from 9am to 12pm |
      | notice_type | Announcement                     |
      | category    | General                          |
    Then the notice should be created successfully
    And the notice type should be "Announcement"

  Scenario: Create an event notice with date and location
    When I create a notice:
      | title          | Spring community BBQ              |
      | content        | Annual BBQ in the garden          |
      | notice_type    | Event                             |
      | category       | Social                            |
      | event_date     | 2026-05-15T14:00:00Z              |
      | event_location | Common garden                     |
    Then the notice should be created successfully
    And the notice type should be "Event"
    And the event date should be set

  Scenario: Notice creation fails with short title
    When I create a notice:
      | title       | Hi                                |
      | content     | Some content here                 |
      | notice_type | Announcement                      |
      | category    | General                           |
    Then the notice creation should fail
    And the error should contain "title" or "5 characters"

  # === PUBLISHING ===

  Scenario: Publish a draft notice
    Given a draft notice "Important update" exists
    When I publish the notice
    Then the notice status should be "Published"

  Scenario: Archive a published notice
    Given a published notice "Old news" exists
    When I archive the notice
    Then the notice status should be "Archived"

  # === PINNING ===

  Scenario: Pin an important notice
    Given a published notice "Fire safety rules" exists
    When I pin the notice
    Then the notice should be pinned

  Scenario: Unpin a notice
    Given a pinned notice "Old pinned" exists
    When I unpin the notice
    Then the notice should not be pinned

  # === EXPIRATION ===

  Scenario: Set expiration date on notice
    When I create a notice with expiration:
      | title       | Temporary parking notice          |
      | content     | Parking lot closed this weekend   |
      | expires_at  | 2026-04-01T00:00:00Z              |
    Then the notice should have an expiration date

  # === LISTING ===

  Scenario: List published notices for building
    Given 3 published notices exist for the building
    When I list published notices
    Then I should get 3 notices

  Scenario: List pinned notices
    Given 1 pinned and 2 unpinned notices exist
    When I list pinned notices
    Then I should get 1 notice

  Scenario: List notices by type
    Given notices of types Announcement and Event exist
    When I list notices with type "Event"
    Then all returned notices should have type "Event"

  Scenario: List notices by category
    Given notices in categories General and Social exist
    When I list notices with category "Social"
    Then all returned notices should have category "Social"

  Scenario: List author's notices
    Given "Marie Auteure" has created 2 notices
    When I list notices by author "Marie Auteure"
    Then I should get 2 notices

  # === UPDATE & DELETE ===

  Scenario: Update a notice
    Given a draft notice "Typo in title" exists
    When I update the notice title to "Corrected title"
    Then the notice title should be updated

  Scenario: Delete a notice
    Given a notice exists
    When I delete the notice
    Then the notice should be deleted
