# Feature: GDPR Consent Records (GDPR Art. 7, Art. 13-14)
# Records and checks user consent for privacy policy and terms of service.
# Full audit trail: IP address, user agent, timestamp, policy version.

Feature: GDPR Consent Management
  As a user of KoproGo
  I want to record and manage my consent to privacy policy and terms
  So that the platform complies with GDPR Articles 7, 13, and 14

  Background:
    Given the system is initialized
    And an organization "Consent Copro ASBL" exists with id "org-consent"
    And a user "Jean Dupont" exists in organization "org-consent"
    And the user is authenticated as "Jean Dupont"

  # === RECORD CONSENT ===

  Scenario: Successfully record consent to privacy policy
    When I record consent:
      | consent_type   | privacy_policy                  |
      | policy_version | 1.0                             |
    Then the consent should be recorded successfully
    And the response consent_type should be "privacy_policy"
    And the response should contain an accepted_at timestamp
    And the response message should contain "Consent to privacy_policy recorded successfully"

  Scenario: Successfully record consent to terms of service
    When I record consent:
      | consent_type   | terms                           |
      | policy_version | 2.1                             |
    Then the consent should be recorded successfully
    And the response consent_type should be "terms"
    And the response message should contain "Consent to terms recorded successfully"

  Scenario: Record consent without specifying policy version defaults to 1.0
    When I record consent:
      | consent_type | privacy_policy                   |
    Then the consent should be recorded successfully
    And the response consent_type should be "privacy_policy"

  Scenario: Record consent includes IP address in audit trail
    When I record consent with header "X-Forwarded-For" set to "192.168.1.100":
      | consent_type   | privacy_policy                  |
    Then the consent should be recorded successfully
    And the audit trail should include the client IP address

  Scenario: Record consent includes user agent in audit trail
    When I record consent with header "User-Agent" set to "KoproGo-Mobile/1.0":
      | consent_type   | terms                           |
    Then the consent should be recorded successfully
    And the audit trail should include the user agent

  Scenario: Reject consent with invalid consent_type
    When I record consent:
      | consent_type | marketing_emails                 |
    Then the consent recording should fail with status 400
    And the error should contain "Invalid consent_type. Must be 'privacy_policy' or 'terms'"

  Scenario: Reject consent with empty consent_type
    When I record consent:
      | consent_type |                                   |
    Then the consent recording should fail with status 400
    And the error should contain "Invalid consent_type"

  Scenario: Reject consent without authentication
    Given the user is not authenticated
    When I record consent:
      | consent_type | privacy_policy                   |
    Then the consent recording should fail with status 401

  # === CHECK CONSENT STATUS ===

  Scenario: Check consent status when no consent has been given
    When I check my consent status
    Then the response should indicate privacy_policy_accepted is false
    And the response should indicate terms_accepted is false
    And the response should contain my user_id

  Scenario: Check consent status after accepting privacy policy
    Given I have already given consent to "privacy_policy"
    When I check my consent status
    Then the response should indicate privacy_policy_accepted is true
    And the privacy_policy_accepted_at should contain a timestamp

  Scenario: Check consent status after accepting both privacy policy and terms
    Given I have already given consent to "privacy_policy"
    And I have already given consent to "terms"
    When I check my consent status
    Then the response should indicate privacy_policy_accepted is true
    And the response should indicate terms_accepted is true

  Scenario: Check consent status without authentication
    Given the user is not authenticated
    When I check my consent status
    Then the response status should be 401

  # === GDPR ART. 7 COMPLIANCE ===

  Scenario: Consent can be re-recorded to update policy version
    Given I have already given consent to "privacy_policy" version "1.0"
    When I record consent:
      | consent_type   | privacy_policy                  |
      | policy_version | 2.0                             |
    Then the consent should be recorded successfully
    And the new consent should reflect version "2.0"

  Scenario: Each consent record preserves a separate audit entry
    When I record consent:
      | consent_type   | privacy_policy                  |
      | policy_version | 1.0                             |
    And I record consent:
      | consent_type   | terms                           |
      | policy_version | 1.0                             |
    Then both consent records should be stored with distinct timestamps
