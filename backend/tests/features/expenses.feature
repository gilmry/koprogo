# Feature: Expenses & Invoice Workflow (Issue #73)
# Belgian TVA rates: 6% (renovations), 12% (intermediate), 21% (standard)
# Approval workflow: Draft -> PendingApproval -> Approved | Rejected

Feature: Expenses and Invoice Workflow
  As a syndic or accountant
  I want to manage building expenses with a full approval workflow
  So that all invoices are properly approved before payment

  Background:
    Given the system is initialized
    And an organization "Expenses Copro ASBL" exists with id "org-expenses"
    And a building "Residence Charges" exists in organization "org-expenses"
    And a user "Marie Comptable" exists with role "accountant"
    And a user "Jean Syndic" exists with role "syndic"

  # === CREATION ===

  Scenario: Create a simple expense
    When I create an expense:
      | description  | Nettoyage parties communes Mars 2026 |
      | amount       | 450.00                               |
      | category     | Maintenance                          |
    Then the expense should be created with status "Draft"
    And the amount should be 450.00

  Scenario: Create expense with Belgian TVA 21%
    When I create an expense with TVA:
      | description      | Réparation ascenseur          |
      | amount_excl_vat  | 1000.00                       |
      | vat_rate         | 21                            |
    Then the expense should be created
    And the amount_incl_vat should be 1210.00

  Scenario: Create expense with reduced TVA 6% (renovation)
    When I create an expense with TVA:
      | description      | Travaux de rénovation façade  |
      | amount_excl_vat  | 5000.00                       |
      | vat_rate         | 6                             |
    Then the expense should be created
    And the amount_incl_vat should be 5300.00

  Scenario: Expense creation fails with zero amount
    When I try to create an expense with amount 0
    Then the creation should fail

  Scenario: Expense creation fails with negative amount
    When I try to create an expense with amount -100
    Then the creation should fail

  # === APPROVAL WORKFLOW ===

  Scenario: Submit expense for approval
    Given a draft expense exists
    When I submit the expense for approval
    Then the expense status should be "PendingApproval"

  Scenario: Approve a pending expense
    Given a pending approval expense exists
    When the syndic approves the expense
    Then the expense status should be "Approved"

  Scenario: Reject a pending expense
    Given a pending approval expense exists
    When the syndic rejects the expense with reason "Missing invoice documentation"
    Then the expense status should be "Rejected"

  Scenario: Cannot modify an approved expense
    Given an approved expense exists
    When I try to update the expense amount to 9999.00
    Then the update should fail
    And the error should mention "approved" or "cannot be modified"

  Scenario: Cannot submit an already approved expense
    Given an approved expense exists
    When I try to submit it for approval again
    Then the submission should fail

  # === MARK AS PAID ===

  Scenario: Mark an approved expense as paid
    Given an approved expense exists
    When I mark the expense as paid
    Then the expense status should be "Paid"
    And the paid_at date should be set

  # === FILTERING & LISTING ===

  Scenario: List expenses for a building
    Given 3 expenses exist for the building
    When I list expenses for the building
    Then I should get 3 expenses

  Scenario: List pending approval expenses
    Given 2 expenses with status "PendingApproval" exist
    And 1 expense with status "Approved" exists
    When I list expenses by status "PendingApproval"
    Then I should get 2 expenses
    And all expenses should have status "PendingApproval"

  # === INVOICE LINE ITEMS (Multi-ligne) ===

  Scenario: Create expense with multiple invoice line items
    When I create an expense with invoice lines:
      | description              | quantity | unit_price | vat_rate |
      | Main d'oeuvre plombier   | 2        | 75.00      | 21       |
      | Pièces de rechange       | 3        | 45.00      | 21       |
    Then the expense should be created
    And the total amount should reflect all line items

  # === DELETION ===

  Scenario: Delete a draft expense
    Given a draft expense exists
    When I delete the expense
    Then the expense should be deleted

  Scenario: Cannot delete an approved expense
    Given an approved expense exists
    When I try to delete the approved expense
    Then the deletion should fail
