# Feature: Security Incident Management (GDPR Art. 33/34 - Belgian APD Notification)
# Report, track, and notify security incidents within 72 hours.
# Belgian Data Protection Authority (APD/GBA) notification compliance.

Feature: Security Incident Management
  As a superadmin
  I want to report and manage security incidents
  So that KoproGo complies with GDPR Art. 33 (72-hour APD notification rule)

  Background:
    Given the system is initialized
    And an organization "Security Copro ASBL" exists with id "org-security"

  # === INCIDENT CREATION ===

  Scenario: SuperAdmin reports a critical data breach incident
    Given the user is authenticated as a superadmin in organization "org-security"
    When I create a security incident:
      | severity                  | critical                          |
      | incident_type             | data_breach                       |
      | title                     | Unauthorized database access      |
      | description               | External actor gained read access to owner table |
      | data_categories_affected  | name,email,phone                  |
      | affected_subjects_count   | 150                               |
    Then the incident should be created successfully with status 201
    And the incident status should be "detected"
    And the incident severity should be "critical"
    And the incident should contain hours_since_discovery
    And the incident should have a discovery_at timestamp

  Scenario: SuperAdmin reports a low severity malware incident
    Given the user is authenticated as a superadmin in organization "org-security"
    When I create a security incident:
      | severity                  | low                               |
      | incident_type             | malware                           |
      | title                     | Suspicious file detected          |
      | description               | Antivirus flagged a file in uploads directory |
      | data_categories_affected  | documents                         |
    Then the incident should be created successfully with status 201
    And the incident status should be "detected"
    And the incident severity should be "low"

  Scenario: Reject creating incident with empty title
    Given the user is authenticated as a superadmin in organization "org-security"
    When I create a security incident:
      | severity    | high                                |
      | incident_type | unauthorized_access               |
      | title       |                                     |
      | description | Some description                    |
      | data_categories_affected | email                  |
    Then the incident creation should fail with status 400
    And the error should contain "title and description are required"

  Scenario: Reject creating incident with empty description
    Given the user is authenticated as a superadmin in organization "org-security"
    When I create a security incident:
      | severity    | high                                |
      | incident_type | unauthorized_access               |
      | title       | Valid title                         |
      | description |                                     |
      | data_categories_affected | email                  |
    Then the incident creation should fail with status 400
    And the error should contain "title and description are required"

  Scenario: Reject creating incident with invalid severity
    Given the user is authenticated as a superadmin in organization "org-security"
    When I create a security incident:
      | severity    | extreme                             |
      | incident_type | data_breach                       |
      | title       | Some incident                       |
      | description | Some description                    |
      | data_categories_affected | email                  |
    Then the incident creation should fail with status 400
    And the error should contain "Invalid severity. Must be: critical, high, medium, or low"

  Scenario: Reject creating incident as a syndic
    Given the user is authenticated as a syndic in organization "org-security"
    When I create a security incident:
      | severity    | high                                |
      | incident_type | data_breach                       |
      | title       | Unauthorized access                 |
      | description | Syndic reporting incident           |
      | data_categories_affected | email                  |
    Then the incident creation should fail with status 403
    And the error should contain "Access denied. SuperAdmin role required."

  # === LISTING INCIDENTS ===

  Scenario: SuperAdmin lists all security incidents
    Given the user is authenticated as a superadmin in organization "org-security"
    And the following security incidents exist:
      | title                    | severity | status     |
      | Data breach attempt      | critical | detected   |
      | Suspicious login         | medium   | investigating |
      | Phishing email received  | low      | contained  |
    When I list all security incidents
    Then I should receive a list of 3 security incidents
    And the incidents should be ordered by discovery_at descending

  Scenario: SuperAdmin filters incidents by severity
    Given the user is authenticated as a superadmin in organization "org-security"
    And security incidents of severity "critical", "medium", and "low" exist
    When I list security incidents with filter severity "critical"
    Then all returned incidents should have severity "critical"

  Scenario: SuperAdmin filters incidents by status
    Given the user is authenticated as a superadmin in organization "org-security"
    And security incidents with statuses "detected", "investigating", and "reported" exist
    When I list security incidents with filter status "detected"
    Then all returned incidents should have status "detected"

  Scenario: SuperAdmin paginates incident list
    Given the user is authenticated as a superadmin in organization "org-security"
    And 25 security incidents exist in organization "org-security"
    When I list security incidents with page 1 and per_page 10
    Then I should receive a list of 10 security incidents
    And the total count should be 25

  Scenario: Reject listing incidents as a non-superadmin
    Given the user is authenticated as a syndic in organization "org-security"
    When I list all security incidents
    Then the response status should be 403
    And the error should contain "Access denied. SuperAdmin role required."

  # === GET SINGLE INCIDENT ===

  Scenario: SuperAdmin retrieves a specific incident by ID
    Given the user is authenticated as a superadmin in organization "org-security"
    And a security incident "Database intrusion" exists in organization "org-security"
    When I get the security incident by its ID
    Then I should receive the incident details
    And the incident title should be "Database intrusion"
    And the response should contain hours_since_discovery

  Scenario: Reject getting a non-existent incident
    Given the user is authenticated as a superadmin in organization "org-security"
    When I get a security incident with a random UUID
    Then the response status should be 404
    And the error should contain "Security incident not found"

  # === APD NOTIFICATION (72-HOUR RULE) ===

  Scenario: SuperAdmin reports incident to APD with reference number
    Given the user is authenticated as a superadmin in organization "org-security"
    And a security incident "Data breach" exists with status "detected" in organization "org-security"
    When I report the incident to APD:
      | apd_reference_number | APD-2026-003891                 |
      | investigation_notes  | Initial investigation completed |
    Then the incident should be updated successfully
    And the incident status should be "reported"
    And the incident notification_at should contain a timestamp
    And the incident apd_reference_number should be "APD-2026-003891"

  Scenario: Reject reporting to APD without reference number
    Given the user is authenticated as a superadmin in organization "org-security"
    And a security incident "Data breach" exists with status "detected" in organization "org-security"
    When I report the incident to APD:
      | apd_reference_number |                                  |
    Then the APD report should fail with status 400
    And the error should contain "apd_reference_number is required"

  Scenario: Reject reporting an already-reported incident to APD
    Given the user is authenticated as a superadmin in organization "org-security"
    And a security incident "Already Reported Breach" exists with status "reported" and notification_at set
    When I report the incident to APD:
      | apd_reference_number | APD-2026-DUPLICATE               |
    Then the APD report should fail with status 409
    And the error should contain "Incident already reported to APD"

  Scenario: Reject reporting to APD for a non-existent incident
    Given the user is authenticated as a superadmin in organization "org-security"
    When I report a random incident ID to APD:
      | apd_reference_number | APD-2026-GHOST                   |
    Then the response status should be 404
    And the error should contain "Security incident not found"

  # === OVERDUE INCIDENTS (72-HOUR COMPLIANCE) ===

  Scenario: List overdue incidents not yet reported to APD
    Given the user is authenticated as a superadmin in organization "org-security"
    And a security incident "Old Breach" was discovered 80 hours ago and not reported to APD
    And a security incident "Recent Breach" was discovered 10 hours ago and not reported to APD
    When I list overdue security incidents
    Then the overdue list should contain "Old Breach"
    And the overdue list should not contain "Recent Breach"
    And each overdue incident should have hours_since_discovery greater than 72

  Scenario: Overdue list excludes already-reported incidents
    Given the user is authenticated as a superadmin in organization "org-security"
    And a security incident "Reported Breach" was discovered 80 hours ago and already reported to APD
    When I list overdue security incidents
    Then the overdue list should not contain "Reported Breach"

  Scenario: Overdue list excludes closed incidents
    Given the user is authenticated as a superadmin in organization "org-security"
    And a security incident "Closed Breach" was discovered 100 hours ago with status "closed"
    When I list overdue security incidents
    Then the overdue list should not contain "Closed Breach"

  Scenario: Reject listing overdue incidents as a non-superadmin
    Given the user is authenticated as a syndic in organization "org-security"
    When I list overdue security incidents
    Then the response status should be 403
    And the error should contain "Access denied. SuperAdmin role required."
