# Feature: Multi-Channel Notification System (Issue #86)
# Channels: Email, InApp, Push
# Types: ExpenseCreated, MeetingConvocation, PaymentReceived, TicketResolved, etc.

Feature: Multi-Channel Notification System
  As a user of the platform
  I want to receive and manage notifications
  So that I stay informed about important events in my building

  Background:
    Given the system is initialized
    And an organization "Notif Copro ASBL" exists with id "org-notif"
    And a building "Residence Connectee" exists in organization "org-notif"
    And a user "user-notif@test.be" exists in organization "org-notif"

  # === NOTIFICATION CREATION ===

  Scenario: Create an in-app notification
    When I create a notification:
      | user_id           | user-notif@test.be       |
      | notification_type | MeetingConvocation       |
      | channel           | InApp                    |
      | priority          | Medium                   |
      | title             | AG convoquée le 15 mars  |
      | message           | L'assemblée générale ordinaire est programmée |
    Then the notification should be created successfully
    And the notification status should be "Pending"
    And the notification channel should be "InApp"

  Scenario: Create a high priority system notification
    When I create a notification:
      | user_id           | user-notif@test.be       |
      | notification_type | System                   |
      | channel           | Email                    |
      | priority          | High                     |
      | title             | Maintenance urgente      |
      | message           | Coupure d'eau prévue demain 8h-12h |
    Then the notification should be created successfully
    And the notification priority should be "High"

  # === READING NOTIFICATIONS ===

  Scenario: Mark notification as read
    Given an unread notification exists for "user-notif@test.be"
    When I mark the notification as read
    Then the notification status should be "Read"
    And the read_at timestamp should be set

  Scenario: Mark all notifications as read
    Given 3 unread notifications exist for "user-notif@test.be"
    When I mark all notifications as read for "user-notif@test.be"
    Then 3 notifications should be marked as read
    And the unread count should be 0

  Scenario: List unread notifications
    Given 2 unread notifications exist for "user-notif@test.be"
    And 1 read notification exists for "user-notif@test.be"
    When I list unread notifications for "user-notif@test.be"
    Then I should get 2 notifications
    And all should have status "Pending" or "Sent"

  # === NOTIFICATION LIFECYCLE ===

  Scenario: Mark notification as sent
    Given a pending notification exists
    When I mark the notification as sent
    Then the notification status should be "Sent"
    And the sent_at timestamp should be set

  Scenario: Mark notification as failed
    Given a pending notification exists
    When I mark the notification as failed with error "SMTP connection refused"
    Then the notification status should be "Failed"
    And the error message should be "SMTP connection refused"

  Scenario: Retry a failed notification
    Given a failed notification exists
    When I retry the notification
    Then the notification status should be "Pending"
    And the error message should be cleared

  # === STATISTICS ===

  Scenario: Get notification statistics for user
    Given 5 notifications exist for "user-notif@test.be" with mixed statuses
    When I get notification stats for "user-notif@test.be"
    Then the stats should include total count
    And the stats should include unread count
    And the stats should include pending count

  # === PREFERENCES ===

  Scenario: Get user notification preferences
    When I get notification preferences for "user-notif@test.be"
    Then I should get a list of preferences

  Scenario: Update notification preference for a type
    When I update preference for "user-notif@test.be" type "MeetingConvocation":
      | email_enabled  | true  |
      | in_app_enabled | true  |
      | push_enabled   | false |
    Then the preference should be updated successfully
    And email should be enabled
    And push should be disabled

  Scenario: Disable all channels for a notification type
    When I update preference for "user-notif@test.be" type "System":
      | email_enabled  | false |
      | in_app_enabled | false |
      | push_enabled   | false |
    Then the preference should be updated successfully

  # === DELETION & CLEANUP ===

  Scenario: Delete a notification
    Given a notification exists for "user-notif@test.be"
    When I delete the notification
    Then the notification should be deleted
    And it should not appear in the user's notification list

  Scenario: Cleanup old notifications
    Given old notifications exist from 60 days ago
    When I cleanup notifications older than 30 days
    Then the old notifications should be deleted
    And recent notifications should remain
