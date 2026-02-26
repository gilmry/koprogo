# Feature: Resource Booking Calendar (Issue #49 Phase 5)
# Resource types: CommonRoom, LaundryRoom, Parking, Garden, RooftopTerrace, GymRoom, etc.
# Statuses: Pending -> Confirmed -> Completed/Cancelled/NoShow

Feature: Resource Booking Calendar
  As a co-owner
  I want to book shared building resources
  So that I can reserve rooms and spaces when I need them

  Background:
    Given the system is initialized
    And an organization "Booking Copro ASBL" exists with id "org-booking"
    And a building "Residence Reservations" exists in organization "org-booking"
    And an owner "Marie Reservatrice" exists in building "Residence Reservations"
    And an owner "Pierre Concurrent" exists in building "Residence Reservations"
    And a resource "Salle Commune" of type "CommonRoom" exists

  # === CREATION ===

  Scenario: Create a booking for the common room
    When "Marie Reservatrice" books "Salle Commune":
      | start_time | 2026-03-15T14:00:00Z |
      | end_time   | 2026-03-15T18:00:00Z |
      | purpose    | Birthday party       |
    Then the booking should be created
    And the booking status should be "Pending"

  Scenario: Booking creation with deposit
    When "Marie Reservatrice" books "Salle Commune":
      | start_time     | 2026-03-20T10:00:00Z |
      | end_time       | 2026-03-20T12:00:00Z |
      | purpose        | Yoga session         |
      | deposit_amount | 50                   |
    Then the booking should be created

  # === CONFLICT DETECTION ===

  Scenario: Detect booking conflict
    Given "Marie Reservatrice" has booked "Salle Commune" from 14:00 to 18:00 on March 15
    When "Pierre Concurrent" tries to book "Salle Commune" from 16:00 to 20:00 on March 15
    Then the booking should be rejected
    And the error should mention "conflict"

  Scenario: No conflict with adjacent time slots
    Given "Marie Reservatrice" has booked "Salle Commune" from 14:00 to 16:00
    When "Pierre Concurrent" books "Salle Commune" from 16:00 to 18:00
    Then the booking should be created

  # === LIFECYCLE ===

  Scenario: Confirm a pending booking
    Given a pending booking exists
    When the syndic confirms the booking
    Then the booking status should be "Confirmed"

  Scenario: Complete a booking
    Given a confirmed booking that has passed
    When the booking is marked as completed
    Then the booking status should be "Completed"

  Scenario: Cancel a booking
    Given a pending booking exists
    When "Marie Reservatrice" cancels the booking
    Then the booking status should be "Cancelled"

  Scenario: Mark a no-show
    Given a confirmed booking that was missed
    When the syndic marks it as no-show
    Then the booking status should be "NoShow"

  # === LISTING ===

  Scenario: List my bookings
    Given "Marie Reservatrice" has 3 bookings
    When "Marie Reservatrice" lists their bookings
    Then they should get 3 bookings

  Scenario: List active bookings for building
    Given 2 active and 1 cancelled bookings exist
    When I list active bookings
    Then I should get 2 bookings

  Scenario: List bookings by resource type
    Given bookings for CommonRoom and LaundryRoom exist
    When I list bookings for resource type "CommonRoom"
    Then all returned bookings should be for "CommonRoom"

  Scenario: List bookings for specific resource
    Given bookings for 2 different resources exist
    When I list bookings for resource "Salle Commune"
    Then all returned bookings should be for "Salle Commune"

  Scenario: List upcoming bookings
    Given future and past bookings exist
    When I list upcoming bookings
    Then all returned bookings should be in the future

  Scenario: List past bookings
    Given future and past bookings exist
    When I list past bookings
    Then all returned bookings should be in the past

  # === UPDATE ===

  Scenario: Update booking details
    Given a pending booking exists
    When I update the booking purpose to "Team meeting"
    Then the purpose should be updated

  # === STATISTICS ===

  Scenario: Get booking statistics for building
    Given multiple bookings in various statuses exist
    When I get booking statistics
    Then the stats should include total bookings
    And the stats should include bookings by resource type
    And the stats should include completion rate
