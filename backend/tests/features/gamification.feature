# Feature: Gamification & Achievements (Issue #49 Phase 6)
# Categories: Community, Sel, Booking, Sharing, Skills, Notice, Governance, Milestone
# Tiers: Bronze, Silver, Gold, Platinum, Diamond

Feature: Gamification & Achievements
  As a community member
  I want to earn achievements and participate in challenges
  So that I am recognized for my community contributions

  Background:
    Given the system is initialized
    And an organization "Gamif Copro ASBL" exists with id "org-gamif"
    And a building "Residence Ludique" exists in organization "org-gamif"
    And a user "player@test.be" exists in the organization

  # === ACHIEVEMENTS ===

  Scenario: Create an achievement definition
    When I create an achievement:
      | name        | First Booking                  |
      | description | Book a resource for the first time |
      | category    | Booking                        |
      | tier        | Bronze                         |
      | points      | 10                             |
      | icon        | calendar-check                 |
    Then the achievement should be created

  Scenario: Award achievement to user
    Given an achievement "First Booking" exists
    When I award "First Booking" to "player@test.be"
    Then the user should have the achievement
    And times_earned should be 1

  Scenario: Repeatable achievement increments counter
    Given a repeatable achievement "Weekly Helper" exists
    And "player@test.be" has earned it once
    When I award "Weekly Helper" to "player@test.be" again
    Then times_earned should be 2

  Scenario: Secret achievement hidden until earned
    Given a secret achievement "Easter Egg" exists
    When I list visible achievements for "player@test.be"
    Then "Easter Egg" should not be visible
    When "player@test.be" earns "Easter Egg"
    And I list visible achievements for "player@test.be"
    Then "Easter Egg" should be visible

  Scenario: List achievements by category
    Given achievements in Booking and Community categories exist
    When I list achievements by category "Booking"
    Then all returned achievements should have category "Booking"

  Scenario: List user's earned achievements
    Given "player@test.be" has earned 3 achievements
    When I list earned achievements for "player@test.be"
    Then I should get 3 achievements

  # === CHALLENGES ===

  Scenario: Create a challenge
    When I create a challenge:
      | title         | March Booking Challenge         |
      | description   | Book 5 resources this month     |
      | challenge_type | Individual                     |
      | target_metric | bookings_created                |
      | target_value  | 5                               |
      | reward_points | 100                             |
      | start_date    | 2026-03-01T00:00:00Z            |
      | end_date      | 2026-03-31T23:59:59Z            |
    Then the challenge should be created with status "Draft"

  Scenario: Activate a challenge
    Given a draft challenge "March Challenge" exists
    When I activate the challenge
    Then the challenge status should be "Active"

  Scenario: Increment challenge progress
    Given an active challenge with target 5 exists
    And "player@test.be" has progress 3
    When I increment progress for "player@test.be" by 1
    Then the progress should be 4
    And the challenge should not be completed

  Scenario: Auto-complete challenge when target reached
    Given an active challenge with target 5 exists
    And "player@test.be" has progress 4
    When I increment progress for "player@test.be" by 1
    Then the progress should be 5
    And the challenge should be marked as completed

  Scenario: Complete and cancel challenges
    Given an active challenge exists
    When I complete the challenge
    Then the challenge status should be "Completed"

  # === STATISTICS & LEADERBOARD ===

  Scenario: Get user gamification stats
    Given "player@test.be" has earned achievements worth 50 points
    And completed challenges worth 100 points
    When I get gamification stats for "player@test.be"
    Then the total points should be 150

  Scenario: Organization leaderboard
    Given multiple users with different point totals exist
    When I get the organization leaderboard
    Then users should be ordered by total points descending
