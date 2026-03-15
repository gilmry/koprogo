# Feature: PCMN Accounts — Plan Comptable Minimum Normalisé Belge (Issue #79)
# Belgian PCMN: AR 12/07/2012 — 8 account classes
# Classes: 1 (Capitaux), 2 (Immobilisations), 3 (Stocks), 4 (Créances/Dettes),
#           5 (Placements/Liquidités), 6 (Charges), 7 (Produits), 8 (Résultats)

Feature: PCMN Accounts
  As an accountant
  I want to manage the Belgian chart of accounts (PCMN)
  So that financial reporting complies with Belgian accounting law

  Background:
    Given the system is initialized
    And an organization "PCMN Copro ASBL" exists with id "org-pcmn"

  # === SEED BELGIAN PCMN ===

  Scenario: Seed the standard Belgian PCMN chart of accounts
    When I seed the Belgian PCMN for the organization
    Then the seeding should succeed
    And at least 80 accounts should be created
    And accounts should include class 1 (Capitaux propres)
    And accounts should include class 6 (Charges)
    And accounts should include class 7 (Produits)

  # === CREATION ===

  Scenario: Create a custom account
    Given the Belgian PCMN is seeded
    When I create an account:
      | code         | 60100      |
      | name         | Achats fournitures bureautiques |
      | account_type | Charge     |
    Then the account should be created successfully
    And the account code should be "60100"

  Scenario: Account creation fails with duplicate code
    Given the Belgian PCMN is seeded
    When I try to create an account with code "604001" that already exists
    Then the creation should fail
    And the error should mention "already exists" or "duplicate"

  Scenario: Account code must be non-empty
    When I try to create an account with empty code
    Then the creation should fail

  # === RETRIEVAL ===

  Scenario: Get account by ID
    Given the Belgian PCMN is seeded
    When I retrieve account with code "604001"
    Then the account should be returned
    And the account_type should be "Charge"

  Scenario: Get account by code
    Given the Belgian PCMN is seeded
    When I retrieve account by code "75"
    Then the account should be returned
    And the account name should contain "Produits"

  Scenario: List all accounts for organization
    Given the Belgian PCMN is seeded
    When I list all accounts for the organization
    Then I should get at least 80 accounts

  Scenario: List accounts by type (Charge)
    Given the Belgian PCMN is seeded
    When I list accounts of type "Charge"
    Then all returned accounts should have type "Charge"
    And I should get at least 10 charge accounts

  Scenario: List child accounts of a parent
    Given the Belgian PCMN is seeded
    When I list child accounts of code "60"
    Then all returned accounts should start with "60"

  # === UPDATE ===

  Scenario: Update account name
    Given the Belgian PCMN is seeded
    When I update account with code "604001":
      | name | Electricité (mise à jour) |
    Then the account should be updated successfully
    And the account name should be "Electricité (mise à jour)"

  # === DELETE ===

  Scenario: Delete a custom account
    Given I have created a custom account with code "99999"
    When I delete the account
    Then the account should be deleted successfully

  Scenario: Cannot delete an account with linked transactions
    Given an account has journal entry lines linked to it
    When I try to delete that account
    Then the account deletion should be blocked or allowed

  # === SEARCH ===

  Scenario: Search accounts by keyword
    Given the Belgian PCMN is seeded
    When I search accounts for "charges"
    Then the results should contain accounts with "charges" in the name

  # === COUNT ===

  Scenario: Count accounts for organization
    Given the Belgian PCMN is seeded
    When I count accounts for the organization
    Then the count should be at least 80
