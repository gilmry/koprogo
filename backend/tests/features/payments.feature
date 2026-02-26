# Feature: Payment Integration (Issue #84)
# Stripe Payment Intents + SEPA Direct Debit
# Lifecycle: Pending -> Processing -> RequiresAction/Succeeded/Failed/Cancelled -> Refunded

Feature: Payment Integration System
  As a co-owner
  I want to make payments for my building charges
  So that I can settle my debts electronically

  Background:
    Given the system is initialized
    And an organization "Pay Copro ASBL" exists with id "org-pay"
    And a building "Residence Paiement" exists in organization "org-pay"
    And an owner "Jean Payeur" exists in building "Residence Paiement"
    And an expense of 500 EUR exists for building "Residence Paiement"

  # === PAYMENT CREATION ===

  Scenario: Create a card payment for an expense
    When I create a payment:
      | owner_id         | Jean Payeur           |
      | amount_cents     | 50000                 |
      | method_type      | Card                  |
      | idempotency_key  | pay-card-001          |
      | description      | Charges Q1 2026       |
    Then the payment should be created successfully
    And the payment status should be "Pending"
    And the payment amount should be 50000 cents
    And the currency should be "EUR"

  Scenario: Create a SEPA direct debit payment
    When I create a payment:
      | owner_id         | Jean Payeur           |
      | amount_cents     | 25000                 |
      | method_type      | SepaDebit             |
      | idempotency_key  | pay-sepa-001          |
    Then the payment should be created successfully
    And the payment method type should be "SepaDebit"

  Scenario: Payment creation fails with zero amount
    When I create a payment:
      | owner_id         | Jean Payeur           |
      | amount_cents     | 0                     |
      | method_type      | Card                  |
      | idempotency_key  | pay-zero-001          |
    Then the payment creation should fail
    And the error should contain "amount"

  Scenario: Payment creation fails with negative amount
    When I create a payment:
      | owner_id         | Jean Payeur           |
      | amount_cents     | -100                  |
      | method_type      | Card                  |
      | idempotency_key  | pay-neg-001           |
    Then the payment creation should fail

  # === PAYMENT LIFECYCLE ===

  Scenario: Payment lifecycle - Pending to Succeeded
    Given a pending payment of 50000 cents exists
    When I mark the payment as processing
    Then the payment status should be "Processing"
    When I mark the payment as succeeded
    Then the payment status should be "Succeeded"
    And the succeeded_at timestamp should be set

  Scenario: Payment lifecycle - Pending to Failed
    Given a pending payment of 30000 cents exists
    When I mark the payment as processing
    And I mark the payment as failed with reason "Insufficient funds"
    Then the payment status should be "Failed"
    And the failure reason should be "Insufficient funds"

  Scenario: Payment lifecycle - RequiresAction flow (3D Secure)
    Given a pending payment of 50000 cents exists
    When I mark the payment as processing
    And I mark the payment as requires action
    Then the payment status should be "RequiresAction"
    When I mark the payment as succeeded
    Then the payment status should be "Succeeded"

  Scenario: Cancel a pending payment
    Given a pending payment of 50000 cents exists
    When I cancel the payment
    Then the payment status should be "Cancelled"
    And the cancelled_at timestamp should be set

  # === REFUNDS ===

  Scenario: Full refund of a succeeded payment
    Given a succeeded payment of 50000 cents exists
    When I refund 50000 cents
    Then the payment status should be "Refunded"
    And the refunded amount should be 50000 cents
    And the net amount should be 0 cents

  Scenario: Partial refund of a succeeded payment
    Given a succeeded payment of 50000 cents exists
    When I refund 20000 cents
    Then the payment status should be "Refunded"
    And the refunded amount should be 20000 cents
    And the net amount should be 30000 cents

  Scenario: Over-refund prevention
    Given a succeeded payment of 50000 cents exists
    And 30000 cents have already been refunded
    When I try to refund 30000 cents
    Then the refund should fail
    And the error should contain "exceeds"

  Scenario: Cannot refund a pending payment
    Given a pending payment of 50000 cents exists
    When I try to refund 50000 cents
    Then the refund should fail

  # === IDEMPOTENCY ===

  Scenario: Idempotency key prevents duplicate payments
    When I create a payment:
      | owner_id         | Jean Payeur           |
      | amount_cents     | 50000                 |
      | method_type      | Card                  |
      | idempotency_key  | pay-idem-unique       |
    Then the payment should be created successfully
    When I create another payment with idempotency_key "pay-idem-unique"
    Then the duplicate payment creation should fail

  # === LISTING & STATISTICS ===

  Scenario: List payments by owner
    Given 3 payments exist for owner "Jean Payeur"
    When I list payments for owner "Jean Payeur"
    Then I should get 3 payments

  Scenario: List payments by status
    Given a succeeded payment exists
    And a failed payment exists
    When I list payments with status "Succeeded"
    Then all returned payments should have status "Succeeded"

  Scenario: Get payment statistics for owner
    Given 2 succeeded payments of 30000 cents each
    And 1 failed payment of 10000 cents
    When I get payment stats for owner "Jean Payeur"
    Then the total succeeded amount should be 60000 cents
    And the succeeded count should be 2
    And the failed count should be 1

  Scenario: Get total paid for expense
    Given 2 succeeded payments for the expense totaling 80000 cents
    When I get total paid for the expense
    Then the total should be 80000 cents

  # === DELETION ===

  Scenario: Delete a payment
    Given a pending payment exists
    When I delete the payment
    Then the payment should be deleted
