# Feature: Local Exchange System (SEL - Système d'Échange Local)
# Belgian Context: Time-based currency system (1 hour = 1 credit)
# Issue #49 - Phase 1

Feature: Local Exchange System (SEL)
  As a co-owner
  I want to exchange services with neighbors using time-based credits
  So that I can build community solidarity and reduce monetary expenses

  Background:
    Given the system is initialized
    And an organization "SEL Copro ASBL" exists with id "org-sel"
    And a building "Residence Solidaire" exists in organization "org-sel"
    And an owner "Alice Plombier" exists in building "Residence Solidaire"
    And an owner "Bob Bricoleur" exists in building "Residence Solidaire"
    And the user is authenticated as owner "Alice Plombier"

  Scenario: Owner creates service exchange offer
    When I create a service exchange offer:
      | title       | Plumbing repair         |
      | description | Fix leaking faucets and pipes |
      | credits     | 2                       |
      | conditions  | Bring your own parts    |
    Then the exchange should be created successfully
    And the status should be "Offered"
    And the exchange type should be "Service"
    And the offer should appear in building marketplace

  Scenario: Owner creates object loan offer
    When I create an object loan exchange:
      | title       | Electric drill          |
      | description | Makita 18V drill with batteries |
      | credits     | 1                       |
      | conditions  | Return within 2 days    |
    Then the exchange should be created successfully
    And the exchange type should be "ObjectLoan"
    And the status should be "Offered"

  Scenario: Owner creates shared purchase offer
    When I create a shared purchase exchange:
      | title       | Bulk organic vegetables |
      | description | Weekly order from local farm |
      | credits     | 0                       |
      | conditions  | Pickup Friday 6-8pm     |
    Then the exchange should be created successfully
    And the exchange type should be "SharedPurchase"
    And credits should be 0 # Shared purchase is cost-sharing, not time

  Scenario: Browse available exchanges in building
    Given the following exchanges exist in building:
      | Provider     | Title                | Type          | Credits | Status   |
      | Alice        | Plumbing repair      | Service       | 2       | Offered  |
      | Bob          | Electric drill       | ObjectLoan    | 1       | Offered  |
      | Alice        | Garden work          | Service       | 3       | Requested|
    When I browse available exchanges
    Then I should see 2 exchanges # Offered only, exclude own offers
    And I should see "Electric drill" by Bob
    And I should NOT see "Plumbing repair" # My own offer
    And I should NOT see "Garden work" # Status = Requested

  Scenario: Request service from neighbor
    Given Alice has an exchange offer "Plumbing repair" for 2 credits
    And I am authenticated as Bob
    When I request the exchange
    Then the exchange status should change to "Requested"
    And I should become the requester
    And Alice should receive a notification

  Scenario: Provider starts exchange work
    Given Bob requested Alice's "Plumbing repair" service
    And I am authenticated as Alice (provider)
    When I start the exchange
    Then the exchange status should change to "InProgress"
    And the started_at timestamp should be set

  Scenario: Complete exchange and automatic credit transfer
    Given an exchange in status "InProgress" between Alice (provider) and Bob (requester) for 2 credits
    And Alice's current balance is 5 credits
    And Bob's current balance is 3 credits
    And I am authenticated as Alice
    When I complete the exchange
    Then the exchange status should change to "Completed"
    And Alice's balance should be 7 credits # 5 + 2
    And Bob's balance should be 1 credit # 3 - 2
    And both owners should receive confirmation notifications

  Scenario: Negative balance allowed (trust-based system)
    Given Bob has 0 credits balance
    And Bob requests a 2-credit service from Alice
    When Alice completes the exchange
    Then Bob's balance should be -2 credits
    And the system should allow the negative balance
    And Bob should see warning "Please offer services to rebalance"

  Scenario: Mutual rating after completion
    Given a completed exchange between Alice (provider) and Bob (requester)
    When Bob rates Alice's service with 5 stars and comment "Excellent plumbing work"
    And Alice rates Bob with 4 stars and comment "Punctual and respectful"
    Then Alice's average rating should be updated
    And Bob's average rating should be updated
    And ratings should be visible in profiles

  Scenario: Cancel exchange with reason
    Given an exchange in status "Requested" exists
    When I cancel the exchange with reason "Provider no longer available"
    Then the exchange status should change to "Cancelled"
    And the cancellation reason should be recorded
    And no credit transfer should occur

  Scenario: View credit balance and participation level
    Given I have completed 8 exchanges
    And I have earned 15 credits and spent 10 credits
    When I view my credit balance
    Then my balance should be 5 credits
    And my total exchanges should be 8
    And my participation level should be "Active" # 6-20 exchanges
    And my credit status should be "Positive" # Balance > 0

  Scenario: Building leaderboard shows top contributors
    Given the following owners have balances in building:
      | Owner   | Balance |
      | Alice   | 20      |
      | Bob     | 15      |
      | Charlie | 10      |
      | David   | -5      |
    When I view the building leaderboard
    Then I should see top 10 contributors
    And Alice should be ranked #1 with 20 credits
    And Bob should be ranked #2 with 15 credits
    And the leaderboard should encourage participation

  Scenario: SEL statistics for building
    Given 50 exchanges exist in building with:
      | Total exchanges      | 50 |
      | Active exchanges     | 5  |
      | Completed exchanges  | 40 |
      | Cancelled exchanges  | 5  |
      | Total credits exchanged | 120 |
      | Active participants  | 15 |
      | Average rating       | 4.5 |
    When I request SEL statistics
    Then I should see all statistics
    And most popular exchange type should be "Service"

  Scenario: Owner exchange summary across buildings
    Given I participate in SEL in 2 buildings
    And I have offered 10 services, requested 8, completed 15 total
    When I request my exchange summary
    Then I should see:
      | total_offered   | 10  |
      | total_requested | 8   |
      | total_completed | 15  |
      | credits_earned  | 25  |
      | credits_spent   | 20  |
      | balance         | 5   |
      | average_rating  | 4.7 |
      | participation_level | Active |

  Scenario: Participation levels based on activity
    Given the following owners:
      | Owner   | Total Exchanges | Participation Level |
      | Alice   | 0               | New                 |
      | Bob     | 3               | Beginner            |
      | Charlie | 10              | Active              |
      | David   | 25              | Veteran             |
      | Eve     | 60              | Expert              |
    When I check participation levels
    Then they should be correctly categorized
    And participation level badges should be displayed

  Scenario: Prevent self-requesting exchange
    Given I have created an exchange offer
    When I try to request my own exchange
    Then the request should fail
    And I should see error "Cannot request your own exchange"

  Scenario: Search exchanges by type
    Given 20 exchanges exist in building
    And 8 are type "Service"
    And 7 are type "ObjectLoan"
    And 5 are type "SharedPurchase"
    When I filter by exchange type "Service"
    Then I should see 8 exchanges
    And all should be type "Service"
