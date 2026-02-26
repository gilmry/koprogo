# Feature: Two-Factor Authentication - TOTP (Issue #78)
# Endpoints: setup, enable, verify, disable, regenerate-backup-codes, status

Feature: Two-Factor Authentication (TOTP)
  As a user
  I want to enable 2FA on my account
  So that my account is more secure against unauthorized access

  Background:
    Given the system is initialized
    And an organization "2FA Copro ASBL" exists with id "org-2fa"
    And a registered user "secure@test.be" with password "SecurePass123!"

  # === SETUP ===

  Scenario: Setup 2FA returns QR code and backup codes
    When I setup 2FA for "secure@test.be"
    Then I should receive a QR code URI
    And I should receive backup codes
    And 2FA should not be enabled yet

  # === ENABLE ===

  Scenario: Enable 2FA after verifying TOTP code
    Given 2FA is setup for "secure@test.be"
    When I enable 2FA with a valid TOTP code
    Then 2FA should be enabled
    And the verified_at timestamp should be set

  Scenario: Cannot enable 2FA with invalid TOTP code
    Given 2FA is setup for "secure@test.be"
    When I try to enable 2FA with code "000000"
    Then the enable should fail
    And 2FA should not be enabled

  # === VERIFICATION ===

  Scenario: Verify login with valid TOTP code
    Given 2FA is enabled for "secure@test.be"
    When I verify with a valid TOTP code
    Then the verification should succeed
    And the last_used_at timestamp should be updated

  Scenario: Verify login with invalid TOTP code
    Given 2FA is enabled for "secure@test.be"
    When I verify with code "999999"
    Then the verification should fail

  Scenario: Verify login with backup code
    Given 2FA is enabled for "secure@test.be"
    And I have unused backup codes
    When I verify with a valid backup code
    Then the verification should succeed
    And the backup code should be consumed

  Scenario: Backup code cannot be reused
    Given 2FA is enabled for "secure@test.be"
    And I have used a backup code
    When I try to verify with the same backup code again
    Then the verification should fail

  # === DISABLE ===

  Scenario: Disable 2FA with correct password
    Given 2FA is enabled for "secure@test.be"
    When I disable 2FA with password "SecurePass123!"
    Then 2FA should be disabled
    And the secret should be removed

  Scenario: Cannot disable 2FA with wrong password
    Given 2FA is enabled for "secure@test.be"
    When I try to disable 2FA with password "WrongPassword"
    Then the disable should fail
    And 2FA should still be enabled

  # === BACKUP CODES ===

  Scenario: Regenerate backup codes
    Given 2FA is enabled for "secure@test.be"
    When I regenerate backup codes with a valid TOTP code
    Then I should receive new backup codes
    And the old backup codes should be invalidated

  # === STATUS ===

  Scenario: Check 2FA status when enabled
    Given 2FA is enabled for "secure@test.be"
    When I check 2FA status
    Then the status should show 2FA is enabled
    And it should show the verified_at date

  Scenario: Check 2FA status when disabled
    Given 2FA is not setup for "secure@test.be"
    When I check 2FA status
    Then the status should show 2FA is disabled
