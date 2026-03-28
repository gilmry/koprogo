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

  # =================================================================
  # WORKFLOW COMPLET — Personas Residence du Parc Royal
  # Facture toiture 45.000 EUR HTVA, TVA 21%, impact financier asymetrique
  # Spec: docs/specs/06-approbation-facture.rst
  # Art. 3.86 §3 (charges communes), Art. 3.89 §5 (obligations syndic)
  # =================================================================

  Scenario: Workflow complet — Facture toiture 45.000 EUR HTVA avec impact financier
    # Gisele (comptable) saisit la facture de Toitures Bruxelles (Hassan)
    Given the building "Residence du Parc Royal" with 182 lots and 10000 tantiemes
    And Gisele Vandenberghe is comptable of the organization
    And Francois Leroy is syndic of the building
    And Alice Dubois is presidente of the CdC
    And Diane Peeters is membre of the CdC (avocate)
    And the PCMN account "611001" (Travaux toiture) exists

    # Etape 1: Gisele cree la facture (Draft)
    When Gisele creates an expense:
      | description      | Refection toiture batiment principal — Toitures Bruxelles (Hassan El Amrani) |
      | amount_excl_vat  | 45000.00                                                                      |
      | vat_rate         | 21                                                                            |
      | category         | Works                                                                         |
      | supplier         | Toitures Bruxelles (Hassan El Amrani)                                         |
      | invoice_number   | TB-2026-087                                                                   |
      | account_code     | 611001                                                                        |
    Then the expense should be created with status "Draft"
    And the vat_amount should be 9450.00
    And the amount_incl_vat should be 54450.00

    # Etape 2: Francois soumet pour approbation au CdC
    When Francois submits the expense for approval
    Then the expense status should be "PendingApproval"
    And submitted_at should be set

    # Etape 3: Alice et Diane (CdC) approuvent sous condition d'echelonnement
    When Alice approves the expense on behalf of the CdC
    Then the expense status should be "Approved"
    And approved_by should be Alice
    And approved_at should be set

    # Etape 4: Gisele distribue les charges aux coproprietaires
    When Gisele calculates the charge distribution for the expense
    Then charge distributions should be created according to tantiemes:
      | owner                   | tantiemes | percentage | amount_due |
      | Philippe Vandermeulen   | 1800      | 18.0%      | 9801.00    |
      | Emmanuel Claes          | 1280      | 12.8%      | 6969.60    |
      | Charlie Martin          | 660       | 6.6%       | 3593.70    |
      | Diane Peeters           | 580       | 5.8%       | 3158.10    |
      | Alice Dubois            | 450       | 4.5%       | 2450.25    |
      | Marcel Dupont           | 450       | 4.5%       | 2450.25    |
      | Bob Janssen             | 430       | 4.3%       | 2341.35    |
      | Marguerite Lemaire      | 380       | 3.8%       | 2069.10    |
      | Nadia Benali            | 320       | 3.2%       | 1742.40    |
      | Jeanne Devos            | 290       | 2.9%       | 1579.05    |

    # Etape 5: Francois enregistre le paiement
    When Francois marks the expense as paid
    Then the expense status should be "Paid"
    And the paid_at date should be set

  Scenario: Impact financier asymetrique — personas vulnerables
    # Art. 3.86 §3: charges communes proportionnelles aux quotes-parts
    Given an approved expense of 54450.00 EUR TTC for the Residence du Parc Royal
    When the charges are distributed according to tantiemes
    Then Nadia Benali (320 tantiemes, 3.2%) owes 1742.40 EUR
    And Jeanne Devos (290 tantiemes, 2.9%) owes 1579.05 EUR
    And Marguerite Lemaire (380 tantiemes, 3.8%) owes 2069.10 EUR
    # Pour Jeanne: 1579 EUR = 1.5 mois de pension minimum (1050 EUR/mois)
    # Pour Marguerite: 2069 EUR = 1.7 mois de pension de survie (1200 EUR/mois)
    # Un echelonnement sur 6 mois est vital pour elles

  Scenario: Rejet par Diane — non-conformite juridique (2 devis au lieu de 3)
    Given a Works expense in status "PendingApproval"
    And only 2 quotes have been presented (Belgian law requires 3 for > 5000 EUR)
    When Diane rejects with reason "Seulement 2 devis presentes. La loi exige 3 devis comparatifs pour des travaux superieurs a 5.000 EUR."
    Then the expense status should be "Rejected"
    And rejection_reason should be "Seulement 2 devis presentes. La loi exige 3 devis comparatifs pour des travaux superieurs a 5.000 EUR."

    # Francois obtient un 3e devis et Gisele re-soumet
    When Gisele resubmits the expense for approval
    Then the expense status should be "PendingApproval"

  Scenario: Rejet sans motif interdit
    Given an expense in status "PendingApproval"
    When Francois tries to reject without a reason
    Then an error "Rejection reason cannot be empty" is returned

  Scenario: Paiement impossible sans approbation prealable
    Given an expense in status "Draft"
    When Francois tries to mark the expense as paid
    Then an error "invoice must be approved first" is returned

  Scenario: TVA belge a 6% — renovation immeuble > 10 ans (Residence 1965)
    When Gisele creates an expense with TVA:
      | description      | Ravalement facade nord                |
      | amount_excl_vat  | 5000.00                               |
      | vat_rate         | 6                                     |
    Then the vat_amount should be 300.00
    And the amount_incl_vat should be 5300.00
    # Le taux de 6% s'applique car l'immeuble date de 1965 (> 10 ans)

  Scenario: Facture travaux sans rapport prestataire bloquee
    Given a Works expense in status "PendingApproval"
    And no contractor_report_id is linked
    When Alice tries to approve the expense
    Then an error "Work expenses require a validated contractor report" is returned
