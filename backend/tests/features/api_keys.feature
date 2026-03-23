# Feature: API Key Management (Issues #111, #232)
# Enables third-party integrations with KoproGo via API keys.
# Supports: PropTech, notaries, energy providers, accounting software.

Feature: API Key Management
  As a syndic or superadmin
  I want to manage API keys for external integrations
  So that third-party systems can securely access KoproGo

  Background:
    Given the system is initialized
    And an organization "API Key Copro ASBL" exists with id "org-apikey"
    And the user is authenticated as a syndic in organization "org-apikey"

  # === CREATION ===

  Scenario: Successfully create an API key with valid permissions
    When I create an API key:
      | name        | My PropTech Integration          |
      | permissions | read:buildings,read:expenses      |
      | rate_limit  | 500                               |
    Then the API key should be created successfully
    And the response should contain a full key starting with "kpg_live_"
    And the response should contain a key_prefix "kpg_live_"
    And the API key should be active
    And the response should contain a warning about storing the key securely

  Scenario: Create an API key with default rate limit
    When I create an API key:
      | name        | Notary Integration               |
      | permissions | read:etats-dates,write:etats-dates |
    Then the API key should be created successfully
    And the API key rate_limit should be 100

  Scenario: Create an API key with optional expiration date
    When I create an API key:
      | name        | Temporary Integration            |
      | permissions | read:buildings                    |
      | expires_at  | 2027-06-30T23:59:59Z             |
    Then the API key should be created successfully
    And the API key should have an expiration date

  Scenario: Reject creating API key with empty name
    When I create an API key:
      | name        |                                   |
      | permissions | read:buildings                    |
    Then the API key creation should fail
    And the error should contain "API key name must be between 1 and 255 characters"

  Scenario: Reject creating API key with name exceeding 255 characters
    When I create an API key with a name of 256 characters
    Then the API key creation should fail
    And the error should contain "API key name must be between 1 and 255 characters"

  Scenario: Reject creating API key with invalid permission
    When I create an API key:
      | name        | Bad Perms Integration            |
      | permissions | read:buildings,delete:everything  |
    Then the API key creation should fail
    And the error should contain "Invalid permission: delete:everything"

  Scenario: Reject creating API key with rate limit below minimum
    When I create an API key:
      | name        | Low Rate Integration             |
      | permissions | read:buildings                    |
      | rate_limit  | 0                                 |
    Then the API key creation should fail
    And the error should contain "Rate limit must be between 1 and 10,000"

  Scenario: Reject creating API key with rate limit above maximum
    When I create an API key:
      | name        | High Rate Integration            |
      | permissions | read:buildings                    |
      | rate_limit  | 99999                             |
    Then the API key creation should fail
    And the error should contain "Rate limit must be between 1 and 10,000"

  Scenario: Reject creating API key as a regular owner
    Given the user is authenticated as an owner in organization "org-apikey"
    When I create an API key:
      | name        | Unauthorized Integration         |
      | permissions | read:buildings                    |
    Then the API key creation should fail with status 403
    And the error should contain "Only syndics and admins can create API keys"

  # === LISTING ===

  Scenario: List all API keys for the organization
    Given an API key "Integration A" exists in organization "org-apikey"
    And an API key "Integration B" exists in organization "org-apikey"
    When I list all API keys
    Then I should receive a list of 2 API keys
    And the API key bodies should not be visible in the response
    And each key should have a key_prefix field

  # === GET BY ID ===

  Scenario: Get a specific API key by ID
    Given an API key "My Integration" exists in organization "org-apikey"
    When I get the API key by its ID
    Then I should receive the API key details
    And the API key name should be "My Integration"
    And the full key body should not be visible

  Scenario: Reject getting a non-existent API key
    When I get an API key with a random UUID
    Then the response status should be 404
    And the error should contain "API key not found"

  # === UPDATE ===

  Scenario: Successfully update an API key name and rate limit
    Given an API key "Old Name" exists in organization "org-apikey"
    When I update the API key:
      | name       | New Name                          |
      | rate_limit | 200                               |
    Then the API key should be updated successfully
    And the API key name should be "New Name"
    And the API key rate_limit should be 200

  Scenario: Reject updating an API key created by another user
    Given another syndic created an API key "Other Key" in organization "org-apikey"
    When I update the API key:
      | name | Hijacked Name                      |
    Then the update should fail with status 403
    And the error should contain "Only the API key creator can update it"

  # === REVOCATION ===

  Scenario: Successfully revoke an API key
    Given an API key "Revocable Key" exists in organization "org-apikey"
    When I revoke the API key
    Then the response should indicate success
    And the response message should contain "API key revoked successfully"

  Scenario: Reject revoking a non-existent API key
    When I revoke an API key with a random UUID
    Then the response status should be 404
    And the error should contain "API key not found"

  # === ROTATION ===

  Scenario: Key rotation returns not implemented
    Given an API key "Rotatable Key" exists in organization "org-apikey"
    When I rotate the API key
    Then the response status should be 501
    And the error should contain "Key rotation not yet implemented"
