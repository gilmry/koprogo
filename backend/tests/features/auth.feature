Feature: Authentication and refresh tokens
  As a user
  I want to authenticate and refresh my session
  So that I can access the system securely

  Scenario: Register and login returns tokens
    Given a coproperty management system
    When I register a new user and login
    Then I receive an access token and a refresh token

  Scenario: Refresh token returns a new access token
    Given a coproperty management system
    And I have a valid refresh token
    When I refresh my session
    Then I receive a new access token

  Scenario: Switch active role updates JWT
    Given a coproperty management system
    And a user with multiple roles
    When I switch to the secondary role
    Then my active role should be "accountant"
    And the user response should list multiple roles
    And the JWT claims should use role "accountant"
    And the JWT claims should reference the selected role
