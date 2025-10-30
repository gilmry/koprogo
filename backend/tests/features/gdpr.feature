Feature: GDPR Data Rights (Articles 15 & 17)
  As a data subject
  I want to exercise my GDPR rights
  So that I can control my personal data

  # Article 15 - Right to Access

  Scenario: User exports their personal data
    Given a coproperty management system
    And I am an authenticated user with personal data
    When I request to export my personal data
    Then I should receive a complete data export
    And the export should include my user information
    And the export should include my owner records
    And the export should include my unit ownerships
    And an audit log entry should be created for "GdprDataExported"
    And I should receive an email notification about the export

  Scenario: SuperAdmin exports another user's data
    Given a coproperty management system
    And I am a SuperAdmin
    And another user exists with personal data
    When I export that user's data as an admin
    Then I should receive a complete data export for that user
    And the audit log should mark the operation as admin-initiated
    And the user should receive an email about the admin export

  Scenario: User checks if they can erase their data
    Given a coproperty management system
    And I am an authenticated user
    And I have no active legal holds
    When I check if I can erase my data
    Then I should receive confirmation that erasure is possible
    And an audit log entry should be created for "GdprErasureCheckRequested"

  # Article 17 - Right to Erasure

  Scenario: User successfully erases their personal data
    Given a coproperty management system
    And I am an authenticated user
    And I have no active legal holds
    When I request to erase my personal data
    Then my user account should be anonymized
    And all my owner records should be anonymized
    And the anonymization timestamp should be recorded
    And an audit log entry should be created for "GdprDataErased"
    And I should receive an email confirmation of erasure

  Scenario: SuperAdmin erases user data
    Given a coproperty management system
    And I am a SuperAdmin
    And another user exists with no legal holds
    When I erase that user's data as an admin
    Then the user account should be anonymized
    And all owner records should be anonymized
    And the audit log should mark the operation as admin-initiated
    And the user should receive an email about the admin erasure

  Scenario: User cannot erase data due to legal holds
    Given a coproperty management system
    And I am an authenticated user
    And I have active legal holds on my data
    When I request to erase my personal data
    Then the erasure request should be rejected
    And I should receive an error about legal holds
    And an audit log entry should be created for "GdprDataErasureFailed"

  # Security & Compliance

  Scenario: Rate limiting prevents GDPR request abuse
    Given a coproperty management system
    And I am an authenticated user
    And I have made 10 GDPR export requests in the last hour
    When I attempt an 11th GDPR export request
    Then the request should be rejected with HTTP 429
    And I should receive a Retry-After header

  Scenario: Audit logs capture client information
    Given a coproperty management system
    And I am an authenticated user
    When I request to export my personal data
    Then the audit log should include my IP address
    And the audit log should include my User-Agent
    And the audit log should have a 7-year retention period

  Scenario: Cross-organization access is blocked
    Given a coproperty management system
    And I am a regular user in Organization A
    And another user exists in Organization B
    When I attempt to export that user's data
    Then the request should be rejected with HTTP 403
    And I should receive an authorization error

  Scenario: Data export excludes anonymized records
    Given a coproperty management system
    And I am an authenticated user
    And I have already anonymized some of my owner records
    When I request to export my personal data
    Then the export should not include anonymized owner records
    And the export should only include active personal data

  # Email Notifications

  Scenario: Export notification includes security warnings
    Given a coproperty management system
    And I am an authenticated user
    When I request to export my personal data
    Then I should receive an email with the export ID
    And the email should include security warnings
    And the email should advise me to handle the data securely

  Scenario: Erasure notification includes confirmation details
    Given a coproperty management system
    And I am an authenticated user
    And I have 3 owner records
    When I request to erase my personal data
    Then I should receive an email confirming erasure
    And the email should state the anonymization timestamp
    And the email should indicate 3 owner records were anonymized

  # Multi-tenant Scenarios

  Scenario: User in multiple organizations erases data across all
    Given a coproperty management system
    And I am an authenticated user in 2 organizations
    And I am an owner in both organizations
    When I request to erase my personal data
    Then my user account should be anonymized
    And my owner records in both organizations should be anonymized
    And the audit log should record all anonymizations
