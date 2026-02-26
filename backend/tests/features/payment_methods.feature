# Feature: Payment Methods Management (Issue #84)
# Types: Card, SepaDebit, BankTransfer, Cash
# Supports: Default management, activation/deactivation, expiration

Feature: Payment Methods Management
  As a co-owner
  I want to manage my stored payment methods
  So that I can easily pay my building charges

  Background:
    Given the system is initialized
    And an organization "PM Copro ASBL" exists with id "org-pm"
    And a building "Residence Carte" exists in organization "org-pm"
    And an owner "Sophie Payeuse" exists in building "Residence Carte"

  # === CREATION ===

  Scenario: Add a card payment method
    When I add a payment method:
      | owner_id                  | Sophie Payeuse          |
      | method_type               | Card                    |
      | stripe_payment_method_id  | pm_card_visa_1234       |
      | stripe_customer_id        | cus_sophie_001          |
      | display_label             | Visa ending 1234        |
      | is_default                | true                    |
    Then the payment method should be created
    And it should be marked as default
    And it should be active

  Scenario: Add a SEPA debit payment method
    When I add a payment method:
      | owner_id                  | Sophie Payeuse          |
      | method_type               | SepaDebit               |
      | stripe_payment_method_id  | pm_sepa_be_5678         |
      | stripe_customer_id        | cus_sophie_001          |
      | display_label             | IBAN BE68 **** 5678     |
      | is_default                | false                   |
    Then the payment method should be created
    And it should not be marked as default

  # === DEFAULT MANAGEMENT ===

  Scenario: Set payment method as default (atomic - only one default)
    Given owner "Sophie Payeuse" has a default card "Visa 1234"
    And owner "Sophie Payeuse" has a non-default card "Mastercard 5678"
    When I set "Mastercard 5678" as default
    Then "Mastercard 5678" should be the default
    And "Visa 1234" should no longer be default

  Scenario: Only one default payment method per owner
    Given owner "Sophie Payeuse" has 3 payment methods
    When I set the third one as default
    Then exactly 1 payment method should be default

  # === ACTIVATION/DEACTIVATION ===

  Scenario: Deactivate a payment method
    Given owner "Sophie Payeuse" has an active payment method
    When I deactivate the payment method
    Then the payment method should be inactive
    And it should not appear in the active list

  Scenario: Reactivate a deactivated payment method
    Given owner "Sophie Payeuse" has an inactive payment method
    When I reactivate the payment method
    Then the payment method should be active again

  # === LISTING ===

  Scenario: List active payment methods for owner
    Given owner "Sophie Payeuse" has 2 active and 1 inactive payment methods
    When I list active payment methods for "Sophie Payeuse"
    Then I should get 2 payment methods
    And all should be active

  Scenario: Check if owner has active payment methods
    Given owner "Sophie Payeuse" has at least 1 active payment method
    When I check if "Sophie Payeuse" has active payment methods
    Then the result should be true

  Scenario: Count active payment methods
    Given owner "Sophie Payeuse" has 3 active payment methods
    When I count active payment methods for "Sophie Payeuse"
    Then the count should be 3

  # === DELETION ===

  Scenario: Delete a payment method
    Given owner "Sophie Payeuse" has a non-default payment method
    When I delete the payment method
    Then the payment method should be deleted
