# Feature: GDPR Article 30 Register (Processing Activities & Sub-Processors)
# Belgian GDPR compliance: records of processing activities and
# sub-processor agreements (DPA status, transfer mechanisms, certifications).

Feature: GDPR Article 30 Processing Register
  As a superadmin
  I want to view the register of processing activities and sub-processors
  So that KoproGo maintains GDPR Article 30 compliance

  Background:
    Given the system is initialized
    And an organization "GDPR Copro ASBL" exists with id "org-gdpr"

  # === PROCESSING ACTIVITIES ===

  Scenario: SuperAdmin lists all processing activities
    Given the user is authenticated as a superadmin
    And the following processing activities exist:
      | activity_name          | purpose                      | legal_basis       |
      | Owner Data Management  | Manage co-owner personal data | Contractual       |
      | Payment Processing     | Process owner contributions   | Legal obligation  |
      | Meeting Minutes        | Record AG proceedings         | Legitimate interest |
    When I list all processing activities
    Then I should receive a list of 3 processing activities
    And each activity should have an activity_name, purpose, and legal_basis
    And the total count should be 3

  Scenario: Processing activity contains required GDPR fields
    Given the user is authenticated as a superadmin
    And a processing activity "Owner Data Management" exists with:
      | controller_name    | Copro ASBL Syndic              |
      | purpose            | Manage co-owner personal data  |
      | legal_basis        | Contractual                    |
      | data_categories    | name,email,phone,address       |
      | data_subjects      | co-owners,tenants              |
      | recipients         | syndic,accountant              |
      | retention_period   | Duration of ownership + 5 years |
      | security_measures  | Encryption at rest, TLS, RBAC  |
    When I list all processing activities
    Then the activity "Owner Data Management" should contain all mandatory Art. 30 fields
    And the data_categories should include "name" and "email"
    And the data_subjects should include "co-owners"
    And the recipients should include "syndic"

  Scenario: Processing activities are returned in reverse chronological order
    Given the user is authenticated as a superadmin
    And processing activities were created in order: "Activity A", "Activity B", "Activity C"
    When I list all processing activities
    Then "Activity C" should appear before "Activity A" in the results

  Scenario: SuperAdmin gets empty list when no activities exist
    Given the user is authenticated as a superadmin
    And no processing activities exist
    When I list all processing activities
    Then I should receive an empty list of processing activities
    And the total count should be 0

  Scenario: Reject listing processing activities as a syndic
    Given the user is authenticated as a syndic in organization "org-gdpr"
    When I list all processing activities
    Then the response status should be 403
    And the error should contain "Access denied. SuperAdmin role required."

  Scenario: Reject listing processing activities as an owner
    Given the user is authenticated as an owner in organization "org-gdpr"
    When I list all processing activities
    Then the response status should be 403
    And the error should contain "Access denied. SuperAdmin role required."

  Scenario: Reject listing processing activities without authentication
    Given the user is not authenticated
    When I list all processing activities
    Then the response status should be 401

  # === SUB-PROCESSORS ===

  Scenario: SuperAdmin lists all sub-processor agreements
    Given the user is authenticated as a superadmin
    And the following sub-processor agreements exist:
      | processor_name | service_description            | dpa_signed_at        |
      | Stripe         | Payment processing             | 2025-01-15T10:00:00Z |
      | SendGrid       | Transactional email delivery   | 2025-02-01T10:00:00Z |
      | Hetzner        | Cloud infrastructure hosting   | 2025-03-01T10:00:00Z |
    When I list all sub-processors
    Then I should receive a list of 3 sub-processors
    And each processor should have a processor_name and service_description
    And the total count should be 3

  Scenario: Sub-processor agreement contains DPA status fields
    Given the user is authenticated as a superadmin
    And a sub-processor agreement "Stripe" exists with:
      | service_description | Payment processing              |
      | dpa_signed_at       | 2025-01-15T10:00:00Z           |
      | dpa_url             | https://stripe.com/dpa          |
      | transfer_mechanism  | Standard Contractual Clauses    |
      | data_categories     | payment_data,owner_email        |
      | certifications      | SOC2,PCI-DSS                    |
    When I list all sub-processors
    Then the processor "Stripe" should have dpa_signed_at set
    And the processor "Stripe" should have a dpa_url
    And the transfer_mechanism should be "Standard Contractual Clauses"
    And the certifications should include "PCI-DSS"

  Scenario: Sub-processor without signed DPA is flagged
    Given the user is authenticated as a superadmin
    And a sub-processor agreement "New Vendor" exists with:
      | service_description | Analytics service               |
      | dpa_signed_at       |                                 |
      | dpa_url             |                                 |
    When I list all sub-processors
    Then the processor "New Vendor" should have dpa_signed_at as null
    And the processor "New Vendor" should have dpa_url as null

  Scenario: Sub-processors are returned in alphabetical order by name
    Given the user is authenticated as a superadmin
    And the following sub-processor agreements exist:
      | processor_name | service_description            |
      | Stripe         | Payment processing             |
      | AWS            | Cloud infrastructure           |
      | Mailgun        | Email delivery                 |
    When I list all sub-processors
    Then "AWS" should appear before "Mailgun" in the results
    And "Mailgun" should appear before "Stripe" in the results

  Scenario: SuperAdmin gets empty list when no sub-processors exist
    Given the user is authenticated as a superadmin
    And no sub-processor agreements exist
    When I list all sub-processors
    Then I should receive an empty list of sub-processors
    And the total count should be 0

  Scenario: Reject listing sub-processors as a syndic
    Given the user is authenticated as a syndic in organization "org-gdpr"
    When I list all sub-processors
    Then the response status should be 403
    And the error should contain "Access denied. SuperAdmin role required."

  Scenario: Reject listing sub-processors without authentication
    Given the user is not authenticated
    When I list all sub-processors
    Then the response status should be 401
