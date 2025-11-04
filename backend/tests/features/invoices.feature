# Invoice Workflow Feature (Issue #73)
# Système complet d'encodage de factures avec workflow de validation

Feature: Invoice Workflow with VAT and Approval
  As an accountant
  I want to create invoices with VAT calculation
  And submit them for syndic approval
  So that charges can be distributed to owners

  Background:
    Given an organization "Syndic Test SPRL"
    And a building "Résidence Bellevue" with 5 units
    And an accountant user "comptable@test.be"
    And a syndic user "syndic@test.be"
    And 5 active unit-owner relationships with ownership percentages

  # ========== Invoice Creation with VAT ==========

  Scenario: Create invoice draft with VAT 21%
    When the accountant creates an invoice draft with:
      | description          | Réparation ascenseur   |
      | amount_excl_vat      | 1000.00                |
      | vat_rate             | 21.00                  |
      | invoice_date         | 2025-01-15             |
      | due_date             | 2025-02-15             |
      | supplier             | ACME Elevators SPRL    |
      | invoice_number       | INV-2025-042           |
    Then the invoice should be created successfully
    And the invoice status should be "draft"
    And the invoice VAT amount should be 210.00
    And the invoice total (TTC) should be 1210.00

  Scenario: Create invoice draft with reduced VAT 6%
    When the accountant creates an invoice draft with:
      | description          | Travaux isolation toit |
      | amount_excl_vat      | 5000.00                |
      | vat_rate             | 6.00                   |
      | invoice_date         | 2025-01-15             |
    Then the invoice should be created successfully
    And the invoice VAT amount should be 300.00
    And the invoice total (TTC) should be 5300.00

  Scenario: Create invoice with invalid VAT rate fails
    When the accountant creates an invoice draft with:
      | description          | Test                   |
      | amount_excl_vat      | 1000.00                |
      | vat_rate             | 150.00                 |
      | invoice_date         | 2025-01-15             |
    Then the invoice creation should fail
    And the error should contain "VAT rate must be between 0 and 100"

  # ========== Invoice Modification ==========

  Scenario: Update draft invoice
    Given a draft invoice exists
    When the accountant updates the invoice with:
      | amount_excl_vat      | 1500.00                |
      | vat_rate             | 21.00                  |
    Then the invoice should be updated successfully
    And the invoice VAT amount should be 315.00
    And the invoice total (TTC) should be 1815.00

  Scenario: Cannot update approved invoice
    Given an approved invoice exists
    When the accountant tries to update the invoice
    Then the update should fail
    And the error should contain "cannot be modified"

  Scenario: Can update rejected invoice
    Given a rejected invoice exists
    When the accountant updates the invoice with:
      | description          | Réparation corrigée    |
    Then the invoice should be updated successfully
    And the invoice status should be "rejected"

  # ========== Workflow: Submit for Approval ==========

  Scenario: Accountant submits draft invoice for approval
    Given a draft invoice exists
    When the accountant submits the invoice for approval
    Then the invoice status should be "pending_approval"
    And the submitted_at timestamp should be set

  Scenario: Cannot submit already pending invoice
    Given a pending invoice exists
    When the accountant tries to submit the invoice again
    Then the submission should fail
    And the error should contain "already pending approval"

  Scenario: Can resubmit rejected invoice
    Given a rejected invoice exists
    When the accountant submits the invoice for approval
    Then the invoice status should be "pending_approval"
    And the rejection_reason should be cleared

  # ========== Workflow: Syndic Approval ==========

  Scenario: Syndic approves pending invoice
    Given a pending invoice exists
    When the syndic approves the invoice
    Then the invoice status should be "approved"
    And the approved_by field should be set to syndic user
    And the approved_at timestamp should be set

  Scenario: Cannot approve draft invoice
    Given a draft invoice exists
    When the syndic tries to approve the invoice
    Then the approval should fail
    And the error should contain "must be submitted first"

  Scenario: Syndic rejects invoice with reason
    Given a pending invoice exists
    When the syndic rejects the invoice with reason "Montant incorrect - devis différent"
    Then the invoice status should be "rejected"
    And the rejected_by field should be set to syndic user
    And the rejection_reason should be "Montant incorrect - devis différent"

  Scenario: Cannot reject without reason
    Given a pending invoice exists
    When the syndic tries to reject the invoice with empty reason
    Then the rejection should fail
    And the error should contain "Rejection reason cannot be empty"

  # ========== Permissions & Role-Based Access ==========

  Scenario: Owner cannot create invoices
    Given an owner user "owner@test.be"
    When the owner tries to create an invoice draft
    Then the creation should fail with forbidden error
    And the error should contain "Only accountant"

  Scenario: Accountant cannot approve invoices
    Given a pending invoice exists
    When the accountant tries to approve the invoice
    Then the approval should fail with forbidden error
    And the error should contain "Only syndic"

  Scenario: Owner can view invoices (read-only)
    Given an approved invoice exists
    When the owner retrieves the invoice
    Then the invoice should be returned successfully

  # ========== Pending Invoices Dashboard ==========

  Scenario: Syndic views pending invoices dashboard
    Given 3 pending invoices exist
    And 2 approved invoices exist
    When the syndic requests the pending invoices list
    Then 3 invoices should be returned
    And all invoices should have status "pending_approval"

  Scenario: Accountant cannot view pending dashboard
    Given 3 pending invoices exist
    When the accountant tries to view pending invoices
    Then the request should fail with forbidden error

  # ========== Charge Distribution ==========

  Scenario: Calculate charge distribution after approval
    Given a pending invoice with total 1210.00 EUR
    And 5 unit-owner relationships with percentages:
      | unit | owner    | percentage |
      | A1   | Owner 1  | 0.25       |
      | A2   | Owner 2  | 0.25       |
      | A3   | Owner 3  | 0.20       |
      | A4   | Owner 4  | 0.20       |
      | A5   | Owner 5  | 0.10       |
    When the syndic approves the invoice
    And the charge distribution is calculated
    Then 5 charge distributions should be created
    And Owner 1 amount due should be 302.50 EUR
    And Owner 2 amount due should be 302.50 EUR
    And Owner 3 amount due should be 242.00 EUR
    And Owner 4 amount due should be 242.00 EUR
    And Owner 5 amount due should be 121.00 EUR
    And the total distributed should be 1210.00 EUR

  Scenario: Cannot calculate distribution for non-approved invoice
    Given a draft invoice exists
    When trying to calculate charge distribution
    Then the calculation should fail
    And the error should contain "non-approved invoice"

  Scenario: Get all distributions for an owner
    Given 3 approved invoices with distributions exist
    When requesting distributions for Owner 1
    Then 3 distributions should be returned
    And each distribution should have an amount_due

  Scenario: Get total amount due for owner
    Given 3 approved invoices with distributions for Owner 1:
      | invoice  | amount_due |
      | INV-001  | 302.50     |
      | INV-002  | 500.00     |
      | INV-003  | 150.00     |
    When requesting total due for Owner 1
    Then the total amount due should be 952.50 EUR

  # ========== Complete Workflow Cycle ==========

  Scenario: Complete invoice lifecycle - Draft to Paid
    # Step 1: Create draft
    Given the accountant creates an invoice draft with 1000 EUR HT and 21% VAT
    Then the invoice status should be "draft"

    # Step 2: Submit for approval
    When the accountant submits the invoice for approval
    Then the invoice status should be "pending_approval"

    # Step 3: Syndic approves
    When the syndic approves the invoice
    Then the invoice status should be "approved"

    # Step 4: Calculate distribution
    When the charge distribution is calculated
    Then 5 charge distributions should be created

    # Step 5: Mark as paid
    When the accountant marks the invoice as paid
    Then the invoice payment_status should be "paid"
    And the paid_date should be set

  Scenario: Complete workflow with rejection and resubmission
    # Step 1: Create and submit
    Given the accountant creates and submits an invoice
    Then the invoice status should be "pending_approval"

    # Step 2: Syndic rejects
    When the syndic rejects the invoice with reason "Montant trop élevé"
    Then the invoice status should be "rejected"

    # Step 3: Accountant corrects
    When the accountant updates the rejected invoice
    And the accountant submits the invoice for approval
    Then the invoice status should be "pending_approval"
    And the rejection_reason should be cleared

    # Step 4: Syndic approves
    When the syndic approves the invoice
    Then the invoice status should be "approved"
